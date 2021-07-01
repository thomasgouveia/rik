use tokio::sync::mpsc::channel;
use rik_scheduler::Event;
use tonic::{Status, Request, Response};
use log::info;
use rik_scheduler::{Send};
use crate::grpc::GRPCService;
use tokio_stream::wrappers::ReceiverStream;
use proto::controller::controller_server::{
    Controller as ControllerClient
};
use proto::common::{WorkerStatus, Workload};

#[tonic::async_trait]
impl ControllerClient for GRPCService {
    async fn schedule_instance(&self, _request: Request<Workload>) -> Result<Response<()>, Status> {
        self.send(Event::ScheduleRequest(_request.get_ref().clone())).await?;

        Ok(Response::new(()))
    }

    type GetStatusUpdatesStream = ReceiverStream<Result<WorkerStatus, Status>>;

    async fn get_status_updates(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::GetStatusUpdatesStream>, Status> {
        let (stream_tx, stream_rx) = channel::<Result<WorkerStatus, Status>>(1024);
        let addr = _request.remote_addr().expect("No remote address found");
        self.send(Event::Subscribe(stream_tx, addr)).await?;

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }
}