use std::path::Path;
use std::time::Duration;
use anyhow::{Result, anyhow};
use crate::Timestamp;
use crate::util::extensions::command_extensions::*;

// TODO get the keyframes only around the keyframe, the ones in between are useless
/// Get keyframes from the file
pub fn get_keyframes(path: &Path, region: (Timestamp, Timestamp), offset: u128) -> Result<Vec<Timestamp>> {
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
                    region.1.saturating_add(offset)
                ),
            ])
            .arg(path)
            .output()
            .expect("Error executing ffprobe")
    };

    match cmd.to_exitcode() {
        Ok(_) => {
            // parse the time
            let stdout = String::from_utf8_lossy(&cmd.stdout);
            let mut times: Vec<u128> = vec![];

            let data: serde_json::Value = serde_json::from_str(&stdout)
                .expect("Error while parsing json from ffprobe");

            for frame in data["frames"].as_array().expect("Json data is not an array") {
                let pts_time = frame["pts_time"]
                    .as_str()
                    .expect("pts_time is not a string");

                // the panic here should not happen unless something truly breaks
                let time_float: f64 = pts_time.parse::<f64>()
                    .expect("Error parsing pts_time from json");

                times.push(Duration::from_secs_f64(time_float).as_micros());
            }

            // the times may not be in correct order sometimes
            times.sort();

            // save into cache and return reference
            Ok(times)
        }
        Err(x) => Err(anyhow!("ffprobe exited with code {}: \n{}", x, String::from_utf8(cmd.stderr.clone()).unwrap())),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum KeyframeMatch {
    /// Timemap matches keyframe exactly
    Exact(Timestamp),

    /// Timemap is between two timestamps, first one being the one first one above
    Between(Timestamp, Timestamp),
}

/// Finds keyframes that are equal to or larger than region
pub fn find_closest_keyframes(keyframes: &Vec<Timestamp>, region: (Timestamp, Timestamp)) -> Result<(KeyframeMatch, KeyframeMatch)> {
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
            KeyframeMatch::Between(
                start,
                *keyframes.iter().nth_back(start_pos - 1).unwrap(),
            )
        }
    };

    let end_pos = keyframes.iter().position(|&x| region.1 <= x)
        .ok_or_else(|| anyhow!("Could not find end keyframe {}us", region.1))?;

    let end = {
        let end = *keyframes.get(end_pos).unwrap();
        if end == region.1 {
            KeyframeMatch::Exact(end)
        } else {
            KeyframeMatch::Between(
                // NOTE these are swapped as im finding the position larger than region
                *keyframes.get(end_pos - 1).unwrap(),
                end
            )
        }
    };

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use crate::operations::keyframes::KeyframeMatch;

    use super::find_closest_keyframes;

    /// Test if keyframes are properly searched
    #[test]
    fn find_outer_keyframes_test() {
        let keyframes = vec![0, 2_000_000, 4_000_000, 6_000_000, 8_000_000];

        assert_eq!(
            find_closest_keyframes(&keyframes, (1_500_000, 2_500_000)).unwrap(),
            (
                KeyframeMatch::Between(0, 2_000_000),
                KeyframeMatch::Between(2_000_000, 4_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (2_500_000, 2_500_000)).unwrap(),
            (
                KeyframeMatch::Between(2_000_000, 4_000_000),
                KeyframeMatch::Between(2_000_000, 4_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (2_000_000, 2_500_000)).unwrap(),
            (
                KeyframeMatch::Exact(2_000_000),
                KeyframeMatch::Between(2_000_000, 4_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (1_500_000, 2_000_000)).unwrap(),
            (
                KeyframeMatch::Between(0, 2_000_000),
                KeyframeMatch::Exact(2_000_000),
            )
        );
    }
}
