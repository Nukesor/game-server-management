use std::{fs::create_dir, os::unix::fs::symlink};

use clap::Parser;
use color_eyre::{Result, eyre::WrapErr};
use utils::prelude::*;

#[derive(Debug, Parser)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Debug, Parser)]
#[clap(
    name = "Satisfactory",
    about = "A small binary to manage my Satisfactory server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "satisfactory";

fn main() -> Result<()> {
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();
    let server = Satisfactory::new()?;

    match args.cmd {
        SubCommand::Startup => server.startup(),
        SubCommand::Shutdown => server.shutdown(),
        SubCommand::Update => server.update(),
    }
}

struct Satisfactory {
    config: Config,
}

impl Satisfactory {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self { config })
    }
}

impl TmuxServer for Satisfactory {}

impl GameServer for Satisfactory {
    fn config(&self) -> &Config {
        &self.config
    }
    fn startup_inner(&self) -> Result<()> {
        // Don't start the server if the session is already running.
        self.ensure_session_not_open()?;

        // Satisfactory expects the steamclient.so library to be at a different location.
        // We create a symlink to the expected location.
        let folder = expand_home("~/.steam/steamcmd/sdk64/");
        if !folder.exists() {
            create_dir(folder)?;
        }
        let link_src = expand_home("~/.steam/steamcmd/linux64/steamclient.so");
        let link_dest = expand_home("~/.steam/steamcmd/sdk64/steamclient.so");
        if link_src.exists() && !link_dest.exists() {
            symlink(link_src, link_dest)?;
        }

        // Create a new session for this instance
        self.start_session(None)?;

        self.send_input_newline("./FactoryServer.sh")?;

        Ok(())
    }

    fn update_inner(&self) -> Result<()> {
        // Check if the server is running and shut it down if it is.
        if self.is_session_open()? {
            self.shutdown()?;
            sleep_seconds(10)
        }

        // The Satisfactory server has the id 1690800.
        cmd!(
            r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 1690800 \
        validate +quit"#,
            self.config.game_dir_str()
        )
        .io_passthrough()
        .run_success()?;

        self.startup()
    }

    fn shutdown_inner(&self) -> Result<()> {
        // Exit if the server is not running.
        self.ensure_session_is_open()?;

        self.send_ctrl_c()?;
        self.send_input_newline("exit")?;

        Ok(())
    }
}
