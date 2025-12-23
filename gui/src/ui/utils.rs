//! UI utility functions for widget extraction.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::Builder;

/// Helper to extract widgets from builder with consistent error handling.
pub fn extract_widget<T: IsA<glib::Object>>(builder: &Builder, name: &str) -> T {
    builder
        .object(name)
        .unwrap_or_else(|| panic!("Failed to get widget with id '{}'", name))
}
