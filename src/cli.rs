use std::path::PathBuf;

use clap::{Parser, Subcommand, Args};

/// Wrapper around ffmpeg to do media file editing with minimal transcoding when possible
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_BIN_NAME"), author, version = librcut::FULL_VERSION, about)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: CliCommands,
}

#[derive(Subcommand, Debug)]
pub enum CliCommands {
    // NOTE no command should operate on file in place, always output to a new one

    /// Chain together multiple commands delimited by ';'
    ///
    /// Ex. `chain cut file.mkv 20m 25m out.mkv \; concat file.mkv out.mkv`
    Chain {
        // capture everything as it will be parsed again
        #[arg(required = true, num_args = 1.., use_value_delimiter = true)]
        args: Vec<String>,
    },

    /// Extract a specific part of the video into a new file
    Cut(CutArgs),

    /// Merge several files together in sequence
    Sequence(SequenceArgs),

    /// Split video file at specific point, or interval
    Split(SplitArgs),

    /// Probe file to see information about, useful to check keyframe frequency
    Probe(ProbeArgs),

    // TODO command that overlays video or image
    // TODO command that creates empty black screen video for timing
    // TODO command that converts image into video
}

#[derive(Args, Debug, Clone)]
pub struct CutArgs {
    /// Input file
    pub input: PathBuf,

    /// Start time of the segment (for detailed format see help)
    pub start_time: humantime::Duration,

    /// End time of the segment (for detailed format see help)
    pub end_time: humantime::Duration,

    /// File to output to (if not specified default suffix will be added to source name)
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SequenceArgs {
    /// Output file
    pub output: PathBuf,

    /// Input files to merge together in order of appearance
    #[arg(required=true, num_args=2..)]
    pub input: Vec<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SplitArgs {
    /// File to operate on
    pub source: PathBuf,

    #[clap(flatten)]
    pub group: TimeOrIntervalGroup,

    // TODO this should be a format, cause there will be at least 2 files
    /// File to output to (if not specified default suffix will be added to source name)
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = false)]
pub struct TimeOrIntervalGroup {
    /// Interval to split the file at
    #[arg(short, long)]
    pub interval: Option<humantime::Duration>,

    /// Time to split at
    #[arg(short, long)]
    pub time: Option<humantime::Duration>,
}

#[derive(Args, Debug, Clone)]
pub struct ProbeArgs {
    /// File to probe
    pub input: PathBuf,

    /// Sample size to analyze, the longer it is the longer it will take
    #[arg(short, long, default_value = "120s")]
    pub sample_size: humantime::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}

