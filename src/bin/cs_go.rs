use std::collections::HashMap;
use std::fs::create_dir;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use utils::config::Config;
use utils::path::expand_home;
use utils::process::*;
use utils::secret::copy_secret_file;
use utils::tmux::*;
use utils::{cmd, sleep_seconds};

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Parser, Debug)]
#[clap(name = "CS:GO", about = "A small binary to manage my CS:GO server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which creates the server dir from a given config.
fn csgo_dir(config: &Config) -> PathBuf {
    config.game_files().join("cs_go")
}

const GAME_NAME: &'static str = "csgo";

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
        println!("Instance cs_go already running");
        return Ok(());
    }

    // Create a new session for this instance
    start_session(GAME_NAME, csgo_dir(config))?;

    // CS:GO expects the steamclient.so library to be at a different location.
    // Hence, we create a symlink to the expected location.
    let folder = expand_home("~/.steam/sdk32/");
    if !folder.exists() {
        create_dir(folder)?;
    }
    let link_src = expand_home("~/.steam/steamcmd/linux32/steamclient.so");
    let link_dest = expand_home("~/.steam/sdk32/steamclient.so");
    if link_src.exists() && !link_dest.exists() {
        symlink(link_src, link_dest)?;
    }

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.clone());

    // Get the command by gamemode and copy the respective config file
    copy_secret_file(
        &config.cs_go.server_config_path(),
        &csgo_dir(config).join("csgo/cfg/server.cfg"),
        &secrets,
    )?;

    let mut server_command = concat!(
        "./srcds_run ",
        "-console ",
        "-game csgo ",
        "-ip 0.0.0.0 ",
        "-usercon ",
        "+map de_dust2 ",
        "+game_type 0 ",
        "+game_mode 1 ",
        "+mapgroup mg_active ",
    )
    .to_string();
    server_command.push_str(&format!(
        "+sv_setsteamaccount {} ",
        config.cs_go.login_token
    ));

    send_input_newline(GAME_NAME, &server_command)?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    if is_session_open(GAME_NAME)? {
        println!("Shutting down running server");
        shutdown()?;
        sleep_seconds(10);
    }

    // The CS:GO server has the id 740.
    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 740 \
        validate +quit"#,
        csgo_dir(config).to_string_lossy()
    )
    .run_success()?;

    // Restart the server
    startup(config)?;

    Ok(())
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
