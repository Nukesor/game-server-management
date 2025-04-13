use std::{
    collections::HashMap,
    fs::{remove_dir_all, rename},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use utils::{
    backup::backup_file,
    cmd,
    config::Config,
    path::get_newest_file,
    process::*,
    secret::copy_secret_file,
    tmux::*,
};

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

const GAME_NAME: &str = "factorio";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new(GAME_NAME).context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config)?,
        SubCommand::Shutdown => shutdown(&config)?,
        SubCommand::Backup => backup(&config)?,
        SubCommand::Update { version } => update(&config, version)?,
    };

    Ok(())
}

fn startup(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_not_open(config)?;

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.clone());

    // Deploy the server config file
    let server_config_path = config.game_dir().join("config/custom-server-config.json");
    copy_secret_file(
        &config
            .default_config_dir()
            .join("factorio-server-settings.json"),
        &server_config_path,
        &secrets,
    )
    .context("Failed while copying server config file")?;

    // Create a new session for this instance
    start_session(config, None)?;

    let server_command = format!(
        "{}/bin/x64/factorio \
        --start-server-load-latest \
        --use-server-whitelist \
        --server-whitelist {} \
        --server-settings {}",
        config.game_dir_str(),
        config
            .game_dir()
            .join("config/server-whitelist.json")
            .to_string_lossy(),
        server_config_path.to_string_lossy(),
    );

    // Start the server
    send_input_newline(config, &server_command)?;

    Ok(())
}

fn backup(config: &Config) -> Result<()> {
    let save_file = get_newest_file(&config.game_dir().join("saves"))?;
    if let Some(file_to_backup) = save_file {
        backup_file(
            file_to_backup,
            config.create_backup_dir()?,
            "factorio",
            "zip",
        )?;
    }

    Ok(())
}

fn update(config: &Config, version: String) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    shutdown(config).context("Failed during shutdown")?;

    let temp_dir = config.create_temp_dir()?;

    let files_to_backup = vec!["saves", "config", "mods", "mod-settings.json"];

    // Move all important files to a temporary directory
    for file_to_backup in &files_to_backup {
        let path: PathBuf = config.game_dir().join(file_to_backup);
        let dest: PathBuf = temp_dir.join(file_to_backup);
        if path.exists() {
            println!("Backing up {path:?} to {dest:?}");
            rename(&path, &dest)?;
        }
    }

    // Download the file to the server file directory
    let url = format!("https://factorio.com/get-download/{version}/headless/linux64",);
    let tar_name = format!("factorio_headless_x64_{version}.tar.xz");
    cmd!("http --download \"{url}\" > /tmp/{tar_name}").run_success()?;

    // Remove the factorio directory
    if config.game_dir().exists() {
        remove_dir_all(config.game_dir())?;
    }

    // Untar the server files to the game directory
    cmd!("tar xf /tmp/{} -C {}", tar_name, config.game_dir_str()).run_success()?;

    // Move the files back in place
    for file_to_backup in &files_to_backup {
        let path: PathBuf = temp_dir.join(file_to_backup);
        let dest: PathBuf = config.game_dir().join(file_to_backup);
        if path.exists() {
            rename(&path, &dest)?;
            println!("Restoring {dest:?} from {path:?}");
        }
    }

    startup(config).context("Failed during startup:")
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    // Send Ctrl+C and wait a few seconds to save the map and shutdown the server
    send_ctrl_c(config)?;

    let five_seconds = std::time::Duration::from_millis(5000);
    std::thread::sleep(five_seconds);

    // Backup the map
    backup(config).context("Failed during backup:")?;

    // Exit the session
    send_input_newline(config, "exit")?;

    Ok(())
}
