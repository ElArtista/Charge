use math::*;
use std::io::BufRead;
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
    pub fn from_buf<B: BufRead>(reader: &mut B) -> Result<Model, String> {
        Self::load(reader)
    }

    fn load<B: BufRead>(reader: &mut B) -> Result<Model, String> {
        let mut m = try!(Self::load_obj(reader));
        for shape in m.shapes.iter_mut() {
            if shape.normals.len() == 0 {
                shape.normals = Self::generate_normals(&mut shape.positions, &mut shape.indices);
            }
        }
        Ok(m)
    }

    fn generate_normals(positions: &mut [f32], indices: &mut [u32]) -> Vec<f32> {
        let mut normals = vec![0.0; positions.len()];
        for c in indices.chunks(3) {
            let tr = c.iter().map(|x| (x * 3) as usize).collect::<Vec<_>>();
            let v = tr
                .iter()
                .map(|i| make_vec3(&positions[*i..(*i + 3)]))
                .collect::<Vec<_>>();
            let e1 = v[1] - v[0];
            let e2 = v[2] - v[0];
            let nm = e1.cross(&e2).normalize();
            tr.iter().for_each(|i| {
                normals[*i..(*i + 3)]
                    .iter_mut()
                    .zip(&[nm.x, nm.y, nm.z])
                    .for_each(|(a, b)| *a += b)
            });
        }

        for nm in normals.chunks_mut(3) {
            let nnm = make_vec3(nm).normalize();
            nm.copy_from_slice(&[nnm.x, nnm.y, nnm.z]);
        }
        normals
    }

    fn load_obj<B: BufRead>(reader: &mut B) -> Result<Model, String> {
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
}
