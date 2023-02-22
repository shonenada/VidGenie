extern crate gstreamer as gst;

use std::ffi::CString;
use std::ptr;
use anyhow::anyhow;
use gst::prelude::*;
use glfw::Context;
use derive_more::Display;
use thiserror::Error;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const TITLE: &str = "VidGenie";

const VS_SRC: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
void main() {
    gl_Position = vec4(aPos, 1.0);
}"#;

const FS_SRC: &str = r#"
#version 330 core
out vec4 fragColor;
void main() {
    fragColor = vec4(1.0f, 1.0f, 0.2f, 1.0f);
}"#;

#[derive(Display, Debug, Error)]
enum ShaderError {
    CompileError(String),
}

fn create_shader(shader_type: gl::types::GLenum, src: CString) -> anyhow::Result<gl::types::GLuint> {
    unsafe {
        let shader_id = gl::CreateShader(shader_type);
        gl::ShaderSource(shader_id, 1, &src.as_ptr(), ptr::null());
        gl::CompileShader(shader_id);

        let mut is_success: gl::types::GLint = 0;
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut is_success);

        if is_success == 1 {
            Ok(shader_id)
        } else {
            let mut error_log_size: gl::types::GLint = 0;
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

            Err(anyhow!(ShaderError::CompileError(log)))
        }
    }
}

#[derive(Display, Debug, Error)]
enum ProgramError {
    LinkError(String),
}

fn create_program(vertex_shader: gl::types::GLuint, fragment_shader: gl::types::GLuint) -> anyhow::Result<gl::types::GLuint> {
    unsafe {
        let program_id = gl::CreateProgram();
        gl::AttachShader(program_id, vertex_shader);
        gl::AttachShader(program_id, fragment_shader);
        gl::LinkProgram(program_id);

        let mut is_success: gl::types::GLint = 0;
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut is_success);

        if is_success == 1 {
            Ok(program_id)
        } else {
            let mut error_log_size: gl::types::GLint = 0;
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
            Err(anyhow!(ProgramError::LinkError(log)))
        }
    }
}

fn main() {
    gst::init().unwrap();

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    let (mut window, events) = glfw.create_window(
        WIDTH, HEIGHT,
        TITLE, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    window.set_key_polling(true);
    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let vs_shader = create_shader(gl::VERTEX_SHADER, CString::new(VS_SRC).unwrap())
        .expect("Failed to create vertex shader");
    let fs_shader = create_shader(gl::FRAGMENT_SHADER, CString::new(FS_SRC).unwrap())
        .expect("Failed to create fragment shader");

    let program = create_program(vs_shader, fs_shader)
        .expect("Failed to link program");

    while !window.should_close() {

    }
}