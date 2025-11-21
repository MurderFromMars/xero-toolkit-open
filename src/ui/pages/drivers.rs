//! Drivers and hardware tools page button handlers.
//!
//! Handles:
//! - GPU drivers (AMD, Intel, NVIDIA) via selection dialog
//! - Tailscale VPN
//! - ASUS ROG laptop tools

use crate::ui::selection_dialog;
use crate::ui::terminal;
use crate::utils;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Label};
use log::{info, warn};
use vte4::prelude::*;
use vte4::Terminal;

/// Set up all button handlers for the drivers page
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    setup_gpu_drivers(&page_builder, &terminal_output, &terminal_title);
    setup_tailscale(&page_builder, &terminal_output, &terminal_title);
    setup_asus_rog(&page_builder, &terminal_output, &terminal_title);
}

fn setup_gpu_drivers(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_gpu_drivers) = page_builder.object::<gtk4::Button>("btn_gpu_drivers") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_gpu_drivers.connect_clicked(move |button| {
            info!("Drivers: GPU Drivers button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(
                    b"\r\nAnother action is already running. Please wait for it to complete.\r\n",
                );
                return;
            }

            show_gpu_driver_selection(button, &terminal_clone, &title_clone);
        });
    }
}

fn show_gpu_driver_selection(button: &gtk4::Button, terminal: &Terminal, terminal_title: &Label) {
    let widget = button.clone().upcast::<gtk4::Widget>();
    let window = widget
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok());

    if let Some(window) = window {
        let terminal_for_dialog = terminal.clone();
        let title_for_dialog = terminal_title.clone();
        let button_for_dialog = button.clone();
        let window_ref = window.upcast_ref::<gtk4::Window>();

        let config = selection_dialog::SelectionDialogConfig::new(
            "GPU Driver Selection",
            "Select your GPU vendor. Note: Dual/Hybrid GPU setups require manual terminal configuration.",
        )
        .add_option(selection_dialog::SelectionOption::new(
            "amd",
            "AMD GPU",
            "Radeon graphics drivers and Vulkan support",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "intel",
            "Intel GPU",
            "Intel graphics drivers and media acceleration",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "nvidia_closed",
            "NVIDIA (Closed Source)",
            "Proprietary NVIDIA drivers with CUDA support",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "nvidia_open",
            "NVIDIA (Open Source)",
            "Open source NVIDIA drivers (Turing+ GPUs)",
            false,
        ))
        .confirm_label("Install");

        selection_dialog::show_selection_dialog(window_ref, config, move |selected_ids| {
            if selected_ids.is_empty() {
                return;
            }

            let helper = match utils::detect_aur_helper() {
                Some(h) => h,
                None => {
                    warn!("No AUR helper detected");
                    terminal_for_dialog
                        .feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                    return;
                }
            };

            let mut commands = vec![];

            if selected_ids.contains(&"amd".to_string()) {
                commands.push(terminal::TerminalCommand::new(
                    helper,
                    &[
                        "-S",
                        "--needed",
                        "--noconfirm",
                        "linux-headers",
                        "vulkan-radeon",
                        "lib32-vulkan-radeon",
                        "vulkan-icd-loader",
                        "lib32-vulkan-icd-loader",
                        "linux-firmware-radeon",
                        "vulkan-mesa-layers",
                        "lib32-vulkan-mesa-layers",
                    ],
                ));
            }

            if selected_ids.contains(&"intel".to_string()) {
                commands.push(terminal::TerminalCommand::new(
                    helper,
                    &[
                        "-S",
                        "--needed",
                        "--noconfirm",
                        "linux-headers",
                        "vulkan-intel",
                        "lib32-vulkan-intel",
                        "vulkan-icd-loader",
                        "lib32-vulkan-icd-loader",
                        "intel-media-driver",
                        "intel-gmmlib",
                        "onevpl-intel-gpu",
                        "gstreamer-vaapi",
                    ],
                ));
            }

            if selected_ids.contains(&"nvidia_closed".to_string()) {
                commands.push(terminal::TerminalCommand::new(
                    helper,
                    &[
                        "-S",
                        "--needed",
                        "--noconfirm",
                        "linux-headers",
                        "nvidia-dkms",
                        "nvidia-utils",
                        "lib32-nvidia-utils",
                        "nvidia-settings",
                        "vulkan-icd-loader",
                        "lib32-vulkan-icd-loader",
                        "egl-wayland",
                        "opencl-nvidia",
                        "lib32-opencl-nvidia",
                        "libvdpau-va-gl",
                        "libvdpau",
                        "linux-firmware-nvidia",
                    ],
                ));

                commands.push(terminal::TerminalCommand::new("sudo",
                    &["sh", "-c", "sed -i 's/\\(GRUB_CMDLINE_LINUX_DEFAULT=[\"'\\''']\\)/\\1nvidia-drm.modeset=1 /' /etc/default/grub"]));
                commands.push(terminal::TerminalCommand::new(
                    "sudo",
                    &["grub-mkconfig", "-o", "/boot/grub/grub.cfg"],
                ));
                commands.push(terminal::TerminalCommand::new("sudo",
                    &["sh", "-c", "sed -i 's/^MODULES=()/MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)/' /etc/mkinitcpio.conf"]));
                commands.push(terminal::TerminalCommand::new(
                    "sudo",
                    &[
                        "systemctl",
                        "enable",
                        "nvidia-suspend.service",
                        "nvidia-hibernate.service",
                        "nvidia-resume.service",
                    ],
                ));
                commands.push(terminal::TerminalCommand::new("sudo", &["mkinitcpio", "-P"]));
            }

            if selected_ids.contains(&"nvidia_open".to_string()) {
                commands.push(terminal::TerminalCommand::new(
                    helper,
                    &[
                        "-S",
                        "--needed",
                        "--noconfirm",
                        "linux-headers",
                        "nvidia-open-dkms",
                        "nvidia-utils",
                        "lib32-nvidia-utils",
                        "nvidia-settings",
                        "vulkan-icd-loader",
                        "lib32-vulkan-icd-loader",
                        "egl-wayland",
                        "opencl-nvidia",
                        "lib32-opencl-nvidia",
                        "libvdpau-va-gl",
                        "libvdpau",
                        "linux-firmware-nvidia",
                    ],
                ));

                commands.push(terminal::TerminalCommand::new("sudo",
                    &["sh", "-c", "sed -i 's/\\(GRUB_CMDLINE_LINUX_DEFAULT=[\"'\\''']\\)/\\1nvidia-drm.modeset=1 /' /etc/default/grub"]));
                commands.push(terminal::TerminalCommand::new(
                    "sudo",
                    &["grub-mkconfig", "-o", "/boot/grub/grub.cfg"],
                ));
                commands.push(terminal::TerminalCommand::new("sudo",
                    &["sh", "-c", "sed -i 's/^MODULES=()/MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)/' /etc/mkinitcpio.conf"]));
                commands.push(terminal::TerminalCommand::new(
                    "sudo",
                    &[
                        "systemctl",
                        "enable",
                        "nvidia-suspend.service",
                        "nvidia-hibernate.service",
                        "nvidia-resume.service",
                    ],
                ));
                commands.push(terminal::TerminalCommand::new("sudo", &["mkinitcpio", "-P"]));
            }

            if !commands.is_empty() {
                terminal::run_terminal_commands(
                    &button_for_dialog,
                    &terminal_for_dialog,
                    &title_for_dialog,
                    commands,
                    "GPU Driver Installation",
                );
            }
        });
    }
}

