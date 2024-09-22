mod util;
mod operations;

use util::PathExt;
use std::path::{PathBuf, Path};
use anyhow::{Context, Result};

pub const FULL_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "-", env!("VERGEN_GIT_DESCRIBE"));
pub type Timestamp = u64;

#[derive(Debug, PartialEq, Clone)]
pub enum MediaFragment {
    /// Create video from multiple fragments
    Sequence(Vec<Self>),

    /// Take whole video file
    Video(PathBuf),

    /// Extract a specific part of video file
    VideoSegment {
        /// Path to file
        file: PathBuf,

        /// Cut this exact span, `None` is treated as start and end respectively
        span: (Option<Timestamp>, Option<Timestamp>),
    },

    // NOTE do this later
    // /// Create overlay using
    // Overlay {
    //     image_path: PathBuf,
    //     duration: Timestamp,
    // }
}

impl MediaFragment {
    /// Apply the operation and return path of the result
    ///
    /// The output file is suggestion where to output the file, may not be used
    pub fn apply(&self, output_file: &Path) -> Result<PathBuf> {
        match self {
            // NOTE video requires no work
            Self::Video(x) => Ok(x.to_path_buf()),

            // sequence has to apply all other fragments then add them together
            // TODO this could benefit from multithreading
            Self::Sequence(fragments) => {
                let mut count: u32 = 0;
                // let mut output_files: Vec<PathBuf> = vec![];
                let mut sequence_file: String = "".into();

                for fragment in fragments {
                    count += 1;
                    let path = fragment.apply(&output_file.with_suffix(format!("seq{}", count).as_str()))?;

                    sequence_file += format!("file '{}'\n", path.to_string_lossy()).as_str();
                    // output_files.push(fragment.apply(&path));
                }

                // TODO remove the expect
                std::fs::write("sequence.txt", &sequence_file)
                    .with_context(|| "Error while writing to sequence.txt")?;

                // TODO concat with ffmpeg
                // TODO delete temp files

                Ok(output_file.to_path_buf())
            },
            x @ Self::VideoSegment { file, span: (start, end) } => {
                // TODO support option start and end
                operations::extract_segment(file, output_file, (start.unwrap(), end.unwrap()))?;

                Ok(output_file.to_path_buf())
            },
        }
    }
}

