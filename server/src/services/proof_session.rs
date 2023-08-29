use std::{error::Error, io::Read, time::Instant};

use flate2::read::GzDecoder;
use reqwest::{Client, StatusCode};
use risc0_zkvm::{
    default_prover,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::sql::Thing;
use tar::Archive;
use tokio::task;
use uuid::Uuid;

use crate::db::DB;

const SESSION: &str = "session";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DynType {
    I32,
    I64,
    F32,
    F64,
    Integer,
    Float,
}
impl DynType {
    pub fn default() -> Self {
        DynType::Integer
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProofSessionArgument {
    value: String,
    arg_type: DynType,
}

#[derive(Debug, Serialize, Deserialize)]
enum ProofSessionStatus {
    Preparing,
    InProgress,
    Completed,
    TimedOut,
    Failed,
    Cancelled,
}
impl ProofSessionStatus {
    fn default() -> Self {
        ProofSessionStatus::Preparing
    }
}

#[derive(Debug, Serialize)]
struct ProofSession<'a> {
    session_id: &'a String,
    is_wasm: bool,
    image_id: &'a String,
    receipt_id: Option<&'a String>,
    status: ProofSessionStatus,
    arguments_type: &'a Vec<DynType>,
    arguments: &'a Vec<ProofSessionArgument>,
    result_type: &'a DynType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofSessionRecord {
    id: Thing,
    pub session_id: String,
    pub image_id: String,
    is_wasm: bool,

    #[serde(default = "ProofSessionStatus::default")]
    status: ProofSessionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProofSessionStatusRecord {
    status: ProofSessionStatus,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    wasm_path: Option<String>,
    elf_path: String,
    elf_id: [u32; 8],
}

#[derive(Debug, Deserialize)]
struct ProofSessionRequest {
    cid: String,
    is_wasm: bool,
    arguments: Vec<ProofSessionArgument>,
    result_type: DynType,
}

pub async fn list_by_image_id(
    image_id: &String,
) -> Result<Vec<ProofSessionRecord>, Box<dyn Error>> {
    let mut response = DB
        .query("SELECT * FROM type::table($table) WHERE image_id = $image_id")
        .bind(("table", "session"))
        .bind(("image_id", image_id))
        .await
        .expect("msg");

    let records: Vec<ProofSessionRecord> = response.take(0).unwrap();

    Ok(records)
}

pub async fn fetch(id: &String) -> Result<ProofSessionRecord, Box<dyn Error>> {
    let mut response = DB
        .query("SELECT * FROM type::table($table) WHERE session_id = $session_id")
        .bind(("table", "session"))
        .bind(("session_id", id))
        .await
        .expect("msg");

    let record: Option<ProofSessionRecord> = response.take(0).unwrap();

    if let Some(record) = record {
        Ok(record)
    } else {
        Err("Error::RowNotFound".into())
    }
}

pub async fn create(
    image_id: &String,
    arguments: &Vec<ProofSessionArgument>,
) -> Result<ProofSessionRecord, Box<dyn Error>> {
    // Generate a random session UUID
    let session_id: String = Uuid::new_v4().to_string();

    // Load image manifest
    let arguments_type: Vec<DynType> = vec![DynType::default()];
    let result_type: DynType = DynType::default();

    // Create a proof session record
    let record: ProofSessionRecord = DB
        .create(SESSION)
        .content(ProofSession {
            session_id: &session_id,
            image_id,
            status: ProofSessionStatus::Preparing,
            is_wasm: true,
            receipt_id: None,
            arguments_type: &arguments_type,
            result_type: &result_type,
            arguments,
        })
        .await
        .unwrap();

    let record_id = record.id.id.clone().to_string();
    let record_request = ProofSessionRequest {
        cid: image_id.into(),
        is_wasm: true,
        arguments: arguments.to_vec(),
        result_type,
    };

    // Start task in background
    task::spawn(async {
        let updated_status;

        // Proofs
        match do_prove(record_request).await {
            Ok(_) => {
                println!("Proof ran successfully");
                updated_status = ProofSessionStatus::Completed
            }
            Err(_) => {
                println!("Failed to run proof");
                updated_status = ProofSessionStatus::Failed
            }
        };

        // TODO: Update session data
        let _: ProofSessionRecord = DB
            .update((SESSION, record_id))
            .merge(ProofSessionStatusRecord {
                status: updated_status,
            })
            .await
            .expect("Failed to update proof session status");
    });

    Ok(record)
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

pub async fn read_from_ipfs(cid: &String) -> Result<Vec<u8>, Box<dyn Error>> {
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

async fn do_prove(payload: ProofSessionRequest) -> Result<(), Box<dyn Error>> {
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
            DynType::Integer | DynType::I32 => {
                let value: i32 = arg.value.parse().expect("Should parse int.");
                env_builder.add_input(&to_vec(&value).unwrap());
            }
            DynType::Float | DynType::F32 => {
                let value: f32 = arg.value.parse().expect("Should parse float.");
                env_builder.add_input(&to_vec(&value).unwrap());
            }
            DynType::I64 => {
                let value: i64 = arg.value.parse().expect("Should parse int.");
                env_builder.add_input(&to_vec(&value).unwrap());
            }
            DynType::F64 => {
                let value: f64 = arg.value.parse().expect("Should parse int.");
                env_builder.add_input(&to_vec(&value).unwrap());
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
        DynType::Integer | DynType::I32 => {
            let int_result: i32 = from_slice(&receipt.journal).unwrap();
            int_result.into()
        }
        DynType::Float | DynType::F32 => {
            let float_result: f32 = from_slice(&receipt.journal).unwrap();
            float_result.into()
        }
        DynType::I64 => {
            let int_result: i64 = from_slice(&receipt.journal).unwrap();
            int_result.into()
        }
        DynType::F64 => {
            let int_result: f64 = from_slice(&receipt.journal).unwrap();
            int_result.into()
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
