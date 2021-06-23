use route_recognizer;
use std::io;

use crate::api;

#[allow(dead_code)]
pub fn get(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
) -> Result<tiny_http::Response<io::Empty>, api::RikError> {
    println!("Get All instances");
    Ok(tiny_http::Response::empty(tiny_http::StatusCode::from(200)))
}

#[allow(dead_code)]
pub fn create(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
) -> Result<tiny_http::Response<io::Empty>, api::RikError> {
    println!("Create instance");
    Ok(tiny_http::Response::empty(tiny_http::StatusCode::from(200)))
}

#[allow(dead_code)]
pub fn delete() {}
