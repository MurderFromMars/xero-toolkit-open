//! System dependency checks and validation.

use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button, Label};
use log::{error, info, warn};

/// Result of dependency check containing missing dependencies.
#[derive(Debug, Clone)]
pub struct DependencyCheckResult {
    pub flatpak_missing: bool,
    pub aur_helper_missing: bool,
}

impl DependencyCheckResult {
    /// Check if any dependencies are missing.
    pub fn has_missing_dependencies(&self) -> bool {
        self.flatpak_missing || self.aur_helper_missing
    }

    /// Get list of missing dependency names.
    pub fn missing_dependencies(&self) -> Vec<&str> {
        let mut missing = Vec::new();
        if self.flatpak_missing {
            missing.push("flatpak");
        }
        if self.aur_helper_missing {
            missing.push("paru or yay");
        }
        missing
    }

    /// Generate formatted list of missing dependencies for display.
    pub fn format_missing_list(&self) -> String {
        self.missing_dependencies()
            .iter()
            .map(|dep| format!("• <b>{}</b>", dep))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate installation hint based on missing dependencies.
    pub fn generate_install_hint(&self) -> String {
        let mut hints = Vec::new();

        if self.flatpak_missing {
            hints.push("Install flatpak: <tt>sudo pacman -S flatpak</tt>");
        }
        if self.aur_helper_missing {
            hints.push("AUR Helper repositories:\n• Paru: <a href=\"https://github.com/Morganamilo/paru\">https://github.com/Morganamilo/paru</a>\n• Yay: <a href=\"https://github.com/Jguer/yay\">https://github.com/Jguer/yay</a>");
        }

        if hints.is_empty() {
            return String::new();
        }

        hints.join("\n\n")
    }
}

/// Check if flatpak is installed and available.
fn check_flatpak() -> bool {
    info!("Checking for flatpak availability");
    match std::process::Command::new("flatpak")
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("flatpak found: {}", version.trim());
            true
        }
        Ok(_) => {
            warn!("flatpak command exists but returned error");
            false
        }
        Err(_) => {
            warn!("flatpak not found in PATH");
            false
        }
    }
}

/// Check if an AUR helper (paru or yay) is installed.
fn check_aur_helper() -> bool {
    info!("Checking for AUR helper availability");

    if let Ok(output) = std::process::Command::new("paru").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("paru found: {}", version.trim());
            return true;
        }
    }

    if let Ok(output) = std::process::Command::new("yay").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("yay found: {}", version.trim());
            return true;
        }
    }

    warn!("No AUR helper (paru or yay) found in PATH");
    false
}

/// Perform all dependency checks and return results.
pub fn check_dependencies() -> DependencyCheckResult {
    info!("Performing system dependency checks");

    let flatpak_missing = !check_flatpak();
    let aur_helper_missing = !check_aur_helper();

    let result = DependencyCheckResult {
        flatpak_missing,
        aur_helper_missing,
    };

    if result.has_missing_dependencies() {
        let issues = result.missing_dependencies();
        error!("Issues detected: {}", issues.join(", "));
    } else {
        info!("All required dependencies are available");
    }

    result
}

/// Show dependency error dialog and prevent app from continuing.
pub fn show_dependency_error_dialog(
    main_window: &ApplicationWindow,
    check_result: &DependencyCheckResult,
) {
    error!("Showing dependency error dialog");

    // Load error dialog from UI file
    let builder = Builder::from_resource(crate::config::resources::dialogs::DEPENDENCY_ERROR);

    let error_window: gtk4::Window = extract_widget(&builder, "dependency_error_window");

    let missing_deps_label: Label = extract_widget(&builder, "missing_deps_label");

    let install_hint_label: Label = extract_widget(&builder, "install_hint_label");

    let exit_button: Button = extract_widget(&builder, "exit_button");

    missing_deps_label.set_label(&check_result.format_missing_list());

    install_hint_label.set_label(&check_result.generate_install_hint());

    error_window.set_transient_for(Some(main_window));

    let main_window_clone = main_window.clone();
    exit_button.connect_clicked(move |_| {
        error!("User clicked exit on dependency error dialog");
        main_window_clone.close();
        std::process::exit(1);
    });

    error_window.present();
}
