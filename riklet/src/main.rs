use runner::image_manager::ImageManager;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();
    // The path should be set by the top level riklet module, this is just for test purposes.
    let mut im = ImageManager::new("/tmp/rik/riklet");
    let _image = im.pull_image("httpd:latest");
}
