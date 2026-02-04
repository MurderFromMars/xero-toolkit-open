//! AUR helper detection and management.
//!
//! This module handles detection and access to AUR helpers (paru/yay)
//! used for installing packages from the Arch User Repository.

use log::debug;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Global storage for the detected AUR helper.
static AUR_HELPER: OnceLock<String> = OnceLock::new();

/// Priority order for AUR helper detection.
const AUR_HELPERS: [&str; 2] = ["paru", "yay"];

/// Detect and return the available AUR helper.
///
/// Searches for AUR helpers in priority order (paru, then yay).
/// Returns the first found helper or None if none are available.
pub fn detect() -> Option<&'static str> {
    AUR_HELPERS.iter().find(|&&helper| {
        if is_executable_in_path(helper) {
            debug!("Found AUR helper: {}", helper);
            true
        } else {
            false
        }
    }).copied()
        .or_else(|| {
            debug!("No AUR helper found");
            None
        })
}

/// Initialize the global AUR helper.
///
/// Should be called once at startup after dependency checks pass.
/// Returns true if an AUR helper was found and initialized.
pub fn init() -> bool {
    if let Some(helper) = detect() {
        let _ = AUR_HELPER.set(helper.to_string());
        true
    } else {
        false
    }
}

/// Get the initialized AUR helper.
///
/// Returns None if no helper has been initialized.
pub fn get() -> Option<&'static str> {
    AUR_HELPER.get().map(String::as_str)
}

/// Check if a command is executable in PATH.
fn is_executable_in_path(cmd: &str) -> bool {
    if cmd.contains(std::path::MAIN_SEPARATOR) {
        return PathBuf::from(cmd).is_file();
    }

    let paths = match env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };

    for dir in env::split_paths(&paths) {
        let candidate = dir.join(cmd);
        if let Ok(metadata) = std::fs::metadata(&candidate) {
            if metadata.permissions().mode() & 0o111 != 0 {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_returns_valid_helper() {
        // This test just verifies the function doesn't panic
        let _ = detect();
    }
}
