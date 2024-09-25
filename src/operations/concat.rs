use std::{fs, ops::Deref, path::Path};
use anyhow::{Result, anyhow, Context};
use crate::util::extensions::command_extensions::*;

pub fn concat_files(files: &[impl Deref<Target=Path>], dest: impl Deref<Target=Path>) -> Result<()> {
    let list_file = dest.with_extension("txt");

    // make the file up
    let mut list = String::new();
    for file in files {
        list.push_str("file '");
        list.push_str(&file.to_string_lossy());
        list.push_str("'\n");
    }

    fs::write(&list_file, list)
        .with_context(|| format!("Error while writing to {:?}", list_file))?;

    Command::new("ffmpeg")
        .args([
            // print only errors
            "-loglevel", "error",

            // do not ask to overwrite
            "-y",

            "-f", "concat",

            // use following file for the concat list
            "-i"
        ])
        .arg(&list_file)
        .args(["-c", "copy"])
        .arg(dest.as_os_str())
        .status()
        .expect("Error executing ffmpeg")
        .to_exitcode()
        .map_err(|x| anyhow!("ffmpeg command failed with code: {}", x))?;

    // TODO remove leftover files

    Ok(())
}

