use std::path::Path;
use std::time::Duration;
use anyhow::{Result, anyhow};
use crate::Timestamp;
use crate::util::command_extensions::*;

/// Get keyframes from the file, if region is supplied then limit it to that region
pub fn get_keyframes(path: &Path, region: Option<(Timestamp, Timestamp)>) -> Result<Vec<u64>> {
    let mut args: Vec<String> = vec![];

    if let Some((start, end)) = region {
        // NOTE ffprobe does not care if the start is negative or end is after EOF
        args.extend([
            // limit the reading to requested region
            "-read_intervals".into(), format!("{}us%{}us", start, end),
        ]);
    }

    let cmd = {
        let mut cmd = Command::new("ffprobe");
        cmd.args([
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
                "-of", "json",
            ]);
        cmd.args(args);

        // if self.dry_run {
        //     let _ = cmd.print_escaped_cmd();
        // }

        cmd
            .output()
            .expect("Error executing ffprobe")
    };

    match cmd.to_exitcode() {
        Ok(_) => {
            // parse the time
            let stdout = String::from_utf8_lossy(&cmd.stdout);
            let mut times: Vec<u64> = vec![];

            let data: serde_json::Value = serde_json::from_str(&stdout)
                .expect("Error while parsing json from ffprobe");

            for frame in data["frames"].as_array().expect("Json data is not an array") {
                let pts_time = frame["pts_time"]
                    .as_str()
                    .expect("pts_time is not a string");

                // the panic here should not happen unless something truly breaks
                let time_float: f64 = pts_time.parse::<f64>().expect("Error parsing pts_time from json");

                // im pretty sure u64 can store all the keyframes i can find in a real video..
                let micros: u64 = Duration::from_secs_f64(time_float).as_micros().try_into().unwrap();

                times.push(micros);
            }

            // the times may not be in correct order sometimes
            times.sort();

            Ok(times)
        }
        Err(x) => Err(anyhow!("Error while running ffprobe on {:?} (exit code {}): \n{}", path, x, String::from_utf8(cmd.stderr.clone()).unwrap())),
    }
}

fn find_keyframes(keyframes: &Vec<u64>, region: (Timestamp, Timestamp)) -> Result<(Timestamp, Timestamp)> {
    // find keyframe that is closes to the start time but not after it
    let start_keyframe: Option<u64> = keyframes.iter()
        .filter(|x| region.0 >= **x)
        .cloned()
        .last();

    // find keyframe that is closes to the end time but not before it
    let end_keyframe: Option<u64> = keyframes.iter()
        .filter(|x| region.1 <= **x)
        .cloned()
        .nth(0);

    match (start_keyframe, end_keyframe) {
        (Some(start), Some(end)) => Ok((start, end)),
        (start, end) => {
            if start.is_none() {
                return Err(anyhow!("Could not find keyframe for start time {}us\n", region.0));
            }

            if end.is_none() {
                return Err(anyhow!("Could not find keyframe for end time {}us\n", region.1));
            }

            // this should not be reached
            unreachable!()
        }
    }
}

/// Find closest keyframes to the region, output will always be equal or larger than region
pub fn find_closest_keyframes(path: &Path, region: (Timestamp, Timestamp)) -> Result<(Timestamp, Timestamp)> {
    // add 5 seconds before and after region to make sure any keyframes are cought
    let keyframes = get_keyframes(
        path,
        Some((
            region.0.saturating_sub(5_000_000),
            region.1.saturating_add(5_000_000)
        ))
    )?;

    find_keyframes(&keyframes, region)
}

