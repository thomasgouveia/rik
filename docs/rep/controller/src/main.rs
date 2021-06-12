use std::thread;

mod api;
mod tools;

use api::{external::ExternalAPI, internal::InternalAPI};
use tools::Logger;

fn main() {
    let logger = Logger::new(String::from("Main"));
    let internal_api = InternalAPI::new(&logger);
    let external_api = ExternalAPI::new(&logger);

    thread::spawn(move || {
        internal_api.run();
    })
    .join()
    .unwrap();
    thread::spawn(move || {
        external_api.run();
    })
    .join()
    .unwrap();
}
