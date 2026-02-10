pub use render::structs::ImageClipTexture;
pub use render::shape_texture::ShapeClipTexture;
pub use request::structs::RenderRequest;
pub use asset::asset::{Asset, AssetType};
pub use asset::shape::{ShapeAssetSpec, ShapeKind};
pub use request::clip::{Transition, TransitionType};
pub use request::transform::{Transform, TransformPreset, Keyframe, PresetConfig, interpolate_keyframes};
pub use vg_gst::Frame;
pub use video_encoder::VideoEncoder;
pub use request::caption::CaptionConfig;
pub use overlay::caption_overlay::CaptionOverlay;

mod video_encoder;
pub mod asset;
mod render;
mod request;
pub mod text;
pub mod overlay;
