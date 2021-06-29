use crate::api::{ApiChannel, CRUD};
use crate::logger::{LogType, LoggingChannel};
use std::sync::mpsc::{Receiver, Sender};

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

    pub fn run(&self) {
        self.logger
            .send(LoggingChannel {
                message: String::from("Internal server not implemented"),
                log_type: LogType::Error,
            })
            .unwrap();
        self.listen();
    }

    fn listen(&self) {
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
