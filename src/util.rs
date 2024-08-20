/// Simple extension trait to avoid duplicating code, allow easy conversion to `ExitCode`
pub trait CommandOutputExt {
    /// Convert into `std::process::ExitCode` easily consistantly
    ///
    /// Equal to `ExitCode::from(1)` in case of signal termination (or any exit code larger than 255)
    fn to_exitcode(&self) -> Result<(), u8>;
}

impl CommandOutputExt for std::process::ExitStatus {
    fn to_exitcode(&self) -> Result<(), u8> {
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
    fn print_escaped_cmd(&self) -> Result<(), u8>;
}

impl CommandExt for std::process::Command {
    fn print_escaped_cmd(&self) -> Result<(), u8> {
        println!("(CMD) {:#?}", self.get_program());
        for arg in self.get_args() {
            println!("      {:#?}", arg);
        }

        Ok(())
    }
}

