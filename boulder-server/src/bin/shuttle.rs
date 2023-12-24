use boulder_core::postgres::Postgres;
use boulder_core::secrets::KeyFile;
use boulder_server::router::init_router;
use boulder_server::state::DynDatabase;
use sqlx::PgPool;
use std::sync::Arc;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] db: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!().run(&db).await.unwrap();

    let file: KeyFile = match std::fs::read("boulder.bin") {
        Ok(res) => bincode::deserialize(&res).unwrap(),
        Err(_) => KeyFile::new()
    };

    let pg = Postgres::from_pool(db).with_cfg_file(file);

    let state = Arc::new(pg) as DynDatabase;
    let router = init_router(state);

    Ok(router.into())
}
