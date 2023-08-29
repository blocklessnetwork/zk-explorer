use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AxumResult;

#[derive(Debug, Serialize, Deserialize)]
struct VerifyPayload {}

pub fn routes() -> Router {
    Router::new().route("/api/verify", post(api_verify))
}

async fn api_verify(Json(payload): Json<VerifyPayload>) -> AxumResult<Json<Value>> {
    Ok(Json(json!({
        "success": true,
        "payload": payload
    })))
}
