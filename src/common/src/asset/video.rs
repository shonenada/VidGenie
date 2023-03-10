use serde::Deserialize;

use crate::asset::asset::AssetType;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct VideoAsset {
    #[serde(rename = "type")]
    pub(crate) src: String,
}

impl VideoAsset {}