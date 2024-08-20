mod util;
mod video;

use std::process::ExitCode;
use video::cut_video;

fn main() -> ExitCode {
    cut_video("video.mkv", "output.mkv", 2_000_000, 12_000_000, true);
    // let times = get_keyframes("video.mkv", Some((2.0, 10.0)));
    // dbg!(times);

    ExitCode::FAILURE
}

