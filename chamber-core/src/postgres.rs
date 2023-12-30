use crate::core::Database;
use crate::errors::DatabaseError;
use crate::secrets::{EncryptedSecret, Secret};
use crate::users::User;

use sqlx::types::BigDecimal;
use sqlx::PgPool;

use crate::secrets::SecretInfo;

#[derive(Clone)]
pub struct Postgres(pub PgPool);

impl Postgres {
    pub fn from_pool(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[async_trait::async_trait]
impl Database for Postgres {
    async fn create_secret(&self, new_secret: EncryptedSecret) -> Result<(), DatabaseError> {
        // you might need to convert to Vec<u8> here for the Nonce
        sqlx::query(
            "INSERT INTO SECRETS 
                    (key, nonce, ciphertext, tags, access_level, role_whitelist)
                    VALUES
                    ($1, $2, $3, $4, $5, $6)",
        )
        .bind(new_secret.key())
        .bind(BigDecimal::from(new_secret.nonce.0 - 1))
        .bind(new_secret.ciphertext())
        .bind(new_secret.tags())
        .bind(new_secret.access_level())
        .bind(new_secret.role_whitelist())
        .execute(&self.0)
        .await?;

        Ok(())
    }

    async fn view_all_secrets_admin(
        &self,
    ) -> Result<Vec<EncryptedSecret>, DatabaseError> {
        let retrieved_keys = sqlx::query_as::<_, EncryptedSecret>(
            "SELECT 
            key, nonce, ciphertext, tags, access_level, role_whitelist
            FROM secrets
                ",
        )
        .fetch_all(&self.0)
        .await?;

        Ok(retrieved_keys)
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
        .fetch_all(&self.0)
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
            .execute(&self.0)
            .await?;

        Ok(())
    }

    async fn rekey_all_secrets(&self, secrets: Vec<EncryptedSecret>) -> Result<(), DatabaseError> {
        let transaction = self.0.try_begin().await?.unwrap();

        for secret in secrets {
            if let Err(e) = sqlx::query("UPDATE secrets SET ciphertext = $1 WHERE key = $2")
            .bind(secret.ciphertext())
            .bind(secret.key())
            .execute(&self.0)
            .await {
                transaction.rollback().await?;
                return Err(DatabaseError::SQLError(e));
            }
        }

        transaction.commit().await?;
        Ok(())

    }

    async fn view_secret(&self, user: User, key: String) -> Result<EncryptedSecret, DatabaseError> {
        let retrieved_key = sqlx::query_as::<_, EncryptedSecret>(
            "SELECT key, nonce, ciphertext, tags, access_level, role_whitelist FROM secrets WHERE key = $1 AND $2 >= access_level",
        )
        .bind(key)
        .bind(user.access_level())
        .fetch_one(&self.0)
        .await?;

        Ok(retrieved_key)
    }

    async fn view_secret_decrypted(
        &self,
        user: User,
        key: String,
    ) -> Result<Secret, DatabaseError> {
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
        .fetch_one(&self.0)
        .await?;

        Ok(retrieved_key)
    }

    async fn delete_secret(&self, key: String) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM secrets WHERE key = $1")
            .bind(key)
            .execute(&self.0)
            .await?;

        Ok(())
    }
    async fn view_users(&self) -> Result<Vec<User>, DatabaseError> {
        let query = sqlx::query_as::<_, User>("SELECT username, role FROM USERS")
            .fetch_all(&self.0)
            .await?;

        Ok(query)
    }

    async fn get_user_from_name(&self, username: String) -> Result<User, DatabaseError> {
        let query = sqlx::query_as::<_, User>(
            "SELECT username, password, access_level, roles FROM USERS WHERE USERNAME = $1",
        )
        .bind(username)
        .fetch_one(&self.0)
        .await?;

        Ok(query)
    }

    async fn get_user_from_password(&self, password: String) -> Result<User, DatabaseError> {
        let query = sqlx::query_as::<_, User>(
            "SELECT username, password, access_level, roles FROM users WHERE PASSWORD = $1",
        )
        .bind(password)
        .fetch_one(&self.0)
        .await?;

        Ok(query)
    }

    async fn create_user(&self, user: User) -> Result<String, DatabaseError> {
        let query = sqlx::query_as::<_, SingleValue>(
            "INSERT INTO users
            (username, password)
            VALUES
            ($1, $2) RETURNING PASSWORD",
        )
        .bind(user.username)
        .bind(user.password)
        .fetch_one(&self.0)
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
        .execute(&self.0)
        .await?;

        Ok(())
    }

    async fn delete_user(&self, name: String) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM USERS WHERE USERNAME = $1")
            .bind(name)
            .execute(&self.0)
            .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct SingleValue(String);
