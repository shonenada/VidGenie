use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

use clap::Parser;
use colors_transform::Color;
use gl::types::{GLenum, GLsizei};
use log::debug;

use vg_gl::{init_gl, Renderer, Texture};
use vg_video::{Frame, ImageClipTexture, VideoEncoder};
use vg_video::RenderRequest;

const VS_SRC: &str = r#"
#version 450 core
layout (location = 0) in float inTexIdx;
layout (location = 1) in vec2 position;
layout (location = 2) in vec2 verTexCoord;

out vec2 texCoord;
out float texIdxf;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    texCoord = verTexCoord;
    texIdxf = inTexIdx;
}"#;

const FS_SRC: &str = r#"
#version 450 core
out vec4 FragColor;

in vec2 texCoord;
// in float texIdxf;

uniform float texIdxf;
uniform sampler2D textures[32];

void main() {
    int texIdx = int(texIdxf);
    FragColor = texture(textures[texIdx], texCoord);
}"#;

#[derive(Parser, Debug)]
struct Args {
    /// File to genie.
    #[clap(short, long)]
    file: String,

    /// Output path, default `vg-output.mp4`
    #[clap(short, long)]
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Args::parse();
    let file_path = cli.file;
    let output = match cli.output {
        Some(v) => v,
        None => "./vg-output.mp4".to_string(),
    };
    let mut file = File::open(file_path.clone()).map_err(anyhow::Error::from)?;
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let params: RenderRequest = serde_json::from_str(&data).map_err(anyhow::Error::from)?;

    debug!("Genie with {}; request: {:?}", file_path, params);

    let width = params.output.width;
    let height = params.output.height;

    let _gl_context = init_gl(width, height);
    let renderer = Renderer::new(VS_SRC, FS_SRC)?;

    let mut textures: Vec<Texture> = Vec::new();
    for track in &params.timeline.tracks {
        for (idx, clip) in track.clips.iter().enumerate() {
            let unit = ((gl::TEXTURE0 as usize) + idx) as GLenum;
            let mut texture = ImageClipTexture::new(&clip.asset.src, unit);
            texture.load()?;
            textures.push(texture.into_gl_texture());
        }
    }
    let textures_uniform: Vec<i32> = textures.iter().map(|each| {
        (each.unit - gl::TEXTURE0) as i32
    }).collect();
    renderer.program.set_int_array_uniform("textures", textures_uniform.as_slice())?;
    renderer.program.set_float_uniform("texIdxf", 0.0)?;

    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
    }
    vg_gst::init_gst();

    let mut video = VideoEncoder::builder()
        .width(width)
        .height(height)
        .output_path(&output)
        .build()?;

    let data_len = width * height * 3;
    let bg = params.timeline.background;

    unsafe {
        gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
    }
    video.start_render()?;
    for i in 0..60 {
        debug!("Writing frame num: {}", i);
        let mut pixels = vec![0u8; data_len as usize];
        let red = bg.get_red() / 255.0;
        let green = bg.get_green() / 255.0;
        let blue = bg.get_blue() / 255.0;
        unsafe {
            gl::ClearColor(red, green, blue, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            for texture in &textures {
                texture.activate();
            }
            renderer.program.use_this();
            renderer.vertex_array.bind();
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
