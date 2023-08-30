use axum::{
    extract::{Json, Path},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error::AxumResult,
    services::proof_session::{self, ProofSessionArgument, ProofSessionRecord},
};

#[derive(Debug, Deserialize)]
struct ProofSessionPayload {
    image_cid: String,
    arguments: Vec<ProofSessionArgument>,
}

pub fn routes() -> Router {
    Router::new()
        .route("/api/proofs", post(api_proof_create))
        .route(
            "/api/proofs/by-image/:image_cid",
            get(api_list_proof_by_image),
        )
        .route("/api/proofs/:id", get(api_fetch_proof_status))
        .route("/api/proofs/:id/verify", get(api_fetch_proof_verification))
}

async fn api_fetch_proof_status(Path(id): Path<String>) -> AxumResult<Json<Value>> {
    let proof_session: ProofSessionRecord = proof_session::fetch(&id)
        .await
        .expect("Proof Session not Found");

    Ok(Json(json!(proof_session)))
}

async fn api_list_proof_by_image(Path(image_cid): Path<String>) -> AxumResult<Json<Value>> {
    let proof_sessions: Vec<ProofSessionRecord> = proof_session::list_by_image(&image_cid)
        .await
        .expect("Unable to list proof sessions");

    Ok(Json(json!(proof_sessions)))
}

async fn api_proof_create(Json(payload): Json<ProofSessionPayload>) -> AxumResult<Json<Value>> {
    let proof_session: ProofSessionRecord =
        proof_session::create(&payload.image_cid, &payload.arguments)
            .await
            .expect("Unable to create the proof session");

    Ok(Json(json!({ "session_id": proof_session.session_id })))
}

async fn api_fetch_proof_verification(Path(id): Path<String>) -> AxumResult<Json<Value>> {
    let proof_session_result = proof_session::verify(&id)
        .await
        .expect("Unable to verify the proof session");

    Ok(Json(json!({
        "verified": true,
        "result": proof_session_result
    })))
}
