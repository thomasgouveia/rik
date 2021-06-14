mod api;
mod tools;

use std::sync::mpsc::channel;
use std::thread;

use api::{external::ExternalAPI, internal::InternalAPI};
use tools::{Logger, Logging};

fn main() {
    let (logging_sender, logging_receiver) = channel::<Logging>();
    let (internal_sender, internal_receiver) = channel::<String>();
    let (external_sender, external_receiver) = channel::<String>();

    let logger = Logger::new(logging_receiver, String::from("Main"));

    let internal_api = InternalAPI::new(
        logging_sender.clone(),
        external_sender.clone(),
        internal_receiver,
    );
    let external_api = ExternalAPI::new(
        logging_sender.clone(),
        internal_sender.clone(),
        external_receiver,
    );
    let mut threads = Vec::new();

    threads.push(thread::spawn(move || {
        internal_api.run();
    }));
    threads.push(thread::spawn(move || {
        external_api.run();
    }));
    threads.push(thread::spawn(move || {
        logger.run();
    }));

    for thread in threads {
        thread.join().unwrap();
    }
}
