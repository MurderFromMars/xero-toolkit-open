//! Containers and VMs page button handlers.
//!
//! Handles:
//! - Docker installation and setup (+ uninstall)
//! - Podman installation with optional Desktop (+ uninstall)
//! - VirtualBox installation (+ uninstall)
//! - DistroBox installation (+ uninstall)
//! - KVM/QEMU virtualization setup (+ uninstall)
//! - iOS iPA Sideloader / Plume Impactor from Flathub (+ uninstall)

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Helper to update button appearance based on installation status.
///
/// When installed: install button shows "Installed ✓" and the uninstall button appears.
/// When not installed: install button shows normal label and uninstall is hidden.
fn update_button_state(
    install_button: &Button,
    uninstall_button: &Button,
    is_installed: bool,
    default_label: &str,
) {
    if is_installed {
        install_button.set_label(&format!("{} ✓", default_label));
        install_button.set_sensitive(false);
        install_button.remove_css_class("suggested-action");
        install_button.add_css_class("dim-label");
        uninstall_button.set_visible(true);
    } else {
        install_button.set_label(default_label);
        install_button.set_sensitive(true);
        install_button.add_css_class("suggested-action");
        install_button.remove_css_class("dim-label");
        uninstall_button.set_visible(false);
    }
}

/// Set up all button handlers for the containers/VMs page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_docker(page_builder, window);
    setup_podman(page_builder, window);
    setup_vbox(page_builder, window);
    setup_distrobox(page_builder, window);
    setup_kvm(page_builder, window);
    setup_ipa_sideloader(page_builder, window);
}

// ─── Docker ──────────────────────────────────────────────────────────────────

fn is_docker_installed() -> bool {
    core::is_package_installed("docker")
}

fn setup_docker(builder: &Builder, window: &ApplicationWindow) {
    let btn_install = extract_widget::<Button>(builder, "btn_docker");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_docker_uninstall");

    // Initial state
    update_button_state(&btn_install, &btn_uninstall, is_docker_installed(), "Docker");

    // Refresh on window focus
    let btn_i = btn_install.clone();
    let btn_u = btn_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            update_button_state(&btn_i, &btn_u, is_docker_installed(), "Docker");
        }
    });

    // Install handler
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("Docker install button clicked");

        let user = crate::config::env::get().user.clone();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "docker",
                        "docker-compose",
                        "docker-buildx",
                    ])
                    .description("Installing Docker engine and tools...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "docker.service"])
                    .description("Enabling Docker service...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("groupadd")
                    .args(&["-f", "docker"])
                    .description("Ensuring docker group exists...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("usermod")
                    .args(&["-aG", "docker", &user])
                    .description("Adding your user to docker group...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "Docker Setup");
    });

    // Uninstall handler
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("Docker uninstall button clicked");

        let user = crate::config::env::get().user.clone();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["stop", "docker.service", "docker.socket"])
                    .description("Stopping Docker services...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["disable", "docker.service", "docker.socket"])
                    .description("Disabling Docker services...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("gpasswd")
                    .args(&["-d", &user, "docker"])
                    .description("Removing your user from docker group...")
                    .build(),
            )
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-Rns",
                        "--noconfirm",
                        "docker",
                        "docker-compose",
                        "docker-buildx",
                    ])
                    .description("Removing Docker packages and dependencies...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "Docker Uninstall");
    });
}

// ─── Podman ──────────────────────────────────────────────────────────────────

fn is_podman_installed() -> bool {
    core::is_package_installed("podman")
}

