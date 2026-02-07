//! Servicing and system tweaks page button handlers.

use crate::core;
use crate::ui::dialogs::terminal;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder};
use log::info;

pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_clr_pacman(page_builder, window);
    setup_unlock_pacman(page_builder, window);
    setup_plasma_x11(page_builder, window);
    setup_pacman_db_fix(page_builder, window);
    setup_waydroid_guide(page_builder);
    setup_fix_gpgme(page_builder, window);
    setup_fix_arch_keyring(page_builder, window);
    setup_update_mirrorlist(page_builder, window);
    setup_parallel_downloads(page_builder, window);
    setup_cachyos_repos(page_builder, window);
    setup_chaotic_aur(page_builder, window);
    setup_xero_repo(page_builder, window);
    setup_xpackagemanager(page_builder, window);
}

fn setup_clr_pacman(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_clr_pacman = extract_widget::<gtk4::Button>(page_builder, "btn_clr_pacman");
    let window = window.clone();
    btn_clr_pacman.connect_clicked(move |_| {
        info!("Servicing: Clear Pacman Cache button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Scc", "--noconfirm"])
                    .description("Clearing Pacman cache...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Clear Pacman Cache");
    });
}

fn setup_unlock_pacman(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_unlock_pacman = extract_widget::<gtk4::Button>(page_builder, "btn_unlock_pacman");
    let window = window.clone();
    btn_unlock_pacman.connect_clicked(move |_| {
        info!("Servicing: Unlock Pacman DB button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/var/lib/pacman/db.lck"])
                    .description("Removing Pacman lock file...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Unlock Pacman Database");
    });
}

fn setup_plasma_x11(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_plasma_x11 = extract_widget::<gtk4::Button>(page_builder, "btn_plasma_x11");
    let window = window.clone();
    btn_plasma_x11.connect_clicked(move |_| {
        info!("Servicing: Plasma X11 Session button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "kwin-x11", "plasma-x11-session"])
                    .description("Installing KDE Plasma X11 session components...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Install KDE X11 Session");
    });
}

fn setup_pacman_db_fix(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_pacman_db_fix = extract_widget::<gtk4::Button>(page_builder, "btn_pacman_db_fix");
    let window = window.clone();
    btn_pacman_db_fix.connect_clicked(move |_| {
        info!("Servicing: Pacman DB Fix button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "find /var/lib/pacman/local/ -type f -name 'desc' -exec sed -i '/^%INSTALLED_DB%$/,+2d' {} \\;",
                    ])
                    .description("Fixing Pacman local database...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Pacman DB Fix");
    });
}

fn setup_waydroid_guide(page_builder: &Builder) {
    let btn_waydroid_guide = extract_widget::<gtk4::Button>(page_builder, "btn_waydroid_guide");
    btn_waydroid_guide.connect_clicked(move |_| {
        info!("Servicing: WayDroid Guide button clicked - opening guide");
        let _ = std::process::Command::new("xdg-open")
            .arg("https://xerolinux.xyz/posts/waydroid-guide/")
            .spawn();
    });
}

fn setup_fix_gpgme(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fix_gpgme = extract_widget::<gtk4::Button>(page_builder, "btn_fix_gpgme");
    let window = window.clone();
    btn_fix_gpgme.connect_clicked(move |_| {
        info!("Servicing: Fix GPGME Database button clicked");
        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "Fix GPGME Database",
            "pkexec",
            &["sh", "-c", "rm -rf /var/lib/pacman/sync && pacman -Syy"],
        );
    });
}

