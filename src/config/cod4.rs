use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use super::expand;

/// Everything necessary for the cod4 setup.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Cod4 {
    /// The location of the default server config
    default_config: PathBuf,

    /// The location of the promod server config
    promod_config: PathBuf,
}

impl Cod4 {
    pub fn default_config_path(&self) -> PathBuf {
        expand(&self.default_config)
    }

    pub fn promod_config_path(&self) -> PathBuf {
        expand(&self.promod_config)
    }
}
