//! Task runner for executing commands with progress UI.
//!
//! This module provides a command execution system with:
//! - Step-by-step execution status
//! - Progress tracking with task list
//! - Cancellation support
//! - Privilege escalation handling
//! - AUR helper integration
//!
//! ## Usage
//!
//! ```no_run
//! use crate::ui::task_runner::{run, Command};
//!
//! let commands = vec![
//!     Command::privileged("pacman", &["-Syu"], "System update"),
//!     Command::aur(&["-S", "package-name"], "Install AUR package"),
//! ];
//!
//! run(&parent_window, commands, "Installation");
//! ```

mod command;
mod executor;
mod widgets;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Button, Label, Separator, Window};
use log::{error, warn};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

// Re-export public API
pub use command::{Command, TaskStatus};

use executor::execute_commands;
use widgets::{TaskItem, TaskRunnerWidgets};

/// Global flag to track if an action is currently running.
static ACTION_RUNNING: AtomicBool = AtomicBool::new(false);

/// Check if an action is currently running.
pub fn is_running() -> bool {
    ACTION_RUNNING.load(Ordering::SeqCst)
}

/// Run commands with a progress dialog.
///
/// Displays a modal dialog showing command execution progress with:
/// - Task list indicating current step status
/// - Cancel and close buttons
/// - Auto-scroll to current task
///
/// # Arguments
///
/// * `parent` - Parent window for the dialog
/// * `commands` - Vector of commands to execute
/// * `title` - Dialog title
pub fn run(
    parent: &Window,
    commands: Vec<Command>,
    title: &str,
) {
    if commands.is_empty() {
        error!("No commands provided");
        return;
    }

    if is_running() {
        warn!("Action already running - ignoring request");
        return;
    }

    ACTION_RUNNING.store(true, Ordering::SeqCst);

    let builder =
        gtk4::Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/dialogs/task_list_dialog.ui");

    let window: Window = builder
        .object("task_window")
        .expect("Failed to get task_window");
    let title_label: Label = builder
        .object("task_title")
        .expect("Failed to get task_title");
    let task_list_container: gtk4::Box = builder
        .object("task_list_container")
        .expect("Failed to get task_list_container");
    let scrolled_window: gtk4::ScrolledWindow = builder
        .object("task_scrolled_window")
        .expect("Failed to get task_scrolled_window");
    let cancel_button: Button = builder
        .object("cancel_button")
        .expect("Failed to get cancel_button");
    let close_button: Button = builder
        .object("close_button")
        .expect("Failed to get close_button");

    window.set_transient_for(Some(parent));
    window.set_title(Some(title));

    // Create task items for each command
    let mut task_items = Vec::new();
    for (i, cmd) in commands.iter().enumerate() {
        let task_item = TaskItem::new(&cmd.description);
        task_item.set_status(TaskStatus::Pending);
        task_list_container.append(&task_item.container);

        if i < commands.len() - 1 {
            let sep = Separator::new(gtk4::Orientation::Horizontal);
            task_list_container.append(&sep);
        }
        task_items.push(task_item);
    }

    let widgets = Rc::new(TaskRunnerWidgets {
        window: window.clone(),
        title_label,
        task_list_container,
        scrolled_window,
        cancel_button: cancel_button.clone(),
        close_button: close_button.clone(),
        task_items,
    });

    let cancelled = Rc::new(RefCell::new(false));
    let current_process = Rc::new(RefCell::new(None::<gtk4::gio::Subprocess>));
    let commands = Rc::new(commands);

    // Cancel button handler
    let widgets_clone = widgets.clone();
    let cancelled_clone = cancelled.clone();
    cancel_button.connect_clicked(move |_| {
        *cancelled_clone.borrow_mut() = true;
        widgets_clone.disable_cancel();
        widgets_clone.set_title("Waiting for current command to finish...");
    });

    // Close button handler
    let widgets_clone = widgets.clone();
    close_button.connect_clicked(move |_| {
        widgets_clone.window.close();
    });

    // Window close handler
    let cancelled_clone = cancelled.clone();
    window.connect_close_request(move |_| {
        ACTION_RUNNING.store(false, Ordering::SeqCst);
        *cancelled_clone.borrow_mut() = true;
        glib::Propagation::Proceed
    });

    window.present();

    // Start executing commands
    execute_commands(widgets, commands, 0, cancelled, current_process);
}
