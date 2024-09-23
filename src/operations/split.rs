use crate::{util::extensions::{command_extensions::*, PathExt}, Timestamp};
use std::{ops::Deref, path::{Path, PathBuf}};
use anyhow::{Result, anyhow};
use crate::{extract_segment, operations::keyframes::KeyframeMatch};

pub fn split_at(source: impl Deref<Target=Path>, dest_format: impl Deref<Target=Path>, time: Timestamp) -> Result<()> {
    extract_segment(source, dest_format.with_suffix("cut1"), (0, time))?;

    // TODO get length
    // extract_segment(source, dest_format.with_suffix("cut2"), (time, 2))?;
    todo!()
}

pub fn split_every(source: impl Deref<Target=Path>, dest_format: impl Deref<Target=Path>, interval: Timestamp) -> Result<()> {
    todo!()
}
