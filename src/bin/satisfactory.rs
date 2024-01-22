use std::fs::create_dir;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use utils::config::Config;
use utils::path::expand_home;
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
    name = "Satisfactory",
    about = "A small binary to manage my Satisfactory server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which creates the server dir from a given config.
fn satisfactory_dir(config: &Config) -> PathBuf {
    config.game_files().join("satisfactory")
}

const GAME_NAME: &'static str = "satisfactory";

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
        println!("Instance satisfactory already running");
        return Ok(());
    }

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
    start_session(GAME_NAME, satisfactory_dir(config))?;

    send_input_newline(GAME_NAME, "./FactoryServer.sh")?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    if is_session_open(GAME_NAME)? {
        println!("Shutting down running server");
        shutdown()?;
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
        satisfactory_dir(config).to_string_lossy()
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
