use chamber_server::router::init_router;


use chamber_core::secrets::KeyFile;

use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;
use chamber_core::postgres::Postgres;
use chamber_core::core::LockedStatus;

#[tokio::main]
async fn main() {
    let conn_string = std::env::var("DATABASE_URL").expect("Couldn't get DATABASE_URL env var, does it exist?");

    let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();

    let db = PgPoolOptions::new()
            .min_connections(5)
            .max_connections(5)
            .connect(&conn_string)
            .await
            .unwrap();
    
    sqlx::migrate!().run(&db).await.unwrap();

    let db = Postgres::from_pool(db);

    let state = chamber_core::traits::RegularAppState { db, lock: LockedStatus::default()}; 

    if std::fs::read("chamber.bin").is_err() {
        println!("No chamber.bin file attached, generating one now...");
        let key = KeyFile::new();
        println!("Your root key is: {}", key.unseal_key());
        let encoded = bincode::serialize(&key).unwrap();

             std::fs::write("chamber.bin", encoded).unwrap();
        println!("Successfully saved. Don't forget that you can generate a new chamber file from the CLI and upload it!");
    }

    let router = init_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
