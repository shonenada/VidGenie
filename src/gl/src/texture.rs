use anyhow::{anyhow, Result};
use gl::types::{GLuint, GLenum, GLsizeiptr};

pub struct Texture {
    pub id: GLuint,
}

impl Texture {
    pub fn new() -> Self {
        let id = unsafe {
            let mut id: GLuint = 0;
            gl::GenTextures(1, &mut id);

            id
        };
        Self { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, [self.id].as_ptr());;;;
        }
    }
}