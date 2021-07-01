use std::hash::Hash;
use shared::utils::generate_hash;
use std::path::{Path, PathBuf};

#[derive(Debug, Hash)]
pub struct Image {
    pub oci: String,
    pub name: String,
    pub tag: String,
    pub bundle: Option<PathBuf>,
}

impl Image {
    /// Create a new image
    pub fn from(img: &str) -> Self {

        let splitted_image: Vec<&str> = img.split(':').collect();
        let image_name = *splitted_image.get(0).unwrap();
        let image_tag = *splitted_image.get(1).unwrap();

        Image {
            oci: String::from(img),
            name: String::from(image_name),
            tag: String::from(image_tag),
            bundle: None
        }
    }
    
    pub fn get_uuid(&self) -> String {
        format!("{}-{}", self.name, self.get_hash())
    }

    pub fn set_bundle(&mut self, bundle: &str) {
        self.bundle = Some(PathBuf::from(bundle));
    }

    pub fn get_hash(&self) -> u64 {
        generate_hash(self)
    }

    pub fn get_hashed_oci(&self) -> String {
        format!("{}-{}:{}", self.name, self.get_hash(), self.tag)
    }
}

#[cfg(test)]
mod tests {
    use crate::image::Image;

    #[test]
    fn test_it_parse_a_image_string() {
        let image_str = "alpine:latest";

        let image = Image::from(image_str);

        assert_eq!(image.name, "alpine");
        assert_eq!(image.tag, "latest");
    }
}




