use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

pub static DB: Surreal<Client> = Surreal::init();

pub async fn connect_db(
    db_uri: &str,
    db_username: &str,
    db_password: &str,
    db_namespace: &str,
) -> surrealdb::Result<()> {
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
