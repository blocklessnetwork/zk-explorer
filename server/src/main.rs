pub use self::error::{AxumResult, Error};
use axum::{
    routing::{get, get_service},
    Router,
};
use std::env;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use tower_http::services::ServeDir;

mod error;
mod web;

pub static DB: Surreal<Client> = Surreal::init();

async fn hello_world() -> &'static str {
    "Blockless ZK Playground."
}

fn routes_test() -> Router {
    return Router::new().route("/", get(hello_world));
}

fn routes_static() -> Router {
    return Router::new().nest_service("/", get_service(ServeDir::new("./")));
}

async fn connect_db(
    db_uri: &str,
    db_username: &str,
    db_password: &str,
    db_namespace: &str,
) -> surrealdb::Result<()> {
    // Connect to the server
    DB.connect::<Ws>(db_uri).await.expect("Failed to connect");

    DB.signin(Root {
        username: &db_username,
        password: &db_password,
    })
    .await
    .expect("Failed to signin");

    DB.use_ns(db_namespace)
        .use_db(db_namespace)
        .await
        .expect("Failed to set namespace");

    println!("Database connected at {}", db_uri);

    Ok(())
}

#[tokio::main]
async fn main() {
    // Connect DB
    let db_uri = env::var("DB_URI").unwrap();
    let db_username = env::var("DB_USERNAME").unwrap();
    let db_password = env::var("DB_PASSWORD").unwrap();
    let db_namespace = env::var("DB_NAMESPACE").unwrap();

    connect_db(&db_uri, &db_username, &db_password, &db_namespace)
        .await
        .expect("Had some errors running migrations :(");

    // build our application with a single route
    let router = Router::new()
        .merge(routes_test())
        .merge(web::routes_prove::routes())
        .fallback_service(routes_static());

    // run it with hyper on localhost:3000
    println!("Server running on port 3000");
    axum::Server::bind(&"0.0.0.0:3005".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
