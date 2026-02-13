//! Centralized configuration and constants for the application.
//!
//! This module provides:
//! - **constants**: Application information, paths, links, and UI resources
//! - **env**: Environment variable caching and initialization
//! - **user**: User-configurable settings (TOML-based config)

pub mod constants;
pub mod env;
pub mod user;

// Re-export constants submodules for convenience
pub use constants::app_info;
pub use constants::links;
pub use constants::paths;
pub use constants::resources;
pub use constants::seasonal_debug;
pub use constants::sidebar;
