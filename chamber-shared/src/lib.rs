use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthBody {
   pub access_token: String,
   pub token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct SecretPublic {
    pub key: String,
    pub value: String
}
