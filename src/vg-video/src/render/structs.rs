extern crate image as image_crate;

use gl::types::GLenum;
use log::{debug, info};

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
    canvas_half_width: f32,
    canvas_half_height: f32,
    image_width: f32,
    image_height: f32,
    offset: ImageClipOffset,
    transformer: Transformer,
}

impl ImageClipTexture {
    pub fn new(url: &str, canvas_width: f32, canvas_height: f32, idx: u32, scale: f32, rotate: f32) -> Self {
        let mut transformer = Transformer::default();
        transformer.set_scale(scale);
        transformer.set_rotate(rotate);

        let canvas_half_width = canvas_width / 2.0;
        let canvas_half_height = canvas_height / 2.0;

        Self {
            url: url.to_string(),
            texture_idx: idx,
            canvas_half_width,
            canvas_half_height,
            transformer,

            texture: None,
            offset: ImageClipOffset::default(),
            image_width: 0.0,
            image_height: 0.0,
        }
    }

    pub fn set_offset(&mut self, x: f32, y: f32) {
        self.offset.x = x;
        self.offset.y = y;
        self.transformer.set_translation(
            self.offset.x / self.canvas_half_width,
            self.offset.y / self.canvas_half_height,
            0.0,
        );
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        info!("start download from {}", self.url);
        let img_bytes = reqwest::blocking::get(&self.url)?.bytes()?;
        let data = image_crate::load_from_memory(&img_bytes)?;

        self.image_width = data.width() as f32;
        self.image_height = data.height() as f32;

        let unit = (gl::TEXTURE0 + self.texture_idx) as GLenum;
        let texture = Texture::new(unit);
        texture.set_wrapping(gl::REPEAT);
        texture.set_filtering(gl::LINE_LOOP);
        texture.load_from_image(data)?;
        texture.bind_unit();

        self.texture = Some(texture);

        Ok(())
    }

    pub fn quad(&self) -> Quad {
        let idx = self.texture_idx as f32;

        let (x0, y0) = (0.0, 0.0);
        let (x1, y1) = (self.image_width / self.canvas_half_width, self.image_height / self.canvas_half_height);

        let p0 = self.transformer.apply_similarity(x0, y0, 1.0);
        let p2 = self.transformer.apply_similarity(x1, y1, 1.0);

        let (p0_x, p0_y) = (p0.0 - 1.0, p0.1 - 1.0);
        let (p2_x, p2_y) = (p2.0 - 1.0, p2.1 - 1.0);

        Quad([
            Vertex([p0_x, p0_y], [0.0, 0.0], idx),
            Vertex([p2_x, p0_y], [1.0, 0.0], idx),
            Vertex([p2_x, p2_y], [1.0, 1.0], idx),
            Vertex([p0_x, p2_y], [0.0, 1.0], idx),
        ])
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
}
