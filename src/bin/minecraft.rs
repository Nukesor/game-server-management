use clap::Parser;
use utils::prelude::*;

#[derive(Debug, Parser)]
enum SubCommand {
    Startup { instance: String },
    Shutdown { instance: String },
    Backup { instance: String },
}

#[derive(Debug, Parser)]
#[clap(
    name = "Minecraft",
    about = "A small binary to manage my Minecraft server"
)]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "minecraft";

fn main() -> Result<()> {
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();
    let mut server = Minecraft::new()?;

    match args.cmd {
        SubCommand::Startup { instance } => {
            server.set_instance(instance);
            server.startup()
        }
        SubCommand::Shutdown { instance } => {
            server.set_instance(instance);
            server.shutdown()
        }
        SubCommand::Backup { instance } => {
            server.set_instance(instance);
            server.backup()
        }
    }
}

struct Minecraft {
    config: Config,
}

impl Minecraft {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self { config })
    }

    fn set_instance(&mut self, instance: String) {
        self.config.instance = Some(instance);
    }
}

impl TmuxServer for Minecraft {}

impl GameServer for Minecraft {
    fn config(&self) -> &Config {
        &self.config
    }
    fn startup_inner(&self) -> Result<()> {
        // Don't start the server if the session is already running.
        self.ensure_session_not_open()?;

        // Create a new session for this instance
        self.start_session(None)?;

        // Start the server
        self.send_input_newline("./ServerStart.sh")?;

        Ok(())
    }

    fn backup_inner(&self) -> Result<()> {
        // Inform users and save the map if the server is running.
        if self.is_session_open()? {
            // Send a backup message
            self.send_input_newline("/say Running full backup")?;

            // Save the world to disk
            self.send_input_newline("/save-all flush")?;

            // Wait for at least a minute to give minecraft enough time to write the backup
            let delay = std::time::Duration::from_millis(60000);
            std::thread::sleep(delay);
        }

        backup_directory(
            self.config.game_dir(),
            self.config.create_backup_dir()?,
            &self.config.session_name(),
        )?;

        Ok(())
    }

    fn shutdown_inner(&self) -> Result<()> {
        // Exit if the server is not running.
        self.ensure_session_is_open()?;

        self.backup()?;

        // Send Ctrl+C and exit
        self.send_input_newline("/say Server is gracefully shutting down")?;
        self.send_input_newline("/stop")?;

        // Wait for at least a minute to give minecraft enough time to gracefully shutdown
        let delay = std::time::Duration::from_millis(60000);
        std::thread::sleep(delay);

        // Exit the session
        self.send_input_newline("exit")?;

        Ok(())
    }
}
