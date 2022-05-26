use std::collections::HashMap;
use std::fs::{create_dir_all, remove_file};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;

use script_utils::cmd;
use script_utils::config::Config;
use script_utils::process::*;
use script_utils::secret::copy_secret_file;

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Backup,
}

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
    config.game_files().join("terraria")
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
    let exit_status = cmd!("tmux has-session -t terraria").run()?;

    // Don't start the server if the tmux shell is already running.
    if exit_status.success() {
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
        &config.terraria.server_config(),
        &server_config_path,
        secrets,
    )
    .context("Failed while copying server config file")?;

    cmd!("tmux new -d -s terraria")
        .cwd(terraria_dir(config))
        .run_success()?;

    cmd!(
        "tmux send -t terraria 'terraria-server -config {}' ENTER",
        server_config_path.to_string_lossy()
    )
    .run_success()
}

fn backup(config: &Config) -> Result<()> {
    let backup_dir = config.backup_root().join("terraria");
    // Get and create backup dir
    create_dir_all(&backup_dir)?;

    // Ignore the result. This might happen if terraria isn't running right now.
    let _result = cmd!("tmux send -t terraria save ENTER").run();
    std::thread::sleep(Duration::from_millis(2000));

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

    println!("Copying {:?} to {:?}", save_file, dest);
    std::fs::copy(save_file, dest)?;

    Ok(())
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t terraria C-c").run_success()?;
    cmd!("tmux send-keys -t terraria exit ENTER").run_success()?;

    Ok(())
}
