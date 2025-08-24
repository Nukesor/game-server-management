use std::time::Duration;

pub mod backup;
pub mod config;
pub mod game_server;
pub mod path;
pub mod process;
pub mod secret;
pub mod tmux;
pub mod zellij;

pub fn sleep_seconds(seconds: u64) {
    let duration = Duration::from_secs(seconds);
    std::thread::sleep(duration);
}
