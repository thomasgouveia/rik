use futures_util::stream;
use log::{debug, error, info, warn, LevelFilter};
use proto::common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkerStatus};
use proto::worker as Worker;
use proto::worker::worker_client::WorkerClient;
use simple_logger::SimpleLogger;
use tonic::{transport::Channel, Request, Status, Streaming};

#[derive(Clone)]
struct RikletClient {
    client: WorkerClient<Channel>,
}

impl RikletClient {
    pub async fn connect() -> Result<Self, tonic::transport::Error> {
        let client = WorkerClient::connect("http://127.0.0.1:8081").await?;
        Ok(Self { client })
    }

    pub async fn register(&mut self) -> Result<(), Status> {
        // creating a new Request
        let request = Request::new({});

        // sending request and waiting for response
        let mut stream = self.client.register(request).await?.into_inner();
        info!("Waiting for workload");
        while let res = stream.message().await {
            info!("{:?}", res);
        }
        Ok(())
    }

    pub async fn send_status_updates(&mut self, status: Vec<WorkerStatus>) -> Result<(), Status> {
        // creating a new Request
        let request = Request::new(stream::iter(status.clone()));
        // sending request and waiting for response
        let response = self.client.send_status_updates(request).await?;
        info!("RESPONSE={:?}", response);
        Ok(())
    }
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

    // worker registration
    let mut client = RikletClient::connect().await?;
    let mut register_client = client.clone();
    tokio::spawn(async move {
        register_client.register().await;
    });

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
    client.send_status_updates(status).await?;

    Ok(())
}
