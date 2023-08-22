pub use self::error::{Error, AxumResult};
use axum::{
    routing::{get, get_service},
    Router,
};
use tower_http::services::ServeDir;

mod error;
mod web;

async fn hello_world() -> &'static str {
    "Blockless ZK Playground."
}

fn routes_test() -> Router {
    return Router::new().route("/", get(hello_world));
}

fn routes_static() -> Router {
    return Router::new().nest_service("/", get_service(ServeDir::new("./")));
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let router = Router::new()
        .merge(routes_test())
        .merge(web::routes_prove::routes())
        .fallback_service(routes_static());

    // run it with hyper on localhost:3000
    println!("Server running on port 3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
