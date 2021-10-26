use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use script_utils::cmd;
use script_utils::config::Config;
use script_utils::process::*;

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
    // Don't start the server if the tmux shell is already running.
    let exit_status = cmd!("tmux has-session -t {}", instance).run()?;
    if exit_status.success() {
        return Ok(());
    }

    // Create a new tmux session for this instance
    cmd!("tmux new -d -s {}", instance)
        .cwd(instance_dir(config, instance))
        .run_success()?;

    // Start the server
    cmd!("tmux send -t {} ./ServerStart.sh ENTER", instance).run_success()
}

fn shutdown(config: &Config, instance: &str) -> Result<()> {
    // Server has to be running
    cmd!("tmux has-session -t {}", instance).run_success()?;

    backup(config, instance)?;

    // Send Ctrl+C and exit
    cmd!("tmux send-keys -t {} C-c", instance).run_success()?;

    let five_seconds = std::time::Duration::from_millis(5000);
    std::thread::sleep(five_seconds);

    // Exit the session
    cmd!("tmux send-keys -t {} exit ENTER", instance).run_success()?;

    Ok(())
}

fn backup(config: &Config, instance: &str) -> Result<()> {
    // Server has to be running
    cmd!("tmux has-session -t {}", instance).run_success()?;

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
    .run_success()
}
