//! Daemon implementation that runs as root and executes commands.

use crate::protocol::{ClientMessage, DaemonMessage};
use crate::shared::{get_socket_path, is_process_running};
use anyhow::{Context, Result};
use log::{info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command;

/// Run the authentication daemon.
///
/// # Arguments
///
/// * `effective_uid` - Optional user ID of the original user (when running via pkexec).
///   If provided, the socket will be created in that user's runtime directory.
/// * `parent_pid` - Optional parent process ID to monitor. If provided, the daemon will
///   shut down if the parent process is no longer running.
pub async fn run_daemon(effective_uid: Option<u32>, parent_pid: Option<u32>) -> Result<()> {
    let uid = unsafe { libc::getuid() };
    if uid != 0 {
        anyhow::bail!("Daemon must run as root");
    }

    let socket_path = get_socket_path(effective_uid)?;

    if socket_path.exists() {
        std::fs::remove_file(&socket_path).context("Failed to remove old socket")?;
    }

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    info!("Starting xero-authd daemon");
    info!("Socket path: {:?}", socket_path);

    let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;

    if let Some(uid) = effective_uid {
        use std::ffi::CString;
        use std::os::unix::fs::PermissionsExt;

        let socket_path_cstr = CString::new(socket_path.to_string_lossy().as_ref())
            .context("Failed to convert socket path to CString")?;

        unsafe {
            let passwd = libc::getpwuid(uid as libc::uid_t);
            if !passwd.is_null() {
                let gid = (*passwd).pw_gid;
                let result = libc::chown(socket_path_cstr.as_ptr(), 0, gid);
                if result == 0 {
                    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o660))
                        .context("Failed to set socket permissions")?;
                } else {
                    warn!(
                        "Failed to chown socket (errno: {}), using 0666 permissions",
                        std::io::Error::last_os_error()
                    );
                    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o666))
                        .context("Failed to set socket permissions")?;
                }
            } else {
                warn!(
                    "Failed to get user info for UID {}, using 0666 permissions",
                    uid
                );
                std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o666))
                    .context("Failed to set socket permissions")?;
            }
        }
    } else {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
            .context("Failed to set socket permissions")?;
    }

    info!("Daemon listening on {:?}", socket_path);
    if let Some(pid) = parent_pid {
        info!("Monitoring parent process PID: {}", pid);
    }

    let shutdown = Arc::new(AtomicBool::new(false));

    if let Some(pid) = parent_pid {
        let shutdown_clone = shutdown.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
            loop {
                interval.tick().await;
                if !is_process_running(pid) {
                    warn!(
                        "Parent process {} is no longer running, shutting down daemon",
                        pid
                    );
                    shutdown_clone.store(true, Ordering::SeqCst);
                    break;
                }
            }
        });
    }

    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("Failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down");
            }
        }
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        let accept_future = listener.accept();
        let timeout_future = tokio::time::sleep(tokio::time::Duration::from_millis(100));

        tokio::select! {
            result = accept_future => {
                match result {
                    Ok((stream, _addr)) => {
                        info!("New client connection");
                        let shutdown_clone = shutdown.clone();
                        let parent_pid_clone = parent_pid;
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, shutdown_clone, parent_pid_clone).await {
                                log::error!("Error handling client: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        if shutdown.load(Ordering::SeqCst) {
                            break;
                        }
                        log::error!("Failed to accept connection: {}", e);
                    }
                }
            }
            _ = timeout_future => {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
                continue;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down");
                shutdown.store(true, Ordering::SeqCst);
                break;
            }
        }
    }

    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    Ok(())
}