fn setup_podman(builder: &Builder, window: &ApplicationWindow) {
    let btn_install = extract_widget::<Button>(builder, "btn_podman");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_podman_uninstall");

    // Initial state
    update_button_state(&btn_install, &btn_uninstall, is_podman_installed(), "Podman");

    // Refresh on window focus
    let btn_i = btn_install.clone();
    let btn_u = btn_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            update_button_state(&btn_i, &btn_u, is_podman_installed(), "Podman");
        }
    });

    // Install handler
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("Podman install button clicked");

        let config = SelectionDialogConfig::new(
            "Podman Installation",
            "Podman will be installed. Optionally include the Podman Desktop GUI.",
        )
        .selection_type(SelectionType::Single)
        .selection_required(false)
        .add_option(SelectionOption::new(
            "podman_desktop",
            "Podman Desktop",
            "Graphical interface for managing containers",
            core::is_flatpak_installed("io.podman_desktop.PodmanDesktop"),
        ))
        .confirm_label("Install");

        let window_for_closure = window_clone.clone();
        show_selection_dialog(window_clone.upcast_ref(), config, move |selected| {
            let mut commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "podman", "podman-docker"])
                        .description("Installing Podman container engine...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("systemctl")
                        .args(&["enable", "--now", "podman.socket"])
                        .description("Enabling Podman socket...")
                        .build(),
                );

            if selected.iter().any(|s| s == "podman_desktop") {
                commands = commands.then(
                    Command::builder()
                        .normal()
                        .program("flatpak")
                        .args(&[
                            "install",
                            "-y",
                            "flathub",
                            "io.podman_desktop.PodmanDesktop",
                        ])
                        .description("Installing Podman Desktop GUI...")
                        .build(),
                );
            }

            if !commands.is_empty() {
                task_runner::run(
                    window_for_closure.upcast_ref(),
                    commands.build(),
                    "Podman Setup",
                );
            }
        });
    });

    // Uninstall handler
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("Podman uninstall button clicked");

        let mut commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["stop", "podman.socket"])
                    .description("Stopping Podman socket...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["disable", "podman.socket"])
                    .description("Disabling Podman socket...")
                    .build(),
            );

        // Remove Podman Desktop flatpak if installed
        if core::is_flatpak_installed("io.podman_desktop.PodmanDesktop") {
            commands = commands.then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["uninstall", "-y", "io.podman_desktop.PodmanDesktop"])
                    .description("Removing Podman Desktop GUI...")
                    .build(),
            );
        }

        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-Rns", "--noconfirm", "podman", "podman-docker"])
                .description("Removing Podman packages and dependencies...")
                .build(),
        );

        task_runner::run(
            window_clone.upcast_ref(),
            commands.build(),
            "Podman Uninstall",
        );
    });
}

// ─── VirtualBox ──────────────────────────────────────────────────────────────

fn is_vbox_installed() -> bool {
    core::is_package_installed("virtualbox-meta")
}

fn setup_vbox(builder: &Builder, window: &ApplicationWindow) {
    let btn_install = extract_widget::<Button>(builder, "btn_vbox");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_vbox_uninstall");

    // Initial state
    update_button_state(
        &btn_install,
        &btn_uninstall,
        is_vbox_installed(),
        "Virtual Box",
    );

    // Refresh on window focus
    let btn_i = btn_install.clone();
    let btn_u = btn_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            update_button_state(&btn_i, &btn_u, is_vbox_installed(), "Virtual Box");
        }
    });

    // Install handler
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("VirtualBox install button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "virtualbox-meta"])
                    .description("Installing VirtualBox...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "VirtualBox Setup");
    });

    // Uninstall handler
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("VirtualBox uninstall button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-Rns", "--noconfirm", "virtualbox-meta"])
                    .description("Removing VirtualBox and dependencies...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "VirtualBox Uninstall",
        );
    });
}

// ─── DistroBox ───────────────────────────────────────────────────────────────

fn is_distrobox_installed() -> bool {
    core::is_package_installed("distrobox")
}

fn setup_distrobox(builder: &Builder, window: &ApplicationWindow) {
    let btn_install = extract_widget::<Button>(builder, "btn_distrobox");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_distrobox_uninstall");

    // Initial state
    update_button_state(
        &btn_install,
        &btn_uninstall,
        is_distrobox_installed(),
        "DistroBox",
    );

    // Refresh on window focus
    let btn_i = btn_install.clone();
    let btn_u = btn_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            update_button_state(&btn_i, &btn_u, is_distrobox_installed(), "DistroBox");
        }
    });

    // Install handler
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("DistroBox install button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "distrobox"])
                    .description("Installing DistroBox...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["install", "-y", "io.github.dvlv.boxbuddyrs"])
                    .description("Installing BoxBuddy GUI...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "DistroBox Setup");
    });

    // Uninstall handler
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("DistroBox uninstall button clicked");

        let mut commands = CommandSequence::new();

        // Remove BoxBuddy flatpak if installed
        if core::is_flatpak_installed("io.github.dvlv.boxbuddyrs") {
            commands = commands.then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["uninstall", "-y", "io.github.dvlv.boxbuddyrs"])
                    .description("Removing BoxBuddy GUI...")
                    .build(),
            );
        }

        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-Rns", "--noconfirm", "distrobox"])
                .description("Removing DistroBox and dependencies...")
                .build(),
        );

        task_runner::run(
            window_clone.upcast_ref(),
            commands.build(),
            "DistroBox Uninstall",
        );
    });
}

