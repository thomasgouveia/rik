use crate::image::Image;
use std::collections::HashMap;
use command::skopeo::Skopeo;
use log::{info, error};
use command::umoci::Umoci;

mod hash;
pub mod image;

#[derive(Debug)]
pub struct ImageManager {
    /// Directory where the pulled images will be stored.
    root_directory: String,
    pull_directory: String,
    bundle_directory: String,
    pulled_images: HashMap<u64, Image>,
    umoci: Option<Umoci>,
    skopeo: Option<Skopeo>
}

/// Implementation of the skopeo library
/// coupled with umoci in order to pull images compatible with cri.
impl ImageManager {

    /// Create a new Puller
    pub fn new(root_directory: &str) -> Self {

        ImageManager {
            root_directory: String::from(root_directory),
            pull_directory: format!("{}/images", root_directory),
            bundle_directory: format!("{}/bundles", root_directory),
            pulled_images: HashMap::<u64, Image>::new(),
            umoci: Umoci::new(),
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
    pub fn pull(&mut self, image_str: &str) -> Option<Image> {

        info!("Pulling image {}", image_str);

        let mut image = Image::from(image_str);

        let src = &self.format_image_src(&image.oci[..])[..];
        let dst = &self.format_image_dest(&image.get_hashed_oci())[..];

        match self.skopeo.as_ref().unwrap().copy(src, dst) {
            true => {
                info!("Successfully pulled image {}", image_str);
                let image_src = format!("{}/{}", self.pull_directory, image.get_hashed_oci());
                let image_dst = format!("{}/{}", self.bundle_directory, image.get_hash());
                self.umoci.as_ref().unwrap().unpack(&image_src[..], &image_dst[..]);

                image.set_bundle(&image_dst);

                Some(image)
            },
            false => {
                error!("An error occured during the pulling process for {}", image_str);
                None
            }
        }
    }
}
