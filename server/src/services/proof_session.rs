use std::{collections::HashMap, error::Error, str::FromStr};

use hex::FromHex;
use reqwest::multipart::Part;
use risc0_zkvm::{
    default_prover,
    serde::{from_slice, to_vec},
    sha::Digest,
    ExecutorEnv, MemoryImage, Program, Receipt, ReceiptMetadata, MEM_SIZE, PAGE_SIZE,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::sql::{Datetime, Thing};
use tokio::task;
use uuid::Uuid;

use crate::{
    db::DB,
    utils::ipfs::{download_from_ipfs, list_manifest_from_ipfs, upload_to_ipfs},
};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProofSessionArgument {
    value: String,
    arg_type: DynType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProofSessionStatus {
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

    image_id: Option<&'a String>,
    image_cid: &'a String,
    receipt_cid: Option<&'a String>,
    receipt_metadata: Option<&'a ReceiptMetadata>,

    status: ProofSessionStatus,
    argument_type: &'a Vec<DynType>,
    method: &'a String,
    arguments: &'a Vec<ProofSessionArgument>,
    result_type: &'a DynType,

    result: Option<&'a Value>,

    created_at: Datetime,
    completed_at: Option<Datetime>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProofSessionCompleteRecord {
    status: ProofSessionStatus,
    completed_at: Datetime,
    image_id: Option<String>,
    receipt_cid: Option<String>,
    receipt_metadata: Option<ReceiptMetadata>,
    result: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofSessionRecord {
    id: Thing,
    pub session_id: String,
    pub is_wasm: bool,

    pub image_id: Option<String>,
    pub image_cid: String,
    pub receipt_cid: Option<String>,
    pub receipt_metadata: Option<ReceiptMetadata>,

    pub result: Option<Value>,

    #[serde(default = "ProofSessionStatus::default")]
    pub status: ProofSessionStatus,
    pub method: String,
    pub argument_type: Vec<DynType>,
    pub arguments: Vec<ProofSessionArgument>,
    pub result_type: DynType,

    pub created_at: Datetime,
    pub completed_at: Option<Datetime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    wasm_path: Option<String>,
    elf_path: String,
    elf_id: String,
    method: String,
    argument_type: Vec<DynType>,
    result_type: DynType,
}

#[derive(Debug, Deserialize)]
struct ProofSessionRequest {
    manifest: Manifest,
    files: HashMap<String, String>,
    arguments: Vec<ProofSessionArgument>,
    result_type: DynType,
}

pub async fn list_by_image(image_cid: &String) -> Result<Vec<ProofSessionRecord>, Box<dyn Error>> {
    let mut response = DB
        .query("SELECT * FROM type::table($table) WHERE image_cid = $image_cid ORDER BY created_at DESC")
        .bind(("table", "session"))
        .bind(("image_cid", image_cid))
        .await
        .expect("Failed to find proof sessions.");

    let records: Vec<ProofSessionRecord> = response.take(0).unwrap();

    Ok(records)
}

pub async fn fetch(id: &String) -> Result<ProofSessionRecord, Box<dyn Error>> {
    let mut response = DB
        .query("SELECT * FROM type::table($table) WHERE session_id = $session_id")
        .bind(("table", "session"))
        .bind(("session_id", id))
        .await
        .expect("Failed to find proof sessions.");

    let record: Option<ProofSessionRecord> = response.take(0).unwrap();

    if let Some(record) = record {
        Ok(record)
    } else {
        Err("Error::RowNotFound".into())
    }
}

pub async fn verify(id: &String) -> Result<Value, Box<dyn Error>> {
    let proof_session = fetch(id).await.unwrap();
    let receipt_url = proof_session.receipt_cid.unwrap();
    let receipt_buf = download_from_ipfs(&receipt_url).await.unwrap();
    let receipt: Receipt = bincode::deserialize(&receipt_buf).unwrap();
    let image_id: Digest = Digest::from_hex(proof_session.image_id.unwrap()).unwrap();

    receipt
        .verify(image_id)
        .expect("Receipt verification failed");

    // Parse result into a JSON value
    let result: Value = match proof_session.result_type {
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

    Ok(result)
}

pub async fn create(
    image_cid: &String,
    arguments: &Vec<ProofSessionArgument>,
) -> Result<ProofSessionRecord, Box<dyn Error>> {
    // Generate a random session UUID
    let random_id: String = Uuid::new_v4().to_string();
    let (manifest, files) = list_manifest_from_ipfs(&image_cid)
        .await
        .expect("Failed to fetch manifest.");

    // Create a proof session record
    let record: ProofSessionRecord = DB
        .create(SESSION)
        .content(ProofSession {
            session_id: &random_id,
            image_id: None,
            image_cid,
            status: ProofSessionStatus::Preparing,
            is_wasm: manifest.wasm_path.is_some(),
            receipt_cid: None,
            argument_type: &manifest.argument_type,
            method: &manifest.method,
            result_type: &manifest.result_type,
            arguments,
            created_at: Datetime::default(),
            completed_at: None,
            receipt_metadata: None,
            result: None,
        })
        .await
        .unwrap();

    let record_id = record.id.id.clone().to_string();
    let record_request = ProofSessionRequest {
        manifest: manifest.into(),
        files: files.into(),
        arguments: arguments.to_vec(),
        result_type: record.result_type.clone(),
    };

    // Start task in background
    task::spawn(async {
        let updated_status;
        let image_id: Option<String>;
        let receipt: Option<Vec<u8>>;
        let receipt_metadata: Option<ReceiptMetadata>;
        let receipt_result: Option<Value>;
        let receipt_cid: Option<String>;
        let session_id = random_id;

        // // Proofs
        match do_prove(record_request).await {
            Ok((image_id_data, receipt_data, result, metadata)) => {
                updated_status = ProofSessionStatus::Completed;
                receipt = Some(receipt_data);
                image_id = Some(image_id_data);
                receipt_metadata = Some(metadata);
                receipt_result = Some(result);
            }
            Err(_) => {
                updated_status = ProofSessionStatus::Failed;
                receipt = None;
                receipt_metadata = None;
                image_id = None;
                receipt_result = None;
            }
        };

        if let Some(receipt) = receipt {
            let file_name = format!("{}_receipt.bin", session_id);
            let part = Part::bytes(receipt.to_vec())
                .file_name(file_name.to_string())
                .mime_str("application/bincode")
                .unwrap();
            receipt_cid = Some(upload_to_ipfs(&file_name, part).await.unwrap());
        } else {
            receipt_cid = None
        }

        // TODO: Update session data
        let _: ProofSessionRecord = DB
            .update((SESSION, record_id))
            .merge(ProofSessionCompleteRecord {
                status: updated_status,
                completed_at: Datetime::default(),
                image_id,
                receipt_cid,
                receipt_metadata,
                result: receipt_result,
            })
            .await
            .expect("Failed to update proof session status");
    });

    Ok(record)
}

async fn do_prove(
    payload: ProofSessionRequest,
) -> Result<(String, Vec<u8>, Value, risc0_zkvm::ReceiptMetadata), Box<dyn Error>> {
    // Add WASM
    let wasm_file: Option<Vec<u8>>;
    if let Some(wasm_path) = &payload.manifest.wasm_path {
        wasm_file = Some(download_from_ipfs(payload.files.get(wasm_path).unwrap()).await?);
    } else {
        wasm_file = None
    }

    // Add ELF Binary
    let elf_file: Vec<u8> =
        download_from_ipfs(payload.files.get(&payload.manifest.elf_path).unwrap()).await?;
    let mut env_builder = ExecutorEnv::builder();

    if wasm_file.is_some() {
        env_builder.add_input(&to_vec(&wasm_file.unwrap()).unwrap());
    }

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

    // Obtain the default prover.
    let prover = default_prover();

    let program = Program::load_elf(&elf_file, MEM_SIZE as u32).expect("Failed to execute proof.");
    let image = MemoryImage::new(&program, PAGE_SIZE as u32)?;
    let image_id = hex::encode(image.compute_id());

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover
        .prove_elf(env_builder.build().unwrap(), &elf_file)
        .unwrap();
    receipt
        .verify(Digest::from_hex(&payload.manifest.elf_id).unwrap())
        .unwrap();

    // Parse result into a JSON value
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

    let metadata: risc0_zkvm::ReceiptMetadata = receipt.get_metadata().unwrap();

    // Searlize the binary reciept data
    let receipt_data = bincode::serialize(&receipt).unwrap();

    Ok((image_id, receipt_data, result, metadata))
}
