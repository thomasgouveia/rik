use proto::common::{WorkerStatus, Workload};
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;
use tonic::Status;
use std::error::Error;
use std::fmt;
use std::collections::HashMap;

/// Define the structure of message send through the channel between
/// the manager and a worker
pub type WorkloadChannelType = Result<Workload, Status>;

/// The state is shared across multiple threads (Manager & Scheduler)
/// so it is necessary to have a smart pointer & its mutex
pub type StateType = HashMap<String, WorkloadInstance>;

#[derive(Debug)]
pub enum Event {
    /// Workers register to the Scheduler so they can serve
    /// the cluster
    Register(Sender<WorkloadChannelType>, SocketAddr),
    /// Controller can send workload, we use the verb Schedule to describe
    /// this event
    ScheduleRequest(Workload),
    /// This is meant for a controller subscription event
    /// Controller subscribe to the scheduler in order to get updqtes
    Subscribe(Sender<Result<WorkerStatus, Status>>, SocketAddr),
}

#[derive(Debug)]
pub enum SchedulerError {
    ClusterFull,
    RegistrationFailed(String),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for SchedulerError {}


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

#[derive(Debug, Clone)]
pub struct Worker {
    /// Unique ID for the worker, only used internally for now
    pub id: u8,
    /// This channel is used to communicate between the manager
    /// and the worker instance
    /// # Examples
    ///
    /// The following code is used in order to schedule an instance
    /// ```
    /// use rik_scheduler::{Worker, WorkloadChannelType};
    /// use proto::common::{Workload};
    /// use tokio::sync::mpsc::{channel, Receiver, Sender};
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    /// let (sender, receiver) = channel::<WorkloadChannelType>(1024);
    /// let worker = Worker {
    ///     id: 0,
    ///     channel: sender,
    ///     addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    /// }
    /// ```
    pub channel: Sender<WorkloadChannelType>,
    /// Remote addr of the worker
    pub addr: SocketAddr,
}

impl Worker {
    pub fn new(id: u8, channel: Sender<WorkloadChannelType>, addr: SocketAddr) -> Worker {
        Worker { id, channel, addr }
    }
}

#[tonic::async_trait]
pub trait Send<T> {
    async fn send(&self, data: T) -> Result<(), Status>;
}

#[derive(Debug, Clone)]
pub struct WorkloadInstance {
    worker_id: Option<u8>,
    workload: Workload,
}

impl WorkloadInstance {
    pub fn new(workload: Workload, worker: Option<Worker>) -> WorkloadInstance {
        WorkloadInstance {
            workload,
            worker_id: match worker {
                Some(worker) => Some(worker.id),
                _ => None,
            },
        }
    }

    pub fn set_worker(&mut self, id: u8) {
        self.worker_id = Some(id);
    }

    pub fn has_worker(&self) -> bool {
        match self.worker_id {
            Some(_) => true,
            _ => false
        }
    }

    pub fn get_worker_id(&self) -> Option<u8> {
        self.worker_id
    }

    pub fn get_instance_id(&self) -> String {
        self.workload.instance_id.clone()
    }

    pub fn get_workload(self) -> Workload {
        self.workload
    }
}

