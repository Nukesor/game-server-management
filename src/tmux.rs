use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use subprocess::CaptureData;

use crate::cmd;
use crate::process::*;

pub fn start_session(session: &str, cwd: PathBuf) -> Result<CaptureData> {
    cmd!("tmux new -d -s {session}")
        .cwd(cwd)
        .run_success()
        .context(format!("Failed to spawn session {session}"))
}

pub fn is_session_open(session: &str) -> Result<bool> {
    let capture_data = cmd!("tmux has-session -t {session}").run()?;
    Ok(capture_data.exit_status.success())
}

/// Send an input.
pub fn send_input(session: &str, input: &str) -> Result<CaptureData> {
    // tmux needs input to be formatted in a special way.
    let formatted_input = input.replace(" ", " SPACE ");
    cmd!("tmux send -t {session} {formatted_input}")
        .run_success()
        .context(format!(
            "Failed to send input to session {session}: {input}"
        ))
}

/// Send an input including a newline.
pub fn send_input_newline(session: &str, input: &str) -> Result<CaptureData> {
    // Send the input
    send_input(session, input).context(format!(
        "Failed to send input to session {session}: {input}"
    ))?;

    // Send the newline
    cmd!("tmux send -t {session} ENTER")
        .run_success()
        .context(format!("Failed to send newline to {session}"))
}

/// Send a Ctrl-c to a session.
pub fn send_ctrl_c(session: &str) -> Result<CaptureData> {
    cmd!("tmux send-keys -t {session} C-c")
        .run_success()
        .context(format!("Failed to send Ctrl-C to session {session}"))
}

/// Send an input including a newline.
pub fn send_input_newline_with_env(
    session: &str,
    input: &str,
    envs: HashMap<&'static str, String>,
) -> Result<CaptureData> {
    // Send the input
    send_input(session, input).context(format!(
        "Failed to send input to session {session}: {input}"
    ))?;

    // Send the newline
    let mut cmd = cmd!("tmux send -t {session} ENTER");
    for (key, value) in envs.iter() {
        cmd = cmd.env(key, value);
    }

    cmd.run_success()
        .context(format!("Failed to send newline to {session}"))
}
