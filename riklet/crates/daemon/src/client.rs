use log::{debug, error, info, warn, LevelFilter};
use proto::worker::worker_client::WorkerClient;
use proto::{worker as Worker};
use proto::common::{WorkerStatus, InstanceMetric, WorkerMetric, ResourceStatus };
use futures_util::stream;
use simple_logger::SimpleLogger;
use tonic::{transport::Channel, Request, Status, Streaming};

/*enum ResourceStatus {
    UNKNOWN = 0,
    PENDING = 1,
    RUNNING = 2,
    FAILED = 3,
    TERMINATED = 4
}

struct Metric {
    status: ResourceStatus,
    metrics: &'static str
}

struct WorkerStatus {
    instance: Metric,
    worker: Metric
}*/

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
        while let Some(res) = stream.message().await? {
            info!("{:?}", res);
        }
        Ok(())
    }

    pub async fn send_status_updates(&mut self, status: Vec<WorkerStatus>) -> Result<(), Status>  {
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
    client.register().await?;

    let status = vec![
        WorkerStatus {
            status: Some(proto::common::worker_status::Status::Instance(InstanceMetric {
                status: 2,
                metrics: "{}".to_string()
            }))
        },
        WorkerStatus {
            status: Some(proto::common::worker_status::Status::Worker(WorkerMetric {
                status: 2,
                metrics: "{}".to_string()
            }))
        }
    ];

    // send status updates
    client.send_status_updates(status).await?;

    Ok(())
}
