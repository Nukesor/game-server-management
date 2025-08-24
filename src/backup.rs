use std::{fs::remove_file, path::PathBuf};

use crate::{cmd, errors::*, process::*};

const DATEFORMAT_STRING: &str = "%Y-%m-%d_%H-%M";

/// Take a directory and back it up to the target directory.
/// The directory is tar.zst compressed and and saved in the following pattern:
/// "{save_name}_%Y-%m-%d_%H-%M.tar.zst"
///
/// In case there's already a backup with that name, delete the old one.
pub fn backup_directory(
    dir_to_backup: PathBuf,
    target_dir: PathBuf,
    save_name: &str,
) -> Result<PathBuf> {
    // Get path for the backup file
    let now = chrono::offset::Local::now();
    let dest: PathBuf = target_dir.join(format!(
        "{save_name}_{}.tar.zst",
        now.format(DATEFORMAT_STRING)
    ));

    let dest = target_dir.join(dest);

    // Remove any already existing backup file with the same name.
    if dest.exists() {
        remove_file(&dest)?;
    }

    info!("Backing up {dir_to_backup:?} to {dest:?}");
    cmd!(
        "tar -I zstd -cvf {} {}",
        dest.to_string_lossy(),
        dir_to_backup.to_string_lossy()
    )
    .run_success()?;

    Ok(dest)
}

/// Take a file and back it up to the target dir.
/// The file is saved in the following pattern:
/// "{save_name}_%Y-%m-%d_%H-%M.tar.zst"
///
/// In case there's already a backup with that name, delete the old one.
pub fn backup_file(
    file_to_backup: PathBuf,
    target_dir: PathBuf,
    save_name: &str,
    extension: &str,
) -> Result<PathBuf> {
    // Get path for the backup file
    let now = chrono::offset::Local::now();
    let dest: PathBuf = target_dir.join(format!(
        "{save_name}_{}.{extension}",
        now.format(DATEFORMAT_STRING)
    ));

    // Remove any already existing backup file with the same name.
    let dest = target_dir.join(dest);
    if dest.exists() {
        remove_file(&dest)?;
    }

    info!("Copying {file_to_backup:?} to {dest:?}");
    std::fs::copy(file_to_backup, &dest)?;

    Ok(dest)
}
