use crate::api::{ApiChannel, CRUD};
use crate::database::RikDataBase;
use crate::database::RikRepository;
use crate::logger::{LogType, LoggingChannel};
use dotenv::dotenv;
use proto::common::Workload;
use proto::controller::controller_client::ControllerClient;
use rusqlite::Connection;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tonic;

#[derive(Clone)]
struct RikControllerClient {
    client: ControllerClient<tonic::transport::Channel>,
}

#[allow(dead_code)]
impl RikControllerClient {
    pub async fn connect() -> Result<RikControllerClient, tonic::transport::Error> {
        dotenv().ok();
        let scheduler_url = match std::env::var("SCHEDULER_URL") {
            Ok(val) => val,
            Err(_e) => "http://127.0.0.1:4996".to_string(),
        };
        let client = ControllerClient::connect(scheduler_url).await?;
        Ok(RikControllerClient { client })
    }

    pub async fn schedule_instance(&mut self, instance: Workload) -> Result<(), tonic::Status> {
        let request = tonic::Request::new(instance);
        self.client.schedule_instance(request).await?;
        Ok(())
    }

    pub async fn get_status_updates(
        &mut self,
        database: Arc<RikDataBase>,
    ) -> Result<(), tonic::Status> {
        let connection: Connection = database.open().unwrap();
        let request = tonic::Request::new(());

        let mut stream = self.client.get_status_updates(request).await?.into_inner();
        while let Some(status) = stream.message().await? {
            println!("Instance Status {:?}", status);
            RikRepository::update(&connection, 1).unwrap();
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

    pub async fn run(&self, database: Arc<RikDataBase>) {
        let client: RikControllerClient = RikControllerClient::connect().await.unwrap();

        let mut client_clone = client.clone();
        let database = database.clone();

        tokio::spawn(async move {
            client_clone.get_status_updates(database).await.unwrap();
        });

        self.listen_notification(client).await;
    }

    async fn listen_notification(&self, mut client: RikControllerClient) {
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
                                    instance_id: instance_id as u32,
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
