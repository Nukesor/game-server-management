use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{ValueEnum, Parser};

use utils::cmd;
use utils::config::Config;
use utils::process::*;

#[derive(Parser, Debug, ValueEnum, Clone)]
enum GameMode {
    Normal,
    Promod,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Startup {
        #[clap(value_enum)]
        gamemode: GameMode,
    },
    Shutdown,
}

#[derive(Parser, Debug)]
#[clap(name = "CoD4", about = "A small binary to manage my Cod4 server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which returns the cod4 server dir from a given config.
fn cod4_dir(config: &Config) -> PathBuf {
    config.game_files().join("cod4")
}

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup { gamemode } => startup(&config, gamemode),
        SubCommand::Shutdown => shutdown(),
    }
}

fn startup(config: &Config, gamemode: GameMode) -> Result<()> {
    let exit_status = cmd!("tmux has-session -t cod").run()?;

    // Don't start the server if the tmux shell is already running.
    if exit_status.success() {
        return Ok(());
    }

    cmd!("tmux new -d -s cod")
        .cwd(cod4_dir(config))
        .run_success()?;

    let server_command = match gamemode {
        GameMode::Normal => std::concat!(
            "./cod4_lnxded ",
            "+exec promod.cfg ",
            "+set fs_homepath ./ ",
            "+set fs_game mods/pml220 ",
            "+set sv_punkbuster 0 ",
            "+map_rotatefi",
        ),

        GameMode::Promod => std::concat!(
            "./games/cod4/cod4_lnxded ",
            "+exec promod.cfg ",
            "+set fs_homepath ./ ",
            "+set fs_game mods/pml220 ",
            "+set sv_punkbuster 0 ",
            "+map_rotatefi",
        ),
    };

    cmd!("tmux send -t cod '{}' ENTER", server_command).run_success()
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t cod C-c").run_success()?;
    cmd!("tmux send-keys -t cod exit ENTER").run_success()?;

    Ok(())
}
