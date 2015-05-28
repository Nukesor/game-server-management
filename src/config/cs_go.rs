use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use super::expand;

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CsGo {
    /// The location of the csgo server config
    server_config: PathBuf,

    /// This token can be received over here:
    /// https://steamcommunity.com/dev/managegameservers
    ///
    /// The app Id for the CS:GO client is 730. This should be used!
    pub login_token: String,
}

impl CsGo {
    pub fn server_config(&self) -> PathBuf {
        expand(&self.server_config)
    }
}
