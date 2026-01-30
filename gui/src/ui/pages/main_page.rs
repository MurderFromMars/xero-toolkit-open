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
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::dialogs::terminal;
use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the main page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_update_system(page_builder, window);
    setup_pkg_manager(page_builder, window);
    setup_download_arch_iso(page_builder, window);
    setup_install_nix(page_builder, window);
    setup_external_links(page_builder);
}

/// Setup system update button.
fn setup_update_system(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_update_system");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Update System button clicked");

        // Use terminal dialog for interactive system update
        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "System Update",
            "/usr/local/bin/upd",
            &[],
        );
    });
}

/// Setup package manager GUI button.
fn setup_pkg_manager(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_pkg_manager");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("PKG Manager GUI button clicked");

        // Check which package managers are already installed
        let config = SelectionDialogConfig::new(
            "Package Manager GUI Applications",
            "Select which package manager GUIs to install. Multiple selections allowed.",
        )
        .selection_type(SelectionType::Multi)
        .selection_required(true)
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

        let window_for_closure = window.clone();
        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            let commands = build_pkg_manager_commands(&selected);

            if !commands.is_empty() {
                task_runner::run(
                    window_for_closure.upcast_ref(),
                    commands.build(),
                    "Package Manager GUI Installation",
                );
            }
        });
    });
}

/// Build commands for selected package managers.
fn build_pkg_manager_commands(selected: &[String]) -> CommandSequence {
    let mut commands = CommandSequence::new();

    if selected.iter().any(|s| s == "octopi") {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "octopi"])
                .description("Installing Octopi package manager...")
                .build(),
        );
    }

    if selected.iter().any(|s| s == "pacseek") {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "pacseek", "pacfinder"])
                .description("Installing PacSeek package browser...")
                .build(),
        );
    }

    if selected.iter().any(|s| s == "bauh") {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "bauh"])
                .description("Installing Bauh package manager...")
                .build(),
        );
    }

    if selected.iter().any(|s| s == "warehouse") {
        commands = commands.then(
            Command::builder()
                .normal()
                .program("flatpak")
                .args(&["install", "-y", "io.github.flattool.Warehouse"])
                .description("Installing Warehouse from Flathub...")
                .build(),
        );
    }

    if selected.iter().any(|s| s == "flatseal") {
        commands = commands.then(
            Command::builder()
                .normal()
                .program("flatpak")
                .args(&["install", "-y", "com.github.tchx84.Flatseal"])
                .description("Installing Flatseal from Flathub...")
                .build(),
        );
    }

    if selected.iter().any(|s| s == "bazaar") {
        commands = commands.then(
            Command::builder()
                .normal()
                .program("flatpak")
                .args(&["install", "-y", "io.github.kolunmi.Bazaar"])
                .description("Installing Bazaar from Flathub...")
                .build(),
        );
    }

    commands
}

/// Setup download Arch ISO button.
fn setup_download_arch_iso(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_download_arch_iso");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Download Arch ISO button clicked");

        show_download_dialog(window.upcast_ref());
    });
}

/// Setup Nix package manager installation button.
fn setup_install_nix(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_install_nix");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Install Nix button clicked");

        // Show warning dialog before installation
        let window_clone = window.clone();
        show_warning_confirmation(
            window.upcast_ref(),
            "Experimental Feature Warning",
            "Nix Package Manager is an <span foreground=\"red\" weight=\"bold\">EXPERIMENTAL</span> feature.\n\n\
            This is intended for <span foreground=\"red\" weight=\"bold\">EXPERIENCED USERS ONLY</span>.\n\
            <span foreground=\"red\" weight=\"bold\">Do NOT enable</span> unless you know what you are doing.\n\
            <span foreground=\"red\" weight=\"bold\">NO SUPPORT</span> will be provided for Nix-related issues.\n\n\
            Proceed at your own risk.",
            move || {
                info!("User confirmed Nix installation after warning");

                // Show selection dialog to choose installation type
                let window_for_selection = window_clone.clone();
                let config = SelectionDialogConfig::new(
                    "Nix Installation Type",
                    "Choose the installation type for Nix Package Manager. Multi-user is recommended for most users.",
                )
                .selection_type(SelectionType::Single)
                .selection_required(true)
                .add_option(SelectionOption::new(
                    "multi-user",
                    "Multi-user Installation (Recommended)",
                    "Better build isolation, security, and sharing between users. Requires systemd and sudo.",
                    false,
                ))
                .add_option(SelectionOption::new(
                    "single-user",
                    "Single-user Installation",
                    "Simpler installation owned by your user. Easier to uninstall.",
                    false,
                ))
                .confirm_label("Continue");

                show_selection_dialog(window_clone.upcast_ref(), config, move |selected| {
                    if selected.is_empty() {
                        return;
                    }

                    // Get the selected installation type (should be only one)
                    let install_type = &selected[0];
                    let install_command = if install_type == "multi-user" {
                        info!("Installing Nix with multi-user (daemon) mode");
                        "sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --daemon"
                    } else {
                        info!("Installing Nix with single-user (no-daemon) mode");
                        "sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --no-daemon"
                    };

                    terminal::show_terminal_dialog(
                        window_for_selection.upcast_ref(),
            "Install Nix Package Manager",
                        "sh",
                        &["-c", install_command],
                    );
                });
            },
        );
    });
}

/// Setup external link buttons.
fn setup_external_links(builder: &Builder) {
    let btn_discord = extract_widget::<Button>(builder, "link_discord");
    btn_discord.connect_clicked(|_| {
        info!("Discord link clicked");
        let _ = core::package::open_url(config::links::DISCORD);
    });

    let btn_youtube = extract_widget::<Button>(builder, "link_youtube");
    btn_youtube.connect_clicked(|_| {
        info!("YouTube link clicked");
        let _ = core::package::open_url(config::links::YOUTUBE);
    });

    let btn_github = extract_widget::<Button>(builder, "link_github");
    btn_github.connect_clicked(|_| {
        info!("GitHub link clicked");
        let _ = core::package::open_url(config::links::GITHUB);
    });

    let btn_website = extract_widget::<Button>(builder, "link_website");
    btn_website.connect_clicked(|_| {
        info!("Website link clicked");
        let _ = core::package::open_url(config::links::WEBSITE);
    });

    let btn_donate = extract_widget::<Button>(builder, "link_donate");
    btn_donate.connect_clicked(|_| {
        info!("Donate link clicked");
        let _ = core::package::open_url(config::links::DONATE);
    });
}
