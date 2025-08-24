use std::collections::HashMap;

use clap::{Parser, ValueEnum};
use utils::prelude::*;

#[derive(Clone, Copy, Debug, Default, Parser, ValueEnum)]
enum GameMode {
    #[default]
    Normal,
    Promod,
}

#[derive(Debug, Parser)]
enum SubCommand {
    Startup {
        #[clap(value_enum, default_value_t = GameMode::Normal)]
        gamemode: GameMode,
    },
    Shutdown,
}

#[derive(Debug, Parser)]
#[clap(name = "CoD4", about = "A small binary to manage my Cod4 server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "cod";

fn main() -> Result<()> {
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();
    let mut server = Cod4::new()?;

    match args.cmd {
        SubCommand::Startup { gamemode } => {
            server.gamemode = gamemode;
            server.startup()
        }
        SubCommand::Shutdown => server.shutdown(),
    }
}

struct Cod4 {
    config: Config,
    pub gamemode: GameMode,
}

impl Cod4 {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self {
            config,
            gamemode: GameMode::default(),
        })
    }
}

impl TmuxServer for Cod4 {}

impl GameServer for Cod4 {
    fn config(&self) -> &Config {
        &self.config
    }
    fn startup_inner(&self) -> Result<()> {
        // Don't start the server if the session is already running.
        self.ensure_session_not_open()?;

        // Create a new session for this instance
        self.start_session(None)?;

        // Load all secrets
        let mut secrets = HashMap::new();
        secrets.insert("password", self.config.default_password.clone());

        // Deploy the default config file
        copy_secret_file(
            &self.config.default_config_dir().join("cod4/default.cfg"),
            &self.config.game_dir().join("main/default.cfg"),
            &secrets,
        )
        .wrap_err("Failed to copy default cod4 config")?;

        let server_command = match self.gamemode {
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
                    &self.config.default_config_dir().join("cod4/promod.cfg"),
                    &self.config.game_dir().join("main/promod.cfg"),
                    &secrets,
                )
                .wrap_err("Failed to copy default cod4 config")?;

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

        self.send_input_newline(server_command)?;

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
