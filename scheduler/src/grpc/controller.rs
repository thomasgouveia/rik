use crate::grpc::GRPCService;
use log::info;
use proto::common::{WorkerStatus, Workload};
use proto::controller::controller_server::Controller as ControllerClient;
use rik_scheduler::Event;
use rik_scheduler::Send;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl ControllerClient for GRPCService {
    async fn schedule_instance(&self, _request: Request<Workload>) -> Result<Response<()>, Status> {
        self.send(Event::ScheduleRequest(_request.get_ref().clone()))
            .await?;

        Ok(Response::new(()))
    }

    async fn unschedule_instance(&self, _request: Request<Workload>) -> Result<Response<()>, Status> {
        info!("Received unscheduling order");

        Ok(Response::new(()))
    }

    type GetStatusUpdatesStream = ReceiverStream<Result<WorkerStatus, Status>>;

    async fn get_status_updates(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::GetStatusUpdatesStream>, Status> {
        let (stream_tx, stream_rx) = channel::<Result<WorkerStatus, Status>>(1024);
        let addr = _request
            .remote_addr()
            .unwrap_or_else(|| "0.0.0.0:000".parse().unwrap());
        self.send(Event::Subscribe(stream_tx, addr)).await?;

        Ok(Response::new(ReceiverStream::new(stream_rx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::sync::mpsc::error::SendError;
    use tonic::{Code, Request};

    #[tokio::test]
    async fn test_schedule_event() -> Result<(), Status> {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);
        let workload = Workload {
            instance_id: "uuid".to_string(),
            definition: "{}".to_string(),
        };

        let mock_request = Request::new(workload.clone());

        service.schedule_instance(mock_request).await?;

        let message = receiver.recv().await.unwrap();
        match message {
            Event::ScheduleRequest(content) => assert_eq!(workload, content),
            _ => assert!(false),
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_status_update_no_remote() {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);

        let mock_request = Request::new(());

        service.get_status_updates(mock_request).await;

        let message = receiver.recv().await.unwrap();
        match message {
            Event::Subscribe(_, socket) => {
                let default_socket: SocketAddr = "0.0.0.0:0".parse().unwrap();
                assert_eq!(default_socket, socket);
            }
            _ => assert!(false),
        };
    }

    #[tokio::test]
    async fn test_status_update_stream() -> Result<(), SendError<Result<WorkerStatus, Status>>> {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);

        let mock_request = Request::new(());

        let mut stream = service
            .get_status_updates(mock_request)
            .await
            .unwrap()
            .into_inner()
            .into_inner();

        let message = receiver.recv().await.unwrap();
        match message {
            Event::Subscribe(sender, socket) => {
                sender.send(Err(Status::cancelled("Sample"))).await?;
                let rcv = stream.recv().await.unwrap();
                assert!(rcv.is_err());
                assert_eq!(rcv.unwrap_err().code(), Code::Cancelled)
            }
            _ => assert!(false),
        };
        Ok(())
    }
}
