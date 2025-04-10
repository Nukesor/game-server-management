use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use utils::{config::Config, tmux::*};

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

const GAME_NAME: &str = "ut";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new(GAME_NAME).context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup { gamemode } => startup(&config, gamemode),
        SubCommand::Shutdown => shutdown(&config),
    }
}

fn startup(config: &Config, gamemode: GameMode) -> Result<()> {
    // Don't start the server if the session is already running.
    ensure_session_not_open(config)?;

    // Create a new session for this instance
    start_session(config, Some(config.game_dir().join("System")))?;

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
    send_input_newline(config, &server_command)?;

    Ok(())
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    send_ctrl_c(config)?;
    send_input_newline(config, "exit")?;

    Ok(())
}
