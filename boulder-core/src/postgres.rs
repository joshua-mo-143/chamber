use crate::core::Database;
use crate::errors::DatabaseError;
use crate::secrets::{EncryptedSecret, KeyFile, Secret};
use crate::users::User;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
};
use nanoid::nanoid;

use crate::core::CreateSecretParams;
use sqlx::PgPool;

use crate::core::LockedStatus;
use crate::secrets::SecretInfo;

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
            sealkey: "test".to_string(),
            key: Aes256Gcm::generate_key(OsRng),
            pool,
            lock: LockedStatus::default(),
        }
    }

    pub fn with_config_file(mut self, vec: Vec<u8>) -> Self {
        let decoded: KeyFile = bincode::deserialize(&vec).unwrap();

        self.sealkey = decoded.clone().unseal_key();
        self.key = decoded.crypto_key();
        self
    }
}

#[async_trait::async_trait]
impl Database for Postgres {
    async fn create_secret(&self, secret: CreateSecretParams) -> Result<(), DatabaseError> {
        let mut new_secret = EncryptedSecret::new(self.key, secret.key, secret.value);
        new_secret.set_access_level(secret.access_level);
        new_secret.clone().add_tags(secret.tags);
        new_secret.set_role_whitelist(secret.role_whitelist);
        // you might need to convert to Vec<u8> here for the Nonce
        sqlx::query(
            "INSERT INTO SECRETS 
                    (key, nonce, ciphertext, tags, access_level, role_whitelist)
                    VALUES
                    ($1, $2, $3, $4, $5, $6)",
        )
        .bind(new_secret.clone().key)
        .bind(new_secret.clone().nonce_as_u8())
        .bind(new_secret.clone().ciphertext)
        .bind(new_secret.clone().tags())
        .bind(new_secret.access_level())
        .bind(new_secret.role_whitelist())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn view_all_secrets(
        &self,
        user: User,
        tag: Option<String>,
    ) -> Result<Vec<SecretInfo>, DatabaseError> {
        let retrieved_keys = sqlx::query_as::<_, SecretInfo>(
            "SELECT 
            key, tags FROM secrets WHERE (
                    case when $1 is not null 
                    then $1 = ANY(tags)
                    else 1=1 
                    end)
                    AND $2 >= access_level
                ",
        )
        .bind(tag)
        .bind(user.access_level())
        .fetch_all(&self.pool)
        .await?;

        Ok(retrieved_keys)
    }

    async fn update_secret(
        &self,
        key: String,
        secret: EncryptedSecret,
    ) -> Result<(), DatabaseError> {
        // Might need to convert back from Vec<u8> to Nonce<U12>
        sqlx::query("UPDATE secrets SET tags = $1 WHERE key = $2")
            .bind(secret.tags())
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn view_secret(&self, user: User, key: String) -> Result<EncryptedSecret, DatabaseError> {
        let retrieved_key = sqlx::query_as::<_, EncryptedSecret>(
            "SELECT nonce, ciphertext, tags FROM secrets WHERE key = $1 AND $2 >= access_level",
        )
        .bind(key)
        .bind(user.access_level())
        .fetch_one(&self.pool)
        .await?;

        Ok(retrieved_key)
    }

    async fn view_secret_decrypted(
        &self,
        user: User,
        key: String,
    ) -> Result<String, DatabaseError> {
        let retrieved_key = sqlx::query_as::<_, Secret>(
            "SELECT nonce, ciphertext FROM secrets WHERE 
            key = $1 
            AND $2 >= access_level 
            AND ( CASE 
            WHEN ARRAY_LENGTH(role_whitelist, 1) > 0 
            then role_whitelist && $3
            else 1=1 end
            )
            ",
        )
        .bind(key)
        .bind(user.access_level())
        .bind(user.roles())
        .fetch_one(&self.pool)
        .await?;

        let key = Aes256Gcm::new(&self.key);
        let plaintext = key.decrypt(&retrieved_key.nonce.0, retrieved_key.ciphertext.as_ref())?;

        let text_str = std::str::from_utf8(&plaintext)?;

        let string = String::from(text_str);
        Ok(string)
    }

    async fn delete_secret(&self, key: String) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM secrets WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    async fn view_users(&self) -> Result<Vec<User>, DatabaseError> {
        let query = sqlx::query_as::<_, User>("SELECT username, role FROM USERS")
            .fetch_all(&self.pool)
            .await?;

        Ok(query)
    }

    async fn view_user_by_name(&self, username: String) -> Result<User, DatabaseError> {
        let query = sqlx::query_as::<_, User>(
            "SELECT username, password, access_level, roles FROM USERS WHERE USERNAME = $1",
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;

        Ok(query)
    }

    async fn get_user_from_password(&self, password: String) -> Result<User, DatabaseError> {
        let query = sqlx::query_as::<_, User>(
            "SELECT username, password, access_level, roles FROM users WHERE PASSWORD = $1",
        )
        .bind(password)
        .fetch_one(&self.pool)
        .await?;

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
        .await?;

        Ok(query.0)
    }

    async fn update_user(&self, user: User) -> Result<(), DatabaseError> {
        sqlx::query(
            "
            UPDATE users SET
            access_level = $1,
            roles = $2
            where username = $3
            ",
        )
        .bind(user.access_level())
        .bind(user.clone().roles())
        .bind(user.username)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_user(&self, name: String) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM USERS WHERE USERNAME = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;

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

#[derive(sqlx::FromRow)]
pub struct SingleValue(String);
