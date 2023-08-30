use std::error::Error;

use hex::FromHex;
use reqwest::multipart::Part;
use risc0_zkvm::{
    default_prover,
    serde::{from_slice, to_vec},
    sha::Digest,
    ExecutorEnv, MemoryImage, Program, Receipt, MEM_SIZE, PAGE_SIZE,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::sql::{Datetime, Thing};
use tokio::task;
use uuid::Uuid;

use crate::{
    db::DB,
    utils::{
        archive::read_from_archive,
        ipfs::{download_from_ipfs, upload_to_ipfs},
    },
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

    status: ProofSessionStatus,
    arguments_type: &'a Vec<DynType>,
    arguments: &'a Vec<ProofSessionArgument>,
    result_type: &'a DynType,

    created_at: Datetime,
    completed_at: Option<Datetime>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProofSessionCompleteRecord {
    status: ProofSessionStatus,
    completed_at: Datetime,
    image_id: Option<String>,
    receipt_cid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofSessionRecord {
    id: Thing,
    pub session_id: String,
    pub is_wasm: bool,

    pub image_id: Option<String>,
    pub image_cid: String,
    pub receipt_cid: Option<String>,

    #[serde(default = "ProofSessionStatus::default")]
    pub status: ProofSessionStatus,
    pub arguments_type: Vec<DynType>,
    pub arguments: Vec<ProofSessionArgument>,
    pub result_type: DynType,

    pub created_at: Datetime,
    pub completed_at: Option<Datetime>,
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

pub async fn list_by_image(image_cid: &String) -> Result<Vec<ProofSessionRecord>, Box<dyn Error>> {
    let mut response = DB
        .query("SELECT * FROM type::table($table) WHERE image_cid = $image_cid")
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

    // Load image manifest
    let arguments_type: Vec<DynType> = vec![DynType::default()];
    let result_type: DynType = DynType::default();

    // Create a proof session record
    let record: ProofSessionRecord = DB
        .create(SESSION)
        .content(ProofSession {
            session_id: &random_id,
            image_id: None,
            image_cid,
            status: ProofSessionStatus::Preparing,
            is_wasm: true,
            receipt_cid: None,
            arguments_type: &arguments_type,
            result_type: &result_type,
            arguments,
            created_at: Datetime::default(),
            completed_at: None,
        })
        .await
        .unwrap();

    let record_id = record.id.id.clone().to_string();
    let record_request = ProofSessionRequest {
        cid: image_cid.into(),
        is_wasm: true,
        arguments: arguments.to_vec(),
        result_type,
    };

    // Start task in background
    task::spawn(async {
        let updated_status;
        let image_id: Option<String>;
        let receipt: Option<Vec<u8>>;
        let receipt_cid: Option<String>;
        let session_id = random_id;

        // Proofs
        match do_prove(record_request).await {
            Ok((image_id_data, receipt_data, _)) => {
                updated_status = ProofSessionStatus::Completed;
                receipt = Some(receipt_data);
                image_id = Some(image_id_data);
            }
            Err(_) => {
                updated_status = ProofSessionStatus::Failed;
                receipt = None;
                image_id = None;
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
            })
            .await
            .expect("Failed to update proof session status");
    });

    Ok(record)
}

async fn do_prove(
    payload: ProofSessionRequest,
) -> Result<(String, Vec<u8>, Value), Box<dyn Error>> {
    let cid = payload.cid;
    let content = download_from_ipfs(&cid)
        .await
        .expect("Failed to find package.");
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

    // Obtain the default prover.
    let prover = default_prover();

    let program = Program::load_elf(&elf_file, MEM_SIZE as u32)?;
    let image = MemoryImage::new(&program, PAGE_SIZE as u32)?;
    let image_id = hex::encode(image.compute_id());

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover
        .prove_elf(env_builder.build().unwrap(), &elf_file)
        .unwrap();
    receipt.verify(manifest.elf_id).unwrap();

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

    // Searlize the binary reciept data
    let receipt_data = bincode::serialize(&receipt).unwrap();

    Ok((image_id, receipt_data, result))
}
