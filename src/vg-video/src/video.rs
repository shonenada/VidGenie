use std::sync::mpsc::{channel, Receiver, Sender};

use gst::prelude::*;

pub struct Frame {
    data: Vec<u8>,
    frame_num: u32,
}

pub struct GSVideo {
    width: u32,
    height: u32,
    fps: gst::Fraction,
    output_path: &str,

    appsrc: gst_app::AppSrc,
    pipeline: gst::Pipeline,
    bus: Option<gst::Bus>,

    tx: Sender<Frame>,
    rx: Receiver<Frame>,
}

impl GSVideo {

    pub fn new(width: u32, height: u32, output_path: &str) -> anyhow::Result<Self> {
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

        let (tx, rx) = channel();

        let rt = Self {
            width,
            height,
            output_path,
            fps,
            tx,
            rx,
            appsrc,
            bus: None,
            pipeline: gst::Pipeline::default(),
        };
        rt.setup_pipeline(output_path)?;
        rt.setup_appsrc()?;

        Ok(self_)
    }

    fn setup_pipeline(&self, output_location: &str) -> anyhow::Result<()> {
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

    fn setup_appsrc(&self) -> anyhow::Result<()> {
        self.appsrc.set_callbacks(
            gst_app::AppSrcCallbacks::builder()
                .need_data(move |appsrc, _| {
                    if let Ok(frame) = self.rx.recv() {
                        let pixels = frame.data;
                        let frame_num = frame.frame_num;
                        let mut buffer = gst::Buffer::with_size(pixels.len()).unwrap();
                        {
                            let buffer = buffer.get_mut().unwrap();
                            let fps = (1_000 / FPS) as u64;
                            buffer.set_pts(frame_num * fps * gst::ClockTime::MSECOND);
                            buffer.copy_from_slice(0, &pixels[..]).unwrap();
                        }
                        appsrc.push_buffer(buffer).unwrap();
                    } else {
                        appsrc.end_of_stream().unwrap();
                    }
                })
                .build()
        );

        Ok(())
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.pipeline.set_state(gst::State::Playing)?;
        self.bus = self.pipeline.bus();

        Ok(())
    }

    pub fn sttp(&mut self) -> anyhow::Result<()> {
        self.pipeline.set_state(gst::State::Null)?;

        Ok(())
    }

    fn handle_message(&self) -> anyhow::Result<()> {
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

    pub fn send_frame(&self, frame: Frame) -> anyhow::Result<()> {
        self.tx.send(frame)?;
        Ok(())
    }

}