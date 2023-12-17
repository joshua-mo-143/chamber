use axum::{
    extract::State,
    http::{StatusCode},
    response::IntoResponse,
    TypedHeader,
    Json,
};
use serde::Deserialize;

use boulder_db::core::{Database, Role};
use crate::errors::ApiError;
use crate::state::AppState;
use crate::header::BoulderHeader;

#[derive(Deserialize)]
pub struct UserParams {
    name: String,
}

#[derive(Deserialize)]
pub struct UserRoleParams {
    name: String,
    role: Role
}

pub async fn create_user(
        State(state): State<AppState>,
        TypedHeader(auth): TypedHeader<BoulderHeader>, 
        Json(UserParams{ name }): Json<UserParams>
    ) -> Result<impl IntoResponse, ApiError> {
        let res = state.db.create_user(name).await?;

        Ok((StatusCode::CREATED, res))
}

pub async fn delete_user(
        State(state): State<AppState>,
        TypedHeader(auth): TypedHeader<BoulderHeader>, 
        Json(UserParams{ name }): Json<UserParams>  
    ) -> Result<StatusCode, ApiError> {
        state.db.delete_user(name).await?;

        Ok(StatusCode::OK)
}

pub async fn view_user_roles(
        State(state): State<AppState>,
        TypedHeader(auth): TypedHeader<BoulderHeader>, 
        Json(UserParams{ name }): Json<UserParams> 
    ) -> Result<Json<Vec<Role>>, ApiError> {
        let res = state.db.view_user_by_name(name).await?;

        let roles = res.roles();

        Ok(Json(roles))
}

pub async fn grant_user_role(
        State(state): State<AppState>,
        TypedHeader(auth): TypedHeader<BoulderHeader>, 
        Json(UserRoleParams{ name, role }): Json<UserRoleParams> 
    ) -> Result<StatusCode, ApiError> {
        let user = state.db.view_user_by_name(name).await?;

        user.grant_user_role(role)?;

        Ok(StatusCode::OK)
}

pub async fn revoke_user_role(
        State(state): State<AppState>,
        TypedHeader(auth): TypedHeader<BoulderHeader>, 
        Json(UserRoleParams{ name, role }): Json<UserRoleParams> 
    ) -> Result<StatusCode, ApiError> {
        let user = state.db.view_user_by_name(name).await?;

        user.revoke_user_role(role)?;

        Ok(StatusCode::OK)
}
