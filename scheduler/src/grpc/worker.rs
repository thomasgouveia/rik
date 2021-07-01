use tokio::sync::mpsc::channel;
use rik_scheduler::{Event, WorkloadChannelType};
use tonic::{Status, Request, Response};
use log::{error, info};
use rik_scheduler::{Send};
use crate::grpc::GRPCService;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use proto::worker::worker_server::{Worker as WorkerClient};
use proto::common::{WorkerStatus};

#[tonic::async_trait]
impl WorkerClient for GRPCService {
    type RegisterStream = ReceiverStream<WorkloadChannelType>;

    async fn register(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        // Streaming channel that sends workloads
        let (stream_tx, stream_rx) = channel::<WorkloadChannelType>(1024);
        let addr = _request.remote_addr().expect("No remote address found");
        self.send(Event::Register(stream_tx, addr)).await?;

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