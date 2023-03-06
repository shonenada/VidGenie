use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// File to genie.
    #[clap(short, long)]
    file: String,
}

fn main() {
    let cli = Args::parse();
    let file_path = cli.file;

    println!("Genie with {}", file_path);
}
