use std::ffi::OsString;
use std::path::Path;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;
use anyhow::{Result, anyhow};
use crate::Timestamp;
use crate::util::extensions::command_extensions::*;
use std::collections::HashMap;

// TODO remove region and offset
/// Get keyframes from the file
pub fn get_keyframes(path: &Path) -> Result<Rc<Vec<Timestamp>>> {
    static mut CACHE: Option<Mutex<HashMap<OsString, Rc<Vec<Timestamp>>>>> = None;

    // initialize the cache
    unsafe {
        // TODO this as_ref is kinda confusing as &CACHE gave me warning about
        // https://github.com/rust-lang/rust/issues/114447
        match CACHE.as_ref() {
            // get value from cache
            Some(cache) => return Ok(Rc::clone(cache.lock().unwrap().get(path.as_os_str()).unwrap())),

            // cache not initialized
            None => CACHE = Some(Mutex::new(HashMap::new())),
        }
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

        cmd.arg(path)
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

            let times = unsafe {
                CACHE.as_mut()
                    .unwrap()
                    .get_mut()
                    .unwrap()
                    .insert(path.as_os_str().to_owned(), Rc::new(times));

                let cache = CACHE
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap();

                // return cloned Rc
                Rc::clone(cache.get(path.as_os_str()).unwrap())
            };

            // save into cache and return reference
            Ok(times)
        }
        Err(x) => Err(anyhow!("ffprobe exited with code {}: \n{}", x, String::from_utf8(cmd.stderr.clone()).unwrap())),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum KeyframeMatch {
    /// Timestamp matches keyframe exactly
    Exact(Timestamp),

    /// Timestamp is between two timestamps ordered as: before, target, after
    Between(Timestamp, Timestamp, Timestamp),
}

pub fn find_closest_keyframes(keyframes: &[Timestamp], span: (Option<Timestamp>, Option<Timestamp>)) -> Result<(KeyframeMatch, KeyframeMatch)> {
    let start = if let Some(time) = span.0 {
        let start_pos = keyframes
            .iter()
            .rev()
            .position(|&x| time >= x)
            .ok_or_else(|| anyhow!("Could not find start keyframe {}us", time))?;

        let start = *keyframes
            .iter()
            .nth_back(start_pos)
            .unwrap();

        if start == time {
            KeyframeMatch::Exact(start)
        } else {
            KeyframeMatch::Between(
                start,
                time,
                *keyframes.iter().nth_back(start_pos - 1).unwrap(),
            )
        }
    } else {
        // just take the first one
        KeyframeMatch::Exact(*keyframes.iter().next().unwrap())
    };

    let end = if let Some(time) = span.1 {
        let end_pos = keyframes
            .iter()
            .position(|&x| time <= x)
            .ok_or_else(|| anyhow!("Could not find end keyframe {}us", time))?;

        let end = *keyframes
            .get(end_pos)
            .unwrap();

        if end == time {
            KeyframeMatch::Exact(end)
        } else {
            KeyframeMatch::Between(
                // NOTE these are swapped as im finding the position larger than region
                *keyframes.get(end_pos - 1).unwrap(),
                time,
                end
            )
        }
    } else {
        // just take the last one
        KeyframeMatch::Exact(*keyframes.iter().next_back().unwrap())
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
            find_closest_keyframes(&keyframes, (Some(1_500_000), Some(2_500_000))).unwrap(),
            (
                KeyframeMatch::Between(0, 1_500_000, 2_000_000),
                KeyframeMatch::Between(2_000_000, 2_500_000, 4_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (Some(2_500_000), Some(2_500_000))).unwrap(),
            (
                KeyframeMatch::Between(2_000_000, 2_500_000, 4_000_000),
                KeyframeMatch::Between(2_000_000, 2_500_000, 4_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (Some(2_000_000), Some(2_500_000))).unwrap(),
            (
                KeyframeMatch::Exact(2_000_000),
                KeyframeMatch::Between(2_000_000, 2_500_000, 4_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (Some(1_500_000), Some(2_000_000))).unwrap(),
            (
                KeyframeMatch::Between(0, 1_500_000, 2_000_000),
                KeyframeMatch::Exact(2_000_000),
            )
        );

        // test if out of bounds
        assert_eq!(
            find_closest_keyframes(&keyframes, (None, None)).unwrap(),
            (
                KeyframeMatch::Exact(0),
                KeyframeMatch::Exact(8_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (Some(1_500_000), None)).unwrap(),
            (
                KeyframeMatch::Between(0, 1_500_000, 2_000_000),
                KeyframeMatch::Exact(8_000_000),
            )
        );

        assert_eq!(
            find_closest_keyframes(&keyframes, (None, Some(1_500_000))).unwrap(),
            (
                KeyframeMatch::Exact(0),
                KeyframeMatch::Between(0, 1_500_000, 2_000_000),
            )
        );
    }
}
