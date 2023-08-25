use clap::Parser;
use std::io::{self};

fn build_project() -> Result<(), io::Error> {
    println!("Building project...");

    Ok(())
}

/// Build Risc0 binaries with a WASM file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name
    #[arg(short, long)]
    name: String,

    /// Number
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        build_project().expect("msg");
        println!("Built {}!", args.name)
    }
}
