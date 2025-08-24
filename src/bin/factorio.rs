use std::{
    collections::HashMap,
    fs::{remove_dir_all, rename},
    path::PathBuf,
};

use clap::Parser;
use utils::prelude::*;

#[derive(Debug, Parser)]
enum SubCommand {
    Startup,
    Shutdown,
    /// Update the game to a specific version.
    /// The version is expected in the `1.1.34` format.
    Update {
        version: String,
    },
    Backup,
}

#[derive(Debug, Parser)]
#[clap(
    name = "Factorio",
    about = "A small binary to manage my factorio server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "factorio";

fn main() -> Result<()> {
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();

    match args.cmd {
        SubCommand::Startup => {
            let server = Factorio::new()?;
            server.startup()
        }
        SubCommand::Shutdown => {
            let server = Factorio::new()?;
            server.shutdown()
        }
        SubCommand::Backup => {
            let server = Factorio::new()?;
            server.backup()
        }
        SubCommand::Update { version } => {
            let server = Factorio::new_with_version(version)?;
            server.update()
        }
    }
}

struct Factorio {
    config: Config,
    version: Option<String>,
}

impl Factorio {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self {
            config,
            version: None,
        })
    }

    fn new_with_version(version: String) -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self {
            config,
            version: Some(version),
        })
    }
}

impl TmuxServer for Factorio {}

impl GameServer for Factorio {
    fn config(&self) -> &Config {
        &self.config
    }
    fn startup_inner(&self) -> Result<()> {
        // Exit if the server is not running.
        self.ensure_session_not_open()?;

        // Load all secrets
        let mut secrets = HashMap::new();
        secrets.insert("password", self.config.default_password.clone());

        // Deploy the server config file
        let server_config_path = self
            .config
            .game_dir()
            .join("config/custom-server-config.json");
        copy_secret_file(
            &self
                .config
                .default_config_dir()
                .join("factorio-server-settings.json"),
            &server_config_path,
            &secrets,
        )
        .wrap_err("Failed while copying server config file")?;

        // Create a new session for this instance
        self.start_session(None)?;

        let server_command = format!(
            "{}/bin/x64/factorio \
        --start-server-load-latest \
        --use-server-whitelist \
        --server-whitelist {} \
        --server-settings {}",
            self.config.game_dir_str(),
            self.config
                .game_dir()
                .join("config/server-whitelist.json")
                .to_string_lossy(),
            server_config_path.to_string_lossy(),
        );

        // Start the server
        self.send_input_newline(&server_command)?;

        Ok(())
    }

    fn backup_inner(&self) -> Result<()> {
        let save_file = get_newest_file(&self.config.game_dir().join("saves"))?;
        if let Some(file_to_backup) = save_file {
            backup_file(
                file_to_backup,
                self.config.create_backup_dir()?,
                "factorio",
                "zip",
            )?;
        }

        Ok(())
    }

    fn update_inner(&self) -> Result<()> {
        let version = self
            .version
            .as_ref()
            .ok_or_else(|| eyre!("Version not specified for update"))?;

        // Check if the server is running and shut it down if it is.
        if self.is_session_open()? {
            self.shutdown().wrap_err("Failed during shutdown")?;
        }

        let temp_dir = self.config.create_temp_dir()?;

        let files_to_backup = vec!["saves", "config", "mods", "mod-settings.json"];

        // Move all important files to a temporary directory
        info!("Moving game files away.");
        for file_to_backup in &files_to_backup {
            let path: PathBuf = self.config.game_dir().join(file_to_backup);
            let dest: PathBuf = temp_dir.join(file_to_backup);
            if path.exists() {
                info!("Backing up {path:?} to {dest:?}");
                rename(&path, &dest)?;
            }
        }

        // Download the file to the server file directory
        let url = format!("https://factorio.com/get-download/{version}/headless/linux64",);
        info!("Downloading file from {url}");
        let tar_name = format!("factorio_headless_x64_{version}.tar.xz");
        cmd!("http --download \"{url}\" > /tmp/{tar_name}").run_success()?;

        // Remove the factorio directory
        if self.config.game_dir().exists() {
            remove_dir_all(self.config.game_dir())?;
        }

        // Untar the server files to the game directory
        info!("Extracting file");
        cmd!("tar xf /tmp/{} -C {}", tar_name, self.config.game_dir_str()).run_success()?;

        // Move the files back in place
        info!("Restoring game files.");
        for file_to_backup in &files_to_backup {
            let path: PathBuf = temp_dir.join(file_to_backup);
            let dest: PathBuf = self.config.game_dir().join(file_to_backup);
            if path.exists() {
                rename(&path, &dest)?;
                info!("Restoring {dest:?} from {path:?}");
            }
        }

        self.startup().wrap_err("Failed during startup:")
    }

    fn shutdown_inner(&self) -> Result<()> {
        // Exit if the server is not running.
        self.ensure_session_is_open()?;

        // Send Ctrl+C and wait a few seconds to save the map and shutdown the server
        self.send_ctrl_c()?;

        info!("Giving the server some time to shut down.");
        sleep_seconds(5);

        // Backup the map
        self.backup().wrap_err("Failed during backup:")?;

        // Exit the session
        self.send_input_newline("exit")?;

        Ok(())
    }
}
