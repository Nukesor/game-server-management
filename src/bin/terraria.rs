use std::{collections::HashMap, thread::sleep, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;
use utils::{backup::backup_file, config::Config, secret::copy_secret_file, tmux::*};

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Backup,
}

const GAME_NAME: &str = "terraria";

#[derive(Parser, Debug)]
#[clap(
    name = "Terraria",
    about = "A small binary to manage my Terraria server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new(GAME_NAME).context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config),
        SubCommand::Shutdown => shutdown(&config),
        SubCommand::Backup => backup(&config),
    }
}

fn startup(config: &Config) -> Result<()> {
    // Don't start the server if the session is already running.
    ensure_session_not_open(config)?;

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.to_string());
    secrets.insert("port", config.terraria.port.to_string());
    secrets.insert("world_name", config.terraria.world_name.to_string());
    secrets.insert(
        "world_path",
        config.terraria.world_path().to_string_lossy().to_string(),
    );

    // Copy the terraria config to the expected location.
    let server_config_path = config.game_dir().join("config.txt");
    copy_secret_file(
        &config.default_config_dir().join("terraria-config.txt"),
        &server_config_path,
        &secrets,
    )
    .context("Failed while copying server config file")?;

    // Create a new session for this instance
    start_session(config, None)?;

    let server_command = format!(
        "terraria-server -config {}",
        server_config_path.to_string_lossy()
    );
    send_input_newline(config, &server_command)?;

    Ok(())
}

fn backup(config: &Config) -> Result<()> {
    // Save the game if the server is running right now.
    if is_session_open(config)? {
        send_input_newline(config, "save")?;
        sleep(Duration::from_millis(5000));
    }

    backup_file(
        config.terraria.world_path(),
        config.create_backup_dir()?,
        &config.terraria.world_name,
        "wld",
    )?;

    Ok(())
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    // Exit the server via the exit command
    send_input_newline(config, "exit")?;
    sleep(Duration::from_millis(3000));
    // Exit the shell
    send_input_newline(config, "exit")?;

    Ok(())
}
