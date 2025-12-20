//! Customization page button handlers.
//!
//! Handles:
//! - ZSH All-in-One setup
//! - Save Desktop tool
//! - GRUB theme installation
//! - Plasma wallpapers
//! - Layan GTK4 patch

use crate::ui::task_runner::{self, Command};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
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
    let Some(button) = builder.object::<Button>("btn_zsh_aio") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("ZSH AiO button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let home = std::env::var("HOME").unwrap_or_default();
        let user = std::env::var("USER").unwrap_or_default();

        let commands = vec![
            Command::aur(
                &[
                    "-S",
                    "--needed",
                    "--noconfirm",
                    "zsh",
                    "grml-zsh-config",
                    "fastfetch",
                ],
                "Installing ZSH and dependencies...",
            ),
            Command::privileged(
                "sh",
                &[
                    "-c",
                    "sh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\" \"\" --unattended",
                ],
                "Installing Oh My Zsh framework...",
            ),
            Command::aur(
                &[
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
                ],
                "Installing fonts and terminal enhancements...",
            ),
            Command::normal(
                "git",
                &[
                    "clone",
                    "https://github.com/zsh-users/zsh-completions",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-completions", home),
                ],
                "Installing ZSH completions plugin...",
            ),
            Command::normal(
                "git",
                &[
                    "clone",
                    "https://github.com/zsh-users/zsh-autosuggestions",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-autosuggestions", home),
                ],
                "Installing ZSH autosuggestions plugin...",
            ),
            Command::normal(
                "git",
                &[
                    "clone",
                    "https://github.com/zsh-users/zsh-syntax-highlighting.git",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-syntax-highlighting", home),
                ],
                "Installing ZSH syntax highlighting plugin...",
            ),
            Command::normal(
                "sh",
                &[
                    "-c",
                    &format!(
                        "mv -f {}/.zshrc {}/.zshrc.user 2>/dev/null || true",
                        home, home
                    ),
                ],
                "Backing up existing ZSH configuration...",
            ),
            Command::normal(
                "wget",
                &[
                    "-q",
                    "-P",
                    &home,
                    "https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/.zshrc",
                ],
                "Downloading XeroLinux ZSH configuration...",
            ),
            Command::privileged(
                "chsh",
                &[&user, "-s", "/bin/zsh"],
                "Setting ZSH as default shell...",
            ),
        ];

        task_runner::run(
            window.upcast_ref(),
            commands,
            "ZSH All-in-One Setup",
        );
    });
}

fn setup_save_desktop(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_save_desktop") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Save Desktop Tool button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![Command::normal(
            "flatpak",
            &["install", "-y", "io.github.vikdevelop.SaveDesktop"],
            "Installing Save Desktop tool from Flathub...",
        )];

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Save Desktop Tool Installation",
        );
    });
}

fn setup_grub_theme(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_grub_theme") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("GRUB Theme button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let home = std::env::var("HOME").unwrap_or_default();

        let commands = vec![
            Command::normal(
                "git",
                &[
                    "clone",
                    "--depth",
                    "1",
                    "https://github.com/xerolinux/xero-grubs",
                    &format!("{}/xero-grubs", home),
                ],
                "Downloading GRUB theme repository...",
            ),
            Command::privileged(
                "sh",
                &["-c", &format!("cd {}/xero-grubs && ./install.sh", home)],
                "Installing GRUB theme...",
            ),
            Command::normal(
                "rm",
                &["-rf", &format!("{}/xero-grubs", home)],
                "Cleaning up temporary files...",
            ),
        ];

        task_runner::run(
            window.upcast_ref(),
            commands,
            "XeroLinux GRUB Theme Installation",
        );
    });
}

fn setup_wallpapers(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_wallpapers") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Plasma Wallpapers button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let commands = vec![Command::aur(
            &["-S", "--noconfirm", "--needed", "kde-wallpapers-extra"],
            "Installing KDE wallpapers collection (~1.2GB)...",
        )];

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Plasma Wallpapers Installation (~1.2GB)",
        );
    });
}

fn setup_layan_patch(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_layan_patch") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Layan GTK4 Patch button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let home = std::env::var("HOME").unwrap_or_default();

        let commands = vec![
            Command::normal(
                "git",
                &[
                    "clone",
                    "--depth",
                    "1",
                    "https://github.com/vinceliuice/Layan-gtk-theme.git",
                    &format!("{}/Layan-gtk-theme", home),
                ],
                "Downloading Layan GTK theme...",
            ),
            Command::privileged(
                "sh",
                &[
                    "-c",
                    &format!(
                        "cd {}/Layan-gtk-theme && sh install.sh -l -c dark -d {}/.themes",
                        home, home
                    ),
                ],
                "Installing Layan GTK theme...",
            ),
            Command::normal(
                "rm",
                &["-rf", &format!("{}/Layan-gtk-theme", home)],
                "Cleaning up GTK theme files...",
            ),
            Command::normal(
                "git",
                &[
                    "clone",
                    "--depth",
                    "1",
                    "https://github.com/vinceliuice/Layan-kde.git",
                    &format!("{}/Layan-kde", home),
                ],
                "Downloading Layan KDE theme...",
            ),
            Command::privileged(
                "sh",
                &["-c", &format!("cd {}/Layan-kde && sh install.sh", home)],
                "Installing Layan KDE theme...",
            ),
            Command::normal(
                "rm",
                &["-rf", &format!("{}/Layan-kde", home)],
                "Cleaning up KDE theme files...",
            ),
        ];

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Layan GTK4 Patch & Update",
        );
    });
}

/// Helper to get the parent window from a button.
fn get_window(button: &Button) -> Option<ApplicationWindow> {
    button
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok())
}
