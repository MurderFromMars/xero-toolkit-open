//! UI widgets for task runner dialog.
//!
//! This module provides the UI components for displaying command execution progress,
//! including task items, status icons, and scroll management.

use super::command::TaskStatus;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Label, ScrolledWindow, Window};

/// Container for all task runner dialog widgets.
pub struct TaskRunnerWidgets {
    pub window: Window,
    pub title_label: Label,
    #[allow(dead_code)]
    pub task_list_container: GtkBox,
    pub scrolled_window: ScrolledWindow,
    pub cancel_button: Button,
    pub close_button: Button,
    pub task_items: Vec<TaskItem>,
}

/// A single task item in the task list.
pub struct TaskItem {
    pub container: GtkBox,
    pub status_icon: Image,
    pub spinner_icon: Image,
}

impl TaskItem {
    /// Create a new task item.
    pub fn new(description: &str) -> Self {
        let container = GtkBox::new(gtk4::Orientation::Horizontal, 12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        let label = Label::new(Some(description));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_wrap(true);

        // Spinner icon for running state
        let spinner_icon = Image::new();
        spinner_icon.set_icon_name(Some("circle-noth-symbolic"));
        spinner_icon.set_pixel_size(24);
        spinner_icon.set_visible(false);
        spinner_icon.add_css_class("spinning");

        // Status icon for success/failure
        let status_icon = Image::new();
        status_icon.set_pixel_size(24);
        status_icon.set_visible(false);

        container.append(&label);
        container.append(&spinner_icon);
        container.append(&status_icon);

        Self {
            container,
            status_icon,
            spinner_icon,
        }
    }

    /// Update the status of this task item.
    pub fn set_status(&self, status: TaskStatus) {
        match status {
            TaskStatus::Pending => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Running => {
                self.spinner_icon.set_visible(true);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Success => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-check"));
                self.status_icon.set_visible(true);
            }
            TaskStatus::Failed => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-xmark"));
                self.status_icon.set_visible(true);
            }
            TaskStatus::Cancelled => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-stop"));
                self.status_icon.set_visible(true);
            }
        }
    }
}

impl TaskRunnerWidgets {
    /// Scroll to a specific task in the list (only if outside visible area).
    ///
    /// This ensures the task at the given index is visible in the scrolled window
    /// by scrolling if necessary, but avoids scrolling if the task is already visible.
    fn scroll_to_task(&self, index: usize) {
        if self.task_items.get(index).is_none() {
            return;
        }

        let vadjustment = self.scrolled_window.vadjustment();
        let viewport_top = vadjustment.value();
        let viewport_bottom = viewport_top + vadjustment.page_size();

        // Estimate task position (assuming roughly equal height per task)
        let total_tasks = self.task_items.len() as f64;
        if total_tasks == 0.0 {
            return;
        }

        let content_height = vadjustment.upper();
        let estimated_task_height = content_height / total_tasks;
        let task_top = (index as f64) * estimated_task_height;
        let task_bottom = task_top + estimated_task_height;

        // Only scroll if task is outside visible viewport
        if task_bottom > viewport_bottom {
            // Task is below viewport - scroll down to show it
            let target = (task_bottom - vadjustment.page_size())
                .max(0.0)
                .min(vadjustment.upper() - vadjustment.page_size());
            vadjustment.set_value(target);
        } else if task_top < viewport_top {
            // Task is above viewport - scroll up to show it
            vadjustment.set_value(task_top.max(0.0));
        }
    }

    /// Update the status of a specific task.
    pub fn update_task_status(&self, index: usize, status: TaskStatus) {
        if let Some(task_item) = self.task_items.get(index) {
            task_item.set_status(status);
            self.scroll_to_task(index);
        }
    }

    /// Set the dialog title.
    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }

    /// Disable the cancel button.
    pub fn disable_cancel(&self) {
        self.cancel_button.set_sensitive(false);
    }

    /// Enable the close button and hide cancel button.
    pub fn enable_close(&self) {
        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
        self.close_button.set_sensitive(true);
    }

    /// Show completion state with a final message.
    ///
    /// Updates the dialog title with the completion message and enables
    /// the close button while hiding the cancel button.
    /// Applies visual styling based on success or failure.
    pub fn show_completion(&self, success: bool, message: &str) {
        self.set_title(message);

        // Add CSS classes to provide visual feedback
        if success {
            // Success styling: make close button prominent
            self.close_button.add_css_class("suggested-action");
            // Remove any error styling from title if present
            self.title_label.remove_css_class("error");
            self.title_label.add_css_class("success");
        } else {
            // Failure styling: remove success styling, add error styling
            self.close_button.remove_css_class("suggested-action");
            self.title_label.remove_css_class("success");
            self.title_label.add_css_class("error");
        }

        self.enable_close();
    }
}
