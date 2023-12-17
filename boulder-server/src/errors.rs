use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use boulder_db::errors::DatabaseError;

pub enum ApiError {
    Forbidden,
    Unauthorised,
    Locked,
    DBError(DatabaseError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden => (StatusCode::FORBIDDEN, "Forbidden!".to_string()).into_response(),
            Self::Unauthorised => {
                (StatusCode::UNAUTHORIZED, "Unauthorised!".to_string()).into_response()
            }
            Self::Locked => (
                StatusCode::LOCKED,
                "The vault is locked!".to_string(),
            ).into_response(),
            Self::DBError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

impl From<DatabaseError> for ApiError {
    fn from(e: DatabaseError) -> Self {
        Self::DBError(e)
    }
}
