use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, TypedHeader};
use serde::Deserialize;

use crate::errors::ApiError;
use crate::header::ChamberHeader;
use std::sync::Arc;

use chamber_core::users::User;

use chamber_core::core::Database;
use chamber_core::traits::AppState;
#[derive(Deserialize)]
pub struct UserParams {
    name: String,
}

#[derive(Deserialize)]
pub struct CreateUserParams {
    pub username: String,
    pub password: String,
    pub access_level: Option<i32>,
    pub roles: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UserRoleParams {
    name: String,
    role: String,
}

pub async fn create_user<S: AppState>(
    State(state): State<Arc<S>>,
    TypedHeader(_auth): TypedHeader<ChamberHeader>,
    Json(params): Json<CreateUserParams>,
) -> Result<impl IntoResponse, ApiError> {
    let user = User::new(params.username, params.password);

    let res = state.db().create_user(user).await?;

    Ok((StatusCode::CREATED, res))
}

pub async fn delete_user<S: AppState>(
    State(state): State<Arc<S>>,
    TypedHeader(_auth): TypedHeader<ChamberHeader>,
    Json(UserParams { name }): Json<UserParams>,
) -> Result<StatusCode, ApiError> {
    state.db().delete_user(name).await?;

    Ok(StatusCode::OK)
}

pub async fn view_user_roles<S: AppState>(
    State(state): State<Arc<S>>,
    TypedHeader(_auth): TypedHeader<ChamberHeader>,
    Json(UserParams { name }): Json<UserParams>,
) -> Result<Json<User>, ApiError> {
    let res = state.db().get_user_from_name(name).await?;

    Ok(Json(res))
}

pub async fn grant_user_role<S: AppState>(
    State(state): State<Arc<S>>,
    TypedHeader(_auth): TypedHeader<ChamberHeader>,
    Json(UserRoleParams { name, role }): Json<UserRoleParams>,
) -> Result<StatusCode, ApiError> {
    let user = state.db().get_user_from_name(name).await?;

    user.grant_user_role(role)?;

    Ok(StatusCode::OK)
}

pub async fn revoke_user_role<S: AppState>(
    State(state): State<Arc<S>>,
    TypedHeader(_auth): TypedHeader<ChamberHeader>,
    Json(UserRoleParams { name, role }): Json<UserRoleParams>,
) -> Result<StatusCode, ApiError> {
    let user = state.db().get_user_from_name(name).await?;

    user.revoke_user_role(role)?;

    Ok(StatusCode::OK)
}
