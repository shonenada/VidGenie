use std::fmt::Error;

use anyhow::anyhow;
use gl::types::{GLenum, GLint, GLuint};

use crate::RenderBuffer;
use crate::Texture;

pub struct FrameBuffer {
    pub id: GLuint,
    attachment: GLenum,
}

impl FrameBuffer {
    pub fn new(attachment: GLenum) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
        }
        Self { id, attachment }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
        }
    }

    pub fn bind_read(&self) {
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.id);
        }
    }

    pub fn bind_draw(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.id);
        }
    }

    pub fn check_status(&self) -> anyhow::Result<()> {
        unsafe {
            let rt = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if rt == gl::FRAMEBUFFER_COMPLETE {
                Ok(())
            } else {
                Err(anyhow!("Framebuffer is uncompleted"))
            }
        }
    }

    pub fn attach_texture(&self, tex: &Texture) {
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                self.attachment,
                tex.target,
                tex.id,
                0,
            );
            tex.unbind();
        }
    }

    pub fn bind_renderbuffer(&self, render_buffer: &RenderBuffer) {
        unsafe {
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                render_buffer.id,
            );
        }
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}