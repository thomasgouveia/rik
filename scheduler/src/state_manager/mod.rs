use std::collections::HashMap;
use proto::common::ResourceStatus;
use rik_scheduler::{Worker, WorkloadChannelType, Event, SchedulerError, WorkloadRequest};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use log::{info, error, debug};
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum StateManagerEvent {
    Schedule(WorkloadRequest),
    Shutdown,
}

impl fmt::Display for StateManagerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum WorkloadStatus {
    PENDING,
    CREATING,
    DESTROYING,
    RUNNING,
}

pub struct StateManager {
    state: HashMap<String, Workload>,
    manager_channel: Sender<Event>,
}

impl StateManager {
    pub async fn new(manager_channel: Sender<Event>, mut receiver: Receiver<StateManagerEvent>) -> Result<(), SchedulerError> {
        debug!("Creating StateManager...");
        let mut state_manager = StateManager {
            // We define a mini capacity
            state: HashMap::with_capacity(20),
            manager_channel,
        };
        debug!("StateManager receiver is ready");
        state_manager.run(receiver).await
    }

    async fn run(&mut self, mut receiver: Receiver<StateManagerEvent>) -> Result<(), SchedulerError> {
        while let Some(message) = receiver.recv().await {
            match message {
                StateManagerEvent::Shutdown => {
                    info!("Shutting down StateManager");
                    return Ok(())
                },
                StateManagerEvent::Schedule(workload) => self.process_schedule_request(workload),
            };
        }
        Err(SchedulerError::StateManagerFailed)
    }

    async fn send(&self, data: Event) -> Result<(), SchedulerError> {
        self.manager_channel.send(data).await.map_err(|e| {
            error!(
                "Failed to send message from StateManager to Manager, error: {}",
                e
            );
            SchedulerError::ClientDisconnected
        })
    }

    fn process_schedule_request(&mut self, workload: WorkloadRequest) -> Result<(), SchedulerError> {
        debug!("[process_schedule_request] Received workload id {}", workload.workload_id);
        if self.state.contains_key(&workload.workload_id) {
            error!("Re-scheduled an already implemented workload isn't available yet! Stay tuned");
            return Ok(())
        }

        self.state.insert(workload.workload_id.clone(), Workload {
            definition: workload.workload_id,
            instances: HashMap::new(),
            status: WorkloadStatus::PENDING,
            replicas: 0
        });
        Ok(())
    }
}

#[derive(Debug)]
pub struct Workload {
    replicas: u32,
    definition: String,
    instances: HashMap<String, WorkloadInstance>,
    status: WorkloadStatus,
}

#[derive(Debug)]
pub struct WorkloadInstance {
    /// Part of the instance id that define the instance
    id: String,
    /// Current status of this instance
    status: ResourceStatus,
    /// Must be filled, the current id of the worker
    worker_id: u8,
}

impl WorkloadInstance {
    pub fn new(id: String, status: ResourceStatus, worker_id: u8) -> WorkloadInstance {
        WorkloadInstance {
            id,
            status,
            worker_id
        }
    }

    pub fn status_update(&mut self, status: ResourceStatus) {
        debug!("WorkloadInstance {} went to {:#?}", self.id, self.status);
        self.status = status;
    }
}