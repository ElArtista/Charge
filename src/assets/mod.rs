pub mod image;
pub mod model;

pub use self::image::*;
pub use self::model::*;
use std::path::Path;

pub trait Load
where
    Self: Sized,
{
    fn from_file(fpath: &Path) -> Result<Self, String>;
}
