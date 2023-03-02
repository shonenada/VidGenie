extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;

mod common;

use gst::prelude::*;

const WIDTH: i32 = 320;
const HEIGHT: i32 = 240;

fn _main() {
    gst::init().unwrap();

    let pipeline = gst::Pipeline::default();

    let video_info =
        gst_video::VideoInfo::builder(gst_video::VideoFormat::Rgba, WIDTH as u32, HEIGHT as u32)
            .fps(gst::Fraction::new(5, 1))
            .build()
            .expect("Failed to create video info");

    let appsrc = gst_app::AppSrc::builder()
        .caps(
            &gst::Caps::builder("video/x-raw")
                .field("format", &gst_video::VideoFormat::Rgba.to_string())
                .field("width", &WIDTH)
                .field("height", &HEIGHT)
                .field("framerate", &gst::Fraction::new(2, 1)).build())
        .format(gst::Format::Time)
        .build();

    let videoconvert = gst::ElementFactory::make("videoconvert")
        .build()
        .unwrap();

    let videoenc = gst::ElementFactory::make("x264enc")
        .build()
        .unwrap();

    let videoparse = gst::ElementFactory::make("h264parse")
        .property_from_str("config-interval", &"3")
        .build()
        .unwrap();

    let qtmux = gst::ElementFactory::make("qtmux").build().unwrap();

    let filesink = gst::ElementFactory::make("filesink")
        .property_from_str("location", &"output.mp4")
        .build()
        .unwrap();

    let links = [
        &appsrc.upcast_ref(),
        &videoconvert,
        &videoenc,
        &videoparse,
        &qtmux,
        &filesink,
    ];

    pipeline.add_many(&links).unwrap();
    gst::Element::link_many(&links).unwrap();

    let mut frame_num = 0;
    appsrc.set_callbacks(
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
                return;
            })
            .build()
    );

    pipeline.set_state(gst::State::Playing).unwrap();

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null).unwrap();
                eprintln!("{:?}", err)
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null).unwrap();
}

fn main() {
    common::run(_main)
}