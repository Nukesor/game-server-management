use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use utils::config::Config;
use utils::process::*;
use utils::tmux::*;
use utils::{cmd, sleep_seconds};

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Parser, Debug)]
#[clap(
    name = "Unturned",
    about = "A small binary to manage my Unturned server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which creates the server dir from a given config.
fn unturned_dir(config: &Config) -> PathBuf {
    config.game_files().join("unturned")
}

const GAME_NAME: &'static str = "unturned";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config),
        SubCommand::Shutdown => shutdown(),
        SubCommand::Update => update(&config),
    }
}

fn startup(config: &Config) -> Result<()> {
    // Don't start the server if the session is already running.
    if is_session_open(GAME_NAME)? {
        println!("Instance unturned already running");
        return Ok(());
    }

    // Create a new session for this instance
    start_session(GAME_NAME, unturned_dir(config))?;

    let server_command = "./ServerHelper.sh +InternetServer/Jarvis";
    send_input_newline(GAME_NAME, server_command)?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    if is_session_open(GAME_NAME)? {
        println!("Shutting down running server");
        shutdown()?;
        sleep_seconds(10)
    }

    // The Unturned server has the id 1110390.
    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 1110390 \
        validate +quit"#,
        unturned_dir(config).to_string_lossy()
    )
    .run_success()?;

    startup(config)
}

fn shutdown() -> Result<()> {
    // Exit if the server is not running.
    if !is_session_open(GAME_NAME)? {
        println!("Instance {GAME_NAME} is not running.");
        return Ok(());
    }

    send_ctrl_c(GAME_NAME)?;
    send_input_newline(GAME_NAME, "exit")?;

    Ok(())
}
