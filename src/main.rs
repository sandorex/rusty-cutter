mod util;
mod cli;
mod commands;

use clap::Parser;
use anyhow::{Context, Result};
use cli::{Cli, CliCommands};
use commands::probe_file;

// TODO add command to show how frequent are the keyframes in a file for troublesome high
// compression cases....
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
        }

        // everything went well quit early
        return Ok(());
    } else {
        // handle regular commands
        handle_command(cli_args)
    }
}

// TODO create function for generating name with prefix and index cut1 cut2..etc

fn handle_command(cmd: Cli) -> Result<()> {
    match cmd.cmd {
        // handled before this function is called
        CliCommands::Chain { .. } => unreachable!(),
        // CliCommands::Split(cli::SplitArgs { source, group, output }) => {
        //     // TODO should split be in librcut?
        //     if let Some(time) = group.time {
        //         librcut::MediaFragment::VideoSegment {
        //             file: source.clone(),
        //             span: (Some(0), Some(time.as_micros().try_into().unwrap())), // TODO timestamp should be
        //                                                              // u128
        //         }.apply(output.unwrap_or_else(|| &source.with_prefix("cut0.")))?;
        //
        //         Ok(())
        //     } else if let Some(interval) = group.interval {
        //         // TODO get length of the file?
        //         todo!()
        //     } else {
        //         unreachable!();
        //     }
        // },
        CliCommands::Probe(x) => probe_file(x),

        _ => todo!(),
    }
}

