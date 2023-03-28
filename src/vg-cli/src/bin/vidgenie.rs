use std::fs::File;
use std::io::Read;
use std::ptr;

use clap::Parser;
use colors_transform::Color;
use gl::types::{GLint, GLsizei};
use log::debug;

use vg_gl::{FrameBuffer, Indices, INDICES_PER_QUAD, init_gl, Quad, RenderBuffer, Renderer, Texture};
use vg_video::{Frame, ImageClipTexture, VideoEncoder};
use vg_video::RenderRequest;

const ALIASING_SAMPLES: GLsizei = 4;

#[derive(Parser, Debug)]
struct Args {
    /// File to genie.
    #[clap(short, long)]
    file: String,

    /// Output path, default `vg-output.mp4`
    #[clap(short, long)]
    output: Option<String>,
}

impl Args {
    fn get_output(&self) -> String {
        match &self.output {
            Some(v) => v.clone(),
            None => "./vg-output.mp4".to_string(),
        }
    }

    fn into_request(self) -> anyhow::Result<RenderRequest> {
        let file_path = self.file;
        let mut file = File::open(file_path.clone()).map_err(anyhow::Error::from)?;
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let params: RenderRequest = serde_json::from_str(&data).map_err(anyhow::Error::from)?;
        debug!("Genie with {}; request: {:?}", file_path, params);
        Ok(params)
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    let output = args.get_output();
    let params = args.into_request()?;
    let width = params.output.width;
    let height = params.output.height;

    vg_gst::init_gst();
    let _gl_context = init_gl(width, height);

    let renderer = Renderer::new()?;
    let mut indices_arr: Vec<Indices> = Vec::new();
    let mut quads: Vec<Quad> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    for track in &params.timeline.tracks {
        for (idx, clip) in track.clips.iter().enumerate() {
            let mut texture = ImageClipTexture::new(
                &clip.asset.src,
                width as f32,
                height as f32,
                idx as u32,
                clip.scale,
                clip.rotate,
            );
            texture.set_offset(clip.offset.x, clip.offset.y);
            texture.load()?;

            let quad = texture.quad();
            quads.push(quad);
            let indices = texture.indices();
            indices_arr.push(indices);
            let inner_texture = texture.into_gl_texture();
            textures.push(inner_texture);
        }
    }
    let textures_uniform: Vec<i32> = textures.iter().map(|each| {
        (each.unit - gl::TEXTURE0) as i32
    }).collect();

    debug!("Quads: {:#?}", quads.as_slice());
    debug!("Indices: {:#?}", indices_arr.as_slice());

    renderer.set_vertex_buffer_data(quads.as_slice())?;
    renderer.set_index_buffer_data(indices_arr.as_slice())?;
    renderer.set_attrs()?;
    renderer.program.set_int_array_uniform("textures", textures_uniform.as_slice())?;

    // Multi sample for anti aliasing
    let color_texture = Texture::new_without_unit(gl::TEXTURE_2D_MULTISAMPLE);
    color_texture.multi_sample(ALIASING_SAMPLES, width as i32, height as i32);
    let renderbuffer = RenderBuffer::new();
    renderbuffer.storage_multi_sample(ALIASING_SAMPLES, width as i32, height as i32);

    let color_frame = FrameBuffer::new(gl::COLOR_ATTACHMENT0);
    color_frame.bind();
    color_frame.attach_texture(&color_texture);
    color_frame.bind_renderbuffer(&renderbuffer);
    match color_frame.check_status() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("framebuffer error: {}", err);
        }
    }
    color_frame.unbind();

    let screen_texture = Texture::new_without_unit(gl::TEXTURE_2D);
    screen_texture.bind();
    screen_texture.load_for_framebuffer(width as i32, height as i32);
    screen_texture.set_filtering(gl::LINEAR);

    let result_frame = FrameBuffer::new(gl::COLOR_ATTACHMENT0);
    result_frame.bind();
    result_frame.attach_texture(&screen_texture);
    match result_frame.check_status() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("framebuffer error: {}", err);
        }
    }
    result_frame.unbind();

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::MULTISAMPLE);
        gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
    }
    for texture in &textures {
        texture.activate();
    }

    let mut video = VideoEncoder::builder()
        .width(width)
        .height(height)
        .output_path(&output)
        .build()?;

    let bg = params.timeline.background;
    let data_len = width * height * 3;
    let draw_count = INDICES_PER_QUAD * indices_arr.len();

    video.start_render()?;
    for i in 0..60 {
        debug!("Writing frame num: {}", i);
        let mut pixels = vec![0u8; data_len as usize];
        let red = bg.get_red() / 255.0;
        let green = bg.get_green() / 255.0;
        let blue = bg.get_blue() / 255.0;
        unsafe {
            color_frame.bind();
            gl::Enable(gl::DEPTH_TEST);
            gl::ClearColor(red, green, blue, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            renderer.program.use_this();
            renderer.vertex_array.bind();

            gl::DrawElements(gl::TRIANGLES, draw_count as GLsizei, gl::UNSIGNED_INT, ptr::null());

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, color_frame.id);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, result_frame.id);
            gl::BlitFramebuffer(
                0,
                0,
                width as GLint,
                height as GLint,
                0,
                0,
                width as GLint,
                height as GLint,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST);

            result_frame.bind();

            gl::ReadPixels(
                0,
                0,
                width as i32,
                height as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut gl::types::GLvoid,
            );

            gl::Disable(gl::DEPTH_TEST);
            color_frame.unbind();
        };
        let frame = Frame::new(pixels, i);
        video.send_frame(frame)?;
    }
    video.finish()?;
    video.until_rendered();
    println!("Render into {}", output);

    Ok(())
}
