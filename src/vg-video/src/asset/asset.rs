use std::fmt;
use std::fmt::Formatter;

use serde::Deserialize;

use crate::asset::{ImageAsset, MediaAsset};
use crate::asset::VideoAsset;

#[derive(Debug, PartialEq, Deserialize)]
pub enum AssetType {
    #[serde(rename = "image")]
    Image,

    #[serde(rename = "video")]
    Video,
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let printable = match *self {
            AssetType::Image => "image",
            AssetType::Video => "video",
        };
        write!(f, "{}", printable)
    }
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Asset {
    #[serde(rename = "type")]
    pub asset_type: AssetType,
    pub src: String,
}

impl Asset {
    pub fn into_image_asset(self) -> anyhow::Result<ImageAsset> {
        Ok(ImageAsset {
            src: self.src,
            data: image::DynamicImage::default(),
        })
    }

    pub fn into_video_asset(self) -> anyhow::Result<VideoAsset> {
        Ok(VideoAsset { src: self.src })
    }

    pub fn into(self) -> Box<dyn MediaAsset> {
        match self.asset_type {
            AssetType::Image => Box::new(self.into_image_asset().unwrap()),
            AssetType::Video => Box::new(self.into_video_asset().unwrap()),
        }
    }
}
