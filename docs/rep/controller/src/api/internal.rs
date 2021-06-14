use crate::tools::{LogType, Logging};
use std::sync::mpsc::{Receiver, Sender};

pub struct InternalAPI {
    logger: Sender<Logging>,
    external_sender: Sender<String>,
    internal_receiver: Receiver<String>,
}

impl InternalAPI {
    pub fn new(
        logger_sender: Sender<Logging>,
        external_sender: Sender<String>,
        internal_receiver: Receiver<String>,
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
                message: String::from("Server running..."),
                log_type: LogType::Log,
            })
            .unwrap();
        self.external_sender
            .send(String::from("From Internal to External"))
            .unwrap();
        self.listen();
    }

    fn listen(&self) {
        for notification in &self.internal_receiver {
            println!("{}", notification);
        }
    }
}
