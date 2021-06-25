use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::Sender;

use crate::api;
use crate::api::types::tenant::Tenant;
use crate::api::ApiChannel;
use crate::database::RickTenantRepository;
use crate::logger::{LogType, LoggingChannel};

pub fn get(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    if let Ok(tenants) = RickTenantRepository::find_all_tenant(connection) {
        let tenants_json = serde_json::to_string(&tenants).unwrap();
        logger
            .send(LoggingChannel {
                message: String::from("Cannot find tenant"),
                log_type: LogType::Error,
            })
            .unwrap();
        Ok(tiny_http::Response::from_string(tenants_json)
            .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
            .with_status_code(tiny_http::StatusCode::from(200)))
    } else {
        Ok(tiny_http::Response::from_string("Cannot find tenant")
            .with_status_code(tiny_http::StatusCode::from(500)))
    }
}

pub fn create(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    let mut content = String::new();
    req.as_reader().read_to_string(&mut content).unwrap();
    let tenant: Tenant = serde_json::from_str(&content).unwrap();

    if let Ok(()) = RickTenantRepository::create_tenant(connection, &tenant) {
        logger
            .send(LoggingChannel {
                message: String::from("Create tenant"),
                log_type: LogType::Log,
            })
            .unwrap();
        Ok(tiny_http::Response::from_string(content)
            .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
            .with_status_code(tiny_http::StatusCode::from(200)))
    } else {
        logger
            .send(LoggingChannel {
                message: String::from("Cannot create tenant"),
                log_type: LogType::Error,
            })
            .unwrap();
        Ok(tiny_http::Response::from_string("Cannot create tenant")
            .with_status_code(tiny_http::StatusCode::from(500)))
    }
}

pub fn delete(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    let mut content = String::new();
    req.as_reader().read_to_string(&mut content).unwrap();
    let tenant: Tenant = serde_json::from_str(&content).unwrap();

    if let Ok(tenant) = RickTenantRepository::find_one_tenant(connection, tenant.id) {
        RickTenantRepository::delete_tenants(connection, tenant.id).unwrap();

        logger
            .send(LoggingChannel {
                message: String::from("Delete tenant"),
                log_type: LogType::Log,
            })
            .unwrap();
        Ok(tiny_http::Response::from_string("").with_status_code(tiny_http::StatusCode::from(204)))
    } else {
        logger
            .send(LoggingChannel {
                message: String::from(format!("Tenant id {} not found", tenant.id)),
                log_type: LogType::Error,
            })
            .unwrap();
        Ok(
            tiny_http::Response::from_string(format!("Tenant id {} not found", tenant.id))
                .with_status_code(tiny_http::StatusCode::from(404)),
        )
    }
}
