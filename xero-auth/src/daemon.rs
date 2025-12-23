//! Daemon implementation that runs as root and executes commands.

use crate::protocol::{ClientMessage, DaemonMessage};
use crate::shared::{get_socket_path, is_process_running};
use anyhow::{Context, Result};
use log::{error, info, warn};
use pty::fork::Fork;
use std::ffi::CString;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

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
    set_socket_permissions(&socket_path, effective_uid)?;

    info!("Daemon listening on {:?}", socket_path);
    if let Some(pid) = parent_pid {
        info!("Monitoring parent process PID: {}", pid);
    }

    let shutdown = Arc::new(AtomicBool::new(false));

    if let Some(pid) = parent_pid {
        spawn_parent_monitor(shutdown.clone(), pid);
    }

    spawn_signal_handler(shutdown.clone());

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
                                error!("Error handling client: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        if shutdown.load(Ordering::SeqCst) {
                            break;
                        }
                        error!("Failed to accept connection: {}", e);
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

fn set_socket_permissions(socket_path: &std::path::Path, effective_uid: Option<u32>) -> Result<()> {
    if let Some(uid) = effective_uid {
        let socket_path_cstr = CString::new(socket_path.to_string_lossy().as_ref())
            .context("Failed to convert socket path to CString")?;

        unsafe {
            let passwd = libc::getpwuid(uid as libc::uid_t);
            if !passwd.is_null() {
                let gid = (*passwd).pw_gid;
                let result = libc::chown(socket_path_cstr.as_ptr(), 0, gid);
                if result == 0 {
                    std::fs::set_permissions(socket_path, PermissionsExt::from_mode(0o660))
                        .context("Failed to set socket permissions")?;
                } else {
                    warn!(
                        "Failed to chown socket (errno: {}), using 0666 permissions",
                        std::io::Error::last_os_error()
                    );
                    std::fs::set_permissions(socket_path, PermissionsExt::from_mode(0o666))
                        .context("Failed to set socket permissions")?;
                }
            } else {
                warn!(
                    "Failed to get user info for UID {}, using 0666 permissions",
                    uid
                );
                std::fs::set_permissions(socket_path, PermissionsExt::from_mode(0o666))
                    .context("Failed to set socket permissions")?;
            }
        }
    } else {
        std::fs::set_permissions(socket_path, PermissionsExt::from_mode(0o600))
            .context("Failed to set socket permissions")?;
    }
    Ok(())
}

fn spawn_parent_monitor(shutdown: Arc<AtomicBool>, pid: u32) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if !is_process_running(pid) {
                warn!(
                    "Parent process {} is no longer running, shutting down daemon",
                    pid
                );
                shutdown.store(true, Ordering::SeqCst);
                break;
            }
        }
    });
}

fn spawn_signal_handler(shutdown: Arc<AtomicBool>) {
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
        shutdown.store(true, Ordering::SeqCst);
    });
}

async fn handle_client(
    mut stream: UnixStream,
    shutdown: Arc<AtomicBool>,
    parent_pid: Option<u32>,
) -> Result<()> {
    let (mut reader, writer) = stream.split();
    let mut buf_reader = BufReader::new(&mut reader);
    let mut line = String::new();
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
                send_error(&writer_arc, "Parent process is no longer running").await?;
                shutdown.store(true, Ordering::SeqCst);
                break;
            }
        }

        let message: ClientMessage = match serde_json::from_str(line.trim()) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to parse message: {}", e);
                send_error(&writer_arc, &format!("Failed to parse message: {}", e)).await?;
                continue;
            }
        };

        match message {
            ClientMessage::Ping => {
                send_message(&writer_arc, &DaemonMessage::Pong).await?;
            }
            ClientMessage::Shutdown => {
                info!("Received shutdown request from client");
                send_message(&writer_arc, &DaemonMessage::ShutdownAck).await?;
                shutdown.store(true, Ordering::SeqCst);
                break;
            }
            ClientMessage::Execute {
                program,
                args,
                working_dir,
            } => {
                execute_command(&writer_arc, program, args, working_dir).await?;
            }
        }
    }

    Ok(())
}

async fn send_message(
    writer: &Arc<Mutex<tokio::net::unix::WriteHalf<'_>>>,
    message: &DaemonMessage,
) -> Result<()> {
    let response = serde_json::to_string(message)? + "\n";
    let mut w = writer.lock().await;
    w.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn send_error(
    writer: &Arc<Mutex<tokio::net::unix::WriteHalf<'_>>>,
    error: &str,
) -> Result<()> {
    send_message(writer, &DaemonMessage::ErrorMessage(error.to_string())).await
}

async fn execute_command(
    writer: &Arc<Mutex<tokio::net::unix::WriteHalf<'_>>>,
    program: String,
    args: Vec<String>,
    working_dir: Option<String>,
) -> Result<()> {
    info!("Executing: {} {:?}", program, args);

    let fork = Fork::from_ptmx().map_err(|e| anyhow::anyhow!("Failed to create PTY: {}", e))?;

    match fork {
        Fork::Child(_) => {
            if let Some(dir) = &working_dir {
                if let Err(e) = std::env::set_current_dir(dir) {
                    eprintln!("Failed to change directory: {}", e);
                    std::process::exit(1);
                }
            }

            let error = std::process::Command::new(&program).args(&args).exec();

            eprintln!("Failed to execute {}: {}", program, error);
            std::process::exit(1);
        }
        Fork::Parent(pid, master) => {
            let exit_code = read_pty_output(writer.clone(), master, pid).await?;
            send_message(writer, &DaemonMessage::Completed { exit_code }).await?;
        }
    }

    Ok(())
}

async fn read_pty_output(
    writer: Arc<Mutex<tokio::net::unix::WriteHalf<'_>>>,
    master: pty::prelude::Master,
    pid: libc::pid_t,
) -> Result<i32> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<String, std::io::Error>>();

    let read_handle = tokio::task::spawn_blocking(move || {
        use std::io::{BufRead, BufReader};
        let mut reader = BufReader::new(master);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(Ok(line.clone())).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::Interrupted {
                        let _ = tx.send(Err(e));
                        break;
                    }
                }
            }
        }
    });

    let writer_output = writer.clone();
    let output_task = async move {
        while let Some(result) = rx.recv().await {
            match result {
                Ok(line) => {
                    let msg = DaemonMessage::Output(line);
                    if let Ok(response) = serde_json::to_string(&msg) {
                        let response = response + "\n";
                        let mut w = writer_output.lock().await;
                        let _ = w.write_all(response.as_bytes()).await;
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::UnexpectedEof {
                        warn!("Error reading from PTY: {}", e);
                    }
                    break;
                }
            }
        }
    };

    tokio::select! {
        _ = read_handle => {},
        _ = output_task => {},
    }

    let exit_code = tokio::task::spawn_blocking(move || {
        let mut status: libc::c_int = 0;
        let result = unsafe { libc::waitpid(pid, &mut status, 0) };

        if result == pid {
            if libc::WIFEXITED(status) {
                libc::WEXITSTATUS(status) as i32
            } else if libc::WIFSIGNALED(status) {
                128 + libc::WTERMSIG(status) as i32
            } else {
                -1
            }
        } else {
            warn!("Failed to wait for child process {}", pid);
            -1
        }
    })
    .await
    .unwrap_or(-1);

    Ok(exit_code)
}
