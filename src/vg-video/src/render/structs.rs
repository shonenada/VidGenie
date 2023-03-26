extern crate image as image_crate;

use gl::types::GLenum;
use log::{debug, info};

use vg_gl::{Indices, Quad, Texture, Transformer, Vertex, VERTEX_PER_QUAD};

#[derive(Default)]
pub struct ImageClipOffset {
    x: u32,
    y: u32,
}

pub struct ImageClipTexture {
    url: String,
    texture_idx: u32,
    scale: f32,

    texture: Option<Texture>,
    image_width: u32,
    image_height: u32,
    offset: ImageClipOffset,
    transformer: Transformer,
}

impl ImageClipTexture {
    pub fn new(url: &str, idx: u32, scale: f32) -> Self {
        let mut transformer = Transformer::default();
        transformer.set_scale(scale);
        Self {
            url: url.to_string(),
            texture_idx: idx,
            scale,

            transformer,
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
        let idx = self.texture_idx as f32;
        let (x0, y0) = (0.0, 0.0);
        let (x1, y1) = (self.image_width as f32 / width * 2.0, self.image_height as f32 / height * 2.0);

        // let (x0, y0) = get_coord(width, height, x0_, y0_);
        // let (x1, y1) = get_coord(width, height, x1_, y1_);

        let p0 = self.transformer.apply_transform(x0, y0, 1.0);
        let p2 = self.transformer.apply_transform(x1, y1, 1.0);

        let ret = Quad([
            Vertex([p0.0, p0.1], [0.0, 0.0], idx),
            Vertex([p2.0, p0.1], [1.0, 0.0], idx),
            Vertex([p2.0, p2.1], [1.0, 1.0], idx),
            Vertex([p0.0, p2.1], [0.0, 1.0], idx),
        ]);
        debug!("Ret: {:?}", ret);
        ret
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
