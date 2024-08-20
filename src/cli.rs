mod util;
mod video;

use std::process::ExitCode;
use video::cut_video;

fn main() -> ExitCode {
    match cut_video("video.mkv", "output.mkv", (1_500_000, 11_500_000), false) {
        Ok(_) => ExitCode::SUCCESS,
        Err(x) => ExitCode::from(x),
    }
}

