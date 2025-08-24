use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::errors::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputMode {
    /// Captures stdout and stderr (default)
    Piped,
    /// Inherits parent's stdout and stderr
    Inherited,
}

#[macro_export]
macro_rules! cmd {
    ($( $x:expr ),*) => {
        Cmd::new(format!(
            $($x,)*
        ))
    };
}

/// This is a convenience layer around [std::process::Command].
/// It provides simple exit handling for single Commands.
/// This doesn't work with pipes.
pub struct Cmd {
    cwd: Option<PathBuf>,
    env: HashMap<String, String>,
    command: String,
    output_mode: OutputMode,
}

impl Cmd {
    /// Create a new wrapper with the command that should be executed.
    pub fn new<T: ToString>(command: T) -> Cmd {
        Cmd {
            command: command.to_string(),
            env: HashMap::new(),
            cwd: None,
            output_mode: OutputMode::Piped,
        }
    }

    /// Set the current working directory of the process.
    pub fn cwd(mut self, dir: PathBuf) -> Cmd {
        self.cwd = Some(dir);
        self
    }

    /// Set an environment variable for the process.
    pub fn env<S: ToString, T: ToString>(mut self, key: S, value: T) -> Cmd {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the output mode for the process.
    pub fn io_passthrough(mut self) -> Cmd {
        self.output_mode = OutputMode::Inherited;
        self
    }

    /// Run the command and return the output
    pub fn run(&self) -> Result<std::process::Output> {
        let mut command = Command::new("sh");
        command.arg("-c").arg(&self.command);

        // Configure stdout and stderr based on output mode
        match self.output_mode {
            OutputMode::Piped => {
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
            OutputMode::Inherited => {
                command.stdin(Stdio::inherit());
                command.stdout(Stdio::inherit());
                command.stderr(Stdio::inherit());
            }
        }

        // Set the current working directory.
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }

        // Set environment variables
        for (key, value) in self.env.iter() {
            command.env(key, value);
        }

        debug!("Executing command: {}", self.command);
        if matches!(self.output_mode, OutputMode::Inherited) {
            info!(
                "Running command '{} ...'",
                self.command.split(' ').next().unwrap()
            )
        }

        // Execute the command
        let output = match command.output() {
            Ok(output) => output,
            Err(error) => {
                bail!(
                    "Failed during: {} \nCritical error: {}",
                    &self.command,
                    error
                );
            }
        };

        Ok(output)
    }

    /// A wrapper around `run` that also errors on non-zero exit statuses
    pub fn run_success(&self) -> Result<std::process::Output> {
        let output = self.run()?;

        // Return an error on any non-zero exit codes
        if !output.status.success() {
            bail!(
                "Failed during: {}\nGot non-zero exit code: {:?}",
                &self.command,
                output.status
            );
        }

        Ok(output)
    }
}
