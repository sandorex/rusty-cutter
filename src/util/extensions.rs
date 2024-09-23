use std::path::{PathBuf, Path};

#[allow(unused_imports)]
pub mod command_extensions {
    pub use std::process::Command;
    pub use super::CommandOutputExt;
}

pub type CommandExit = Result<(), u8>;

/// Simple extension trait to avoid duplicating code, allow easy conversion to `ExitCode`
pub trait CommandOutputExt {
    /// Convert into `std::process::ExitCode` easily consistantly
    ///
    /// Equal to `ExitCode::from(1)` in case of signal termination (or any exit code larger than 255)
    fn to_exitcode(&self) -> CommandExit;
}

impl CommandOutputExt for std::process::ExitStatus {
    fn to_exitcode(&self) -> CommandExit {
        // the unwrap_or(1) s are cause even if conversion fails it still failed just termination
        // by signal is larger than 255 that u8 exit code on unix allows
        match TryInto::<u8>::try_into(self.code().unwrap_or(1)).unwrap_or(1) {
            0 => Ok(()),
            x => Err(x),
        }
    }
}

impl CommandOutputExt for std::process::Output {
    fn to_exitcode(&self) -> CommandExit {
        self.status.to_exitcode()
    }
}

pub trait PathExt {
    fn with_suffix(&self, suffix: &str) -> PathBuf;
    fn with_prefix(&self, prefix: &str) -> PathBuf;
}

impl PathExt for Path {
    fn with_suffix(&self, suffix: &str) -> PathBuf {
        match self.extension() {
            Some(x) => self.with_extension(format!("{}.{}", suffix, x.to_string_lossy())),
            None => self.with_extension(suffix),
        }
    }

    fn with_prefix(&self, prefix: &str) -> PathBuf {
        let name = match self.file_name() {
            Some(x) => format!("{}{}", prefix, x.to_string_lossy()),
            None => prefix.to_string(),
        };

        self.with_file_name(name)
    }
}

impl PathExt for PathBuf {
    fn with_suffix(&self, suffix: &str) -> PathBuf {
        self.as_path().with_suffix(suffix)
    }

    fn with_prefix(&self, prefix: &str) -> PathBuf {
        self.as_path().with_prefix(prefix)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::PathExt;

    #[test]
    fn path_ext_prefix_test() {
        // absolute path
        assert_eq!(
            Path::new("/etc/file.txt").with_prefix("temp."),
            Path::new("/etc/temp.file.txt"),
        );

        // no path / parent
        assert_eq!(
            Path::new("file.txt").with_prefix("temp."),
            Path::new("temp.file.txt"),
        );

        // root
        assert_eq!(
            Path::new("/file.txt").with_prefix("temp."),
            Path::new("/temp.file.txt"),
        );

        // no extension
        assert_eq!(
            Path::new("file").with_prefix("temp."),
            Path::new("temp.file"),
        );
    }

    #[test]
    fn path_ext_suffix_test() {
        // file without absolute path
        assert_eq!(
            Path::new("file.txt").with_suffix("temp"),
            Path::new("file.temp.txt"),
        );

        // file with absolute path
        assert_eq!(
            Path::new("/etc/file.txt").with_suffix("temp"),
            Path::new("/etc/file.temp.txt"),
        );

        // file without extension
        assert_eq!(
            Path::new("/etc/file").with_suffix("temp"),
            Path::new("/etc/file.temp"),
        );

        // multiple existing extensions
        assert_eq!(
            Path::new("/file.txt.txt").with_suffix("temp"),
            Path::new("/file.txt.temp.txt"),
        );
    }
}
