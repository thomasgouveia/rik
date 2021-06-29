use crate::image::Image;
use std::collections::HashMap;
use crate::skopeo::{Skopeo, SkopeoConfiguration};
use log::{info, error, debug};
use crate::umoci::{Umoci, UmociConfiguration, UnpackArgs};
use crate::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ImageManager {
    /// Directory where the pulled images will be stored
    pulled_images: HashMap<u64, Image>,
    umoci: Umoci,
    skopeo: Skopeo
}

/// Implementation of the skopeo library
/// coupled with umoci in order to pull images compatible with cri.
impl ImageManager {

    /// Create a new Puller
    pub fn new(umoci_config: UmociConfiguration, skopeo_config: SkopeoConfiguration) -> Result<Self> {

        let umoci = Umoci::new(umoci_config)?;
        let skopeo = Skopeo::new(skopeo_config)?;

        debug!("ImageManager initialized.");

        Ok(ImageManager {
            pulled_images: HashMap::<u64, Image>::new(),
            umoci,
            skopeo,
        })
    }

    /// Format the image for skopeo with the following format:
    /// docker://<IMAGE>
    fn format_image_src(&self, image: &String) -> String {
        format!("docker://{}", image)
    }

    /// Pull image locally
    pub async fn pull(&mut self, image_str: &str) -> Result<Image> {

        info!("Pulling image {}", image_str);

        let mut image = Image::from(image_str);

        let src = self.format_image_src(&image.oci);

        let image_path = self
            .skopeo
            .copy(&src, &format!("{}", &image.get_hashed_oci()), Default::default())
            .await?;

        debug!("{} copied into {}", image_str, image_path);

        let bundle = self.umoci.unpack(&image.get_uuid(), Some(&UnpackArgs {
            image: PathBuf::from(&format!("{}:{}", image_path, image.tag)),
            rootless: false,
            uid_map: None,
            gid_map: None,
            keep_dirlinks: false,
        })).await?;

        image.set_bundle(&bundle[..]);

        info!("Successfully pulled image {}", image_str);

        Ok(image)
    }
}
