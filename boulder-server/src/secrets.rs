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

use crate::header::BoulderHeader;
use boulder_core::core::CreateSecretParams;

pub async fn create_secret(
    State(db): State<DynDatabase>,
    _claim: Claims,
    Json(secret): Json<CreateSecretParams>,
) -> Result<impl IntoResponse, ApiError> {
    db.create_secret(secret).await.unwrap();

    Ok(StatusCode::CREATED)
}

pub async fn delete_secret(
    State(db): State<DynDatabase>,
    _claim: Claims,
    Json(SecretKey { key }): Json<SecretKey>,
) -> Result<impl IntoResponse, ApiError> {
    db.delete_secret(key).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct SecretKey {
    key: String,
}

#[derive(Deserialize)]
pub struct ListSecretsArgs {
    pub tag_filter: Option<String>,
}

pub async fn view_secret(
    State(db): State<DynDatabase>,
    claim: Claims,
    Json(secret): Json<SecretKey>,
) -> Result<impl IntoResponse, ApiError> {
    let user = db.view_user_by_name(claim.sub).await?;
    let string = db.view_secret_decrypted(user, secret.key).await?;

    Ok(string)
}

pub async fn view_all_secrets(
    State(db): State<DynDatabase>,
    claim: Claims,
    Json(secret): Json<ListSecretsArgs>,
) -> Result<impl IntoResponse, ApiError> {
    let user = db.view_user_by_name(claim.sub).await?;

    let string = db.view_all_secrets(user, secret.tag_filter).await?;

    Ok(Json(string))
}

#[derive(Deserialize)]
pub struct UpdateSecret {
    key: String,
    update_data: Vec<String>,
}

pub async fn update_secret(
    State(db): State<DynDatabase>,
    claim: Claims,
    Json(secret): Json<UpdateSecret>,
) -> Result<impl IntoResponse, ApiError> {
    let user = db.view_user_by_name(claim.sub).await?;
    let mut secret_key = db.view_secret(user, secret.key.to_owned()).await?;

    secret_key.replace_tags(secret.update_data);

    db.update_secret(secret.key, secret_key).await?;

    Ok(StatusCode::OK)
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
