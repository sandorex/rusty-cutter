use std::{ops::Deref, path::Path, time::Duration};
use anyhow::{Result, anyhow};
use crate::util::extensions::command_extensions::*;

pub mod extensions;

/// Type representing timestamp inside the video
pub type Timestamp = u128;

// TODO try rewriting it but use frames to get last frame and then read time
/// NOTE the result is not correct! do not trust it
pub fn get_file_length(file: impl Deref<Target=Path>) -> Result<Timestamp> {
    // the output looks like this
    // {
    //     "format": {
    //         "filename": "recording.mkv",
    //         "nb_streams": 2,
    //         "nb_programs": 0,
    //         "format_name": "matroska,webm",
    //         "format_long_name": "Matroska / WebM",
    //         "start_time": "0.000000",
    //         "duration": "1800.000000",
    //         "size": "730801660",
    //         "bit_rate": "3248007",
    //         "probe_score": 100,
    //         "tags": {
    //             "ENCODER": "Lavf60.16.100"
    //         }
    //     }
    // }

    let cmd = Command::new("ffprobe")
        .args([
            "-loglevel", "error",
            "-of", "json",
            "-i",
        ])
        .arg(file.as_os_str())
        .arg("-show_format")
        .output()
        .expect("Error executing ffprobe");

    match cmd.to_exitcode() {
        Ok(()) => {
            let stdout = String::from_utf8_lossy(&cmd.stdout);

            // this looks funny but i dont feel like adding serde as dependency just to parse this

            let data: serde_json::Value = serde_json::from_str(&stdout)
                .expect("Error while parsing json from ffprobe");

            let data = data.as_object()
                .expect("json data is not an object")
                .get("format")
                .expect("json no key 'format'")
                .as_object()
                .expect("json key 'format' is not an object");

            let duration: f64 = data.get("duration")
                .expect("json no key 'duration' in 'format'")
                .as_str()
                .expect("json 'duration' is not a string")
                .parse()
                .expect("json 'duration' is not a valid float");

            // let start_time: f64 = data.get("start_time")
            //     .expect("json no key 'start_time' in 'format'")
            //     .as_str()
            //     .expect("json 'start_time' is not a string")
            //     .parse()
            //     .expect("json 'start_time' is not a valid float");
            //
            // dbg!(start_time, &duration);

            Ok(Duration::from_secs_f64(duration).as_micros())
        },
        Err(x) => Err(anyhow!("ffprobe exited with code {}: \n{}", x, String::from_utf8(cmd.stderr.clone()).unwrap())),
    }
}

