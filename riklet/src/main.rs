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

    // The path should be set by the top level riklet module, this is just for test purposes.
    // let mut im = ImageManager::new(config.manager)?;

    // Instanciate our container runtime
    // let runc = Runc::new(config.runner)?;

    // Pull image from docker hub
    // let image = im.pull("busybox:latest").await?;

    /*let socket_path = PathBuf::from(String::from(format!("/tmp/{}.sock", image.name)));
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

    debug!("Containers : {:?}", containers);*/

    riklet.accept().await?;

    Ok(())
}

