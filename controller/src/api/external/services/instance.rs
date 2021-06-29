use crate::api::{ApiChannel, CRUD};
use crate::database::RickRepository;
use names::Generator;
use rusqlite::Connection;
use std::sync::mpsc::Sender;

pub fn send_create_instance(
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
    workload_id: usize,
    name: &Option<String>,
) {
    let mut random_name_generator = Generator::default();
    let random_name = random_name_generator.next().unwrap();
    let instance_name = match name {
        Some(name) => name,
        None => &random_name,
    };

    RickRepository::insert(connection, instance_name, "").unwrap();
    let instance_id = connection.last_insert_rowid();

    internal_sender
        .send(ApiChannel {
            action: CRUD::Create,
            workload_id: Some(workload_id),
            instance_id: Some(instance_id as usize),
        })
        .unwrap();
}
