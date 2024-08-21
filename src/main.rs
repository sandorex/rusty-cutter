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
    let result: ExitResult = match cli_args.cmd {
        CliCommands::Extract(x) => extract_video_cmd(cli_args.dry_run, x),
        _ => {
            dbg!(&cli_args);
            Ok(())
        },
    };

    // convert u8 to ExitCode
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(x) => ExitCode::from(x),
    }
}

fn extract_video_cmd(dry_run: bool, args: cli::ExtractArgs) -> ExitResult {
    let vfile = video::VideoFile {
        path: PathBuf::from(args.source),
        dry_run,
    };

    // TODO add cut0 .. cut99 so you can just cut without naming them yourself
    let dest = args.output.unwrap_or_else(|| vfile.new_with_suffix("cut"));

    vfile.extract_segment((args.start_time, args.end_time), args.align_keyframe, &dest)
}
