use crate::errors::DatabaseError;
use num_traits::cast::ToPrimitive;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;
use ring::{
    aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey},
    error::Unspecified,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;
use sqlx::types::BigDecimal;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::consts::KEYFILE_PATH;

#[derive(sqlx::FromRow, Zeroize, ZeroizeOnDrop)]
pub struct EncryptedSecret {
    pub key: String,
    #[sqlx(try_from = "BigDecimal")]
    pub nonce: U64Wrapper,
    pub ciphertext: Vec<u8>,
    tags: Vec<String>,
    access_level: i32,
    role_whitelist: Vec<String>,
}

#[derive(Default)]
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
            ..Default::default()
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

    pub fn build(
        self,
        mut sealing_key: SealingKey<NonceCounter>,
        nonce_num: u64,
    ) -> EncryptedSecret {
        let aad = Aad::empty();

        let mut transformed_in_place: Vec<u8> = self.value.into_bytes();

        sealing_key
            .seal_in_place_append_tag(aad, &mut transformed_in_place)
            .unwrap();

        EncryptedSecret {
            key: self.key,
            nonce: U64Wrapper(nonce_num),
            ciphertext: transformed_in_place,
            tags: if let Some(tags) = self.tags {
                tags
            } else {
                Vec::new()
            },
            access_level: if let Some(access_level) = self.access_level {
                access_level
            } else {
                0
            },
            role_whitelist: if let Some(whitelist) = self.role_whitelist {
                whitelist
            } else {
                Vec::new()
            },
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct Secret {
    #[sqlx(try_from = "BigDecimal")]
    pub nonce: U64Wrapper,
    pub ciphertext: Vec<u8>,
}

impl Secret {
    pub fn decrypt(&self, mut seq: OpeningKey<NonceCounter>) -> String {
        let aad = Aad::empty();

        let mut tag = self.ciphertext.clone();

        let plaintext = seq.open_in_place(aad, &mut tag).unwrap();

        String::from_utf8(plaintext.to_vec()).unwrap()
    }
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

    pub fn nonce(&self) -> u64 {
        self.nonce.0
    }

    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
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

    pub fn reencrypt(
        &mut self,
        mut open_key: OpeningKey<NonceCounter>,
        mut sealing_key: SealingKey<NonceCounter>
        ) {
        let aad = Aad::empty();

        let mut tag = self.ciphertext.clone();

        let key = open_key.open_in_place(aad, &mut tag).unwrap();

        let plaintext = String::from_utf8(key.to_vec()).unwrap();
        
        let mut transformed_in_place: Vec<u8> = plaintext.into_bytes();

        sealing_key
            .seal_in_place_append_tag(aad, &mut transformed_in_place)
            .unwrap();

        self.ciphertext = transformed_in_place;

    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SerializeKey(pub Vec<u8>);

impl SerializeKey {
    pub fn new() -> Self {
        let rand = SystemRandom::new();
        let mut key: [u8; 32] = [0u8; 32];
        let _ = rand.fill(&mut key);
        Self(key.to_vec())
    }

    pub fn make_key(&self) -> ring::aead::UnboundKey {
        ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, &self.0).unwrap()
    }
}

impl Default for SerializeKey {
    fn default() -> Self {
        Self::new()
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

        Ok(Self(bytes))
    }
}

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyFile {
    unlock_key: String,
    crypto_key: SerializeKey,
    pub nonce_number: u64,
}

impl<'b> KeyFile {
    pub fn new() -> Self {
        Self {
            unlock_key: nanoid::nanoid!(100),
            crypto_key: SerializeKey::new(),
            nonce_number: 1,
        }
    }

    pub fn crypto_key(&'b self) -> &'b SerializeKey {
        &self.crypto_key
    }

    pub fn from_key(string: &str) -> Self {
        Self {
            unlock_key: string.to_owned(),
            crypto_key: SerializeKey::new(),
            nonce_number: 1,
        }
    }

    pub fn unseal_key(&'b self) -> &'b str {
        &self.unlock_key
    }

    pub fn save(&self) -> Result<(), DatabaseError> {
        let _thing = self;
        let encoded = bincode::serialize(&self).unwrap();

        match std::fs::write(KEYFILE_PATH, encoded) {
            Ok(res) => res,
            Err(e) => return Err(DatabaseError::IoError(e)),
        };
        Ok(())
    }

    pub fn get_crypto_seal_key(&mut self) -> SealingKey<NonceCounter> {
        let nonce_sequence = NonceCounter(self.nonce_number);

        let unbound_key = self.crypto_key.make_key();
        self.nonce_number += 1;

        let _ = self.save();
        SealingKey::new(unbound_key, nonce_sequence)
    }

    pub fn get_crypto_open_key(&self, num: u64) -> OpeningKey<NonceCounter> {
        let nonce_sequence = NonceCounter(num);

        let unbound_key = self.crypto_key.make_key();
        OpeningKey::new(unbound_key, nonce_sequence)
    }
}

impl Default for KeyFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct U64Wrapper(pub u64);

impl U64Wrapper {
    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl From<BigDecimal> for U64Wrapper {
    fn from(decimal: BigDecimal) -> Self {
        Self(decimal.to_u64().unwrap())
    }
}

pub struct NonceCounter(u64);

impl Default for NonceCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl NonceCounter {
    pub fn new() -> Self {
        Self(1)
    }

    pub fn from_num(num: u64) -> Self {
        Self(num)
    }
}

impl NonceSequence for NonceCounter {
    // called once for each seal operation
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes: [u8; 12] = [0; 12];

        let bytes = self.0.to_be_bytes();
        nonce_bytes[4..].copy_from_slice(&bytes);

        self.0 += 1; // advance the counter
        Ok(Nonce::try_assume_unique_for_key(&nonce_bytes).unwrap())
    }
}
