use super::{Span, VideoFile};
use crate::util::{self, command_extensions::*};

impl VideoFile {
    pub fn extract_segment(&self, region: Span, force_align_keyframes: bool, dest: &str) -> crate::ExitResult {
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
            cut_video_at_keyframe(&self.path.to_string_lossy(), dest, keyframes, self.dry_run)
        } else {
            // create temp file at same place as dest but with different name
            let temp_file = self.new_with_suffix("temp");

            println!("Cutting video between keyframes (transcoding is required)");

            // cut the bigger part of the video to temp file
            cut_video_at_keyframe(&self.path.to_string_lossy(), &temp_file.to_string_lossy(), keyframes, self.dry_run)?;

            println!("Cutting the resulting video to exact size");

            // make sure the temp file is deleted later
            let _ = util::TempFile(&temp_file.to_string_lossy());

            // cut and transcode the actual video
            cut_video_between_keyframe(&temp_file.to_string_lossy(), &dest, region, self.dry_run)
        }
    }
}

/// This function is supposed to be used only on keyframes, non keyframe span produces
/// unpredictable results
fn cut_video_at_keyframe(source: &str, dest: &str, span: (u64, u64), dry_run: bool) -> Result<(), u8> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        // print only errors
        "-loglevel", "error", "-y",
        "-i", source,
        // do not re-encode audio
        "-acodec", "copy",
    ]);

    // simple copy on keyframes
    cmd.args(["-vcodec", "copy"]);
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

fn cut_video_between_keyframe(source: &str, dest: &str, span: (u64, u64), dry_run: bool) -> Result<(), u8> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        // print only errors
        "-loglevel", "error", "-y",
        "-i", source,
        // do not re-encode audio
        "-acodec", "copy",
    ]);
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
