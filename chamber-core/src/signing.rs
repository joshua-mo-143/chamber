use rand::rngs::OsRng;
use ed25519_dalek::SigningKey;
use ed25519_dalek::Signature;
use crate::errors::DatabaseError;
use std::path::Path;
use serde::{Serialize, Deserialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub static SIGNING_KEY_PATH: &str = "data/signing_key.bin";

pub fn check_signing_key_exists() -> Result<(), DatabaseError> {
    if Path::new(SIGNING_KEY_PATH).exists() {
       return Ok(()); 
    } 

    println!("The signing key file doesn't exist! :(");
    println!("Generating one now...");
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);

    std::fs::write(SIGNING_KEY_PATH, signing_key.to_keypair_bytes()).unwrap();

    println!("Signing key generated.");

    Ok(())
}

pub fn fetch_signing_key() -> Result<SigningKey, DatabaseError> {
    let bytes = std::fs::read(SIGNING_KEY_PATH).unwrap();

    let bytes: [u8; 64] = bytes.try_into().unwrap();

    Ok(SigningKey::from_keypair_bytes(&bytes).unwrap())
}

pub fn verify_bytes(message: &[u8], signature: &[u8; 64], signing_key: SigningKey) -> Result<(), DatabaseError> {
    let sig: Signature = Signature::from_bytes(signature);

    signing_key.verify(message, &sig).unwrap();

    Ok(())
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SigWrapper([u8; 64]);

impl From<Vec<u8>> for SigWrapper {
    fn from(vec: Vec<u8>) -> Self {
        let bytes: [u8; 64] = vec.try_into().unwrap();
        let sig = Signature::from_bytes(&bytes);

        Self(sig.into())
    }
}

impl SigWrapper {
    pub fn new(bytes: Signature) -> Self {
        Self(bytes.to_bytes())
    }

    pub fn inner(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn as_sig(&self) -> Signature {
        let sig = Signature::from_bytes(&self.0);

        sig
    }
}
