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

use crate::core::LockedStatus;

#[derive(Clone)]
pub struct InMemoryDatabase {
    pub sealkey: String,
    pub key: Key<Aes256Gcm>,
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

    fn new() -> Self {
        let sealkey = nanoid::nanoid!(20);
        println!("Your root key is: 111");
        Self {
            sealkey: "111".to_string(),
            key: Aes256Gcm::generate_key(OsRng),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(Vec::new())),
            lock: LockedStatus::default(),
        }
    }
}

#[async_trait::async_trait]
impl Database for InMemoryDatabase {
    async fn create_secret(self, key: String, value: String) -> Result<(), DatabaseError> {
        let encrypted_secret = EncryptedSecret::new(self.key, key.clone(), value);

        let mut secrets = self.secrets.write().await;
        secrets.insert(key, encrypted_secret);
        Ok(())
    }

    async fn view_secret(self, user_roles: Vec<Role>, key: String) -> Result<String, DatabaseError> {
        let store = self.secrets.read().await;

        let retrieved_key = match store.get(&*key) {
            Some(res) => res,
            None => return Err(DatabaseError::KeyNotFound),
        };
//        if !user_roles.iter().any(|item| retrieved_key.role_whitelist.contains(item)) {
//            return Err(DatabaseError::Forbidden);
//        } 

        let key = Aes256Gcm::new(&self.key);
        let plaintext = key
            .decrypt(&retrieved_key.nonce, retrieved_key.ciphertext.as_ref())?;

        let hehe = std::str::from_utf8(&plaintext)?;

        let meme = String::from(hehe);
        Ok(meme)
    }

    async fn view_users(self) -> Result<Vec<User>, DatabaseError> { 
        let store = self.users.read().await;

        Ok(store.to_vec()) 
    } 

    async fn get_user_from_password(self, password: String) -> Result<User, DatabaseError> {
        let store = self.users.read().await;

        let user = match store.iter().find(|x| x.passkey == password) {
            Some(user) => user,
            None => return Err(DatabaseError::UserNotFound) 
        };

        Ok(user.clone())
    }

    async fn get_roles_for_user(&self, name: String) -> Result<Vec<Role>, DatabaseError> {
        let user = self.view_user_by_name(name).await.unwrap();

        Ok(user.roles())
    }
    async fn view_user_by_name(&self, id: String) -> Result<User, DatabaseError> { 
        let store = self.users.read().await;

        let user = store.clone().into_iter().find(|x| x.name == id);

        if user.is_none() {
            return Err(DatabaseError::UserNotFound);
        }

        Ok(user.unwrap()) 
    } 

    async fn create_user(self, name: String) -> Result<String, DatabaseError> {  
        let mut store = self.users.write().await;
        
        let user = User {
            name: name.clone(),
            passkey: nanoid!(20),
            roles: Vec::new(),
            jwt: None,
            jwt_expires: None
        };
        let username_is_taken = store.iter().any(|x| x.name == user.name);
        if !username_is_taken {
        store.push(user.clone());
        } else {
            return Err(DatabaseError::UserAlreadyExists);
        }

        Ok(user.passkey)
    }

    async fn delete_user(self, name: String) -> Result<(), DatabaseError> { 
        let mut store = self.users.write().await;
        
        store.retain(|user| user.name == name);

        Ok(())
    }
}

pub struct EncryptedSecret {
    pub nonce: Nonce<U12>,
    pub ciphertext: Vec<u8>,
    pub role_whitelist: Vec<Role>
}

impl EncryptedSecret {
    pub fn new(cipher_key: Key<Aes256Gcm>, _key: String, val: String) -> Self {
        let cipher = Aes256Gcm::new(&cipher_key);
    
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(&nonce, val.as_ref()).unwrap();

        Self { nonce, ciphertext, role_whitelist: Vec::new() }
    }
}
