use tonic::{transport::Server, Request, Response, Status};
use common::{ResourceStatus, WorkerMetric, InstanceMetric, WorkerStatus, Workload};
use worker::worker_server::{Worker, WorkerServer};
use protobuf::well_known_types::Empty;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::thread::sleep;

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

#[derive(Debug)]
pub struct WorkerService;

#[tonic::async_trait]
impl Worker for WorkerService {
    type RegisterStream = ReceiverStream<Result<Workload, Status>>;

    async fn register(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1024);
        tokio::spawn(async move {
            for feature in 1..100 {
                tx.send(Ok(Workload {
                    name: String::from(format!("{} test", feature)),
                    tag: String::from(format!("{} test", feature)),
                    image: String::from(format!("{} test", feature)),
                })).await.unwrap();
                sleep(Duration::new(1,10));
                println!("Sending now");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn send_status_updates(
        &self,
        _request: Request<tonic::Streaming<WorkerStatus>>,
    ) -> Result<Response<()>, Status> {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let service = WorkerService {};
    let server = WorkerServer::new(service);

    Server::builder()
        .add_service(server)
        .serve("127.0.0.1:8081".parse().unwrap())
        .await?;

    Ok(())
}