fn setup_tailscale(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_tailscale) = page_builder.object::<gtk4::Button>("btn_tailscale") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_tailscale.connect_clicked(move |button| {
            info!("Drivers: Tailscale VPN button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(
                    b"\r\nAnother action is already running. Please wait for it to complete.\r\n",
                );
                return;
            }

            let commands = vec![terminal::TerminalCommand::new("bash",
                &["-c", "curl -fsSL https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/install.sh | bash"])];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Install Tailscale VPN",
            );
        });
    }
}

fn setup_asus_rog(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_asus_rog) = page_builder.object::<gtk4::Button>("btn_asus_rog") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_asus_rog.connect_clicked(move |button| {
            info!("Drivers: ASUS ROG Tools button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(
                    b"\r\nAnother action is already running. Please wait for it to complete.\r\n",
                );
                return;
            }

            let helper = match utils::detect_aur_helper() {
                Some(h) => h,
                None => {
                    warn!("No AUR helper detected");
                    terminal_clone
                        .feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                    return;
                }
            };

            let commands = vec![
                terminal::TerminalCommand::new(
                    helper,
                    &[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "rog-control-center",
                        "asusctl",
                        "supergfxctl",
                    ],
                ),
                terminal::TerminalCommand::new(
                    "sudo",
                    &["systemctl", "enable", "--now", "asusd", "supergfxd"],
                ),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Install ASUS ROG Tools",
            );
        });
    }
}

