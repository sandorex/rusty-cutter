mod util;
mod operations;

/// Crate version with git describe appended
pub const FULL_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "-", env!("VERGEN_GIT_DESCRIBE"));

pub use util::Timestamp;

pub use operations::keyframes::get_keyframes;
pub use operations::concat::concat_files;
pub use operations::cut::extract_segment;
pub use operations::split::{split_at, split_every};