// ─── KVM / QEMU ─────────────────────────────────────────────────────────────

fn is_kvm_installed() -> bool {
    core::is_package_installed("virt-manager-meta")
}

fn setup_kvm(builder: &Builder, window: &ApplicationWindow) {
    let btn_install = extract_widget::<Button>(builder, "btn_kvm");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_kvm_uninstall");

    // Initial state
    update_button_state(
        &btn_install,
        &btn_uninstall,
        is_kvm_installed(),
        "Qemu Virtual Manager",
    );

    // Refresh on window focus
    let btn_i = btn_install.clone();
    let btn_u = btn_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            update_button_state(&btn_i, &btn_u, is_kvm_installed(), "Qemu Virtual Manager");
        }
    });

    // Install handler
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("KVM install button clicked");

        let mut commands = CommandSequence::new();

        // Remove conflicting packages if installed
        if core::is_package_installed("iptables") {
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-Rdd", "--noconfirm", "iptables"])
                    .description("Removing conflicting iptables...")
                    .build(),
            );
        }

        if core::is_package_installed("gnu-netcat") {
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-Rdd", "--noconfirm", "gnu-netcat"])
                    .description("Removing conflicting gnu-netcat...")
                    .build(),
            );
        }

        commands = commands.then(
            Command::builder()
                .aur()
                .args(&[
                    "-S",
                    "--noconfirm",
                    "--needed",
                    "virt-manager-meta",
                    "openbsd-netcat",
                ])
                .description("Installing virtualization packages...")
                .build(),
        );

        commands = commands.then(
            Command::builder()
                .privileged()
                .program("sh")
                .args(&[
                    "-c",
                    "echo 'options kvm-intel nested=1' > /etc/modprobe.d/kvm-intel.conf",
                ])
                .description("Enabling nested virtualization...")
                .build(),
        );

        commands = commands.then(
            Command::builder()
                .privileged()
                .program("systemctl")
                .args(&["restart", "libvirtd.service"])
                .description("Restarting libvirtd service...")
                .build(),
        );

        task_runner::run(window_clone.upcast_ref(), commands.build(), "KVM / QEMU Setup");
    });

    // Uninstall handler
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("KVM uninstall button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["stop", "libvirtd.service"])
                    .description("Stopping libvirtd service...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["disable", "libvirtd.service"])
                    .description("Disabling libvirtd service...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/etc/modprobe.d/kvm-intel.conf"])
                    .description("Removing nested virtualization config...")
                    .build(),
            )
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-Rns",
                        "--noconfirm",
                        "virt-manager-meta",
                        "openbsd-netcat",
                    ])
                    .description("Removing virtualization packages and dependencies...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "KVM / QEMU Uninstall",
        );
    });
}

// ─── iOS iPA Sideloader ─────────────────────────────────────────────────────

fn is_ipa_sideloader_installed() -> bool {
    core::is_flatpak_installed("dev.khcrysalis.PlumeImpactor")
}

fn setup_ipa_sideloader(builder: &Builder, window: &ApplicationWindow) {
    let btn_install = extract_widget::<Button>(builder, "btn_ipa_sideloader");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_ipa_sideloader_uninstall");

    // Initial state
    update_button_state(
        &btn_install,
        &btn_uninstall,
        is_ipa_sideloader_installed(),
        "iOS iPA Sideloader",
    );

    // Refresh on window focus
    let btn_i = btn_install.clone();
    let btn_u = btn_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            update_button_state(
                &btn_i,
                &btn_u,
                is_ipa_sideloader_installed(),
                "iOS iPA Sideloader",
            );
        }
    });

    // Install handler
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("iOS iPA Sideloader install button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["install", "-y", "flathub", "dev.khcrysalis.PlumeImpactor"])
                    .description("Installing Plume Impactor from Flathub...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "iOS iPA Sideloader Setup");
    });

    // Uninstall handler
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("iOS iPA Sideloader uninstall button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["uninstall", "-y", "dev.khcrysalis.PlumeImpactor"])
                    .description("Removing Plume Impactor...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "iOS iPA Sideloader Uninstall",
        );
    });
}
