use crate::errors::DatabaseError;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use zeroize::{Zeroize, ZeroizeOnDrop};
use tokio::sync::Mutex;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait Database {
    async fn create_secret(self, key: String, value: String) -> Result<(), DatabaseError>;
    async fn view_secret(self, user_roles: Vec<Role>, key: String) -> Result<String, DatabaseError>;
    async fn view_users(self) -> Result<Vec<User>, DatabaseError>;
    async fn get_roles_for_user(&self, name: String) -> Result<Vec<Role>, DatabaseError>;  
    async fn view_user_by_name(&self, id: String) -> Result<User, DatabaseError>; 
    async fn get_user_from_password(self, password: String) -> Result<User, DatabaseError>; 
    async fn create_user(self, name: String) -> Result<String, DatabaseError>; 
    async fn delete_user(self, name: String) -> Result<(), DatabaseError>; 
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

#[derive(Clone)]
pub struct User {
pub    name: String,
pub    passkey: String,
pub    roles: Vec<Role>,
pub jwt: Option<String>,
pub jwt_expires: Option<String>
}

impl User {
    pub fn roles(&self) -> Vec<Role> {
        self.roles.clone()
    }

    pub fn grant_user_role(mut self, role: Role) -> Result<(), DatabaseError> {
        if self.roles.iter().any(|x| *x == role) {
            return Err(DatabaseError::RoleAlreadyExists);
        }

        self.roles.push(role);
            self.roles.zeroize();
            Ok(())
    }

    pub fn revoke_user_role(mut self, role: Role) -> Result<(), DatabaseError> {
        if !self.roles.iter().any(|x| *x == role) {
            return Err(DatabaseError::RoleNotFound);
        };

        self.roles.retain(|x| *x != role);
        self.roles.zeroize();
            Ok(())
}
}

#[derive(Clone, Zeroize, ZeroizeOnDrop, Serialize, Deserialize, PartialEq, Debug)]
pub enum Role {
    Guest,
    User,
    Editor,
    AlmostRoot,
    Root
}
