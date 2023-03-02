extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;

#[path = "../common.rs"]
mod common;

use std::ffi::CString;
use std::mem::size_of;
use std::{ptr, thread, time};
use std::time::Instant;
use std::sync::mpsc::{channel, Receiver};

use anyhow::{anyhow, Result};
use gst::prelude::*;
use derive_more::Display;
use glutin::dpi::LogicalSize;
use glutin::event::{Event, KeyboardInput, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use gst::prelude::*;
use thiserror::Error;

// const WIDTH: u32 = 1080;
// const HEIGHT: u32 = 720;
const WIDTH: u32 = 160;
const HEIGHT: u32 = 90;

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


struct Video {
    appsrc: gst_app::AppSrc,
    video_info: gst_video::VideoInfo,
    pipeline: gst::Pipeline,
    bus: Option<gst::Bus>,
}

impl Video {

    fn new(width: u32, height: u32, fps: gst::Fraction) -> Result<Self> {

        let video_info = gst_video::VideoInfo::builder(
                gst_video::VideoFormat::Rgb,
                width as u32,
                height as u32)
            .fps(fps)
            .build()?;

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
            video_info,
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
        let video_info = self.video_info.clone();
        let start = Instant::now();

        self.appsrc.set_callbacks(
            gst_app::AppSrcCallbacks::builder()
                .need_data(move |appsrc, _| {
                     if frame_num == 10 {
                        appsrc.end_of_stream().unwrap();
                        return;
                    }

                    let pixels = rx.recv().unwrap();
                    // println!("{:?}", pixels);

                    let mut buffer = gst::Buffer::with_size(pixels.len()).unwrap();
                    {
                        let buffer = buffer.get_mut().unwrap();
                        buffer.set_pts(frame_num * 200 * gst::ClockTime::MSECOND);
                        buffer.copy_from_slice(0, &pixels[..]).unwrap();
                    }

                    println!("Producing frame {}", frame_num);
                    // let r = if frame_num % 2 == 0 { 0 } else { 255 };
                    // let g = if frame_num % 3 == 0 { 0 } else { 255 };
                    // let b = if frame_num % 5 == 0 { 0 } else { 255 };
                    //
                    // let mut buffer = gst::Buffer::with_size(video_info.size()).unwrap();
                    // {
                    //     let buffer = buffer.get_mut().unwrap();
                    //
                    //     buffer.set_pts(frame_num * 200 * gst::ClockTime::MSECOND);
                    //
                    //     let mut vframe =
                    //         gst_video::VideoFrameRef::from_buffer_ref_writable(buffer, &video_info)
                    //             .unwrap();
                    //
                    //     let width = vframe.width() as usize;
                    //     let height = vframe.height() as usize;
                    //
                    //     let stride = vframe.plane_stride()[0] as usize;
                    //
                    //     for line in vframe
                    //         .plane_data_mut(0)
                    //         .unwrap()
                    //         .chunks_exact_mut(stride)
                    //         .take(height)
                    //     {
                    //         for pixel in line[..(4 * width)].chunks_exact_mut(4) {
                    //             pixel[0] = r;
                    //             pixel[1] = g;
                    //             pixel[2] = b;
                    //             pixel[3] = 255;
                    //         }
                    //     }
                    // }
                    //
                    frame_num += 1;
                    //
                    appsrc.push_buffer(buffer).unwrap();

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
        })
    }

    pub fn setup_draw(&mut self) -> Result<()> {
        let vertices = vec![
            0.5, 0.5, 0.0,
            0.5, -0.5, 0.0,
            -0.5, -0.5, 0.0,
            -0.5, 0.5, 0.0,
        ];

        let mut vbo: gl::types::GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
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
            // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Ok(())
    }

    pub fn draw(&self, start: &Instant) -> Result<Vec<u8>> {
        let time = Instant::now();
        let delta_start = time.duration_since(*start).as_secs_f32();
        let color = delta_start.sin() / 2.0 + 0.5;

        unsafe {
            gl::ClearColor(0.1, color, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(self.program_id);
            gl::Uniform4f(gl::GetUniformLocation(self.program_id, b"outColor\0".as_ptr() as *const _), 1.0, color, 1.0, 1.0);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
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
            // context.swap_buffers();
        };

        Ok(pixels)
    }
}

fn go(rx: Receiver<Vec<u8>>) -> Result<()> {
    // rx.recv().unwrap();
    gst::init()?;
    let mut video = Video::new(WIDTH, HEIGHT, gst::Fraction::new(5, 1))?;
    video.setup_pipeline("gl_output.mp4")?;
    video.setup_appsrc(rx)?;
    video.start()?;
    video.handle_message()?;
    video.stop()?;
    Ok(())
}

fn _main() -> Result<()> {
    let (tx, rx) = channel();
    thread::spawn(move || {
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

    let start = Instant::now();

    let one_sec = time::Duration::from_secs(1);
    for i in 0..10 {
        let tx1 = tx.clone();
        let pixels = program.draw(&start).unwrap();
        tx1.send(pixels).unwrap();
        thread::sleep(one_sec);
        println!("Sent {}", i);
    }

    Ok(())
}

fn main() {
    common::run(_main).unwrap();
}
