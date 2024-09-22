use std::path::Path;
use std::time::Duration;
use anyhow::{Result, anyhow};
use crate::Timestamp;
use crate::util::command_extensions::*;

/// Get keyframes from the file
pub fn get_keyframes(path: &Path, region: (Timestamp, Timestamp), offset: u64) -> Result<Vec<u64>> {
    let cmd = {
        Command::new("ffprobe")
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
            "-of", "json",
        ])
            .args([
            // limit the reading to requested region
            "-read_intervals".into(), format!(
                "{}us%{}us",
                // i am adding offset here as keyframes are never at exactly the spot you need
                region.0.saturating_sub(offset),
                region.1.saturating_add(offset)),
        ])
            .arg(path)
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

            // save into cache and return reference
            Ok(times)
        }
        Err(x) => Err(anyhow!("ffprobe exited with code {}: \n{}", x, String::from_utf8(cmd.stderr.clone()).unwrap())),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum KeyframeMatch {
    /// Keyframe matched exactly
    Exact(Timestamp),

    /// Keyframe is between two keyframes
    Between {
        before: Timestamp,
        after: Timestamp
    },
}

/// Finds keyframes that are equal to or larger than region
pub fn find_closest_keyframes(keyframes: &Vec<u64>, region: (Timestamp, Timestamp)) -> Result<(KeyframeMatch, KeyframeMatch)> {
    // TODO rewrite this bit more readable
    // NOTE: the iterators are reveresed as position returns first element that the filter lambda
    // returns true so i had to reverse so it found the closest one
    let start_pos = keyframes.iter().rev().position(|&x| region.0 >= x)
        .ok_or_else(|| anyhow!("Could not find start keyframe {}us", region.0))?;

    let start = {
        let start = *keyframes.iter().nth_back(start_pos).unwrap();
        if start == region.0 {
            KeyframeMatch::Exact(start)
        } else {
            KeyframeMatch::Between {
                before: start,
                after: *keyframes.iter().nth_back(start_pos - 1).unwrap(),
            }
        }
    };

    let end_pos = keyframes.iter().position(|&x| region.1 <= x)
        .ok_or_else(|| anyhow!("Could not find end keyframe {}us", region.1))?;

    let end = {
        let end = *keyframes.iter().nth(end_pos).unwrap();
        if end == region.1 {
            KeyframeMatch::Exact(end)
        } else {
            KeyframeMatch::Between {
                before: end,
                after: *keyframes.iter().nth(end_pos + 1).unwrap(),
            }
        }
    };

    Ok((start, end))
}

// /// Finds keyframes that are equal to or smaller than region
// pub fn find_inner_keyframes(keyframes: &Vec<u64>, region: (Timestamp, Timestamp)) -> Result<(Timestamp, Timestamp)> {
//     // find keyframe that is closes to the start time but not after it
//     let start_keyframe: Option<u64> = keyframes.iter()
//         .filter(|x| region.0 <= **x)
//         .cloned()
//         .last();
//
//     // find keyframe that is closes to the end time but not before it
//     let end_keyframe: Option<u64> = keyframes.iter()
//         .filter(|x| region.1 >= **x)
//         .cloned()
//         .nth(0);
//
//     match (start_keyframe, end_keyframe) {
//         (Some(start), Some(end)) => Ok((start, end)),
//         _ => return Err(anyhow!("Could not find keyframes for region {}us - {}us\n", region.0, region.1)),
//     }
// }

#[cfg(test)]
mod tests {
    use super::find_closest_keyframes;

    /// Test if keyframes are properly searched
    #[test]
    fn find_outer_keyframes_test() {
        let keyframes = vec![0, 2_000_000, 4_000_000, 6_000_000, 8_000_000];

        assert_eq!(
            find_closest_keyframes(&keyframes, (1_500_000, 2_500_000)).unwrap(),
            (0, 4_000_000)
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (2_500_000, 2_500_000)).unwrap(),
            (2_000_000, 4_000_000)
        );
    }
}
