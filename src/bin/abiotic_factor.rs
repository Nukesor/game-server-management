use std::collections::HashMap;

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
    name = "Abiotic Factor Server",
    about = "A small binary to manage my Abiotic Factor server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "abiotic-factor";

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

    // Create a new session for this instance
    start_session(config, None)?;

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.clone());

    let mut server_command = concat!(
        "./",
        "-log ",
        "-newconsole ",
        "-useperfthreads ",
        r#"-SteamServerName="Nukes's Playground" "#,
        "-PORT=40450 ",
        "-QueryPort=40451 ",
        "-MaxServerPlayers=6 ",
    )
    .to_string();
    server_command.push_str(&format!(
        "+sv_setsteamaccount {} ",
        config.cs_go.login_token
    ));

    send_input_newline(config, &server_command)?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    if is_session_open(config)? {
        println!("Shutting down running server");
        shutdown(config)?;
        sleep_seconds(10);
    }

    // The CS:GO server has the id 740.
    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 2857200 \
        validate +quit"#,
        config.game_dir_str()
    )
    .run_success()?;

    // Restart the server
    startup(config)?;

    Ok(())
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    send_ctrl_c(config)?;
    send_input_newline(config, "exit")?;

    Ok(())
}
