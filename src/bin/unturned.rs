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

const GAME_NAME: &str = "unturned";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new(GAME_NAME).context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config),
        SubCommand::Shutdown => shutdown(&config),
        SubCommand::Update => update(&config),
    }
}

fn startup(config: &Config) -> Result<()> {
    // Don't start the server if the session is already running.
    if is_session_open(config)? {
        println!("Instance unturned already running");
        return Ok(());
    }

    // Create a new session for this instance
    start_session(config, None)?;

    let server_command = "./ServerHelper.sh +InternetServer/Jarvis";
    send_input_newline(config, server_command)?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    if is_session_open(config)? {
        println!("Shutting down running server");
        shutdown(config)?;
        sleep_seconds(10)
    }

    // The Unturned server has the id 1110390.
    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 1110390 \
        validate +quit"#,
        config.game_dir_str()
    )
    .run_success()?;

    startup(config)
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    if !is_session_open(config)? {
        println!("Instance {GAME_NAME} is not running.");
        return Ok(());
    }

    send_ctrl_c(config)?;
    send_input_newline(config, "exit")?;

    Ok(())
}
