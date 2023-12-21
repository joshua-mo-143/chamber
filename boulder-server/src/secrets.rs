use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json, TypedHeader,
};
use serde::Deserialize;

use crate::auth::Claims;
use crate::errors::ApiError;
use crate::state::DynDatabase;
use boulder_db::users::Role;

use crate::header::BoulderHeader;

#[derive(Deserialize)]
pub struct Secret {
    key: String,
    value: String,
}

pub async fn create_secret(
    State(db): State<DynDatabase>,
    _claim: Claims,
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
    State(db): State<DynDatabase>,
    _claim: Claims,
    Json(secret): Json<SecretKey>,
) -> Result<impl IntoResponse, ApiError> {
    let role = Role::Guest;
    let string = db.view_secret(role, secret.key).await.unwrap();

    Ok(string)
}

pub async fn view_all_secrets(
    State(db): State<DynDatabase>,
    _claim: Claims,
) -> Result<impl IntoResponse, ApiError> {
    let role = Role::Guest;
    let string = db.view_all_secrets(role).await.unwrap();

    Ok(Json(string))
}

pub async fn check_locked<B>(
    State(state): State<DynDatabase>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ApiError> {
    match state.is_locked().await {
        false => Ok(next.run(req).await),
        true => Err(ApiError::Locked),
    }
}

pub async fn unlock(
    State(state): State<DynDatabase>,
    TypedHeader(auth): TypedHeader<BoulderHeader>,
) -> Result<impl IntoResponse, ApiError> {
    match state.unlock(auth.key()).await {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err(ApiError::Forbidden),
        Err(_e) => Err(ApiError::Forbidden),
    }
}
