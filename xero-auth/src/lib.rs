//! Xero Authentication Daemon
//!
//! Provides a daemon-based privilege escalation system that maintains
//! an authenticated session to avoid repeated password prompts.

pub mod daemon;
pub mod protocol;
pub mod shared;

pub use daemon::run_daemon;
pub use shared::{get_socket_path, is_daemon_running, wait_for_socket};
