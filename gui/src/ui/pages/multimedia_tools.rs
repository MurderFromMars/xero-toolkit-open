//! Multimedia tools page button handlers.
//!
//! Handles:
//! - OBS-Studio with plugins and V4L2
//! - Kdenlive video editor
//! - Jellyfin server installation
//! - GPU Screen Recorder GTK (repo-first, AUR fallback)

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder};
use log::info;

/// Set up all button handlers for the multimedia tools page
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_obs_studio_aio(page_builder, window);
    setup_kdenlive(page_builder, window);
    setup_jellyfin(page_builder, window);
    setup_gpu_screen_recorder(page_builder, window);
}

fn setup_obs_studio_aio(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_obs_studio_aio = extract_widget::<gtk4::Button>(page_builder, "btn_obs_studio_aio");
    let window = window.clone();
    btn_obs_studio_aio.connect_clicked(move |_| {
        info!("Multimedia tools: OBS-Studio AiO button clicked");
        let window_ref = window.upcast_ref();

                let wayland_hotkeys_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.WaylandHotkeys");
                let v4l2_installed = core::is_package_installed("v4l2loopback-dkms");

                let graphics_capture_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.OBSVkCapture") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.Gstreamer") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.GStreamerVaapi");

                let transitions_effects_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.MoveTransition") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.TransitionTable") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.ScaleToSound");

                let streaming_tools_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.WebSocket") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.SceneSwitcher") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.DroidCam");

                let audio_video_tools_installed =
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.waveform") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.VerticalCanvas") &&
                    core::is_flatpak_installed("com.obsproject.Studio.Plugin.BackgroundRemoval");

                let config = SelectionDialogConfig::new(
                    "OBS-Studio & Plugins Installation",
                    "OBS-Studio will be installed. Optionally select plugins to install.",
                )
                .selection_type(SelectionType::Multi)
                .selection_required(false)
                .add_option(SelectionOption::new(
                    "wayland_hotkeys",
                    "Wayland Hotkeys Plugin",
                    "Enable hotkey support for OBS on Wayland",
                    wayland_hotkeys_installed,
                ))
                .add_option(SelectionOption::new(
                    "graphics_capture",
                    "Graphics Capture Plugins",
                    "VkCapture, GStreamer, GStreamer VA-API",
                    graphics_capture_installed,
                ))
                .add_option(SelectionOption::new(
                    "transitions_effects",
                    "Transitions & Effects",
                    "Move Transition, Transition Table, Scale to Sound",
                    transitions_effects_installed,
                ))
                .add_option(SelectionOption::new(
                    "streaming_tools",
                    "Streaming & Recording Tools",
                    "WebSocket API, Scene Switcher, DroidCam",
                    streaming_tools_installed,
                ))
                .add_option(SelectionOption::new(
                    "audio_video_tools",
                    "Audio & Video Tools",
                    "Waveform, Vertical Canvas, Background Removal",
                    audio_video_tools_installed,
                ))
                .add_option(SelectionOption::new(
                    "v4l2",
                    "V4L2loopback Virtual Camera",
                    "Enable OBS virtual camera functionality",
                    v4l2_installed,
                ))
                .confirm_label("Install");

                let window_for_closure = window.clone();
                show_selection_dialog(window_ref, config, move |selected_ids| {
                    let mut commands = CommandSequence::new();

                    // Always install OBS-Studio
                    commands = commands.then(Command::builder()
                        .normal()
                        .program("flatpak")
                        .args(&["install", "-y", "com.obsproject.Studio"])
                        .description("Installing OBS-Studio...")
                        .build());

                    if selected_ids.iter().any(|s| s == "wayland_hotkeys") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&["install", "-y", "com.obsproject.Studio.Plugin.WaylandHotkeys"])
                            .description("Installing Wayland Hotkeys plugin...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "graphics_capture") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.OBSVkCapture",
                                "org.freedesktop.Platform.VulkanLayer.OBSVkCapture/x86_64/25.08",
                                "com.obsproject.Studio.Plugin.Gstreamer",
                                "com.obsproject.Studio.Plugin.GStreamerVaapi",
                            ])
                            .description("Installing graphics capture plugins...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "transitions_effects") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.MoveTransition",
                                "com.obsproject.Studio.Plugin.TransitionTable",
                                "com.obsproject.Studio.Plugin.ScaleToSound",
                            ])
                            .description("Installing transitions & effects plugins...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "streaming_tools") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.WebSocket",
                                "com.obsproject.Studio.Plugin.SceneSwitcher",
                                "com.obsproject.Studio.Plugin.DroidCam",
                            ])
                            .description("Installing streaming tools...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "audio_video_tools") {
                        commands = commands.then(Command::builder()
                            .normal()
                            .program("flatpak")
                            .args(&[
                                "install",
                                "-y",
                                "com.obsproject.Studio.Plugin.waveform",
                                "com.obsproject.Studio.Plugin.VerticalCanvas",
                                "com.obsproject.Studio.Plugin.BackgroundRemoval",
                            ])
                            .description("Installing audio/video enhancement plugins...")
                            .build());
                    }
                    if selected_ids.iter().any(|s| s == "v4l2") {
                        commands = commands.then(Command::builder()
                            .aur()
                            .args(&["-S", "--noconfirm", "--needed", "v4l2loopback-dkms", "v4l2loopback-utils"])
                            .description("Installing V4L2 loopback modules...")
                            .build());
                        commands = commands.then(Command::builder()
                            .privileged()
                            .program("sh")
                            .args(&["-c", "echo 'v4l2loopback' > /etc/modules-load.d/v4l2loopback.conf"])
                            .description("Enabling V4L2 loopback module at boot...")
                            .build());
                        commands = commands.then(Command::builder()
                            .privileged()
                            .program("sh")
                            .args(&[
                                "-c",
                                "echo 'options v4l2loopback exclusive_caps=1 card_label=\"OBS Virtual Camera\"' > /etc/modprobe.d/v4l2loopback.conf",
                            ])
                            .description("Configuring virtual camera options...")
                            .build());
                    }

                    task_runner::run(window_for_closure.upcast_ref(), commands.build(), "OBS-Studio Setup");
                });
    });
}

