use std::sync::mpsc::Sender;

pub struct ExternalAPI {
    logger: Sender<String>,
    pub internal_api: Option<Sender<String>>,
}

impl ExternalAPI {
    pub fn new(logger_sender: Sender<String>) -> ExternalAPI {
        ExternalAPI {
            logger: logger_sender,
            internal_api: None,
        }
    }

    pub fn run(&self) {
        self.logger.send(String::from("Server running...")).unwrap();
        // self.logger.log("Server running...");
    }
}
