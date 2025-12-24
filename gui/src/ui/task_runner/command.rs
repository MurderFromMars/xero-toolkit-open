//! Command types and definitions for task execution.
//!
//! This module provides the core data structures for representing commands
//! and their execution results in the task runner system.

/// Type of command to execute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandType {
    /// Normal command (no special handling)
    Normal,
    /// Command that needs privilege escalation (pkexec)
    Privileged,
    /// AUR helper command (paru/yay)
    Aur,
}

/// Status of a task in the UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending (not started yet)
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Success,
    /// Task failed with error
    Failed,
    /// Task was canceled by user
    Cancelled,
}

/// Result of command execution.
#[derive(Clone, Debug)]
pub enum CommandResult {
    /// Command executed successfully
    Success,
    /// Command failed with optional exit code
    Failure {
        /// Exit code of the command, if available
        exit_code: Option<i32>,
    },
}

/// A command step to be executed by the task runner.
///
/// Commands can be of different types (normal, privileged, AUR) and include
/// the program name, arguments, and a user-facing description.
#[derive(Clone, Debug)]
pub struct Command {
    /// The type of command, determining how it should be executed
    pub command_type: CommandType,
    /// The program/executable to run
    pub program: String,
    /// Command-line arguments to pass to the program
    pub args: Vec<String>,
    /// Human-readable description shown in the UI
    pub description: String,
}

/// Builder for constructing `Command` objects with a fluent API.
///
/// # Examples
///
/// ```no_run
/// use crate::ui::task_runner::Command;
///
/// // Privileged command
/// let cmd = Command::builder()
///     .privileged()
///     .program("bash")
///     .args(&["-c", "echo hello"])
///     .description("Running command")
///     .build();
///
/// // AUR command (program is automatically set)
/// let cmd = Command::builder()
///     .aur()
///     .args(&["-S", "package-name"])
///     .description("Installing package")
///     .build();
///
/// // Normal command
/// let cmd = Command::builder()
///     .normal()
///     .program("flatpak")
///     .args(&["install", "-y", "app.id"])
///     .description("Installing Flatpak app")
///     .build();
/// ```
#[derive(Debug)]
pub struct CommandBuilder {
    command_type: CommandType,
    program: Option<String>,
    args: Vec<String>,
    description: Option<String>,
}

impl CommandBuilder {
    /// Set the program/executable to run.
    ///
    /// For AUR commands, the program is automatically set and this is ignored.
    pub fn program(mut self, program: &str) -> Self {
        self.program = Some(program.to_string());
        self
    }

    /// Set all command-line arguments at once.
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set the human-readable description shown in the UI.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Build the final `Command` object.
    ///
    /// # Panics
    ///
    /// Panics if required fields (program for normal/privileged, description) are missing.
    pub fn build(self) -> Command {
        let program = match self.command_type {
            CommandType::Aur => "aur".to_string(),
            _ => self
                .program
                .expect("program is required for normal and privileged commands"),
        };

        let description = self.description.expect("description is required");

        Command {
            command_type: self.command_type,
            program,
            args: self.args,
            description,
        }
    }
}

impl Command {
    /// Create a new command builder.
    ///
    /// This is the recommended way to construct commands with a fluent API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crate::ui::task_runner::Command;
    ///
    /// let cmd = Command::builder()
    ///     .privileged()
    ///     .program("systemctl")
    ///     .args(&["enable", "--now", "service"])
    ///     .description("Enabling service")
    ///     .build();
    /// ```
    pub fn builder() -> CommandBuilderType {
        CommandBuilderType
    }
}

/// Entry point for the command builder API.
///
/// Start with `Command::builder()` and chain method calls to construct a command.
///
/// # Example
///
/// ```no_run
/// use crate::ui::task_runner::Command;
///
/// // Privileged command
/// let cmd = Command::builder()
///     .privileged()
///     .program("bash")
///     .args(&["-c", "echo hello"])
///     .description("Running command")
///     .build();
///
/// // AUR command
/// let cmd = Command::builder()
///     .aur()
///     .args(&["-S", "package-name"])
///     .description("Installing package")
///     .build();
///
/// // Normal command
/// let cmd = Command::builder()
///     .normal()
///     .program("flatpak")
///     .args(&["install", "-y", "app.id"])
///     .description("Installing Flatpak app")
///     .build();
/// ```
#[derive(Debug)]
pub struct CommandBuilderType;

impl CommandBuilderType {
    /// Create a builder for a normal command (no special handling).
    pub fn normal(self) -> CommandBuilder {
        CommandBuilder {
            command_type: CommandType::Normal,
            program: None,
            args: Vec::new(),
            description: None,
        }
    }

    /// Create a builder for a privileged command (runs through pkexec).
    pub fn privileged(self) -> CommandBuilder {
        CommandBuilder {
            command_type: CommandType::Privileged,
            program: None,
            args: Vec::new(),
            description: None,
        }
    }

    /// Create a builder for an AUR helper command (paru/yay).
    pub fn aur(self) -> CommandBuilder {
        CommandBuilder {
            command_type: CommandType::Aur,
            program: None,
            args: Vec::new(),
            description: None,
        }
    }
}
