use axum::{
    TypedHeader,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use boulder_db::core::Database;
use crate::errors::ApiError;
use crate::state::AppState;
use crate::auth::Claims;

use crate::header::BoulderHeader;

#[derive(Deserialize)]
pub struct Secret {
    key: String,
    value: String,
}

pub async fn create_secret(
    State(AppState { db, .. }): State<AppState>,
    claim: Claims,
    Json(Secret { key, value }): Json<Secret>,
) -> Result<impl IntoResponse, ApiError> {
    db.create_secret(key, value).await.unwrap();

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
pub struct SecretKey {
    key: String,
}

pub async fn view_secret(
    State(AppState { db, .. }): State<AppState>,
    claim: Claims,
    Json(secret): Json<SecretKey>,
) -> Result<impl IntoResponse, ApiError> {
    let roles = db.get_roles_for_user(claim.sub).await.unwrap();
    let string = db.view_secret(roles, secret.key).await.unwrap();

    Ok(string)
}

pub async fn check_locked<B>(
    State(state): State<AppState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::error!("Middleware triggered!");

    let state = state.db.lock.is_sealed.lock().await;

    if *state {
        return Err(ApiError::Locked);
    }

    Ok(next.run(req).await)
}

pub async fn unlock(
    State(mut state): State<AppState>,
        TypedHeader(auth): TypedHeader<BoulderHeader>, 
) -> Result<impl IntoResponse, ApiError> {
    if auth.key() != state.db.sealkey {
        return Err(ApiError::Forbidden);
    }

    let mut state = state.db.lock.is_sealed.lock().await;
    
    *state = false;

    Ok(StatusCode::OK)
}
