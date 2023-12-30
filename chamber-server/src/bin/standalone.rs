use chamber_server::router::init_router;

use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;

use chamber_core::traits::AppState;

#[tokio::main]
async fn main() {
    let conn_string =
        std::env::var("DATABASE_URL").expect("Couldn't get DATABASE_URL env var, does it exist?");

    let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();

    let db = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(5)
        .connect(&conn_string)
        .await
        .unwrap();

    sqlx::migrate!().run(&db).await.unwrap();

    let state = chamber_core::traits::RegularAppState::new(db);

    state.check_keyfile_exists();

    let router = init_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
