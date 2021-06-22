use crate::api::{ApiPipe, CRUD};
use crate::logger::{LogType, Logging};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use tiny_http::{Request, Server};

pub struct ExternalAPI {
    logger: Sender<Logging>,
    internal_sender: Sender<ApiPipe>,
    external_receiver: Receiver<ApiPipe>,
}

impl ExternalAPI {
    pub fn new(
        logger_sender: Sender<Logging>,
        internal_sender: Sender<ApiPipe>,
        external_receiver: Receiver<ApiPipe>,
    ) -> ExternalAPI {
        ExternalAPI {
            logger: logger_sender,
            internal_sender,
            external_receiver,
        }
    }

    pub fn run(&self) {
        self.internal_sender
            .send(ApiPipe {
                action: CRUD::Delete,
                workload_id: 1,
            })
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
        let host = String::from("127.0.0.1");
        let port = 5000;
        let server = Server::http(format!("{}:{}", host, port)).unwrap();
        let server = Arc::new(server);
        let mut guards = Vec::with_capacity(4);

        for _ in 0..4 {
            let server = server.clone();

            let guard = thread::spawn(move || loop {
                let rq: Request = server.recv().unwrap();
                println!("{}", rq.method());
                // TODO handle request + router
            });

            guards.push(guard);
        }
        self.logger
            .send(Logging {
                message: format!("Server running on http://{}:{}", host, port),
                log_type: LogType::Log,
            })
            .unwrap();
    }
}
