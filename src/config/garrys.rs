use serde_derive::{Deserialize, Serialize};

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Garrys {
    /// This token can be received over here:
    /// https://steamcommunity.com/dev/managegameservers
    ///
    /// The app Id for the CS:GO client is 730. This should be used!
    pub steam_web_api_key: String,
}
