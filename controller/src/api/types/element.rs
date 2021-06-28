use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct OnlyId {
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Element {
    pub id: usize,
    pub name: String,
    pub value: String,
}

impl Element {
    pub fn new(id: usize, name: String, value: String) -> Element {
        Element { id, name, value }
    }
}
impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Id: {}, Name: {}, Value: {}",
            self.id, self.name, self.value
        )
    }
}
