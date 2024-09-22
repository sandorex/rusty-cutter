use std::path::{PathBuf, Path};

#[allow(unused_imports)]
pub mod command_extensions {
    pub use std::process::Command;
    pub use super::{CommandExt, CommandOutputExt};
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

pub trait CommandExt {
    /// Prints the command in readable and copy-able format
    fn print_escaped_cmd(&self) -> CommandExit;
}

impl CommandExt for std::process::Command {
    /// Print the whole command with quotes around each argument
    fn print_escaped_cmd(&self) -> CommandExit {
        println!("(CMD) {:?} \\", self.get_program().to_string_lossy());
        let mut iter = self.get_args();
        while let Some(arg) = iter.next() {
            print!("      {:?}", arg.to_string_lossy());

            // do not add backslash on the last argument
            if iter.len() != 0 {
                print!(" \\");
            }

            println!();
        }

        Ok(())
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

    use crate::PathExt;

    #[test]
    fn path_ext_prefix_test() {
        let path = Path::new("/etc/test.sh");
        assert_eq!(path.with_suffix(".bak").as_path(), Path::new("/etc/test.bak.sh"));
    }
}

