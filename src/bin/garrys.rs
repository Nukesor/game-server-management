use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use utils::config::Config;
use utils::process::*;
use utils::secret::copy_secret_file;
use utils::tmux::*;
use utils::{cmd, sleep_seconds};

#[derive(Parser, Debug, ValueEnum, Clone)]
enum GameMode {
    Ttt,
    Prophunt,
    Zombie,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Startup {
        #[clap(value_enum, default_value_t = GameMode::Ttt)]
        gamemode: GameMode,
    },
    Shutdown,
    Update,
}

#[derive(Parser, Debug)]
#[clap(
    name = "Garry's mod",
    about = "A small binary to manage my Garry's mod server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which returns the garrys mod server dir from a given config.
fn garrys_dir(config: &Config) -> PathBuf {
    config.game_files().join(GAME_NAME)
}

const GAME_NAME: &str = "garrys";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup { gamemode } => startup(&config, gamemode),
        SubCommand::Shutdown => shutdown(),
        SubCommand::Update => update(&config),
    }
}

fn startup(config: &Config, gamemode: GameMode) -> Result<()> {
    // Don't start the server if the session is already running.
    if is_session_open(GAME_NAME)? {
        println!("Instance garrys already running");
        return Ok(());
    }

    start_session(GAME_NAME, garrys_dir(config))?;

    // Remove the old compiled server config to avoid caching fuckery
    let server_vdf = garrys_dir(config).join("garrysmod/cfg/server.vdf");
    if server_vdf.exists() {
        std::fs::remove_file(server_vdf)?;
    }

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.clone());

    // Get the command by gamemode and copy the respective config file
    let server_command = match gamemode {
        GameMode::Ttt => {
            // Deploy the server config file
            copy_secret_file(
                &config.garrys.ttt_server_config_path(),
                &garrys_dir(config).join("garrysmod/cfg/server.cfg"),
                &secrets,
            )
            .context("Failed to copy ttt server config")?;

            concat!(
                "./srcds_run ",
                "-game garrysmod ",
                "-usercon ",
                "-authkey $STEAM_WEB_API_KEY ",
                "+gamemode terrortown ",
                "+hostname Nukesors_garry_playground ",
                "+map ttt_rooftops_2016_v1 ",
                "+host_workshop_collection 2089206449",
            )
        }

        GameMode::Prophunt => {
            copy_secret_file(
                &config.garrys.prophunt_server_config_path(),
                &garrys_dir(config).join("garrysmod/cfg/server.cfg"),
                &secrets,
            )
            .context("Failed to copy prophunt server config")?;

            concat!(
                "./srcds_run ",
                "-game garrysmod ",
                "-usercon ",
                "-authkey $STEAM_WEB_API_KEY ",
                "+gamemode prop_hunt ",
                "+hostname Nukesors_garry_playground ",
                "+map ph_indoorpool ",
                "+host_workshop_collection 2090357275",
            )
        }
        GameMode::Zombie => concat!(
            "./srcds_run ",
            "-game garrysmod ",
            "-usercon ",
            "-authkey $STEAM_WEB_API_KEY ",
            "+gamemode zombiesurvival ",
            "+hostname Nukesors_garry_playground ",
            "+map zs_cleanoffice_v2 ",
            "+host_workshop_collection 157384458",
        ),
    };

    let envs = map_macro::hash_map! {
        "STEAM_WEB_API_KEY" => config.garrys.steam_web_api_key.clone()
    };
    send_input_newline_with_env(GAME_NAME, server_command, envs)?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    if is_session_open(GAME_NAME)? {
        println!("Shutting down running server");
        shutdown()?;
        sleep_seconds(10);
    }

    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 4020 \
        validate +quit"#,
        garrys_dir(config).to_string_lossy()
    )
    .run_success()?;

    Ok(())
}

fn shutdown() -> Result<()> {
    // Check if the server is running and exit if it isn't.
    if !is_session_open(GAME_NAME)? {
        println!("Instance {GAME_NAME} is not running.");
        return Ok(());
    }

    send_ctrl_c(GAME_NAME)?;
    send_input_newline(GAME_NAME, "exit")?;

    Ok(())
}
