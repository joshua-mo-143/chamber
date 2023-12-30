use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};

use crate::core::{CreateSecretParams, Database};
use crate::errors::DatabaseError;
use crate::secrets::{EncryptedSecret, SecretInfo, KeyFile};
use crate::users::User;
use crate::core::LockedStatus;

#[derive(Clone)]
pub struct InMemoryDatabase {
    pub sealkey: Arc<Mutex<Option<String>>>,
    pub key: Arc<Mutex<Option<Key<Aes256Gcm>>>>,
    pub secrets: Arc<RwLock<HashMap<String, EncryptedSecret>>>,
    pub users: Arc<RwLock<Vec<User>>>,
    pub lock: LockedStatus,
}

impl Default for InMemoryDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        let _sealkey = nanoid::nanoid!(20);
        println!("Your root key is: 111");
        Self {
            sealkey: Arc::new(Mutex::new(None)),
            key: Arc::new(Mutex::new(None)),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(Vec::new())),
            lock: LockedStatus::default(),
        }
    }
}

#[async_trait::async_trait]
impl Database for InMemoryDatabase {
    async fn create_secret(&self, secret: CreateSecretParams) -> Result<(), DatabaseError> {
        let crypto_key = self.key.lock().await;
        let mut new_secret = EncryptedSecret::new(crypto_key.unwrap(), secret.key, secret.value);
        new_secret.set_access_level(secret.access_level);
        new_secret.clone().add_tags(secret.tags);
        new_secret.set_role_whitelist(secret.role_whitelist);
        let mut secrets = self.secrets.write().await;
        secrets.insert(new_secret.key.clone(), new_secret);
        Ok(())
    }

    async fn delete_secret(&self, key: String) -> Result<(), DatabaseError> {
        let mut secrets = self.secrets.write().await;

        secrets.remove(&key);

        Ok(())
    }
    async fn view_all_secrets(
        &self,
        _role: User,
        _tag: Option<String>,
    ) -> Result<Vec<SecretInfo>, DatabaseError> {
        let store = self.secrets.read().await;

        let retrieved_keys: Vec<SecretInfo> = store
            .iter()
            .map(|(k, v)| SecretInfo::from_encrypted(v.clone(), k.to_owned()))
            .collect();

        Ok(retrieved_keys)
    }

    async fn view_secret(
        &self,
        _user_role: User,
        key: String,
    ) -> Result<EncryptedSecret, DatabaseError> {
        let store = self.secrets.read().await;

        let retrieved_key = match store.get(&*key) {
            Some(res) => res,
            None => return Err(DatabaseError::KeyNotFound),
        };

        Ok(retrieved_key.clone())
    }

    async fn update_secret(
        &self,
        key: String,
        secret: EncryptedSecret,
    ) -> Result<(), DatabaseError> {
        let mut store = self.secrets.write().await;

        store.insert(key, secret);

        Ok(())
    }

    async fn view_secret_decrypted(
        &self,
        _user_roles: User,
        key: String,
    ) -> Result<String, DatabaseError> {
        let store = self.secrets.read().await;

        let retrieved_key = match store.get(&*key) {
            Some(res) => res,
            None => return Err(DatabaseError::KeyNotFound),
        };
         let key = self.key.lock().await;
         let key = Aes256Gcm::new(&key.unwrap());

        let plaintext = key.decrypt(&retrieved_key.nonce(), retrieved_key.ciphertext.as_ref())?;

        let string_from_utf8 = std::str::from_utf8(&plaintext)?;

        let string = String::from(string_from_utf8);
        Ok(string)
    }

    async fn view_users(&self) -> Result<Vec<User>, DatabaseError> {
        let store = self.users.read().await;

        Ok(store.to_vec())
    }

    async fn get_user_from_password(&self, password: String) -> Result<User, DatabaseError> {
        let store = self.users.read().await;

        let user = match store.iter().find(|x| x.password == password) {
            Some(user) => user,
            None => return Err(DatabaseError::UserNotFound),
        };

        Ok(user.clone())
    }

    async fn view_user_by_name(&self, id: String) -> Result<User, DatabaseError> {
        let store = self.users.read().await;

        let user = store.clone().into_iter().find(|x| x.username == id);

        if user.is_none() {
            return Err(DatabaseError::UserNotFound);
        }

        Ok(user.unwrap())
    }

    async fn create_user(&self, name: String) -> Result<String, DatabaseError> {
        let mut store = self.users.write().await;

        let user = User::new(name.clone(), None);

        let username_is_taken = store.iter().any(|x| x.username == user.username);
        if !username_is_taken {
            store.push(user.clone());
        } else {
            return Err(DatabaseError::UserAlreadyExists);
        }

        Ok(user.password)
    }

    async fn update_user(&self, _user: User) -> Result<(), DatabaseError> {
        Ok(())
    }

    async fn delete_user(&self, name: String) -> Result<(), DatabaseError> {
        let mut store = self.users.write().await;

        store.retain(|user| user.username == name);

        Ok(())
    }

    async fn unlock(&self, key: String) -> Result<bool, DatabaseError> {
let unseal_key = self.sealkey.lock().await;

        if key != *unseal_key.clone().unwrap() {
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
        let key = self.sealkey.lock().await;

        key.as_ref().cloned().unwrap()
    }
}
