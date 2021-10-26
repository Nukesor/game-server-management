use std::fs::create_dir;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use script_utils::cmd;
use script_utils::config::Config;
use script_utils::path::expand_home;
use script_utils::process::*;

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Parser, Debug)]
#[clap(
    name = "Satisfactory",
    about = "A small binary to manage my Satisfactory server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which creates the server dir from a given config.
fn satisfactory_dir(config: &Config) -> PathBuf {
    config.game_files().join("satisfactory")
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
    let exit_status = cmd!("tmux has-session -t satisfactory").run()?;

    // Don't start the server if the tmux shell is already running.
    if exit_status.success() {
        return Ok(());
    }

    // The satisfactory expects the steamclient.so library to be at a different location.
    // We create a symlink to the expected location.
    let folder = expand_home("~/.steam/steamcmd/sdk64/");
    if !folder.exists() {
        create_dir(folder)?;
    }
    let link_src = expand_home("~/.steam/steamcmd/linux64/steamclient.so");
    let link_dest = expand_home("~/.steam/steamcmd/sdk64/steamclient.so");
    if link_src.exists() && !link_dest.exists() {
        symlink(link_src, link_dest)?;
    }

    cmd!("tmux new -d -s satisfactory")
        .cwd(satisfactory_dir(config))
        .run_success()?;

    cmd!("tmux send -t satisfactory ./FactoryServer.sh ENTER").run_success()
}

fn update(config: &Config) -> Result<()> {
    // The Satisfactory server has the id 1690800.
    cmd!(
        r#"steamcmd \
        +login anonymous \
        +force_install_dir {} \
        +app_update 1690800 \
        validate +quit"#,
        satisfactory_dir(config).to_string_lossy()
    )
    .run_success()
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t satisfactory C-c").run_success()?;
    cmd!("tmux send-keys -t satisfactory exit ENTER").run_success()?;

    Ok(())
}
