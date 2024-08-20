mod util;
mod video;
mod cli;

use std::process::ExitCode;
use clap::Parser;

fn main() -> ExitCode {
    let cli_args = cli::Cli::parse();

    use cli::CliCommands;
    let result = match cli_args.cmd {
        CliCommands::Cut(x) => video::cut_video_cmd(cli_args.dry_run, x),
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

