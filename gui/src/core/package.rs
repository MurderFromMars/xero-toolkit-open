//! Package and system utility functions.
//!
//! This module provides utilities for checking installed packages,
//! flatpaks, and system operations.

use super::aur;
use anyhow::Result;
use log::debug;

/// Check if a package is installed using AUR helper or pacman.
pub fn is_package_installed(package: &str) -> bool {
    debug!("Checking if package '{}' is installed", package);

    // Try AUR helper first (prefer initialized, fall back to detection)
    if let Some(helper) = aur::get().or_else(aur::detect) {
        if check_with_helper(helper, package) {
            return true;
        }
    } else {
        debug!("No AUR helper available, skipping AUR check");
    }

    // Fallback to pacman
    check_with_pacman(package)
}

/// Check if a package is installed using a specific helper.
fn check_with_helper(helper: &str, package: &str) -> bool {
    debug!("Using '{}' to check package '{}'", helper, package);
    
    match std::process::Command::new(helper).args(["-Q", package]).output() {
        Ok(output) if output.status.success() => {
            debug!("Package '{}' found via {}", package, helper);
            true
        }
        Ok(_) => {
            debug!("'{}' did not find package '{}'", helper, package);
            false
        }
        Err(e) => {
            debug!("Failed to execute '{}': {}", helper, e);
            false
        }
    }
}

/// Check if a package is installed using pacman.
fn check_with_pacman(package: &str) -> bool {
    let installed = std::process::Command::new("pacman")
        .args(["-Q", package])
        .output()
        .map_or(false, |output| output.status.success());

    if installed {
        debug!("Package '{}' found via pacman", package);
    } else {
        debug!("Package '{}' not installed", package);
    }

    installed
}

/// Check if a flatpak package is installed.
pub fn is_flatpak_installed(package: &str) -> bool {
    debug!("Checking if Flatpak '{}' is installed", package);

    let installed = std::process::Command::new("flatpak")
        .args(["list", "--columns=application"])
        .output()
        .map_or(false, |output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line.trim() == package)
        });

    debug!(
        "Flatpak '{}' {}",
        package,
        if installed { "found" } else { "not installed" }
    );

    installed
}

/// Open a URL in the default browser.
pub fn open_url(url: &str) -> Result<()> {
    debug!("Opening URL: {}", url);
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_package_installed_nonexistent() {
        // A package that definitely doesn't exist
        assert!(!is_package_installed(
            "this-package-definitely-does-not-exist-12345"
        ));
    }
}
