use axum::{
    extract::{Json, Path},
    routing::{get, post},
    Router,
};
use flate2::read::GzDecoder;
use reqwest::{Client, StatusCode};
use risc0_zkvm::{
    default_prover,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{error::Error, result::Result};
use std::{io::Read, time::Instant};
use tar::Archive;
use tokio::task;

use crate::{error::AxumResult, services::proof_session::{self, SessionRecord}};

#[derive(Debug, Deserialize)]
enum DynType {
    Integer,
    Float,
    String,
}
impl DynType {
    fn default() -> Self {
        DynType::Integer
    }
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
struct ProofPayload {
    cid: String,
    arguments: Vec<Argument>,

    #[serde(default = "bool::default")]
    is_wasm: bool,

    #[serde(default = "DynType::default")]
    result_type: DynType,
}

pub fn routes() -> Router {
    Router::new()
        .route("/api/proof", post(api_proof_create))
        .route("/api/proof/:id/status", get(api_proof_status))
}

async fn api_proof_status(Path(id): Path<String>) -> AxumResult<Json<Value>> {
    let proof_session: SessionRecord = proof_session::fetch(&id).await.expect("Proof Session not Found");

    Ok(Json(json!(proof_session)))
}

async fn api_proof_create(Json(payload): Json<ProofPayload>) -> AxumResult<Json<Value>> {
    // Create a session with Payload
    let proof_session: SessionRecord = proof_session::create()
        .await
        .expect("Unable to create the proof session");

    // Start task in background
    task::spawn(async {
        // Proofs
        match do_prove(payload).await {
            Ok(_) => {
                println!("Proof ran successfully");
            }
            Err(_) => {
                println!("Failed to run proof");
            }
        };

        // TODO: Update session data
    });

    Ok(Json(json!({ "session_id": proof_session.session_id })))
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

async fn do_prove(payload: ProofPayload) -> Result<(), Box<dyn Error>> {
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

    let now = Instant::now();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove_elf(env, &elf_file).unwrap();
    receipt.verify(manifest.elf_id).unwrap();

    // let program = Program::load_elf(&elf_file, MEM_SIZE as u32).unwrap();
    // let image = MemoryImage::new(&program, PAGE_SIZE as u32).unwrap();
    // let image_id = hex::encode(image.compute_id());

    // let metadata: ReceiptMetadata = receipt.get_metadata().unwrap();

    let result: Value = match payload.result_type {
        DynType::Integer => {
            let int_result: i32 = from_slice(&receipt.journal).unwrap();
            int_result.into()
        }
        DynType::Float => {
            let float_result: f32 = from_slice(&receipt.journal).unwrap();
            float_result.into()
        }
        DynType::String => {
            let string_result: String = from_slice(&receipt.journal).unwrap();
            string_result.into()
        }
    };

    // let receipt_data = bincode::serialize(&receipt).unwrap();
    println!("Receipt: {:?}", result);

    // println!("Seal Bytes: {:?}", receipt.inner.flat());
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    // let body: Json<Value> = Json(json!({
    //     "result": result,
    //     "metadata": metadata
    // }));

    Ok(())
}
