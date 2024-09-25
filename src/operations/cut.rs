use crate::{operations::keyframes::{find_closest_keyframes, get_keyframes, KeyframeMatch}, util::get_file_length, Timestamp};
use std::{ops::Deref, path::{Path, PathBuf}};
use anyhow::{Result, anyhow};
use crate::concat_files;
use crate::util::extensions::{PathExt, command_extensions::*};

const COMMON_FFMPEG_ARGS: &[&str] = &[
    // print only errors
    "-loglevel", "error",

    // do not ask to overwrite
    "-y",

    // do not re-encode audio
    "-acodec", "copy",
];

// TODO rework this function so it can use duration cause timestamps get weird after cutting
// without re-encoding
pub fn extract_segment(source: impl Deref<Target=Path>, dest: impl Deref<Target=Path>, span: (Option<Timestamp>, Option<Timestamp>)) -> Result<()> {
    let keyframes = get_keyframes(&source)?;

    match find_closest_keyframes(&keyframes, span)? {
        (KeyframeMatch::Exact(start) | KeyframeMatch::Boundary(start), KeyframeMatch::Exact(end)) => {
            // no transcoding needed
            segment_aligned(&source, &dest, (start, end))
        },
        (start_m, end_m) => {
            let mut files: Vec<PathBuf> = vec![];
            let length = get_file_length(source.as_ref())?;

            // cut head if needed
            let start = match start_m {
                KeyframeMatch::Between(before, target, after) => {
                    // cut at keyframes
                    let temp_dest = dest.with_prefix("head_extra.");
                    segment_aligned(&source, &temp_dest, (before, after))?;

                    files.push(dest.with_prefix("head."));
                    let new_dest = files.last().unwrap();

                    // trim the cut to correct size, NOTE the time is starting from zero
                    segment_not_aligned(&temp_dest, new_dest, (target - before, after - before))?;

                    // TODO delete temp file

                    after
                },
                // NOTE no extra processing required if start is at boundary as there is usually a
                // keyframe at 0
                KeyframeMatch::Exact(x) | KeyframeMatch::Boundary(x) => x,
            };

            // dbg!(source.as_os_str(), dest.as_os_str(), &end_m, span.1, &end_m, length);

            // cut tail if needed
            let end = match end_m {
                KeyframeMatch::Between(before, target, after) => {
                    // cut at keyframes
                    let temp_dest = dest.with_prefix("tail_extra.");
                    segment_aligned(&source, &temp_dest, (before, after))?;

                    let new_dest = {
                        files.push(dest.with_prefix("tail."));
                        files.last().unwrap()
                    };

                    // trim the cut to correct size, NOTE the time is starting from zero
                    segment_not_aligned(&temp_dest, new_dest, (0, after - target))?;

                    // TODO delete temp file

                    before
                },
                KeyframeMatch::Exact(x) => x,
                // common case where end is not a keyframe, just cut till keyframe and add the
                // extra when concating
                // TODO this causes a lot of work tho
                KeyframeMatch::Boundary(x) => {
                    println!("end is not at the end..");
                    println!("cutting {}..{}", x, length);

                    let new_dest = {
                        files.push(dest.with_prefix("tail."));
                        files.last().unwrap()
                    };

                    segment_not_aligned(&source, new_dest, (x, length))?;

                    x
                },
            };

            // there should be either head or tail as exact keyframes are handled above
            assert!(!files.is_empty());

            files.push(dest.with_prefix("mid."));
            let temp_dest = files.last().unwrap();
            segment_aligned(&source, temp_dest, (start, end))?;

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
