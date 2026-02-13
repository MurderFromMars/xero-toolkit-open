//! Application constants.

/// Application information constants.
pub mod app_info {
    pub const NAME: &str = "xero-toolkit";
    pub const ID: &str = "xyz.xerolinux.xero-toolkit";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// Sidebar configuration.
pub mod sidebar {
    pub const MIN_WIDTH: i32 = 200;
    pub const MAX_WIDTH: i32 = 400;
}

/// External links.
pub mod links {
    pub const DISCORD: &str = "https://discord.xerolinux.xyz/";
    pub const YOUTUBE: &str = "https://www.youtube.com/@XeroLinux";
    pub const WEBSITE: &str = "https://xerolinux.xyz/";
    pub const DONATE: &str = "https://ko-fi.com/xerolinux";
}

/// Binary paths for system executables.
pub mod paths {
    use std::path::PathBuf;

    /// Path to the sources directory (contains scripts and systemd).
    #[allow(dead_code)]
    pub const SOURCES: &str = "/opt/xero-toolkit/sources";

    pub const DAEMON: &str = "/opt/xero-toolkit/xero-authd";
    pub const CLIENT: &str = "/opt/xero-toolkit/xero-auth";
    pub const SCRIPTS: &str = "/opt/xero-toolkit/sources/scripts";
    pub const SYSTEMD: &str = "/opt/xero-toolkit/sources/systemd";
    pub const DESKTOP_FILE: &str = "/usr/share/applications/xero-toolkit.desktop";
    pub const SYSTEM_AUTOSTART: &str = "/etc/xdg/autostart/xero-toolkit.desktop";

    pub fn daemon() -> PathBuf {
        PathBuf::from(DAEMON)
    }

    pub fn client() -> PathBuf {
        PathBuf::from(CLIENT)
    }

    #[allow(dead_code)]
    pub fn sources() -> PathBuf {
        PathBuf::from(SOURCES)
    }

    pub fn scripts() -> PathBuf {
        PathBuf::from(SCRIPTS)
    }

    pub fn systemd() -> PathBuf {
        PathBuf::from(SYSTEMD)
    }

    pub fn desktop_file() -> PathBuf {
        PathBuf::from(DESKTOP_FILE)
    }

    pub fn system_autostart() -> PathBuf {
        PathBuf::from(SYSTEM_AUTOSTART)
    }
}

/// Debug environment variables for seasonal effects.
pub mod seasonal_debug {
    pub const ENABLE_SNOW: &str = "XERO_TOOLKIT_ENABLE_SNOW";
    pub const ENABLE_HALLOWEEN: &str = "XERO_TOOLKIT_ENABLE_HALLOWEEN";

    pub fn check_effect_env(var_name: &str) -> Option<bool> {
        std::env::var(var_name).ok().and_then(|value| {
            if let Ok(enabled) = value.parse::<bool>() {
                return Some(enabled);
            }

            let lower = value.to_lowercase();
            match lower.as_str() {
                "1" | "true" | "yes" => Some(true),
                "0" | "false" | "no" => Some(false),
                _ => None,
            }
        })
    }
}

/// UI resource paths for GResource files.
pub mod resources {
    pub const MAIN_UI: &str = "/xyz/xerolinux/xero-toolkit/ui/main.ui";
    pub const ICONS: &str = "/xyz/xerolinux/xero-toolkit/icons";
    pub const CSS: &str = "/xyz/xerolinux/xero-toolkit/css/style.css";

    pub mod dialogs {
        pub const ABOUT: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/about_dialog.ui";
        pub const DEPENDENCY_ERROR: &str =
            "/xyz/xerolinux/xero-toolkit/ui/dialogs/dependency_error_dialog.ui";
        pub const DOWNLOAD: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/download_dialog.ui";
        pub const DOWNLOAD_SETUP: &str =
            "/xyz/xerolinux/xero-toolkit/ui/dialogs/download_setup_dialog.ui";
        pub const SCHEDULER_SELECTION: &str =
            "/xyz/xerolinux/xero-toolkit/ui/dialogs/scheduler_selection_dialog.ui";
        pub const SELECTION: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/selection_dialog.ui";
        pub const TASK_LIST: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/task_list_dialog.ui";
        pub const TERMINAL: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/terminal_dialog.ui";
        pub const WARNING: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/warning_dialog.ui";
        pub const XEROLINUX_CHECK: &str =
            "/xyz/xerolinux/xero-toolkit/ui/dialogs/xerolinux_check_dialog.ui";
    }

    pub mod tabs {
        pub const BIOMETRICS: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/biometrics.ui";
        pub const CONTAINERS_VMS: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/containers_vms.ui";
        pub const CUSTOMIZATION: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/customization.ui";
        pub const DRIVERS: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/drivers.ui";
        pub const GAMESCOPE: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/gamescope.ui";
        pub const GAMING_TOOLS: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/gaming_tools.ui";
        pub const KERNEL_SCHEDULERS: &str =
            "/xyz/xerolinux/xero-toolkit/ui/tabs/kernel_schedulers.ui";
        pub const MAIN_PAGE: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/main_page.ui";
        pub const SERVICING_SYSTEM_TWEAKS: &str =
            "/xyz/xerolinux/xero-toolkit/ui/tabs/servicing_system_tweaks.ui";
    }
}
