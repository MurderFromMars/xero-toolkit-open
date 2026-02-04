//! Page-specific button handlers and logic.
//!
//! This module organizes button handlers by page:
//! - `main_page`: System update, package managers
//! - `drivers`: GPU drivers, Tailscale, ASUS ROG tools
//! - `gaming_tools`: Steam, controllers, game launchers
//! - `gamescope`: Gamescope command generator
//! - `containers_vms`: Docker, Podman, VirtualBox, KVM
//! - `customization`: ZSH, themes, wallpapers
//! - `kernel_schedulers`: Kernel Manager and SCX Scheduler (with subtabs)
//! - `servicing`: System fixes and maintenance
//! - `biometrics`: Fingerprint and facial recognition setup

pub mod biometrics;
pub mod containers_vms;
pub mod customization;
pub mod drivers;
pub mod gamescope;
pub mod gaming_tools;
pub mod kernel_schedulers;
pub mod main_page;
pub mod servicing;
