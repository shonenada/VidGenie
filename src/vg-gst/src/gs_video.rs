use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};

use gst::prelude::*;

#[derive(Debug)]
pub enum FrameType {
    DATA,
    EOS,
}

#[derive(Debug)]
pub struct Frame {
    data: Vec<u8>,
    frame_num: u64,
    frame_type: FrameType,
}

impl Frame {
    pub fn new(data: Vec<u8>, frame_num: u64) -> Self {
        Self {
            data,
            frame_num,
            frame_type: FrameType::DATA,
        }
    }

    pub fn eos() -> Self {
        Self {
            data: Vec::default(),
            frame_num: 0,
            frame_type: FrameType::EOS,
        }
    }
}

unsafe impl Send for Frame {}

unsafe impl Sync for Frame {}

#[derive(Clone)]
pub struct GSVideo {
    width: u32,
    height: u32,
    fps: gst::Fraction,
    output_path: String,

    appsrc: gst_app::AppSrc,
    pipeline: gst::Pipeline,
    bus: Option<gst::Bus>,
}

impl GSVideo {
    pub fn new(width: u32, height: u32, output_path: String) -> anyhow::Result<Self> {
        let fps = gst::Fraction::new(30, 1);

        let caps = gst::Caps::builder("video/x-raw")
            .field("format", &gst_video::VideoFormat::Rgb.to_string())
            .field("width", &(width as i32))
            .field("height", &(height as i32))
            .field("framerate", fps)
            .build();

        let appsrc = gst_app::AppSrc::builder()
            .caps(&caps)
            .format(gst::Format::Time)
            .build();

        let rt = Self {
            width,
            height,
            fps,
            appsrc,
            output_path,
            bus: None,
            pipeline: gst::Pipeline::default(),
        };
        rt.setup_pipeline(&rt.output_path)?;

        Ok(rt)
    }

    pub fn setup_pipeline(&self, output_location: &str) -> anyhow::Result<()> {
        let video_conv = gst::ElementFactory::make("videoconvert").build()?;
        let video_enc = gst::ElementFactory::make("x264enc").build()?;
        let video_parse = gst::ElementFactory::make("h264parse")
            .property_from_str("config-interval", &"3")
            .build()?;
        let qtmux = gst::ElementFactory::make("qtmux").build()?;
        let filesink = gst::ElementFactory::make("filesink")
            .property_from_str("location", output_location)
            .build()?;

        let links = [
            &(self.appsrc.upcast_ref()),
            &video_conv,
            &video_enc,
            &video_parse,
            &qtmux,
            &filesink,
        ];

        self.pipeline.add_many(&links)?;
        gst::Element::link_many(&links)?;

        Ok(())
    }

    pub fn setup_appsrc(&self, rx: Arc<Mutex<Receiver<Frame>>>) -> anyhow::Result<()> {
        let fps = self.fps.numer();
        self.appsrc.set_callbacks(
            gst_app::AppSrcCallbacks::builder()
                .need_data(move |appsrc, _| {
                    if let Ok(frame) = rx.lock().unwrap().recv() {
                        match frame.frame_type {
                            FrameType::EOS => {
                                appsrc.end_of_stream().unwrap();
                            }
                            FrameType::DATA => {
                                let pixels = frame.data;
                                let frame_num = frame.frame_num;
                                let mut buffer = gst::Buffer::with_size(pixels.len()).unwrap();
                                {
                                    let buffer = buffer.get_mut().unwrap();
                                    let fps = (1_000 / fps) as u64;
                                    buffer.set_pts(frame_num * fps * gst::ClockTime::MSECOND);
                                    buffer.copy_from_slice(0, &pixels[..]).unwrap();
                                }
                                appsrc.push_buffer(buffer).unwrap();
                            }
                        }
                    } else {
                        // TODO: handle recv() failed?
                        appsrc.end_of_stream().unwrap();
                    }
                })
                .build(),
        );

        Ok(())
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.pipeline.set_state(gst::State::Playing)?;
        self.bus = self.pipeline.bus();

        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.pipeline.set_state(gst::State::Null)?;

        Ok(())
    }

    pub fn handle_message(&self) -> anyhow::Result<()> {
        use gst::MessageView;

        if let Some(bus) = &self.bus {
            for msg in bus.iter_timed(gst::ClockTime::NONE) {
                match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(err) => {
                        self.pipeline.set_state(gst::State::Null)?;
                        eprintln!("{:?}", err);
                    }
                    _ => (),
                }
            }
        } else {
            eprintln!("Pipeline without bus. Shouldn't happen!");
        }

        Ok(())
    }
}
