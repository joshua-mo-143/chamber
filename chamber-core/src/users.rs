use crate::errors::DatabaseError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use serde::Serialize;

#[derive(Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
    access_level: i32,
    roles: Vec<String>,
}

impl<'a> User {
    pub fn new(username: String, password: String) -> Self {
        let password = password.into_bytes();

        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2.hash_password(&password, &salt).unwrap().to_string();

        Self {
            username,
            password: password_hash,
            access_level: 0,
            roles: Vec::new(),
        }
    }

    pub fn verify(&self, pw: &str) -> Result<(), DatabaseError> {
        let parsed_hash = PasswordHash::new(&self.password)?;
        Argon2::default().verify_password(pw.as_bytes(), &parsed_hash)?;

        Ok(())
    }

    pub fn access_level(&self) -> i32 {
        self.access_level
    }

    pub fn set_access_level(&mut self, access_level: i32) {
        self.access_level = access_level;
    }

    pub fn roles(&'a self) -> &'a [String] {
        &self.roles
    }

    pub fn set_roles(&mut self, vec: Vec<String>) {
        self.roles = vec;
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
