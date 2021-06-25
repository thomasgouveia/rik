use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::sync::mpsc::Sender;

use crate::api;
use crate::api::ApiChannel;
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
