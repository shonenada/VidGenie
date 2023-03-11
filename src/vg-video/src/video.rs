use anyhow::Result;

use vg_gst::GSVideo;

pub struct Video {
    gs_video: GSVideo,
}

impl Video {
    pub fn builder() -> VideoBuilder {
        VideoBuilder::default()
    }
}

pub struct VideoBuilder {
    width: u32,
    height: u32,
    output_path: String,
}

impl VideoBuilder {
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn output_path(mut self, output: &str) -> Self {
        self.output_path = output.to_string();
        self
    }

    pub fn build(self) -> anyhow::Result<Video> {
        let gs_video = GSVideo::new(self.width, self.height, self.output_path)?;
        Ok(Video { gs_video })
    }
}

impl Default for VideoBuilder {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            output_path: "/tmp/output.mp4".to_string(),
        }
    }
}
