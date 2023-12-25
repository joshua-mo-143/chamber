use chamber_core::postgres::Postgres;

use chamber_core::secrets::KeyFile;
use chamber_server::router::init_router;
use chamber_server::state::DynDatabase;
use sqlx::PgPool;
use std::sync::Arc;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] db: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!().run(&db).await.unwrap();

    let pg = Postgres::from_pool(db);

    if std::fs::read("chamber.bin").is_err() {
        println!("No chamber.bin file attached, generating one now...");
        let key = KeyFile::new();
        println!("Your root key is: {}", key.clone().unseal_key());
        let encoded = bincode::serialize(&key).unwrap();

             std::fs::write("chamber.bin", encoded).unwrap();
        println!("Successfully saved. Don't forget that you can generate a new chamber file from the CLI and upload it!");
    }

    let state = Arc::new(pg) as DynDatabase;
    let router = init_router(state);

    Ok(router.into())
}
