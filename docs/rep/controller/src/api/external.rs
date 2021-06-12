use crate::tools::Logger;

pub struct ExternalAPI {
    logger: Logger,
}

impl ExternalAPI {
    pub fn new(logger: Logger) -> ExternalAPI {
        ExternalAPI { logger }
    }

    pub fn run(&self) {
        self.logger.log("Server running...");
    }
}
