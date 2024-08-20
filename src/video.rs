use std::{process::{Command, ExitCode}, time::Duration};
use crate::util::{CommandOutputExt, CommandExt};

/// Get all keyframes in the file, only works on video files!
///
/// Region can speed things up by limiting the search to that region only
fn get_keyframes(file: &str, region: Option<(u64, u64)>) -> Result<Vec<u64>, (String, ExitCode)> {
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

    if cmd.status.success() {
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
    } else {
        Err((String::from_utf8(cmd.stderr.clone()).unwrap(), cmd.to_exitcode()))
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

pub fn cut_video(source: &str, dest: &str, start_time: u64, end_time: u64, dry_run: bool) -> ExitCode {
    let keyframes = get_keyframes(source, Some((start_time, end_time)))
        .expect(format!("Unable to get keyframes from {}", source).as_str());

    dbg!(&keyframes);

    assert_ne!(keyframes.len(), 0, "Got zero keyframes");

    let (start_keyframe, end_keyframe) = match find_keyframes(&keyframes, start_time, end_time) {
        Ok(x) => x,
        Err(x) => {
            eprintln!("{}", x);
            return ExitCode::FAILURE;
        }
    };

    assert!(start_keyframe <= end_keyframe, "Start keyframe is after the end keyframe");

    // no transcoding can only be done if cutting is done at exactly the keyframe
    let needs_transcoding = start_keyframe != start_time || end_keyframe != end_time;

    dbg!(start_keyframe);
    dbg!(end_keyframe);
    dbg!(needs_transcoding);

    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        // print only errors
        "-loglevel", "error",
        "-i", source,
        // do not re-encode audio
        "-acodec", "copy",
    ]);

    if !needs_transcoding {
        // simple copy on keyframes
        cmd.args(["-vcodec", "copy"]);
        cmd.args([
            "-ss".into(), format!("{}us", start_keyframe),
            "-to".into(), format!("{}us", end_keyframe),
        ]);
        cmd.arg(dest);

        if dry_run {
            cmd.print_escaped_cmd()
        } else {
            cmd.status()
                .expect("Error executing ffmpeg")
                .to_exitcode()
        }
    } else {
        todo!("Non keyframe cutting is not supported atm");
        // TODO if not exactly on keyframe then cut bigger then transcode to exact place
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
