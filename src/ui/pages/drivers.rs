//! Drivers and hardware tools page button handlers.
//!
//! Handles:
//! - Tailscale VPN
//! - ASUS ROG laptop tools

use crate::ui::task_runner::{self, Command};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the drivers page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder) {
    setup_tailscale(page_builder);
    setup_asus_rog(page_builder);
}

fn setup_tailscale(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_tailscale") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Tailscale VPN button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![Command::privileged(
            "bash",
            &[
                "-c",
                "curl -fsSL https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/install.sh | bash",
            ],
            "Installing Tailscale VPN...",
        )];

        task_runner::run(window.upcast_ref(), commands, "Install Tailscale VPN");
    });
}

fn setup_asus_rog(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_asus_rog") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("ASUS ROG Tools button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![
            Command::aur(
                &[
                    "-S",
                    "--noconfirm",
                    "--needed",
                    "rog-control-center",
                    "asusctl",
                    "supergfxctl",
                ],
                "Installing ASUS ROG control tools...",
            ),
            Command::privileged(
                "systemctl",
                &["enable", "--now", "asusd", "supergfxd"],
                "Enabling ASUS ROG services...",
            ),
        ];

        task_runner::run(window.upcast_ref(), commands, "Install ASUS ROG Tools");
    });
}

/// Helper to get the parent window from a button.
fn get_window(button: &Button) -> Option<ApplicationWindow> {
    button
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok())
}
