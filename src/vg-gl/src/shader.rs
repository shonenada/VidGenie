use std::ffi::CString;
use std::ptr;

use anyhow::{anyhow, Result};
use gl::types::{GLuint, GLint, GLenum};

use crate::error::ShaderError;

pub struct Shader {
    pub id: GLuint,
}

impl Shader {
    pub fn new(shader_type: GLenum, source: &str) -> Result<Self> {
        let shader = unsafe {
            let src = CString::new(source)?;
            let shader_id = gl::CreateShader(shader_type);
            gl::ShaderSource(shader_id, 1, &src.as_ptr(), ptr::null());
            gl::CompileShader(shader_id);

            let mut is_success: GLint = 0;
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut is_success);

            if is_success != 1 {
                let mut error_log_size: GLint = 0;
                gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut error_log_size);

                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
                gl::GetShaderInfoLog(
                    shader_id,
                    error_log_size,
                    &mut error_log_size,
                    error_log.as_mut_ptr() as *mut _,
                );

                error_log.set_len(error_log_size as usize);

                let log = String::from_utf8(error_log)?;
                return Err(anyhow!(ShaderError::CompileError(log)));
            }

            Self {
                id: shader_id,
            }
        };
        Ok(shader)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}