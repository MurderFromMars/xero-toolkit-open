//! Command execution logic and running context management.
//!
//! This module handles the actual execution of commands, including:
//! - Process spawning and management
//! - Output capture (stdout/stderr)
//! - Error handling and result processing
//! - Command resolution (privilege escalation, AUR helpers)

use super::command::{Command, CommandResult, CommandType, TaskStatus};
use super::widgets::TaskRunnerWidgets;
use crate::core;
use gtk4::gio;
use log::{error, info};
use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

/// Context for a running command execution.
pub struct RunningContext {
    pub widgets: Rc<TaskRunnerWidgets>,
    pub commands: Rc<Vec<Command>>,
    pub index: usize,
    pub cancelled: Rc<RefCell<bool>>,
    pub current_process: Rc<RefCell<Option<gio::Subprocess>>>,
    exit_result: RefCell<Option<CommandResult>>,
}

impl RunningContext {
    /// Create a new running command context.
    pub fn new(
        widgets: Rc<TaskRunnerWidgets>,
        commands: Rc<Vec<Command>>,
        index: usize,
        cancelled: Rc<RefCell<bool>>,
        current_process: Rc<RefCell<Option<gio::Subprocess>>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            widgets,
            commands,
            index,
            cancelled,
            current_process,
            exit_result: RefCell::new(None),
        })
    }

    /// Set the exit result for the current command.
    pub fn set_exit_result(self: &Rc<Self>, result: CommandResult) {
        *self.exit_result.borrow_mut() = Some(result);
        self.try_finalize();
    }

    /// Try to finalize the current command.
    fn try_finalize(self: &Rc<Self>) {
        let result = {
            let mut exit_result = self.exit_result.borrow_mut();
            exit_result.take()
        };

        let Some(result) = result else {
            return;
        };

        // Clear current process
        self.current_process.borrow_mut().take();

        // Check if cancelled
        if *self.cancelled.borrow() {
            // Mark the current task as cancelled
            self.widgets
                .update_task_status(self.index, TaskStatus::Cancelled);
            finalize_execution(&self.widgets, false, super::CANCELLED_MESSAGE);
            return;
        }

        // Handle result normally
        match result {
            CommandResult::Success => {
                self.widgets
                    .update_task_status(self.index, TaskStatus::Success);
                execute_commands(
                    self.widgets.clone(),
                    self.commands.clone(),
                    self.index + 1,
                    self.cancelled.clone(),
                    self.current_process.clone(),
                );
            }
            CommandResult::Failure { .. } => {
                self.widgets
                    .update_task_status(self.index, TaskStatus::Failed);

                let error_msg = result.error_message();
                let final_message = if error_msg.is_empty() || error_msg == "No output captured" {
                    format!(
                        "Operation failed at step {} of {}",
                        self.index + 1,
                        self.commands.len()
                    )
                } else {
                    // Truncate very long error messages for display
                    let max_length = 500;
                    let truncated = if error_msg.len() > max_length {
                        format!("{}...", &error_msg[..max_length])
                    } else {
                        error_msg.clone()
                    };
                    format!(
                        "Operation failed at step {} of {}:\n{}",
                        self.index + 1,
                        self.commands.len(),
                        truncated
                    )
                };

                finalize_execution(&self.widgets, false, &final_message);
            }
        }
    }
}

