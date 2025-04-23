use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use utils::{
    backup::backup_directory,
    cmd,
    config::Config,
    process::*,
    secret::copy_secret_file,
    sleep_seconds,
    tmux::*,
};

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Backup,
    Update,
    Shutdown,
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
const WORLD_SAVE_NAME: &str = "MadLab";

fn server_dir(config: &Config) -> PathBuf {
    config
        .game_dir()
        .join("AbioticFactor/Saved/SaveGames/Server")
}

fn world_dir(config: &Config) -> PathBuf {
    server_dir(config).join("Worlds").join(WORLD_SAVE_NAME)
}

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new(GAME_NAME).context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config),
        SubCommand::Backup => backup(&config),
        SubCommand::Update => update(&config),
        SubCommand::Shutdown => shutdown(&config),
    }
}

fn startup(config: &Config) -> Result<()> {
    // Don't start the server if the session is already running.
    ensure_session_not_open(config)?;

    // Create a new session for this instance
    start_session(config, None)?;

    let mut secrets = HashMap::new();
    secrets.insert("admin_steam_id", config.admin_steam_id.clone());

    // Copy the world-spanning admin settings
    copy_secret_file(
        &config.default_config_dir().join("abiotic_factor/Admin.ini"),
        &server_dir(config).join("Admin.ini"),
        &secrets,
    )?;

    // Copy the world config file.
    copy_secret_file(
        &config
            .default_config_dir()
            .join("abiotic_factor/AbioticFactor.ini"),
        &world_dir(config).join("SandboxSettings.ini"),
        &secrets,
    )?;

    let mut server_command = concat!(
        "WINEDEBUG=fixme-all ",
        "wine ./AbioticFactor/Binaries/Win64/AbioticFactorServer-Win64-Shipping.exe ",
        "-log ",
        "-newconsole ",
        "-useperfthreads ",
        "-NoAsyncLoadingThread ",
        r#"-SteamServerName="MadLab Hamburg" "#,
        "-PORT=7780 ",
        "-QueryPort=7781 ",
        "-MaxServerPlayers=6 ",
        "-SteamServerName=Jarvis ",
    )
    .to_string();
    server_command.push_str(&format!("-WorldSaveName={WORLD_SAVE_NAME} "));
    server_command.push_str(&format!(
        r#"-ServerPassword="{}" "#,
        config.default_password
    ));

    send_input_newline(config, &server_command)?;

    Ok(())
}

/// Save the game.
///
/// There's currently no way to force saving via the CLI.
/// The game apparently saves automatically from time to time, so we have to rely on that.
/// It's seemingly possible to force saving via the admin interface as well.
fn backup(config: &Config) -> Result<()> {
    backup_directory(
        world_dir(config),
        config.create_backup_dir()?,
        WORLD_SAVE_NAME,
    )?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    if is_session_open(config)? {
        println!("Shutting down running server");
        shutdown(config)?;
        sleep_seconds(10);
    }

    // Run a quick backup for good measure.
    backup(config)?;

    // The CS:GO server has the id 740.
    cmd!(
        r#"steamcmd \
        +@sSteamCmdForcePlatformType windows \
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
