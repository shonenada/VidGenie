use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use anyhow::Result;

use vg_gst::{Frame, GSVideo};

pub struct VideoEncoder {
    gs_video: GSVideo,

    render_thread: Option<JoinHandle<()>>,

    tx: Arc<Sender<Frame>>,
    rx: Arc<Mutex<Receiver<Frame>>>,
}

impl VideoEncoder {
    pub fn builder() -> VideoEncoderBuilder {
        VideoEncoderBuilder::default()
    }

    pub fn start_gst_render(&mut self) -> Result<()> {
        self.gs_video.start()?;
        self.gs_video.handle_message()?;
        self.gs_video.stop()?;
        Ok(())
    }

    pub fn send_frame(&self, frame: Frame) -> Result<()> {
        self.tx.send(frame)?;
        Ok(())
    }

    pub fn finish(&self) -> Result<()> {
        self.tx.send(Frame::eos())?;
        Ok(())
    }

    pub fn start_render(&mut self) -> Result<()> {
        fn gst_render(rx: Arc<Mutex<Receiver<Frame>>>, mut gs_video: GSVideo) -> Result<()> {
            gs_video.setup_appsrc(rx)?;
            gs_video.start()?;
            gs_video.handle_message()?;
            gs_video.stop()?;
            Ok(())
        }
        let gs_video = self.gs_video.to_owned();
        let rx = self.rx.to_owned();
        let t = thread::spawn(move || {
            gst_render(rx, gs_video).unwrap();
        });
        self.render_thread = Some(t);
        Ok(())
    }

    pub fn until_rendered(&mut self) {
        self.render_thread.take().unwrap().join().unwrap();
    }
}

pub struct VideoEncoderBuilder {
    width: u32,
    height: u32,
    output_path: String,
}

impl VideoEncoderBuilder {
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

    pub fn build(self) -> anyhow::Result<VideoEncoder> {
        let gs_video = GSVideo::new(self.width, self.height, self.output_path)?;
        let (tx, rx) = channel();
        Ok(VideoEncoder {
            gs_video,
            render_thread: None,
            tx: Arc::new(tx),
            rx: Arc::new(Mutex::new(rx)),
        })
    }
}

impl Default for VideoEncoderBuilder {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            output_path: "/tmp/output.mp4".to_string(),
        }
    }
}
