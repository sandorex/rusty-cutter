mod util;

use std::process::{Command, ExitCode};
use util::CommandOutputExt;

/// Get all keyframes in the file, only works on video files!
///
/// Region can speed things up by limiting the search to that region only (DANGER if too narrow may
/// exclude the keyframes and screw up things!)
fn get_keyframes(file: &str, region: Option<(f64, f64)>) -> Result<Vec<f64>, (String, ExitCode)> {
    let mut args: Vec<String> = vec![];

    if let Some((start, end)) = region {
        // NOTE ffprobe does not care if the start is negative or end is after EOF
        args.extend([
            // read only specified region and add 5s to start and end so keyframes must be found
            "-read_intervals".into(), format!("{:0.5}%{:0.5}", start - 5.0, end + 5.0),
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
        let mut times: Vec<f64> = vec![];

        for line in stdout.lines() {
            // im panicking here as it truly should not happen unless something breaks
            times.push(line.parse::<f64>().expect("Error parsing pts_time from ffprobe"));
        }

        Ok(times)
    } else {
        Err((String::from_utf8(cmd.stderr.clone()).unwrap(), cmd.to_exitcode()))
    }
}

pub fn cut_video(file: &str, start_time: f64, end_time: f64) -> ExitCode {
    let keyframes = get_keyframes(file, Some((start_time, end_time)))
        .expect(format!("Unable to get keyframes from {}", file).as_str());

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

fn main() -> ExitCode {
    println!("Hello, world!");
    let times = get_keyframes("video.mkv", Some((2.0, 10.0)));
    dbg!(times);

    ExitCode::FAILURE
}

