use serde::Deserialize;

use crate::asset::Asset;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct RenderOutput {
    format: String,
    width: u32,
    height: u32,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct ClipOffset {
    x: u32,
    y: u32,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Position {
    #[serde(rename = "center")]
    Center,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Clip {
    asset: Asset,
    start: u32,
    length: u32,
    offset: ClipOffset,
    position: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Track {
    clips: Vec<Clip>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Timeline {
    background: String,
    tracks: Vec<Track>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    pub output: RenderOutput,
    pub timeline: Timeline,
}
