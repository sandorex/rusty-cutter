use std::path::Path;
use anyhow::{Result, anyhow};

pub fn concat_files(files: &[impl AsRef<Path> + std::fmt::Debug], dest: &Path) -> Result<()> {
    println!("concatting {:#?}", files);

    Ok(())
}

