use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use utils::config::Config;
use utils::tmux::*;

#[derive(Parser, Debug, ValueEnum, Clone)]
enum GameMode {
    Am,
    Tam,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Startup {
        #[clap(value_enum, default_value_t = GameMode::Am)]
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

const GAME_NAME: &str = "ut";

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
    // Don't start the server if the session is already running.
    if is_session_open(GAME_NAME)? {
        println!("Instance ut already running");
        return Ok(());
    }

    // Create a new session for this instance
    start_session(GAME_NAME, ut2004_dir(config).join("System"))?;

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
    send_input_newline(GAME_NAME, &server_command)?;

    Ok(())
}

fn shutdown() -> Result<()> {
    // Exit if the server is not running.
    if !is_session_open(GAME_NAME)? {
        println!("Instance {GAME_NAME} is not running.");
        return Ok(());
    }

    send_ctrl_c(GAME_NAME)?;
    send_input_newline(GAME_NAME, "exit")?;

    Ok(())
}
