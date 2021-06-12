use crate::tools::Logger;

pub struct InternalAPI {
    logger: &Logger,
}

impl InternalAPI {
    pub fn new(logger: &Logger) -> InternalAPI {
        InternalAPI { logger }
    }

    pub fn run(&self) {
        self.logger.log("Server running...");
    }
}
