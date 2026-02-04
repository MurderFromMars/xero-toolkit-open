# 🛠️ CyberXero Toolkit - A Fork of the XeroLinux Toolkit

A GTK4 GUI application for managing system tools, configurations, and customizations on **any Arch-based distribution**.

> **Fork Info:** I previously collaborated with DarkXero and have always appreciated the quality of the XeroLinux project. I wanted a version that was minimal enough to use as my daily system, but after discussing the idea with him, it became clear that he did not plan to create a minimal edition or make the toolkit available outside the official distribution. Because of that, I decided to take on the work myself and bring XeroLinux features to a minimal Arch installation.
This fork fulfills that goal by providing an installation process that removes the distribution check and introduces a few additional features described below.
---
## 🎯 What It Does

This tool lets you easily manage and customize your Arch-based system through a clean, modern interface:

* **Update your system** with a single click
* **Install package managers** - Octopi, Bauh, Warehouse, Flatseal, and more
* **Set up drivers** - GPU drivers (NVIDIA, AMD), Tailscale VPN, ASUS ROG tools
* **Configure gaming** - Steam with dependencies, Lutris, Heroic, Bottles, Gamescope, Falcond
* **Customize your desktop** - ZSH setup, GRUB themes, Plymouth, desktop themes
* **Manage containers & VMs** - Docker, Podman, VirtualBox, DistroBox, KVM/QEMU
* **Install multimedia tools** - OBS Studio, Jellyfin, and more
* **Service your system** - Clear caches, fix keyrings, update mirrors, add third-party repos

## 💻 Supported Distributions

Any **Arch-based** distribution:
- Arch Linux
- EndeavourOS
- Manjaro
- CachyOS
- Garuda Linux
- ArcoLinux
- And others...

## ⚙️ Requirements

- **AUR Helper** - Paru or Yay (required for most features)
- **Flatpak** - optional but recommended

## 📦 Installation

**One-liner:**
```sh
rm -rf /tmp/xero-toolkit-open && git clone https://github.com/MurderFromMars/CyberXero-Toolkit.git /tmp/xero-toolkit-open && sh /tmp/xero-toolkit-open/install.sh && rm -rf /tmp/xero-toolkit-open
```

**Manual:**
```bash
git clone https://github.com/MurderFromMars/xero-toolkit-open.git
cd xero-toolkit-open
./install.sh
```

The installer will:
1. Install build dependencies via pacman
2. Patch the XeroLinux distribution check
3. Build from source using Cargo
4. Install to `/opt/xero-toolkit`
5. Create desktop entry and icon

## 🗑️ Uninstallation

```bash
cd xero-toolkit-open
./uninstall.sh
```

Or manually:
```bash
sudo rm -rf /opt/xero-toolkit
sudo rm -f /usr/bin/xero-toolkit
sudo rm -f /usr/share/applications/xero-toolkit.desktop
sudo rm -f /usr/share/icons/hicolor/scalable/apps/xero-toolkit.png
```

## 🔧 Build Dependencies

Installed automatically by the installer:
- `rust` & `cargo`
- `pkgconf`
- `gtk4`
- `glib2`
- `libadwaita`
- `vte4`
- `flatpak`
- `polkit`
- `scx-scheds`

## ✨ Changes from Original

### Distribution Freedom
- Removed XeroLinux distribution check - works on any Arch-based distro
- Added `install.sh` for easy building from source
- Added `uninstall.sh` for clean removal

### New Features: Third-Party Repository Installation
Added buttons in the **Servicing / System Tweaks** page to easily add popular Arch repositories:

- **Install CachyOS Repos** - Adds the [CachyOS](https://cachyos.org/) repositories, providing access to performance-optimized packages, kernels, and tools like Falcond
- **Install Chaotic-AUR** - Adds the [Chaotic-AUR](https://aur.chaotic.cx/) repository, providing pre-built AUR packages for faster installation
- **Add XeroLinux Repo** - Providing access to the full suite of features of XeroLinux.
- **Imported Biometrics and fixed xero-howdy-qt** in their most recent updates, they added the ability to use facial recognition ith howdy on a QT frontend. *this was designed in such a way that it fails to install/build outside of Xerolinux*   i solved this problem by replacng the deprecated howdy listed with one that actually builds correctly, rewriting the logic to build xero-howdy-qt from source with these modiified dependencies 
### Smart Package Installation
- **Falcond Gaming Utility** - Now intelligently checks if packages are available in your configured repos (e.g., CachyOS) before falling back to AUR
  - Installs `falcond`, `falcond-gui`, `falcond-profiles`, and `tuned-ppd`
  - Automatically uses pacman for repo packages, AUR helper only when needed

### Rebranding
The fork has been lightly rebranded to reflect its enhanced/jailbroken status:

- **Updated About Dialog** - Reflects the fork's origin and enhancements
- **Modified Links** - Discord and YouTube links updated (configurable in `gui/src/config.rs`)
- **Logo** - changed to a more appropriate Arch logo 

### Updates
imported biometrics updates from upstream development branch, including support for xero-howdy-qt this required me completely rewriting installation functionality to build the package from source, using howdy-git instead of the fundamentally broken howdy-bin 



## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Credits

- Original [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) by [synsejse](https://github.com/synsejse)
- [XeroLinux](https://xerolinux.xyz/) team
- [CachyOS](https://cachyos.org/) for their optimized repositories
- [Chaotic-AUR](https://aur.chaotic.cx/) for pre-built AUR packages
