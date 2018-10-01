use gl;
use gl::types::*;
use std;

#[derive(Clone, Copy)]
pub enum Vattr {
    Position = 0,
    Normal,
    UV0,
    UV1,
    Tangent,
    Color,
}

// Parallel to the Vattr enum
const VATTR_MAP: &[(GLenum, usize)] = &[
    (gl::FLOAT, 3),
    (gl::FLOAT, 3),
    (gl::FLOAT, 2),
    (gl::FLOAT, 2),
    (gl::FLOAT, 3),
    (gl::FLOAT, 3),
];

pub fn vattr_flag(a: Vattr) -> u32 {
    1 << (a as u32)
}

pub struct Mesh {
    vbo: GLuint,
    ebo: GLuint,
    num_verts: usize,
    num_indcs: usize,
    attrib_mask: u32,
}

impl Mesh {
    pub fn from_data(vertices: &[f32], indices: Option<&[u32]>, attrib_mask: u32) -> Mesh {
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        let num_verts = vertices.len();
        let num_indcs = match indices {
            Some(indices) => indices.len(),
            None => 0,
        };
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(vertices) as GLsizeiptr,
                vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            if let Some(indices) = indices {
                gl::GenBuffers(1, &mut ebo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    std::mem::size_of_val(indices) as GLsizeiptr,
                    indices.as_ptr() as *const GLvoid,
                    gl::STATIC_DRAW,
                );
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }
        Mesh {
            vbo,
            ebo,
            num_verts,
            num_indcs,
            attrib_mask,
        }
    }

    fn attrib_setup(&self) {
        let mut offset = 0;
        for attr in [
            Vattr::Position,
            Vattr::Normal,
            Vattr::UV0,
            Vattr::UV1,
            Vattr::Tangent,
            Vattr::Color,
        ]
            .iter()
        {
            if (self.attrib_mask & vattr_flag(*attr)) != 0 {
                let attr_idx = *attr as u32;
                let (component_type, num_components) = VATTR_MAP[attr_idx as usize];
                unsafe {
                    gl::EnableVertexAttribArray(attr_idx);
                    gl::VertexAttribPointer(
                        attr_idx,
                        num_components as GLint,
                        component_type,
                        gl::FALSE,
                        0,
                        offset as *const GLvoid,
                    );
                }
                offset += self.num_verts
                    * num_components
                    * (match component_type {
                        gl::BYTE => std::mem::size_of::<GLbyte>(),
                        gl::UNSIGNED_BYTE => std::mem::size_of::<GLubyte>(),
                        gl::SHORT => std::mem::size_of::<GLshort>(),
                        gl::UNSIGNED_SHORT => std::mem::size_of::<GLushort>(),
                        gl::INT => std::mem::size_of::<GLint>(),
                        gl::UNSIGNED_INT => std::mem::size_of::<GLuint>(),
                        gl::HALF_FLOAT => std::mem::size_of::<GLhalf>(),
                        gl::FLOAT => std::mem::size_of::<GLfloat>(),
                        gl::DOUBLE => std::mem::size_of::<GLdouble>(),
                        _ => 0,
                    });
            }
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.attrib_setup();
            if self.is_indexed() {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.num_indcs as GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            } else {
                gl::DrawArrays(gl::TRIANGLES, 0, self.num_verts as GLsizei);
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn is_indexed(&self) -> bool {
        self.num_indcs != 0
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            if self.is_indexed() {
                gl::DeleteBuffers(1, &mut self.ebo);
            }
            gl::DeleteBuffers(1, &mut self.vbo);
        }
    }
}
