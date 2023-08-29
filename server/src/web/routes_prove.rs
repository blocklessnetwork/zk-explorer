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
    image_id: String,
    arguments: Vec<ProofSessionArgument>,
}

pub fn routes() -> Router {
    Router::new()
        .route("/api/proofs", post(api_proof_create))
        .route(
            "/api/proofs/by-image/:image_id",
            get(api_list_proof_by_image),
        )
        .route("/api/proofs/:id", get(api_fetch_proof_status))
}

async fn api_fetch_proof_status(Path(id): Path<String>) -> AxumResult<Json<Value>> {
    let proof_session: ProofSessionRecord = proof_session::fetch(&id)
        .await
        .expect("Proof Session not Found");

    Ok(Json(json!(proof_session)))
}

async fn api_list_proof_by_image(Path(image_id): Path<String>) -> AxumResult<Json<Value>> {
    let proof_sessions: Vec<ProofSessionRecord> = proof_session::list_by_image_id(&image_id)
        .await
        .expect("msg");

    Ok(Json(json!(proof_sessions)))
}

async fn api_proof_create(Json(payload): Json<ProofSessionPayload>) -> AxumResult<Json<Value>> {
    let proof_session: ProofSessionRecord =
        proof_session::create(&payload.image_id, &payload.arguments)
            .await
            .expect("Unable to create the proof session");

    Ok(Json(json!({ "session_id": proof_session.session_id })))
}
