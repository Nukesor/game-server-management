use anyhow::{Context, Result};
use clap::Parser;

use utils::cmd;
use utils::config::Config;
use utils::process::*;
use utils::tmux::*;

#[derive(Parser, Debug)]
enum SubCommand {
    Startup { instance: String },
    Shutdown { instance: String },
    Backup { instance: String },
}

#[derive(Parser, Debug)]
#[clap(
    name = "Minecraft",
    about = "A small binary to manage my Minecraft server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "minecraft";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let mut config = Config::new(GAME_NAME).context("Failed to read config:")?;

    // Set the current minecraft instance.
    match &args.cmd {
        SubCommand::Startup { instance }
        | SubCommand::Shutdown { instance }
        | SubCommand::Backup { instance } => config.instance = Some(instance.clone()),
    }

    match args.cmd {
        SubCommand::Startup { .. } => startup(&config),
        SubCommand::Shutdown { .. } => shutdown(&config),
        SubCommand::Backup { .. } => backup(&config),
    }
}

fn startup(config: &Config) -> Result<()> {
    // Don't start the server if the session is already running.
    ensure_session_not_open(config)?;

    // Create a new session for this instance
    start_session(config, None)?;

    // Start the server
    send_input_newline(config, "./ServerStart.sh")?;

    Ok(())
}

fn backup(config: &Config) -> Result<()> {
    // Inform users and save the map if the server is running.
    if is_session_open(config)? {
        // Send a backup message
        send_input_newline(config, "/say Running full backup")?;

        // Save the world to disk
        send_input_newline(config, "/save-all flush")?;

        // Wait for at least a minute to give minecraft enough time to gracefully shutdown
        let delay = std::time::Duration::from_millis(60000);
        std::thread::sleep(delay);
    }

    // Get and create backup dir
    let backup_dir = config.backup_dir();
    std::fs::create_dir_all(&backup_dir)?;

    // Get path for the backup file
    let now = chrono::offset::Local::now();
    let dest = backup_dir.join(format!(
        "{}_{}.tar.zst",
        config.session_name(),
        now.format("%Y.%m.%d-%H-%M")
    ));

    // Remove any already existing backups
    if dest.exists() {
        std::fs::remove_file(&dest)?;
    }

    cmd!(
        "tar -I zstd -cvf {} {}",
        dest.to_string_lossy(),
        config.game_dir_str()
    )
    .run_success()?;

    Ok(())
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    backup(config)?;

    // Send Ctrl+C and exit
    send_input_newline(config, "/say Server is gracefully shutting down")?;
    send_input_newline(config, "/stop")?;

    // Wait for at least a minute to give minecraft enough time to gracefully shutdown
    let delay = std::time::Duration::from_millis(60000);
    std::thread::sleep(delay);

    // Exit the session
    send_input_newline(config, "exit")?;

    Ok(())
}
