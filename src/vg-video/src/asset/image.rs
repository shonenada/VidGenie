extern crate image as image_crate;

use crate::asset::MediaAsset;

pub struct ImageAsset {
    pub(crate) src: String,
    pub(crate) data: image_crate::DynamicImage,
}

impl MediaAsset for ImageAsset {
    fn load(&mut self) -> anyhow::Result<()> {
        self.data = load_image_from_src(&self.src)?;
        Ok(())
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
