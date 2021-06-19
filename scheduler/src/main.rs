use tonic::{transport::Server, Request, Response, Status, Code};
use rik_scheduler::common::{WorkerStatus, Workload};
use rik_scheduler::worker::worker_server::{WorkerServer, Worker as WorkerClient};
use rik_scheduler::{Event, SchedulerError, WorkloadChannelType};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use log::{info, error};
use env_logger::Env;
use std::sync::{Arc};
use std::net::SocketAddr;
use tokio_stream::StreamExt;


#[derive(Debug, Clone)]
pub struct WorkerService {
    /// Channel used in order to communicate with the main thread
    /// In the case the worker doesn't know its ID yet, put 0 in the first
    /// item of the tuple
    pub sender: Sender<Event>,
}

#[tonic::async_trait]
impl WorkerClient for WorkerService {
    type RegisterStream = ReceiverStream<WorkloadChannelType>;

    async fn register(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        // Streaming channel that sends workloads
        let (stream_tx, stream_rx) = channel::<WorkloadChannelType>(1024);
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

impl Worker {
    fn new(id: u8, channel: Sender<Result<Workload, Status>>, addr: SocketAddr) -> Worker {
        Worker {
            id,
            channel,
            addr
        }
    }
}

#[derive(Debug)]
pub struct Manager {
    workers: Vec<Worker>,
    channel: Receiver<Event>,
    worker_increment: u8,
}

impl Manager {
    async fn run() -> Result<Manager, Box<dyn std::error::Error>> {
        let (sender, receiver) = channel::<Event>(1024);
        let mut instance = Manager {
            workers: Vec::new(),
            channel: receiver,
            worker_increment: 0,
        };
        instance.run_workers_listener(sender);
        let channel_listener = instance.listen();
        channel_listener.await?;
        Ok(instance)
    }

    fn run_workers_listener(&self, sender: Sender<Event>) {
        let server = WorkerServer::new(WorkerService {
            sender,
        });
        tokio::spawn(async move {
            let server =  Server::builder()
                .add_service(server)
                .serve("127.0.0.1:8081".parse().unwrap());

            info!("Worker gRPC listening thread up");

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
                    match self.workers.len() {
                        0 => self.register(channel.clone(), addr.clone()),
                        _ => channel.send(Err(Status::aborted("Cluster is full"))).await?,
                    }
                },
                kind => info!("Received event : {:#?}", kind),
            }
        };
        Ok(())
    }

    fn register(&mut self, channel: Sender<WorkloadChannelType>, addr: SocketAddr) {
        let worker = Worker::new(*&self.worker_increment, channel, addr);
        info!("Worker with ID {} is now registered, ip: {}", worker.id, worker.addr);
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
