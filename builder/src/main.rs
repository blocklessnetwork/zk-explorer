mod utils;
mod wasm;

use std::{fs::File, io::Read, str::FromStr};

use clap::{Parser, ValueEnum};
use utils::is_wasm_file;
use wasm::build::generate_wasm_elf_binaries;

fn build_elf(args: &Args) {
    // Validate path

    dbg!(args);
}

async fn build_wasm(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    if !is_wasm_file(&args.path) {
        return Err("Path is not a wasm file.".into());
    }

    let mut wasm_file: Vec<u8> = Vec::new();
    let mut file = File::open(&args.path)?;
    file.read_to_end(&mut wasm_file)
        .expect("Failed to load WASM file.");

    let (image_id, image) =
        generate_wasm_elf_binaries(&args.method, &args.argument_type, &args.result_type)
            .await
            .expect("Unable to generate WASM Elf binaries.");

    dbg!(&image_id);
    // dbg!(&image);

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Risc0 ELF
    ELF,
    /// WASM
    WASM,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DynType {
    I32,
    I64,
    F32,
    F64,
    Integer,
    Float,
}

impl ToString for DynType {
    fn to_string(&self) -> String {
        match &self {
            DynType::I32 => "i32".into(),
            DynType::I64 => "i64".into(),
            DynType::F32 => "f32".into(),
            DynType::F64 => "f64".into(),
            DynType::Integer => "i32".into(),
            DynType::Float => "f32".into(),
        }
    }
}

impl FromStr for DynType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i32" => Ok(DynType::I32),
            "i64" => Ok(DynType::I64),
            "f32" => Ok(DynType::F32),
            "f64" => Ok(DynType::F64),
            "integer" => Ok(DynType::I32),
            "float" => Ok(DynType::F32),
            _ => Err(()),
        }
    }
}

/// Build Risc0 binaries with a WASM file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Type of source package
    #[arg(value_enum)]
    #[arg(long)]
    mode: Mode,

    /// Path of the file or project to build
    #[arg(short, long)]
    path: String,

    #[arg(short, long, default_value = "zkmain")]
    method: String,

    #[arg(short, long)]
    argument_type: Vec<DynType>,

    #[arg(short, long)]
    result_type: DynType,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    match args.mode {
        Mode::ELF => {
            println!("Build elf");
            dbg!(&args);
            build_elf(&args);
        }
        Mode::WASM => {
            println!("Build wasm");
            dbg!(&args);
            build_wasm(&args).await.unwrap();
        }
    };

    // println!("Buildin path {}!", args.path);
}
