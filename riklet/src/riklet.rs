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
use crate::config::Configuration;
use oci::image_manager::ImageManager;
use cri::container::{Runc, CreateArgs};
use crate::structs::{WorkloadDefinition, Container};
use std::path::PathBuf;
use uuid::Uuid;
use cri::console::ConsoleSocket;
use shared::utils::get_random_hash;
use std::sync::Arc;


#[derive(Debug)]
pub struct Riklet {
    config: Configuration,
    client: WorkerClient<Channel>,
    stream: Streaming<Workload>,
    image_manager: ImageManager,
    container_runtime: Runc,
}

impl Riklet {

    /// Connect to the Scheduler gRPC API
    pub async fn bootstrap() -> Result<Self, Box<dyn Error>> {

        let config = Configuration::load(None)?;
        config.bootstrap()?;

        let mut client = WorkerClient::connect(config.scheduler.clone()).await?;
        debug!("gRPC WorkerClient connected.");

        let request = Request::new({});
        let stream = client.register(request).await.unwrap().into_inner();

        info!("Registration success");

        let container_runtime = Runc::new(config.runner.clone())?;
        let image_manager = ImageManager::new(config.manager.clone())?;

        Ok(Self {
            container_runtime,
            image_manager,
            config,
            client,
            stream
        })
    }

    pub async fn handle_workload(&mut self, workload: WorkloadDefinition) -> Result<(), Box<dyn Error>> {
        let workload_uuid = workload.get_uuid();
        for container in &workload.spec.containers {
            let container_uuid = container.get_uuid();
            let image = &self
                .image_manager
                .pull(&container.image[..])
                .await.unwrap();

            let socket_path = PathBuf::from(
                format!("/tmp/{}-{}", container.name, container_uuid));

            let console_socket = ConsoleSocket::new(&socket_path).unwrap();
            tokio::spawn(async move {
                match console_socket.get_listener().as_ref().unwrap().accept().await {
                    Ok((stream, _socket_addr)) => {
                        Box::leak(Box::new(stream));
                    },
                    Err(err) => {
                        error!("Receive PTY master error : {:?}", err)
                    }
                }
            });

            let id = format!("{}-{}-{}-{}", workload.name, container.name, uuid, container_uuid);

            &self.container_runtime.run(&id[..], &image.bundle.as_ref().unwrap(), Some(&CreateArgs {
                pid_file: None,
                console_socket: Some(socket_path),
                no_pivot: false,
                no_new_keyring: false,
                detach: true
            })).await.unwrap();
        }

        Ok(())
    }

    pub async fn accept(&mut self) ->  Result<(), Box<dyn Error>> {
        let client = self.client.clone();

        tokio::spawn(async move {
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
        });

        info!("Riklet is ready to accept connections.");
        while let Some(workload) = &self.stream.message().await? {
            let workload_definition: WorkloadDefinition = serde_json::from_str(&workload.definition[..]).unwrap();
            &self.handle_workload(workload_definition).await;
        }
        Ok(())
    }
}