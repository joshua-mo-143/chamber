use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
};
use generic_array::typenum::{U12, U32};
use generic_array::GenericArray;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use sqlx::Row;

pub struct EncryptedSecret {
    pub key: String,
    nonce: Nonce,
    pub ciphertext: Vec<u8>,
    tags: Vec<String>,
    access_level: i32,
    role_whitelist: Vec<String>,
}


impl<'a> FromRow<'a, PgRow> for EncryptedSecret {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let vec: Vec<u8> = row.try_get("nonce")?;
        let nonce: GenericArray<u8, U12> = *GenericArray::from_slice(&vec[..]);

        let nonce = Nonce(nonce);

        Ok(Self {
            key: row.try_get("key")?,
            nonce, 
            ciphertext: row.try_get("ciphertext")?,
            tags: row.try_get("tags")?,
            access_level: row.try_get("access_level")?,
            role_whitelist: row.try_get("role_whitelist")?,
            }
        )
    }
}

#[derive(sqlx::FromRow, Clone, Default)]
pub struct EncryptedSecretBuilder {
    pub key: String,
    value: String,
    tags: Option<Vec<String>>,
    access_level: Option<i32>,
    role_whitelist: Option<Vec<String>>,
}

impl EncryptedSecretBuilder {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            .. Default::default()
        }
    }

    pub fn with_tags(mut self, tags: Option<Vec<String>>) -> Self {
        if let Some(tags) = tags {
        self.tags = Some(tags);
        }
        self
    }

    pub fn with_access_level(mut self, access_level: Option<i32>) -> Self {
        if let Some(access_level) = access_level {
        self.access_level = Some(access_level);
        }
        self
    }

    pub fn with_whitelist(mut self, role_whitelist: Option<Vec<String>>) -> Self {
        if let Some(role_whitelist) = role_whitelist {
        self.role_whitelist = Some(role_whitelist);
        }
        self
    }

    pub fn build(self, key: Key<Aes256Gcm>) -> EncryptedSecret { 
        let cipher = Aes256Gcm::new(&key);

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(&nonce, self.value.as_ref()).unwrap();

        EncryptedSecret {
           key: self.key,
           nonce: Nonce(nonce),
           ciphertext,
           tags: if let Some(tags) = self.tags {    
                tags
           } else {Vec::new()},
           access_level: if let Some(access_level) = self.access_level {access_level} else {0},
           role_whitelist: if let Some(whitelist) = self.role_whitelist {whitelist} else {Vec::new()}
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct Secret {
    #[sqlx(try_from = "Vec<u8>")]
    pub nonce: Nonce,
    pub ciphertext: Vec<u8>,
}

#[derive(sqlx::FromRow, Clone, Serialize, Deserialize, Debug)]
pub struct SecretInfo {
    pub key: String,
    pub tags: Vec<String>,
    pub access_level: i32,
    pub role_whitelist: Vec<String>,
}

impl<'a> EncryptedSecret {
    pub fn key(&'a self) -> &'a str {
        &self.key
    }

    pub fn nonce(&self) -> GenericArray<u8, U12> {
        self.nonce.0
    }

    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    pub fn nonce_as_u8(&'a self) -> &'a [u8] {
        &self.nonce.0
    }

    pub fn tags(&'a self) -> Vec<&'a str> {
        self.tags.iter().map(AsRef::as_ref).collect()
    }

    pub fn remove_all_tags(mut self) {
        self.tags = Vec::new();
    }

    pub fn add_tag(mut self, string: &str) {
        self.tags.push(string.to_owned());
    }

    pub fn replace_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
    }

    pub fn remove_tag(mut self, tag: &str) {
        self.tags.retain(|x| x == tag);
    }

    pub fn access_level(&self) -> i32 {
        self.access_level
    }

    pub fn set_access_level(&mut self, level: Option<i32>) {
        if let Some(level) = level {
            self.access_level = level;
        }
    }

    pub fn role_whitelist(&'a self) -> Vec<&'a str> {
        self.role_whitelist.iter().map(AsRef::as_ref).collect()
    }

    pub fn set_role_whitelist(&mut self, whitelist: Option<Vec<String>>) {
        if let Some(mut whitelist) = whitelist {
            self.role_whitelist.append(&mut whitelist);
        }
    }

    pub fn add_role_to_whitelist(&mut self, role: String) {
        self.role_whitelist.push(role);
    }

    pub fn remove_role_from_whitelist(&mut self, role: String) {
        self.role_whitelist.retain(|x| x != &role);
    }
}

#[derive(Clone)]
pub struct Nonce(pub GenericArray<u8, U12>);

impl From<Vec<u8>> for Nonce {
    fn from(vec: Vec<u8>) -> Self {
        let nonce: GenericArray<u8, U12> = *GenericArray::from_slice(&vec[..]);

        Self(nonce)
    }
}

#[derive(Clone)]
struct SerializeKey(GenericArray<u8, U32>);

impl SerializeKey {
    pub fn new() -> Self {
        Self(Aes256Gcm::generate_key(OsRng))
    }
}

impl Serialize for SerializeKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec = self.0.to_vec();
        serializer.serialize_bytes(&vec)
    }
}

impl<'de> Deserialize<'de> for SerializeKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Deserialize::deserialize(deserializer)
            .map(ByteBuf::into_vec)
            .unwrap();

        let bytes: GenericArray<u8, U32> = *GenericArray::from_slice(&bytes[..]);

        Ok(Self(bytes))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyFile {
    unseal_key: String,
    crypto_key: SerializeKey,
}

impl KeyFile {
    pub fn new() -> Self {
        Self {
            unseal_key: nanoid::nanoid!(100),
            crypto_key: SerializeKey::new(),
        }
    }

    pub fn from_key(string: &str) -> Self {
        Self {
            unseal_key: string.to_owned(),
            crypto_key: SerializeKey::new(),
        }
    }

    pub fn unseal_key(self) -> String {
        self.unseal_key
    }

    pub fn crypto_key(self) -> GenericArray<u8, U32> {
        self.crypto_key.0
    }
}

impl Default for KeyFile {
    fn default() -> Self {
        Self::new()
    }
}
