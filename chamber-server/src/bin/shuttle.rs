use shuttle_persist::PersistInstance;
use sqlx::PgPool;

use chamber_core::traits::AppState;
use chamber_core::traits::ShuttleAppState;
use chamber_server::router::init_router;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] db: PgPool,
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!().run(&db).await.unwrap();

    let state = ShuttleAppState::new(db, persist);

    state.check_keyfile_exists();

    let router = init_router(state);

    Ok(router.into())
}
