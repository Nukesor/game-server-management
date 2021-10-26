use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use script_utils::cmd;
use script_utils::config::Config;
use script_utils::process::*;
use script_utils::secret::copy_secret_file;

#[derive(Parser, Debug)]
enum SubCommand {
    Startup,
    Shutdown,
    Update,
}

#[derive(Parser, Debug)]
#[clap(name = "CS:GO", about = "A small binary to manage my CS:GO server")]
struct CliArguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Small helper which creates the factorio server dir from a given config.
fn csgo_dir(config: &Config) -> PathBuf {
    config.game_files().join("cs_go")
}

fn main() -> Result<()> {
    // Parse commandline options.
    let args = CliArguments::parse();
    let config = Config::new().context("Failed to read config:")?;

    match args.cmd {
        SubCommand::Startup => startup(&config),
        SubCommand::Shutdown => shutdown(),
        SubCommand::Update => update(&config),
    }
}

fn startup(config: &Config) -> Result<()> {
    let exit_status = cmd!("tmux has-session -t csgo").run()?;

    // Don't start the server if the tmux shell is already running.
    if exit_status.success() {
        return Ok(());
    }

    cmd!("tmux new -d -s csgo")
        .cwd(csgo_dir(config))
        .run_success()?;

    // Load all secrets
    let mut secrets = HashMap::new();
    secrets.insert("password".into(), config.default_password.clone());

    // Get the command by gamemode and copy the respective config file
    copy_secret_file(
        &config.cs_go.server_config(),
        &csgo_dir(config).join("csgo/cfg/server.cfg"),
        secrets,
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
        config.cs_go.login_token
    ));

    cmd!("tmux send -t csgo '{}' ENTER", server_command).run_success()
}

fn update(config: &Config) -> Result<()> {
    // The CS:GO server has the id 740.
    cmd!(
        "steamcmd +login anonymous \
        +force_install_dir {} \
        +app_update 740 \
        validate +quit",
        csgo_dir(config).to_string_lossy()
    )
    .run_success()
}

fn shutdown() -> Result<()> {
    cmd!("tmux send-keys -t csgo C-c").run_success()?;
    cmd!("tmux send-keys -t csgo exit ENTER").run_success()?;

    Ok(())
}
