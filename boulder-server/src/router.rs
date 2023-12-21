use crate::{auth, secrets, users};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::state::DynDatabase;

pub fn init_router(state: DynDatabase) -> Router {
    let user_router = Router::new()
        .route("/create", post(users::create_user))
        .route("/delete", delete(users::delete_user))
        .route("/roles", post(users::view_user_roles))
        .route(
            "/roles/edit",
            put(users::grant_user_role).delete(users::revoke_user_role),
        );

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/secrets/set", post(secrets::create_secret))
        .route("/secrets/get", post(secrets::view_secret))
        .route("/secrets/all", post(secrets::view_all_secrets))
        .nest("/users", user_router)
        .route("/login", post(auth::login))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            secrets::check_locked,
        ));

    Router::new()
        .route("/unseal", post(secrets::unlock))
        .merge(router)
        .with_state(state)
}

pub async fn hello_world() -> &'static str {
    "Hello, world!"
}
