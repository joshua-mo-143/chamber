use crate::errors::DatabaseError;
use serde::{Serialize};

#[derive(Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub username: String,
    #[sqlx(skip)]
    pub password: String,
    access_level: i32,
    roles: Vec<String>
}

impl User {
    pub fn new(username: String, password: Option<String>) -> Self {
        let password = match password {
            Some(password) => password,
            None => nanoid::nanoid!(20)
        };

        Self {
        username,
        password,
        access_level: 0,
        roles: Vec::new()
        }
    }

    pub fn access_level(&self) -> i32 {
        self.access_level
    }

    pub fn set_access_level(mut self, access_level: i32) {
        self.access_level = access_level;
    }

    pub fn roles(self) -> Vec<String> {
        self.roles
    }

    pub fn grant_user_role(mut self, role: String) -> Result<(), DatabaseError> {
        if self.roles.contains(&role) {
            return Err(DatabaseError::RoleAlreadyExists);
        }

        self.roles.push(role);
        Ok(())
    }

    pub fn revoke_user_role(mut self, role: String) -> Result<(), DatabaseError> {
        self.roles.retain(|x| x != &role);

        Ok(())
    }
}
