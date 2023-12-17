use boulder_server::router::init_router;
use boulder_server::state::AppState;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let state = AppState::new();
    let router = init_router(state);

    let addr = SocketAddr::from(([0,0,0,0],8000));

    axum::Server::bind(&addr).serve(router.into_make_service()).await.unwrap();
}
