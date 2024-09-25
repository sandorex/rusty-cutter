use crate::{util::{extensions::PathExt, get_file_length}, Timestamp};
use crate::extract_segment;
use std::path::Path;
use anyhow::Result;

/// Split file at time, exports as `dest_format` with added prefix `cutX.`
pub fn split_at(source: &Path, dest_format: &Path, time: Timestamp) -> Result<()> {
    // cut left side
    extract_segment(source, dest_format.with_prefix("split_a."), (None, Some(time)))?;

    // cut right side
    extract_segment(source, dest_format.with_prefix("split_b."), (Some(time), None))
}

/// Split file every X interval, exports as `dest_format` with added prefix `cutX.`
pub fn split_every(source: &Path, dest_format: &Path, interval: Timestamp) -> Result<()> {
    let file_length = get_file_length(source)?;
    let file_count = file_length / interval;

    // cut first (mostly so i can utilize the automatic start/end finding)
    extract_segment(
        source,
        dest_format.with_prefix("split0."),
        (None, Some(interval)),
    )?;

    // cut the middle ones
    for i in 1..file_count {
        extract_segment(
            source,
            dest_format.with_prefix(format!("split{}.", i).as_str()),
            (Some(i * interval), Some((i + 1) * interval)),
        )?;
    }

    // cut the end, same reason as the first cut
    extract_segment(
        source,
        dest_format.with_prefix(format!("split{}.", file_count).as_str()),
        (Some(interval * file_count), None),
    )?;

    Ok(())
}
