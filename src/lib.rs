use std::time::Duration;

pub mod config;
pub mod path;
pub mod process;
pub mod secret;

pub fn sleep_seconds(seconds: u64) {
    let duration = Duration::from_secs(seconds);
    std::thread::sleep(duration);
}
