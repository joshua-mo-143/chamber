use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
};
use generic_array::typenum::{U12, U32};
use generic_array::GenericArray;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;

#[derive(sqlx::FromRow, Clone)]
pub struct EncryptedSecret {
    pub key: String,
    #[sqlx(try_from = "Vec<u8>")]
    nonce: Nonce,
    pub ciphertext: Vec<u8>,
    tags: Vec<String>,
    access_level: i32,
    role_whitelist: Vec<String>,
}

#[derive(sqlx::FromRow, Clone)]
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

impl SecretInfo {
    pub fn from_encrypted(es: EncryptedSecret, key: String) -> Self {
        Self {
            key,
            tags: es.clone().tags(),
            access_level: es.access_level(),
            role_whitelist: es.role_whitelist(),
        }
    }
}

impl EncryptedSecret {
    pub fn new(cipher_key: Key<Aes256Gcm>, key: String, val: String) -> Self {
        let cipher = Aes256Gcm::new(&cipher_key);

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(&nonce, val.as_ref()).unwrap();

        Self {
            key,
            nonce: Nonce(nonce),
            ciphertext,
            tags: Vec::new(),
            access_level: 0,
            role_whitelist: Vec::new(),
        }
    }

    pub fn nonce(&self) -> GenericArray<u8, U12> {
        self.nonce.0
    }

    pub fn nonce_as_u8(&self) -> Vec<u8> {
        self.nonce.0.to_vec()
    }

    pub fn tags(self) -> Vec<String> {
        self.tags
    }

    pub fn replace_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
    }
    pub fn remove_all_tags(mut self) {
        self.tags = Vec::new();
    }

    pub fn add_tags(mut self, vec: Option<Vec<String>>) {
        if let Some(mut vec) = vec {
            self.tags.append(&mut vec);
        }
    }

    pub fn add_tag(mut self, string: &str) {
        self.tags.push(string.to_owned());
    }

    pub fn remove_tag(mut self, tag: &str) {
        self.tags.retain(|x| x == &tag.to_owned());
    }

    pub fn access_level(&self) -> i32 {
        self.access_level
    }

    pub fn set_access_level(&mut self, level: Option<i32>) {
        if let Some(level) = level {
            self.access_level = level;
        }
    }

    pub fn role_whitelist(self) -> Vec<String> {
        self.role_whitelist
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
