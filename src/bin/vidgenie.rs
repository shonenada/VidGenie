extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;

#[path = "../common.rs"]
mod common;

use std::ffi::CString;
use std::{ptr, thread, time};
use std::time::Instant;
use std::sync::mpsc::{channel, Receiver};

use anyhow::{anyhow, Result};
use derive_more::Display;
use glutin::dpi::LogicalSize;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use gst::prelude::*;
use image::EncodableLayout;
use thiserror::Error;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const FPS: u32 = 60;

const VS_SRC: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

out vec3 ver_color;
out vec2 ver_texture;

uniform vec4 outColor;

void main() {
    gl_Position = vec4(aPos, 1.0);
    ver_color = vec3(outColor.x, outColor.y, outColor.z);
    ver_texture = aTexCoord;
}"#;

const FS_SRC: &str = r#"
#version 330 core

out vec4 FragColor;

in vec3 ver_color;
in vec2 ver_texture;

uniform sampler2D inTexture;

void main() {
    FragColor = texture(inTexture, ver_texture) * vec4(ver_color, 1.0);
}"#;


struct Video {
    appsrc: gst_app::AppSrc,
    pipeline: gst::Pipeline,
    bus: Option<gst::Bus>,
}

impl Video {
    pub fn new(width: u32, height: u32, fps: gst::Fraction) -> Result<Self> {
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", &gst_video::VideoFormat::Rgb.to_string())
            .field("width", &(width as i32))
            .field("height", &(height as i32))
            .field("framerate", fps)
            .build();

        let appsrc = gst_app::AppSrc::builder()
            .caps(&caps)
            .format(gst::Format::Time)
            .build();

        Ok(Self {
            appsrc,
            bus: None,
            pipeline: gst::Pipeline::default(),
        })
    }

    fn setup_pipeline(&self, output_location: &str) -> Result<()> {
        let video_conv = gst::ElementFactory::make("videoconvert").build()?;
        let video_enc = gst::ElementFactory::make("x264enc").build()?;
        let video_parse = gst::ElementFactory::make("h264parse")
            .property_from_str("config-interval", &"3")
            .build()?;
        let qtmux = gst::ElementFactory::make("qtmux").build()?;
        let filesink = gst::ElementFactory::make("filesink")
            .property_from_str("location", output_location)
            .build()?;

        let links = [
            &(self.appsrc.upcast_ref()),
            &video_conv,
            &video_enc,
            &video_parse,
            &qtmux,
            &filesink,
        ];

        self.pipeline.add_many(&links)?;
        gst::Element::link_many(&links)?;

        Ok(())
    }

    fn setup_appsrc(&self, rx: Receiver<Vec<u8>>) -> Result<()> {
        let mut frame_num = 0;
        self.appsrc.set_callbacks(
            gst_app::AppSrcCallbacks::builder()
                .need_data(move |appsrc, _| {
                    if let Ok(pixels) = rx.recv() {
                        let mut buffer = gst::Buffer::with_size(pixels.len()).unwrap();
                        {
                            let buffer = buffer.get_mut().unwrap();
                            let fps = (1_000 / FPS) as u64;
                            buffer.set_pts(frame_num * fps * gst::ClockTime::MSECOND);
                            buffer.copy_from_slice(0, &pixels[..]).unwrap();
                        }
                        println!("Producing frame {}", frame_num);
                        frame_num += 1;
                        appsrc.push_buffer(buffer).unwrap();
                    } else {
                        appsrc.end_of_stream().unwrap();
                    }
                })
                .build()
        );

        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        self.pipeline.set_state(gst::State::Playing)?;
        self.bus = self.pipeline.bus();

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.pipeline.set_state(gst::State::Null)?;

        Ok(())
    }

    fn handle_message(&self) -> Result<()> {
        use gst::MessageView;

        if let Some(bus) = &self.bus {
            for msg in bus.iter_timed(gst::ClockTime::NONE) {
                match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(err) => {
                        self.pipeline.set_state(gst::State::Null)?;
                        eprintln!("{:?}", err);
                    }
                    _ => (),
                }
            }
        } else {
            eprintln!("Pipeline without bus. Shouldn't happen!");
        }

        Ok(())
    }

}

#[derive(Display, Debug, Error)]
enum ShaderError {
    CompileError(String),
}

