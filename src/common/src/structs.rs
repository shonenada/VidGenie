
use strum_macros::EnumString;
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct RenderOutput {
    format: String,
    width: u32,
    height: u32,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct Asset {
    #[serde(rename = "type")]
    asset_type: String,
    src: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct ClipOffset {
    x: u32,
    y: u32,
}

#[allow(unused)]
#[derive(Debug, Deserialize, EnumString)]
pub enum Position {
    #[strum(serialize="center")]
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
    output: RenderOutput,
    timeline: Timeline,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_from_str() {
        let position = Position::from_str("center");
        assert_eq!(position, Position::Center);
    }
}
