mod config_parser;
mod state_manager;
mod grpc;

use env_logger::Env;
use log::{error, info, debug};
use proto::common::{Workload};
use rik_scheduler::{Controller, SchedulerError, WorkloadInstance, StateType, Worker};
use proto::worker::worker_server::{Worker as WorkerClient, WorkerServer};
use rik_scheduler::{Event, WorkloadChannelType};
use std::default::Default;
use std::net::{SocketAddr, SocketAddrV4};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use std::collections::{HashMap};
use tokio::sync::mpsc::error::SendError;
use rand::seq::SliceRandom;
use crate::config_parser::{ConfigParser};
use crate::grpc::GRPCService;
use tonic::transport::Server;
use proto::controller::controller_server::ControllerServer;


#[derive(Debug)]
pub struct Manager {
    workers: Vec<Worker>,
    channel: Receiver<Event>,
    controller: Option<Controller>,
    worker_increment: u8,
    state: StateType,
    expected_state: StateType,
}


impl Manager {
    async fn run(workers_listener: SocketAddrV4, controllers_listener: SocketAddrV4) -> Result<Manager, Box<dyn std::error::Error>> {
        let (sender, receiver) = channel::<Event>(1024);
        let mut instance = Manager {
            workers: Vec::new(),
            channel: receiver,
            controller: None,
            worker_increment: 0,
            state: HashMap::new(),
            expected_state: HashMap::new(),
        };
        instance.run_workers_listener(workers_listener, sender.clone());
        instance.run_controllers_listener(controllers_listener, sender.clone());
        let channel_listener = instance.listen();
        channel_listener.await?;
        Ok(instance)
    }

    fn run_workers_listener(&self, listener: SocketAddrV4, sender: Sender<Event>) {
        let server = WorkerServer::new(GRPCService::new(sender));
        tokio::spawn(async move {
            let server = Server::builder()
                .add_service(server)
                .serve(listener.into());

            info!("Worker gRPC listening on {}", listener);

            if let Err(e) = server.await {
                error!("{}", e);
            }
        });
    }

    fn run_controllers_listener(&self, listener: SocketAddrV4, sender: Sender<Event>) {
        let server = ControllerServer::new(GRPCService::new(sender));
        tokio::spawn(async move {
            let server = Server::builder()
                .add_service(server)
                .serve(listener.into());

            info!("Controller gRPC listening on {}", listener);

            if let Err(e) = server.await {
                error!("{}", e);
            }
        });
    }

    async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(e) = &self.channel.recv().await {
            match e {
                Event::Register(channel, addr) => {
                    self.register(channel.clone(), addr.clone())?
                },
                Event::ScheduleRequest(workload) => {
                    debug!("New workload definition received to schedule {}", workload.instance_id);
                    self.update_expected_state(WorkloadInstance::new(workload.clone(), None)).await;
                },
                Event::Subscribe(channel, addr) => {
                    self.controller = Some(Controller::new(channel.clone(), addr.clone()));
                }
            }
        }
        Ok(())
    }

    fn get_next_id(&mut self) -> Result<u8, SchedulerError> {
        match self.worker_increment {
            u8::MAX => Err(SchedulerError::ClusterFull),
            _ => {
                self.worker_increment += 1;
                Ok(self.worker_increment)
            },
        }
    }

    fn register(&mut self, channel: Sender<WorkloadChannelType>, addr: SocketAddr) -> Result<(), SchedulerError> {
        let worker = Worker::new(self.get_next_id()?, channel, addr);
        info!(
            "Worker with ID {} is now registered, ip: {}",
            worker.id, worker.addr
        );
        self.workers.push(worker);
        Ok(())
    }

    async fn schedule(&mut self, workload: WorkloadInstance) -> Result<(), SendError<WorkloadChannelType>> {
        if !workload.has_worker() {
            error!("Tried to schedule workload while no worker assigned");
            return Ok(());
        }
        match self.get_worker_sender(workload.get_worker_id().unwrap()) {
            Some(sender) => {
                info!(
                    "A workload was sent to the worker {}: {:?}",
                    workload.get_worker_id().unwrap(), workload
                );
                sender.send(Ok(workload.get_workload())).await?;
            },
            _ => {
                error!("Tried to schedule workload on a no longer existing worker");
                return Ok(());
            }
        }
        Ok(())
    }

    fn get_worker_sender(&self, worker_id: u8) -> Option<Sender<WorkloadChannelType>> {
        for worker in self.workers.iter() {
            if worker.id == worker_id {
                return Some(worker.channel.clone());
            }
        }

        None
    }

    async fn update_expected_state(&mut self, item: WorkloadInstance) {
        let instance_id = item.get_instance_id();
        let old_instance = self.expected_state.insert(instance_id.clone(), item);

        match old_instance {
            Some(_) => debug!("Instance {} updated into the expected state", instance_id),
            None => debug!("Inserted a new instance into the expected state, id: {}", instance_id)
        };

        self.scan_diff_state().await;
    }

    async fn scan_diff_state(&mut self) {
        for (instance_id, workload) in self.expected_state.clone().iter() {
            if !self.state.contains_key(instance_id) {
                info!("Detected diff between expected state & new state, updating with instance_id {}", instance_id);
                let worker = self.workers.choose(&mut rand::thread_rng());
                match worker {
                    Some(worker) => {
                        let mut workload = workload.clone();
                        workload.set_worker(worker.id);
                        self.schedule(workload.clone()).await;
                        self.state.insert(workload.get_instance_id(), workload);
                    },
                    _ => (),
                };
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigParser::new()?;
    env_logger::Builder::from_env(Env::default().default_filter_or(&config.verbosity_level)).init();
    info!("Starting up...");
    let manager = Manager::run(config.workers_endpoint, config.controller_endpoint);
    manager.await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Request;

    #[tokio::test]
    #[should_panic(expected = "No remote address found")]
    async fn test_grpc_service_register_should_panic() -> () {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);

        let mock_request = Request::new(());
        service.register(mock_request).await.unwrap();
        receiver.recv().await;
        ()
    }
}
