use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use shellexpand::tilde;

use crate::errors::*;

pub fn expand_home<T: ToString>(path: T) -> PathBuf {
    PathBuf::from(&tilde(&path.to_string()).to_string())
}

pub fn get_newest_file(path: &Path) -> Result<Option<PathBuf>> {
    let dir = std::fs::read_dir(path)?;

    let mut path: Option<PathBuf> = None;
    let mut modified = SystemTime::UNIX_EPOCH;

    for entry_result in dir {
        let entry = entry_result?;
        let metadata = entry.metadata()?;

        if path.is_none() {
            path = Some(entry.path());
            modified = metadata.modified()?;
            continue;
        }

        let last_modified = metadata.modified()?;
        if last_modified > modified {
            modified = last_modified;
            path = Some(entry.path());
        }
    }

    Ok(path)
}
