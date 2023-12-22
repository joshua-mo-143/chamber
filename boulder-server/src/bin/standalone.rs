use boulder_core::kv::InMemoryDatabase;
use boulder_server::router::init_router;
use boulder_server::state::DynDatabase;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
    let router = init_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
