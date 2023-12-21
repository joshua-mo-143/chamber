use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
    Nonce,
};
use generic_array::typenum::U12;
use generic_array::GenericArray;

pub struct EncryptedSecret {
    pub nonce: Nonce<U12>,
    pub ciphertext: Vec<u8>,
}

impl EncryptedSecret {
    pub fn new(cipher_key: Key<Aes256Gcm>, _key: String, val: String) -> Self {
        let cipher = Aes256Gcm::new(&cipher_key);

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(&nonce, val.as_ref()).unwrap();

        Self { nonce, ciphertext }
    }
}

impl From<EncryptedSecretPG> for EncryptedSecret {
    fn from(secret: EncryptedSecretPG) -> Self {
        let nonce: GenericArray<u8, U12> = *GenericArray::from_slice(&secret.nonce[..]);

        Self {
            nonce,
            ciphertext: secret.ciphertext,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct EncryptedSecretPG {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl From<EncryptedSecret> for EncryptedSecretPG {
    fn from(secret: EncryptedSecret) -> Self {
        Self {
            nonce: secret.nonce.to_vec(),
            ciphertext: secret.ciphertext,
        }
    }
}
