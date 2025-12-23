//! UI utility functions for widget extraction and window access.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};

/// Helper to extract widgets from builder with consistent error handling.
pub fn extract_widget<T: IsA<glib::Object>>(builder: &Builder, name: &str) -> T {
    builder
        .object(name)
        .unwrap_or_else(|| panic!("Failed to get widget with id '{}'", name))
}

/// Get the parent ApplicationWindow from a button widget.
///
/// This helper traverses the widget hierarchy to find the root ApplicationWindow,
/// which is commonly needed for dialogs and other window operations.
pub fn get_window_from_button(button: &Button) -> Option<ApplicationWindow> {
    button
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok())
}
