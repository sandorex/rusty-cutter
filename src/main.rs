// mod cli;
// mod util;
// mod video;

use std::path::PathBuf;
// use clap::Parser;

/// Default return type for most functions
// pub type ExitResult = Result<(), u8>;

fn main() -> anyhow::Result<()> {
    use librcut;
    use librcut::MediaFragment;

    let x = MediaFragment::Sequence(vec![
        // MediaFragment::Video(PathBuf::from("/home/user/video.mkv")),
        MediaFragment::VideoSegment { file: PathBuf::from("recording.mkv"), span: (Some(1_000_000u64), Some(4_000_000u64)) },
        MediaFragment::VideoSegment { file: PathBuf::from("recording.mkv"), span: (Some(6_000_000u64), Some(12_000_000u64)) },
    ]);

    println!("got: {:#?}", x.apply(&PathBuf::from("out.mkv"))?);

    Ok(())
    // let cli_args = cli::Cli::parse();

    // TODO check if ffprobe and ffmpeg are available in PATH

    // use cli::CliCommands;
    // let result: ExitResult = match cli_args.cmd {
    //     CliCommands::Extract(x) => extract_video_cmd(cli_args.dry_run, x),
    //     CliCommands::Probe(x) => probe_cmd(x),
    //     _ => {
    //         println!("Not implemented");
    //         dbg!(&cli_args);
    //         Ok(())
    //     },
    // };

    // // convert u8 to ExitCode
    // match result {
    //     Ok(_) => ExitCode::SUCCESS,
    //     Err(x) => ExitCode::from(x),
    // }
}

// fn extract_video_cmd(dry_run: bool, args: cli::ExtractArgs) -> ExitResult {
//     let vfile = video::VideoFile {
//         path: PathBuf::from(args.source),
//         dry_run,
//     };
//
//     // TODO add cut0 .. cut99 so you can just cut without naming them yourself
//     let dest = args.output.unwrap_or_else(|| vfile.new_with_suffix("cut"));
//
//     vfile.extract_segment((args.start_time, args.end_time), args.align_keyframe, &dest)
// }
//
// fn probe_cmd(args: cli::ProbeArgs) -> ExitResult {
//     let vfile = video::VideoFile {
//         path: PathBuf::from(args.file),
//         dry_run: false,
//     };
//
//     let keyframes = vfile.get_keyframes(None).unwrap();
//     println!("Total keyframes: {}", keyframes.len());
//
//     Ok(())
// }
