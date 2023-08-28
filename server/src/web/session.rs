use std::error::Error;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use uuid::Uuid;

use crate::DB;

const SESSION: &str = "session";

#[derive(Debug, Serialize, Deserialize)]
enum SessionStatus {
    Preparing,
    InProgress,
    Completed,
    TimedOut,
    Failed,
    Cancelled,
}
impl SessionStatus {
    fn default() -> Self {
        SessionStatus::Preparing
    }
}

#[derive(Debug, Serialize)]
struct SessionProof<'a> {
    cid: &'a String,
}

#[derive(Debug, Serialize)]
struct Session<'a> {
    session_id: &'a String,
    image_id: &'a String,
    is_wasm: bool,
    proof: Option<SessionProof<'a>>,
    status: SessionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    id: Thing,
    pub session_id: String,
    pub image_id: String,
    is_wasm: bool,

    #[serde(default = "SessionStatus::default")]
    status: SessionStatus,
}

pub async fn fetch(id: &String) -> Result<SessionRecord, Box<dyn Error>> {
    let mut response = DB
        .query("SELECT * FROM type::table($table) WHERE session_id = $session_id")
        .bind(("table", "session"))
        .bind(("session_id", id))
        .await
        .expect("msg");

    let record: Option<SessionRecord> = response.take(0).unwrap();

    if let Some(record) = record {
        Ok(record)
    } else {
        Err("Error::RowNotFound".into())
    }
}

pub async fn create() -> Result<SessionRecord, Box<dyn Error>> {
    let random_id = Uuid::new_v4().to_string();
    let str: String = "str".into();

    let record: SessionRecord = DB
        .create(SESSION)
        .content(Session {
            session_id: &random_id,
            status: SessionStatus::Preparing,
            image_id: &str,
            is_wasm: false,
            proof: None,
        })
        .await
        .unwrap();

    // let session = create_session().await.expect("msg");
    // let sessions: Vec<SessionRecord> = DB.select("session").await.expect("msg");
    // let mut response = DB
    //     .query("SELECT title FROM type::table($table) WHERE title = $title")
    //     .bind(("table", "session"))
    //     .bind(("title", "Some title"))
    //     .await
    //     .expect("msg");

    // let ids: Vec<String> = response.take("title").unwrap();

    // let body: Json<Value> = Json(json!({
    //     "result": 1,
    //     "metadata": 2,
    //     "ids": ids,
    //     "sessions": sessions
    // }));

    // Ok(body)

    Ok(record)
}
