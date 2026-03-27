use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

const MASTER_KEY_SALT: &[u8] = b"visor-vps-v1-salt-2024";

/// Derive a 32-byte key from a machine-specific secret
fn derive_key() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(MASTER_KEY_SALT);
    // Use a stable machine identifier (hostname or fixed secret)
    hasher.update(b"visor-vps-local-encryption-key");
    hasher.finalize().into()
}

pub fn encrypt_password(plaintext: &str) -> Result<String> {
    let key_bytes = derive_key();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext, then base64-encode
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(general_purpose::STANDARD.encode(&combined))
}

pub fn decrypt_password(encoded: &str) -> Result<String> {
    let combined = general_purpose::STANDARD.decode(encoded)?;
    if combined.len() < 12 {
        return Err(anyhow::anyhow!("Invalid ciphertext"));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key_bytes = derive_key();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    Ok(String::from_utf8(plaintext)?)
}
