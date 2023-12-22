use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
};
use generic_array::typenum::{U12, U32};
use generic_array::GenericArray;

use serde_bytes::ByteBuf;
use serde::{Deserialize, Serialize, Serializer, Deserializer};

#[derive(sqlx::FromRow)]
pub struct EncryptedSecret {
    #[sqlx(try_from = "Vec<u8>")]
    nonce: Nonce,
    pub ciphertext: Vec<u8>,
}

impl EncryptedSecret {
    pub fn nonce(&self) -> GenericArray<u8, U12> {
        self.nonce.0
    }

    pub fn nonce_as_u8(&self) -> Vec<u8> {
        self.nonce.0.to_vec()
    }
}

impl From<Vec<u8>> for Nonce {
    fn from(vec: Vec<u8>) -> Self {
        let nonce: GenericArray<u8, U12> = *GenericArray::from_slice(&vec[..]);

        Self(nonce)
    }
}

struct Nonce(GenericArray<u8, U12>);

impl EncryptedSecret {
    pub fn new(cipher_key: Key<Aes256Gcm>, _key: String, val: String) -> Self {
        let cipher = Aes256Gcm::new(&cipher_key);

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(&nonce, val.as_ref()).unwrap();

        Self { nonce: Nonce(nonce), ciphertext }
    }
}

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
        let bytes = Deserialize::deserialize(deserializer).map(ByteBuf::into_vec).unwrap();

        let bytes: GenericArray<u8, U32> = *GenericArray::from_slice(&bytes[..]);

        Ok(Self(bytes))
    }
}

#[derive(Serialize, Deserialize)]
pub struct KeyFile {
    unseal_key: String,
    crypto_key: SerializeKey
}

impl KeyFile {
    pub fn new() -> Self {
        Self {
            unseal_key: nanoid::nanoid!(100),
            crypto_key: SerializeKey::new(),

        }
    }

    pub fn unseal_key(self) -> String {
        self.unseal_key
    }
}

impl Default for KeyFile {
    fn default() -> Self {
        Self::new()
    }
}
