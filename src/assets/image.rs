use super::Load;
use image;
pub use image::RgbaImage as Image;
use std::io::BufRead;

impl Load for Image {
    fn from_buf<B: BufRead>(mut buf: B) -> Result<Self, String> {
        let mut data = Vec::new();
        let _bytes_read = try!(buf.read_to_end(&mut data).map_err(|e| e.to_string()));
        let mut img = try!(image::load_from_memory(&data).map_err(|e| e.to_string()));
        img = img.flipv();
        Ok(img.to_rgba())
    }
}
