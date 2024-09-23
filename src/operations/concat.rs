use std::{fs, ops::Deref, path::Path};
use anyhow::{Result, anyhow, Context};
use crate::util::command_extensions::*;

pub fn concat_files(files: &[impl Deref<Target=Path>], dest: &Path) -> Result<()> {
    // println!("concatting {:#?}", files);

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
        .args(["-f", "concat", "-i"])
        .arg(&list_file)
        .args(["-c", "copy"])
        .arg(dest)
        .status()
        .expect("Error executing ffmpeg")
        .to_exitcode()
        .map_err(|x| anyhow!("ffmpeg command failed with code: {}", x))?;

    // TODO remove leftover files

    // ffmpeg -f concat -safe 0 -i mylist.txt -c copy output.wav

    Ok(())
}

