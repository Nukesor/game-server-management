use std::path::PathBuf;

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

/// Small helper which returns the minecraft server dir from a given config.
fn instance_dir(config: &Config, instance: &str) -> PathBuf {
    config.game_files().join(instance)
}

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup { instance } => startup(&config, &instance),
        SubCommand::Shutdown { instance } => shutdown(&config, &instance),
        SubCommand::Backup { instance } => backup(&config, &instance),
    }
}

fn startup(config: &Config, instance: &str) -> Result<()> {
    // Don't start the server if the session is already running.
    if is_session_open(instance)? {
        println!("Instance {instance} already running");
        return Ok(());
    }

    // Create a new session for this instance
    start_session(instance, instance_dir(config, instance))?;

    // Start the server
    send_input_newline(instance, "./ServerStart.sh")?;

    Ok(())
}

fn shutdown(config: &Config, instance: &str) -> Result<()> {
    // Exit if the server is not running.
    if !is_session_open(instance)? {
        println!("Instance {instance} is not running");
        return Ok(());
    }

    backup(config, instance)?;

    // Send Ctrl+C and exit
    send_input_newline(instance, "/say Server rebooted gleich und ist kurz weg")?;
    send_input_newline(instance, "/stop")?;

    // Wait for at least a minute to give minecraft enough time to gracefully shutdown
    let delay = std::time::Duration::from_millis(60000);
    std::thread::sleep(delay);

    // Exit the session
    send_input_newline(instance, "exit")?;

    Ok(())
}

fn backup(config: &Config, instance: &str) -> Result<()> {
    // Inform users and save the map if the server is running.
    if is_session_open(instance)? {
        // Send a backup message
        send_input_newline(instance, "/say Running full backup")?;

        // Save the world to disk
        send_input_newline(instance, "/save-all flush")?;

        // Wait for at least a minute to give minecraft enough time to gracefully shutdown
        let delay = std::time::Duration::from_millis(60000);
        std::thread::sleep(delay);
    }

    // Get and create backup dir
    let backup_dir = config.backup_root().join("minecraft").join(instance);
    std::fs::create_dir_all(&backup_dir)?;

    // Get path for the backup file
    let now = chrono::offset::Local::now();
    let dest = backup_dir.join(format!(
        "{}_{}.tar.zst",
        instance,
        now.format("%Y.%m.%d-%H-%M")
    ));

    // Remove any already existing backups
    if dest.exists() {
        std::fs::remove_file(&dest)?;
    }

    cmd!(
        "tar -I zstd -cvf {} {}",
        dest.to_string_lossy(),
        instance_dir(config, instance).to_string_lossy()
    )
    .run_success()?;

    Ok(())
}
