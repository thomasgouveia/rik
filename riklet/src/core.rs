use proto::common::{InstanceMetric, WorkerMetric, WorkerStatus, Workload, WorkerRegistration};
use proto::worker::worker_client::WorkerClient;
use std::error::Error;
use tonic::{transport::Channel, Request, Streaming};
use crate::emitters::metrics_emitter::MetricsEmitter;
use crate::traits::EventEmitter;
use std::time::Duration;
use crate::config::Configuration;
use oci::image_manager::ImageManager;
use cri::container::{Runc, CreateArgs};
use crate::structs::{WorkloadDefinition};
use std::path::PathBuf;
use cri::console::ConsoleSocket;
use clap::crate_version;


#[derive(Debug)]
pub struct Riklet {
    config: Configuration,
    client: WorkerClient<Channel>,
    stream: Streaming<Workload>,
    image_manager: ImageManager,
    container_runtime: Runc,
}

impl Riklet {

    /// Display a banner
    fn banner() {
        println!(r#"
        ______ _____ _   __ _      _____ _____
        | ___ \_   _| | / /| |    |  ___|_   _|
        | |_/ / | | | |/ / | |    | |__   | |
        |    /  | | |    \ | |    |  __|  | |
        | |\ \ _| |_| |\  \| |____| |___  | |
        \_| \_|\___/\_| \_/\_____/\____/  \_/
        "#);
    }

    /// Bootstrap a Riklet in order to run properly.
    pub async fn bootstrap() -> Result<Self, Box<dyn Error>> {

        // Display the banner, just for fun :D
        Riklet::banner();

        // Load the configuration
        let config = Configuration::load()?;
        // Create directories and files used by Riklet
        config.bootstrap()?;

        // Connect to the master node scheduler
        let mut client = WorkerClient::connect(config.master_ip.clone()).await?;
        log::debug!("gRPC WorkerClient connected.");

        // Register this node to the master
        let request = Request::new(WorkerRegistration {
            hostname: "node".to_string(),
        });
        let stream = client.register(request).await?.into_inner();

        log::trace!("Registration success");

        // Initialize the container runtime
        let container_runtime = Runc::new(config.runner.clone())?;
        // Initialize the image manager
        let image_manager = ImageManager::new(config.manager.clone())?;

        Ok(Self {
            container_runtime,
            image_manager,
            config,
            client,
            stream
        })
    }

    /// Handle a workload (eg CREATE, UPDATE, DELETE, READ)
    pub async fn handle_workload(&mut self, workload: &WorkloadDefinition) -> Result<(), Box<dyn Error>> {
        let workload_uuid = workload.get_uuid();
        for container in workload.get_containers() {
            let container_uuid = container.get_uuid();
            let image = &self
                .image_manager
                .pull(&container.image[..])
                .await?;

            // New console socket for the container
            let socket_path = PathBuf::from(
                format!("/tmp/{}-{}", container.name, container_uuid));
            let console_socket = ConsoleSocket::new(&socket_path)?;

            tokio::spawn(async move {
                match console_socket.get_listener().as_ref().unwrap().accept().await {
                    Ok((stream, _socket_addr)) => {
                        Box::leak(Box::new(stream));
                    },
                    Err(err) => {
                        log::error!("Receive PTY master error : {:?}", err)
                    }
                }
            });

            let id = format!("{}-{}-{}-{}", workload.name, container.name, workload_uuid, container_uuid);
            &self.container_runtime.run(&id[..], &image.bundle.as_ref().unwrap(), Some(&CreateArgs {
                pid_file: None,
                console_socket: Some(socket_path),
                no_pivot: false,
                no_new_keyring: false,
                detach: true
            })).await?;
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
                ]).await.unwrap();
                tokio::time::sleep(Duration::from_millis(15000)).await;
            }
        });

        log::info!("Riklet (v{}) is ready to accept connections.", crate_version!());
        while let Some(workload) = &self.stream.message().await? {
            let workload_definition: WorkloadDefinition = serde_json::from_str(&workload.definition[..]).unwrap();
            &self.handle_workload(&workload_definition).await;
        }
        Ok(())
    }
}