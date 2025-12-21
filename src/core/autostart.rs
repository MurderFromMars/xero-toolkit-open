//! Autostart management for the toolkit.
//!
//! Handles enabling/disabling autostart by managing the desktop file
//! in the user's autostart directory.

use std::fs;
use std::path::PathBuf;

/// Get the autostart desktop file path
pub fn get_autostart_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    config_dir.join("autostart").join("xero-toolkit.desktop")
}

/// Check if autostart is enabled
pub fn is_enabled() -> bool {
    get_autostart_path().exists()
}

/// Enable autostart by copying the desktop file to autostart directory
pub fn enable() -> Result<(), std::io::Error> {
    let autostart_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("autostart");

    // Create autostart directory if it doesn't exist
    fs::create_dir_all(&autostart_dir)?;

    // Try to find the desktop file from common installation paths
    let desktop_sources = [
        PathBuf::from("/usr/share/applications/xero-toolkit.desktop"),
        PathBuf::from("/usr/local/share/applications/xero-toolkit.desktop"),
    ];

    for source in &desktop_sources {
        if source.exists() {
            return fs::copy(source, get_autostart_path()).map(|_| ());
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Desktop file not found in system applications",
    ))
}

/// Disable autostart by removing the desktop file
pub fn disable() -> Result<(), std::io::Error> {
    let path = get_autostart_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
