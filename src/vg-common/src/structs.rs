use colors_transform::Rgb;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

use crate::clip::Clip;

pub enum ParseError {
    Message(String),
}

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

fn from_color_hex<'de, D>(deserializer: D) -> Result<Rgb, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Rgb::from_hex_str(s).map_err(|_| D::Error::custom("failed"))
}

#[derive(Debug, Deserialize)]
pub struct Timeline {
    #[serde(deserialize_with = "from_color_hex")]
    pub background: Rgb,

    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    pub output: RenderOutput,
    pub timeline: Timeline,
}
