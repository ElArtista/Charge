pub mod image;
pub mod model;

pub use self::image::*;
pub use self::model::*;
use std::path::Path;
use std::io::BufRead;

pub trait Load
where
    Self: Sized,
{
    fn from_buf<B: BufRead>(buf: B) -> Result<Self, String>;
}

#[cfg(not(target_os = "android"))]
pub fn load<P: AsRef<Path>>(path: P) -> Result<Box<BufRead>, String> {
    use std::fs::File;
    use std::io::BufReader;

    let fullpath = Path::new("assets").join(&path);
    let file = try!(File::open(&fullpath).map_err(|e| e.to_string()));
    let reader = BufReader::new(file);
    Ok(Box::new(reader))
}

#[cfg(target_os = "android")]
pub fn load<P: AsRef<Path>>(path: P) -> Result<Box<BufRead>, String> {
    use android_glue;
    use std::io::Cursor;

    let fullpath = path.as_ref().to_str().expect("Can`t convert Path to &str");
    let buf = try!(android_glue::load_asset(fullpath).or(Err(format!("Could not load asset {}", fullpath))));
    Ok(Box::new(Cursor::new(buf)))
}
