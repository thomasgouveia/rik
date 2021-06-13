use std::sync::mpsc::Sender;

pub struct InternalAPI {
    logger: Sender<String>,
    external_api: Option<Sender<String>>,
}

impl InternalAPI {
    pub fn new(logger_sender: Sender<String>) -> InternalAPI {
        InternalAPI {
            logger: logger_sender,
            external_api: None,
        }
    }

    pub fn external_api_mut(&mut self) -> &mut Option<Sender<String>> {
        &mut self.external_api
    }

    pub fn run(&self) {
        self.logger.send(String::from("Server running...")).unwrap();
        // self.logger.log("Server running...");
    }
}
