use serde::Deserialize;

use vg_gst::Frame;

use crate::asset::{Asset, MediaAsset};

#[derive(Debug, Deserialize)]
pub struct ClipOffset {
    pub x: u32,
    pub y: u32,
}

fn default_scale() -> f32 { 1.0 }

#[derive(Debug, Deserialize)]
pub struct Clip {
    pub asset: Asset,
    pub start: f32,
    pub length: f32,
    pub offset: ClipOffset,
    #[serde(default = "default_scale")]
    pub scale: f32,
    pub position: String,
}

impl Into<VideoClip> for Clip {
    fn into(self) -> VideoClip {
        VideoClip {
            asset: self.asset.into(),
            offset: self.offset,
            position: self.position,
            scale: self.scale,

            frame_start: 0,
            frame_end: 0,
            cur_frame: 0,
        }
    }
}

pub struct VideoClip {
    cur_frame: u64,
    frame_start: u64,
    frame_end: u64,
    offset: ClipOffset,
    position: String,
    scale: f32,
    asset: Box<dyn MediaAsset>,
}

impl VideoClip {
    pub fn load_asset(&mut self) {
        self.asset.load().expect("Failed to load asset");
    }

    pub fn next_frame(&self) -> Option<Frame> {
        if self.cur_frame > self.frame_end || self.cur_frame < self.frame_start {
            None
        } else {
            let frame = Frame::new(Vec::new(), self.cur_frame);
            Some(frame)
        }
    }
}
