use env_logger::Env;
use log::{error, info};
use protobuf::well_known_types::Empty;
use rik_scheduler::common::{WorkerStatus, Workload};
use rik_scheduler::controller::controller_server::{
    Controller as ControllerClient, ControllerServer,
};
use rik_scheduler::{Controller, Worker};
use rik_scheduler::worker::worker_server::{Worker as WorkerClient, WorkerServer};
use rik_scheduler::{Event, SchedulerError, WorkloadChannelType};
use std::default::Default;
use std::net::SocketAddr;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{transport::Server, Code, Request, Response, Status};

#[derive(Debug, Clone)]
pub struct GRPCService {
    /// Channel used in order to communicate with the main thread
    /// In the case the worker doesn't know its ID yet, put 0 in the first
    /// item of the tuple
    pub sender: Sender<Event>,
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

#[tonic::async_trait]
impl ControllerClient for GRPCService {
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
        let addr = _request.remote_addr().expect("No remote address found");
        self.sender
            .send(Event::Subscribe(stream_tx, addr))
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
pub struct Manager {
    workers: Vec<Worker>,
    channel: Receiver<Event>,
    controller: Option<Controller>,
    worker_increment: u8,
}


impl Manager {
    async fn run() -> Result<Manager, Box<dyn std::error::Error>> {
        let (sender, receiver) = channel::<Event>(1024);
        let mut instance = Manager {
            workers: Vec::new(),
            channel: receiver,
            controller: None,
            worker_increment: 0,
        };
        instance.run_workers_listener(sender.clone());
        instance.run_controllers_listener(sender.clone());
        let channel_listener = instance.listen();
        channel_listener.await?;
        Ok(instance)
    }

    fn run_workers_listener(&self, sender: Sender<Event>) {
        let server = WorkerServer::new(GRPCService { sender });
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

    fn run_controllers_listener(&self, sender: Sender<Event>) {
        let server = ControllerServer::new(GRPCService { sender });
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
                    if self.workers.is_empty() {
                        self.register(channel.clone(), addr.clone());
                    } else {
                        channel
                            .send(Err(Status::aborted("Cluster is full")))
                            .await?;
                    }
                }
                Event::Schedule(workload) => {
                    if self.workers.is_empty() {
                        error!(
                            "No worker are registered to schedule the workload: {:?}",
                            workload
                        );
                    } else {
                        let worker = &self.workers[0];
                        worker.channel.send(Ok(workload.clone())).await?;
                        info!(
                            "A workload was sent to the worker {}: {:?}",
                            worker.id, workload
                        );
                    }
                }
                Event::Subscribe(channel, addr) => {
                    self.controller = Some(Controller::new(channel.clone(), addr.clone()));
                }
            }
        }
        Ok(())
    }

    fn register(&mut self, channel: Sender<WorkloadChannelType>, addr: SocketAddr) {
        let worker = Worker::new(*&self.worker_increment, channel, addr);
        info!(
            "Worker with ID {} is now registered, ip: {}",
            worker.id, worker.addr
        );
        self.workers.push(worker);
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
