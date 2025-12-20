//! Main page button handlers.
//!
//! Handles:
//! - System update
//! - Package manager GUI installation
//! - Download Arch ISO
//! - External links (Discord, YouTube, Website, Donate)

use crate::config;
use crate::core;
use crate::ui::dialogs::download::show_download_dialog;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption,
};
use crate::ui::task_runner::{self, Command};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the main page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder) {
    setup_update_system(page_builder);
    setup_pkg_manager(page_builder);
    setup_download_arch_iso(page_builder);
    setup_external_links(page_builder);
}

/// Setup system update button.
fn setup_update_system(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_update_system") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Update System button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![Command::privileged(
            "/usr/local/bin/upd",
            &[],
            "Updating system packages...",
        )];

        task_runner::run(window.upcast_ref(), commands, "System Update");
    });
}

/// Setup package manager GUI button.
fn setup_pkg_manager(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_pkg_manager") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("PKG Manager GUI button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let window_clone = window.clone();

        // Check which package managers are already installed
        let config = SelectionDialogConfig::new(
            "Package Manager GUI Applications",
            "Select which package manager GUIs to install. Multiple selections allowed.",
        )
        .add_option(SelectionOption::new(
            "octopi",
            "Octopi",
            "Powerful Pacman GUI with AUR support",
            core::is_package_installed("octopi"),
        ))
        .add_option(SelectionOption::new(
            "pacseek",
            "PacSeek",
            "Terminal UI package manager with search",
            core::is_package_installed("pacseek"),
        ))
        .add_option(SelectionOption::new(
            "bauh",
            "Bauh",
            "Manage Pacman, AUR, Flatpak, Snap packages",
            core::is_package_installed("bauh"),
        ))
        .add_option(SelectionOption::new(
            "warehouse",
            "Warehouse",
            "Flatpak package manager (Flatpak)",
            core::is_flatpak_installed("io.github.flattool.Warehouse"),
        ))
        .add_option(SelectionOption::new(
            "flatseal",
            "Flatseal",
            "Flatpak permissions manager (Flatpak)",
            core::is_flatpak_installed("com.github.tchx84.Flatseal"),
        ))
        .add_option(SelectionOption::new(
            "bazaar",
            "Bazaar",
            "Browse and install Flatpak apps (Flatpak)",
            core::is_flatpak_installed("io.github.kolunmi.Bazaar"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            let commands = build_pkg_manager_commands(&selected);

            if !commands.is_empty() {
                task_runner::run(window_clone.upcast_ref(), commands, "Package Manager GUI Installation");
             }
         });
     });
}

/// Build commands for selected package managers.
fn build_pkg_manager_commands(selected: &[String]) -> Vec<Command> {
    let mut commands = Vec::new();

    if selected.contains(&"octopi".to_string()) {
        commands.push(Command::aur(
            &["-S", "--noconfirm", "--needed", "octopi"],
            "Installing Octopi package manager...",
        ));
    }

    if selected.contains(&"pacseek".to_string()) {
        commands.push(Command::aur(
            &["-S", "--noconfirm", "--needed", "pacseek", "pacfinder"],
            "Installing PacSeek package browser...",
        ));
    }

    if selected.contains(&"bauh".to_string()) {
        commands.push(Command::aur(
            &["-S", "--noconfirm", "--needed", "bauh"],
            "Installing Bauh package manager...",
        ));
    }

    if selected.contains(&"warehouse".to_string()) {
        commands.push(Command::normal(
            "flatpak",
            &["install", "-y", "io.github.flattool.Warehouse"],
            "Installing Warehouse from Flathub...",
        ));
    }

    if selected.contains(&"flatseal".to_string()) {
        commands.push(Command::normal(
            "flatpak",
            &["install", "-y", "com.github.tchx84.Flatseal"],
            "Installing Flatseal from Flathub...",
        ));
    }

    if selected.contains(&"bazaar".to_string()) {
        commands.push(Command::normal(
            "flatpak",
            &["install", "-y", "io.github.kolunmi.Bazaar"],
            "Installing Bazaar from Flathub...",
        ));
    }

    commands
}

/// Setup download Arch ISO button.
fn setup_download_arch_iso(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_download_arch_iso") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Download Arch ISO button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        show_download_dialog(window.upcast_ref());
    });
}

/// Setup external link buttons.
fn setup_external_links(builder: &Builder) {
    if let Some(btn) = builder.object::<Button>("link_discord") {
        btn.connect_clicked(|_| {
            info!("Discord link clicked");
            let _ = core::package::open_url(config::links::DISCORD);
        });
    }

    if let Some(btn) = builder.object::<Button>("link_youtube") {
        btn.connect_clicked(|_| {
            info!("YouTube link clicked");
            let _ = core::package::open_url(config::links::YOUTUBE);
        });
    }

    if let Some(btn) = builder.object::<Button>("link_website") {
        btn.connect_clicked(|_| {
            info!("Website link clicked");
            let _ = core::package::open_url(config::links::WEBSITE);
        });
    }

    if let Some(btn) = builder.object::<Button>("link_donate") {
        btn.connect_clicked(|_| {
            info!("Donate link clicked");
            let _ = core::package::open_url(config::links::DONATE);
        });
    }
}

/// Helper to get the parent window from a button.
fn get_window(button: &Button) -> Option<ApplicationWindow> {
    button
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok())
}
