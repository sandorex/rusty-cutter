mod cli;
mod util;
mod video;

use std::{path::PathBuf, process::ExitCode};
use clap::Parser;

/// Default return type for most functions
pub type ExitResult = Result<(), u8>;

fn main() -> ExitCode {
    let cli_args = cli::Cli::parse();

    use cli::CliCommands;
    let result = match cli_args.cmd {
        CliCommands::Cut(x) => cut_video_cmd(cli_args.dry_run, x),
        _ => {
            dbg!(&cli_args);
            todo!()
        },
    };

    // convert u8 to ExitCode
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(x) => ExitCode::from(x),
    }
}

pub fn cut_video_cmd(dry_run: bool, args: cli::CutArgs) -> ExitResult {
    use util::time_from_f;

    // TODO this is temporary until i fgure out how to parse time in multiple formats from args
    let span = (
        time_from_f(args.start_time),
        time_from_f(args.end_time),
    );

    let vfile = video::VideoFile {
        path: PathBuf::from(args.source),
        dry_run,
    };

    // TODO better suffix with time requested maybe?
    let dest = args.output.unwrap_or_else(|| vfile.new_with_suffix("cut"));

    vfile.extract_segment(span, args.align_keyframe, &dest)
}
