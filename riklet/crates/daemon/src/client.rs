use futures_util::{stream, Stream};
use log::{debug, error, info, warn, LevelFilter};
use proto::common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkerStatus, Workload};
use proto::worker as Worker;
use proto::worker::worker_client::WorkerClient;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::error::Error;
use tonic::{transport::Channel, Request, Status, Streaming, Response};
use tokio_stream::wrappers::ReceiverStream;

pub async fn connect() -> Result<WorkerClient<Channel>, Box<dyn Error>> {
    let client = WorkerClient::connect("http://127.0.0.1:4995").await?;
    Ok(client)
}

pub async fn register(client: &mut WorkerClient<Channel>) -> Streaming<Workload> {
    // sending request and waiting for workload
    info!("Worker registration...");
    let request = Request::new({});
    client.register(request).await.unwrap().into_inner()
}

pub async fn waiting_workloads(stream: &mut Streaming<Workload>) ->  Result<(), Box<dyn Error>> {
    info!("Waiting for workloads...");
    while let Some(workload) = stream.message().await? {
        info!("{:?}", workload);
    }
    Ok(())
}

pub async fn send_status_updates(client: &mut WorkerClient<Channel>, status: Vec<WorkerStatus>) -> Result<(), Box<dyn Error>> {
    // creating a new Request
    let request = Request::new(stream::iter(status));

    info!("Send status updates...");

    // sending request and waiting for response
    match client.send_status_updates(request).await {
        Ok(response) => info!("{:?}", response.into_inner()),
        Err(e) => error!("something went wrong: {:?}", e)
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // logs
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("client", LevelFilter::Info)
        .with_module_level("client", LevelFilter::Debug)
        .with_module_level("client", LevelFilter::Error)
        .init()
        .unwrap();

    // worker connection
    let mut client = connect().await?;
    let mut client2 = connect().await?;
    let mut stream = register(&mut client).await;

    let status = vec![
        WorkerStatus {
            status: Some(proto::common::worker_status::Status::Instance(
                InstanceMetric {
                    status: 2,
                    metrics: "{}".to_string(),
                },
            )),
        },
        WorkerStatus {
            status: Some(proto::common::worker_status::Status::Worker(WorkerMetric {
                status: 2,
                metrics: "{}".to_string(),
            })),
        },
    ];

    // send status updates
    tokio::spawn(async move {
        send_status_updates(&mut client2, status).await;
    });

    waiting_workloads(&mut stream).await;

    Ok(())
}
