use std::fmt;
use std::fmt::Formatter;

use serde::Deserialize;

use crate::asset::{ImageAsset, MediaAsset};
use crate::asset::VideoAsset;
use crate::asset::shape::{ShapeAsset, ShapeAssetSpec};

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub enum AssetType {
    #[serde(rename = "image")]
    Image,

    #[serde(rename = "video")]
    Video,

    #[serde(rename = "shape")]
    Shape,
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let printable = match *self {
            AssetType::Image => "image",
            AssetType::Video => "video",
            AssetType::Shape => "shape",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Asset {
    #[serde(rename = "image")]
    Image {
        src: String,
    },
    #[serde(rename = "video")]
    Video {
        src: String,
    },
    #[serde(rename = "shape")]
    Shape(ShapeAssetSpec),
}

impl Asset {
    pub fn asset_type(&self) -> AssetType {
        match self {
            Asset::Image { .. } => AssetType::Image,
            Asset::Video { .. } => AssetType::Video,
            Asset::Shape(_) => AssetType::Shape,
        }
    }

    pub fn src(&self) -> Option<&str> {
        match self {
            Asset::Image { src } => Some(src),
            Asset::Video { src } => Some(src),
            Asset::Shape(_) => None,
        }
    }

    pub fn shape_spec(&self) -> Option<&ShapeAssetSpec> {
        match self {
            Asset::Shape(spec) => Some(spec),
            _ => None,
        }
    }

    pub fn into_image_asset(self) -> anyhow::Result<ImageAsset> {
        match self {
            Asset::Image { src } => Ok(ImageAsset {
                src,
                data: image::DynamicImage::default(),
            }),
            _ => anyhow::bail!("Asset is not an image"),
        }
    }

    pub fn into_video_asset(self) -> anyhow::Result<VideoAsset> {
        match self {
            Asset::Video { src } => Ok(VideoAsset { src }),
            _ => anyhow::bail!("Asset is not a video"),
        }
    }

    pub fn into_shape_asset(self) -> anyhow::Result<ShapeAsset> {
        match self {
            Asset::Shape(spec) => Ok(ShapeAsset {
                spec,
                data: image::DynamicImage::default(),
            }),
            _ => anyhow::bail!("Asset is not a shape"),
        }
    }

    pub fn into_media(self) -> Box<dyn MediaAsset> {
        match self {
            Asset::Image { .. } => Box::new(self.into_image_asset().unwrap()),
            Asset::Video { .. } => Box::new(self.into_video_asset().unwrap()),
            Asset::Shape(_) => Box::new(self.into_shape_asset().unwrap()),
        }
    }
}
