use std::process::{Command, ExitStatus};
use std::ffi::OsStr;

#[derive(Debug)]
pub struct RikCommand {
    binary: String
}

impl RikCommand {

    pub fn create(binary: &str) -> Self {
        RikCommand {
            binary: String::from(binary)
        }
    }

    /// Command executor wrapper
    pub fn execute<I, S>(&self, args: I) -> (Vec<u8>, ExitStatus)
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr>
    {


        let output = Command::new(&self.binary[..])
            .args(args)
            .output()
            .unwrap();

        let stdout = output.stdout;
        let stderr = output.stderr;
        let status = output.status;

        if stderr.len() != 0 && !status.success() {
            return (stderr, status)
        }

        (stdout, status)
    }
}


#[cfg(test)]
mod tests {
    use crate::command::{RikCommand, RikCommandError};

    #[test]
    fn it_execute_echo_command() {
        let command = RikCommand::create("echo");

        let (result, status) = command.execute(&["test"]);

        let result_str = String::from_utf8_lossy(&result);

        assert_eq!(result_str, "test\n");
        assert_eq!(status.success(), true);
    }

    #[test]
    fn it_return_error() {
        let command = RikCommand::create("cat");

        let (result, status) = command.execute(&["/somethingnotexisting"]);

        assert_eq!(status.success(), false);
    }
}