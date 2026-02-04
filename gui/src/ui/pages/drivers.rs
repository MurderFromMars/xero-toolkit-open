//! Drivers and hardware tools page button handlers.
//!
//! Handles:
//! - Tailscale VPN
//! - ASUS ROG laptop tools
//! - OpenRazer drivers

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the drivers page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_tailscale(page_builder, window);
    setup_asus_rog(page_builder, window);
    setup_openrazer(page_builder, window);
    setup_zenergy(page_builder, window);
    setup_nvidia_legacy(page_builder, window);
    setup_rocm(page_builder, window);
    setup_cuda(page_builder, window);
}

fn setup_tailscale(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_tailscale");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Tailscale VPN button clicked");

        let commands = CommandSequence::new()
            .then(Command::builder()
                .privileged()
                .program("bash")
                .args(&[
                    "-c",
                    "curl -fsSL https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/install.sh | bash",
                ])
                .description("Installing Tailscale VPN...")
                .build())
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install Tailscale VPN");
    });
}

fn setup_asus_rog(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_asus_rog");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("ASUS ROG Tools button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "rog-control-center",
                        "asusctl",
                        "supergfxctl",
                    ])
                    .description("Installing ASUS ROG control tools...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "asusd", "supergfxd"])
                    .description("Enabling ASUS ROG services...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install ASUS ROG Tools");
    });
}

fn setup_openrazer(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_openrazer");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("OpenRazer Drivers button clicked");

        // Show selection dialog for optional frontends
        let window_clone = window.clone();
        let config = SelectionDialogConfig::new(
            "OpenRazer Drivers & Frontend",
            "OpenRazer drivers will be installed. Optionally select a frontend application for managing your Razer devices.",
        )
        .selection_type(SelectionType::Multi)
        .selection_required(false)
        .add_option(SelectionOption::new(
            "polychromatic",
            "Polychromatic",
            "Graphical frontend for managing Razer devices (GTK-based)",
            core::is_package_installed("polychromatic"),
        ))
        .add_option(SelectionOption::new(
            "razergenie",
            "RazerGenie",
            "Graphical frontend for managing Razer devices (Qt-based)",
            core::is_package_installed("razergenie"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            let commands = build_openrazer_commands(&selected);
            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Install OpenRazer Drivers (Reboot Required)",
            );
        });
    });
}

/// Build commands for OpenRazer installation.
fn build_openrazer_commands(selected_frontends: &[String]) -> CommandSequence {
    let user = crate::config::env::get().user.clone();
    let mut commands = CommandSequence::new();

    // Always install openrazer-meta-git
    commands = commands.then(
        Command::builder()
            .aur()
            .args(&["-S", "--noconfirm", "--needed", "openrazer-meta-git"])
            .description("Installing OpenRazer drivers...")
            .build(),
    );

    // Add user to plugdev group
    commands = commands.then(
        Command::builder()
            .privileged()
            .program("usermod")
            .args(&["-aG", "plugdev", &user])
            .description("Adding user to plugdev group...")
            .build(),
    );

    // Optionally install selected frontends
    if selected_frontends.iter().any(|s| s == "polychromatic") {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "polychromatic"])
                .description("Installing Polychromatic frontend...")
                .build(),
        );
    }

    if selected_frontends.iter().any(|s| s == "razergenie") {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "razergenie"])
                .description("Installing RazerGenie frontend...")
                .build(),
        );
    }

    commands
}

fn setup_zenergy(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_zenergy");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Zenergy Driver button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "zenergy-dkms-git"])
                    .description("Installing Zenergy Driver...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install Zenergy Driver");
    });
}

fn setup_nvidia_legacy(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_nvidia_legacy");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Nvidia Legacy Drivers button clicked");

        let window_clone = window.clone();
        show_warning_confirmation(
            window.upcast_ref(),
            "Nvidia Legacy Drivers",
            "This is only intended for <span foreground=\"red\" weight=\"bold\">GTX900/1000</span> Series Legacy GPUs\n\
            For <span foreground=\"cyan\" weight=\"bold\">RTX/Turing+</span> GPUs download the <span foreground=\"green\" weight=\"bold\">nVidia</span> ISO instead.\n\n\
            <span foreground=\"red\" weight=\"bold\">No Support/Help</span> will be provided for those Legacy GPUs !",
            move || {
                // Use configured path
                let script_dir = crate::config::paths::scripts();
                let grub_script = script_dir.join("nvidia_grub.sh").to_string_lossy().into_owned();
                let mkinitcpio_script = script_dir
                    .join("nvidia_mkinitcpio.sh")
                    .to_string_lossy()
                    .into_owned();

                let commands = CommandSequence::new()
                    .then(
                        Command::builder()
                            .aur()
                            .args(&[
                                "-S",
                                "--noconfirm",
                                "--needed",
                                "lib32-nvidia-580xx-utils",
                                "lib32-opencl-nvidia-580xx",
                                "nvidia-580xx-dkms",
                                "nvidia-580xx-utils",
                                "opencl-nvidia-580xx",
                            ])
                            .description("Installing Nvidia Legacy Drivers...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("bash")
                            .args(&[&grub_script])
                            .description("Configuring GRUB (nvidia-drm.modeset=1)...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("bash")
                            .args(&[&mkinitcpio_script])
                            .description("Configuring mkinitcpio modules...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&[
                                "enable",
                                "nvidia-suspend.service",
                                "nvidia-hibernate.service",
                                "nvidia-resume.service",
                            ])
                            .description("Enabling Nvidia power management services...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("mkinitcpio")
                            .args(&["-P"])
                            .description("Rebuilding initramfs...")
                            .build(),
                    )
                    .build();

                task_runner::run(
                    window_clone.upcast_ref(),
                    commands,
                    "Install Nvidia Legacy Drivers",
                );
            },
        );
    });
}

fn setup_rocm(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_rocm");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("AMD ROCm button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "rocm-hip-sdk",
                        "rocm-opencl-sdk",
                    ])
                    .description("Installing AMD ROCm SDK...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install AMD ROCm");
    });
}

fn setup_cuda(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_cuda");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("NVIDIA CUDA button clicked");

        // Show selection dialog for CUDA version
        let window_clone = window.clone();
        let config = SelectionDialogConfig::new(
            "NVIDIA CUDA Toolkit",
            "Select the CUDA version to install. The latest version is recommended for most users.",
        )
        .selection_type(SelectionType::Single)
        .selection_required(true)
        .add_option(SelectionOption::new(
            "cuda",
            "CUDA (Latest)",
            "Install the latest CUDA toolkit from official repositories",
            core::is_package_installed("cuda"),
        ))
        .add_option(SelectionOption::new(
            "cuda-12.9",
            "CUDA 12.9",
            "Install CUDA Toolkit version 12.9 specifically",
            core::is_package_installed("cuda-12.9"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            if let Some(package) = selected.first() {
                let description = format!("Installing {}...", package);
                let commands = CommandSequence::new()
                    .then(
                        Command::builder()
                            .aur()
                            .args(&["-S", "--noconfirm", "--needed", package])
                            .description(&description)
                            .build(),
                    )
                    .build();

                task_runner::run(window_clone.upcast_ref(), commands, "Install NVIDIA CUDA");
            }
        });
    });
}
