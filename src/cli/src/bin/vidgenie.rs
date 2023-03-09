use std::fs::File;
use std::io;
use std::io::Read;

use clap::Parser;

use common::structs;

#[derive(Parser, Debug)]
struct Args {
    /// File to genie.
    #[clap(short, long)]
    file: String,
}

fn main() -> io::Result<()>{
    let cli = Args::parse();
    let file_path = cli.file;
    let mut file = File::open(file_path.clone())?;
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let input: structs::RenderRequest = serde_json::from_str(&data)?;

    println!("Genie with {}; request: {:?}", file_path, input);

    Ok(())
}
