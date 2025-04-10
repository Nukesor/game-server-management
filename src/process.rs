use std::{collections::HashMap, path::PathBuf};

use anyhow::{Result, bail};
use subprocess::{CaptureData, Exec};

#[macro_export]
macro_rules! cmd {
    ($( $x:expr ),*) => {
        Cmd::new(format!(
            $($x,)*
        ))
    };
}

/// This is a convenience layer around [Subprocess's Exec](subprocess.Exec).
/// It provides simple exit handling for single Commands.
/// This doesn't work with pipes.
pub struct Cmd {
    cwd: Option<PathBuf>,
    env: HashMap<String, String>,
    command: String,
}

impl Cmd {
    /// Create a new wrapper with the command that should be executed.
    pub fn new<T: ToString>(command: T) -> Cmd {
        Cmd {
            command: command.to_string(),
            env: HashMap::new(),
            cwd: None,
        }
    }

    /// Set the current working directory of the process.
    pub fn cwd(mut self, dir: PathBuf) -> Cmd {
        self.cwd = Some(dir);

        self
    }

    /// Set the current working directory of the process.
    pub fn env<S: ToString, T: ToString>(mut self, key: S, value: T) -> Cmd {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Run the command and return the exit status
    pub fn run(&self) -> Result<CaptureData> {
        let mut exec = Exec::shell(&self.command);

        // Set the current working directory.
        if let Some(cwd) = &self.cwd {
            exec = exec.cwd(cwd);
        }

        for (key, value) in self.env.iter() {
            exec = exec.env(key, value);
        }

        // Check if there are any critical errors.
        let capture_data = match exec.capture() {
            Ok(exit_status) => exit_status,
            Err(error) => {
                bail!(
                    "Failed during: {} \nCritical error: {}",
                    &self.command,
                    error
                );
            }
        };

        Ok(capture_data)
    }

    /// A wrapper around `run` that also errors on non-zero exit statuses
    pub fn run_success(&self) -> Result<CaptureData> {
        let capture_data = self.run()?;

        // Return an error on any non-1 exit codes
        if !capture_data.exit_status.success() {
            bail!(
                "Failed during: {}\nGot non-zero exit code: {:?}",
                &self.command,
                capture_data.exit_status
            );
        }

        Ok(capture_data)
    }
}
