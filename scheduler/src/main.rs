use common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkerStatus, Workload};
use controller::controller_server::{Controller, ControllerServer};
use env_logger::Env;
use log::{error, info};
use protobuf::well_known_types::Empty;
use std::default::Default;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{transport::Server, Code, Request, Response, Status};
use worker::worker_server::{Worker as WorkerClient, WorkerServer};

// Common is needed to be included, as controller & worker
// are using it
pub mod common {
    tonic::include_proto!("common");
}

pub mod controller {
    tonic::include_proto!("controller");
}

pub mod worker {
    tonic::include_proto!("worker");
}

#[derive(Debug, Clone)]
pub struct WorkerService {
    /// Channel used in order to communicate with the main thread
    /// In the case the worker doesn't know its ID yet, put 0 in the first
    /// item of the tuple
    sender: Sender<Event>,
}

#[tonic::async_trait]
impl WorkerClient for WorkerService {
    type RegisterStream = ReceiverStream<Result<Workload, Status>>;

    async fn register(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        // Streaming channel that sends workloads
        let (stream_tx, stream_rx) = channel::<Result<Workload, Status>>(1024);
        let addr = _request.remote_addr().expect("No remote address found");

        self.sender
            .send(Event::Register(stream_tx, addr))
            .await
            .map_err(|_| {
                Status::new(
                    Code::Unavailable,
                    "Worker service cannot process your request",
                )
            })?;

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

#[derive(Debug, Clone)]
pub struct ControllerService {
    // Channel used in order to communicate with the Manager
    sender: Sender<Event>,
}

#[tonic::async_trait]
impl Controller for ControllerService {
    async fn schedule_instance(&self, _request: Request<Workload>) -> Result<Response<()>, Status> {
        self.sender
            .send(Event::Schedule(_request.get_ref().clone()))
            .await
            .map_err(|_| {
                Status::new(
                    Code::Unavailable,
                    "Controller service cannot process your request",
                )
            })?;

        Ok(Response::new(()))
    }

    type GetStatusUpdatesStream = ReceiverStream<Result<WorkerStatus, Status>>;

    async fn get_status_updates(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::GetStatusUpdatesStream>, Status> {
        let (stream_tx, stream_rx) = channel::<Result<WorkerStatus, Status>>(1024);
        self.sender
            .send(Event::Subscribe(stream_tx))
            .await
            .map_err(|_| {
                Status::new(
                    Code::Unavailable,
                    "Controller service cannot process your request",
                )
            })?;

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }
}

#[derive(Debug)]
pub enum Event {
    Register(Sender<Result<Workload, Status>>, SocketAddr),
    Schedule(Workload),
    Subscribe(Sender<Result<WorkerStatus, Status>>),
}

#[derive(Debug)]
pub struct Worker {
    /// Unique ID for the worker, only used internally for now
    id: u8,
    /// This channel is used to communicate between the manager
    /// and the worker instance
    /// # Examples
    ///
    /// The following code is used in order to schedule an instance
    /// ```
    /// worker.channel.send(Ok(Workload {
    ///     instance_id: String::from("testing"),
    ///     definition: String::from("{}"),
    /// })).await?;
    /// ```
    channel: Sender<Result<Workload, Status>>,
    /// Remote addr of the worker
    addr: SocketAddr,
}

#[derive(Debug)]
struct Manager {
    workers: Vec<Arc<Worker>>,
    channel: Receiver<Event>,
    controller_channel: Option<Sender<Result<WorkerStatus, Status>>>,
    worker_increment: u8,
}

impl Manager {
    async fn run() -> Result<Manager, Box<dyn std::error::Error>> {
        let (sender, receiver) = channel::<Event>(1024);
        let mut instance = Manager {
            workers: Vec::new(),
            channel: receiver,
            controller_channel: None,
            worker_increment: 0,
        };
        instance.run_worker_listener(sender.clone());
        instance.run_controller_listener(sender.clone());
        let channel_listener = instance.listen();
        channel_listener.await?;
        Ok(instance)
    }

    fn run_worker_listener(&self, sender: Sender<Event>) {
        let server = WorkerServer::new(WorkerService { sender });
        tokio::spawn(async move {
            let server = Server::builder()
                .add_service(server)
                .serve("127.0.0.1:8081".parse().unwrap());

            info!("Worker gRPC listening thread up");

            if let Err(e) = server.await {
                error!("{}", e);
            }
        });
    }

    fn run_controller_listener(&self, sender: Sender<Event>) {
        let server = ControllerServer::new(ControllerService { sender });
        tokio::spawn(async move {
            let server = Server::builder()
                .add_service(server)
                .serve("127.0.0.1:10000".parse().unwrap());

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
                    self.worker_increment += 1;
                    let worker = Arc::new(Worker {
                        id: *&self.worker_increment,
                        channel: channel.clone(),
                        addr: addr.clone(),
                    });
                    info!(
                        "Worker with ID {} is now registered, ip: {}",
                        worker.id, worker.addr
                    );

                    worker
                        .channel
                        .send(Ok(Workload {
                            instance_id: String::from("testing"),
                            definition: String::from("{}"),
                        }))
                        .await?;
                    worker
                        .channel
                        .send(Err(Status::aborted("Cannot register now")))
                        .await?;
                    self.workers.push(worker);
                }
                Event::Schedule(workload) => {
                    // TODO: Handle empty workers case
                    // let worker = self.workers[0].clone();
                    // worker.channel.send(Ok(workload.clone())).await?;
                    info!("reveived workload: {:?}", workload);
                }
                Event::Subscribe(channel) => {
                    self.controller_channel = Some(channel.clone());
                }
            }
        }
        Ok(())
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
