use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use utils::{cmd, config::Config, process::*, secret::copy_secret_file, sleep_seconds, tmux::*};

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

const GAME_NAME: &str = "garrys";

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new(GAME_NAME).context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup { gamemode } => startup(&config, gamemode),
        SubCommand::Shutdown => shutdown(&config),
        SubCommand::Update => update(&config),
    }
}

fn startup(config: &Config, gamemode: GameMode) -> Result<()> {
    // Don't start the server if the session is already running.
    ensure_session_not_open(config)?;

    let game_dir = config.game_dir();
    start_session(config, None)?;

    // Remove the old compiled server config to avoid caching fuckery
    let server_vdf = game_dir.join("garrysmod/cfg/server.vdf");
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
                &config.default_config_dir().join("garrys/ttt.cfg"),
                &game_dir.join("garrysmod/cfg/server.cfg"),
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
                &config.default_config_dir().join("garrys/prop_hunt.cfg"),
                &game_dir.join("garrysmod/cfg/server.cfg"),
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
    send_input_newline_with_env(config, server_command, envs)?;

    Ok(())
}

fn update(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    if is_session_open(config)? {
        println!("Shutting down running server");
        shutdown(config)?;
        sleep_seconds(10);
    }

    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 4020 \
        validate +quit"#,
        config.game_dir_str()
    )
    .run_success()?;

    Ok(())
}

fn shutdown(config: &Config) -> Result<()> {
    // Exit if the server is not running.
    ensure_session_is_open(config)?;

    send_ctrl_c(config)?;
    send_input_newline(config, "exit")?;

    Ok(())
}
