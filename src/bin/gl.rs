use std::ffi::CString;
use std::mem::size_of;
use std::ptr;
use std::time::Instant;

use anyhow::anyhow;
use derive_more::Display;
use glutin::dpi::LogicalSize;
use glutin::event::{Event, KeyboardInput, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use image::imageops::index_colors;
use thiserror::Error;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
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
uniform vec4 outColor;
void main() {
    fragColor = outColor;
}"#;

#[derive(Display, Debug, Error)]
enum ShaderError {
    CompileError(String),
}

fn create_shader(
    shader_type: gl::types::GLenum,
    src: CString,
) -> anyhow::Result<gl::types::GLuint> {
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

fn create_program(
    vertex_shader: gl::types::GLuint,
    fragment_shader: gl::types::GLuint,
) -> anyhow::Result<gl::types::GLuint> {
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

const GST_GSTREAMER_PER_OUTPUT: usize = 30;
const GST_BUFFER_SIZE: usize = 1024;

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("VidGenie")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT));
    let cb = ContextBuilder::new().with_vsync(true);
    let context = cb.build_windowed(wb, &event_loop).unwrap();
    let context = unsafe { context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Viewport(
            0,
            0,
            WIDTH as gl::types::GLsizei,
            HEIGHT as gl::types::GLsizei,
        );
    }

    let vs_shader = create_shader(gl::VERTEX_SHADER, CString::new(VS_SRC).unwrap())
        .expect("Failed to create vertex shader");
    let fs_shader = create_shader(gl::FRAGMENT_SHADER, CString::new(FS_SRC).unwrap())
        .expect("Failed to create fragment shader");

    let program = create_program(vs_shader, fs_shader).expect("Failed to link program");

    unsafe {
        gl::DeleteShader(vs_shader);
        gl::DeleteShader(fs_shader);
    }

    let vertices = vec![
        0.5, 0.5, 0.0,
        0.5, -0.5, 0.0,
        -0.5, -0.5, 0.0,
        -0.5, 0.5, 0.0,
    ];

    let indices = vec![0, 1, 3, 1, 2, 3];

    let mut VAO: gl::types::GLuint = 0;
    let mut VBO: gl::types::GLuint = 0;
    let mut EBO: gl::types::GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut VAO);
        gl::GenBuffers(1, &mut VBO);
        gl::GenBuffers(1, &mut EBO);
        gl::BindVertexArray(VAO);
        gl::BindBuffer(gl::ARRAY_BUFFER_BINDING, VBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<gl::types::GLfloat>() as gl::types::GLint,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    let mut start = Instant::now();
    let mut last_time = Instant::now();
    loop {
        let time = Instant::now();
        let delta = time.duration_since(last_time).as_secs_f32();
        last_time = time;

        let delta_start = time.duration_since(start).as_secs_f32();
        let color = delta_start.sin() / 2.0 + 0.5;

        // Clear screen
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);

            gl::Uniform4f(gl::GetUniformLocation(program, b"outColor\0".as_ptr() as *const _), 1.0, color, 1.0, 1.0);
            gl::BindVertexArray(VAO);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        context.swap_buffers();
    }

    /*
    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(program);
            gl::BindVertexArray(VAO);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
     */
}
