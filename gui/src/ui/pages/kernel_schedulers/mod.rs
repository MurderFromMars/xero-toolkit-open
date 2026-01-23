//! Kernel & Schedulers page with subtabs for Kernel Manager and SCX Scheduler.
//!
//! This module provides a unified page with two subtabs:
//! - Kernel Manager: Install/remove kernels and headers
//! - SCX Scheduler: Manage sched-ext BPF CPU schedulers

pub mod kernel_manager_tab;
pub mod scheduler_tab;

use gtk4::{ApplicationWindow, Builder};
use log::info;

/// Set up all handlers for the kernel & schedulers page with subtabs.
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder, window: &ApplicationWindow) {
    info!("Setting up Kernel & Schedulers page with subtabs");

    // Setup handlers for both subtabs
    kernel_manager_tab::setup_handlers(page_builder, main_builder, window);
    scheduler_tab::setup_handlers(page_builder, main_builder, window);

    info!("Kernel & Schedulers page handlers initialized");
}
