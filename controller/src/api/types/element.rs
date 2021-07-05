use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct OnlyId {
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Element {
    pub id: usize,
    pub name: String,
    pub value: String,
}

#[allow(dead_code)]
impl Element {
    pub fn new(id: usize, name: String, value: String) -> Element {
        Element { id, name, value }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
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
