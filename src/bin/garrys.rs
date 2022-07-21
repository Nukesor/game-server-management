use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{ArgEnum, Parser};

use script_utils::config::Config;
use script_utils::process::*;
use script_utils::secret::copy_secret_file;
use script_utils::{cmd, sleep_seconds};

#[derive(Parser, Debug, ArgEnum, Clone)]
enum GameMode {
    Ttt,
    Prophunt,
    Zombie,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Startup {
        #[clap(arg_enum)]
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
    config.game_files().join("garrys")
}

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
    let exit_status = cmd!("tmux has-session -t garry").run()?;

    // Don't start the server if the tmux shell is already running.
    if exit_status.success() {
        return Ok(());
    }

    cmd!("tmux new -d -s garry")
        .cwd(garrys_dir(config))
        .run_success()?;

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
                &config.garrys.ttt_server_config(),
                &garrys_dir(config).join("garrysmod/cfg/server.cfg"),
                secrets,
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
                &config.garrys.prophunt_server_config(),
                &garrys_dir(config).join("garrysmod/cfg/server.cfg"),
                secrets,
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

    cmd!("tmux send -t garry '{}' ENTER", server_command)
        .env("STEAM_WEB_API_KEY", config.garrys.steam_web_api_key.clone())
        .run_success()
}

fn update(config: &Config) -> Result<()> {
    // Check if the server is running and shut it down if it is.
    let exit_status = cmd!("tmux has-session -t garry").run()?;
    if exit_status.success() {
        println!("Shutting down running server");
        shutdown()?;
        sleep_seconds(10)
    }

    cmd!(
        r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 4020 \
        validate +quit"#,
        garrys_dir(config).to_string_lossy()
    )
    .run_success()
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t garry C-c").run_success()?;
    cmd!("tmux send-keys -t garry exit ENTER").run_success()?;

    Ok(())
}
