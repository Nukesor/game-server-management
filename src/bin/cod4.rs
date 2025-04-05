use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use utils::config::Config;
use utils::secret::copy_secret_file;
use utils::tmux::*;

#[derive(Parser, Debug, ValueEnum, Clone)]
enum GameMode {
    Normal,
    Promod,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Startup {
        #[clap(value_enum, default_value_t = GameMode::Normal)]
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

const GAME_NAME: &str = "cod";

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
    start_session(config, None)?;

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.clone());

    // Deploy the default config file
    copy_secret_file(
        &config.cod4.default_config_path(),
        &config.game_dir().join("main/default.cfg"),
        &secrets,
    )
    .context("Failed to copy default cod4 config")?;

    let server_command = match gamemode {
        GameMode::Normal => std::concat!(
            "./cod4x18_dedrun ",
            "+exec default.cfg ",
            "+set fs_homepath ./ ",
            "+set sv_punkbuster 0 ",
            "+map_rotate",
        ),

        GameMode::Promod => {
            // Deploy the promod config file
            copy_secret_file(
                &config.cod4.promod_config_path(),
                &config.game_dir().join("main/promod.cfg"),
                &secrets,
            )
            .context("Failed to copy default cod4 config")?;

            std::concat!(
                "./cod4x18_dedrun ",
                "+exec default.cfg ",
                "+exec promod.cfg ",
                "+set fs_homepath ./ ",
                "+set fs_game mods/pml220 ",
                "+set sv_punkbuster 0 ",
                "+map_rotate",
            )
        }
    };

    send_input_newline(config, server_command)?;

    Ok(())
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    send_ctrl_c(config)?;
    send_input_newline(config, "exit")?;

    Ok(())
}
