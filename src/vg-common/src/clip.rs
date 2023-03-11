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
}

impl Into<VideoClip> for Clip {
    fn into(self) -> VideoClip {
        VideoClip{
            asset: self.asset.into(),
        }
    }
}

pub struct VideoClip {
    asset: Box<dyn MediaAsset>,
}

impl VideoClip {
    pub fn load_asset(&mut self) {
        self.asset.load().expect("Failed to load asset");
    }
}
