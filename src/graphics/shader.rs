use gl;
use gl::types::*;
use std;

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn from_sources(vs_src: &str, gs_src: Option<&str>, fs_src: &str) -> Shader {
        let attachments = vec![
            (gl::VERTEX_SHADER, Some(vs_src)),
            (gl::GEOMETRY_SHADER, gs_src),
            (gl::FRAGMENT_SHADER, Some(fs_src)),
        ];
        let prog;
        unsafe {
            prog = gl::CreateProgram();
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

    pub fn setup_attrib_indexes(&self, attribs: &[&str]) {
        for (i, attrib) in attribs.iter().enumerate() {
            unsafe {
                gl::BindAttribLocation(self.id, i as GLuint, attrib.as_ptr() as *const GLchar);
            }
        }
    }

    pub fn activate(&self) {
        unsafe { gl::UseProgram(self.id) }
    }

    //pub fn id(&self) -> GLuint {
    //    self.id
    //}
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}
