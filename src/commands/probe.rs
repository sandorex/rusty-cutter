use std::time::Duration;
use anyhow::Result;
use librcut::get_keyframes;

use crate::{cli, util};

pub fn probe_file(args: cli::ProbeArgs) -> Result<()> {
    let millis = args.sample_size.as_millis();

    println!("Probing file {:?}:", args.input);
    println!("Sample size: {}ms", millis);

    let keyframes = get_keyframes(&args.input, (0, args.sample_size.as_micros().try_into().unwrap()), 0)?;

    // calculate avg spacing between frames
    let avg_diff = {
        let diff = keyframes.iter()
            .zip(keyframes.iter().skip(1))
            .map(|(x, y)| y - x)
            .collect::<Vec<_>>();

        let sum: u128 = diff.iter().sum();

        sum / u128::try_from(diff.len()).unwrap()
    };

    println!("Total keyframes: {}", keyframes.len());
    println!("Keyframe freq: 1/{}ms", millis / u128::try_from(keyframes.len()).unwrap());
    println!("Keyframe avg spacing: {}ms", avg_diff / 1_000);

    println!(
        "Duration: {}",
        humantime::format_duration(Duration::from_micros(util::get_file_length(args.input)?.try_into().unwrap()))
    );

    Ok(())
}
