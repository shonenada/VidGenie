use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

use clap::Parser;
use colors_transform::Color;
use gl::types::GLsizei;
use glutin::ContextBuilder;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use log::{debug, info};

use vg_common::structs::RenderRequest;
use vg_gl::{GLBuffer, Program, set_attribute, Shader, Texture, Vertex, VertexArray};
use vg_video::{Frame, Video};

const VS_SRC: &str = r#"
#version 330 core
layout (location = 0) in vec2 position;
layout (location = 1) in vec2 verTexCoord;

out vec2 texCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    texCoord = verTexCoord;
}"#;

const FS_SRC: &str = r#"
#version 330 core
out vec4 FragColor;

in vec2 texCoord;

uniform sampler2D texture0;

void main() {
    FragColor = texture(texture0, texCoord);
    // FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}"#;

#[derive(Parser, Debug)]
struct Args {
    /// File to genie.
    #[clap(short, long)]
    file: String,
}

struct ProgramRet(Program, VertexArray);

#[rustfmt::skip]
const VERTICES: [Vertex; 4] = [
    Vertex([-0.5, -0.5], [0.0, 1.0]),
    Vertex([0.5, -0.5], [1.0, 1.0]),
    Vertex([0.5, 0.5], [1.0, 0.0]),
    Vertex([-0.5, 0.5], [0.0, 0.0]),
];

#[rustfmt::skip]
const INDICES: [i32; 6] = [
    0, 1, 2,
    2, 3, 0
];

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Args::parse();
    let file_path = cli.file;
    let mut file = File::open(file_path.clone()).map_err(anyhow::Error::from)?;
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let params: RenderRequest = serde_json::from_str(&data).map_err(anyhow::Error::from)?;

    debug!("Genie with {}; request: {:?}", file_path, params);

    let width = params.output.width;
    let height = params.output.height;

    let output = "./vid-output.mp4";
    vg_gst::init_gst();

    let mut video = Video::builder()
        .width(width)
        .height(height)
        .output_path(output)
        .build()?;

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("VidGenie");
    let cb = ContextBuilder::new().with_vsync(false);
    let context = cb.build_windowed(wb, &event_loop).unwrap();
    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let ver_shader = Shader::new(gl::VERTEX_SHADER, VS_SRC)?;
    let frag_shader = Shader::new_fragment(FS_SRC)?;
    let program = Program::new(&[ver_shader, frag_shader])?;
    unsafe { gl::Viewport(0, 0, width as GLsizei, height as GLsizei); }

    let vertex_array = VertexArray::new();
    vertex_array.bind();

    let vertex_buffer = GLBuffer::new_array_buffer();
    vertex_buffer.set_data(&VERTICES, gl::STATIC_DRAW);

    let index_buffer = GLBuffer::new_element_array_buffer();
    index_buffer.set_data(&INDICES, gl::STATIC_DRAW);

    let pos_attrib = program.get_attrib_location("position")?;
    let color_attrib = program.get_attrib_location("verTexCoord")?;
    unsafe {
        set_attribute!(vertex_array, pos_attrib, Vertex::0);
        set_attribute!(vertex_array, color_attrib, Vertex::1);
    }

    let texture0 = Texture::new();
    texture0.set_wrapping(gl::REPEAT);
    texture0.set_filtering(gl::LINE_LOOP);
    texture0.load(&Path::new("./resources/wood.jpg"))?; // TODO: replace me
    program.set_int_uniform("texture0", 0)?;

    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
    }

    let data_len = width * height * 3;
    let bg = params.timeline.background;

    video.start_render()?;
    for i in 0..60 {
        debug!("Writing frame num: {}", i);
        let mut pixels = vec![0u8; data_len as usize];
        let red = bg.get_red() / 255.0;
        let green = bg.get_green() / 255.0;
        let blue = bg.get_blue() / 255.0;
        unsafe {
            gl::ClearColor(red, green, blue, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            texture0.activate(gl::TEXTURE0);
            program.use_this();
            vertex_array.bind();
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

            gl::ReadPixels(
                0,
                0,
                width as i32,
                height as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut gl::types::GLvoid,
            );
        };
        let frame = Frame::new(pixels, i);
        video.send_frame(frame)?;
    }
    video.finish()?;
    video.until_rendered();
    println!("Render into {}", output);

    Ok(())
}
