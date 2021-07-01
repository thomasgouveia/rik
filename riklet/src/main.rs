use std::process::{Command};
use std::fs::File;
use simple_logger::SimpleLogger;
use log::LevelFilter;
use futures_util::{stream, Stream};
use proto::common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkerStatus, Workload};
use proto::worker as Worker;
use proto::worker::worker_client::WorkerClient;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tonic::{transport::Channel, Request, Status, Streaming, Response};
use tokio_stream::wrappers::ReceiverStream;
use crate::riklet::Riklet;
use std::sync::Arc;

mod riklet;
mod traits;
mod emitters;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("riklet", LevelFilter::Info)
        .with_module_level("riklet", LevelFilter::Debug)
        .with_module_level("riklet", LevelFilter::Error)
        .init()?;

    let mut riklet = Riklet::bootstrap(String::from("http://127.0.0.1:4995")).await?;

    riklet.accept().await?;

    Ok(())
}
