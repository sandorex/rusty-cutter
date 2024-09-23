mod cli;

use clap::Parser;
use anyhow::{Context, Result};
use cli::{Cli, CliCommands};

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

fn handle_command(cmd: Cli) -> Result<()> {
    match cmd.cmd {
        // handled before this function is called
        CliCommands::Chain { .. } => unreachable!(),

        _ => dbg!(&cmd),
    };

    Ok(())
}

