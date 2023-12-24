use boulder_server::router::init_router;
use boulder_server::state::DynDatabase;
use boulder_core::secrets::KeyFile;
use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use boulder_core::postgres::Postgres;

#[tokio::main]
async fn main() {
    let conn_string = std::env::var("DATABASE_URL").expect("Couldn't get DATABASE_URL env var, does it exist?");

    let db = PgPoolOptions::new()
            .min_connections(5)
            .max_connections(5)
            .connect(&conn_string)
            .await
            .unwrap();
    
    sqlx::migrate!().run(&db).await.unwrap();

    let file: KeyFile = match std::fs::read("boulder.bin") {
        Ok(res) => bincode::deserialize(&res).unwrap(),
        Err(_) => KeyFile::new()
    };

    let pg = Postgres::from_pool(db).with_cfg_file(file);

    let state = Arc::new(pg) as DynDatabase;
    let router = init_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
