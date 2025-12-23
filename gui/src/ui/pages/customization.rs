//! Customization page button handlers.
//!
//! Handles:
//! - ZSH All-in-One setup
//! - Save Desktop tool
//! - GRUB theme installation
//! - Plasma wallpapers
//! - Layan GTK4 patch

use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::{extract_widget, get_window_from_button};
use gtk4::prelude::*;
use gtk4::{Builder, Button};
use log::info;

/// Set up all button handlers for the customization page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder) {
    setup_zsh_aio(page_builder);
    setup_save_desktop(page_builder);
    setup_grub_theme(page_builder);
    setup_wallpapers(page_builder);
    setup_layan_patch(page_builder);
}

fn setup_zsh_aio(builder: &Builder) {
    let button = extract_widget::<Button>(builder, "btn_zsh_aio");

    button.connect_clicked(move |btn| {
        info!("ZSH AiO button clicked");

        let Some(window) = get_window_from_button(btn) else {
            return;
        };

        let home = std::env::var("HOME").unwrap_or_default();
        let user = std::env::var("USER").unwrap_or_default();

        let commands = CommandSequence::new()
            .then(Command::builder()
                .aur()
                .args(&[
                    "-S",
                    "--needed",
                    "--noconfirm",
                    "zsh",
                    "grml-zsh-config",
                    "fastfetch",
                ])
                .description("Installing ZSH and dependencies...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("sh")
                .args(&[
                    "-c",
                    "sh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\" \"\" --unattended",
                ])
                .description("Installing Oh My Zsh framework...")
                .build())
            .then(Command::builder()
                .aur()
                .args(&[
                    "-S",
                    "--noconfirm",
                    "--needed",
                    "pacseek",
                    "ttf-meslo-nerd",
                    "siji-git",
                    "otf-unifont",
                    "bdf-unifont",
                    "noto-color-emoji-fontconfig",
                    "xorg-fonts-misc",
                    "ttf-dejavu",
                    "ttf-meslo-nerd-font-powerlevel10k",
                    "noto-fonts-emoji",
                    "powerline-fonts",
                    "oh-my-posh-bin",
                ])
                .description("Installing fonts and terminal enhancements...")
                .build())
            .then(Command::builder()
                .normal()
                .program("git")
                .args(&[
                    "clone",
                    "https://github.com/zsh-users/zsh-completions",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-completions", home),
                ])
                .description("Installing ZSH completions plugin...")
                .build())
            .then(Command::builder()
                .normal()
                .program("git")
                .args(&[
                    "clone",
                    "https://github.com/zsh-users/zsh-autosuggestions",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-autosuggestions", home),
                ])
                .description("Installing ZSH autosuggestions plugin...")
                .build())
            .then(Command::builder()
                .normal()
                .program("git")
                .args(&[
                    "clone",
                    "https://github.com/zsh-users/zsh-syntax-highlighting.git",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-syntax-highlighting", home),
                ])
                .description("Installing ZSH syntax highlighting plugin...")
                .build())
            .then(Command::builder()
                .normal()
                .program("sh")
                .args(&[
                    "-c",
                    &format!(
                        "mv -f {}/.zshrc {}/.zshrc.user 2>/dev/null || true",
                        home, home
                    ),
                ])
                .description("Backing up existing ZSH configuration...")
                .build())
            .then(Command::builder()
                .normal()
                .program("wget")
                .args(&[
                    "-q",
                    "-P",
                    &home,
                    "https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/.zshrc",
                ])
                .description("Downloading XeroLinux ZSH configuration...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("chsh")
                .args(&[&user, "-s", "/bin/zsh"])
                .description("Setting ZSH as default shell...")
                .build())
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "ZSH All-in-One Setup",
        );
    });
}

fn setup_save_desktop(builder: &Builder) {
    let button = extract_widget::<Button>(builder, "btn_save_desktop");

    button.connect_clicked(move |btn| {
        info!("Save Desktop Tool button clicked");

        let Some(window) = get_window_from_button(btn) else {
            return;
        };

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["install", "-y", "io.github.vikdevelop.SaveDesktop"])
                    .description("Installing Save Desktop tool from Flathub...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Save Desktop Tool Installation",
        );
    });
}

fn setup_grub_theme(builder: &Builder) {
    let button = extract_widget::<Button>(builder, "btn_grub_theme");

    button.connect_clicked(move |btn| {
        info!("GRUB Theme button clicked");

        let Some(window) = get_window_from_button(btn) else {
            return;
        };

        let home = std::env::var("HOME").unwrap_or_default();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("git")
                    .args(&[
                        "clone",
                        "--depth",
                        "1",
                        "https://github.com/xerolinux/xero-grubs",
                        &format!("{}/xero-grubs", home),
                    ])
                    .description("Downloading GRUB theme repository...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&["-c", &format!("cd {}/xero-grubs && ./install.sh", home)])
                    .description("Installing GRUB theme...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("rm")
                    .args(&["-rf", &format!("{}/xero-grubs", home)])
                    .description("Cleaning up temporary files...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "XeroLinux GRUB Theme Installation",
        );
    });
}

fn setup_wallpapers(builder: &Builder) {
    let button = extract_widget::<Button>(builder, "btn_wallpapers");

    button.connect_clicked(move |btn| {
        info!("Plasma Wallpapers button clicked");

        let Some(window) = get_window_from_button(btn) else {
            return;
        };

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "kde-wallpapers-extra"])
                    .description("Installing KDE wallpapers collection (~1.2GB)...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Plasma Wallpapers Installation (~1.2GB)",
        );
    });
}

fn setup_layan_patch(builder: &Builder) {
    let button = extract_widget::<Button>(builder, "btn_layan_patch");

    button.connect_clicked(move |btn| {
        info!("Layan GTK4 Patch button clicked");

        let Some(window) = get_window_from_button(btn) else {
            return;
        };

        let home = std::env::var("HOME").unwrap_or_default();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("git")
                    .args(&[
                        "clone",
                        "--depth",
                        "1",
                        "https://github.com/vinceliuice/Layan-gtk-theme.git",
                        &format!("{}/Layan-gtk-theme", home),
                    ])
                    .description("Downloading Layan GTK theme...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        &format!(
                            "cd {}/Layan-gtk-theme && sh install.sh -l -c dark -d {}/.themes",
                            home, home
                        ),
                    ])
                    .description("Installing Layan GTK theme...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("rm")
                    .args(&["-rf", &format!("{}/Layan-gtk-theme", home)])
                    .description("Cleaning up GTK theme files...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("git")
                    .args(&[
                        "clone",
                        "--depth",
                        "1",
                        "https://github.com/vinceliuice/Layan-kde.git",
                        &format!("{}/Layan-kde", home),
                    ])
                    .description("Downloading Layan KDE theme...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&["-c", &format!("cd {}/Layan-kde && sh install.sh", home)])
                    .description("Installing Layan KDE theme...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("rm")
                    .args(&["-rf", &format!("{}/Layan-kde", home)])
                    .description("Cleaning up KDE theme files...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Layan GTK4 Patch & Update");
    });
}
