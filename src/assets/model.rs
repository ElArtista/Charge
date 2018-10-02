use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tobj;

pub struct Shape {
    pub name: String,
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub indices: Vec<u32>,
}

pub struct Model {
    pub shapes: Vec<Shape>,
}

impl Model {
    pub fn from_buf<B>(reader: &mut B) -> Result<Model, String>
    where
        B: BufRead,
    {
        let obj = try!(
            tobj::load_obj_buf(reader, |_| Err(tobj::LoadError::MaterialParseError))
                .map_err(|e| e.to_string())
        );
        let (models, _) = obj;
        let mut model = Model { shapes: Vec::new() };
        for m in models {
            let shape = Shape {
                name: m.name,
                positions: m.mesh.positions,
                normals: m.mesh.normals,
                texcoords: m.mesh.texcoords,
                indices: m.mesh.indices,
            };
            model.shapes.push(shape);
        }
        Ok(model)
    }

    pub fn from_file(fpath: &Path) -> Result<Model, String> {
        let file = try!(File::open(fpath).map_err(|e| e.to_string()));
        let mut reader = BufReader::new(file);
        Self::from_buf(&mut reader)
    }
}
