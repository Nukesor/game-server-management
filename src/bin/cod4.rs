use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use utils::cmd;
use utils::config::Config;
use utils::process::*;
use utils::secret::copy_secret_file;

#[derive(Parser, Debug, ValueEnum, Clone)]
enum GameMode {
    Normal,
    Promod,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Startup {
        #[clap(value_enum, default_value_t = GameMode::Normal)]
        gamemode: GameMode,
    },
    Shutdown,
}

#[derive(Parser, Debug)]
#[clap(name = "CoD4", about = "A small binary to manage my Cod4 server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which returns the cod4 server dir from a given config.
fn cod4_dir(config: &Config) -> PathBuf {
    config.game_files().join("cod4")
}

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup { gamemode } => startup(&config, gamemode),
        SubCommand::Shutdown => shutdown(),
    }
}

fn startup(config: &Config, gamemode: GameMode) -> Result<()> {
    // Don't start the server if the tmux shell is already running.
    let exit_status = cmd!("tmux has-session -t cod").run()?;
    if exit_status.success() {
        return Ok(());
    }

    cmd!("tmux new -d -s cod")
        .cwd(cod4_dir(config))
        .run_success()?;

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password", config.default_password.clone());

    // Deploy the default config file
    copy_secret_file(
        &config.cod4.default_config_path(),
        &cod4_dir(config).join("main/default.cfg"),
        &secrets,
    )
    .context("Failed to copy default cod4 config")?;

    let server_command = match gamemode {
        GameMode::Normal => std::concat!(
            "./cod4_lnxded ",
            "+exec default.cfg ",
            "+set fs_homepath ./ ",
            "+set sv_punkbuster 0 ",
            "+map_rotate",
        ),

        GameMode::Promod => {
            // Deploy the promod config file
            copy_secret_file(
                &config.cod4.promod_config_path(),
                &cod4_dir(config).join("main/promod.cfg"),
                &secrets,
            )
            .context("Failed to copy default cod4 config")?;

            std::concat!(
                "./games/cod4/cod4_lnxded ",
                "+exec default.cfg ",
                "+exec promod.cfg ",
                "+set fs_homepath ./ ",
                "+set fs_game mods/pml220 ",
                "+set sv_punkbuster 0 ",
                "+map_rotate",
            )
        }
    };

    cmd!("tmux send -t cod '{}' ENTER", server_command).run_success()
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t cod C-c").run_success()?;
    cmd!("tmux send-keys -t cod exit ENTER").run_success()?;

    Ok(())
}
