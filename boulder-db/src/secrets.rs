use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, // Or `Aes128Gcm`
    Key,
};
use generic_array::typenum::U12;
use generic_array::GenericArray;

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
