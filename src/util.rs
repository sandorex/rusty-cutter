use crate::ExitResult;

#[allow(unused_imports)]
pub mod command_extensions {
    pub use std::process::Command;
    pub use super::{CommandExt, CommandOutputExt};
}

/// Simple extension trait to avoid duplicating code, allow easy conversion to `ExitCode`
pub trait CommandOutputExt {
    /// Convert into `std::process::ExitCode` easily consistantly
    ///
    /// Equal to `ExitCode::from(1)` in case of signal termination (or any exit code larger than 255)
    fn to_exitcode(&self) -> ExitResult;
}

impl CommandOutputExt for std::process::ExitStatus {
    fn to_exitcode(&self) -> ExitResult {
        // the unwrap_or(1) s are cause even if conversion fails it still failed just termination
        // by signal is larger than 255 that u8 exit code on unix allows
        match TryInto::<u8>::try_into(self.code().unwrap_or(1)).unwrap_or(1) {
            0 => Ok(()),
            x => Err(x),
        }
    }
}

impl CommandOutputExt for std::process::Output {
    fn to_exitcode(&self) -> Result<(), u8> {
        self.status.to_exitcode()
    }
}

pub trait CommandExt {
    /// Prints the command in readable and copy-able format
    fn print_escaped_cmd(&self) -> ExitResult;
}

impl CommandExt for std::process::Command {
    /// Print the whole command with quotes around each argument
    fn print_escaped_cmd(&self) -> ExitResult {
        println!("(CMD) {:#?}", self.get_program());
        for arg in self.get_args() {
            println!("      {:#?}", arg);
        }

        Ok(())
    }
}

/// Simple struct to delete a file after it goes out of scope
pub struct TempFile<'a>(pub &'a str);

impl<'a> Drop for TempFile<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

