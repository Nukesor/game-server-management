use anyhow::{Context, Result};
use subprocess::CaptureData;

use crate::cmd;
use crate::process::*;

pub fn start_session(session: &str) -> Result<CaptureData> {
    cmd!("zellij {session}").run_success().context("Failed to ")
}

pub fn is_session_open(session: &str) -> Result<bool> {
    let output = cmd!("zellij list-session")
        .run_success()
        .context(format!("Failed to check if session is up: {session}"))?;

    Ok(output.stdout_str().lines().any(|line| line == session))
}

/// Send an input.
pub fn send_input(session: &str, input: &str) -> Result<CaptureData> {
    cmd!("zellij -s {session} action write-chars '{input}'")
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
    cmd!("zellij -s {session} write 10")
        .run_success()
        .context(format!("Failed to send newline to {session}"))
}

/// Send a Ctrl-c to a session.
pub fn send_ctrl_c(session: &str) -> Result<CaptureData> {
    cmd!("zellij -s {session} action write 3")
        .run_success()
        .context(format!("Failed to send Ctrl-C to session {session}"))
}
