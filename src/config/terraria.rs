use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use super::expand;

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Terraria {
    /// The server port
    pub port: usize,
    /// The location of the terraria world file
    world_path: PathBuf,
    /// The name of the world
    pub world_name: String,
}

impl Terraria {
    pub fn world_path(&self) -> PathBuf {
        expand(&self.world_path)
    }
}
