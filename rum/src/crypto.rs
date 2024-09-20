use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes128GcmSiv, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::get_config;

#[derive(Error, Debug)]
pub enum Error {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("aes error: {0}")]
    AesError(aes_gcm_siv::Error),
}

impl From<aes_gcm_siv::Error> for Error {
    fn from(error: aes_gcm_siv::Error) -> Error {
        Error::AesError(error)
    }
}

fn nonce() -> Vec<u8> {
    rand::thread_rng().gen::<[u8; 96 / 8]>().to_vec()
}

#[derive(Serialize, Deserialize)]
struct Encrypted {
    #[serde(rename = "c")]
    ciphertext: Vec<u8>,

    #[serde(rename = "n")]
    nonce: Vec<u8>,
}

impl Encrypted {
    fn to_base64(&self) -> Result<String, Error> {
        let json = serde_json::to_string(self)?;
        Ok(general_purpose::STANDARD_NO_PAD.encode(&json))
    }

    fn from_base64(value: &str) -> Result<Self, Error> {
        let decoded = general_purpose::STANDARD_NO_PAD.decode(value)?;
        Ok(serde_json::from_slice(&decoded)?)
    }

    fn to_bytes(&self) -> Result<String, Error> {
        Ok(self.to_base64()?)
    }
}

pub fn encrypt(data: &[u8]) -> Result<String, Error> {
    let config = get_config();
    let nonce = nonce();

    let key = config.aes_key;
    let cipher = Aes128GcmSiv::new(&key);
    let aes_nonce = Nonce::from_slice(&nonce); // 96-bits; unique per message
    let ciphertext = cipher
        .encrypt(aes_nonce, data)
        .expect("aes-128 encryption failed");

    Encrypted { ciphertext, nonce }.to_bytes()
}

pub fn decrypt(data: &str) -> Result<Vec<u8>, Error> {
    let config = get_config();
    let encrypted = Encrypted::from_base64(data)?;

    let key = config.aes_key;
    let cipher = Aes128GcmSiv::new(&key);
    let aes_nonce = Nonce::from_slice(&encrypted.nonce);
    let plaintext = cipher.decrypt(aes_nonce, encrypted.ciphertext.as_ref())?;

    Ok(plaintext)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let text = "test hello world";
        let cipher = encrypt(text.as_bytes()).expect("encrypt");
        let plain = decrypt(&cipher).expect("decrypt");
        assert_eq!(text, String::from_utf8_lossy(&plain));
    }
}
