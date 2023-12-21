use crate::core::Database;
use crate::errors::DatabaseError;
use crate::secrets::{EncryptedSecret};
use crate::users::{Role, User};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
};
use nanoid::nanoid;

use sqlx::PgPool;

use crate::core::LockedStatus;

#[derive(Clone)]
pub struct Postgres {
    pub sealkey: String,
    pub key: Key<Aes256Gcm>,
    pub pool: PgPool,
    pub lock: LockedStatus,
}

impl Postgres {
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            sealkey: "111".to_string(),
            key: Aes256Gcm::generate_key(OsRng),
            pool,
            lock: LockedStatus::default(),
        }
    }
}

#[async_trait::async_trait]
impl Database for Postgres {
    async fn create_secret(&self, key: String, value: String) -> Result<(), DatabaseError> {
        let encrypted_secret = EncryptedSecret::new(self.key, key.clone(), value);

        // you might need to convert to Vec<u8> here for the Nonce
        sqlx::query(
            "INSERT INTO SECRETS 
                    (key, nonce, ciphertext)
                    VALUES
                    ($1, $2, $3)",
        )
        .bind(key)
        .bind(encrypted_secret.nonce_as_u8())
        .bind(encrypted_secret.ciphertext)
        .execute(&self.pool)
        .await
        .unwrap();

        Ok(())
    }

    async fn view_all_secrets(&self, _user_roles: Role) -> Result<Vec<String>, DatabaseError> {
        let retrieved_keys = sqlx::query_as::<_, SingleValue>(
            "SELECT key FROM secrets",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();

        let string_vec = retrieved_keys.into_iter().map(|x| x.0).collect::<Vec<String>>();

        Ok(string_vec)
    
    }
    async fn view_secret(&self, _user_roles: Role, key: String) -> Result<String, DatabaseError> {
        // Might need to convert back from Vec<u8> to Nonce<U12>
        let retrieved_key = sqlx::query_as::<_, EncryptedSecret>(
            "SELECT nonce, ciphertext FROM secrets WHERE key = $1",
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await
        .unwrap();

        let key = Aes256Gcm::new(&self.key);
        let plaintext = key.decrypt(&retrieved_key.nonce(), retrieved_key.ciphertext.as_ref())?;

        let hehe = std::str::from_utf8(&plaintext)?;

        let meme = String::from(hehe);
        Ok(meme)
    }

    async fn view_users(&self) -> Result<Vec<User>, DatabaseError> {
        let query = sqlx::query_as::<_, User>("SELECT * FROM USERS")
            .fetch_all(&self.pool)
            .await
            .unwrap();

        Ok(query)
    }

    async fn view_user_by_name(&self, id: String) -> Result<User, DatabaseError> {
        let query = sqlx::query_as::<_, User>("SELECT * FROM USERS WHERE ID = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        Ok(query)
    }
    async fn get_user_from_password(&self, password: String) -> Result<User, DatabaseError> {
        let query = sqlx::query_as::<_, User>("SELECT username, password, role FROM USERS WHERE PASSWORD = $1")
            .bind(password)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        Ok(query)
    }
    async fn create_user(&self, name: String) -> Result<String, DatabaseError> {
        let password = nanoid!(20);
        let query = sqlx::query_as::<_, SingleValue>(
            "INSERT INTO users
            (username, password)
            VALUES
            ($1, $2) RETURNING PASSWORD",
        )
        .bind(name)
        .bind(password)
        .fetch_one(&self.pool)
        .await
        .unwrap();

        Ok(query.0)
    }
    async fn delete_user(&self, name: String) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM USERS WHERE USERNAME = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }
    async fn unlock(&self, key: String) -> Result<bool, DatabaseError> {
        if key != self.sealkey {
            return Err(DatabaseError::Forbidden);
        }

        let mut state = self.lock.is_sealed.lock().await;

        *state = false;

        Ok(true)
    }
    async fn is_locked(&self) -> bool {
        let state = self.lock.is_sealed.lock().await;

        *state
    }
    fn get_root_key(&self) -> String {
        self.sealkey.clone()
    }
}

pub static MIGRATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS secrets (
    id SERIAL PRIMARY KEY,
    key VARCHAR NOT NULL UNIQUE,
    nonce BYTEA NOT NULL UNIQUE,
    ciphertext BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);"#;

#[derive(sqlx::FromRow)]
pub struct SingleValue(String);
