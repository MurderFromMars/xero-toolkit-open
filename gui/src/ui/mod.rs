//! User interface components and functionality.
//!
//! This module contains all UI-related components organized by functionality:
//! - `app`: Application setup and initialization
//! - `context`: Application state and UI components
//! - `navigation`: Tab navigation and sidebar management
//! - `dialogs`: Dialog windows (error, selection, download)
//! - `task_runner`: Command execution with progress UI
//! - `pages`: Page-specific button handlers

pub mod app;
pub mod context;
pub mod dialogs;
pub mod navigation;
pub mod pages;
pub mod task_runner;
pub mod utils;

// Re-export the main entry point
pub use app::setup_application_ui;
