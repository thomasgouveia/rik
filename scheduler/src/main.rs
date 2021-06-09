use tonic::{transport::Server, Request, Response, Status};
use common::{ResourceStatus, WorkerMetric, InstanceMetric, WorkerStatus, Workload};
use worker::worker_server::{Worker, WorkerServer};
use protobuf::well_known_types::Empty;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::thread::sleep;
use tokio::sync::mpsc::Sender;

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
    sender: Sender<Event>,
}

#[tonic::async_trait]
impl Worker for WorkerService {
    type RegisterStream = ReceiverStream<Result<Workload, Status>>;



    async fn register(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1024);
        self.sender.send(Event {
            kind: EventKind::Register,
        }).await.unwrap();
        let sender = tx.clone();
        tokio::task::spawn(async move {
            for feature in 1..10 {
                sender.send(Ok(Workload {
                    name: String::from(format!("{} test", feature)),
                    tag: String::from(format!("{} test", feature)),
                    image: String::from(format!("{} test", feature)),
                })).await.unwrap_or_else(|e| { println!("Error is {}", e)});
            }
            println!("End of register thread");
        });
        println!("End of register ");
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn send_status_updates(
        &self,
        _request: Request<tonic::Streaming<WorkerStatus>>,
    ) -> Result<Response<()>, Status> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum EventKind {
    Register,
}

#[derive(Debug)]
pub struct Event {
    kind: EventKind
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let (sender, mut receiver) = mpsc::channel::<Event>(1024);
    let service = WorkerService {
        sender,
    };
    let server = WorkerServer::new(service.clone());

    let grpc = tokio::spawn(async move {
        let server =  Server::builder()
            .add_service(server)
            .serve("127.0.0.1:8081".parse().unwrap());

        if let Err(e) = server.await {
            println!("Error {}", e);
        }
        println!("Ended the thread");
    });

     let thread = thread::spawn(move || {
        loop {
            let message = receiver.blocking_recv();
            println!("New event received {:#?}", message.unwrap());
        }
    });
    grpc.await;
    println!("Reaching the end of program");

    Ok(())
}
