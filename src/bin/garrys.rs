use std::collections::HashMap;

use clap::{Parser, ValueEnum};
use utils::prelude::*;

#[derive(Clone, Copy, Debug, Default, Parser, ValueEnum)]
enum GameMode {
    #[default]
    Ttt,
    Prophunt,
    Zombie,
}

#[derive(Debug, Parser)]
enum SubCommand {
    Startup {
        #[clap(value_enum, default_value_t = GameMode::Ttt)]
        gamemode: GameMode,
    },
    Shutdown,
    Update,
}

#[derive(Debug, Parser)]
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
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();
    let mut server = Garrys::new()?;

    match args.cmd {
        SubCommand::Startup { gamemode } => {
            server.gamemode = gamemode;
            server.startup()
        }
        SubCommand::Shutdown => server.shutdown(),
        SubCommand::Update => server.update(),
    }
}

struct Garrys {
    config: Config,
    pub gamemode: GameMode,
}

impl Garrys {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self {
            config,
            gamemode: GameMode::default(),
        })
    }
}

impl TmuxServer for Garrys {}

impl GameServer for Garrys {
    fn config(&self) -> &Config {
        &self.config
    }
    fn startup_inner(&self) -> Result<()> {
        // Don't start the server if the session is already running.
        self.ensure_session_not_open()?;

        let game_dir = self.config.game_dir();
        self.start_session(None)?;

        // Remove the old compiled server config to avoid caching fuckery
        let server_vdf = game_dir.join("garrysmod/cfg/server.vdf");
        if server_vdf.exists() {
            std::fs::remove_file(server_vdf)?;
        }

        // Load all secrets
        let mut secrets = HashMap::new();
        secrets.insert("password", self.config.default_password.clone());

        // Get the command by gamemode and copy the respective config file
        let server_command = match self.gamemode {
            GameMode::Ttt => {
                // Deploy the server config file
                copy_secret_file(
                    &self.config.default_config_dir().join("garrys/ttt.cfg"),
                    &game_dir.join("garrysmod/cfg/server.cfg"),
                    &secrets,
                )
                .wrap_err("Failed to copy ttt server config")?;

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
                    &self
                        .config
                        .default_config_dir()
                        .join("garrys/prop_hunt.cfg"),
                    &game_dir.join("garrysmod/cfg/server.cfg"),
                    &secrets,
                )
                .wrap_err("Failed to copy prophunt server config")?;

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
            "STEAM_WEB_API_KEY" => self.config.garrys.steam_web_api_key.clone()
        };
        self.send_input_newline_with_env(server_command, envs)?;

        Ok(())
    }

    fn update_inner(&self) -> Result<()> {
        // Check if the server is running and shut it down if it is.
        if self.is_session_open()? {
            println!("Shutting down running server");
            self.shutdown()?;
            sleep_seconds(10);
        }

        cmd!(
            r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 4020 \
        validate +quit"#,
            self.config.game_dir_str()
        )
        .run_success()?;

        Ok(())
    }

    fn shutdown_inner(&self) -> Result<()> {
        // Exit if the server is not running.
        self.ensure_session_is_open()?;

        self.send_ctrl_c()?;
        self.send_input_newline("exit")?;

        Ok(())
    }
}
