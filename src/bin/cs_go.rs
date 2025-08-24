use std::{collections::HashMap, fs::create_dir, os::unix::fs::symlink};

use clap::Parser;
use utils::prelude::*;

#[derive(Debug, Parser)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Debug, Parser)]
#[clap(name = "CS:GO", about = "A small binary to manage my CS:GO server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "csgo";

fn main() -> Result<()> {
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();
    let server = CsGo::new()?;

    match args.cmd {
        SubCommand::Startup => server.startup(),
        SubCommand::Shutdown => server.shutdown(),
        SubCommand::Update => server.update(),
    }
}

struct CsGo {
    config: Config,
}

impl CsGo {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self { config })
    }
}

impl TmuxServer for CsGo {}

impl GameServer for CsGo {
    fn config(&self) -> &Config {
        &self.config
    }

    fn startup_inner(&self) -> Result<()> {
        // Don't start the server if the session is already running.
        self.ensure_session_not_open()?;

        // Create a new session for this instance
        self.start_session(None)?;

        // CS:GO expects the steamclient.so library to be at a different location.
        // Hence, we create a symlink to the expected location.
        let folder = expand_home("~/.steam/sdk32/");
        if !folder.exists() {
            create_dir(folder)?;
        }
        let link_src = expand_home("~/.steam/steamcmd/linux32/steamclient.so");
        let link_dest = expand_home("~/.steam/sdk32/steamclient.so");
        if link_src.exists() && !link_dest.exists() {
            symlink(link_src, link_dest)?;
        }

        // Load all secrets
        let mut secrets = HashMap::new();
        secrets.insert("password", self.config.default_password.clone());

        // Get the command by gamemode and copy the respective config file
        copy_secret_file(
            &self.config.default_config_dir().join("csgo.cfg"),
            &self.config.game_dir().join("csgo/cfg/server.cfg"),
            &secrets,
        )?;

        let mut server_command = concat!(
            "./srcds_run ",
            "-console ",
            "-game csgo ",
            "-ip 0.0.0.0 ",
            "-usercon ",
            "+map de_dust2 ",
            "+game_type 0 ",
            "+game_mode 1 ",
            "+mapgroup mg_active ",
        )
        .to_string();
        server_command.push_str(&format!(
            "+sv_setsteamaccount {} ",
            self.config.cs_go.login_token
        ));

        self.send_input_newline(&server_command)?;

        Ok(())
    }

    fn update_inner(&self) -> Result<()> {
        // Check if the server is running and shut it down if it is.
        if self.is_session_open()? {
            println!("Shutting down running server");
            self.shutdown()?;
            sleep_seconds(10);
        }

        // The CS:GO server has the id 740.
        cmd!(
            r#"steamcmd \
        +force_install_dir {} \
        +login anonymous \
        +app_update 740 \
        validate +quit"#,
            self.config.game_dir_str()
        )
        .run_success()?;

        // Restart the server
        self.startup()?;

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
