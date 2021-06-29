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

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DEFAULT_COMMAND_TIMEOUT: u64 = 30000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().with_module_level("mio", LevelFilter::Info).init()?;

    info!("Riklet v{}", VERSION);

    let mut skopeo_config: SkopeoConfiguration = Default::default();
    skopeo_config.images_directory = Some(PathBuf::from(String::from("/.riklet/images")));
    skopeo_config.timeout = Some(Duration::from_millis(DEFAULT_COMMAND_TIMEOUT));

    let mut umoci_config: UmociConfiguration = Default::default();
    umoci_config.bundle_directory = Some(PathBuf::from(String::from("/.riklet/bundles")));
    umoci_config.timeout = Some(Duration::from_millis(DEFAULT_COMMAND_TIMEOUT));

    // The path should be set by the top level riklet module, this is just for test purposes.
    let mut im = ImageManager::new(umoci_config, skopeo_config)?;

    // Instanciate our container runtime
    let runc = Runc::new(Default::default())?;

    // Pull image from docker hub
    let image = im.pull("ubuntu:latest").await?;

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

    let _result = runc.run(&image.name, &image.bundle.unwrap(), Some(&CreateArgs {
        pid_file: None,
        console_socket: Some(socket_path),
        no_pivot: false,
        no_new_keyring: false,
        detach: true
    })).await;

    info!("Riklet initialized.");

    let containers = runc.list().await?;

    debug!("Containers : {:?}", containers);

    loop {}
}
