use crate::errors::DatabaseError;
use crate::secrets::{EncryptedSecret, SecretInfo, KeyFile};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::users::User;

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

#[derive(Deserialize)]
pub struct CreateSecretParams {
    pub key: String,
    pub value: String,
    pub tags: Option<Vec<String>>,
    pub access_level: Option<i32>,
    pub role_whitelist: Option<Vec<String>>,
}

#[async_trait::async_trait]
pub trait Database {
    async fn view_all_secrets(
        &self,
        user: User,
        tag: Option<String>,
    ) -> Result<Vec<SecretInfo>, DatabaseError>;
    async fn view_secret_decrypted(&self, user: User, key: String)
        -> Result<String, DatabaseError>;
    async fn view_secret(&self, user: User, key: String) -> Result<EncryptedSecret, DatabaseError>;
    async fn create_secret(&self, secret: CreateSecretParams) -> Result<(), DatabaseError>;
    async fn update_secret(
        &self,
        key: String,
        secret: EncryptedSecret,
    ) -> Result<(), DatabaseError>;
    async fn delete_secret(&self, key: String) -> Result<(), DatabaseError>;
    async fn view_users(&self) -> Result<Vec<User>, DatabaseError>;
    async fn view_user_by_name(&self, id: String) -> Result<User, DatabaseError>;
    async fn get_user_from_password(&self, password: String) -> Result<User, DatabaseError>;
    async fn create_user(&self, name: String) -> Result<String, DatabaseError>;
    async fn update_user(&self, user: User) -> Result<(), DatabaseError>;
    async fn delete_user(&self, name: String) -> Result<(), DatabaseError>;
    async fn unlock(&self, key: String) -> Result<bool, DatabaseError>;
    async fn is_locked(&self) -> bool;
    fn get_root_key(&self) -> String;
    fn get_key_data(&self) -> KeyFile {
            let file = std::fs::read("boulder.bin").unwrap();
            let keyfile: KeyFile = bincode::deserialize(&file).unwrap();

            keyfile
    }
}

#[derive(Clone)]
pub struct LockedStatus {
    pub is_sealed: Arc<Mutex<bool>>,
    pub relock_datetime: Option<DateTime<Utc>>,
}

impl Default for LockedStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl LockedStatus {
    fn new() -> Self {
        Self {
            is_sealed: Arc::new(Mutex::new(true)),
            relock_datetime: None,
        }
    }
}
