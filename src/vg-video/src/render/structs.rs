extern crate image as image_crate;

use gl::types::GLenum;
use log::{debug, info};

use vg_gl::{Quad, Texture, Vertex};

pub struct ImageClipOffset {
    x: u32,
    y: u32,
}

impl Default for ImageClipOffset {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

pub struct ImageClipTexture {
    url: String,
    texture_unit: GLenum,
    texture: Option<Texture>,

    image_width: u32,
    image_height: u32,
    offset: ImageClipOffset,
}

impl ImageClipTexture {
    pub fn new(url: &str, texture_unit: GLenum) -> Self {
        Self {
            url: url.to_string(),
            texture_unit,
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
        let url = self.url.clone();
        info!("start download from {}", url);
        let img_bytes = reqwest::blocking::get(url)?.bytes()?;
        let data = image_crate::load_from_memory(&img_bytes)?;

        self.image_width = data.width();
        self.image_height = data.height();

        let texture = Texture::new(self.texture_unit);
        texture.set_wrapping(gl::REPEAT);
        texture.set_filtering(gl::LINE_LOOP);
        texture.load_from_image(data)?;
        texture.bind_unit();

        self.texture = Some(texture);

        Ok(())
    }

    pub fn quad(&self, window_width: f32, window_height: f32) -> Quad {
        let (x0, y0) = get_coord(window_width, window_height, self.offset.x as f32, self.offset.y as f32);
        let (x1, y1) = get_coord(window_width, window_height, (self.offset.x + self.image_width) as f32, (self.offset.y + self.image_height) as f32);

        debug!("w, h = ({}, {}); iw, ih = ({}, {});", window_width, window_height, self.image_width, self.image_height);
        debug!("p0 = ({}, {}); p1 = ({}, {});", x0, y0, x1, y1);

        Quad([
            Vertex([x0, y0], [0.0, 1.0]),
            Vertex([x1, y0], [1.0, 1.0]),
            Vertex([x1, y1], [1.0, 0.0]),
            Vertex([x0, y1], [0.0, 0.0]),
        ])
    }

    pub fn into_gl_texture(self) -> Texture {
        self.texture.unwrap()
    }
}

fn get_coord(width: f32, height: f32, x: f32, y: f32) -> (f32, f32) {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    debug!("ix, iy = {}, {}", x, y);

    let x0: f32 = if x < half_width {
        ((half_width - x) / half_width) * -1.0
    } else {
        (x - half_width) / half_width
    };

    let y0: f32 = if y < half_height {
        ((half_height - y) / half_height) * -1.0
    } else {
        (y - half_height) / half_height
    };

    (x0, y0)
}
