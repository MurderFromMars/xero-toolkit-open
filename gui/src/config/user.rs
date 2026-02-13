//! User-configurable settings stored in ~/.config/xero-toolkit/config.toml

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub warnings: WarningsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GeneralConfig {
    /// Whether to launch xero-toolkit on login
    pub autostart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WarningsConfig {
    /// User dismissed the "limited support on non-XeroLinux" notice
    pub dismissed_generic_distro_notice: bool,
    // Add future "don't show again" flags here, not as loose keys
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("xero-toolkit")
        .join("config.toml")
}

impl Config {
    /// Load config from disk, returning defaults for any missing keys or
    /// if the file does not exist yet.
    pub fn load() -> Self {
        let path = config_path();

        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Self::default();
            }
            Err(e) => {
                eprintln!("Warning: could not read config ({e}), using defaults");
                return Self::default();
            }
        };

        match toml::from_str(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: config parse error ({e}), using defaults");
                Self::default()
            }
        }
    }

    /// Atomically write config to disk.
    /// Writes to a temp file first, then renames — avoids corruption on crash.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path();

        // Ensure parent directory exists
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(ConfigError::Io)?;
        }

        let content = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;

        // Write to a temp file alongside the real one
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, &content).map_err(ConfigError::Io)?;

        // Atomic rename
        std::fs::rename(&tmp_path, &path).map_err(ConfigError::Io)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Serialize(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Serialize(e) => write!(f, "Serialize error: {e}"),
        }
    }
}
