use proto::worker::worker_client::WorkerClient;
use proto::common::WorkerStatus;
use crate::traits::EventEmitter;
use tonic::transport::Channel;
use std::error::Error;
use tonic::Request;
use futures_util::stream;

pub struct MetricsEmitter;

#[async_trait::async_trait]
impl EventEmitter<Vec<WorkerStatus>, WorkerClient<Channel>> for MetricsEmitter {

    async fn emit_event(mut client: WorkerClient<Channel>, event: Vec<WorkerStatus>) -> std::result::Result<(), Box<dyn Error>> {
        // creating a new Request
        let request = Request::new(stream::iter(event));

        log::info!("Update metrics");

        // sending request and waiting for response
        match client.send_status_updates(request).await {
            Ok(response) => response.into_inner(),
            Err(e) => log::error!("something went wrong: {:?}", e)
        };

        Ok(())
    }
}