mod util;
mod video;

use std::process::ExitCode;
use video::cut_video;

fn main() -> ExitCode {
    cut_video("video.mkv", 2_500_000, 11_000_000);
    // let times = get_keyframes("video.mkv", Some((2.0, 10.0)));
    // dbg!(times);

    ExitCode::FAILURE
}