fn setup_kdenlive(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_kdenlive = extract_widget::<gtk4::Button>(page_builder, "btn_kdenlive");
    let window = window.clone();
    btn_kdenlive.connect_clicked(move |_| {
        info!("Multimedia tools: Kdenlive button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "kdenlive"])
                    .description("Installing Kdenlive...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Kdenlive Installation");
    });
}

fn setup_jellyfin(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_jellyfin = extract_widget::<gtk4::Button>(page_builder, "btn_jellyfin");
    let window = window.clone();
    btn_jellyfin.connect_clicked(move |_| {
        info!("Multimedia tools: Jellyfin button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "jellyfin-server",
                        "jellyfin-web",
                        "jellyfin-ffmpeg",
                    ])
                    .description("Installing Jellyfin server and components...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "jellyfin.service"])
                    .description("Starting Jellyfin service...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Jellyfin Server Setup");
    });
}

fn setup_gpu_screen_recorder(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_gpu_screen_recorder =
        extract_widget::<gtk4::Button>(page_builder, "btn_gpu_screen_recorder");
    let window = window.clone();
    btn_gpu_screen_recorder.connect_clicked(move |_| {
        info!("Multimedia tools: GPU Screen Recorder button clicked");

        // Check official repos first; fall back to AUR if unavailable.
        let in_repos = core::is_package_in_repos("gpu-screen-recorder-gtk");

        let install_cmd = if in_repos {
            info!("gpu-screen-recorder-gtk found in official repos – installing via pacman");
            Command::builder()
                .privileged()
                .program("pacman")
                .args(&["-S", "--noconfirm", "--needed", "gpu-screen-recorder-gtk"])
                .description("Installing GPU Screen Recorder GTK from official repos...")
                .build()
        } else {
            info!("gpu-screen-recorder-gtk not in official repos – installing via AUR");
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "gpu-screen-recorder-gtk"])
                .description("Installing GPU Screen Recorder GTK from AUR...")
                .build()
        };

        let commands = CommandSequence::new().then(install_cmd).build();

        task_runner::run(window.upcast_ref(), commands, "GPU Screen Recorder Setup");
    });
}
