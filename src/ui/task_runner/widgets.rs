//! UI widgets for task runner dialog.
//!
//! This module provides the UI components for displaying command execution progress,
//! including task items, status icons, and scroll management.

use super::command::TaskStatus;
use adw::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Image, Label, ScrolledWindow, TextBuffer, TextView, ToggleButton, Window,
};

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
    pub sidebar_toggle: ToggleButton,
    pub split_view: adw::OverlaySplitView,
    pub output_text_view: TextView,
    pub output_text_buffer: TextBuffer,
}

impl TaskRunnerWidgets {
    /// Create a new TaskRunnerWidgets instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        window: Window,
        title_label: Label,
        task_list_container: GtkBox,
        scrolled_window: ScrolledWindow,
        cancel_button: Button,
        close_button: Button,
        task_items: Vec<TaskItem>,
        sidebar_toggle: ToggleButton,
        split_view: adw::OverlaySplitView,
        output_text_view: TextView,
        output_text_buffer: TextBuffer,
    ) -> Self {
        let widgets = Self {
            window,
            title_label,
            task_list_container,
            scrolled_window,
            cancel_button,
            close_button,
            task_items,
            sidebar_toggle,
            split_view,
            output_text_view,
            output_text_buffer,
        };

        // Set up color tags for output
        widgets.setup_color_tags();

        widgets
    }

    /// Set up color tags for styled output.
    fn setup_color_tags(&self) {
        use gtk4::TextTag;

        let tag_table = self.output_text_buffer.tag_table();

        // Header tag (blue)
        let header_tag = TextTag::new(Some("header"));
        header_tag.set_property("foreground", "rgb(100, 149, 237)");
        header_tag.set_property("weight", 700); // Bold
        tag_table.add(&header_tag);

        // Timestamp tag (gray)
        let timestamp_tag = TextTag::new(Some("timestamp"));
        timestamp_tag.set_property("foreground", "rgb(128, 128, 128)");
        tag_table.add(&timestamp_tag);

        // Stdout tag (green)
        let stdout_tag = TextTag::new(Some("stdout"));
        stdout_tag.set_property("foreground", "rgb(46, 204, 113)");
        tag_table.add(&stdout_tag);

        // Stderr tag (orange/red)
        let stderr_tag = TextTag::new(Some("stderr"));
        stderr_tag.set_property("foreground", "rgb(255, 140, 0)");
        tag_table.add(&stderr_tag);

        // Error tag (red)
        let error_tag = TextTag::new(Some("error"));
        error_tag.set_property("foreground", "rgb(231, 76, 60)");
        error_tag.set_property("weight", 700);
        tag_table.add(&error_tag);
    }

    /// Bind the sidebar toggle button to the split view.
    pub fn setup_sidebar_toggle(&self) {
        // Bind toggle button's active state to split view's show-sidebar
        self.sidebar_toggle
            .bind_property("active", &self.split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();

        // Update tooltip based on state
        let toggle = self.sidebar_toggle.clone();
        self.split_view
            .connect_show_sidebar_notify(move |split_view| {
                let tooltip = if split_view.shows_sidebar() {
                    "Hide command output"
                } else {
                    "Show command output"
                };
                toggle.set_tooltip_text(Some(tooltip));
            });
    }
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
    fn scroll_to_task(&self, index: usize) {
        if self.task_items.get(index).is_none() {
            return;
        }

        let vadjustment = self.scrolled_window.vadjustment();
        let viewport_top = vadjustment.value();
        let viewport_bottom = viewport_top + vadjustment.page_size();

        let total_tasks = self.task_items.len() as f64;
        if total_tasks == 0.0 {
            return;
        }

        let content_height = vadjustment.upper();
        let estimated_task_height = content_height / total_tasks;
        let task_top = (index as f64) * estimated_task_height;
        let task_bottom = task_top + estimated_task_height;

        if task_bottom > viewport_bottom {
            let target = (task_bottom - vadjustment.page_size())
                .max(0.0)
                .min(vadjustment.upper() - vadjustment.page_size());
            vadjustment.set_value(target);
        } else if task_top < viewport_top {
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
    pub fn show_completion(&self, success: bool, message: &str) {
        self.set_title(message);

        if success {
            self.close_button.add_css_class("suggested-action");
            self.title_label.remove_css_class("error");
            self.title_label.add_css_class("success");
        } else {
            self.close_button.remove_css_class("suggested-action");
            self.title_label.remove_css_class("success");
            self.title_label.add_css_class("error");
        }

        self.enable_close();
    }

    /// Clear the output buffer.
    #[allow(dead_code)] // Useful API method for future use
    pub fn clear_output(&self) {
        self.output_text_buffer.set_text("");
    }

    /// Append text with a specific color tag.
    pub fn append_colored(&self, text: &str, tag_name: &str) {
        // Get start position before insertion
        let start_offset = self.output_text_buffer.end_iter().offset();

        // Insert text
        let mut end = self.output_text_buffer.end_iter();
        self.output_text_buffer.insert(&mut end, text);

        // Get fresh iterators after insertion
        let start = self.output_text_buffer.iter_at_offset(start_offset);
        let end_fresh = self.output_text_buffer.end_iter();

        // Apply tag
        if let Some(tag) = self.output_text_buffer.tag_table().lookup(tag_name) {
            self.output_text_buffer.apply_tag(&tag, &start, &end_fresh);
        }
        self.scroll_to_bottom();
    }

    /// Append text with timestamp and color tag.
    pub fn append_timestamped(&self, text: &str, tag_name: &str) {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| {
                let secs = d.as_secs();
                let hours = (secs / 3600) % 24;
                let minutes = (secs / 60) % 60;
                let seconds = secs % 60;
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            })
            .unwrap_or_else(|_| "??:??:??".to_string());

        // Get start position (character offset) before insertion
        let start_offset = self.output_text_buffer.end_iter().offset();

        // Insert timestamp and text together
        let timestamp_text = format!("[{}] ", timestamp);
        let full_text = format!("{}{}", timestamp_text, text);
        let mut end = self.output_text_buffer.end_iter();
        self.output_text_buffer.insert(&mut end, &full_text);

        // Get fresh iterators after insertion
        let timestamp_start = self.output_text_buffer.iter_at_offset(start_offset);
        let timestamp_end = self
            .output_text_buffer
            .iter_at_offset(start_offset + timestamp_text.len() as i32);
        let text_start = timestamp_end.clone();
        let text_end = self.output_text_buffer.end_iter();

        // Apply timestamp tag
        if let Some(tag) = self.output_text_buffer.tag_table().lookup("timestamp") {
            self.output_text_buffer
                .apply_tag(&tag, &timestamp_start, &timestamp_end);
        }

        // Apply color tag
        if let Some(tag) = self.output_text_buffer.tag_table().lookup(tag_name) {
            self.output_text_buffer
                .apply_tag(&tag, &text_start, &text_end);
        }

        self.scroll_to_bottom();
    }

    /// Append a command header.
    pub fn append_command_header(&self, description: &str) {
        let header = format!("\n=== {} ===\n", description);
        self.append_colored(&header, "header");
    }

    /// Scroll output view to bottom.
    fn scroll_to_bottom(&self) {
        let mut end = self.output_text_buffer.end_iter();
        let _ = self
            .output_text_view
            .scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
    }

    /// Toggle the sidebar visibility.
    #[allow(dead_code)] // Useful API method for future use
    pub fn toggle_sidebar(&self) {
        self.sidebar_toggle
            .set_active(!self.sidebar_toggle.is_active());
    }

    /// Initialize sidebar to collapsed state.
    pub fn init_sidebar_collapsed(&self) {
        self.sidebar_toggle.set_active(false);
        self.split_view.set_show_sidebar(false);
    }
}
