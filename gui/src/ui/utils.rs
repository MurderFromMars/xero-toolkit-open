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

/// Strip ANSI color codes and escape sequences from a string.
///
/// This removes all ANSI escape sequences including:
/// - Color codes (e.g., \x1b[31m, \x1b[0m)
/// - Cursor movement codes (e.g., \x1b[?25l, \x1b[?25h)
/// - Other control sequences
pub fn strip_ansi_codes(text: &str) -> String {
    let bytes = text.as_bytes();
    let stripped = strip_ansi_escapes::strip(bytes);
    String::from_utf8_lossy(&stripped).to_string()
}
