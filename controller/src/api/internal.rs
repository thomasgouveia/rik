use crate::api::{ApiPipe, CRUD};
use crate::logger::{LogType, Logging};
use std::sync::mpsc::{Receiver, Sender};

#[allow(dead_code)]
pub struct InternalAPI {
    logger: Sender<Logging>,
    external_sender: Sender<ApiPipe>,
    internal_receiver: Receiver<ApiPipe>,
}

impl InternalAPI {
    pub fn new(
        logger_sender: Sender<Logging>,
        external_sender: Sender<ApiPipe>,
        internal_receiver: Receiver<ApiPipe>,
    ) -> InternalAPI {
        InternalAPI {
            logger: logger_sender,
            external_sender,
            internal_receiver,
        }
    }

    pub fn run(&self) {
        self.logger
            .send(Logging {
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
                    // Create workload
                    // Send workload to sheduler
                    self.logger
                        .send(Logging {
                            message: format!("Create Workload: {}", notification.workload_id),
                            log_type: LogType::Log,
                        })
                        .unwrap();
                }
                CRUD::Delete => {
                    // Delete workload
                    // Send instruction to sheduler
                    self.logger
                        .send(Logging {
                            message: format!("Delte Workload: {}", notification.workload_id),
                            log_type: LogType::Log,
                        })
                        .unwrap();
                }
            }
        }
    }
}
