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

    #[error("{0}")]
    Generic(&'static str),
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

/// Encrypt some bytes using the global configured encryption key.
///
/// # Example
///
/// ```
/// use rwf::crypto::encrypt;
///
/// let ciphertext = encrypt(b"hello world").expect("encryption failed");
/// ```
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

pub fn encrypt_number(n: i64) -> Result<String, Error> {
    let config = get_config();
    let nonce = nonce();

    let key = config.secure_id_key;
    let cipher = Aes128GcmSiv::new(&key);
    let aes_nonce = Nonce::from_slice(&nonce);
    let data = n.to_be_bytes();

    let ciphertext = cipher
        .encrypt(aes_nonce, data.as_slice())
        .expect("aes-128 encryption failed");

    let mut bytes = ciphertext.to_vec();
    bytes.extend(nonce);

    let encrypted = format!("{:02x?}", bytes);

    // Remove the pretty format.
    let split = encrypted[1..encrypted.len() - 1]
        .split(", ")
        .collect::<Vec<_>>();

    // Split into 4 40-bit numbers.
    let part_size = split.len() / 4;

    let mut uuid = Vec::new();
    for i in 0..4 {
        uuid.push(split[i * part_size..i * part_size + part_size].join(""));
    }

    Ok(uuid.join("-"))
}

pub fn decrypt_number(s: &str) -> Result<i64, Error> {
    let config = get_config();

    let key = config.secure_id_key;
    let cipher = Aes128GcmSiv::new(&key);

    // Remove the pretty format.
    let s = s.replace("-", "");

    if s.len() % 2 != 0 {
        return Err(Error::Generic("incorrect secure id format"));
    }

    let bytes = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(0))
        .collect::<Vec<_>>();

    // Should contain at least the nonce.
    if bytes.len() < 12 {
        return Err(Error::Generic("incorrect secure id format"));
    }

    let ciphertext = &bytes[0..bytes.len() - 96 / 8];
    let nonce = &bytes[bytes.len() - 96 / 8..];

    let aes_nonce = Nonce::from_slice(nonce);

    let plaintext = cipher.decrypt(aes_nonce, ciphertext.as_ref())?;

    // Should be a i64-size structure.
    if plaintext.len() != 8 {
        return Err(Error::Generic("incorrect secure id format"));
    }

    Ok(i64::from_be_bytes(plaintext.try_into().unwrap()))
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

    #[test]
    fn test_encrypt_number() {
        let n = 2345;
        let encrypted = encrypt_number(n).unwrap();
        let decrypted = decrypt_number(&encrypted).unwrap();
        assert_eq!(n, decrypted);

        let bad_input = "sdf";
        let result = decrypt_number(&bad_input);
        assert!(result.is_err());
    }
}
