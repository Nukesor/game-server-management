use std::collections::HashMap;
use std::fs::{create_dir_all, remove_dir_all, remove_file, rename};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use utils::cmd;
use utils::config::Config;
use utils::path::get_newest_file;
use utils::process::*;
use utils::secret::copy_secret_file;
use utils::tmux::*;

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    /// Update the game to a specific version.
    /// The version is expected in the `1.1.34` format.
    Update {
        version: String,
    },
    Backup,
}

#[derive(Parser, Debug)]
#[clap(
    name = "Factorio",
    about = "A small binary to manage my factorio server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which returns the factorio server dir from a given config.
fn factorio_dir(config: &Config) -> PathBuf {
    config.game_files().join(GAME_NAME)
}

const GAME_NAME: &str = "factorio";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config)?,
        SubCommand::Shutdown => shutdown(&config)?,
        SubCommand::Backup => backup(&config)?,
        SubCommand::Update { version } => update(&config, version)?,
    };

    Ok(())
}

fn startup(config: &Config) -> Result<()> {
    // Don't start the server if the session is already running.
    if is_session_open(GAME_NAME)? {
        println!("Instance factorio already running");
        return Ok(());
    }

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.clone());

    // Deploy the server config file
    let server_config_path = factorio_dir(config).join("config/custom-server-config.json");
    copy_secret_file(
        &config.factorio.server_config_path(),
        &server_config_path,
        &secrets,
    )
    .context("Failed while copying server config file")?;

    // Create a new session for this instance
    start_session(GAME_NAME, factorio_dir(config))?;

    let server_command = format!(
        "{}/bin/x64/factorio \
        --start-server-load-latest \
        --use-server-whitelist \
        --server-whitelist {} \
        --server-settings {}",
        factorio_dir(config).to_string_lossy(),
        factorio_dir(config)
            .join("config/server-whitelist.json")
            .to_string_lossy(),
        server_config_path.to_string_lossy(),
    );

    // Start the server
    send_input_newline(GAME_NAME, &server_command)?;

    Ok(())
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    if !is_session_open(GAME_NAME)? {
        println!("Instance {GAME_NAME} is not running.");
        return Ok(());
    }

    // Send Ctrl+C and wait a few seconds to save the map and shutdown the server
    send_ctrl_c(GAME_NAME)?;

    let five_seconds = std::time::Duration::from_millis(5000);
    std::thread::sleep(five_seconds);

    // Backup the map
    backup(config).context("Failed during backup:")?;

    // Exit the session
    send_input_newline(GAME_NAME, "exit")?;

    Ok(())
}

fn backup(config: &Config) -> Result<()> {
    let backup_dir = config.backup_root().join("factorio");
    // Get and create backup dir
    create_dir_all(&backup_dir)?;

    // Get path for the backup file
    let now = chrono::offset::Local::now();
    let dest: PathBuf = backup_dir.join(format!("factorio_{}.zip", now.format("%Y.%m.%d-%H:%M")));

    // Remove any already existing backups
    if dest.exists() {
        remove_file(&dest)?;
    }

    let save_file = get_newest_file(&factorio_dir(config).join("saves"))?;

    if let Some(path) = save_file {
        println!("Copying {path:?} to {dest:?}");
        std::fs::copy(path, dest)?;
    }

    Ok(())
}

fn update(config: &Config, version: String) -> Result<()> {
    shutdown(config).context("Failed during shutdown")?;

    let temp_dir = config.temp_dir().join("factorio");
    let game_files_backup_dir = config.game_files_backup().join("factorio");

    create_dir_all(&temp_dir).context("Failed to create temporary directory")?;
    create_dir_all(&game_files_backup_dir)
        .context("Failed to create game file backup directory")?;

    let files_to_backup = vec!["saves", "config", "mods", "mod-settings.json"];

    // Move all important files to a temporary directory
    for file_to_backup in &files_to_backup {
        let path: PathBuf = factorio_dir(config).join(file_to_backup);
        let dest: PathBuf = temp_dir.join(file_to_backup);
        if path.exists() {
            println!("Backing up {path:?} to {dest:?}");
            rename(&path, &dest)?;
        }
    }

    // Download the file to the server file directory
    let url = format!("https://factorio.com/get-download/{version}/headless/linux64",);
    let tar_name = format!("factorio_headless_x64_{version}.tar.xz");
    cmd!(
        "http --download \"{url}\" > {}/{tar_name}",
        game_files_backup_dir.to_string_lossy()
    )
    .run_success()?;

    // Remove the factorio directory
    if factorio_dir(config).exists() {
        remove_dir_all(factorio_dir(config))?;
    }

    // Untar the server files to the game directory
    cmd!(
        "tar xf {}/{} -C {}",
        game_files_backup_dir.to_string_lossy(),
        tar_name,
        config.game_files().to_string_lossy()
    )
    .run_success()?;

    // Move the files back in place
    for file_to_backup in &files_to_backup {
        let path: PathBuf = temp_dir.join(file_to_backup);
        let dest: PathBuf = factorio_dir(config).join(file_to_backup);
        if path.exists() {
            rename(&path, &dest)?;
            println!("Restoring {dest:?} from {path:?}");
        }
    }

    startup(config).context("Failed during startup:")
}
