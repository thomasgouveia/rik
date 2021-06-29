use std::process::{Command, ExitStatus};
use std::ffi::OsStr;
use std::path::PathBuf;
use log::{debug, error};
use crate::{BinaryError};
use shared::utils::find_binary;

#[derive(Debug)]
pub struct Binary {
    binary: PathBuf
}

impl Binary {

    /// Create a binary instance in order to control a binary programmatically.
    pub fn create(binary: &str) -> Result<Self, BinaryError> {
        match find_binary(binary) {
            Some(binary_path) => {
                debug!("Binary '{}' found : '{}'", binary, binary_path.display());
                Ok(Binary { binary: binary_path })
            },
            None => {
                error!("Binary '{}' not found. It is installed on host and added to the PATH ?", binary);
                Err(BinaryError::NotFound(binary.to_string()))
            }
        }
    }

    /// Command executor wrapper
    pub fn execute<I, S>(&self, args: I) -> (Vec<u8>, ExitStatus)
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr>
    {
        let output = Command::new(&self.binary)
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
    use crate::binary::{Cmd};

    #[test]
    fn it_execute_echo_command() {
        let command = Cmd::create("echo");

        let (result, status) = command.execute(&["test"]);

        let result_str = String::from_utf8_lossy(&result);

        assert_eq!(result_str, "test\n");
        assert_eq!(status.success(), true);
    }

    #[test]
    fn it_return_error() {
        let command = Cmd::create("cat");

        let (result, status) = command.execute(&["/somethingnotexisting"]);

        assert_eq!(status.success(), false);
    }
}