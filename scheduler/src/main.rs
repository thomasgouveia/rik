use tonic::{transport::Server, Request, Response, Status, Code};
use common::{ResourceStatus, WorkerMetric, InstanceMetric, WorkerStatus, Workload};
use worker::worker_server::{Worker as WorkerClient, WorkerServer};
use protobuf::well_known_types::Empty;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc::{channel, Sender};
use log::{info, error};
use env_logger::Env;
use std::sync::{Arc};
use std::net::SocketAddr;

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

type EventChannel = (u8, Event);

#[derive(Debug, Clone)]
pub struct WorkerService {
    /// Channel used in order to communicate with the main thread
    /// In the case the worker doesn't know its ID yet, put 0 in the first
    /// item of the tuple
    sender: Sender<EventChannel>,
}

#[tonic::async_trait]
impl WorkerClient for WorkerService {
    type RegisterStream = ReceiverStream<Result<Workload, Status>>;

    async fn register(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        // Streaming channel that sends workloads
        let (stream_tx, stream_rx) = channel(1024);
        // Channel coming from scheduling thread to send requests to schedule instances
        let (tx, mut rx) = channel::<Event>(1024);
        let addr = _request.remote_addr()
            .expect("No remote address found");

        // At this time the worker doesn't have an id
        self.sender.send((0, Event::Register(tx, addr)))
            .await
            .map_err(|_| Status::new(Code::Unavailable, "Worker service cannot process your request"))?;

        tokio::spawn(async move {
            // There is an issue in here, as if we lose the connection with the remote client
            // this thread continue to leave, it is not what we want!
            while let Some(event) = rx.recv().await {
                match event {
                    Event::RegisterSuccessful(_) => {
                        info!("This worker got registered successfully, now listening for instances");
                    },
                    Event::RegisterFailed => {
                        info!("This worker failed its registration process, closing");
                    },
                    Event::Schedule(e) => {
                        info!("New workload requesting to schedule {:#?}", e);
                        stream_tx.send(Ok(e)).await.unwrap_or_else(|e| { error!("Error is {}", e)});
                    },
                    _ => unimplemented!("Worker event not implemented"),
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }

    async fn send_status_updates(
        &self,
        _request: Request<tonic::Streaming<WorkerStatus>>,
    ) -> Result<Response<()>, Status> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum Event {
    Register(Sender<Event>, SocketAddr),
    RegisterSuccessful(Arc<Worker>),
    RegisterFailed,
    Unregister,
    Schedule(Workload),
    Information(String)
}

#[derive(Debug)]
pub struct Worker {
    id: u8,
    channel: Sender<Event>,
    addr: SocketAddr,
}

#[derive(Debug)]
struct Manager {
    workers: Vec<Arc<Worker>>,
}

impl Manager {
    fn new() -> Manager {
        Manager {
            workers: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    info!("Starting up...");

    let mut worker_id: u8 = 0;
    let (sender, mut receiver) = channel::<EventChannel>(1024);
    let service = WorkerService {
        sender,
    };
    let server = WorkerServer::new(service);
    let mut manager = Manager::new();

    tokio::spawn(async move {
        let server =  Server::builder()
            .add_service(server)
            .serve("127.0.0.1:8081".parse().unwrap());

        info!("Worker gRPC listening thread up");

        if let Err(e) = server.await {
            error!("{}", e);
        }
    });

    while let Some(e) = receiver.recv().await {
        match e {
            (_, Event::Register(channel, addr)) => {
                worker_id += 1;
                let worker = Arc::new(Worker {
                    id: worker_id,
                    channel,
                    addr
                });
                worker.channel.send(Event::RegisterSuccessful(worker.clone())).await?;
                info!("Worker with ID {} is now registered, ip: {}", worker.id, worker.addr);

                worker.channel.send(Event::Schedule(Workload {
                    name: String::from("testing"),
                    tag: String::from("0.5.1"),
                    image: String::from("dind"),
                })).await?;
                manager.workers.push(worker);
            },
            kind => info!("Received event : {:#?}", kind),
        }
    };

    Ok(())
}
