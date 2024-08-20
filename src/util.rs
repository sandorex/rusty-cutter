/// Simple extension trait to avoid duplicating code, allow easy conversion to `ExitCode`
pub trait CommandOutputExt {
    /// Convert into `std::process::ExitCode` easily consistantly
    ///
    /// Equal to `ExitCode::from(1)` in case of signal termination (or any exit code larger than 255)
    fn to_exitcode(&self) -> std::process::ExitCode;
}

impl CommandOutputExt for std::process::ExitStatus {
    fn to_exitcode(&self) -> std::process::ExitCode {
        // the unwrap_or(1) s are cause even if conversion fails it still failed just termination
        // by signal is larger than 255 that u8 exit code on unix allows
        std::process::ExitCode::from(
            TryInto::<u8>::try_into(self.code().unwrap_or(1)
        ).unwrap_or(1))
    }
}

impl CommandOutputExt for std::process::Output {
    fn to_exitcode(&self) -> std::process::ExitCode {
        self.status.to_exitcode()
    }
}

pub trait CommandExt {
    /// Prints the command in readable and copy-able format
    fn print_escaped_cmd(&self) -> std::process::ExitCode;
}

impl CommandExt for std::process::Command {
    fn print_escaped_cmd(&self) -> std::process::ExitCode {
        println!("(CMD) {:#?}", self.get_program());
        for arg in self.get_args() {
            println!("      {:#?}", arg);
        }

        std::process::ExitCode::SUCCESS
    }
}

