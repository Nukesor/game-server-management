use serde_derive::{Deserialize, Serialize};

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CsGo {
    /// This token can be received over here:
    /// https://steamcommunity.com/dev/managegameservers
    ///
    /// The app Id for the CS:GO client is 730. This should be used!
    pub login_token: String,
}
