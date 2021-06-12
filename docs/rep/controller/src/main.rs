mod api;
mod tools;

use std::thread;
use std::sync::mpsc::channel;

use api::{external::ExternalAPI, internal::InternalAPI};
use tools::Logger;

struct Gate {
    logger: Sender,
    
}

fn main() {
    let (logging_sender, logging_receiver) = channel();
    
    let logger = Logger::new(logging_receiver, String::from("Main"));
    let internal_api = InternalAPI::new(logging_sender);
    let external_api = ExternalAPI::new(logging_sender);
    
    let mut threads = Vec::new();

    threads.push(thread::spawn(move || {
        internal_api.run();
    }))
    threads.push(thread::spawn(move || {
        external_api.run();
    }))
    threads.push(thread::spawn(move || {
        logger.run();
    }))

    for thread in threads {
        thread.join().unwrap();
    }
}
