//! Protocol definitions for communication between client and daemon.

use serde::{Deserialize, Serialize};

/// Message sent from client to daemon.
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Execute a command with arguments.
    Execute {
        program: String,
        args: Vec<String>,
        working_dir: Option<String>,
    },
    /// Ping to check if daemon is alive.
    Ping,
    /// Shutdown the daemon.
    Shutdown,
}

/// Message sent from daemon to client.
#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonMessage {
    /// Command output (stdout line).
    Output(String),
    /// Command error output (stderr line).
    Error(String),
    /// Command completed with exit code.
    Completed { exit_code: i32 },
    /// Error occurred.
    ErrorMessage(String),
    /// Pong response to ping.
    Pong,
    /// Shutdown acknowledged.
    ShutdownAck,
}
