pub mod asset;
mod image;
pub mod shape;
mod video;

pub use self::image::ImageAsset;
pub use asset::{Asset, AssetType};
pub use shape::{ShapeAsset, ShapeAssetSpec};
pub use video::VideoAsset;

pub trait MediaAsset {
    fn load(&mut self) -> anyhow::Result<()>;
}
