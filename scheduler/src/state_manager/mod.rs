mod lib;

use std::collections::HashMap;
use rik_scheduler::WorkloadInstance;

pub struct StateManager {
    expected_state: HashMap<String, WorkloadInstance>,
}

pub struct Workload {
    replicas: u32,
    instance_id
}