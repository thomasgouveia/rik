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
use tokio_stream::StreamExt;
use tokio::io::AsyncReadExt;

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
        let (stream_tx, stream_rx) = channel::<Result<Workload, tonic::Status>>(1024);
        let addr = _request.remote_addr()
            .expect("No remote address found");

        self.sender.send(Event::Register(stream_tx, addr))
            .await
            .map_err(|_| Status::new(Code::Unavailable, "Worker service cannot process your request"))?;

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

#[derive(Debug)]
pub enum Event {
    Register(Sender<Result<Workload, Status>>, SocketAddr),
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
    let (sender, mut receiver) = channel::<Event>(1024);

    let server = WorkerServer::new(WorkerService {
        sender,
    });
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
            Event::Register(channel, addr) => {
                worker_id += 1;
                let worker = Arc::new(Worker {
                    id: worker_id,
                    channel,
                    addr
                });
                info!("Worker with ID {} is now registered, ip: {}", worker.id, worker.addr);

                worker.channel.send(Ok(Workload {
                    instance_id: String::from("testing"),
                    definition: String::from("{}"),
                })).await?;
                worker.channel.send(Err(Status::aborted("Cannot register now"))).await?;
                manager.workers.push(worker);
            },
            kind => info!("Received event : {:#?}", kind),
        }
    };

    Ok(())
}
