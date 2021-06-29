use crate::binary::{Binary};
use crate::{RUNC_BIN};
use log::{debug};

/// Runc structure represents the cri binary and provide methods to
/// interact with the binary.
#[derive(Debug)]
pub struct Runc {
    cmd: Binary
}

impl Runc {

    pub fn new() -> Option<Self> {
        match Binary::create(RUNC_BIN) {
            Ok(binary) => Some(Runc { cmd: binary }),
            _ => None
        }
    }

    /// Execute the `cri run` command with arguments
    pub fn run(&self, bundle: &str, name: &str) -> bool {
        let (_result, status) = self.cmd.execute(&["run", "-b", bundle, name]);

        debug!("{} {} {} {} ended with exit code {}", RUNC_BIN, "run -b", bundle, name, status.code().unwrap());

        if !status.success() {
            return false
        }

        true
    }
}