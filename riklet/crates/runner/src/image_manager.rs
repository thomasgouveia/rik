use crate::image::Image;
use std::collections::HashMap;
use crate::oci::OCI;
use command::skopeo::Skopeo;
use log::{info, error};


#[derive(Debug)]
pub struct ImageManager {
    /// Directory where the pulled images will be stored.
    root_directory: String,
    pull_directory: String,
    bundle_directory: String,
    pulled_images: HashMap<u64, Image>,
    oci: OCI,
    skopeo: Skopeo
}

/// Implementation of the skopeo library
/// coupled with umoci in order to pull images compatible with runc.
impl ImageManager {

    /// Create a new Puller
    pub fn new(root_directory: &str) -> Self {
        ImageManager {
            root_directory: String::from(root_directory),
            pull_directory: format!("{}/images", root_directory),
            bundle_directory: format!("{}/bundles", root_directory),
            pulled_images: HashMap::<u64, Image>::new(),
            oci: OCI::new(),
            skopeo: Skopeo::new()
        }
    }

    /// Format the pull directory for skopeo.
    fn format_image_dest(&self, hashed_oci: &String) -> String {
        format!("oci:{}/{}", self.pull_directory, hashed_oci)
    }

    /// Format the image for skopeo with the following format:
    /// docker://<IMAGE>
    fn format_image_src(&self, image: &str) -> String {
        format!("docker://{}", image)
    }

    /// Pull image locally
    pub fn pull_image(&mut self, image_str: &str) -> Option<Image> {

        info!("Pulling image {}", image_str);

        let image = Image::from(image_str);

        let src = &self.format_image_src(&image.oci[..])[..];
        let dst = &self.format_image_dest(&image.get_hashed_oci())[..];

        match self.skopeo.copy(src, dst) {
            true => {
                info!("Successfully pulled image {}", image_str);
                let image_src = format!("{}/{}", self.pull_directory, image.get_hashed_oci());
                let image_dst = format!("{}/{}", self.bundle_directory, image.get_hash());
                self.oci.unpack(&image_src, &image_dst);
                Some(image)
            },
            false => {
                error!("An error occured during the pulling process for {}", image_str);
                None
            }
        }

        // // self.pulled_images.insert(image.get_hash(), image);
        //
        //
        // let image_src = format!("{}/{}", self.pull_directory, image.get_hashed_oci());
        // let image_dst = format!("{}/{}", self.bundle_directory, image.get_hash());
        // self.oci.unpack(&image_src, &image_dst);
    }
}