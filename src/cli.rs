use clap::{Parser, Subcommand, Args};

/// Wrapper around ffmpeg to do media file editing with minimal transcoding when possible
#[derive(Parser, Debug)]
#[command(name = "rcut", author, version, about)]
pub struct Cli {
    /// Just print commands that would've been ran, do not modify filesystem
    #[arg(long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub cmd: CliCommands,
}

#[derive(Subcommand, Debug)]
pub enum CliCommands {
    // NOTE no command should operate on file in place, always output to a new one

    /// Extract a part of video into a new file
    Extract(ExtractArgs),

    /// Remove a part of a video and save the rest into a new file
    Trim,

    /// Split video file at specific point, or interval
    Split(SplitArgs),

    /// Add together two or more video files of the same type into one
    Concat(ConcatArgs),

    // /// Create overlay video from image
    // Overlay,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ExtractArgs {
    /// Force align time to keyframes (allows cutting without transcoding, but cuts wont be exact)
    #[arg(short, long, default_value_t = false)]
    pub align_keyframe: bool,

    /// Source file
    pub source: String,

    /// Start time of the segment in millis (for detailed format see help)
    #[arg(value_parser = parse_time)]
    pub start_time: u64,

    /// End time of the segment in millis (for detailed format see help)
    #[arg(value_parser = parse_time)]
    pub end_time: u64,

    /// File to output to (if not specified default suffix will be added to source name)
    pub output: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct SplitArgs {
    /// Force the interval or time to align to keyframes (allows splitting without transcoding, but
    /// splits wont be exact)
    #[arg(short, long, default_value_t = false)]
    pub align_keyframe: bool,

    /// File to operate on
    pub source: String,

    #[clap(flatten)]
    group: TimeOrIntervalGroup,

    /// File to output to (if not specified default suffix will be added to source name)
    pub output: Option<String>,
}

#[derive(Debug, Clone, Default, clap::Args)]
#[group(required = true, multiple = false)]
pub struct TimeOrIntervalGroup {
    /// Interval to split the file in millis (for detailed format see help)
    #[arg(short, long, value_parser = parse_time)]
    pub interval: Option<f64>,

    /// Time to split the media file at in millis (for detailed format see help)
    #[arg(short, long, value_parser = parse_time)]
    pub time: Option<f64>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ConcatArgs {
    /// File to output to
    pub output: String,

    #[arg(required=true, num_args=2..)]
    pub input: Vec<String>,
}

// #[derive(Args, Debug, Clone, Default)]
// pub struct OverlayArgs {
//     /// Source file
//     pub source: String,
//
//     /// File to overlay
//     pub overlay: String,
//
//     /// Time for the overlay to start
//     pub overlay_start: f64,
//
//     /// Time for the overlay to end
//     pub overlay_end: f64,
//
//     /// File to output to (if not specified default suffix will be added to source name)
//     pub output: Option<String>,
// }

/// Parse time that is either a timestamp or integer with optional unit suffix
pub fn parse_time(input: &str) -> Result<u64, String> {
    use regex;
    use std::time::Duration;

    let re = regex::Regex::new(
        r#"(?x)
        ^
        (?:
            # basically just match a timestamp 00:00:00[.00]
            (?P<h>[0-9]+)
            :
            (?P<m>[0-9]+)
            :
            (?P<s>[0-9]+)
            (?:
                # optional millis
                \.
                (?P<ms>[0-9]+)
            )?
        )
        |
        (?:
            # match integer and optional unit (ascii only) 1000[ms]
            (?P<int>[0-9]+)
            (?P<unit>[[:alpha:]]{1,2})?
        )
        $"#
    ).expect("Error building parse_time regex");

    if let Some(captures) = re.captures(input) {
        if let Some(hours) = captures.name("h") {
            let hours = hours.as_str().parse::<u64>().unwrap();
            let minutes = captures.name("m").unwrap().as_str().parse::<u64>().unwrap();
            let seconds = captures.name("s").unwrap().as_str().parse::<u64>().unwrap();
            let milliseconds = captures.name("ms").map_or(0, |x| x.as_str().parse::<u64>().unwrap());

            let dur = Duration::from_secs(seconds)
                + Duration::from_secs_f64(minutes as f64 * 60.0)
                + Duration::from_secs_f64(hours as f64 * 60.0 * 60.0)
                + Duration::from_millis(milliseconds);

            // u128 is overkill for this so im using u64
            Ok(dur.as_micros().try_into().unwrap())
        } else {
            let integer = captures.name("int").unwrap().as_str().parse::<u64>().unwrap();

            match captures.name("unit").map(|x| x.as_str()) {
                Some("h") => Ok(Duration::from_secs_f64(integer as f64 * 60.0 * 60.0)),
                Some("m") => Ok(Duration::from_secs_f64(integer as f64 * 60.0)),
                Some("s") => Ok(Duration::from_secs(integer)),
                // default to millis if no unit
                Some("ms") | None => Ok(Duration::from_millis(integer)),
                Some("us") => Ok(Duration::from_micros(integer)),
                Some(x) => Err(format!("Invalid unit suffix {:#?}", x)),
            }.map(|x| TryInto::<u64>::try_into(x.as_micros()).unwrap())
        }
    } else {
        Err("Invalid time format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }

    #[test]
    fn test_time_parsing() {
        // NOTE remember the output is in microseconds!

        assert_eq!(parse_time("01:01:01"), Ok(3_661_000_000));
        assert_eq!(parse_time("10:00:00"), parse_time("10h"));

        // make sure all units are properly calculated
        assert_eq!(parse_time("00:00:00"), Ok(0));
        assert_eq!(parse_time("00:00:00.00"), Ok(0));
        assert_eq!(parse_time("00:00:00.1"), Ok(1_000));
        assert_eq!(parse_time("00:00:01"), Ok(1_000_000));
        assert_eq!(parse_time("00:01:00"), Ok(60_000_000));
        assert_eq!(parse_time("01:00:00"), Ok(3_600_000_000));

        // also the literal format
        assert_eq!(parse_time("1us"), Ok(1));
        assert_eq!(parse_time("1ms"), Ok(1_000));
        assert_eq!(parse_time("1s"), Ok(1_000_000));
        assert_eq!(parse_time("1m"), Ok(60_000_000));
        assert_eq!(parse_time("1h"), Ok(3_600_000_000));

        assert!(matches!(parse_time(""), Err(_)));
        assert!(matches!(parse_time("1 "), Err(_)));
        assert!(matches!(parse_time("1 us"), Err(_)));
    }
}

