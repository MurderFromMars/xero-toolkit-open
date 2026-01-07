//! Gaming tools page button handlers.
//!
//! Handles:
//! - Steam AiO installation
//! - LACT GPU overclocking
//! - Game launchers (Lutris, Heroic, Bottles)
//! - Controller tools

use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the gaming tools page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_steam_aio(page_builder, window);
    setup_lact_oc(page_builder, window);
    setup_lutris(page_builder, window);
    setup_heroic(page_builder, window);
    setup_bottles(page_builder, window);
    setup_controller(page_builder, window);
    setup_falcond(page_builder, window);
}

fn setup_steam_aio(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_steam_aio");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Steam AiO button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "steam",
                        "gamescope",
                        "mangohud",
                        "mangoverlay",
                        "lib32-mangohud",
                        "wine-meta",
                        "wine-nine",
                        "ttf-liberation",
                        "lib32-fontconfig",
                        "wqy-zenhei",
                        "vkd3d",
                        "giflib",
                        "lib32-giflib",
                        "libpng",
                        "lib32-libpng",
                        "libldap",
                        "lib32-libldap",
                        "gnutls",
                        "lib32-gnutls",
                        "mpg123",
                        "lib32-mpg123",
                        "openal",
                        "lib32-openal",
                        "v4l-utils",
                        "lib32-v4l-utils",
                        "libpulse",
                        "lib32-libpulse",
                        "libgpg-error",
                        "lib32-libgpg-error",
                        "alsa-plugins",
                        "lib32-alsa-plugins",
                        "alsa-lib",
                        "lib32-alsa-lib",
                        "libjpeg-turbo",
                        "lib32-libjpeg-turbo",
                        "sqlite",
                        "lib32-sqlite",
                        "libxcomposite",
                        "lib32-libxcomposite",
                        "libxinerama",
                        "lib32-libgcrypt",
                        "libgcrypt",
                        "lib32-libxinerama",
                        "ncurses",
                        "lib32-ncurses",
                        "ocl-icd",
                        "lib32-ocl-icd",
                        "libxslt",
                        "lib32-libxslt",
                        "libva",
                        "lib32-libva",
                        "gtk3",
                        "lib32-gtk3",
                        "gst-plugins-base-libs",
                        "lib32-gst-plugins-base-libs",
                        "vulkan-icd-loader",
                        "lib32-vulkan-icd-loader",
                        "cups",
                        "dosbox",
                        "lib32-opencl-icd-loader",
                        "lib32-vkd3d",
                        "opencl-icd-loader",
                    ])
                    .description("Installing Steam and gaming dependencies...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Steam AiO Installation");
    });
}

fn setup_lact_oc(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_lact_oc");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("LACT OC button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "lact"])
                    .description("Installing LACT GPU control utility...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "lactd"])
                    .description("Enabling LACT background service...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "LACT GPU Tools");
    });
}

fn setup_lutris(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_lutris");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Lutris button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&[
                        "install",
                        "-y",
                        "net.lutris.Lutris",
                        "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/25.08",
                        "org.freedesktop.Platform.VulkanLayer.MangoHud/x86_64/25.08",
                    ])
                    .description("Installing Lutris and Vulkan layers...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Lutris Installation");
    });
}

fn setup_heroic(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_heroic");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Heroic button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&[
                        "install",
                        "-y",
                        "com.heroicgameslauncher.hgl",
                        "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/25.08",
                        "org.freedesktop.Platform.VulkanLayer.MangoHud/x86_64/25.08",
                    ])
                    .description("Installing Heroic Games Launcher...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Heroic Launcher Installation",
        );
    });
}

fn setup_bottles(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_bottles");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Bottles button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&[
                        "install",
                        "-y",
                        "com.usebottles.bottles",
                        "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/25.08",
                        "org.freedesktop.Platform.VulkanLayer.MangoHud/x86_64/25.08",
                    ])
                    .description("Installing Bottles and Vulkan layers...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Bottles Installation");
    });
}

fn setup_controller(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_controller");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Controller Tools button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "gamepad-tool-bin",
                        "sc-controller",
                        "xone-dkms-git",
                        "dualsensectl-git",
                        "xone-dongle-firmware",
                    ])
                    .description("Installing controller tools and drivers...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Controller Tools Installation",
        );
    });
}

fn setup_flacond(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_falcond");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("PikaOS Falcond clicked");

        let commands = CommandSequence::new()
        .then(
            Command::builder()
            .aur()
            .args(&["-S", "--noconfirm", "--needed", "falcond", "falcond-gui", "falcond-profiles"])
            .description("Installing Falcond Gaming utility...")
            .build(),
        )
        .then(
            Command::builder()
            .privileged()
            .program("systemctl")
            .args(&["enable", "--now", "falcond"])
            .description("Enabling falcond background service...")
            .build(),
        )
        .build();

        task_runner::run(window.upcast_ref(), commands, "Flacond Installation");
    });
}
