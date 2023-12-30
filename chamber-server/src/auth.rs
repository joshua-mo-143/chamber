use axum::{
    async_trait,
    extract::{FromRequestParts, State, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use chamber_core::errors::DatabaseError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use std::sync::Arc;
use std::time::SystemTime;

use chamber_core::core::AuthBody;
use chamber_core::core::Database;
use chamber_core::traits::AppState;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let random = SystemRandom::new();
    let mut secret = [0u8; 200];
    let _ = random.fill(&mut secret);
    Keys::new(&secret)
});

#[derive(Deserialize)]
pub struct UserLoginParams {
    username: String,
    password: String,
}

pub async fn login<S: AppState>(
    State(state): State<Arc<S>>,
    Json(user): Json<UserLoginParams>,
) -> Result<(StatusCode, Json<AuthBody>), AuthError> {
    let state = state.db();
    // Check if the user sent the credentials
    if user.username.is_empty() | user.password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    // Here you can check the user credentials from a database
    let returned_user = match state.get_user_from_name(user.username).await {
        Ok(res) => res,
        Err(e) => return Err(AuthError::WrongCredentials(e)),
    }; 

    if let Err(e) = returned_user.verify(&user.password) {
       return Err(AuthError::WrongCredentials(e)); 
    }

    // 24 hour timer
    let exp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 86400;

    let claims = Claims {
        sub: returned_user.username.to_owned(),
        exp: exp.try_into().unwrap(), 
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok((StatusCode::OK, Json(AuthBody::new(token))))
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name: {}", self.sub)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials(e) => {
                (StatusCode::UNAUTHORIZED, format!("Wrong credentials: {e}"))
            }
            AuthError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials".to_string())
            }
            AuthError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".to_string(),
            ),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials(DatabaseError),
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
