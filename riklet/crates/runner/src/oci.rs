use crate::UMOCI_BIN;
use std::process::Command;

#[derive(Debug)]
pub struct OCI {}

impl OCI {

    pub fn new() -> Self { OCI {} }

    pub fn unpack(&self, src: &String, dest: &String) {
        let output = Command::new(UMOCI_BIN)
            .args(&[
                "unpack",
                "--rootless",
                "--image",
                &src[..],
                &dest[..]
            ])
            .output()
            .unwrap();

        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr))
    }

}