use std::hash::Hash;
use crate::hash::generate_hash;

#[derive(Debug, Hash)]
pub struct Image {
    pub oci: String,
    pub name: String,
    pub tag: String,
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
        }
    }

    pub fn get_hash(&self) -> u64 {
        generate_hash(self)
    }

    pub fn get_hashed_oci(&self) -> String {
        format!("{}:{}", self.get_hash(), self.tag)
    }

}

// #[cfg(test)]
// mod tests {
//     use crate::image::Image;
//
//     #[test]
//     fn it_build_an_image() {
//         let image_str = "nginx:latest";
//
//         let image = Image::new(image_str);
//
//         assert_eq!(2 + 2, 4);
//     }
// }