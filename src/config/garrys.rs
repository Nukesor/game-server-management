use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use super::expand;

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Garrys {
    /// The location of the ttt server config
    ttt_server_config: PathBuf,

    /// The location of the prophunt server config
    prophunt_server_config: PathBuf,

    /// This token can be received over here:
    /// https://steamcommunity.com/dev/managegameservers
    ///
    /// The app Id for the CS:GO client is 730. This should be used!
    pub steam_web_api_key: String,
}

impl Garrys {
    pub fn ttt_server_config(&self) -> PathBuf {
        expand(&self.ttt_server_config)
    }

    pub fn prophunt_server_config(&self) -> PathBuf {
        expand(&self.prophunt_server_config)
    }
}
