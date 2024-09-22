use crate::{operations::keyframes::{find_closest_keyframes, get_keyframes}, util::command_extensions::*, PathExt, Timestamp};
use std::path::Path;
use anyhow::{Result, anyhow};

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
    let keyframes_all = get_keyframes(path, region, 5_000_000)?;

    // TODO some files have high compression and keyframes are very far apart, warn the user
    let keyframes = find_closest_keyframes(&keyframes_all, region)?;

    assert!(keyframes.0 <= keyframes.1, "Start keyframe is after the end keyframe");

    // no transcoding is needed if keyframes align
    let needs_transcoding = keyframes.0 != region.0 || keyframes.1 != region.1;

    if !needs_transcoding {
        segment_aligned(path, dest, keyframes)
    } else {
        // it does not align

        // cut the aligning part
        let temp1 = path.with_prefix("temp1");

        // segment_aligned(path, dest, span)

        // create temp file at same place as dest but with different name
        let temp_file = path.with_suffix("temp");

        // cut the bigger part of the video to temp file
        segment_aligned(path, &temp_file, keyframes)?;

        // offset is difference between keyframe and actual wanted region
        let offset: u64 = region.0.saturating_sub(keyframes.0);

        // actually requested length of video (cutting off extra from keyframe)
        let length: u64 = region.1 - region.0;

        // cut and transcode the actual video
        segment_not_aligned(
            &temp_file,
            dest,
            (offset, offset + length)
        )?;

        // remove the temp file
        std::fs::remove_file(&temp_file)?;

        Ok(())
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
