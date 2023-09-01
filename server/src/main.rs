use crate::db::connect_db;

pub use self::error::{AxumResult, Error};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use std::env;
use tower_http::cors::CorsLayer;

mod db;
mod error;
mod services;
mod utils;
mod web;

async fn api_handler_intro() -> &'static str {
    "Blockless ZK Playground."
}

async fn api_handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 page not found.")
}

#[tokio::main]
async fn main() {
    // Connect DB
    let db_uri = env::var("DB_URI").unwrap();
    let db_username = env::var("DB_USERNAME").unwrap();
    let db_password = env::var("DB_PASSWORD").unwrap();
    let db_namespace = env::var("DB_NAMESPACE").unwrap();
    let web_host = env::var("WEB_HOST").unwrap();

    connect_db(&db_uri, &db_username, &db_password, &db_namespace)
        .await
        .expect("Had some errors running migrations :(");

    // Setup routes
    let router = Router::new()
        .route("/", get(api_handler_intro))
        .merge(web::routes_proofs::routes())
        .fallback(api_handler_404)
        .layer(CorsLayer::permissive());

    println!("Server running on port 3005");
    axum::Server::bind(&web_host.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