fn setup_fix_arch_keyring(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fix_arch_keyring = extract_widget::<gtk4::Button>(page_builder, "btn_fix_arch_keyring");
    let window = window.clone();
    btn_fix_arch_keyring.connect_clicked(move |_| {
        info!("Servicing: Fix Arch Keyring button clicked");
        let commands = CommandSequence::new()
            .then(Command::builder()
                .privileged()
                .program("rm")
                .args(&["-rf", "/etc/pacman.d/gnupg"])
                .description("Removing existing GnuPG keyring...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("pacman-key")
                .args(&["--init"])
                .description("Initializing new keyring...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("pacman-key")
                .args(&["--populate"])
                .description("Populating keyring...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("sh")
                .args(&["-c", "echo 'keyserver hkp://keyserver.ubuntu.com:80' >> /etc/pacman.d/gnupg/gpg.conf"])
                .description("Setting keyserver...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("pacman")
                .args(&["-Syy", "--noconfirm", "archlinux-keyring"])
                .description("Reinstalling Arch Linux keyring...")
                .build())
            .build();
        task_runner::run(window.upcast_ref(), commands, "Fix GnuPG Keyring");
    });
}

fn setup_update_mirrorlist(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_update_mirrorlist = extract_widget::<gtk4::Button>(page_builder, "btn_update_mirrorlist");
    let window = window.clone();
    btn_update_mirrorlist.connect_clicked(move |_| {
        info!("Servicing: Update Mirrorlist button clicked");

        let rate_mirrors_installed = core::is_package_installed("rate-mirrors");

        let mirror_mappings: Vec<(&str, &str, &str)> = vec![
            ("/etc/pacman.d/mirrorlist", "arch", "Arch"),
            ("/etc/pacman.d/chaotic-mirrorlist", "chaotic-aur", "Chaotic-AUR"),
            ("/etc/pacman.d/cachyos-mirrorlist", "cachyos", "CachyOS"),
            ("/etc/pacman.d/endeavouros-mirrorlist", "endeavouros", "EndeavourOS"),
            ("/etc/pacman.d/manjaro-mirrorlist", "manjaro", "Manjaro"),
            ("/etc/pacman.d/rebornos-mirrorlist", "rebornos", "RebornOS"),
            ("/etc/pacman.d/artix-mirrorlist", "artix", "Artix"),
        ];

        let mut commands = CommandSequence::new();

        if !rate_mirrors_installed {
            commands = commands.then(Command::builder()
                .aur()
                .args(&["-S", "--needed", "--noconfirm", "rate-mirrors"])
                .description("Installing rate-mirrors utility...")
                .build());
        }

        for (file_path, repo_id, repo_name) in mirror_mappings {
            if std::path::Path::new(file_path).exists() {
                let cmd = format!(
                    "rate-mirrors --allow-root --protocol https {} | tee {}",
                    repo_id, file_path
                );
                let description = format!("Updating {} mirrorlist...", repo_name);
                commands = commands.then(Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&["-c", &cmd])
                    .description(&description)
                    .build());
            }
        }

        task_runner::run(window.upcast_ref(), commands.build(), "Update System Mirrorlists");
    });
}

fn setup_parallel_downloads(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_parallel_downloads = extract_widget::<gtk4::Button>(page_builder, "btn_parallel_downloads");
    let window = window.clone();
    btn_parallel_downloads.connect_clicked(move |_| {
        info!("Servicing: Change Parallel Downloads button clicked");
        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "Change Parallel Downloads",
            "pkexec",
            &["pmpd"],
        );
    });
}

fn setup_cachyos_repos(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_cachyos_repos = extract_widget::<gtk4::Button>(page_builder, "btn_cachyos_repos");
    let window = window.clone();
    btn_cachyos_repos.connect_clicked(move |_| {
        info!("Servicing: Install CachyOS Repos button clicked");
        
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("sh")
                    .args(&[
                        "-c",
                        "curl -fsSL https://mirror.cachyos.org/cachyos-repo.tar.xz -o /tmp/cachyos-repo.tar.xz && cd /tmp && tar xvf cachyos-repo.tar.xz",
                    ])
                    .description("Downloading CachyOS repository files...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "cd /tmp/cachyos-repo && yes | ./cachyos-repo.sh",
                    ])
                    .description("Running CachyOS repository installer...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("rm")
                    .args(&["-rf", "/tmp/cachyos-repo", "/tmp/cachyos-repo.tar.xz"])
                    .description("Cleaning up temporary files...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Syy"])
                    .description("Refreshing package databases...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install CachyOS Repositories");
    });
}

fn setup_chaotic_aur(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_chaotic_aur = extract_widget::<gtk4::Button>(page_builder, "btn_chaotic_aur");
    let window = window.clone();
    btn_chaotic_aur.connect_clicked(move |_| {
        info!("Servicing: Install Chaotic-AUR button clicked");
        
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman-key")
                    .args(&["--recv-key", "3056513887B78AEB", "--keyserver", "keyserver.ubuntu.com"])
                    .description("Receiving Chaotic-AUR signing key...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman-key")
                    .args(&["--lsign-key", "3056513887B78AEB"])
                    .description("Locally signing Chaotic-AUR key...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&[
                        "-U",
                        "--noconfirm",
                        "https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-keyring.pkg.tar.zst",
                    ])
                    .description("Installing Chaotic-AUR keyring...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&[
                        "-U",
                        "--noconfirm",
                        "https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-mirrorlist.pkg.tar.zst",
                    ])
                    .description("Installing Chaotic-AUR mirrorlist...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "grep -q '\\[chaotic-aur\\]' /etc/pacman.conf || echo -e '\\n[chaotic-aur]\\nInclude = /etc/pacman.d/chaotic-mirrorlist' >> /etc/pacman.conf",
                    ])
                    .description("Adding Chaotic-AUR to pacman.conf...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Syy"])
                    .description("Refreshing package databases...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install Chaotic-AUR Repository");
    });
}

fn setup_xero_repo(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_xero_repo = extract_widget::<gtk4::Button>(page_builder, "btn_xero_repo");
    let window = window.clone();
    btn_xero_repo.connect_clicked(move |_| {
        info!("Servicing: Add Xero Linux Repository button clicked");
        
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "grep -q '\\[xerolinux\\]' /etc/pacman.conf || echo -e '\\n[xerolinux]\\nSigLevel = Optional TrustAll\\nServer = https://repos.xerolinux.xyz/$repo/$arch' >> /etc/pacman.conf",
                    ])
                    .description("Adding Xero Linux repository to pacman.conf...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Syy"])
                    .description("Refreshing package databases...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Add Xero Linux Repository");
    });
}

