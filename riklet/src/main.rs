use simple_logger::SimpleLogger;
use log::{error, LevelFilter, debug, info};
use std::process::exit;
use std::path::{PathBuf};

use cri::container::{Runc, CreateArgs};
use cri::console::ConsoleSocket;
use oci::image_manager::ImageManager;
use oci::umoci::UmociConfiguration;
use oci::skopeo::SkopeoConfiguration;
use std::time::Duration;
use snafu::Snafu;

mod config;
mod constants;

use config::Configuration;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().with_module_level("mio", LevelFilter::Info).init()?;

    info!("Riklet v{}", VERSION);

    let config = Configuration::load(None)?;

    config.bootstrap();

    // The path should be set by the top level riklet module, this is just for test purposes.
    let mut im = ImageManager::new(config.manager)?;

    // Instanciate our container runtime
    let runc = Runc::new(config.runner)?;

    // Pull image from docker hub
    let image = im.pull("busybox:latest").await?;

    let socket_path = PathBuf::from(String::from(format!("/tmp/{}.sock", image.name)));
    let console_socket = ConsoleSocket::new(&socket_path).unwrap();

    tokio::spawn(async move {
        match console_socket.get_listener().as_ref().unwrap().accept().await {
            Ok((stream, _socket_addr)) => {
                Box::leak(Box::new(stream));
            },
            Err(err) => {
                error!("Receive PTY master error : {:?}", err)
            }
        }
    });

    runc.run(&image.name, &image.bundle.unwrap(), Some(&CreateArgs {
        pid_file: None,
        console_socket: Some(socket_path),
        no_pivot: false,
        no_new_keyring: false,
        detach: true
    })).await?;

    info!("Riklet initialized.");

    let containers = runc.list().await?;

    debug!("Containers : {:?}", containers);

    loop {}
}

