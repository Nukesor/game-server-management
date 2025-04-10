use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use subprocess::CaptureData;

use crate::{cmd, config::Config, process::*};

/// Spawn a new tmux session based on the game name and optional instance.
/// By default, this will use the [Config.game_dir] as cwd, which can be overwritten via the `cwd`
/// parameter.
pub fn start_session(config: &Config, cwd: Option<PathBuf>) -> Result<CaptureData> {
    let cwd = cwd.unwrap_or_else(|| config.game_dir());
    cmd!("tmux new -d -s {}", config.session_name())
        .cwd(cwd)
        .run_success()
        .context(format!("Failed to spawn session {}", config.session_name()))
}

pub fn is_session_open(config: &Config) -> Result<bool> {
    let capture_data = cmd!("tmux has-session -t {}", config.session_name()).run()?;
    Ok(capture_data.exit_status.success())
}

pub fn ensure_session_not_open(config: &Config) -> Result<()> {
    if is_session_open(config)? {
        println!("Session {} is already running", config.session_name());
        std::process::exit(1)
    }

    Ok(())
}

pub fn ensure_session_is_open(config: &Config) -> Result<()> {
    if !is_session_open(config)? {
        println!("Session {} is not running", config.session_name());
        std::process::exit(1)
    }

    Ok(())
}

/// Send an input.
pub fn send_input(config: &Config, input: &str) -> Result<CaptureData> {
    // tmux needs input to be formatted in a special way.
    cmd!("tmux send -t {} '{input}'", config.session_name())
        .run_success()
        .context(format!(
            "Failed to send input to session {}:\n{input}",
            config.session_name()
        ))
}

/// Send an input including a newline.
pub fn send_input_newline(config: &Config, input: &str) -> Result<CaptureData> {
    // Send the input
    send_input(config, input).context(format!(
        "Failed to send input to session {}:\n{input}",
        config.session_name()
    ))?;

    // Send the newline
    cmd!("tmux send -t {} ENTER", config.session_name())
        .run_success()
        .context(format!(
            "Failed to send newline to {}",
            config.session_name()
        ))
}

/// Send a Ctrl-c to a session.
pub fn send_ctrl_c(config: &Config) -> Result<CaptureData> {
    cmd!("tmux send-keys -t {} C-c", config.session_name())
        .run_success()
        .context(format!(
            "Failed to send Ctrl-C to session {}",
            config.session_name()
        ))
}

/// Send an input including a newline.
pub fn send_input_newline_with_env(
    config: &Config,
    input: &str,
    envs: HashMap<&'static str, String>,
) -> Result<CaptureData> {
    // Send the input
    send_input(config, input).context(format!(
        "Failed to send input to session {}: {input}",
        config.session_name()
    ))?;

    // Send the newline
    let mut cmd = cmd!("tmux send -t {} ENTER", config.session_name());
    for (key, value) in envs.iter() {
        cmd = cmd.env(key, value);
    }

    cmd.run_success().context(format!(
        "Failed to send newline to {}",
        config.session_name()
    ))
}
