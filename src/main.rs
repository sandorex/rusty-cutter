mod util;
mod cli;
mod commands;

use clap::Parser;
use anyhow::{Context, Result};
use cli::{Cli, CliCommands};
use commands::probe_file;
use librcut::{concat_files, extract_segment, split_at, split_every};
use util::extensions::PathExt;

// TODO create next available index for cuts so they are in order cut0 cut1 cut2... cut999
// TODO check if ffprobe and ffmpeg are available in PATH
fn main() -> Result<()> {
    let cli_args = Cli::parse();

    if let CliCommands::Chain { args } = &cli_args.cmd {
        let argv0 = std::env::args().next().unwrap();

        // split commands by ';' to allow chaining
        //
        // convert to references and append argv0 for clap
        let commands = args.split(|x| x == ";")
            .collect::<Vec<_>>()
            .iter()
            .map(|x| {
                // this mess is cause argv0 needs to be set properly in clap
                let mut v = x.iter().map(AsRef::<str>::as_ref).collect::<Vec<_>>();
                v.insert(0, &argv0);
                v
            })
            .collect::<Vec<_>>();

        // run each command in order
        for command in commands {
            let cli_args = Cli::try_parse_from(&command)
                .with_context(|| format!("while parsing chain args {:?}", command.join(" ")))?;

            handle_command(cli_args)?;

            // split command output
            println!();
        }

        // everything went well
        Ok(())
    } else {
        // handle regular single command
        handle_command(cli_args)
    }
}

fn handle_command(cmd: Cli) -> Result<()> {
    match cmd.cmd {
        // handled before this function is called
        CliCommands::Chain { .. } => unreachable!(),
        CliCommands::Cut(cli::CutArgs { input, start_time, end_time, output }) => {
            extract_segment(
                input.as_ref(),
                output.unwrap_or_else(|| input.with_prefix("cut.")),
                (Some(start_time.as_micros()), Some(end_time.as_micros()))
            )
        },
        CliCommands::Split(cli::SplitArgs { interval, time, source, output }) => {
            let path = output.as_ref().unwrap_or(&source);
            if interval {
                split_every(&source, path, time.as_micros())
            } else {
                split_at(&source, path, time.as_micros())
            }
        },
        CliCommands::Sequence(cli::SequenceArgs { output, input }) => concat_files(&input[..], output),
        CliCommands::Probe(x) => probe_file(x),
    }
}

