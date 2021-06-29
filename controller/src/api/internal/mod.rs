use crate::api::{ApiChannel, CRUD};
use crate::database::RickDataBase;
use crate::logger::{LogType, LoggingChannel};
use proto::common::Workload;
use proto::controller::controller_client::ControllerClient;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tonic;

struct RickControllerClient {
    client: ControllerClient<tonic::transport::Channel>,
}

#[allow(dead_code)]
impl RickControllerClient {
    pub async fn connect() -> Result<Self, tonic::transport::Error> {
        let client = ControllerClient::connect("http://127.0.0.1:10000").await?;
        Ok(Self { client })
    }

    pub async fn schedule_instance(&mut self, instance: Workload) -> Result<(), tonic::Status> {
        let request = tonic::Request::new(instance);
        self.client.schedule_instance(request).await?;
        Ok(())
    }

    pub async fn get_status_updates(&mut self) -> Result<(), tonic::Status> {
        let request = tonic::Request::new(());

        let mut stream = self.client.get_status_updates(request).await?.into_inner();
        while let Some(status) = stream.message().await? {
            println!("Instance Status {:?}", status);
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct Server {
    logger: Sender<LoggingChannel>,
    external_sender: Sender<ApiChannel>,
    internal_receiver: Receiver<ApiChannel>,
}

impl Server {
    pub fn new(
        logger_sender: Sender<LoggingChannel>,
        external_sender: Sender<ApiChannel>,
        internal_receiver: Receiver<ApiChannel>,
    ) -> Server {
        Server {
            logger: logger_sender,
            external_sender,
            internal_receiver,
        }
    }

    pub async fn run(&self, _database: Arc<RickDataBase>) {
        let client: RickControllerClient = RickControllerClient::connect().await.unwrap();
        self.listen_notification(client).await;
    }

    async fn listen_notification(&self, mut client: RickControllerClient) {
        for notification in &self.internal_receiver {
            match notification.action {
                CRUD::Create => {
                    // Create instance
                    // Send workload to sheduler
                    self.logger
                        .send(LoggingChannel {
                            message: format!(
                                "Ctrl to scheduler create instance: {:?}, workload_id : {:?}",
                                notification.instance_id, notification.workload_id
                            ),
                            log_type: LogType::Log,
                        })
                        .unwrap();
                    if let Some(instance_id) = notification.instance_id {
                        if let Some(workload_definition) = notification.workload_definition {
                            client
                                .schedule_instance(Workload {
                                    instance_id: instance_id.to_string(),
                                    definition: serde_json::to_string(&workload_definition)
                                        .unwrap(),
                                })
                                .await
                                .unwrap();
                        }
                    }
                }
                CRUD::Delete => {
                    // Delete instance
                    // Send instruction to sheduler
                    self.logger
                        .send(LoggingChannel {
                            message: format!(
                                "Ctrl to scheduler delete instance: {:?}",
                                notification.instance_id
                            ),
                            log_type: LogType::Log,
                        })
                        .unwrap();
                }
            }
        }
    }
}
