use crate::errors::DatabaseError;

use chrono::{DateTime, Utc};

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::users::{Role, User};

#[async_trait::async_trait]
pub trait Database {
    async fn create_secret(&self, key: String, value: String) -> Result<(), DatabaseError>;
    async fn view_all_secrets(&self, user_roles: Role) -> Result<Vec<String>, DatabaseError>;
    async fn view_secret(&self, user_roles: Role, key: String) -> Result<String, DatabaseError>;
    async fn view_users(&self) -> Result<Vec<User>, DatabaseError>;
    async fn view_user_by_name(&self, id: String) -> Result<User, DatabaseError>;
    async fn get_user_from_password(&self, password: String) -> Result<User, DatabaseError>;
    async fn create_user(&self, name: String) -> Result<String, DatabaseError>;
    async fn delete_user(&self, name: String) -> Result<(), DatabaseError>;
    async fn unlock(&self, key: String) -> Result<bool, DatabaseError>;
    async fn is_locked(&self) -> bool;
    fn get_root_key(&self) -> String;
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
