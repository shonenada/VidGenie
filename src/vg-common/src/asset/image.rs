extern crate image as image_crate;
use serde::Deserialize;

use crate::asset::asset::AssetType;
use crate::asset::MediaAsset;

pub struct ImageAsset {
    pub(crate) src: String,
    pub(crate) data: image_crate::DynamicImage,
}

impl MediaAsset for ImageAsset {
    fn load(&mut self) -> anyhow::Result<()> {
        let url = self.src.clone();
        let img_bytes = reqwest::blocking::get(url)?.bytes()?;
        self.data = image_crate::load_from_memory(&img_bytes)?;

        Ok(())
    }
}