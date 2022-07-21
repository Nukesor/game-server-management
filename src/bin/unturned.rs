use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use script_utils::config::Config;
use script_utils::process::*;
use script_utils::{cmd, sleep_seconds};

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Parser, Debug)]
#[clap(
    name = "Unturned",
    about = "A small binary to manage my Unturned server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which creates the server dir from a given config.
fn unturned_dir(config: &Config) -> PathBuf {
    config.game_files().join("unturned")
}

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
    let exit_status = cmd!("tmux has-session -t unturned").run()?;

    // Don't start the server if the tmux shell is already running.
    if exit_status.success() {
        return Ok(());
    }

    cmd!("tmux new -d -s unturned")
        .cwd(unturned_dir(config))
        .run_success()?;

    cmd!("tmux send -t unturned ././ServerHelper.sh +InternetServer/Jarvis ENTER").run_success()
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    let exit_status = cmd!("tmux has-session -t unturned").run()?;
    if exit_status.success() {
        println!("Shutting down running server");
        shutdown()?;
        sleep_seconds(10)
    }

    // The Unturned server has the id 1110390.
    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 1110390 \
        validate +quit"#,
        unturned_dir(config).to_string_lossy()
    )
    .run_success()?;

    startup(config)
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t unturned C-c").run_success()?;
    cmd!("tmux send-keys -t unturned exit ENTER").run_success()?;

    Ok(())
}
