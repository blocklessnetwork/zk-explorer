use hex::FromHex;
use risc0_zkvm::sha::Digest;
use risc0_zkvm::{MemoryImage, Program, MEM_SIZE, PAGE_SIZE};
use std::env::temp_dir;
use std::error::Error;
use std::fs::{self};
use std::process::Command;

#[allow(dead_code)]
pub async fn build_image() -> Result<(), Box<dyn Error>> {
    println!("Building project...");

    // Sample Rust code in a string
    let cargo_toml = r#"[package]
  name = "wasm-methods"
  version = "0.1.0"
  edition = "2021"
  
  [build-dependencies]
  risc0-build = {{ version = "0.17.0" }}
  
  [package.metadata.risc0]
  methods = ["guest"]"#;

    let lib_rs = r#""#;

    let build_rs = r#"fn main() {
      risc0_build::embed_methods();
  }
  "#;

    let guest_cargo_toml = r#"[package]
  name = "wasm-guest"
  version = "0.1.0"
  edition = "2021"
  
  [workspace]
  
  [dependencies]
  risc0-zkvm = { version = "0.17.0", default-features = false, features = ["std"] }
  wasmi = "0.31.0""#;

    let guest_main_rs = format!(
        r#"#![no_main]

  use risc0_zkvm::guest::env;
  use wasmi::{{Engine, Linker, Module, Store}};
  
  risc0_zkvm::guest::entry!(main);
  
  const WASM_NAME: &str = "{}";
  type WasmParams = {};
  type WasmResult = {};
  
  pub fn main() {{
      // Load environment variables
      let wasm_name = WASM_NAME;
      let wasm_file: Vec<u8> = env::read();
      let wasm_params: WasmParams = env::read();
  
      let engine = Engine::default();
      let module = Module::new(&engine, &mut &wasm_file[..]).expect("Failed to create module");
      type HostState = i32;
  
      let linker = <Linker<HostState>>::new(&engine);
      let mut store = Store::new(&engine, 0);
      let instance = linker
          .instantiate(&mut store, &module)
          .expect("Failed to instansitate.")
          .start(&mut store)
          .expect("Failed to start.");
  
      let wasm_fn = instance
          .get_typed_func::<WasmParams, WasmResult>(&store, WASM_NAME)
          .expect("Failed to get typed_func.");
  
      let res = wasm_fn
          .call(&mut store, wasm_params)
          .expect("Failed to call verify state");
  
      env::log(&format!(
          "Compile WASM {{}} - {{:?}}- {{}}",
          wasm_name, wasm_params, res
      ));
  
      env::commit(&res);
  }}"#,
        "zkmain", "i32", "i32"
    );

    // Create a temporary directory to hold the Cargo project
    let temp_dir = temp_dir();
    let dir_name = format!("bls_zk_{}", rand::random::<u64>());
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
    fs::write(&cargo_toml_path, cargo_toml).expect("Failed to write Cargo toml to file");
    fs::write(&lib_rs_path, lib_rs).expect("Failed to write Cargo toml to file");
    fs::write(&build_rs_path, build_rs).expect("Failed to write Cargo toml to file");
    fs::write(&guest_cargo_toml_path, guest_cargo_toml)
        .expect("Failed to write Cargo toml to file");
    fs::write(&guest_main_rs_path, guest_main_rs).expect("Failed to write Rust code to file");

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

    println!("Compiled binary: {:?}", compiled_binary);

    let wasm_file: Vec<u8> = fs::read(compiled_binary).expect("msg");

    let program: Program = Program::load_elf(&wasm_file, MEM_SIZE as u32).expect("msg");
    let image: MemoryImage = MemoryImage::new(&program, PAGE_SIZE as u32).expect("msg");
    let image_id: String = hex::encode(image.compute_id());
    let image_id_digest = Digest::from_hex(&image_id).unwrap();

    println!("Image ID {} - {:?}", image_id, image_id_digest);

    // @TODO Upload image and manifest to IPFS
    // let image = bincode::serialize(&image).expect("Failed to serialize memory img");

    // @TODO Clean up the temporary directory

    Ok(())
}
