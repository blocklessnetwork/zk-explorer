use risc0_zkvm::{MemoryImage, Program, MEM_SIZE, PAGE_SIZE};
use serde::{Deserialize, Serialize};
use std::env::{self, temp_dir};
use std::fs::{self, remove_dir_all};
use std::process::Command;

use crate::templates::templates::{
    WASM_BUILD_TEMPLATE_BUILD_RS, WASM_BUILD_TEMPLATE_CARGO_TOML,
    WASM_BUILD_TEMPLATE_GUEST_CARGO_TOML, WASM_BUILD_TEMPLATE_GUEST_MAIN_RS,
    WASM_BUILD_TEMPLATE_LIB_RS,
};
use crate::DynType;

pub async fn generate_wasm_elf_binaries(
    method: &String,
    argument_type: &Vec<DynType>,
    result_type: &DynType,
) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    println!("Building ...\n");

    // Prepare arguments
    let argument_type_val: Vec<String> = argument_type
        .clone()
        .into_iter()
        .map(|e| e.to_string())
        .collect();

    let result_type_str: String = result_type.to_string();
    let argument_type_str: String = if argument_type_val.len() > 1 {
        format!("({})", argument_type_val.join(", "))
    } else {
        argument_type_val.first().unwrap().to_string()
    };

    // Create a temporary directory to hold the Cargo project
    let temp_dir = temp_dir();
    let dir_name = format!("bls_{}", rand::random::<u64>());
    let project_dir = temp_dir.join(&dir_name);

    // Create the Cargo project structure
    let src_dir = project_dir.join("src");
    let lib_rs_path = src_dir.join("lib.rs");
    let build_rs_path = project_dir.join("build.rs");
    let cargo_toml_path: std::path::PathBuf = project_dir.join("Cargo.toml");
    let guest_dir = project_dir.join("guest");
    let guest_src_dir = guest_dir.join("src");
    let guest_main_rs_path = guest_src_dir.join("main.rs");
    let guest_cargo_toml_path: std::path::PathBuf = guest_dir.join("Cargo.toml");

    // Write the Rust code to a file
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&guest_src_dir)?;
    fs::write(&cargo_toml_path, WASM_BUILD_TEMPLATE_CARGO_TOML!())?;
    fs::write(&lib_rs_path, WASM_BUILD_TEMPLATE_LIB_RS!())?;
    fs::write(&build_rs_path, WASM_BUILD_TEMPLATE_BUILD_RS!())?;
    fs::write(
        &guest_cargo_toml_path,
        WASM_BUILD_TEMPLATE_GUEST_CARGO_TOML!(),
    )?;
    fs::write(
        &guest_main_rs_path,
        format!(
            WASM_BUILD_TEMPLATE_GUEST_MAIN_RS!(),
            method, argument_type_str, result_type_str
        ),
    )?;

    // Initialize Cargo project (cargo init)
    Command::new("cargo")
        .arg("init")
        .arg("--bin")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to initialize Cargo project");

    // Compile the project (cargo build)
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to compile project");

    // Find the path to the compiled binary
    let target_dir = project_dir.join("target");
    let release_id = target_dir
        .join("riscv-guest")
        .join("riscv32im-risc0-zkvm-elf")
        .join("release");
    let compiled_binary = release_id.join("wasm-guest");

    let elf_file: Vec<u8> = fs::read(compiled_binary).expect("Unable to locate ELF binaries.");
    let program: Program =
        Program::load_elf(&elf_file, MEM_SIZE as u32).expect("Failed to load ELF binaries.");
    let image: MemoryImage = MemoryImage::new(&program, PAGE_SIZE as u32)?;
    let image_id: String = hex::encode(image.compute_id());

    // @TODO Clean up the temporary directory
    remove_dir_all(project_dir).expect("Unable to remove temp directory");

    Ok((image_id, elf_file))
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    wasm_path: String,
    elf_path: String,
    elf_id: String,
    method: String,
    argument_type: Vec<DynType>,
    result_type: DynType,
}

pub async fn upload_package_to_ipfs(
    image_id: &String,
    image: &Vec<u8>,
    wasm: Option<&Vec<u8>>,
    method: &String,
    argument_type: &Vec<DynType>,
    result_type: &DynType,
) -> Result<String, Box<dyn std::error::Error>> {
    let temp_dir = temp_dir();
    let dir_name = format!("bls_{}", rand::random::<u64>());
    let package_dir = temp_dir.join(&dir_name);
    fs::create_dir_all(&package_dir)?;

    let elf_file_name = "elf";
    let wasm_file_name = "zk.wasm";

    // Save files in temp directory
    fs::write(package_dir.join(elf_file_name), image).unwrap();

    if let Some(wasm) = wasm {
        fs::write(package_dir.join(wasm_file_name), wasm).unwrap();
    }

    let manifest = Manifest {
        wasm_path: wasm_file_name.into(),
        elf_path: elf_file_name.into(),
        elf_id: image_id.to_string(),
        method: method.to_string(),
        argument_type: argument_type.to_vec(),
        result_type: result_type.to_owned(),
    };

    let json_manifest = serde_json::to_string_pretty(&manifest).expect("JSON serialization failed");

    fs::write(package_dir.join("manifest.json"), json_manifest)
        .expect("Failed to write JSON to file");

    let results = w3s::helper::upload_dir(
        &package_dir.into_os_string().into_string().unwrap(),
        None,
        env::var("WEB3_STORAGE_TOKEN").unwrap(),
        2,
        None,
        None, // if use encryption with password
        None, // if use compression with zstd level
    )
    .await
    .expect("Failed to upload.");

    Ok(results.first().unwrap().to_string())
}
