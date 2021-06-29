use serde::{Deserialize, Serialize};
use names::Generator;

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceDefinition {
    pub name: Option<String>,
    pub workload_id: usize,
    pub replicas: Option<usize>,
}

#[allow(dead_code)]
impl InstanceDefinition {
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn get_replicas(&mut self) -> usize {
        *self.replicas.get_or_insert(1)
    }

    pub fn get_name(&mut self) -> &mut String {
        let mut random_name_generator = Generator::default();
        self.name.get_or_insert(random_name_generator.next().unwrap())
    }
}
