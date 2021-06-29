const SKOPEO_BIN: &str = "skopeo";
const UMOCI_BIN: &str = "umoci";
const RUNC_BIN: &str = "cri";

#[derive(Debug)]
pub enum BinaryError {
    NotFound(String)
}

pub mod skopeo;
pub mod umoci;
pub mod runc;
mod binary;
