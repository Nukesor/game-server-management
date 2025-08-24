use std::time::Duration;

pub mod backup;
pub mod config;
pub mod game_server;
pub mod log;
pub mod path;
pub mod process;
pub mod secret;
pub mod tmux;
pub mod zellij;
pub fn sleep_seconds(seconds: u64) {
    let duration = Duration::from_secs(seconds);
    std::thread::sleep(duration);
}
#[allow(unused_imports)]
pub mod prelude {
    pub use crate::{
        backup::*,
        cmd,
        config::Config,
        errors::*,
        game_server::GameServer,
        log::install_tracing,
        path::*,
        process::*,
        secret::copy_secret_file,
        sleep_seconds,
        tmux::*,
    };
}
#[allow(unused_imports)]
pub(crate) mod errors {
    pub use color_eyre::{
        Result,
        eyre::{WrapErr, bail, eyre},
    };
    pub use tracing::{debug, error, info, trace, warn};
}