/// Handle a client connection.
async fn handle_client(
    mut stream: UnixStream,
    shutdown: Arc<AtomicBool>,
    parent_pid: Option<u32>,
) -> Result<()> {
    let (mut reader, writer) = stream.split();
    let mut buf_reader = BufReader::new(&mut reader);
    let mut line = String::new();

    use std::sync::Arc;
    use tokio::sync::Mutex;
    let writer_arc = Arc::new(Mutex::new(writer));

    loop {
        line.clear();
        let bytes_read = buf_reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            break;
        }

        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        if let Some(pid) = parent_pid {
            if !is_process_running(pid) {
                warn!(
                    "Parent process {} is no longer running, rejecting command",
                    pid
                );
                let error_msg =
                    DaemonMessage::ErrorMessage("Parent process is no longer running".to_string());
                let response = serde_json::to_string(&error_msg)? + "\n";
                let mut w = writer_arc.lock().await;
                w.write_all(response.as_bytes()).await?;
                shutdown.store(true, Ordering::SeqCst);
                break;
            }
        }

        let message: ClientMessage = match serde_json::from_str(line.trim()) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to parse message: {}", e);
                let error_msg =
                    DaemonMessage::ErrorMessage(format!("Failed to parse message: {}", e));
                let response = serde_json::to_string(&error_msg)? + "\n";
                let mut w = writer_arc.lock().await;
                w.write_all(response.as_bytes()).await?;
                continue;
            }
        };

        match message {
            ClientMessage::Ping => {
                let response = serde_json::to_string(&DaemonMessage::Pong)? + "\n";
                let mut w = writer_arc.lock().await;
                w.write_all(response.as_bytes()).await?;
            }
            ClientMessage::Shutdown => {
                info!("Received shutdown request from client");
                let response = serde_json::to_string(&DaemonMessage::ShutdownAck)? + "\n";
                let mut w = writer_arc.lock().await;
                w.write_all(response.as_bytes()).await?;
                shutdown.store(true, Ordering::SeqCst);
                break;
            }
            ClientMessage::Execute {
                program,
                args,
                working_dir,
            } => {
                info!("Executing: {} {:?}", program, args);

                let mut cmd = Command::new(&program);
                cmd.args(&args);

                if let Some(dir) = &working_dir {
                    cmd.current_dir(dir);
                }

                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());

                match cmd.spawn() {
                    Ok(mut child) => {
                        let status = child.wait().await;
                        let writer_exec = writer_arc.clone();

                        if let Some(stdout) = child.stdout.take() {
                            let mut reader = BufReader::new(stdout);
                            let mut line = String::new();
                            loop {
                                line.clear();
                                match reader.read_line(&mut line).await {
                                    Ok(0) => break,
                                    Ok(_) => {
                                        let msg = DaemonMessage::Output(line.clone());
                                        let response = serde_json::to_string(&msg)? + "\n";
                                        let mut w = writer_exec.lock().await;
                                        let _ = w.write_all(response.as_bytes()).await;
                                    }
                                    Err(_) => break,
                                }
                            }
                        }

                        if let Some(stderr) = child.stderr.take() {
                            let mut reader = BufReader::new(stderr);
                            let mut line = String::new();
                            loop {
                                line.clear();
                                match reader.read_line(&mut line).await {
                                    Ok(0) => break,
                                    Ok(_) => {
                                        let msg = DaemonMessage::Error(line.clone());
                                        let response = serde_json::to_string(&msg)? + "\n";
                                        let mut w = writer_exec.lock().await;
                                        let _ = w.write_all(response.as_bytes()).await;
                                    }
                                    Err(_) => break,
                                }
                            }
                        }

                        match status {
                            Ok(status) => {
                                let exit_code = status.code().unwrap_or(-1);
                                let response =
                                    serde_json::to_string(&DaemonMessage::Completed { exit_code })?
                                        + "\n";
                                let mut w = writer_exec.lock().await;
                                w.write_all(response.as_bytes()).await?;
                            }
                            Err(e) => {
                                let error_msg = DaemonMessage::ErrorMessage(format!(
                                    "Failed to wait for process: {}",
                                    e
                                ));
                                let response = serde_json::to_string(&error_msg)? + "\n";
                                let mut w = writer_exec.lock().await;
                                w.write_all(response.as_bytes()).await?;
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg =
                            DaemonMessage::ErrorMessage(format!("Failed to spawn process: {}", e));
                        let response = serde_json::to_string(&error_msg)? + "\n";
                        let mut w = writer_arc.lock().await;
                        w.write_all(response.as_bytes()).await?;
                    }
                }
            }
        }
    }

    Ok(())
}
