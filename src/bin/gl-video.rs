extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;

#[path = "../common.rs"]
mod common;

use anyhow::{anyhow, Result};
use gst::prelude::*;

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;

struct Video {
    appsrc: gst_app::AppSrc,
    video_info: gst_video::VideoInfo,
    pipeline: gst::Pipeline,
    bus: Option<gst::Bus>,
}

impl Video {

    fn new(width: u32, height: u32, fps: gst::Fraction) -> Result<Self> {

        let video_info = gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgba, width as u32, height as u32)
            .fps(fps)
            .build()?;

        let caps = gst::Caps::builder("video/x-raw")
            .field("format", &gst_video::VideoFormat::Rgba.to_string())
            .field("width", &(width as i32))
            .field("height", &(height as i32))
            .field("framerate", fps)
            .build();

        let appsrc = gst_app::AppSrc::builder()
            .caps(&caps)
            .format(gst::Format::Time)
            .build();

        Ok(Self {
            appsrc,
            video_info,
            bus: None,
            pipeline: gst::Pipeline::default(),
        })
    }

    fn setup_pipeline(&self, output_location: &str) -> Result<()> {
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

    fn setup_appsrc(&self) -> Result<()> {
        let mut frame_num = 0;
        let video_info = self.video_info.clone();

        self.appsrc.set_callbacks(
            gst_app::AppSrcCallbacks::builder()
                .need_data(move |appsrc, _| {
                     if frame_num == 10 {
                        appsrc.end_of_stream().unwrap();
                        return;
                    }

                    println!("Producing frame {}", frame_num);

                    let r = if frame_num % 2 == 0 { 0 } else { 255 };
                    let g = if frame_num % 3 == 0 { 0 } else { 255 };
                    let b = if frame_num % 5 == 0 { 0 } else { 255 };

                    let mut buffer = gst::Buffer::with_size(video_info.size()).unwrap();
                    {
                        let buffer = buffer.get_mut().unwrap();

                        buffer.set_pts(frame_num * 200 * gst::ClockTime::MSECOND);

                        let mut vframe =
                            gst_video::VideoFrameRef::from_buffer_ref_writable(buffer, &video_info)
                                .unwrap();

                        let width = vframe.width() as usize;
                        let height = vframe.height() as usize;

                        let stride = vframe.plane_stride()[0] as usize;

                        for line in vframe
                            .plane_data_mut(0)
                            .unwrap()
                            .chunks_exact_mut(stride)
                            .take(height)
                        {
                            for pixel in line[..(4 * width)].chunks_exact_mut(4) {
                                pixel[0] = r;
                                pixel[1] = g;
                                pixel[2] = b;
                                pixel[3] = 255;
                            }
                        }
                    }

                    frame_num += 1;

                    appsrc.push_buffer(buffer).unwrap();

                })
                .build()
        );

        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        self.pipeline.set_state(gst::State::Playing)?;
        self.bus = self.pipeline.bus();

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.pipeline.set_state(gst::State::Null)?;

        Ok(())
    }

    fn handle_message(&self) -> Result<()> {
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

fn _main() -> Result<()> {
    gst::init()?;

    let mut video = Video::new(WIDTH, HEIGHT, gst::Fraction::new(5, 1))?;
    video.setup_pipeline("gl_output.mp4")?;
    video.setup_appsrc()?;
    video.start()?;
    video.handle_message()?;
    video.stop()?;

    Ok(())
}

fn main() {
    common::run(_main).unwrap();
}
