use std::ffi::CString;

use anyhow::{anyhow, Result};
use gl::types::{GLint, GLsizei, GLuint};

use crate::error::ProgramError;
use crate::shader::Shader;

pub struct Program {
    pub id: GLuint,
}

impl Program {
    pub fn new(shaders: &[Shader]) -> Result<Self> {
        let program = unsafe {
            let program_id = gl::CreateProgram();
            for shader in shaders {
                gl::AttachShader(program_id, shader.id);
            }
            gl::LinkProgram(program_id);

            let mut is_success: GLint = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut is_success);

            if is_success != 1 {
                let mut error_log_size: GLint = 0;
                gl::GetShaderiv(program_id, gl::INFO_LOG_LENGTH, &mut error_log_size);

                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
                gl::GetProgramInfoLog(
                    program_id,
                    error_log_size,
                    &mut error_log_size,
                    error_log.as_mut_ptr() as *mut _,
                );

                error_log.set_len(error_log_size as usize);
                let log = String::from_utf8(error_log)?;
                return Err(anyhow!(ProgramError::LinkError(log)));
            }

            Ok(Self { id: program_id })
        };

        program
    }

    pub fn use_this(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn get_attrib_location(&self, attr: &str) -> Result<GLuint> {
        let attrib = CString::new(attr)?;
        let location = unsafe { gl::GetAttribLocation(self.id, attrib.as_ptr()) as GLuint };

        Ok(location)
    }

    pub fn get_uniform_location(&self, name: &str) -> Result<GLint> {
        let ret = unsafe {
            let uniform = CString::new(name)?;
            gl::GetUniformLocation(self.id, uniform.as_ptr())
        };

        Ok(ret)
    }

    pub fn set_int_uniform(&self, name: &str, value: i32) -> Result<()> {
        self.use_this();
        unsafe {
            let uniform = CString::new(name)?;
            let location = gl::GetUniformLocation(self.id, uniform.as_ptr());
            gl::Uniform1i(location, value);
        };
        Ok(())
    }

    pub fn set_float_uniform(&self, name: &str, value: f32) -> Result<()> {
        self.use_this();
        unsafe {
            let uniform = CString::new(name)?;
            let location = gl::GetUniformLocation(self.id, uniform.as_ptr());
            gl::Uniform1f(location, value);
        };
        Ok(())
    }

    pub fn set_int_array_uniform(&self, name: &str, value: &[i32]) -> Result<()> {
        self.use_this();
        unsafe {
            let uniform = CString::new(name)?;
            let location = gl::GetUniformLocation(self.id, uniform.as_ptr());
            gl::Uniform1iv(location, value.len() as GLsizei, value.as_ptr() as *const _);
        }
        Ok(())
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
