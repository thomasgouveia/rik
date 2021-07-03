use crate::grpc::GRPCService;
use log::info;
use proto::common::{WorkerRegistration, WorkerStatus};
use proto::worker::worker_server::Worker as WorkerClient;
use rik_scheduler::Send;
use rik_scheduler::{Event, WorkloadChannelType};
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl WorkerClient for GRPCService {
    type RegisterStream = ReceiverStream<WorkloadChannelType>;

    async fn register(
        &self,
        _request: Request<WorkerRegistration>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        // Streaming channel that sends workloads
        let (stream_tx, stream_rx) = channel::<WorkloadChannelType>(1024);
        let addr = _request
            .remote_addr()
            .unwrap_or("0.0.0.0:000".parse().unwrap());
        let body: String = match &_request.get_ref().hostname {
            hostname if hostname.is_empty() => {
                Err(Status::failed_precondition("No hostname specified"))
            }
            hostname => Ok(hostname.clone()),
        }?;
        self.send(Event::Register(stream_tx, addr, body)).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::sync::mpsc::error::SendError;
    use tonic::{Code, Request};

    #[tokio::test]
    async fn test_no_remote_register() {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);
        let hostname = "debian".to_string();

        let mock_request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });

        service.register(mock_request).await;

        let message = receiver.recv().await.unwrap();
        match message {
            Event::Register(_, socket, host) => {
                assert_eq!(hostname, host);
                let default_socket: SocketAddr = "0.0.0.0:0".parse().unwrap();
                assert_eq!(default_socket, socket);
            }
            _ => assert!(false),
        };
    }

    #[tokio::test]
    async fn test_no_hostname() {
        let (sender, _) = channel::<Event>(1024);

        let service = GRPCService::new(sender);

        let mock_request = Request::new(WorkerRegistration {
            hostname: "".to_string(),
        });
        let fallback = service.register(mock_request).await;
        assert!(fallback.is_err());
        assert_eq!(
            fallback.err().unwrap().code(),
            Status::failed_precondition("").code()
        );
    }

    #[tokio::test]
    async fn test_register_event() -> Result<(), Status> {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);
        let hostname = "debian".to_string();

        let mock_request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });

        service.register(mock_request).await?;

        let message = receiver.recv().await.unwrap();
        match message {
            Event::Register(_, _, _) => assert!(true),
            _ => assert!(false),
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_register_stream() -> Result<(), SendError<WorkloadChannelType>> {
        let (sender, mut receiver) = channel::<Event>(1024);

        let service = GRPCService::new(sender);
        let hostname = "debian".to_string();

        let mock_request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });

        let mut stream = service
            .register(mock_request)
            .await
            .unwrap()
            .into_inner()
            .into_inner();

        let message = receiver.recv().await.unwrap();
        match message {
            Event::Register(sender, _, _) => {
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
