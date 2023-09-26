use std::collections::HashMap;
use std::fs::{create_dir_all, remove_file};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;

use utils::config::Config;
use utils::secret::copy_secret_file;
use utils::tmux::*;

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Backup,
}

const GAME_NAME: &'static str = "terraria";

#[derive(Parser, Debug)]
#[clap(
    name = "Terraria",
    about = "A small binary to manage my Terraria server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which creates the server dir from a given config.
fn terraria_dir(config: &Config) -> PathBuf {
    config.game_files().join(GAME_NAME)
}

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config),
        SubCommand::Shutdown => shutdown(),
        SubCommand::Backup => backup(&config),
    }
}

fn startup(config: &Config) -> Result<()> {
    // Don't start the server if the session is already running.
    if is_session_open(GAME_NAME)? {
        println!("Instance terraria already running");
        return Ok(());
    }

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
    let server_config_path = terraria_dir(config).join("config.txt");
    copy_secret_file(
        &config.terraria.server_config_path(),
        &server_config_path,
        &secrets,
    )
    .context("Failed while copying server config file")?;

    // Create a new session for this instance
    start_session(GAME_NAME, terraria_dir(config))?;

    let server_command = format!(
        "terraria-server -config {}",
        server_config_path.to_string_lossy()
    );
    send_input_newline(GAME_NAME, &server_command)?;

    Ok(())
}

fn backup(config: &Config) -> Result<()> {
    let backup_dir = config.backup_root().join("terraria");
    // Get and create backup dir
    create_dir_all(&backup_dir)?;

    // Save the game if the server is running right now.
    if is_session_open(GAME_NAME)? {
        send_input_newline(GAME_NAME, "save")?;
        std::thread::sleep(Duration::from_millis(5000));
    }

    // Get path for the backup file
    let now = chrono::offset::Local::now();
    let dest: PathBuf = backup_dir.join(format!(
        "{}_{}.wld",
        config.terraria.world_name,
        now.format("%Y.%m.%d-%H:%M")
    ));

    // Remove any already existing backup file with the same name.
    if dest.exists() {
        remove_file(&dest)?;
    }

    let save_file = config.terraria.world_path();

    println!("Copying {save_file:?} to {dest:?}");
    std::fs::copy(save_file, dest)?;

    Ok(())
}

fn shutdown() -> Result<()> {
    // Exit if the server is not running.
    if !is_session_open(GAME_NAME)? {
        println!("Instance {GAME_NAME} is not running.");
        return Ok(());
    }

    // Exit the server via the exit command
    send_input_newline(GAME_NAME, "exit")?;
    std::thread::sleep(Duration::from_millis(3000));
    // Exit the shell
    send_input_newline(GAME_NAME, "exit")?;

    Ok(())
}
