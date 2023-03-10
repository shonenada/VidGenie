use std::fmt;
use std::fmt::Formatter;

use anyhow::anyhow;
use image::DynamicImage;
use serde::Deserialize;

use crate::asset::ImageAsset;
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
    asset_type: AssetType,
    src: String,
}

impl Asset {
    pub fn into_image_asset(self) -> anyhow::Result<ImageAsset> {
        if self.asset_type != AssetType::Image {
            Err(anyhow!(format!("failed to convert type {} to ImageAsset", self.asset_type)))
        } else {
            Ok(ImageAsset {
                src: self.src,
                data: image::DynamicImage::default(),
            })
        }
    }

    pub fn into_video_asset(self) -> anyhow::Result<VideoAsset> {
        if self.asset_type != AssetType::Video {
            Err(anyhow!(format!("failed to convert type {} to VideoAsset", self.asset_type)))
        } else {
            Ok(VideoAsset{
                src: self.src,
            })
        }
    }

}
