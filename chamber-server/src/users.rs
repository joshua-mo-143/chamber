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
pub struct UpdateUserParams {
    pub username: String,
    pub access_level: Option<i32>,
    pub roles: Option<Vec<String>>,
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

pub async fn update_user<S: AppState>(
    State(state): State<Arc<S>>,
    TypedHeader(_auth): TypedHeader<ChamberHeader>,
    Json(UpdateUserParams { username, access_level, roles }): Json<UpdateUserParams>,
) -> Result<StatusCode, ApiError> {
    let mut user = state.db().get_user_from_name(username).await?;

    if let Some(roles) = roles {
        user.set_roles(roles);
    }

    if let Some(access_level) = access_level {
        user.set_access_level(access_level);
    }

    state.db().update_user(user).await?;

    Ok(StatusCode::OK)
}
