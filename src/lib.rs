mod util;
mod operations;

use util::PathExt;
use std::path::{PathBuf, Path};

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
    pub fn apply(&self, output_file: &Path) -> PathBuf {
        match self {
            // NOTE video requires no work
            Self::Video(x) => x.to_path_buf(),

            // sequence has to apply all other fragments then add them together
            // TODO this could benefit from multithreading
            Self::Sequence(fragments) => {
                let mut count: u32 = 0;
                // let mut output_files: Vec<PathBuf> = vec![];
                let mut sequence_file: String = "".into();

                for fragment in fragments {
                    count += 1;
                    let path = fragment.apply(
                        &output_file.with_suffix(format!("seq{}", count).as_str())
                    );

                    sequence_file += format!("file '{}'\n", path.to_string_lossy()).as_str();
                    // output_files.push(fragment.apply(&path));
                }

                // TODO remove the expect
                std::fs::write("sequence.txt", &sequence_file).expect("Failed to write to file");

                // TODO concat with ffmpeg
                // TODO delete temp files

                output_file.to_path_buf()
            },
            x @ Self::VideoSegment { file, span: (start, end) } => {
                // TODO figure out how to cut segment till the end without reading the file length,
                // there is probably an option in ffmpeg
                //
                // TODO cut segment without care for the keyframe
                // operations::extract_segment(file, (start.unwrap(), end.unwrap()), false, output_file);
                println!("Segment {:#?}", x);
                output_file.to_path_buf()
            },
        }
    }
}

