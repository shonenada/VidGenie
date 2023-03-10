use serde::Deserialize;

use crate::asset::asset::AssetType;
use crate::asset::MediaAsset;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct VideoAsset {
    #[serde(rename = "type")]
    pub(crate) src: String,
}

impl MediaAsset for VideoAsset {
    fn load(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}
