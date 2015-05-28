use std::collections::HashMap;
use std::fs::{remove_file, File};
use std::io::{Read, Write};
use std::path::Path;

use anyhow::Result;

/// This function takes a secret file, replaces all placeholders with the given secrets and writes
/// the final version of the file to the specified location.
///
/// The placeholders have to be written like this `{{ name }}`.
/// The secrets are passed to this function via HashMap<name, value>.
pub fn copy_secret_file(src: &Path, dest: &Path, secrets: HashMap<String, String>) -> Result<()> {
    // Remove the destination if it already exists
    if dest.exists() {
        remove_file(dest)?;
    }

    let mut src = File::open(src)?;
    let mut content = String::new();
    src.read_to_string(&mut content)?;

    for (key, value) in secrets {
        content = content.replace(&format!("{{ {} }}", key), &value);
    }

    let mut dest = File::create(dest)?;
    dest.write_all(content.as_bytes())?;

    Ok(())
}
