//! Containers and VMs page button handlers.
//!
//! Handles:
//! - Docker installation and setup
//! - Podman installation (with optional Desktop)
//! - VirtualBox installation
//! - DistroBox installation
//! - KVM/QEMU virtualization setup

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption,
};
use crate::ui::task_runner::{self, Command};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the containers/VMs page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder) {
    setup_docker(page_builder);
    setup_podman(page_builder);
    setup_vbox(page_builder);
    setup_distrobox(page_builder);
    setup_kvm(page_builder);
}

fn setup_docker(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_docker") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Docker button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

        let commands = vec![
            Command::aur(
                &[
                    "-S",
                    "--noconfirm",
                    "--needed",
                    "docker",
                    "docker-compose",
                    "docker-buildx",
                ],
                "Installing Docker engine and tools...",
            ),
            Command::privileged(
                "systemctl",
                &["enable", "--now", "docker.service"],
                "Enabling Docker service...",
            ),
            Command::privileged(
                "groupadd",
                &["-f", "docker"],
                "Ensuring docker group exists...",
            ),
            Command::privileged(
                "usermod",
                &["-aG", "docker", &user],
                "Adding your user to docker group...",
            ),
        ];

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Docker Setup",
        );
    });
}

fn setup_podman(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_podman") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Podman button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let window_clone = window.clone();

        let config = SelectionDialogConfig::new(
            "Podman Installation",
            "Podman will be installed. Optionally include the Podman Desktop GUI.",
        )
        .add_option(SelectionOption::new(
            "podman_desktop",
            "Podman Desktop",
            "Graphical interface for managing containers",
            core::is_flatpak_installed("io.podman_desktop.PodmanDesktop"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            let mut commands = vec![
                Command::aur(
                    &["-S", "--noconfirm", "--needed", "podman", "podman-docker"],
                    "Installing Podman container engine...",
                ),
                Command::privileged(
                    "systemctl",
                    &["enable", "--now", "podman.socket"],
                    "Enabling Podman socket...",
                ),
            ];

            if selected.contains(&"podman_desktop".to_string()) {
                commands.push(Command::normal(
                    "flatpak",
                    &[
                        "install",
                        "-y",
                        "flathub",
                        "io.podman_desktop.PodmanDesktop",
                    ],
                    "Installing Podman Desktop GUI...",
                ));
            }

            if !commands.is_empty() {
                task_runner::run(window_clone.upcast_ref(), commands, "Podman Setup");
            }
        });
    });
}

fn setup_vbox(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_vbox") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("VirtualBox button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![Command::aur(
            &["-S", "--noconfirm", "--needed", "virtualbox-meta"],
            "Installing VirtualBox...",
        )];

        task_runner::run(window.upcast_ref(), commands, "VirtualBox Setup");
    });
}

fn setup_distrobox(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_distrobox") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("DistroBox button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![
            Command::aur(
                &["-S", "--noconfirm", "--needed", "distrobox"],
                "Installing DistroBox...",
            ),
            Command::normal(
                "flatpak",
                &["install", "-y", "io.github.dvlv.boxbuddyrs"],
                "Installing BoxBuddy GUI...",
            ),
        ];

        task_runner::run(window.upcast_ref(), commands, "DistroBox Setup");
    });
}

fn setup_kvm(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_kvm") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("KVM button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let mut commands: Vec<Command> = Vec::new();

        // Remove conflicting packages if installed
        if core::is_package_installed("iptables") {
            commands.push(Command::aur(
                &["-Rdd", "--noconfirm", "iptables"],
                "Removing conflicting iptables...",
            ));
        }

        if core::is_package_installed("gnu-netcat") {
            commands.push(Command::aur(
                &["-Rdd", "--noconfirm", "gnu-netcat"],
                "Removing conflicting gnu-netcat...",
            ));
        }

        commands.push(Command::aur(
            &[
                "-S",
                "--noconfirm",
                "--needed",
                "virt-manager-meta",
                "openbsd-netcat",
            ],
            "Installing virtualization packages...",
        ));

        commands.push(Command::privileged(
            "sh",
            &[
                "-c",
                "echo 'options kvm-intel nested=1' > /etc/modprobe.d/kvm-intel.conf",
            ],
            "Enabling nested virtualization...",
        ));

        commands.push(Command::privileged(
            "systemctl",
            &["restart", "libvirtd.service"],
            "Restarting libvirtd service...",
        ));

        task_runner::run(window.upcast_ref(), commands, "KVM / QEMU Setup");
    });
}

/// Helper to get the parent window from a button.
fn get_window(button: &Button) -> Option<ApplicationWindow> {
    button
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok())
}
