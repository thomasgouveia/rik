use crate::binary::{Binary};
use crate::{SKOPEO_BIN};
use log::{debug};

/// Skopeo structure represents the skopeo binary and provide methods to
/// interact with the binary.
#[derive(Debug)]
pub struct Skopeo {
    cmd: Binary
}

impl Skopeo {

    pub fn new() -> Option<Self> {
        match Binary::create(SKOPEO_BIN) {
            Ok(binary) => Some(Skopeo { cmd: binary }),
            _ => None
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