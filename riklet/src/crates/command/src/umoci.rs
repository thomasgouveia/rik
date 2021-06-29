use crate::binary::{Binary};
use crate::{UMOCI_BIN};
use log::{debug};

#[derive(Debug)]
pub struct Umoci {
    cmd: Binary
}

impl Umoci {

    pub fn new() -> Option<Self> {
        match Binary::create(UMOCI_BIN) {
            Ok(binary) => Some(Umoci { cmd: binary }),
            _ => None
        }
    }

    pub fn unpack(&self, src: &str, dst: &str) -> bool {
        let (_result, status) = self.cmd.execute(&[
            "unpack",
            "--rootless",
            "--image",
            src,
            dst
        ]);

        debug!("{} {} {} {} ended with exit code {}", UMOCI_BIN, "unpack", src, dst, status.code().unwrap());

        if !status.success() {
            return false
        }

        true
    }
}