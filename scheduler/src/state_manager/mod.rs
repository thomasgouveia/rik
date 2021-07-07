mod lib;

use crate::state_manager::lib::get_random_hash;
use definition::workload::WorkloadDefinition;
use log::{debug, error, info};
use proto::common::{InstanceMetric, ResourceStatus, WorkloadRequestKind};
use rand::seq::IteratorRandom;
use rik_scheduler::{Event, SchedulerError, Worker, WorkloadChannelType, WorkloadRequest};
use std::collections::HashMap;
use std::fmt;
use std::slice::Iter;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

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

#[derive(Debug, Eq, PartialEq)]
enum WorkloadStatus {
    PENDING,
    CREATING,
    DESTROYING,
    RUNNING,
}

pub struct StateManager {
    state: HashMap<String, Workload>,
    workers: Arc<Mutex<Vec<Worker>>>,
    manager_channel: Sender<Event>,
}

impl StateManager {
    pub async fn new(
        manager_channel: Sender<Event>,
        workers: Arc<Mutex<Vec<Worker>>>,
        mut receiver: Receiver<StateManagerEvent>,
    ) -> Result<(), SchedulerError> {
        debug!("Creating StateManager...");
        let mut state_manager = StateManager {
            // We define a mini capacity
            state: HashMap::with_capacity(20),
            manager_channel,
            workers,
        };
        debug!("StateManager receiver is ready");
        state_manager.run(receiver).await
    }

    async fn run(
        &mut self,
        mut receiver: Receiver<StateManagerEvent>,
    ) -> Result<(), SchedulerError> {
        while let Some(message) = receiver.recv().await {
            match message {
                StateManagerEvent::Shutdown => {
                    info!("Shutting down StateManager");
                    return Ok(());
                }
                StateManagerEvent::Schedule(workload) => self.process_schedule_request(workload),
            };
            self.update_state().await;
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

    async fn update_state(&mut self) {
        if self.workers.lock().unwrap().len() == 0 {
            info!("State isn't updated as there is no worker available");
            return ();
        }

        let mut scheduled: Vec<(String, WorkloadInstance)> = Vec::new();

        for (id, workload) in self.state.iter_mut() {
            let length_diff = workload.replicas - workload.instances.len() as u16;

            if length_diff > 0 {
                debug!(
                    "Divergence detected on {}, divergence length: {}",
                    workload.id, length_diff
                );
                for _ in 0..length_diff {
                    // Generate an instance ID, and ensure it is unique
                    let mut workload_id = get_random_hash(4).to_ascii_lowercase();
                    while workload.instances.contains_key(id) {
                        workload_id = get_random_hash(4).to_ascii_lowercase();
                    }
                    scheduled.push((id.clone(), WorkloadInstance::new(workload_id.clone(), ResourceStatus::Pending, 1)));
                }
            }
        }

        for (workload_id, instance) in scheduled.into_iter() {
            if let Some(worker_id) = self.get_eligible_worker() {
                self.manager_channel
                    .send(Event::InstanceMetric("scheduler".to_string(), InstanceMetric {
                        status: ResourceStatus::Pending as i32,
                        metrics: format!("\"workload_id\": \"{}\"", worker_id),
                        instance_id: instance.id.clone(),
                    }))
                    .await;
                self.state.get_mut(&workload_id).unwrap().instances.insert(instance.id.clone(), instance);
            } else {
                error!("Trying to schedule but cannot find any eligible worker");
            }
        }
    }

    fn process_schedule_request(&mut self, request: WorkloadRequest) -> Result<(), SchedulerError> {
        debug!(
            "[process_schedule_request] Received workload id {}, action: {:#?}",
            request.workload_id, request.action
        );

        match request.action {
            WorkloadRequestKind::Create => self.action_create_workload(request),
            WorkloadRequestKind::Destroy => self.action_destroy_workload(request),
        }
    }

    fn action_create_workload(&mut self, request: WorkloadRequest) -> Result<(), SchedulerError> {
        if let Some(workload) = self.state.get(&request.workload_id) {
            if workload.status == ResourceStatus::Destroying {
                error!("Cannot double replicas while workload is being destroyed");
                return Err(SchedulerError::CannotDoubleReplicas);
            }

            let def_replicas = &workload.definition.replicas.unwrap_or(1);
            self.action_add_replicas(&request.workload_id, def_replicas)?;
        } else {
            let workload = Workload {
                id: request.workload_id,
                replicas: request.definition.replicas.unwrap_or(1),
                definition: request.definition,
                instances: HashMap::new(),
                status: ResourceStatus::Pending,
            };

            info!("[process_schedule_request] Received scheduling request for {}, with {:#?} replicas", workload.id, workload.definition.replicas);

            self.state.insert(workload.id.clone(), workload);
        }
        Ok(())
    }

    fn action_add_replicas(
        &mut self,
        workload_id: &String,
        replicas: &u16,
    ) -> Result<(), SchedulerError> {
        let mut workload = match self.state.get_mut(workload_id) {
            Some(wk) => Ok(wk),
            None => Err(SchedulerError::WorkloadDontExists(workload_id.clone())),
        }?;

        debug!(
            "[action_double_replicas] Adding replicas for {}, added {} to {}",
            workload_id, replicas, workload.replicas
        );

        workload.replicas += replicas;
        Ok(())
    }

    fn action_minus_replicas(
        &mut self,
        workload_id: &String,
        replicas: &u16,
    ) -> Result<(), SchedulerError> {
        let mut workload = match self.state.get_mut(workload_id) {
            Some(wk) => Ok(wk),
            None => Err(SchedulerError::WorkloadDontExists(workload_id.clone())),
        }?;
        debug!(
            "[action_double_replicas] Minus replicas for {}, removed {} to {}",
            workload_id, replicas, workload.replicas
        );

        workload.replicas -= replicas;

        Ok(())
    }

    fn action_destroy_workload(&mut self, request: WorkloadRequest) -> Result<(), SchedulerError> {
        let mut workload = self.state.get_mut(&request.workload_id).unwrap();

        if workload.status == ResourceStatus::Destroying {
            return Ok(());
        }

        let def_replicas = &workload.definition.replicas.unwrap_or(1);

        info!(
            "[process_schedule_request] Received destroy request for {}, with {:#?} replicas",
            workload.id, workload.definition.replicas
        );

        if workload.replicas > *def_replicas {
            self.action_minus_replicas(&request.workload_id, def_replicas)?;
        } else {
            info!("Workload {} is getting unscheduled", workload.id);
            workload.status = ResourceStatus::Destroying;
            workload.replicas = 0;
        }
        Ok(())
    }

    fn get_eligible_worker(&self) -> Option<String> {
        let workers = self.workers.lock().unwrap();
        {
            let workers = workers.iter().filter(|worker| worker.is_ready());
            if let Some(worker) = workers.choose(&mut rand::thread_rng()) {
                return Some(worker.id.clone());
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Workload {
    /// The current number of replicas deployed for this workload
    replicas: u16,
    definition: WorkloadDefinition,
    instances: HashMap<String, WorkloadInstance>,
    status: ResourceStatus,
    id: String,
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
            worker_id,
        }
    }

    pub fn status_update(&mut self, status: ResourceStatus) {
        debug!("WorkloadInstance {} went to {:#?}", self.id, self.status);
        self.status = status;
    }
}
