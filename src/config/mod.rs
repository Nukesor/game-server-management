use std::fs::{File, create_dir_all};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

mod cod4;
mod cs_go;
mod factorio;
mod garrys;
mod terraria;

use cod4::Cod4;
use cs_go::CsGo;
use factorio::Factorio;
use garrys::Garrys;
use terraria::Terraria;

pub fn expand(path: &Path) -> PathBuf {
    PathBuf::from(tilde(&path.to_string_lossy()).into_owned())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// The name of the game this config is currently used with.
    #[serde(default)]
    pub game_name: String,
    /// The name of the game this config is currently used with.
    #[serde(default)]
    pub instance: Option<String>,
    /// The root directory for all game files.
    game_file_root: PathBuf,
    /// The root location where games can write their backups to.
    backup_root: PathBuf,
    /// A temporary directory, which can be used during updates and other tasks.
    temp_file_root: PathBuf,
    /// The default password that's used by these game-servers
    pub default_password: String,
    /// Game specific sub-configurations
    pub factorio: Factorio,
    #[serde(default)]
    pub cs_go: CsGo,
    #[serde(default)]
    pub garrys: Garrys,
    #[serde(default)]
    pub terraria: Terraria,
    #[serde(default)]
    pub cod4: Cod4,
}

impl Config {
    /// Either get the config from an existing configuration file or
    /// create a new one from scratch
    ///
    /// `game_name` and `instance` are used to automatically build the default game file and backup paths for you.
    /// Games that don't have multiple instances, just provide `None` as argument and it will be ignored.
    pub fn new(game_name: &str) -> Result<Self> {
        let path = Config::get_config_path()?;

        // The config file exists. Try to parse it
        if path.exists() {
            let mut file = File::open(path)?;
            let mut config = String::new();
            file.read_to_string(&mut config)?;

            let mut config: Config = toml::from_str(&config)?;
            config.game_name = game_name.to_string();
            return Ok(config);
        }

        // No config exists yet. Create a default config and persist it onto disk.
        let default_config = Config {
            game_name: game_name.to_string(),
            instance: None,
            game_file_root: "~/game_servers/games/".into(),
            backup_root: "/var/lib/backup/games/".into(),
            temp_file_root: "~/game_servers/tmp/".into(),
            default_password: "your pass".into(),
            factorio: Factorio::default(),
            cs_go: CsGo::default(),
            garrys: Garrys::default(),
            terraria: Terraria::default(),
            cod4: Cod4::default(),
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
        file.write_all(config.as_bytes())?;

        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Couldn't find config dir")?;
        Ok(config_dir.join("games.toml"))
    }
}

impl Config {
    pub fn game_root(&self) -> PathBuf {
        expand(&self.game_file_root)
    }

    /// Return the session name for this game.
    /// Either `game_name` or `{game_name}-{instance}` if an instance is selected.
    pub fn session_name(&self) -> String {
        if let Some(instance) = &self.instance {
            format!("{}-{instance}", self.game_name)
        } else {
            self.game_name.to_string()
        }
    }

    /// Return the sub-path for this game.
    /// Either `game_name` or `{game_name}/{instance}` if an instance is selected.
    pub fn game_subpath(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.game_name);
        if let Some(instance) = &self.instance {
            path = path.join(instance)
        }
        path
    }

    pub fn create_all_dirs(&self) -> Result<()> {
        create_dir_all(self.game_dir())
            .context(format!("Failed to create game dir: {:?}", self.game_dir()))?;
        create_dir_all(self.backup_dir()).context(format!(
            "Failed to create backup dir: {:?}",
            self.backup_dir()
        ))?;

        Ok(())
    }

    pub fn game_dir(&self) -> PathBuf {
        expand(&self.game_file_root).join(self.game_subpath())
    }

    pub fn game_dir_str(&self) -> String {
        self.game_dir().to_string_lossy().to_string()
    }

    pub fn backup_root(&self) -> PathBuf {
        expand(&self.backup_root)
    }

    pub fn backup_dir(&self) -> PathBuf {
        expand(&self.backup_root).join(self.game_subpath())
    }

    pub fn backup_dir_str(&self) -> String {
        self.backup_dir().to_string_lossy().to_string()
    }

    pub fn temp_root(&self) -> PathBuf {
        expand(&self.temp_file_root)
    }

    pub fn temp_dir(&self) -> PathBuf {
        expand(&self.temp_file_root).join(self.game_subpath())
    }

    pub fn temp_dir_str(&self) -> String {
        self.temp_dir().to_string_lossy().to_string()
    }
}
