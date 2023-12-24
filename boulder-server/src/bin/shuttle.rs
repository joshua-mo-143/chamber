use boulder_core::postgres::Postgres;
use boulder_server::router::init_router;
use boulder_server::state::DynDatabase;
use sqlx::PgPool;
use std::sync::Arc;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] db: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!().run(&db).await.unwrap();

    let pg = match std::fs::read("boulder.bin") {
        Ok(file) => Postgres::from_pool(db).with_config_file(file),
        Err(_) => {
            println!("Couldn't get a key file! Reverting to default random key generation.");
            println!("Be sure to create a key file using the CLI otherwise a random crypto key will be generated, invalidating keys on previous deployments.");
            Postgres::from_pool(db)
        }
    };

    let state = Arc::new(pg) as DynDatabase;
    let router = init_router(state);

    Ok(router.into())
}
