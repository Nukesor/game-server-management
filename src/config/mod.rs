use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

mod cs_go;
mod factorio;
mod garrys;

use cs_go::CsGo;
use factorio::Factorio;
use garrys::Garrys;

pub fn expand(path: &Path) -> PathBuf {
    PathBuf::from(tilde(&path.to_string_lossy()).into_owned())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// The root directory for all game files.
    game_files: PathBuf,
    /// The root directory for all game file backups.
    game_files_backup: PathBuf,
    /// The root location where games can write their backups to.
    backup_root: PathBuf,
    /// A temporary directory, which can be used during updates and other tasks.
    temp_dir: PathBuf,
    /// The default password that's used by these game-servers
    pub default_password: String,
    /// Game specific sub-configurations
    pub factorio: Factorio,
    pub cs_go: CsGo,
    pub garrys: Garrys,
}

impl Config {
    /// Either get the config from an existing configuration file or
    /// create a new one from scratch
    pub fn new() -> Result<Self> {
        let path = Config::get_config_path()?;

        // The config file exists. Try to parse it
        if path.exists() {
            let mut file = File::open(path)?;
            let mut config = String::new();
            file.read_to_string(&mut config)?;

            let config: Config = toml::from_str(&config)?;
            return Ok(config);
        }

        // No config exists yet. Create a default config and persist it onto disk.
        let default_config = Config {
            game_files: "~/game_servers/games/".into(),
            game_files_backup: "/var/lib/backup/games/game_files".into(),
            backup_root: "/var/lib/backup/games/".into(),
            temp_dir: "~/game_servers/tmp/".into(),
            default_password: "your pass".into(),
            factorio: Factorio::default(),
            cs_go: CsGo::default(),
            garrys: Garrys::default(),
        };
        default_config.write()?;

        Ok(default_config)
    }

    /// Write the current config to disk.
    pub fn write(&self) -> Result<()> {
        let path = Config::get_config_path()?;

        // The config file exists. Try to parse it
        let mut file = if path.exists() {
            File::open(path)?
        } else {
            File::create(path)?
        };

        let config = toml::to_string(&self)?;
        file.write_all(&config.as_bytes())?;

        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Couldn't find config dir")?;
        Ok(config_dir.join("games.toml"))
    }
}

impl Config {
    pub fn game_files(&self) -> PathBuf {
        expand(&self.game_files)
    }

    pub fn game_files_backup(&self) -> PathBuf {
        expand(&self.game_files_backup)
    }

    pub fn backup_root(&self) -> PathBuf {
        expand(&self.backup_root)
    }

    pub fn temp_dir(&self) -> PathBuf {
        expand(&self.temp_dir)
    }
}
