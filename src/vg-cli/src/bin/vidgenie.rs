use std::fs::File;
use std::io::Read;
use std::ptr;

use clap::Parser;
use colors_transform::Color;
use gl::types::{GLint, GLsizei};
use log::debug;

use vg_gl::{FrameBuffer, Indices, INDICES_PER_QUAD, init_gl, Quad, RenderBuffer, Renderer, Texture};
use anyhow::bail;
use vg_video::{AssetType, Frame, ImageClipTexture, RenderRequest, Transition, TransitionType, VideoEncoder};

const ALIASING_SAMPLES: GLsizei = 4;
const FPS: u32 = 30;
const SINGLE_QUAD_INDICES: Indices = Indices([0, 1, 2, 2, 3, 0]);

struct ClipRender {
    quad: Quad,
    texture: Texture,
    start_seconds: f32,
    start_frame: u64,
    end_frame: u64,
    transition: Option<Transition>,
    length_seconds: f32,
}

fn seconds_to_frames(seconds: f32) -> u64 {
    (seconds as f64 * FPS as f64).round() as u64
}

fn clip_alpha(clip: &ClipRender, frame: u64) -> f32 {
    let time = frame as f32 / FPS as f32;
    let local = time - clip.start_seconds;
    if local < 0.0 || local >= clip.length_seconds {
        return 0.0;
    }
    let mut alpha: f32 = 1.0;
    if let Some(transition) = clip.transition {
        let (fade_in, fade_out, duration) = transition_params(transition);
        let max_duration = (clip.length_seconds / 2.0).max(0.0);
        let duration = duration.min(max_duration);
        if duration > 0.0 {
            if fade_in && local < duration {
                alpha = alpha.min(local / duration);
            }
            if fade_out && local > clip.length_seconds - duration {
                alpha = alpha.min((clip.length_seconds - local) / duration);
            }
        }
    }
    alpha.clamp(0.0, 1.0)
}

fn transition_params(transition: Transition) -> (bool, bool, f32) {
    match transition {
        Transition::Named(TransitionType::Fade) => (true, true, 1.0),
        Transition::Detailed(details) => {
            let duration = details.duration.unwrap_or(1.0);
            let mut fade_in = false;
            let mut fade_out = false;
            if let Some(t) = details.in_transition {
                fade_in = t == TransitionType::Fade;
            }
            if let Some(t) = details.out {
                fade_out = t == TransitionType::Fade;
            }
            if !fade_in && !fade_out {
                if let Some(t) = details.transition_type {
                    if t == TransitionType::Fade {
                        fade_in = true;
                        fade_out = true;
                    }
                }
            }
            (fade_in, fade_out, duration)
        }
    }
}

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
    let mut render_clips: Vec<ClipRender> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut max_end_frame = 0u64;
    let mut texture_idx = 0u32;
    for track in &params.timeline.tracks {
        for clip in &track.clips {
            if clip.asset.asset_type != AssetType::Image {
                bail!("Only image assets are supported right now.");
            }
            if texture_idx >= 32 {
                bail!("Max 32 image clips supported per render.");
            }
            let start_frame = seconds_to_frames(clip.start);
            let end_frame = start_frame + seconds_to_frames(clip.length);
            if end_frame > max_end_frame {
                max_end_frame = end_frame;
            }

            let mut texture = ImageClipTexture::new(
                &clip.asset.src,
                width as f32,
                height as f32,
                texture_idx,
                clip.scale,
                clip.rotate,
            );
            texture.set_offset(clip.offset.x, clip.offset.y);
            texture.load()?;

            let quad = texture.quad();
            let inner_texture = texture.into_gl_texture();
            render_clips.push(ClipRender {
                quad,
                texture: inner_texture,
                start_seconds: clip.start,
                start_frame,
                end_frame,
                transition: clip.transition,
                length_seconds: clip.length,
            });
            texture_idx += 1;
        }
    }
    let textures_uniform: Vec<i32> = render_clips.iter().map(|each| {
        (each.texture.unit - gl::TEXTURE0) as i32
    }).collect();

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
    for clip in &render_clips {
        clip.texture.activate();
    }

    let mut video = VideoEncoder::builder()
        .width(width)
        .height(height)
        .output_path(&output)
        .build()?;

    let bg = params.timeline.background;
    let data_len = width * height * 3;
    let total_frames = max_end_frame.max(1);

    video.start_render()?;
    for i in 0..total_frames {
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

            renderer.program.set_float_uniform("uAlpha", 1.0)?;
            for clip in &render_clips {
                if i < clip.start_frame || i >= clip.end_frame {
                    continue;
                }
                let alpha = clip_alpha(clip, i);
                if alpha <= 0.0 {
                    continue;
                }
                renderer.program.set_float_uniform("uAlpha", alpha)?;
                renderer.set_vertex_buffer_data(&[clip.quad])?;
                renderer.set_index_buffer_data(&[SINGLE_QUAD_INDICES])?;
                gl::DrawElements(
                    gl::TRIANGLES,
                    INDICES_PER_QUAD as GLsizei,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }

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
