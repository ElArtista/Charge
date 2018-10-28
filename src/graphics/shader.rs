use gl;
use gl::types::*;
use std;
use std::convert::From;

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn new(
        vs_src: &str,
        gs_src: Option<&str>,
        fs_src: &str,
        attribs: Option<&[&str]>,
    ) -> Shader {
        let attachments = vec![
            (gl::VERTEX_SHADER, Some(vs_src)),
            (gl::GEOMETRY_SHADER, gs_src),
            (gl::FRAGMENT_SHADER, Some(fs_src)),
        ];
        let prog;
        unsafe {
            prog = gl::CreateProgram();
            if let Some(attribs) = attribs {
                for (i, attrib) in attribs.iter().enumerate() {
                    let name = format!("{}\0", attrib);
                    gl::BindAttribLocation(prog, i as GLuint, name.as_ptr() as *const GLchar);
                }
            }
            for a in attachments {
                if let Some(src) = a.1 {
                    let id = gl::CreateShader(a.0);
                    let s = src.as_ptr() as *const GLchar;
                    let l = src.len() as GLint;
                    gl::ShaderSource(id, 1, &s, &l);
                    gl::CompileShader(id);
                    if let Some(err) = Shader::check_compilation_error(id) {
                        println!("{}", err);
                        panic!("Shader compilation error occured!");
                    }
                    gl::AttachShader(prog, id);
                    gl::DeleteShader(id);
                }
            }
            gl::LinkProgram(prog);
            if let Some(err) = Shader::check_linking_error(prog) {
                println!("{}", err);
                panic!("Shader linking error occured!");
            }
        }
        Shader { id: prog }
    }

    unsafe fn check_compilation_error(id: GLuint) -> Option<String> {
        let mut success: GLint = 1;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = String::with_capacity((len + 1) as usize);
            buf.extend([' '].iter().cycle().take(len as usize));
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                buf.as_bytes_mut().as_mut_ptr() as *mut GLchar,
            );
            return Some(buf);
        }
        None
    }

    unsafe fn check_linking_error(prog: GLuint) -> Option<String> {
        let mut success: GLint = 1;
        gl::GetProgramiv(prog, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = String::with_capacity((len + 1) as usize);
            buf.extend([' '].iter().cycle().take(len as usize));
            gl::GetProgramInfoLog(prog, len, std::ptr::null_mut(), buf.as_ptr() as *mut GLchar);
            return Some(buf);
        }
        None
    }

    fn get_uniform_location(&self, name: &str) -> Option<i32> {
        let n = format!("{}\0", name);
        let location = unsafe { gl::GetUniformLocation(self.id, n.as_ptr() as *const GLchar) };
        if location == -1 {
            return None;
        }
        Some(location)
    }

    pub fn set_uniform<'a, T: Into<Uniform<'a>>>(&self, name: &str, value: T) {
        if let Some(loc) = self.get_uniform_location(name) {
            let count = 1; // TODO: Support uniform arrays
            unsafe {
                match value.into() {
                    Uniform::Bool(v) => gl::Uniform1iv(loc, count, &(v as GLint) as *const GLint),
                    Uniform::Float1(v) => gl::Uniform1fv(loc, count, &v as *const GLfloat),
                    Uniform::Float2(v) => gl::Uniform2fv(loc, count, v.as_ptr() as *const GLfloat),
                    Uniform::Float3(v) => gl::Uniform3fv(loc, count, v.as_ptr() as *const GLfloat),
                    Uniform::Float4(v) => gl::Uniform4fv(loc, count, v.as_ptr() as *const GLfloat),
                    Uniform::Int1(v) => gl::Uniform1iv(loc, count, &v as *const GLint),
                    Uniform::Int2(v) => gl::Uniform2iv(loc, count, v.as_ptr() as *const GLint),
                    Uniform::Int3(v) => gl::Uniform3iv(loc, count, v.as_ptr() as *const GLint),
                    Uniform::Int4(v) => gl::Uniform4iv(loc, count, v.as_ptr() as *const GLint),
                    Uniform::UInt1(v) => gl::Uniform1uiv(loc, count, &v as *const GLuint),
                    Uniform::UInt2(v) => gl::Uniform2uiv(loc, count, v.as_ptr() as *const GLuint),
                    Uniform::UInt3(v) => gl::Uniform3uiv(loc, count, v.as_ptr() as *const GLuint),
                    Uniform::UInt4(v) => gl::Uniform4uiv(loc, count, v.as_ptr() as *const GLuint),
                    Uniform::Matrix2(v) => {
                        gl::UniformMatrix2fv(loc, count, gl::FALSE, v.as_ptr() as *const GLfloat)
                    }
                    Uniform::Matrix3(v) => {
                        gl::UniformMatrix3fv(loc, count, gl::FALSE, v.as_ptr() as *const GLfloat)
                    }
                    Uniform::Matrix4(v) => {
                        gl::UniformMatrix4fv(loc, count, gl::FALSE, v.as_ptr() as *const GLfloat)
                    }
                }
            }
        }
    }

    pub fn activate(&self) {
        unsafe { gl::UseProgram(self.id) }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub enum Uniform<'a> {
    Bool(bool),
    Float1(f32),
    Float2(&'a [f32; 2]),
    Float3(&'a [f32; 3]),
    Float4(&'a [f32; 4]),
    Int1(i32),
    Int2(&'a [i32; 2]),
    Int3(&'a [i32; 3]),
    Int4(&'a [i32; 4]),
    UInt1(u32),
    UInt2(&'a [u32; 2]),
    UInt3(&'a [u32; 3]),
    UInt4(&'a [u32; 4]),
    Matrix2(&'a [[f32; 2]; 2]),
    Matrix3(&'a [[f32; 3]; 3]),
    Matrix4(&'a [[f32; 4]; 4]),
}

impl<'a> From<bool> for Uniform<'a> {
    fn from(item: bool) -> Self {
        Uniform::Bool(item)
    }
}

impl<'a> From<f32> for Uniform<'a> {
    fn from(item: f32) -> Self {
        Uniform::Float1(item)
    }
}

impl<'a> From<&'a [f32; 2]> for Uniform<'a> {
    fn from(item: &'a [f32; 2]) -> Self {
        Uniform::Float2(item)
    }
}

impl<'a> From<&'a [f32; 3]> for Uniform<'a> {
    fn from(item: &'a [f32; 3]) -> Self {
        Uniform::Float3(item)
    }
}

impl<'a> From<&'a [f32; 4]> for Uniform<'a> {
    fn from(item: &'a [f32; 4]) -> Self {
        Uniform::Float4(item)
    }
}

impl<'a> From<i32> for Uniform<'a> {
    fn from(item: i32) -> Self {
        Uniform::Int1(item)
    }
}

impl<'a> From<&'a [i32; 2]> for Uniform<'a> {
    fn from(item: &'a [i32; 2]) -> Self {
        Uniform::Int2(item)
    }
}

impl<'a> From<&'a [i32; 3]> for Uniform<'a> {
    fn from(item: &'a [i32; 3]) -> Self {
        Uniform::Int3(item)
    }
}

impl<'a> From<&'a [i32; 4]> for Uniform<'a> {
    fn from(item: &'a [i32; 4]) -> Self {
        Uniform::Int4(item)
    }
}

impl<'a> From<u32> for Uniform<'a> {
    fn from(item: u32) -> Self {
        Uniform::UInt1(item)
    }
}

impl<'a> From<&'a [u32; 2]> for Uniform<'a> {
    fn from(item: &'a [u32; 2]) -> Self {
        Uniform::UInt2(item)
    }
}

impl<'a> From<&'a [u32; 3]> for Uniform<'a> {
    fn from(item: &'a [u32; 3]) -> Self {
        Uniform::UInt3(item)
    }
}

impl<'a> From<&'a [u32; 4]> for Uniform<'a> {
    fn from(item: &'a [u32; 4]) -> Self {
        Uniform::UInt4(item)
    }
}

impl<'a> From<&'a [[f32; 2]; 2]> for Uniform<'a> {
    fn from(item: &'a [[f32; 2]; 2]) -> Self {
        Uniform::Matrix2(item)
    }
}

impl<'a> From<&'a [[f32; 3]; 3]> for Uniform<'a> {
    fn from(item: &'a [[f32; 3]; 3]) -> Self {
        Uniform::Matrix3(item)
    }
}

impl<'a> From<&'a [[f32; 4]; 4]> for Uniform<'a> {
    fn from(item: &'a [[f32; 4]; 4]) -> Self {
        Uniform::Matrix4(item)
    }
}
