use std::{process::{Command, ExitCode}, time::Duration};
use crate::util::CommandOutputExt;

/// Get all keyframes in the file, only works on video files!
///
/// Region can speed things up by limiting the search to that region only
fn get_keyframes(file: &str, region: Option<(u64, u64)>) -> Result<Vec<u64>, (String, ExitCode)> {
    let mut args: Vec<String> = vec![];

    if let Some((start, end)) = region {
        let start_float = Duration::from_micros(start).as_secs_f64();
        let end_float = Duration::from_micros(end).as_secs_f64();

        // NOTE ffprobe does not care if the start is negative or end is after EOF
        args.extend([
            // read only specified region and add 5s to start and end so keyframes must be found
            "-read_intervals".into(), format!("{:0.5}%{:0.5}", start_float - 5.0, end_float + 5.0),
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

fn find_keyframes(keyframes: &Vec<u64>, start_time: u64, end_time: u64) -> Option<(u64, u64)> {
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
        (Some(start), Some(end)) => Some((start, end)),
        _ => None,
    }
}

pub fn cut_video(file: &str, start_time: u64, end_time: u64) -> ExitCode {
    let keyframes = get_keyframes(file, Some((start_time, end_time)))
        .expect(format!("Unable to get keyframes from {}", file).as_str());

    let time_keyframes = find_keyframes(&keyframes, start_time, end_time);

    dbg!(time_keyframes);

    // TODO
    // find first closest frame
    // find last closest frame
    // if any of the frames is not exact on keyframe then do not use copy codec

    // Command::new("ffmpeg")
    //     .args([
    //         "-loglevel", "error",
    //         // do not re-encode audio
    //         "-vcodec", "copy",
    //     ])
    ExitCode::SUCCESS
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
            Some((0, 4_000_000))
        );

        assert_eq!(
            find_keyframes(&keyframes, 2_500_000, 2_500_000),
            Some((2_000_000, 4_000_000))
        );
    }
}
