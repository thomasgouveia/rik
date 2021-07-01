use simple_logger::SimpleLogger;
use log::LevelFilter;
use crate::riklet::Riklet;

mod structs;
mod riklet;
mod traits;
mod emitters;

mod config;
mod constants;

use config::Configuration;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    SimpleLogger::new()
        .with_module_level("h2", LevelFilter::Off)
        .with_module_level("tracing", LevelFilter::Off)
        .with_module_level("tokio_util", LevelFilter::Off)
        .with_module_level("tonic", LevelFilter::Off)
        .with_module_level("tokio", LevelFilter::Off)
        .with_module_level("hyper", LevelFilter::Off)
        .with_module_level("want", LevelFilter::Off)
        .with_module_level("mio", LevelFilter::Off)
        .with_module_level("tower", LevelFilter::Off)
        .init()?;

    let mut riklet = Riklet::bootstrap().await?;

    log::info!("Riklet v{}", VERSION);

    riklet.accept().await?;

    Ok(())
}

