extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;

pub use gs_video::Frame;
pub use gs_video::GSVideo;

mod gs_video;

pub fn init_gst() {
    gst::init().expect("Failed to init GStreamer");
}
