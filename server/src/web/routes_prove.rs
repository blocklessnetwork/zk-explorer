use axum::{extract::Json, routing::post, Router};
use flate2::read::GzDecoder;
use reqwest::{Client, StatusCode};
use risc0_zkvm::{
    default_prover,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::Read;
use std::{error::Error, result::Result};
use tar::Archive;

use crate::error::AxumResult;

#[derive(Debug, Deserialize)]
enum DynType {
    Integer,
    Float,
    String,
}

#[derive(Debug, Deserialize)]
struct Argument {
    value: String,
    arg_type: DynType,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    wasm_path: Option<String>,
    elf_path: String,
    elf_id: [u32; 8],
}

#[derive(Debug, Deserialize)]
struct ProvePayload {
    cid: String,
    is_wasm: bool,
    arguments: Vec<Argument>,
}

fn read_from_archive(content: &Vec<u8>, file_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let decoder = GzDecoder::new(content.as_slice());
    let mut archive = Archive::new(decoder);

    for entry_result in archive.entries().unwrap() {
        let mut entry = entry_result?;

        if let Some(entry_path) = entry.path()?.to_str() {
            let matchable_path = entry_path.splitn(2, '/').nth(1).unwrap_or("");
            if matchable_path != "" && matchable_path == file_path {
                // Create a buffer to read the contents of the file into
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer)?;

                // You can return the buffer or do something else with it
                return Ok(buffer);
            }
        }
    }

    Err("File not found in archive".into())
}

async fn read_from_ipfs(cid: &String) -> Result<Vec<u8>, Box<dyn Error>> {
    let base_url = "https://dweb.link/api/v0";
    let cid_url = format!("{}/cat/{}", base_url, cid);
    println!("Download file {:?}", &cid_url);

    let client: Client = Client::builder().build().unwrap();
    let response = client.get(&cid_url).send().await.unwrap();

    if response.status() != StatusCode::OK {
        println!("Error");
    }

    let content = response.bytes().await.unwrap().to_vec();
    println!("Got file {:?}", &cid_url);

    Ok(content)
}

pub fn routes() -> Router {
    Router::new().route("/api/prove", post(api_prove))
}

async fn api_prove(Json(payload): Json<ProvePayload>) -> AxumResult<Json<Value>> {
    println!("Run proof");

    let cid = payload.cid;
    let content = read_from_ipfs(&cid).await.expect("msg");
    let manifest_raw =
        read_from_archive(&content, "manifest.json").expect("Manifest should exists");
    let manifest: Manifest =
        serde_json::from_slice(&manifest_raw).expect("Unable to parse manifest");

    let mut env_builder = ExecutorEnv::builder();

    // Add WASM
    if payload.is_wasm && manifest.wasm_path.is_some() {
        let wasm_file = read_from_archive(&content, &manifest.wasm_path.unwrap().replace("./", ""))
            .expect("WASM file not found");

        env_builder.add_input(&to_vec(&wasm_file).unwrap());
    }

    // Add ELF Binary
    let elf_file = read_from_archive(&content, &manifest.elf_path.replace("./", ""))
        .expect("ELF file not found");

    // Add params
    for arg in payload.arguments {
        match arg.arg_type {
            DynType::Integer => {
                let value: i32 = arg.value.parse().expect("Should parse int.");
                env_builder.add_input(&to_vec(&value).unwrap());
            }
            DynType::Float => {
                let value: f32 = arg.value.parse().expect("Should parse float.");
                env_builder.add_input(&to_vec(&value).unwrap());
            }
            DynType::String => {
                env_builder.add_input(&to_vec(&arg.value).unwrap());
            }
        }
    }

    let env = env_builder.build().unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove_elf(env, &elf_file).unwrap();
    // TODO: Implement code for transmitting or serializing the receipt for
    // other parties to verify here

    // Optional: Verify receipt to confirm that recipients will also be able to
    // verify your receipt
    receipt.verify(manifest.elf_id).unwrap();

    let result: i32 = from_slice(&receipt.journal).unwrap();
    let receipt_data = bincode::serialize(&receipt).unwrap();

    println!("Receipt: {:?}", result);

    let body = Json(json!({
        "result": result,
        "receipt": receipt_data
    }));

    Ok(body)
}
