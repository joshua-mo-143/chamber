use std::sync::Arc;
use tokio::sync::RwLock;
use nanoid::nanoid;
use std::collections::{HashMap};
use typenum::consts::U12;
use crate::errors::DatabaseError;
use crate::core::{Database, User, Role};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
    Nonce,
};

use sqlx::PgPool;

use crate::core::LockedStatus;

#[derive(Clone)]
pub struct Postgres {
    pub sealkey: String,
    pub pool: PgPool,
    pub lock: LockedStatus,
}

#[async_trait::async_trait]
impl Postgres {
    fn from_sqlx_pool(pg: PgPool) -> Self {
        pg.execute(MIGRATIONS).await.unwrap();
        Self {
            sealkey: "111".to_string(),
            pool,
            lock: LockedStatus::default()
        }
    } 
}

#[async_trait::async_trait]
impl Database for Postgres {
    async fn create_secret(self, key: String, value: String) -> Result<(), DatabaseError> {
        let encrypted_secret = EncryptedSecret::new(self.key, key.clone(), value);

        sqlx::query(
        "INSERT INTO SECRETS 
                    (key, nonce, ciphertext)
                    VALUES
                    ($1, $2, $3)" 
                    )
                    .bind(key),
                    .bind(encrypted_secret.nonce) 
                    .bind(encrypted_secret.ciphertext)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }

    async fn view_secret(self, user_roles: Vec<Role>, key: String) -> Result<String, DatabaseError> {
        sqlx::query_as::<_, SingleValue>("SELECT") 
    }
    async fn view_users(self) -> Result<Vec<User>, DatabaseError>;
    async fn get_roles_for_user(&self, name: String) -> Result<Vec<Role>, DatabaseError>;  
    async fn view_user_by_name(&self, id: String) -> Result<User, DatabaseError>; 
    async fn get_user_from_password(self, password: String) -> Result<User, DatabaseError>; 
    async fn create_user(self, name: String) -> Result<String, DatabaseError>; 
    async fn delete_user(self, name: String) -> Result<(), DatabaseError>; 
}

static MIGRATIONS: &'static str = r#"
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS secrets (
    id SERIAL PRIMARY KEY,
    key VARCHAR NOT NULL,
    nonce BYTEA NOT NULL,
    ciphertext BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);"#;

#[derive(sqlx::FromRow)]
pub struct SingleValue(String);
