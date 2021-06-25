use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tenant {
    pub id: usize,
    pub name: String,
}

impl Tenant {
    pub fn new(id: usize, name: String) -> Tenant {
        Tenant { id, name }
    }
}
impl fmt::Display for Tenant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id: {}, Name: {}", self.id, self.name)
    }
}
