use common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkerStatus, Workload};
use controller::controller_server::{Controller, ControllerServer};
use env_logger::Env;
use log::{error, info};
use protobuf::well_known_types::Empty;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Code, Request, Response, Status};

// Common is needed to be included, as controller & worker
// are using it
pub mod common {
    tonic::include_proto!("common");
}

pub mod controller {
    tonic::include_proto!("controller");
}

#[derive(Debug, Clone)]
pub struct ControllerService {
    workload: Vec<Workload>,
}

#[tonic::async_trait]
impl Controller for ControllerService {
    async fn schedule_instance(&self, _request: Request<Workload>) -> Result<Response<()>, Status> {
        // Save Workload to send it to Workers
        // Or
        // Send it directly to the Worker
        Ok(Response::new(()))
    }

    type GetStatusUpdatesStream = ReceiverStream<Result<WorkerStatus, Status>>;

    async fn get_status_updates(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::GetStatusUpdatesStream>, Status> {
        let (tx, rx) = channel(1024);
        // Define when we send new status of worker or workload instance
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    // info!("Starting up...");
    let addr = "127.0.0.1:8081".parse().unwrap();

    let controller = ControllerService {
        workload: Vec::new(),
    };

    let svc = ControllerServer::new(controller);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
