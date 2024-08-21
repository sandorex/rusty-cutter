mod cut;

use std::{path::PathBuf, time::Duration};
use crate::util::command_extensions::*;

pub type Span = (u64, u64);

#[derive(Debug, Default, PartialEq, Eq)]
pub struct VideoFile {
    pub path: PathBuf,
    pub dry_run: bool,
}

impl VideoFile {
    /// Copy path of current file and add suffix to the file
    ///
    /// Used all over the place to create output file or temporary files
    pub fn new_with_suffix(&self, suffix: &str) -> String {
        match self.path.extension() {
            Some(x) => self.path.with_extension(format!("{}.{}", suffix, x.to_string_lossy())),
            None => self.path.with_extension(suffix),
        }.to_string_lossy().to_string()
    }

    /// Get keyframes from the file, if region is supplied then limit it to that region
    pub fn get_keyframes(&self, region: Option<Span>) -> Result<Vec<u64>, (String, u8)> {
        let mut args: Vec<String> = vec![];

        if let Some((start, end)) = region {
            // NOTE ffprobe does not care if the start is negative or end is after EOF
            args.extend([
                // limit the reading to requested region
                "-read_intervals".into(), format!("{}us%{}us", start, end),
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
            .arg(&self.path)
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

    fn find_keyframes(keyframes: &Vec<u64>, region: Span) -> Result<Span, (String, u8)> {
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
                let mut err_str: String = "".to_string();

                if start.is_none() {
                    err_str += format!("Could not find keyframe for start time {}us\n", region.0).as_str();
                }

                if end.is_none() {
                    err_str += format!("Could not find keyframe for end time {}us", region.1).as_str();
                }

                Err((err_str, 1))
            }
        }
    }

    /// Find closest keyframes to the region, output will always be equal or larger than region
    pub fn find_closest_keyframes(&self, region: Span) -> Result<Span, (String, u8)> {
        // add 5 seconds before and after region to make sure any keyframes are cought
        let keyframes = self.get_keyframes(Some((
            region.0.saturating_sub(5_000_000),
            region.1.saturating_add(5_000_000)
        )))?;

        Self::find_keyframes(&keyframes, region)
    }
}

#[cfg(test)]
mod tests {
    use super::VideoFile;

    /// Test if keyframes are properly searched
    #[test]
    fn test_find_keyframes() {
        let keyframes = vec![0, 2_000_000, 4_000_000, 6_000_000, 8_000_000];

        assert_eq!(
            VideoFile::find_keyframes(&keyframes, (1_500_000, 2_500_000)),
            Ok((0, 4_000_000))
        );

        assert_eq!(
            VideoFile::find_keyframes(&keyframes, (2_500_000, 2_500_000)),
            Ok((2_000_000, 4_000_000))
        );
    }

    /// Test if suffix is replaced properly
    #[test]
    fn test_new_with_suffix() {
        use std::path::PathBuf;

        // file without absolute path
        assert_eq!(
            VideoFile { path: PathBuf::from("file.txt"), ..Default::default() }.new_with_suffix("temp"),
            "file.temp.txt".to_string()
        );

        // file with absolute path
        assert_eq!(
            VideoFile { path: PathBuf::from("/etc/file.txt"), ..Default::default() }.new_with_suffix("temp"),
            "/etc/file.temp.txt".to_string()
        );

        // file without extension
        assert_eq!(
            VideoFile { path: PathBuf::from("/etc/file"), ..Default::default() }.new_with_suffix("temp"),
            "/etc/file.temp".to_string()
        );

        // multiple existing extensions
        assert_eq!(
            VideoFile { path: PathBuf::from("/file.txt.txt"), ..Default::default() }.new_with_suffix("temp"),
            "/file.txt.temp.txt".to_string()
        );
    }
}

