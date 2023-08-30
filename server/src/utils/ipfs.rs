use reqwest::multipart::{Form, Part};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Web3StorageResponse {
    cid: String,
}

pub async fn download_from_ipfs(cid: &String) -> Result<Vec<u8>, Box<dyn Error>> {
    let base_url = "https://dweb.link/api/v0";
    let cid_url = format!("{}/cat/{}", base_url, cid);

    let client: Client = Client::builder().build().unwrap();
    let response = client.get(&cid_url).send().await.unwrap();

    if response.status() != StatusCode::OK {
        return Err("Failed to download file from IPFS.".into());
    }

    let content = response.bytes().await.unwrap().to_vec();

    Ok(content)
}

pub async fn upload_to_ipfs(file_name: &str, file: Part) -> Result<String, Box<dyn Error>> {
    let mut form = Form::new();
    form = form.part("file", file);

    let response = reqwest::Client::new()
        .post("https://api.web3.storage/upload")
        .bearer_auth(env::var("WEB3_STORAGE_TOKEN").unwrap())
        .multipart(form)
        .header("X-NAME", file_name)
        .send()
        .await
        .unwrap();

    if response.status() != StatusCode::OK {
        return Err("Failed to upload file to IPFS.".into());
    }

    let response_json: Web3StorageResponse = response.json().await.unwrap();
    let cid = response_json.cid;

    Ok(cid)
}
