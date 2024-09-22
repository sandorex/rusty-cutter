use crate::{operations::keyframes::{find_closest_keyframes, get_keyframes}, util::command_extensions::*, PathExt, Timestamp};
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};

use super::{concat_files, keyframes::KeyframeMatch};

const COMMON_FFMPEG_ARGS: &[&str] = &[
    // print only errors
    "-loglevel", "error",

    // do not ask to overwrite
    "-y",

    // do not re-encode audio
    "-acodec", "copy",
];

// TODO make region Option, Option, so it can trim end or beginning or a segment
pub fn extract_segment(path: &Path, dest: &Path, region: (Timestamp, Timestamp)) -> Result<()> {
    let keyframes = get_keyframes(path, region, 5_000_000)?;

    // TODO some files have high compression and keyframes are very far apart, warn the user
    match find_closest_keyframes(&keyframes, region)? {
        (KeyframeMatch::Exact(start), KeyframeMatch::Exact(end)) => {
            // no transcoding needed
            segment_aligned(path, dest, (start, end))
        },
        (start_m, end_m) => {
            let mut files: Vec<PathBuf> = vec![];

            // cut head if needed
            let start = match start_m {
                KeyframeMatch::Between(before, after) => {
                    // cut at keyframes
                    let temp_dest = dest.with_prefix("head_extra.");
                    segment_aligned(path, &temp_dest, (before, after))?;

                    files.push(dest.with_prefix("head."));
                    let new_dest = files.last().unwrap();

                    // trim the cut to correct size, NOTE the time is starting from zero
                    segment_not_aligned(&temp_dest, new_dest, (region.0 - before, region.0 - after))?;

                    // TODO delete temp file

                    after
                },
                KeyframeMatch::Exact(x) => x,
            };

            // cut tail if needed
            let end = match end_m {
                KeyframeMatch::Between(before, after) => {
                    // cut at keyframes
                    let temp_dest = dest.with_prefix("tail_extra.");
                    segment_aligned(path, &temp_dest, (before, after))?;

                    files.push(dest.with_prefix("tail."));
                    let new_dest = files.last().unwrap();

                    // trim the cut to correct size, NOTE the time is starting from zero
                    segment_not_aligned(&temp_dest, new_dest, (0, before - region.1))?;

                    // TODO delete temp file

                    before
                },
                KeyframeMatch::Exact(x) => x,
            };

            // there should be either head or tail as exact keyframes are handled above
            assert!(!files.is_empty());

            files.push(dest.with_prefix("mid."));
            let temp_dest = files.last().unwrap();
            segment_aligned(path, temp_dest, (start, end))?;

            concat_files(&files, dest)

            // TODO delete files
        },
    }
}

/// Extract segment that is aligned on keyframes
fn segment_aligned(source: &Path, dest: &Path, span: (Timestamp, Timestamp)) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-i", &source.to_string_lossy(),
        "-vcodec", "copy",
    ]);
    cmd.args(COMMON_FFMPEG_ARGS);

    // simple copy on keyframes
    cmd.args([
        "-ss".into(), format!("{}us", span.0),
        "-to".into(), format!("{}us", span.1),
    ]);
    cmd.arg(dest);
    cmd.status()
        .expect("Error executing ffmpeg")
        .to_exitcode()
        .map_err(|x| anyhow!("ffmpeg command failed with code: {}", x))?;

    Ok(())
}

/// Extract segment that is not aligned at keyframes (transcoding is required)
fn segment_not_aligned(source: &Path, dest: &Path, span: (Timestamp, Timestamp)) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-i", &source.to_string_lossy()]);
    cmd.args(COMMON_FFMPEG_ARGS);
    cmd.args([
        "-ss".into(), format!("{}us", span.0),
        "-to".into(), format!("{}us", span.1),
    ]);
    cmd.arg(dest);
    cmd.status()
        .expect("Error executing ffmpeg")
        .to_exitcode()
        .map_err(|x| anyhow!("ffmpeg command failed with code: {}", x))?;

    Ok(())
}
