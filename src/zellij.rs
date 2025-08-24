use std::process::Output;

use crate::{cmd, errors::*, process::*};

pub fn start_session(session: &str) -> Result<Output> {
    cmd!("zellij {session}")
        .run_success()
        .wrap_err("Failed to ")
}

pub fn is_session_open(session: &str) -> Result<bool> {
    let output = cmd!("zellij list-session")
        .run_success()
        .wrap_err(format!("Failed to check if session is up: {session}"))?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line == session))
}

/// Send an input.
pub fn send_input(session: &str, input: &str) -> Result<Output> {
    cmd!("zellij -s {session} action write-chars '{input}'")
        .run_success()
        .wrap_err(format!(
            "Failed to send input to session {session}: {input}"
        ))
}

/// Send an input including a newline.
pub fn send_input_newline(session: &str, input: &str) -> Result<Output> {
    // Send the input
    send_input(session, input).wrap_err(format!(
        "Failed to send input to session {session}: {input}"
    ))?;

    // Send the newline
    cmd!("zellij -s {session} write 10")
        .run_success()
        .wrap_err(format!("Failed to send newline to {session}"))
}

/// Send a Ctrl-c to a session.
pub fn send_ctrl_c(session: &str) -> Result<Output> {
    cmd!("zellij -s {session} action write 3")
        .run_success()
        .wrap_err(format!("Failed to send Ctrl-C to session {session}"))
}
