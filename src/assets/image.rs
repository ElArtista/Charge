use super::Load;
use image;
pub use image::{DynamicImage as Image, GenericImageView};
use std::path::Path;

impl Load for Image {
    fn from_file(fpath: &Path) -> Result<Self, String> {
        let mut img = try!(image::open(fpath).map_err(|e| e.to_string()));
        img = img.flipv();
        Ok(img)
    }
}
