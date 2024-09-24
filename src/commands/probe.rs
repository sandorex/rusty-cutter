use anyhow::Result;
use librcut::get_keyframes;
use crate::{cli, util::get_file_length};

pub fn probe_file(args: cli::ProbeArgs) -> Result<()> {
    // just print the keyframes if requested
    if args.keyframes {
        let keyframes = get_keyframes(&args.input)?;
        for i in keyframes.iter() {
            println!("{}", i);
        }

        return Ok(());
    }

    println!("Probing file {:?}:", args.input);

    let keyframes = get_keyframes(&args.input)?;

    // calculate avg spacing between frames
    let avg_diff = {
        let diff = keyframes.iter()
            .zip(keyframes.iter().skip(1))
            .map(|(x, y)| y - x)
            .collect::<Vec<_>>();

        let sum: u128 = diff.iter().sum();

        sum / u128::try_from(diff.len()).unwrap()
    };

    let length_micros = get_file_length(args.input)?;
    let last_keyframe = *keyframes.iter().next_back().unwrap();

    println!("Total keyframes: {}", keyframes.len());
    println!("Keyframe freq: 1/{}ms", (length_micros / 1_000) / u128::try_from(keyframes.len()).unwrap());
    println!("Keyframe avg spacing: {}ms", avg_diff / 1_000);

    println!("Duration (ffprobe): {}", length_micros);
    println!("Duration (last keyframe): {}", last_keyframe);

    Ok(())
}
