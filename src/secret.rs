use std::{
    collections::HashMap,
    fs::{File, remove_file},
    io::{Read, Write},
    path::Path,
};

use crate::errors::*;

/// This function takes a secret file, replaces all placeholders with the given secrets and writes
/// the final version of the file to the specified location.
///
/// The placeholders have to be written like this `{{ name }}`.
/// The secrets are passed to this function via HashMap<name, value>.
pub fn copy_secret_file(src: &Path, dest: &Path, secrets: &HashMap<&str, String>) -> Result<()> {
    // Remove the destination if it already exists
    if dest.exists() {
        remove_file(dest).wrap_err("Failed deleting old config file")?;
    }

    let mut src = File::open(src).wrap_err("Failed to open source config file")?;
    let mut content = String::new();
    src.read_to_string(&mut content)
        .wrap_err("Failed to read source config file")?;

    for (key, value) in secrets {
        content = content.replace(&format!("{{{{ {key} }}}}"), value);
    }

     File::create(dest)
        .wrap_err("Failed to create destination config file")?
        .write_all(content.as_bytes())
        .wrap_err("Failed to write to destination config file")?;

    Ok(())
}
