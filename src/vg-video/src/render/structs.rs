extern crate image as image_crate;

use gl::types::GLenum;
use log::info;

use vg_gl::{Indices, Quad, Texture, Vertex, VERTEX_PER_QUAD};

#[derive(Default)]
pub struct ImageClipOffset {
    x: u32,
    y: u32,
}

pub struct ImageClipTexture {
    url: String,
    texture_idx: u32,

    texture: Option<Texture>,
    image_width: u32,
    image_height: u32,
    offset: ImageClipOffset,
}

impl ImageClipTexture {
    pub fn new(url: &str, idx: u32) -> Self {
        Self {
            url: url.to_string(),
            texture_idx: idx,

            texture: None,
            offset: ImageClipOffset::default(),
            image_width: 0,
            image_height: 0,
        }
    }

    pub fn set_x(&mut self, x: u32) {
        self.offset.x = x;
    }

    pub fn set_y(&mut self, y: u32) {
        self.offset.y = y;
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        info!("start download from {}", self.url);
        let img_bytes = reqwest::blocking::get(&self.url)?.bytes()?;
        let data = image_crate::load_from_memory(&img_bytes)?;

        self.image_width = data.width();
        self.image_height = data.height();

        let unit = (gl::TEXTURE0 + self.texture_idx) as GLenum;
        let texture = Texture::new(unit);
        texture.set_wrapping(gl::REPEAT);
        texture.set_filtering(gl::LINE_LOOP);
        texture.load_from_image(data)?;
        texture.bind_unit();

        self.texture = Some(texture);

        Ok(())
    }

    pub fn quad(&self, width: f32, height: f32) -> Quad {
        let x0_ = self.offset.x as f32;
        let y0_ = self.offset.y as f32;
        let x1_ = x0_ + self.image_width as f32;
        let y1_ = y0_ + self.image_height as f32;

        let (x0, y0) = get_coord(width, height, x0_, y0_);
        let (x1, y1) = get_coord(width, height, x1_, y1_);

        let idx = self.texture_idx as f32;
        Quad([
            Vertex([x0, y0], [0.0, 1.0], idx),
            Vertex([x1, y0], [1.0, 1.0], idx),
            Vertex([x1, y1], [1.0, 0.0], idx),
            Vertex([x0, y1], [0.0, 0.0], idx),
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

fn get_coord(width: f32, height: f32, x: f32, y: f32) -> (f32, f32) {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let x_: f32 = if x < half_width {
        ((half_width - x) / half_width) * -1.0
    } else {
        (x - half_width) / half_width
    };

    let y_: f32 = if y < half_height {
        ((half_height - y) / half_height) * -1.0
    } else {
        (y - half_height) / half_height
    };

    (x_, y_)
}
