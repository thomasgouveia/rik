use log::info;
use proto::common::{InstanceMetric, WorkerMetric, WorkerStatus, Workload};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;
use tonic::Status;
use node_metrics::Metrics;

/// Define the structure of message send through the channel between
/// the manager and a worker
pub type WorkloadChannelType = Result<Workload, Status>;

/// The state is shared across multiple threads (Manager & Scheduler)
/// so it is necessary to have a smart pointer & its mutex
pub type StateType = HashMap<u32, WorkloadInstance>;

#[derive(Debug)]
pub enum Event {
    /// Workers register to the Scheduler so they can serve
    /// the cluster
    Register(Sender<WorkloadChannelType>, SocketAddr, String),
    /// Controller can send workload, we use the verb Schedule to describe
    /// this event
    ScheduleRequest(Workload),
    /// This is meant for a controller subscription event
    /// Controller subscribe to the scheduler in order to get updqtes
    Subscribe(Sender<Result<WorkerStatus, Status>>, SocketAddr),
    /// Metrics received from workers to tell about themselves.
    /// The first string is the identifier
    /// ```
    /// use proto::common::{WorkerMetric};
    /// let metrics = WorkerMetric {
    ///     status: 1,
    ///     metrics: "{metricA: 10, metricB: 100}".to_string()
    /// };
    /// ```
    WorkerMetric(String, WorkerMetric),
    /// Metrics relative to a single instance of a workload
    /// ```
    /// use proto::common::{InstanceMetric};
    /// let metrics = InstanceMetric {
    ///     status: 1,
    ///     metrics: "{metricA: 10, metricB: 100}".to_string()
    /// };
    /// ```
    InstanceMetric(String, InstanceMetric),
}

#[derive(Debug)]
pub enum SchedulerError {
    /// Current max is 256 workers, given more workers, it returns
    /// a cluster full error
    ClusterFull,
    /// Worker registration process failed
    RegistrationFailed(String),
    /// gRPC client got disconnected
    ClientDisconnected,
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for SchedulerError {}

#[derive(Debug)]
pub enum WorkerState {
    /// Worker is ready to receive workloads
    Ready,
    /// Worker is not / no more ready to receive workloads
    /// containers are relocated in case it switches from Ready state to non-ready
    NotReady,
}

impl fmt::Display for WorkerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerState::Ready => write!(f, "Ready"),
            WorkerState::NotReady => write!(f, "Not Ready"),
        }
    }
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
    /// ```
    /// use rik_scheduler::{Worker, WorkloadChannelType};
    /// use proto::common::{Workload};
    /// use tokio::sync::mpsc::{channel, Receiver, Sender};
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    /// let (sender, receiver) = channel::<WorkloadChannelType>(1024);
    /// let worker = Worker::new(0, sender, "127.0.0.1:8080".parse().unwrap(), "debian-test".to_string());
    /// ```
    pub channel: Sender<WorkloadChannelType>,
    /// Remote addr of the worker
    pub addr: SocketAddr,
    /// Worker hostname, must be unique
    pub hostname: String,
    /// State of worker
    state: WorkerState,
    /// Most recent metric the worker has on its state
    metric: Option<Metrics>,
}

impl Worker {
    pub fn new(
        id: u8,
        channel: Sender<WorkloadChannelType>,
        addr: SocketAddr,
        hostname: String,
    ) -> Worker {
        Worker {
            id,
            channel,
            addr,
            hostname,
            state: WorkerState::NotReady,
            metric: None,
        }
    }

    pub fn set_channel(&mut self, sender: Sender<WorkloadChannelType>) {
        self.channel = sender;
    }

    pub fn set_state(&mut self, state: WorkerState) {
        self.state = state;
        info!("Worker {} flipped to {} state", self.hostname, self.state);
    }

    pub fn get_state(&self) -> &WorkerState {
        &self.state
    }

    pub fn set_metrics(&mut self, metric: Metrics) {
        self.metric = Some(metric);
        self.update_state();
    }

    pub fn get_metrics(&self) -> &Option<Metrics> {
        &self.metric
    }

    fn update_state(&mut self) {
        match self.state {
            WorkerState::Ready => {
                if self.channel.is_closed() {
                    self.set_state(WorkerState::NotReady);
                }
            },
            WorkerState::NotReady => {
                if !self.channel.is_closed() {
                    self.set_state(WorkerState::Ready);
                }
            }
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, WorkerState::Ready)
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
            _ => false,
        }
    }

    pub fn get_worker_id(&self) -> Option<u8> {
        self.worker_id
    }

    pub fn get_instance_id(&self) -> u32 {
        self.workload.instance_id.clone()
    }

    pub fn get_workload(self) -> Workload {
        self.workload
    }
}
