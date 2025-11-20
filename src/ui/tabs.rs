//! Tab management and navigation functionality.
//!
//! This module handles the creation and management of vertical tabs in the left sidebar,
//! allowing users to switch between different pages in the application stack.

use gtk4::prelude::*;
use gtk4::{Box, Button, Stack};
use log::info;

/// Represents a single tab in the navigation sidebar.
#[derive(Clone)]
pub struct Tab {
    /// Display name of the tab
    #[allow(dead_code)]
    pub label: String,
    /// Stack page name this tab navigates to
    pub page_name: String,
    /// The button widget for this tab
    pub button: Button,
}

impl Tab {
    /// Create a new tab with the given label and page name.
    pub fn new(label: &str, page_name: &str) -> Self {
        let button = Button::builder()
            .label(label)
            .hexpand(true)
            .css_classes(vec!["tab-button".to_string()])
            .build();

        Tab {
            label: label.to_string(),
            page_name: page_name.to_string(),
            button,
        }
    }

    /// Connect this tab's button to navigate to its associated page.
    pub fn connect_navigation(&self, stack: &Stack, tabs_container: &Box) {
        let stack_clone = stack.clone();
        let page_name = self.page_name.clone();
        let button_clone = self.button.clone();
        let tabs_clone = tabs_container.clone();

        self.button.connect_clicked(move |_| {
            info!("Tab clicked: navigating to page '{}'", page_name);
            stack_clone.set_visible_child_name(&page_name);

            // Update active state for all tabs
            update_active_tab(&tabs_clone, &button_clone);
        });
    }
}

/// Set up the tabs navigation system.
///
/// This function creates a set of tabs and adds them to the tabs container.
/// Each tab is connected to navigate to its corresponding stack page.
pub fn setup_tabs(tabs_container: &Box, stack: &Stack) {
    info!("Setting up navigation tabs");

    // Define the tabs to display
    // These correspond to stack page names defined in main.ui
    let tabs_config = vec![
        ("Main Page", "main_page"),
        ("Customization", "customization"),
        ("Gaming Tools", "gaming_tools"),
        ("Containers/VMs", "containers_vms"),
        ("Multimedia Tools", "multimedia_tools"),
        ("Kernel Manager/SCX", "kernel_manager_scx"),
        ("Servicing/System tweaks", "servicing_system_tweaks"),
    ];

    let mut first_button: Option<Button> = None;

    for (label, page_name) in tabs_config {
        let tab = Tab::new(label, page_name);
        tab.connect_navigation(stack, tabs_container);

        if first_button.is_none() {
            first_button = Some(tab.button.clone());
        }

        tabs_container.append(&tab.button);
        info!("Added tab: {} -> page '{}'", label, page_name);
    }

    // Set first tab as active
    if let Some(button) = first_button {
        button.add_css_class("active");
    }
}

/// Add a new tab to the tabs container at runtime.
///
/// # Arguments
/// * `tabs_container` - The container holding all tabs
/// * `stack` - The stack widget to navigate
/// * `label` - Display name for the tab
/// * `page_name` - Name of the stack page to navigate to
#[allow(dead_code)]
pub fn add_tab(tabs_container: &Box, stack: &Stack, label: &str, page_name: &str) {
    let tab = Tab::new(label, page_name);
    tab.connect_navigation(stack, tabs_container);

    tabs_container.append(&tab.button);
    info!("Dynamically added tab: {} -> page '{}'", label, page_name);
}

/// Update which tab is marked as active
fn update_active_tab(tabs_container: &Box, clicked_button: &Button) {
    let mut child = tabs_container.first_child();

    while let Some(widget) = child {
        if let Ok(button) = widget.clone().downcast::<Button>() {
            if button == *clicked_button {
                button.add_css_class("active");
            } else {
                button.remove_css_class("active");
            }
        }
        child = widget.next_sibling();
    }
}

/// Set the active tab by page name.
///
/// Updates the visual state to indicate which tab is currently active.
///
/// # Arguments
/// * `tabs_container` - The container holding all tabs
/// * `page_name` - The page name to mark as active
#[allow(dead_code)]
pub fn set_active_tab(tabs_container: &Box, page_name: &str) {
    let mut child = tabs_container.first_child();

    while let Some(widget) = child {
        if let Ok(button) = widget.clone().downcast::<Button>() {
            // Check if this button's associated page matches
            if let Some(label) = button.label() {
                // Simple heuristic: if the page_name matches the button label in lowercase
                if label.to_lowercase() == page_name.to_lowercase() {
                    button.add_css_class("active-tab");
                } else {
                    button.remove_css_class("active-tab");
                }
            }
        }
        child = widget.next_sibling();
    }
}
