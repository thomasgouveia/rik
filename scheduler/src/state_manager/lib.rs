use rik_scheduler::WorkerState;
use proto::common::Workload;
use crate::state_manager::Workload;

pub struct Worker {
    status: WorkerState,
    instances: Vec<Workload>
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            status: WorkerState::Unknown,
            instances: Vec::new()
        }
    }
}
