use std::path::Path;
use std::ptr;

use anyhow::Result;
use gl::types::{GLenum, GLint, GLsizei, GLuint};

#[derive(Clone)]
pub struct Texture {
    pub id: GLuint,
    pub unit: GLenum,
    pub target: GLenum,
}

impl Texture {
    pub fn new(unit: GLenum, target: GLenum) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        };
        Self {
            id,
            unit,
            target,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(self.target, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(self.target, 0);
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
                self.target,
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
        let img = image::open(path)?;
        self.load_from_image(img)
    }

    pub fn load_for_framebuffer(&self, width: i32, height: i32) {
        self.bind();
        unsafe {
            gl::TexImage2D(
                self.target,
                0,
                gl::RGB as GLint,
                width as GLsizei,
                height as GLsizei,
                0,
                gl::RGB as GLenum,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            )
        }
    }

    pub fn set_wrapping(&self, mode: GLuint) {
        self.bind();
        unsafe {
            gl::TexParameteri(self.target, gl::TEXTURE_WRAP_S, mode as GLint);
            gl::TexParameteri(self.target, gl::TEXTURE_WRAP_T, mode as GLint);
        }
    }

    pub fn set_filtering(&self, mode: GLuint) {
        self.bind();
        unsafe {
            gl::TexParameteri(self.target, gl::TEXTURE_MIN_FILTER, mode as GLint);
            gl::TexParameteri(self.target, gl::TEXTURE_MAG_FILTER, mode as GLint);
        }
    }

    pub fn activate(&self) {
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

    pub fn multi_sample_2d(&self, sample: GLsizei, width: i32, height: i32) {
        unsafe {
            gl::TexImage2DMultisample(
                self.target,
                sample,
                gl::RGB,
                width as GLsizei,
                height as GLsizei,
                gl::TRUE,
            );
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
