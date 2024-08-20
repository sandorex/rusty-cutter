use std::{process::Command, time::Duration};
use crate::util::{CommandOutputExt, CommandExt};

/// Get all keyframes in the file, only works on video files!
///
/// Region can speed things up by limiting the search to that region only
fn get_keyframes(file: &str, region: Option<(u64, u64)>) -> Result<Vec<u64>, (String, u8)> {
    let mut args: Vec<String> = vec![];

    if let Some((start, end)) = region {
        // NOTE ffprobe does not care if the start is negative or end is after EOF
        args.extend([
            "-read_intervals".into(), format!("{}us%{}us", start.saturating_sub(5_000_000), end + 5_000_000),
        ]);
    }

    let cmd = Command::new("ffprobe")
        .args([
            "-loglevel", "error",
            // there should always be just one stream
            "-select_streams", "v:0",
            // skip non key frames
            "-skip_frame", "nokey",
            // iterate frames
            "-show_frames",
            // print only frame time
            "-show_entries", "frame=pts_time",
            // use csv to print it one per line without any additional mess
            "-of", "csv=print_section=0",
        ])
        .args(args)
        .arg(file)
        .output()
        .expect("Error executing ffprobe");

    match cmd.to_exitcode() {
        Ok(_) => {
            // parse the time
            let stdout = String::from_utf8_lossy(&cmd.stdout);
            let mut times: Vec<u64> = vec![];

            for line in stdout.lines() {
                // im panicking here as it truly should not happen unless something breaks
                let time_float = line.parse::<f64>().expect("Error parsing pts_time from ffprobe");

                // im pretty sure u64 can store all the keyframes i can find in a real video..
                let micros: u64 = Duration::from_secs_f64(time_float).as_micros().try_into().unwrap();

                times.push(micros);
            }

            // the times may not be in correct order sometimes
            times.sort();

            Ok(times)
        }
        Err(x) => Err((String::from_utf8(cmd.stderr.clone()).unwrap(), x)),
    }
}

fn find_keyframes(keyframes: &Vec<u64>, start_time: u64, end_time: u64) -> Result<(u64, u64), String> {
    // find keyframe that is closes to the start time but not after it
    let start_keyframe: Option<u64> = keyframes.iter()
        .filter(|x| start_time >= **x)
        .cloned()
        .last();

    // find keyframe that is closes to the end time but not before it
    let end_keyframe: Option<u64> = keyframes.iter()
        .filter(|x| end_time <= **x)
        .cloned()
        .nth(0);

    match (start_keyframe, end_keyframe) {
        (Some(start), Some(end)) => Ok((start, end)),
        (start, end) => {
            let mut err_str: String = "".to_string();

            if start.is_none() {
                err_str += format!("Could not find keyframe for start time {}us\n", start_time).as_str();
            }

            if end.is_none() {
                err_str += format!("Could not find keyframe for end time {}us", end_time).as_str();
            }

            Err(err_str)
        }
    }
}

/// This function is supposed to be used only on keyframes, non keyframe span produces
/// unpredictable results
fn cut_video_at_keyframe(source: &str, dest: &str, span: (u64, u64), dry_run: bool) -> Result<(), u8> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        // print only errors
        "-loglevel", "error",
        "-i", source,
        // do not re-encode audio
        "-acodec", "copy",
    ]);

    // simple copy on keyframes
    cmd.args(["-vcodec", "copy"]);
    cmd.args([
        "-ss".into(), format!("{}us", span.0),
        "-to".into(), format!("{}us", span.1),
    ]);
    cmd.arg(dest);

    if dry_run {
        cmd.print_escaped_cmd()
    } else {
        cmd.status()
            .expect("Error executing ffmpeg")
            .to_exitcode()
    }
}

fn cut_video_between_keyframe(source: &str, dest: &str, span: (u64, u64), dry_run: bool) -> Result<(), u8> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        // print only errors
        "-loglevel", "error",
        "-i", source,
        // do not re-encode audio
        "-acodec", "copy",
    ]);
    cmd.args([
        "-ss".into(), format!("{}us", span.0),
        "-to".into(), format!("{}us", span.1),
    ]);
    cmd.arg(dest);

    if dry_run {
        cmd.print_escaped_cmd()
    } else {
        cmd.status()
            .expect("Error executing ffmpeg")
            .to_exitcode()
    }
}

pub fn cut_video(source: &str, dest: &str, span: (u64, u64), dry_run: bool) -> Result<(), u8> {
    // TODO this should not panic but be plain error
    let keyframes = get_keyframes(source, Some(span))
        .expect(format!("Unable to get keyframes from {}", source).as_str());

    assert_ne!(keyframes.len(), 0, "Got zero keyframes");

    let (start_time, end_time) = span;

    let (start_keyframe, end_keyframe) = match find_keyframes(&keyframes, start_time, end_time) {
        Ok(x) => x,
        Err(x) => {
            eprintln!("{}", x);
            return Err(1);
        }
    };

    assert!(start_keyframe <= end_keyframe, "Start keyframe is after the end keyframe");

    // no transcoding can only be done if cutting is done at exactly the keyframe
    let needs_transcoding = start_keyframe != start_time || end_keyframe != end_time;

    // TODO make ffmpeg overwrite without asking!
    if !needs_transcoding {
        println!("Cutting video at keyframes");
        cut_video_at_keyframe(source, dest, (start_time, end_time), dry_run)
    } else {
        // create temp file at same place as dest but with different name
        let temp_file = {
            let path = std::path::Path::new(dest);

            // TODO this is very ugly and too many unwraps
            format!("{}.temp.{}", path.file_stem().unwrap().to_str().unwrap(), path.extension().unwrap().to_str().unwrap())
        };

        println!("Cutting video between keyframes (transcoding is required)");

        // cut the bigger part of the video to temp file
        cut_video_at_keyframe(source, &temp_file, (start_keyframe, end_keyframe), dry_run)?;

        println!("Cutting the resulting video to exact size");

        // cut and transcode the actual video
        cut_video_between_keyframe(&temp_file, dest, (start_time, end_time), dry_run)

        // TODO remove tempfile
    }
}

#[cfg(test)]
mod tests {
    /// Test if keyframes are properly searched
    #[test]
    fn find_keyframes() {
        use super::find_keyframes;

        let keyframes = vec![0, 2_000_000, 4_000_000, 6_000_000, 8_000_000];

        assert_eq!(
            find_keyframes(&keyframes, 1_500_000, 2_500_000),
            Ok((0, 4_000_000))
        );

        assert_eq!(
            find_keyframes(&keyframes, 2_500_000, 2_500_000),
            Ok((2_000_000, 4_000_000))
        );
    }
}
