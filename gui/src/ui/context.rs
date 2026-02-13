//! Application context and UI state management.
//!
//! This module contains the application-wide context and UI component
//! references used for navigation and state management.

use crate::config::user::Config;
use adw::prelude::*;
use gtk4::{Box as GtkBox, Stack, ToggleButton};
use std::cell::RefCell;
use std::rc::Rc;

/// Main application context with UI elements.
#[derive(Clone)]
pub struct AppContext {
    pub ui: UiComponents,
    #[allow(dead_code)]
    pub config: Rc<RefCell<Config>>,
}

impl AppContext {
    /// Create a new application context with UI components.
    pub fn new(ui: UiComponents, config: Rc<RefCell<Config>>) -> Self {
        Self { ui, config }
    }

    /// Navigate to a specific page in the stack.
    pub fn navigate_to_page(&self, page_name: &str) {
        self.ui.stack.set_visible_child_name(page_name);
    }
}

/// UI components grouped by functionality.
#[derive(Clone)]
pub struct UiComponents {
    pub stack: Stack,
    #[allow(dead_code)]
    // Stored for potential future use, currently only used during initialization
    pub tabs_container: GtkBox,
    pub main_split_view: adw::OverlaySplitView,
    pub sidebar_toggle: ToggleButton,
}

impl UiComponents {
    /// Create UI components from individual widgets.
    pub fn new(
        stack: Stack,
        tabs_container: GtkBox,
        main_split_view: adw::OverlaySplitView,
        sidebar_toggle: ToggleButton,
    ) -> Self {
        Self {
            stack,
            tabs_container,
            main_split_view,
            sidebar_toggle,
        }
    }

    /// Configure the sidebar split view with size constraints and toggle binding.
    pub fn configure_sidebar(&self, min_width: i32, max_width: i32) {
        // Set min/max widths (convert i32 to f64)
        self.main_split_view.set_min_sidebar_width(min_width as f64);
        self.main_split_view.set_max_sidebar_width(max_width as f64);

        // Bind toggle button to split view's show-sidebar property
        self.sidebar_toggle
            .bind_property("active", &self.main_split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();

        // Update tooltip based on state
        let toggle = self.sidebar_toggle.clone();
        self.main_split_view
            .connect_show_sidebar_notify(move |split_view| {
                let tooltip = if split_view.shows_sidebar() {
                    "Hide sidebar"
                } else {
                    "Show sidebar"
                };
                toggle.set_tooltip_text(Some(tooltip));
            });
    }
}
