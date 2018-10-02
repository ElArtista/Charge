use assets::image::Image;
use gl;
use gl::types::*;

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn from_image(image: &Image) -> Texture {
        let (width, height) = image.dimensions();
        let data = image.as_ptr();
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as GLint,
                width as GLint,
                height as GLint,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data as *const GLvoid,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Texture { id }
    }

    pub fn bind(&self, bindpoint: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + bindpoint);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
