extern crate image as image_crate;

use gl::types::GLenum;
use log::info;

use vg_gl::Texture;

pub struct ImageClipTexture {
    url: String,
    texture_unit: GLenum,
    texture: Option<Texture>,
}

impl ImageClipTexture {
    pub fn new(url: &str, texture_unit: GLenum) -> Self {
        Self {
            url: url.to_string(),
            texture_unit,
            texture: None,
        }
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        let url = self.url.clone();
        info!("start download from {}", url);
        let img_bytes = reqwest::blocking::get(url)?.bytes()?;
        let data = image_crate::load_from_memory(&img_bytes)?;

        let texture = Texture::new(self.texture_unit);
        texture.set_wrapping(gl::REPEAT);
        texture.set_filtering(gl::LINE_LOOP);
        texture.load_from_image(data)?;
        texture.bind_unit();

        self.texture = Some(texture);

        Ok(())
    }

    pub fn into_gl_texture(self) -> Texture {
        self.texture.unwrap()
    }
}