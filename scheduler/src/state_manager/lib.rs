use rik_scheduler::WorkerState;
use proto::common::Workload;

pub struct Worker {
    status: WorkerState,
    instances: Vec<Workload>
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            status: WorkerState::Unknown,
        }
    }
}