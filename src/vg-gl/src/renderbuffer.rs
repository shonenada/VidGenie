use gl::types::{GLsizei, GLuint};

pub struct RenderBuffer {
    pub id: GLuint,
}

impl RenderBuffer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenRenderbuffers(1, &mut id);
        };
        Self { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindRenderbuffer(gl::RENDERBUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        }
    }

    pub fn storage(&self, width: i32, height: i32) {
        unsafe {
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH24_STENCIL8,
                width as GLsizei,
                height as GLsizei,
            );
        }
    }

    pub fn storage_multi_sample(&self, samples: GLsizei, width: GLsizei, height: GLsizei) {
        unsafe {
            self.bind();
            gl::RenderbufferStorageMultisample(
                gl::RENDERBUFFER,
                samples,
                gl::DEPTH24_STENCIL8,
                width,
                height,
            );
            self.unbind();
        }
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.id);
        }
    }
}
