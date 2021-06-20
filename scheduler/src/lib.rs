use common::Workload;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;
use tonic::Status;

/// Define the structure of message send through the channel between
/// the manager and a worker
pub type WorkloadChannelType = Result<Workload, Status>;

// Common is needed to be included, as controller & worker
// are using it
pub mod common {
    tonic::include_proto!("common");
}

pub mod worker {
    tonic::include_proto!("worker");
}

pub mod controller {
    tonic::include_proto!("controller");
}

#[derive(Debug)]
pub enum Event {
    Register(Sender<WorkloadChannelType>, SocketAddr),
    Schedule(Workload),
    Subscribe(Sender<Result<WorkerStatus, Status>>),
}

#[derive(Debug)]
pub enum SchedulerError {
    ClusterFull,
    RegistrationFailed(String),
}