fn create_shader(
    shader_type: gl::types::GLenum,
    src: CString,
) -> Result<gl::types::GLuint> {
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

struct RenderProgram {
    program_id: gl::types::GLuint,
    vao: gl::types::GLuint,
    texture: Option<gl::types::GLuint>,
}

impl RenderProgram {
    pub fn new(vertex_src: &str, fragment_src: &str) -> Result<Self> {
        let vertex_shader = create_shader(gl::VERTEX_SHADER, CString::new(vertex_src).unwrap())?;
        let fragment_shader = create_shader(gl::FRAGMENT_SHADER, CString::new(fragment_src).unwrap())?;

        let program_id;

        unsafe {
            program_id = gl::CreateProgram();
            gl::AttachShader(program_id, vertex_shader);
            gl::AttachShader(program_id, fragment_shader);
            gl::LinkProgram(program_id);

            let mut is_success: gl::types::GLint = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut is_success);

            if is_success != 1 {
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
                return Err(anyhow!(ProgramError::LinkError(log)));
            }

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        Ok(Self {
            program_id,
            vao: 0,
            texture: None,
        })
    }

    fn load_texture(&self, path: &str, fmt: gl::types::GLenum) -> Result<gl::types::GLuint> {
        let mut texture: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as gl::types::GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as gl::types::GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::types::GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::types::GLint);

            let img = image::open(path)?.to_rgb8();
            let img_data = img.as_bytes();
            println!("image: {}x{}", img.width(), img.height());

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as gl::types::GLint,
                img.width() as gl::types::GLsizei,
                img.height() as gl::types::GLsizei,
                0,
                fmt,
                gl::UNSIGNED_BYTE,
                img_data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        };

        Ok(texture)
    }

    pub fn setup_draw(&mut self) -> Result<()> {
        let vertices: Vec<gl::types::GLfloat> = vec![
            0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0,
            0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0,
            -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
            -0.5,  0.5, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0
        ];

        let indices: Vec<gl::types::GLuint> = vec![
            0, 1, 3,
            1, 2, 3,
        ];

        let mut vbo: gl::types::GLuint = 0;
        let mut ebo: gl::types::GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = (8 * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLint;

            let pos_attr = gl::GetAttribLocation(self.program_id, "aPos\0".as_ptr() as *const _) as gl::types::GLuint;
            gl::VertexAttribPointer(
                pos_attr,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(pos_attr);

            let color_attr = gl::GetAttribLocation(self.program_id, "aColor\0".as_ptr() as *const _) as gl::types::GLuint;
            gl::VertexAttribPointer(
                color_attr,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * std::mem::size_of::<gl::types::GLfloat>()) as *const _
            );
            gl::EnableVertexAttribArray(color_attr);

            let texture_attr = gl::GetAttribLocation(self.program_id, "aTexCoord\0".as_ptr() as *const _) as gl::types::GLuint;
            gl::VertexAttribPointer(
                texture_attr,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (6 * std::mem::size_of::<gl::types::GLfloat>()) as *const _
            );
            gl::EnableVertexAttribArray(texture_attr);

            self.texture = Some(self.load_texture("./resources/wood.jpg", gl::RGB).unwrap());
            gl::UseProgram(self.program_id);
            gl::Uniform1i(gl::GetUniformLocation(self.program_id, b"inTexture\0".as_ptr() as *const _), 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Ok(())
    }

    pub fn draw(&self, start: &Instant) -> Result<Vec<u8>> {
        let time = Instant::now();
        let delta_start = time.duration_since(*start).as_secs_f32();
        let color = delta_start.sin() / 2.0 + 0.5;

        let texture = self.texture.unwrap();

        unsafe {
            gl::ClearColor(0.6, 0.6, 0.6, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::Uniform4f(gl::GetUniformLocation(self.program_id, b"outColor\0".as_ptr() as *const _), (color + 0.1) / 1.0, color, (color + 0.3) % 1.0, 1.0);

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _); // FIXME
        }

        let mut pixels = vec![0u8; (WIDTH * HEIGHT * 3) as usize];
        unsafe {
            gl::ReadPixels(
                0,
                0,
                WIDTH as i32,
                HEIGHT as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut gl::types::GLvoid,
            );
        };

        Ok(pixels)
    }
}

fn go(rx: Receiver<Vec<u8>>) -> Result<()> {
    gst::init()?;
    let mut video = Video::new(WIDTH, HEIGHT, gst::Fraction::new(FPS as i32, 1))?;
    video.setup_pipeline("gl_output.mp4")?;
    video.setup_appsrc(rx)?;
    video.start()?;
    video.handle_message()?;
    video.stop()?;
    Ok(())
}

fn _main() -> Result<()> {
    let (tx, rx) = channel();
    let t = thread::spawn(move || {
        go(rx).unwrap();
    });

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("VidGenie")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT));
    let cb = ContextBuilder::new().with_vsync(true);
    let context = cb.build_windowed(wb, &event_loop).unwrap();
    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|s| context.get_proc_address(s) as *const _);
    let mut program = RenderProgram::new(VS_SRC, FS_SRC)?;
    program.setup_draw().expect("Failed to setup render program");

    unsafe {
        gl::UseProgram(program.program_id);
    }

    let start = Instant::now();

    let frames = FPS * 10; // 10s
    println!("Total frames: {}", frames);

    let duration = time::Duration::from_millis((1_000 / FPS) as u64);
    for i in 0..frames {
        let tx1 = tx.clone();
        let pixels = program.draw(&start).unwrap();
        tx1.send(pixels).unwrap();
        thread::sleep(duration);
        println!("Sent {}", i);

        context.swap_buffers().unwrap();
    }
    drop(tx);

    t.join().expect("Failed to join thread");

    Ok(())
}

fn main() {
    common::run(_main).unwrap();
}