fn setup_xpackagemanager(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_xpackagemanager = extract_widget::<gtk4::Button>(page_builder, "btn_xpackagemanager");
    let btn_xpackagemanager_uninstall = extract_widget::<gtk4::Button>(page_builder, "btn_xpackagemanager_uninstall");

    fn update_button_state(setup_btn: &gtk4::Button, uninstall_btn: &gtk4::Button, is_installed: bool) {
        if is_installed {
            setup_btn.set_label("Launch");
            setup_btn.add_css_class("suggested-action");
            uninstall_btn.set_visible(true);
        } else {
            setup_btn.set_label("Install");
            setup_btn.remove_css_class("suggested-action");
            uninstall_btn.set_visible(false);
        }
    }

    let is_installed = std::path::Path::new("/usr/bin/xpackagemanager").exists();
    update_button_state(&btn_xpackagemanager, &btn_xpackagemanager_uninstall, is_installed);

    let btn_setup_clone = btn_xpackagemanager.clone();
    let btn_uninstall_clone = btn_xpackagemanager_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            let is_installed = std::path::Path::new("/usr/bin/xpackagemanager").exists();
            update_button_state(&btn_setup_clone, &btn_uninstall_clone, is_installed);
        }
    });

    let window_clone = window.clone();
    btn_xpackagemanager.connect_clicked(move |_| {
        info!("Servicing: xPackageManager button clicked");

        if std::path::Path::new("/usr/bin/xpackagemanager").exists() {
            info!("Launching xPackageManager...");
            if let Err(e) = std::process::Command::new("xpackagemanager")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                log::error!("Failed to launch xPackageManager: {}", e);
            }
        } else {
            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .privileged()
                        .program("pacman")
                        .args(&["-S", "--needed", "--noconfirm", "rust", "qt6-base", "qt6-declarative", "pacman", "flatpak", "git"])
                        .description("Installing build dependencies...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&[
                            "-c",
                            "rm -rf /tmp/xpm-build && git clone https://github.com/MurderFromMars/xPackageManager.git /tmp/xpm-build",
                        ])
                        .description("Cloning xPackageManager source...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&["-c", "cd /tmp/xpm-build && cargo build --release"])
                        .description("Building xPackageManager (this may take a few minutes)...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "mkdir -p /opt/xpackagemanager && install -Dm755 /tmp/xpm-build/target/release/xpackagemanager /opt/xpackagemanager/xpackagemanager && ln -sf /opt/xpackagemanager/xpackagemanager /usr/bin/xpackagemanager",
                        ])
                        .description("Installing binary to /opt/xpackagemanager...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            r#"cat > /usr/share/applications/xpackagemanager.desktop << 'EOF'
[Desktop Entry]
Name=xPackage Manager
Comment=Modern package manager for Arch Linux
Exec=xpackagemanager
Icon=system-software-install
Terminal=false
Type=Application
Categories=System;PackageManager;
Keywords=package;manager;pacman;flatpak;
EOF"#,
                        ])
                        .description("Installing desktop entry...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            r#"cat > /usr/share/mime/packages/x-alpm-package.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-alpm-package">
    <comment>Arch Linux Package</comment>
    <glob pattern="*.pkg.tar.zst"/>
    <glob pattern="*.pkg.tar.xz"/>
    <glob pattern="*.pkg.tar.gz"/>
  </mime-type>
</mime-info>
EOF"#,
                        ])
                        .description("Installing MIME type definition...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            r#"cat > /usr/share/polkit-1/actions/org.xpackagemanager.policy << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC
 "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <action id="org.xpackagemanager.pkexec">
    <description>Run xPackageManager privileged operations</description>
    <message>Authentication is required to manage packages</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/opt/xpackagemanager/xpackagemanager</annotate>
  </action>
</policyconfig>
EOF"#,
                        ])
                        .description("Installing polkit policy...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "update-desktop-database /usr/share/applications 2>/dev/null || true",
                        ])
                        .description("Updating desktop database...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "update-mime-database /usr/share/mime 2>/dev/null || true",
                        ])
                        .description("Updating MIME database...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("rm")
                        .args(&["-rf", "/tmp/xpm-build"])
                        .description("Cleaning up temporary files...")
                        .build(),
                )
                .build();

            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Install xPackageManager",
            );
        }
    });

    let window_clone = window.clone();
    btn_xpackagemanager_uninstall.connect_clicked(move |_| {
        info!("Servicing: xPackageManager uninstall button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/bin/xpackagemanager"])
                    .description("Removing xPackageManager binary...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-rf", "/opt/xpackagemanager"])
                    .description("Removing application files...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/applications/xpackagemanager.desktop"])
                    .description("Removing desktop entry...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/mime/packages/x-alpm-package.xml"])
                    .description("Removing MIME type...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/polkit-1/actions/org.xpackagemanager.policy"])
                    .description("Removing polkit policy...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("update-desktop-database")
                    .args(&["/usr/share/applications"])
                    .description("Updating desktop database...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "Uninstall xPackageManager",
        );
    });
}
