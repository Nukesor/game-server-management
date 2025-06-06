use std::{
    fs::{File, create_dir_all},
    io::prelude::*,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

mod cs_go;
mod garrys;
mod terraria;

use cs_go::CsGo;
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
    /// The location for default config files (the files contained in this repo)
    default_config_dir: PathBuf,
    /// The default password that's used by these game-servers
    pub default_password: String,
    /// For steam games, use this ID as the admin.
    #[serde(default)]
    pub admin_steam_id: String,
    /// Game specific sub-configurations
    #[serde(default)]
    pub cs_go: CsGo,
    #[serde(default)]
    pub garrys: Garrys,
    #[serde(default)]
    pub terraria: Terraria,
}

impl Config {
    /// Either get the config from an existing configuration file or
    /// create a new one from scratch
    ///
    /// `game_name` and `instance` are used to automatically build the default game file and backup
    /// paths for you. Games that don't have multiple instances, just provide `None` as argument
    /// and it will be ignored.
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
            admin_steam_id: "".into(),
            default_config_dir: "~/server_management".into(),
            cs_go: CsGo::default(),
            garrys: Garrys::default(),
            terraria: Terraria::default(),
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

    pub fn default_config_dir(&self) -> PathBuf {
        expand(&self.default_config_dir)
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

    pub fn game_dir(&self) -> PathBuf {
        expand(&self.game_file_root).join(self.game_subpath())
    }

    pub fn create_game_dir(&self) -> Result<PathBuf> {
        let game_dir = expand(&self.game_file_root).join(self.game_subpath());
        create_dir_all(&game_dir).context(format!("Failed to create game dir: {game_dir:?}"))?;
        Ok(game_dir)
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

    pub fn create_backup_dir(&self) -> Result<PathBuf> {
        let backup_dir = expand(&self.backup_root).join(self.game_subpath());
        create_dir_all(&backup_dir)
            .context(format!("Failed to create backup dir: {backup_dir:?}"))?;
        Ok(backup_dir)
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

    pub fn create_temp_dir(&self) -> Result<PathBuf> {
        let temp_dir = expand(&self.temp_file_root).join(self.game_subpath());
        create_dir_all(&temp_dir).context(format!("Failed to create temp dir: {temp_dir:?}"))?;
        Ok(temp_dir)
    }

    pub fn temp_dir_str(&self) -> String {
        self.temp_dir().to_string_lossy().to_string()
    }
}
