use boulder_server::router::init_router;
use boulder_server::state::DynDatabase;
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

    let pg = match std::env::var("BOULDER_BIN") {
        Ok(var) => Postgres::from_pool(db).with_env_var(var),
        Err(_) => {
            println!("Couldn't get a key file! Reverting to default random key generation.");
            println!("Be sure to create a key file using the CLI otherwise a random crypto key will be generated, invalidating keys on previous deployments.");
            Postgres::from_pool(db)
        }
    };

    let state = Arc::new(pg) as DynDatabase;
    let router = init_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
