use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{ValueEnum, Parser};

use utils::cmd;
use utils::config::Config;
use utils::process::Cmd;

#[derive(Parser, Debug, ValueEnum, Clone)]
enum GameMode {
    Am,
    Tam,
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
#[clap(name = "UT2004", about = "A small binary to manage my UT2004 server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which returns the ut2004 server dir from a given config.
fn ut2004_dir(config: &Config) -> PathBuf {
    config.game_files().join("ut2004")
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
    let exit_status = cmd!("tmux has-session -t ut").run()?;

    // Don't start the server if the tmux shell is already running.
    if exit_status.success() {
        return Ok(());
    }

    cmd!("tmux new -d -s ut")
        .cwd(ut2004_dir(config).join("System"))
        .run_success()?;

    let server_command = match gamemode {
        GameMode::Tam => std::concat!(
            "./ucc-bin server ",
            "\"DM-Asbestos",
            "?game=3SPNv3141.TeamArenaMaster",
            "?AdminName=private",
            "?AdminPassword={{ password }}\" ",
            "ini=ut2004.ini ",
            "-nohomedir",
        ),
        GameMode::Am => std::concat!(
            "./ucc-bin server ",
            "\"DM-Asbestos",
            "?game=3SPNv3141.ArenaMaster",
            "?AdminName=private",
            "?AdminPassword={{ password }}\" ",
            "ini=ut2004.ini ",
            "-nohomedir",
        ),
    };

    let server_command = server_command.replace("{{ password }}", &config.default_password);

    cmd!("tmux send -t ut '{}' ENTER", server_command).run_success()
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t ut C-c").run_success()?;
    cmd!("tmux send-keys -t ut exit ENTER").run_success()?;

    Ok(())
}
