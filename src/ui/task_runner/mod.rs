//! Task runner for executing commands with progress UI.
//!
//! This module provides a command execution system with:
//! - Step-by-step execution status with visual progress tracking
//! - Output capture (stdout/stderr) for better error reporting
//! - Cancellation support (waits for current command to finish)
//! - Automatic privilege escalation via pkexec
//! - AUR helper integration (paru/yay)
//!
//! ## Usage
//!
//! ### Using the Builder API (Recommended)
//!
//! ```no_run
//! use crate::ui::task_runner::{run, Command, CommandSequence};
//!
//! // Single command
//! let commands = CommandSequence::new()
//!     .then(Command::builder()
//!         .privileged()
//!         .program("systemctl")
//!         .args(&["enable", "--now", "service"])
//!         .description("Enabling service")
//!         .build())
//!     .build();
//!
//! run(&parent_window, commands, "System Setup");
//!
//! // Multiple commands with builder
//! let commands = CommandSequence::new()
//!     .then(Command::builder()
//!         .aur()
//!         .args(&["-S", "--noconfirm", "package"])
//!         .description("Installing package")
//!         .build())
//!     .then(Command::builder()
//!         .privileged()
//!         .program("systemctl")
//!         .args(&["enable", "--now", "service"])
//!         .description("Enabling service")
//!         .build())
//!     .build();
//!
//! run(&parent_window, commands, "Installation");
//! ```
//!
//!
//! The task runner will:
//! 1. Display a modal dialog showing all commands to be executed
//! 2. Execute each command sequentially, updating UI status as it progresses
//! 3. Capture command output for error reporting
//! 4. Show completion status with appropriate success/failure messages

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

/// Helper for building sequences of commands with a fluent API.
///
/// # Example
///
/// ```no_run
/// use crate::ui::task_runner::{Command, CommandSequence};
///
/// let commands = CommandSequence::new()
///     .then(Command::builder().aur()
///         .args(&["-S", "package"])
///         .description("Installing package")
///         .build())
///     .then(Command::builder().privileged()
///         .program("systemctl")
///         .args(&["enable", "--now", "service"])
///         .description("Enabling service")
///         .build())
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct CommandSequence {
    pub(super) commands: Vec<Command>,
}

impl CommandSequence {
    /// Create a new empty command sequence.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a command to the sequence.
    ///
    /// Can be chained to add multiple commands in order.
    pub fn then(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    /// Build the final command sequence.
    pub fn build(self) -> Self {
        self
    }

    /// Check if the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Resource path for the task list dialog UI file.
pub(super) const TASK_DIALOG_RESOURCE: &str =
    "/xyz/xerolinux/xero-toolkit/ui/dialogs/task_list_dialog.ui";

/// Message displayed when waiting for current command to finish after cancellation.
pub(super) const CANCEL_WAITING_MESSAGE: &str = "Waiting for current command to finish...";

/// Message displayed when operation is cancelled.
pub(super) const CANCELLED_MESSAGE: &str = "Operation cancelled by user";

/// Message displayed when all operations complete successfully.
pub(super) const SUCCESS_MESSAGE: &str = "All operations completed successfully!";

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
/// * `commands` - Command sequence to execute
/// * `title` - Dialog title
///
/// # Example
///
/// ```no_run
/// use crate::ui::task_runner::{Command, CommandSequence, run};
/// use gtk4::Window;
///
/// let commands = CommandSequence::new()
///     .then(Command::builder()
///         .privileged()
///         .program("systemctl")
///         .args(&["enable", "service"])
///         .description("Enabling service")
///         .build())
///     .build();
/// run(&window, commands, "System Setup");
/// ```
pub fn run(parent: &Window, commands: CommandSequence, title: &str) {
    if commands.is_empty() {
        error!("No commands provided");
        return;
    }

    if is_running() {
        warn!("Action already running - ignoring request");
        return;
    }

    ACTION_RUNNING.store(true, Ordering::SeqCst);

    let builder = gtk4::Builder::from_resource(TASK_DIALOG_RESOURCE);

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

    let commands_vec = commands.commands;

    // Create task items for each command
    let mut task_items = Vec::new();
    for (i, cmd) in commands_vec.iter().enumerate() {
        let task_item = TaskItem::new(&cmd.description);
        task_item.set_status(TaskStatus::Pending);
        task_list_container.append(&task_item.container);

        if i < commands_vec.len() - 1 {
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
    let commands = Rc::new(commands_vec);

    // Cancel button handler
    let widgets_clone = widgets.clone();
    let cancelled_clone = cancelled.clone();
    cancel_button.connect_clicked(move |_| {
        *cancelled_clone.borrow_mut() = true;
        widgets_clone.disable_cancel();
        widgets_clone.set_title(CANCEL_WAITING_MESSAGE);
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
