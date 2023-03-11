use std::path::Path;

use anyhow::Result;
use gl::types::GLuint;
use image::EncodableLayout;

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

    pub fn load(&self, path: &Path) -> Result<()> {
        self.bind();
        unsafe {
            let img = image::open(path)?.to_rgba8();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_bytes().as_ptr() as *const _,
            );
        }
        Ok(())
    }

    pub fn activate(&self, unit: GLuint) {
        unsafe {
            gl::ActiveTexture(unit);
            self.bind();
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, [self.id].as_ptr());
        }
    }
}