/// Execute a sequence of commands.
pub fn execute_commands(
    widgets: Rc<TaskRunnerWidgets>,
    commands: Rc<Vec<Command>>,
    index: usize,
    cancelled: Rc<RefCell<bool>>,
    current_process: Rc<RefCell<Option<gio::Subprocess>>>,
) {
    if *cancelled.borrow() {
        // If there's a current task being processed, mark it as cancelled
        if index < commands.len() {
            widgets.update_task_status(index, TaskStatus::Cancelled);
        }
        finalize_execution(&widgets, false, super::CANCELLED_MESSAGE);
        return;
    }

    if index >= commands.len() {
        finalize_execution(&widgets, true, super::SUCCESS_MESSAGE);
        return;
    }

    let cmd = &commands[index];

    // Mark current task as running
    widgets.update_task_status(index, TaskStatus::Running);
    widgets.set_title(&cmd.description);

    let (program, args) = match resolve_command(cmd) {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to prepare command: {}", err);
            widgets.update_task_status(index, TaskStatus::Failed);
            finalize_execution(
                &widgets,
                false,
                &format!("Failed to prepare command: {}", err),
            );
            return;
        }
    };

    info!("Executing: {} {:?}", program, args);

    let mut argv: Vec<OsString> = Vec::with_capacity(1 + args.len());
    argv.push(OsString::from(program.clone()));
    for arg in &args {
        argv.push(OsString::from(arg));
    }
    let argv_refs: Vec<&std::ffi::OsStr> = argv.iter().map(|s| s.as_os_str()).collect();

    // Capture stdout and stderr for better error reporting
    let flags = gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_PIPE;
    let subprocess = match gio::Subprocess::newv(&argv_refs, flags) {
        Ok(proc) => proc,
        Err(err) => {
            error!("Failed to start command: {}", err);
            widgets.update_task_status(index, TaskStatus::Failed);
            finalize_execution(
                &widgets,
                false,
                &format!("Failed to start operation: {}", err),
            );
            return;
        }
    };

    *current_process.borrow_mut() = Some(subprocess.clone());

    let context = RunningContext::new(
        widgets.clone(),
        commands.clone(),
        index,
        cancelled.clone(),
        current_process.clone(),
    );

    // Capture output and check exit status
    // communicate_utf8_async waits for the process to complete, so we can check exit status after
    let wait_context = context.clone();
    let wait_subprocess_clone = subprocess.clone();
    subprocess.communicate_utf8_async(None, None::<&gio::Cancellable>, move |result| {
        let (stdout, stderr) = match result {
            Ok((stdout_opt, stderr_opt)) => (
                stdout_opt.map(|s| s.to_string()),
                stderr_opt.map(|s| s.to_string()),
            ),
            Err(err) => {
                error!("Failed to communicate with process: {}", err);
                (None, None)
            }
        };

        // Check if process was successful using the cloned reference
        // communicate_utf8_async waits for completion, so exit status is available
        if wait_subprocess_clone.is_successful() {
            wait_context.set_exit_result(CommandResult::Success);
        } else {
            let exit_code = wait_subprocess_clone.exit_status();
            wait_context.set_exit_result(CommandResult::Failure {
                exit_code: Some(exit_code),
                stdout,
                stderr,
            });
        }
    });
}

/// Resolve command with proper privilege escalation and AUR helpers.
///
/// Converts a `Command` into the actual program and arguments that should be executed,
/// handling privilege escalation (pkexec) and AUR helper detection.
///
/// # Returns
///
/// A tuple of `(program, args)` where `program` is the executable to run
/// and `args` are the arguments to pass to it.
///
/// # Errors
///
/// Returns an error if the AUR helper is required but not available.
fn resolve_command(command: &Command) -> Result<(String, Vec<String>), String> {
    match command.command_type {
        CommandType::Normal => Ok((command.program.clone(), command.args.clone())),
        CommandType::Privileged => {
            let mut args = Vec::with_capacity(command.args.len() + 1);
            args.push(command.program.clone());
            args.extend(command.args.clone());
            Ok(("pkexec".to_string(), args))
        }
        CommandType::Aur => {
            let helper = core::aur_helper()
                .ok_or_else(|| "AUR helper not available (paru or yay required)".to_string())?;
            let mut args = Vec::with_capacity(command.args.len() + 2);

            args.push("--sudo".to_string());
            args.push("pkexec".to_string());
            args.extend(command.args.clone());
            Ok((helper.to_string(), args))
        }
    }
}

/// Finalize dialog with success or failure message.
pub fn finalize_execution(widgets: &TaskRunnerWidgets, success: bool, message: &str) {
    use std::sync::atomic::Ordering;
    super::ACTION_RUNNING.store(false, Ordering::SeqCst);

    widgets.show_completion(success, message);
}
