use boulder_server::router::init_router;
use boulder_server::state::AppState;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let state = AppState::new();
    let router = init_router(state);

    Ok(router.into())
}
