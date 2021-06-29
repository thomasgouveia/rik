use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::sync::mpsc::Sender;

use crate::api;
use crate::api::external::services::instance::send_create_instance;
use crate::api::types::instance::InstanceDefinition;
use crate::api::ApiChannel;
use crate::database::RickRepository;
use crate::logger::{LogType, LoggingChannel};

pub fn get(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    _: &Connection,
    _: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    logger
        .send(LoggingChannel {
            message: String::from("Method not implemented"),
            log_type: LogType::Warn,
        })
        .unwrap();
    Ok(tiny_http::Response::from_string("").with_status_code(tiny_http::StatusCode::from(204)))
}

pub fn create(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    let mut content = String::new();
    req.as_reader().read_to_string(&mut content).unwrap();

    let mut instance: InstanceDefinition = serde_json::from_str(&content)?;

    //Workload not found
    if let Err(_) = RickRepository::find_one(connection, instance.workload_id, "/workload") {
        logger
            .send(LoggingChannel {
                message: String::from(format!("Workload id {} not found", instance.workload_id)),
                log_type: LogType::Warn,
            })
            .unwrap();
        return Ok(tiny_http::Response::from_string(format!(
            "Workload id {} not found",
            instance.workload_id
        ))
        .with_status_code(tiny_http::StatusCode::from(404)));
    }

    if !instance.name.is_none() {
        // Check name is not used
        if let Ok(_) = RickRepository::check_duplicate_name(
            connection,
            &format!("/workload/default/{}", instance.get_name()),
        ) {
            logger
                .send(LoggingChannel {
                    message: String::from("Name already used"),
                    log_type: LogType::Warn,
                })
                .unwrap();
            return Ok(tiny_http::Response::from_string("Name already used")
                .with_status_code(tiny_http::StatusCode::from(404)));
        }

        // Name cannot be used with multiple replicas
        if instance.get_replicas() > 1 {
            return Ok(
                tiny_http::Response::from_string("Cannot use name with multiple replicas")
                    .with_status_code(tiny_http::StatusCode::from(400)),
            );
        }
    }

    for _ in 0..instance.get_replicas() {
        send_create_instance(
            connection,
            internal_sender,
            instance.workload_id,
            &instance.name,
        );
    }

    Ok(tiny_http::Response::from_string("").with_status_code(tiny_http::StatusCode::from(201)))
}

pub fn delete(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    _: &Connection,
    _: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    logger
        .send(LoggingChannel {
            message: String::from("Method not implemented"),
            log_type: LogType::Warn,
        })
        .unwrap();
    Ok(tiny_http::Response::from_string("").with_status_code(tiny_http::StatusCode::from(204)))
}
