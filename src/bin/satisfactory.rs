use std::{fs::create_dir, os::unix::fs::symlink};

use anyhow::{Context, Result};
use clap::Parser;
use utils::{cmd, config::Config, path::expand_home, process::*, sleep_seconds, tmux::*};

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Parser, Debug)]
#[clap(
    name = "Satisfactory",
    about = "A small binary to manage my Satisfactory server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "satisfactory";

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
    ensure_session_not_open(config)?;

    // Satisfactory expects the steamclient.so library to be at a different location.
    // We create a symlink to the expected location.
    let folder = expand_home("~/.steam/steamcmd/sdk64/");
    if !folder.exists() {
        create_dir(folder)?;
    }
    let link_src = expand_home("~/.steam/steamcmd/linux64/steamclient.so");
    let link_dest = expand_home("~/.steam/steamcmd/sdk64/steamclient.so");
    if link_src.exists() && !link_dest.exists() {
        symlink(link_src, link_dest)?;
    }

    // Create a new session for this instance
    start_session(config, None)?;

    send_input_newline(config, "./FactoryServer.sh")?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    if is_session_open(config)? {
        println!("Shutting down running server");
        shutdown(config)?;
        sleep_seconds(10)
    }

    // The Satisfactory server has the id 1690800.
    println!("Running update command");
    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 1690800 \
        validate +quit"#,
        config.game_dir_str()
    )
    .run_success()?;

    startup(config)
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    send_ctrl_c(config)?;
    send_input_newline(config, "exit")?;

    Ok(())
}
