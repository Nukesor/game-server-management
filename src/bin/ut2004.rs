use clap::{Parser, ValueEnum};
use utils::prelude::*;

#[derive(Clone, Copy, Debug, Default, Parser, ValueEnum)]
enum GameMode {
    #[default]
    Am,
    Tam,
}

#[derive(Debug, Parser)]
enum SubCommand {
    Startup {
        #[clap(value_enum, default_value_t = GameMode::Am)]
        gamemode: GameMode,
    },
    Shutdown,
}

#[derive(Debug, Parser)]
#[clap(name = "UT2004", about = "A small binary to manage my UT2004 server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

const GAME_NAME: &str = "ut";

fn main() -> Result<()> {
    install_tracing()?;

    // Parse commandline options.
    let args = CliArguments::parse();
    let mut server = Ut2004::new()?;

    match args.cmd {
        SubCommand::Startup { gamemode } => {
            server.gamemode = gamemode;
            server.startup()
        }
        SubCommand::Shutdown => server.shutdown(),
    }
}

struct Ut2004 {
    config: Config,
    pub gamemode: GameMode,
}

impl Ut2004 {
    fn new() -> Result<Self> {
        let config = Config::new(GAME_NAME).wrap_err("Failed to read config")?;
        Ok(Self {
            config,
            gamemode: GameMode::default(),
        })
    }
}

impl TmuxServer for Ut2004 {}

impl GameServer for Ut2004 {
    fn config(&self) -> &Config {
        &self.config
    }
    fn startup_inner(&self) -> Result<()> {
        // Don't start the server if the session is already running.
        self.ensure_session_not_open()?;

        // Create a new session for this instance
        self.start_session(Some(self.config.game_dir().join("System")))?;

        let server_command = match self.gamemode {
            GameMode::Tam => std::concat!(
                "./ucc-bin server ",
                "\"DM-Asbestos",
                "?game=3SPNv3141.TeamArenaMaster",
                "?AdminName=private",
                "?AdminPassword={{ password }}\" ",
                "ini=ut2004.ini ",
                "-nohomedir",
            ),
            GameMode::Am => std::concat!(
                "./ucc-bin server ",
                "\"DM-Asbestos",
                "?game=3SPNv3141.ArenaMaster",
                "?AdminName=private",
                "?AdminPassword={{ password }}\" ",
                "ini=ut2004.ini ",
                "-nohomedir",
            ),
        };

        let server_command =
            server_command.replace("{{ password }}", &self.config.default_password);
        self.send_input_newline(&server_command)?;

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
