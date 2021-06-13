mod api;
mod tools;

use std::sync::mpsc::{channel, Sender};
use std::thread;

use api::{external::ExternalAPI, internal::InternalAPI};
use tools::Logger;

fn main() {
    let (logging_sender, logging_receiver) = channel::<String>();
    let (internal_sender, internal_receiver) = channel::<String>();
    let (external_sender, external_receiver) = channel::<String>();

    let logger = Logger::new(logging_receiver, String::from("Main"));

    let internal_api = InternalAPI::new(logging_sender.clone());
    let external_api = ExternalAPI::new(logging_sender.clone());
    let mut threads = Vec::new();

    // internal_api.external_api_mut() = Option::from(external_sender);

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
