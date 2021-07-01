use crate::command::{RikCommand};
use crate::SKOPEO_BIN;
use log::{debug};

/// Skopeo structure represents the skopeo binary and provide methods to
/// interact with the binary.
#[derive(Debug)]
pub struct Skopeo {
    cmd: RikCommand
}

impl Skopeo {

    pub fn new() -> Self {
        Skopeo {
            cmd: RikCommand::create(SKOPEO_BIN)
        }
    }

    /// Execute the `skopeo copy` command with arguments
    pub fn copy(&self, src: &str, dst: &str) -> bool {
        let (_result, status) = self.cmd.execute(&["copy", src, dst]);

        debug!("{} {} {} {} ended with exit code {}", SKOPEO_BIN, "copy", src, dst, status.code().unwrap());

        if !status.success() {
            return false
        }

        true
    }
}