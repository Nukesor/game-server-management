use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use utils::prelude::*;

#[derive(Debug, Parser)]
enum SubCommand {
    Startup,
    Backup,
    Update,
    Shutdown,
}

#[derive(Debug, Parser)]
#[clap(
    name = "Abiotic Factor Server",
    about = "A small binary to manage my Abiotic Factor server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "abiotic-factor";
const WORLD_SAVE_NAME: &str = "MadLab";

fn server_dir(config: &Config) -> PathBuf {
    config
        .game_dir()
        .join("AbioticFactor/Saved/SaveGames/Server")
}

fn world_dir(config: &Config) -> PathBuf {
    server_dir(config).join("Worlds").join(WORLD_SAVE_NAME)
}

fn main() -> Result<()> {
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();
    let server = AbioticFactor::new()?;

    match args.cmd {
        SubCommand::Startup => server.startup(),
        SubCommand::Backup => server.backup(),
        SubCommand::Update => server.update(),
        SubCommand::Shutdown => server.shutdown(),
    }
}

struct AbioticFactor {
    config: Config,
}

impl AbioticFactor {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self { config })
    }
}

impl TmuxServer for AbioticFactor {}

impl GameServer for AbioticFactor {
    fn config(&self) -> &Config {
        &self.config
    }

    fn startup_inner(&self) -> Result<()> {
        // Don't start the server if the session is already running.
        self.ensure_session_not_open()?;

        // Create a new session for this instance
        self.start_session(None)?;

        let mut secrets = HashMap::new();
        secrets.insert("admin_steam_id", self.config.admin_steam_id.clone());

        // Copy the world-spanning admin settings
        copy_secret_file(
            &self
                .config
                .default_config_dir()
                .join("abiotic_factor/Admin.ini"),
            &server_dir(&self.config).join("Admin.ini"),
            &secrets,
        )?;

        // Copy the world config file.
        copy_secret_file(
            &self
                .config
                .default_config_dir()
                .join("abiotic_factor/AbioticFactor.ini"),
            &world_dir(&self.config).join("SandboxSettings.ini"),
            &secrets,
        )?;

        let mut server_command = concat!(
            "WINEDEBUG=fixme-all ",
            "wine ./AbioticFactor/Binaries/Win64/AbioticFactorServer-Win64-Shipping.exe ",
            "-log ",
            "-newconsole ",
            "-useperfthreads ",
            "-NoAsyncLoadingThread ",
            r#"-SteamServerName="MadLab Hamburg" "#,
            "-PORT=7780 ",
            "-QueryPort=7781 ",
            "-MaxServerPlayers=6 ",
        )
        .to_string();
        server_command.push_str(&format!("-WorldSaveName={WORLD_SAVE_NAME} "));
        server_command.push_str(&format!(
            r#"-ServerPassword="{}" "#,
            self.config.default_password
        ));

        self.send_input_newline(&server_command)?;

        Ok(())
    }

    /// Save the game.
    ///
    /// There's currently no way to force saving via the CLI.
    /// The game apparently saves automatically from time to time, so we have to rely on that.
    /// It's seemingly possible to force saving via the admin interface as well.
    fn backup_inner(&self) -> Result<()> {
        backup_directory(
            world_dir(&self.config),
            self.config.create_backup_dir()?,
            WORLD_SAVE_NAME,
        )?;

        Ok(())
    }

    fn update_inner(&self) -> Result<()> {
        // Check if the server is running and shut it down if it is.
        if self.is_session_open()? {
            println!("Shutting down running server");
            self.shutdown()?;
            // Shutdown twice, as the server doesn't react to CTRL+C for some reason.
            self.shutdown()?;
            sleep_seconds(10);
        }

        // Run a quick backup for good measure.
        self.backup()?;

        // The abiotic factor server has the id 2857200 .
        cmd!(
            r#"steamcmd \
        +@sSteamCmdForcePlatformType windows \
        +force_install_dir {} \
        +login anonymous \
        +app_update 2857200 \
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
