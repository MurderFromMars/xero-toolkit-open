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
use gtk4::glib;
use log::{error, info};
use std::cell::RefCell;
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

                // Only show error in title, not the full error message
                let final_message = format!(
                    "Operation failed at step {} of {}",
                    self.index + 1,
                    self.commands.len()
                );

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

    // Use std::process for real-time output streaming
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use std::sync::Arc;
    use std::thread;

    // Create context for this command
    let context = RunningContext::new(
        widgets.clone(),
        commands.clone(),
        index,
        cancelled.clone(),
        current_process.clone(),
    );

    // Display command header
    widgets.append_command_header(&cmd.description);

    let mut process = Command::new(&program);
    process.args(&args);
    process.stdout(Stdio::piped());
    process.stderr(Stdio::piped());

    let child = match process.spawn() {
        Ok(child) => child,
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

    // Store child process for cancellation
    let child_arc = Arc::new(std::sync::Mutex::new(Some(child)));
    *current_process.borrow_mut() = None; // Clear gio subprocess reference

    // Set up result storage
    use std::sync::Mutex;
    let result_arc: Arc<Mutex<Option<CommandResult>>> = Arc::new(Mutex::new(None));

    // Set up real-time output streaming using channels
    use std::sync::mpsc;
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();

    // Spawn thread to read stdout
    let stdout_handle = if let Some(stdout) = child_arc
        .lock()
        .unwrap()
        .as_mut()
        .and_then(|c| c.stdout.take())
    {
        Some(thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        let _ = stdout_tx.send(text);
                    }
                    Err(_) => {
                        // End of stream or error
                        break;
                    }
                }
            }
        }))
    } else {
        None
    };

    // Spawn thread to read stderr
    let stderr_handle = if let Some(stderr) = child_arc
        .lock()
        .unwrap()
        .as_mut()
        .and_then(|c| c.stderr.take())
    {
        Some(thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        let _ = stderr_tx.send(text);
                    }
                    Err(_) => {
                        // End of stream or error
                        break;
                    }
                }
            }
        }))
    } else {
        None
    };

    // Process output in main thread
    let widgets_stdout = widgets.clone();
    let widgets_stderr = widgets.clone();
    let result_arc_for_output = result_arc.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        // Process stdout
        while let Ok(text) = stdout_rx.try_recv() {
            widgets_stdout.append_timestamped(&format!("{}\n", text), "stdout");
        }
        // Process stderr
        while let Ok(text) = stderr_rx.try_recv() {
            widgets_stderr.append_timestamped(&format!("{}\n", text), "stderr");
        }
        // Stop if result is ready
        if result_arc_for_output.lock().unwrap().is_some() {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });

    // Wait for process to complete in a separate thread
    let result_arc_clone = result_arc.clone();

    thread::spawn(move || {
        // Wait for output threads to finish
        if let Some(handle) = stdout_handle {
            let _ = handle.join();
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        // Wait for process
        let mut child_guard = child_arc.lock().unwrap();
        if let Some(mut child) = child_guard.take() {
            let result = match child.wait() {
                Ok(status) => {
                    if status.success() {
                        CommandResult::Success
                    } else {
                        CommandResult::Failure {
                            exit_code: status.code(),
                            stdout: None, // Already streamed
                            stderr: None, // Already streamed
                        }
                    }
                }
                Err(e) => {
                    error!("Error waiting for process: {}", e);
                    CommandResult::Failure {
                        exit_code: None,
                        stdout: None,
                        stderr: None,
                    }
                }
            };
            *result_arc_clone.lock().unwrap() = Some(result);
        }
    });

    // Check for result in main thread
    let context_clone = context.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        let mut result_guard = result_arc.lock().unwrap();
        if let Some(result) = result_guard.take() {
            context_clone.set_exit_result(result);
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
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
