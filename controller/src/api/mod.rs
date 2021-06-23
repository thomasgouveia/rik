pub mod external;
pub mod internal;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum CRUD {
    Create,
    Delete,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RikError {
    IoError(std::io::Error),
}

pub struct ApiPipe {
    action: CRUD,
    workload_id: usize,
}
impl fmt::Display for ApiPipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Action: {:?}, Workload id: {}",
            self.action, self.workload_id
        )
    }
}
