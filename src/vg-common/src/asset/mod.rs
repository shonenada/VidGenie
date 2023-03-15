pub mod asset;
mod image;
mod video;

pub use self::image::ImageAsset;
pub use asset::Asset;
pub use video::VideoAsset;

pub trait MediaAsset {
    fn load(&mut self) -> anyhow::Result<()>;
}
