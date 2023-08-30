mod build;
mod templates;
mod utils;

use std::{fs::File, io::Read, str::FromStr};

use build::generate_wasm_elf_binaries;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use utils::is_wasm_file;

use crate::build::upload_package_to_ipfs;

fn build_elf(_: &Args) {
    println!("TODO: ELF Builds");
}

async fn build_wasm(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    if !is_wasm_file(&args.path) {
        return Err("Path is not a wasm file.".into());
    }

    // Load Wasm File
    let mut wasm_file: Vec<u8> = Vec::new();
    let mut file = File::open(&args.path)?;
    file.read_to_end(&mut wasm_file)
        .expect("Failed to load WASM file.");

    let (image_id, image) =
        generate_wasm_elf_binaries(&args.method, &args.argument_type, &args.result_type)
            .await
            .expect("Unable to generate WASM Elf binaries.");

    let cid = upload_package_to_ipfs(
        &image_id,
        &image,
        Some(&wasm_file),
        &args.argument_type,
        &args.result_type,
    )
    .await
    .unwrap();

    println!("\n{}", &cid);

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Risc0 ELF
    ELF,
    /// WASM
    WASM,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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
            build_elf(&args);
        }
        Mode::WASM => {
            build_wasm(&args).await.unwrap();
        }
    };
}
