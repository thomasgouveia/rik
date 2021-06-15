use crate::tools::{LogType, Logging};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use tiny_http::{Request, Server};

pub struct ExternalAPI {
    logger: Sender<Logging>,
    internal_sender: Sender<String>,
    external_receiver: Receiver<String>,
}

impl ExternalAPI {
    pub fn new(
        logger_sender: Sender<Logging>,
        internal_sender: Sender<String>,
        external_receiver: Receiver<String>,
    ) -> ExternalAPI {
        ExternalAPI {
            logger: logger_sender,
            internal_sender,
            external_receiver,
        }
    }

    pub fn run(&self) {
        self.logger
            .send(Logging {
                message: String::from("Server running..."),
                log_type: LogType::Log,
            })
            .unwrap();
        self.internal_sender
            .send(String::from("From External to Internal"))
            .unwrap();
        self.run_server();
        self.listen_notification();
    }

    fn listen_notification(&self) {
        for notification in &self.external_receiver {
            println!("{}", notification);
        }
    }

    fn run_server(&self) {
        let server = Server::http("127.0.0.1:5000").unwrap();
        let server = Arc::new(server);
        let mut guards = Vec::with_capacity(4);

        for _ in 0..4 {
            let server = server.clone();

            let guard = thread::spawn(move || loop {
                let rq: Request = server.recv().unwrap();
                println!("{}", rq.method());
            });

            guards.push(guard);
        }
    }
}
