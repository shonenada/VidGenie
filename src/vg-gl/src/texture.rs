use std::path::Path;

use anyhow::Result;
use gl::types::{GLenum, GLint, GLsizei, GLuint};
use log::debug;

pub struct Texture {
    pub id: GLuint,
    pub unit: GLenum,
}

impl Texture {
    pub fn new(unit: GLenum) -> Self {
        let id = unsafe {
            let mut id: GLuint = 0;
            gl::GenTextures(1, &mut id);

            id
        };
        Self {
            id,
            unit,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn load_from_image(&self, img: image::DynamicImage) -> Result<()> {
        self.bind();
        unsafe {
            let format = if img.color().has_alpha() {
                gl::RGBA
            } else {
                gl::RGB
            };

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                img.width() as GLsizei,
                img.height() as GLsizei,
                0,
                format,
                gl::UNSIGNED_BYTE,
                img.as_bytes().as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Ok(())
    }

    pub fn load_from_path(&self, path: &Path) -> Result<()> {
        self.bind();
        let img = image::open(path)?;
        self.load_from_image(img)
    }

    pub fn set_wrapping(&self, mode: GLuint) {
        self.bind();
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, mode as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, mode as GLint);
        }
    }

    pub fn set_filtering(&self, mode: GLuint) {
        self.bind();
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, mode as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mode as GLint);
        }
    }

    pub fn activate(&self) {
        debug!("Activate texture {}", self.unit);
        unsafe {
            gl::ActiveTexture(self.unit);
            self.bind();
        }
    }

    pub fn bind_unit(&self) {
        unsafe {
            gl::BindTextureUnit(self.unit, self.id);
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
