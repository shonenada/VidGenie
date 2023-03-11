use serde::Deserialize;

use crate::asset::Asset;
use crate::clip::Clip;

#[derive(Debug, Deserialize)]
pub struct RenderOutput {
    pub format: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Position {
    #[serde(rename = "center")]
    Center,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub clips: Vec<Clip>,
}

#[derive(Debug, Deserialize)]
pub struct Timeline {
    pub background: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    pub output: RenderOutput,
    pub timeline: Timeline,
}
