use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AxumResult;

#[derive(Debug, Serialize, Deserialize)]
struct ImagePayload {}

pub fn routes() -> Router {
    Router::new().route("/api/image", post(api_upload_image))
}

async fn api_upload_image(Json(payload): Json<ImagePayload>) -> AxumResult<Json<Value>> {
    Ok(Json(json!({
        "success": true,
        "payload": payload
    })))
}
