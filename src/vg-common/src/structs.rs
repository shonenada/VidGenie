use serde::Deserialize;

use crate::asset::Asset;
use crate::clip::Clip;

#[derive(Debug, Deserialize)]
pub struct RenderOutput {
    format: String,
    width: u32,
    height: u32,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Position {
    #[serde(rename = "center")]
    Center,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    clips: Vec<Clip>,
}

#[derive(Debug, Deserialize)]
pub struct Timeline {
    background: String,
    tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    pub output: RenderOutput,
    pub timeline: Timeline,
}
