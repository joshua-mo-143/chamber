use crate::{auth, secrets, users};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use chamber_core::traits::AppState;
use std::sync::Arc;

pub fn init_router<S: AppState>(state: S) -> Router {
    let state = Arc::new(state);

    let user_router = Router::new()
        .route("/create", post(users::create_user))
        .route("/delete", delete(users::delete_user))
        .route("/update", put(users::update_user))
        .route("/roles", post(users::view_user_roles));

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/secrets/set", post(secrets::create_secret))
        .route("/secrets/get", post(secrets::view_secret))
        .route("/secrets/tags", post(secrets::view_secret))
        .route(
            "/secrets",
            post(secrets::view_all_secrets)
                .put(secrets::update_secret)
                .delete(secrets::delete_secret),
        )
        .nest("/users", user_router)
        .route("/login", post(auth::login))
        .route("/binfile", post(secrets::upload_binfile))
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
