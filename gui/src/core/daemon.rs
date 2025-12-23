//! Daemon management for xero-auth.

use crate::config;
use anyhow::{Context, Result};
use log::{info, warn};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use xero_auth::shared::is_daemon_running;

/// Get the path to the xero-authd daemon binary.
fn get_daemon_path() -> PathBuf {
    config::paths::daemon()
}

/// Get the path to the xero-auth client binary.
pub fn get_xero_auth_path() -> PathBuf {
    config::paths::client()
}

/// Start the daemon.
/// Returns Ok(()) if daemon is already running or started successfully.
pub fn start_daemon() -> Result<()> {
    if is_daemon_running() {
        info!("Daemon is already running");
        return Ok(());
    }

    let daemon_path = get_daemon_path();
    let current_uid = unsafe { libc::getuid() };
    let current_pid = std::process::id();
    info!("Starting daemon via pkexec: {}", daemon_path.display());

    let mut child = Command::new("pkexec")
        .arg(daemon_path.as_os_str())
        .arg("--uid")
        .arg(current_uid.to_string())
        .arg("--parent-pid")
        .arg(current_pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn pkexec")?;

    let socket_path = xero_auth::shared::get_socket_path(None)?;
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(60);
    let poll_interval = Duration::from_millis(50);

    loop {
        if socket_path.exists() {
            info!("Daemon started successfully");
            return Ok(());
        }

        // Check if pkexec has exited (including zombie state)
        if let Ok(Some(_status)) = child.try_wait() {
            anyhow::bail!("pkexec process has exited (may have been cancelled)");
        }

        if start.elapsed() >= timeout {
            anyhow::bail!(
                "Daemon socket not found after starting within {:?} at {:?}",
                timeout,
                socket_path
            );
        }

        std::thread::sleep(poll_interval);
    }
}

pub async fn stop_daemon() -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;
    use xero_auth::protocol::ClientMessage;
    use xero_auth::shared::get_socket_path;

    if is_daemon_running() {
        if let Ok(socket_path) = get_socket_path(None) {
            if let Ok(mut stream) = UnixStream::connect(&socket_path).await {
                let (mut reader, mut writer) = stream.split();
                let mut buf_reader = BufReader::new(&mut reader);

                let message = ClientMessage::Shutdown;
                let request = serde_json::to_string(&message)? + "\n";
                writer.write_all(request.as_bytes()).await?;

                let mut line = String::new();
                if let Err(e) = buf_reader.read_line(&mut line).await {
                    warn!("Failed to read shutdown acknowledgment from daemon: {}", e);
                }
            }
        }
    }

    info!("Daemon process terminated");

    Ok(())
}
