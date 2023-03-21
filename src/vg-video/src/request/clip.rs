use serde::Deserialize;

use vg_gst::Frame;

use crate::asset::{Asset, MediaAsset};

#[derive(Debug, Deserialize)]
pub struct ClipOffset {
    x: u32,
    y: u32,
}

#[derive(Debug, Deserialize)]
pub struct Clip {
    pub asset: Asset,
    pub start: f32,
    pub length: f32,
    pub offset: ClipOffset,
    pub position: String,
}

impl Into<VideoClip> for Clip {
    fn into(self) -> VideoClip {
        VideoClip {
            asset: self.asset.into(),
            offset: self.offset,
            position: self.position,

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
