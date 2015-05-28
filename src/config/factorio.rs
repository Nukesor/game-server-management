use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use super::expand;

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Factorio {
    /// The location of the factorio server config
    server_config: PathBuf,
}

impl Factorio {
    pub fn server_config(&self) -> PathBuf {
        expand(&self.server_config)
    }
}
