use crate::{util::{extensions::PathExt, get_file_length}, Timestamp};
use crate::extract_segment;
use std::{cmp::min, path::Path};
use anyhow::Result;

/// Split file at time, exports as `dest_format` with added prefix `cutX.`
pub fn split_at(source: &Path, dest_format: &Path, time: Timestamp) -> Result<()> {
    // cut left side
    extract_segment(source.as_ref(), dest_format.with_suffix("cut1"), (0, time))?;

    // cut right side
    extract_segment(source, dest_format.with_suffix("cut2"), (time, get_file_length(source.as_ref())?))
}

/// Split file every X interval, exports as `dest_format` with added prefix `cutX.`
pub fn split_every(source: &Path, dest_format: &Path, interval: Timestamp) -> Result<()> {
    let file_length = get_file_length(source.as_ref())?;
    let file_count = file_length.div_ceil(interval);

    dbg!(file_length, file_count);

    for i in 0..file_count {
        let path = dest_format.with_prefix(format!("cut{}", i).as_str());
        extract_segment(source, path, (i * interval, min((i + 1) * interval, file_length)))?;
    }

    Ok(())
}
