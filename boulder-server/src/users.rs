use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, TypedHeader};
use serde::Deserialize;

use boulder_core::users::User;
use crate::errors::ApiError;
use crate::header::BoulderHeader;
use crate::state::DynDatabase;

#[derive(Deserialize)]
pub struct UserParams {
    name: String,
}

#[derive(Deserialize)]
pub struct UserRoleParams {
    name: String,
    role: String,
}

pub async fn create_user(
    State(state): State<DynDatabase>,
    TypedHeader(_auth): TypedHeader<BoulderHeader>,
    Json(UserParams { name }): Json<UserParams>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.create_user(name).await?;

    Ok((StatusCode::CREATED, res))
}

pub async fn delete_user(
    State(state): State<DynDatabase>,
    TypedHeader(_auth): TypedHeader<BoulderHeader>,
    Json(UserParams { name }): Json<UserParams>,
) -> Result<StatusCode, ApiError> {
    state.delete_user(name).await?;

    Ok(StatusCode::OK)
}

pub async fn view_user_roles(
    State(state): State<DynDatabase>,
    TypedHeader(_auth): TypedHeader<BoulderHeader>,
    Json(UserParams { name }): Json<UserParams>,
) -> Result<Json<User>, ApiError> {
    let res = state.view_user_by_name(name).await?;

    Ok(Json(res))
}

pub async fn grant_user_role(
    State(state): State<DynDatabase>,
    TypedHeader(_auth): TypedHeader<BoulderHeader>,
    Json(UserRoleParams { name, role }): Json<UserRoleParams>,
) -> Result<StatusCode, ApiError> {
    let user = state.view_user_by_name(name).await?;

    user.grant_user_role(role)?;

    Ok(StatusCode::OK)
}

pub async fn revoke_user_role(
    State(state): State<DynDatabase>,
    TypedHeader(_auth): TypedHeader<BoulderHeader>,
    Json(UserRoleParams { name, role }): Json<UserRoleParams>,
) -> Result<StatusCode, ApiError> {
    let user = state.view_user_by_name(name).await?;

    user.revoke_user_role(role)?;

    Ok(StatusCode::OK)
}
