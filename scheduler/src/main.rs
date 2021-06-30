use env_logger::Env;
use log::{error, info, debug};
use proto::common::{WorkerStatus, Workload};
use proto::controller::controller_server::{
    Controller as ControllerClient, ControllerServer,
};
use rik_scheduler::{Controller, Worker, Send, SchedulerError, WorkloadInstance, StateType};
use proto::worker::worker_server::{Worker as WorkerClient, WorkerServer};
use rik_scheduler::{Event, WorkloadChannelType};
use std::default::Default;
use std::net::SocketAddr;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{transport::Server, Code, Request, Response, Status};
use std::collections::{HashMap};
use tokio::sync::mpsc::error::SendError;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct GRPCService {
    /// Channel used in order to communicate with the main thread
    /// In the case the worker doesn't know its ID yet, put 0 in the first
    /// item of the tuple
    sender: Sender<Event>,
}

impl GRPCService {
    fn new(sender: Sender<Event>) -> GRPCService {
        GRPCService {
            sender
        }
    }
}

#[tonic::async_trait]
impl Send<Event> for GRPCService {
    async fn send(&self, data: Event) -> Result<(), Status> {
        self.sender
            .send(data)
            .await
            .map_err(|e| {
                error!("Failed to send message from gRPCService to Manager, error: {}", e);
                Status::new(
                    Code::Unavailable,
                    "We cannot process your request at this time"
                )
            })

    }
}

#[tonic::async_trait]
impl WorkerClient for GRPCService {
    type RegisterStream = ReceiverStream<WorkloadChannelType>;

    async fn register(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        // Streaming channel that sends workloads
        let (stream_tx, stream_rx) = channel::<WorkloadChannelType>(1024);
        let addr = _request.remote_addr().expect("No remote address found");
        self.send(Event::Register(stream_tx, addr)).await?;

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }

    /**
     * For now we can only fetch a single status updates, as `try_next`
     * isn't blocking! :(
     * We should make this update of data blocking, so we can receive any status
     * update
     *
     **/
    async fn send_status_updates(
        &self,
        _request: Request<tonic::Streaming<WorkerStatus>>,
    ) -> Result<Response<()>, Status> {
        let mut stream = _request.into_inner();

        while let Some(data) = stream.try_next().await? {
            info!("Getting some info");
            info!("{:#?}", data);
        }

        Ok(Response::new(()))
    }
}

#[tonic::async_trait]
impl ControllerClient for GRPCService {
    async fn schedule_instance(&self, _request: Request<Workload>) -> Result<Response<()>, Status> {
        self.send(Event::ScheduleRequest(_request.get_ref().clone())).await?;

        Ok(Response::new(()))
    }

    type GetStatusUpdatesStream = ReceiverStream<Result<WorkerStatus, Status>>;

    async fn get_status_updates(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::GetStatusUpdatesStream>, Status> {
        let (stream_tx, stream_rx) = channel::<Result<WorkerStatus, Status>>(1024);
        let addr = _request.remote_addr().expect("No remote address found");
        self.send(Event::Subscribe(stream_tx, addr)).await?;

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }
}

#[derive(Debug)]
pub struct Manager {
    workers: Vec<Worker>,
    channel: Receiver<Event>,
    controller: Option<Controller>,
    worker_increment: u8,
    state: StateType,
    expected_state: StateType
}


impl Manager {
    async fn run() -> Result<Manager, Box<dyn std::error::Error>> {
        let (sender, receiver) = channel::<Event>(1024);
        let mut instance = Manager {
            workers: Vec::new(),
            channel: receiver,
            controller: None,
            worker_increment: 0,
            state: HashMap::new(),
            expected_state: HashMap::new(),
        };
        instance.run_workers_listener(sender.clone());
        instance.run_controllers_listener(sender.clone());
        let channel_listener = instance.listen();
        channel_listener.await?;
        Ok(instance)
    }

    fn run_workers_listener(&self, sender: Sender<Event>) {
        let server = WorkerServer::new(GRPCService::new(sender));
        tokio::spawn(async move {
            let server = Server::builder()
                .add_service(server)
                .serve("127.0.0.1:4995".parse().unwrap());

            info!("Worker gRPC listening thread up");

            if let Err(e) = server.await {
                error!("{}", e);
            }
        });
    }

    fn run_controllers_listener(&self, sender: Sender<Event>) {
        let server = ControllerServer::new(GRPCService::new(sender));
        tokio::spawn(async move {
            let server = Server::builder()
                .add_service(server)
                .serve("127.0.0.1:4996".parse().unwrap());

            info!("Controller gRPC listening thread up");

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
        debug!("[schedule] {:#?}", workload.get_worker_id());
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
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    info!("Starting up...");
    let manager = Manager::run();
    manager.await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[should_panic(expected = "No remote address found")]
    async fn test_grpc_service_register_should_panic() -> () {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService {
            sender: sender,
        };

        let mock_request = Request::new(());
        service.register(mock_request).await.unwrap();
        receiver.recv().await;
        ()
    }
}
