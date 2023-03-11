use std::fs::File;
use std::io;
use std::io::Read;

use clap::Parser;

use vg_common::structs::RenderRequest;
use vg_video::{Frame, Video};

#[derive(Parser, Debug)]
struct Args {
    /// File to genie.
    #[clap(short, long)]
    file: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let file_path = cli.file;
    let mut file = File::open(file_path.clone()).map_err(anyhow::Error::from)?;
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let params: RenderRequest = serde_json::from_str(&data).map_err(anyhow::Error::from)?;
    println!("Genie with {}; request: {:?}", file_path, params);

    let output = "./vid-output.mp4";
    vg_gst::init_gst();
    let mut video = Video::builder()
        .width(params.output.width)
        .height(params.output.height)
        .output_path(output)
        .build()?;

    video.start_render()?;
    let frame = Frame::new(vec![128; 800 * 600 * 3], 1);
    video.send_frame(frame)?;
    video.finish()?;
    video.until_rendered();
    println!("Render into {}", output);


    Ok(())
}
