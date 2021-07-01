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
use crate::emitters::metrics_emitter::MetricsEmitter;
use crate::traits::EventEmitter;
use std::time::Duration;


pub struct Riklet {
    client: WorkerClient<Channel>,
    stream: Streaming<Workload>
}

impl Riklet {

    /// Connect to the Scheduler gRPC API
    pub async fn bootstrap(addr: String) -> Result<Self, Box<dyn Error>> {
        let mut client = WorkerClient::connect(addr).await?;
        debug!("gRPC WorkerClient connected.");

        debug!("Worker registration...");
        let request = Request::new({});
        let stream = client.register(request).await.unwrap().into_inner();

        Ok(Self {
            client,
            stream
        })
    }

    pub async fn accept(&mut self) ->  Result<(), Box<dyn Error>> {
        //let client = self.client.clone();

        /*tokio::spawn(async move {
            loop {
                MetricsEmitter::emit_event(client.clone(), vec![
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
                ]).await;
                tokio::time::sleep(Duration::from_millis(15000)).await;
            }
        });*/

        info!("Awaiting workloads");
        while let Some(workload) = &self.stream.message().await? {
            info!("{:?}", workload);
        }
        Ok(())
    }
}