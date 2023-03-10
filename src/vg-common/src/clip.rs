use serde::Deserialize;

use crate::asset::{Asset, MediaAsset};

#[derive(Debug, Deserialize)]
pub struct ClipOffset {
    x: u32,
    y: u32,
}

#[derive(Debug, Deserialize)]
pub struct Clip {
    asset: Asset,
    start: u32,
    length: u32,
    offset: ClipOffset,
    position: String,

    #[serde(default)]
    inner_asset: Box<dyn MediaAsset>,
}

impl Clip {
    pub fn load_asset(&mut self) {
        self.inner_asset = self.asset.into();
        self.inner_asset.load().expect("Failed to load asset");
    }
}
