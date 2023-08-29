use clap::Parser;

fn build_project() -> Result<(), Box<dyn std::error::Error>> {
    println!("TODO: Call [POST] /api/image");

    Ok(())
}

/// Build Risc0 binaries with a WASM file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the WASM file to build
    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();

    println!("Buildin path {}!", args.path);

    build_project().expect("msg");
}
