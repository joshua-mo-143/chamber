use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chamber_core::errors::DatabaseError;

pub enum ApiError {
    Forbidden,
    Unauthorised,
    Locked,
    IOError(std::io::Error),
    DBError(DatabaseError),
    Utf8Error(std::str::Utf8Error),
    CryptoError(chamber_crypto::errors::DatabaseError)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden => (StatusCode::FORBIDDEN, "Forbidden!".to_string()).into_response(),
            Self::Unauthorised => {
                (StatusCode::UNAUTHORIZED, "Unauthorised!".to_string()).into_response()
            }
            Self::Locked => {
                (StatusCode::LOCKED, "The vault is locked!".to_string()).into_response()
            }
            Self::IOError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            Self::DBError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            Self::CryptoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            Self::Utf8Error(e) => {
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

impl From<std::str::Utf8Error> for ApiError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<chamber_crypto::errors::DatabaseError> for ApiError {
    fn from(e: chamber_crypto::errors::DatabaseError) -> Self {
        Self::CryptoError(e)
    }
}
