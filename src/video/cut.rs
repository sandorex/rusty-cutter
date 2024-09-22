use super::{Span, VideoFile};
use crate::util::{self, command_extensions::*};

const COMMON_FFMPEG_ARGS: &[&str] = &[
    // print only errors
    "-loglevel", "error",

    // do not ask to overwrite
    "-y",

    // do not re-encode audio
    "-acodec", "copy",
];

impl VideoFile {
    pub fn extract_segment(&self, region: Span, force_align_keyframes: bool, dest: &str) -> crate::ExitResult {
        // TODO some files have high compression and there are not many keyframes, find a way to
        // detect that so the user is warned
        let keyframes = match self.find_closest_keyframes(region) {
            Ok(x) => x,
            Err((err, code)) => {
                eprintln!("{}", err);
                return Err(code);
            }
        };

        assert!(keyframes.0 <= keyframes.1, "Start keyframe is after the end keyframe");

        // no transcoding is needed if keyframes align
        let needs_transcoding = keyframes.0 != region.0 || keyframes.1 != region.1;

        if force_align_keyframes || !needs_transcoding {
            println!("Cutting video at keyframes");
            segment_aligned(&self.path.to_string_lossy(), dest, keyframes, self.dry_run)
        } else {
            /*

                the video
            |-----------|

            The wanted part
              V
            |--|--------|

             keyframes in the file
            |-|-|-|-|-|-|

            cut 1
            |-|

            cut 2 and transcode it to precisely cut
              |-|

            concat together
            |--|

             */

            // create temp file at same place as dest but with different name
            let temp_file = self.new_with_suffix("temp");

            println!("Cutting video between keyframes (transcoding is required)");

            // cut the bigger part of the video to temp file
            segment_aligned(&self.path.to_string_lossy(), &temp_file, keyframes, self.dry_run)?;

            println!("Cutting the resulting video to exact size");

            // make sure the temp file is deleted later
            let _x = util::TempFile(&temp_file);

            // offset is difference between keyframe and actual wanted region
            let offset: u64 = region.0.saturating_sub(keyframes.0);

            // actually requested length of video (cutting off extra from keyframe)
            let length: u64 = region.1 - region.0;

            // cut and transcode the actual video
            segment_not_aligned(
                &temp_file,
                &dest,
                (offset, offset + length),
                self.dry_run
            )
        }
    }
}

/// Extract segment that is aligned on keyframes
fn segment_aligned(source: &str, dest: &str, span: (u64, u64), dry_run: bool) -> Result<(), u8> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-i", source,
        "-vcodec", "copy",
    ]);
    cmd.args(COMMON_FFMPEG_ARGS);

    // simple copy on keyframes
    cmd.args([
        "-ss".into(), format!("{}us", span.0),
        "-to".into(), format!("{}us", span.1),
    ]);
    cmd.arg(dest);

    if dry_run {
        cmd.print_escaped_cmd()
    } else {
        cmd.status()
            .expect("Error executing ffmpeg")
            .to_exitcode()
    }
}

/// Extract segment that is not aligned at keyframes (transcoding is required)
fn segment_not_aligned(source: &str, dest: &str, span: (u64, u64), dry_run: bool) -> Result<(), u8> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-i", source]);
    cmd.args(COMMON_FFMPEG_ARGS);
    cmd.args([
        "-ss".into(), format!("{}us", span.0),
        "-to".into(), format!("{}us", span.1),
    ]);
    cmd.arg(dest);

    if dry_run {
        cmd.print_escaped_cmd()
    } else {
        cmd.status()
            .expect("Error executing ffmpeg")
            .to_exitcode()
    }
}
