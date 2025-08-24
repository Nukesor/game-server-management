use std::{collections::HashMap, path::PathBuf};

use subprocess::CaptureData;

use crate::{cmd, errors::*, prelude::GameServer, process::*};

pub trait TmuxServer: GameServer {
    /// Spawn a new tmux session based on the game name and optional instance.
    /// By default, this will use the [Config.game_dir] as cwd, which can be overwritten via the
    /// `cwd` parameter.
    fn start_session(&self, cwd: Option<PathBuf>) -> Result<CaptureData> {
        let cwd = cwd.unwrap_or_else(|| self.config().game_dir());
        cmd!("tmux new -d -s {}", self.session_name())
            .cwd(cwd)
            .run_success()
            .wrap_err(format!("Failed to spawn session {}", self.session_name()))
    }

    fn is_session_open(&self) -> Result<bool> {
        let capture_data = cmd!("tmux has-session -t {}", self.session_name()).run()?;
        Ok(capture_data.exit_status.success())
    }

    fn ensure_session_not_open(&self) -> Result<()> {
        if self.is_session_open()? {
            println!("Session {} is already running", self.session_name());
            std::process::exit(1)
        }

        Ok(())
    }

    fn ensure_session_is_open(&self) -> Result<()> {
        if !self.is_session_open()? {
            println!("Session {} is not running", self.session_name());
            std::process::exit(1)
        }

        Ok(())
    }

    /// Send an input.
    fn send_input(&self, input: &str) -> Result<CaptureData> {
        // tmux needs input to be formatted in a special way.
        cmd!("tmux send -t {} '{input}'", self.session_name())
            .run_success()
            .wrap_err(format!(
                "Failed to send input to session {}:\n{input}",
                self.session_name()
            ))
    }

    /// Send an input including a newline.
    fn send_input_newline(&self, input: &str) -> Result<CaptureData> {
        // Send the input
        self.send_input(input).wrap_err(format!(
            "Failed to send input to session {}:\n{input}",
            self.session_name()
        ))?;

        // Send the newline
        cmd!("tmux send -t {} ENTER", self.session_name())
            .run_success()
            .wrap_err(format!("Failed to send newline to {}", self.session_name()))
    }

    /// Send a Ctrl-c to a session.
    fn send_ctrl_c(&self) -> Result<CaptureData> {
        cmd!("tmux send-keys -t {} C-c", self.session_name())
            .run_success()
            .wrap_err(format!(
                "Failed to send Ctrl-C to session {}",
                self.session_name()
            ))
    }

    /// Send an input including a newline.
    fn send_input_newline_with_env(
        &self,
        input: &str,
        envs: HashMap<&'static str, String>,
    ) -> Result<CaptureData> {
        // Send the input
        self.send_input(input).wrap_err(format!(
            "Failed to send input to session {}: {input}",
            self.session_name()
        ))?;

        // Send the newline
        let mut cmd = cmd!("tmux send -t {} ENTER", self.session_name());
        for (key, value) in envs.iter() {
            cmd = cmd.env(key, value);
        }

        cmd.run_success()
            .wrap_err(format!("Failed to send newline to {}", self.session_name()))
    }
}
