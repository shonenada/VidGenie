extern crate image as image_crate;

use gl::types::GLenum;
use log::info;

use crate::request::transform::{interpolate_keyframes, Keyframe};
use vg_gl::{Indices, Quad, Texture, Transformer, Vertex, VERTEX_PER_QUAD};

#[derive(Default)]
pub struct ImageClipOffset {
    x: f32,
    y: f32,
}

pub struct ImageClipTexture {
    url: String,
    texture_idx: u32,

    texture: Option<Texture>,
    canvas_width: f32,
    canvas_height: f32,
    image_width: f32,
    image_height: f32,
    offset: ImageClipOffset,
    base_scale: f32,
    base_rotate: f32,
    keyframes: Option<Vec<Keyframe>>,
}

impl ImageClipTexture {
    pub fn new(
        url: &str,
        canvas_width: f32,
        canvas_height: f32,
        idx: u32,
        scale: f32,
        rotate: f32,
    ) -> Self {
        Self {
            url: url.to_string(),
            texture_idx: idx,
            canvas_width,
            canvas_height,
            base_scale: scale,
            base_rotate: rotate,
            keyframes: None,

            texture: None,
            offset: ImageClipOffset::default(),
            image_width: 0.0,
            image_height: 0.0,
        }
    }

    pub fn set_offset(&mut self, x: f32, y: f32) {
        self.offset.x = x;
        self.offset.y = y;
    }

    pub fn set_keyframes(&mut self, keyframes: Vec<Keyframe>) {
        self.keyframes = Some(keyframes);
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        info!("loading image from {}", self.url);
        let data = load_image_from_src(&self.url)?;

        self.image_width = data.width() as f32;
        self.image_height = data.height() as f32;

        let unit = (gl::TEXTURE0 + self.texture_idx) as GLenum;
        let texture = Texture::new(gl::TEXTURE_2D, unit);
        texture.load_from_image(data)?;
        texture.set_wrapping(gl::REPEAT);
        texture.set_filtering(gl::LINEAR);
        texture.bind_unit();

        self.texture = Some(texture);

        Ok(())
    }

    fn build_transformer(&self, local_time: f32) -> Transformer {
        let mut transformer = Transformer::default();

        if let Some(ref keyframes) = self.keyframes {
            let kf = interpolate_keyframes(keyframes, local_time);
            transformer.set_scale(self.base_scale * kf.scale);
            transformer.set_rotate(self.base_rotate + kf.rotate);
            transformer.set_translation(self.offset.x + kf.x, self.offset.y + kf.y, 0.0);
            transformer.set_skew(kf.skew_x, kf.skew_y);
            transformer.set_flip(kf.flip_x, kf.flip_y);
            transformer.set_opacity(kf.opacity);
        } else {
            transformer.set_scale(self.base_scale);
            transformer.set_rotate(self.base_rotate);
            transformer.set_translation(self.offset.x, self.offset.y, 0.0);
            transformer.set_opacity(1.0);
        }

        transformer
    }

    pub fn quad(&self) -> Quad {
        self.quad_at_time(0.0)
    }

    pub fn quad_at_time(&self, local_time: f32) -> Quad {
        let transformer = self.build_transformer(local_time);
        let idx = self.texture_idx as f32;

        let (x0, y0) = (0.0, 0.0);
        let (x1, y1) = (self.image_width, self.image_height);

        let mid_x = x0 + (x1 - x0) / 2.0;
        let mid_y = y0 + (y1 - y0) / 2.0;

        let p0 = transformer.apply_similarity(
            x0,
            y0,
            1.0,
            mid_x,
            mid_y,
            self.canvas_width,
            self.canvas_height,
        );
        let p1 = transformer.apply_similarity(
            x1,
            y0,
            1.0,
            mid_x,
            mid_y,
            self.canvas_width,
            self.canvas_height,
        );
        let p2 = transformer.apply_similarity(
            x1,
            y1,
            1.0,
            mid_x,
            mid_y,
            self.canvas_width,
            self.canvas_height,
        );
        let p3 = transformer.apply_similarity(
            x0,
            y1,
            1.0,
            mid_x,
            mid_y,
            self.canvas_width,
            self.canvas_height,
        );

        let (p0_x, p0_y) = (p0.0, p0.1);
        let (p1_x, p1_y) = (p1.0, p1.1);
        let (p2_x, p2_y) = (p2.0, p2.1);
        let (p3_x, p3_y) = (p3.0, p3.1);

        Quad([
            Vertex([p0_x, p0_y], [0.0, 0.0], idx),
            Vertex([p1_x, p1_y], [1.0, 0.0], idx),
            Vertex([p2_x, p2_y], [1.0, 1.0], idx),
            Vertex([p3_x, p3_y], [0.0, 1.0], idx),
        ])
    }

    pub fn opacity_at_time(&self, local_time: f32) -> f32 {
        let transformer = self.build_transformer(local_time);
        transformer.get_opacity()
    }

    pub fn indices(&self) -> Indices {
        let idx = self.texture_idx as usize;
        let offset = (idx * VERTEX_PER_QUAD) as i32;
        Indices([
            offset,
            offset + 1,
            offset + 2,
            offset + 2,
            offset + 3,
            offset,
        ])
    }

    pub fn into_gl_texture(self) -> Texture {
        self.texture.unwrap()
    }

    pub fn texture(&self) -> Option<Texture> {
        self.texture.clone()
    }
}

fn load_image_from_src(src: &str) -> anyhow::Result<image_crate::DynamicImage> {
    if src.starts_with("http://") || src.starts_with("https://") {
        let img_bytes = reqwest::blocking::get(src)?.bytes()?;
        Ok(image_crate::load_from_memory(&img_bytes)?)
    } else if src.starts_with("file://") {
        let path = src.strip_prefix("file://").unwrap();
        Ok(image_crate::open(path)?)
    } else {
        // Treat as local relative or absolute path
        Ok(image_crate::open(src)?)
    }
}
