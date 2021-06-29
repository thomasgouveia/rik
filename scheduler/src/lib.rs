use common::{WorkerStatus, Workload};
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
    Subscribe(Sender<Result<WorkerStatus, Status>>, SocketAddr),
}

#[derive(Debug)]
pub enum SchedulerError {
    ClusterFull,
    RegistrationFailed(String),
}


#[derive(Debug)]
pub struct Controller {
    /// This channel is used to communicate between the manager
    /// and the controller
    channel: Sender<Result<WorkerStatus, Status>>,
    /// Remote addr of the controller
    addr: SocketAddr,
}

impl Controller {
    pub fn new(channel: Sender<Result<WorkerStatus, Status>>, addr: SocketAddr) -> Controller {
        Controller { channel, addr }
    }
}

#[derive(Debug)]
pub struct Worker {
    /// Unique ID for the worker, only used internally for now
    pub id: u8,
    /// This channel is used to communicate between the manager
    /// and the worker instance
    /// # Examples
    ///
    /// The following code is used in order to schedule an instance
    /// ```ignore
    /// worker.channel.send(Ok(Workload {
    ///     instance_id: String::from("testing"),
    ///     definition: String::from("{}"),
    /// })).await?;
    /// ```
    // TODO: Create a trait to send data
    pub channel: Sender<Result<Workload, Status>>,
    /// Remote addr of the worker
    pub addr: SocketAddr,
}

impl Worker {
    pub fn new(id: u8, channel: Sender<Result<Workload, Status>>, addr: SocketAddr) -> Worker {
        Worker { id, channel, addr }
    }
}