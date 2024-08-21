use clap::{Parser, Subcommand, Args};

/// Wrapper around ffmpeg to do media file editing with minimal transcoding when possible
#[derive(Parser, Debug)]
#[command(name = "rcut", author, version, about)]
pub struct Cli {
    /// Just print commands that would've been ran, do not modify filesystem
    #[arg(long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub cmd: CliCommands,
}

#[derive(Subcommand, Debug)]
pub enum CliCommands {
    // NOTE no command should operate on file in place, always output to a new one

    // TODO rename into extract?
    /// Cut a segment of a video file into a new file
    Cut(CutArgs),

    // TODO this should probably be the one that is called cut
    /// Remove a segment of a video and save the rest into a new file
    Trim,

    /// Split video file at specific point, or interval
    Split(SplitArgs),

    /// Add together two or more video files of the same type into one
    Concat(ConcatArgs),

    // /// Create overlay video from image
    // Overlay,
}

#[derive(Args, Debug, Clone, Default)]
pub struct CutArgs {
    /// Force align time to keyframes (allows cutting without transcoding, but cuts wont be exact)
    #[arg(short, long, default_value_t = false)]
    pub align_keyframe: bool,

    /// Source file
    pub source: String,

    /// Start time of the segment in seconds
    pub start_time: f64,

    /// End time of the segment in seconds
    pub end_time: f64,

    /// File to output to (if not specified default suffix will be added to source name)
    pub output: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct SplitArgs {
    /// Force the interval or time to align to keyframes (allows splitting without transcoding, but
    /// splits wont be exact)
    #[arg(short, long, default_value_t = false)]
    pub align_keyframe: bool,

    /// File to operate on
    pub source: String,

    #[clap(flatten)]
    group: TimeOrIntervalGroup,

    /// File to output to (if not specified default suffix will be added to source name)
    pub output: Option<String>,
}

#[derive(Debug, Clone, Default, clap::Args)]
#[group(required = true, multiple = false)]
pub struct TimeOrIntervalGroup {
    /// Interval to split the file
    #[arg(short, long)]
    pub interval: Option<f64>,

    /// Time to split the media file at
    #[arg(short, long)]
    pub time: Option<f64>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ConcatArgs {
    /// File to output to
    pub output: String,

    #[arg(required=true, num_args=2..)]
    pub input: Vec<String>,
}

// #[derive(Args, Debug, Clone, Default)]
// pub struct OverlayArgs {
//     /// Source file
//     pub source: String,
//
//     /// File to overlay
//     pub overlay: String,
//
//     /// Time for the overlay to start
//     pub overlay_start: f64,
//
//     /// Time for the overlay to end
//     pub overlay_end: f64,
//
//     /// File to output to (if not specified default suffix will be added to source name)
//     pub output: Option<String>,
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}

