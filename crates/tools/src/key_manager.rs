use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::Rng;
use std::fmt;
use zeroize::Zeroize;

/// A secure key container that zeroizes memory when dropped
#[derive(Clone)]
pub struct EncryptedKey {
    /// Nonce used for encryption
    nonce: Vec<u8>,
    /// Encrypted key material
    ciphertext: Vec<u8>,
}

impl fmt::Debug for EncryptedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedKey")
            .field("nonce_len", &self.nonce.len())
            .field("ciphertext_len", &self.ciphertext.len())
            .finish()
    }
}

impl Drop for EncryptedKey {
    fn drop(&mut self) {
        self.nonce.zeroize();
        self.ciphertext.zeroize();
    }
}

/// Key Manager for encrypting and decrypting private keys
#[derive(Debug)]
pub struct KeyManager {
    master_key: [u8; 32],
    salt: [u8; 16],
}

impl Drop for KeyManager {
    fn drop(&mut self) {
        self.master_key.zeroize();
        self.salt.zeroize();
    }
}

impl KeyManager {
    /// Initialize KeyManager from a master password/key.
    /// Derives a 256-bit key using Argon2id with a random salt.
    #[must_use]
    pub fn from_password(password: &str) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let mut salt = [0u8; 16];
        rng.fill(&mut salt);
        Self::from_password_with_salt(password, &salt)
    }

    /// Initialize KeyManager from password with a specific salt (for loading).
    #[must_use]
    pub fn from_password_with_salt(password: &str, salt: &[u8; 16]) -> Result<Self> {
        let params = Params::new(19_456, 2, 1, Some(32))
            .map_err(|e| anyhow::anyhow!("Failed to create Argon2 parameters: {}", e))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut master_key = [0u8; 32];
        argon
            .hash_password_into(password.as_bytes(), salt, &mut master_key)
            .map_err(|e| anyhow::anyhow!("Argon2 key derivation failed: {}", e))?;

        Ok(Self {
            master_key,
            salt: *salt,
        })
    }

    /// Initialize KeyManager from a 32-byte hex string.
    #[must_use]
    pub fn from_hex_key(hex_key: &str) -> Result<Self> {
        let key_bytes = hex::decode(hex_key).context("Failed to decode hex key")?;
        if key_bytes.len() != 32 {
            anyhow::bail!("Key must be exactly 32 bytes, got {}", key_bytes.len());
        }

        let mut rng = rand::thread_rng();
        let mut salt = [0u8; 16];
        rng.fill(&mut salt);

        let mut master_key = [0u8; 32];
        master_key.copy_from_slice(&key_bytes);

        Ok(Self { master_key, salt })
    }

    /// Encrypt a private key (secret key) using AES-256-GCM
    pub fn encrypt_key(&self, secret_key: &str) -> Result<EncryptedKey> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        // Generate a random 96-bit nonce (12 bytes for GCM)
        let mut rng = rand::thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the secret key
        let ciphertext = cipher
            .encrypt(nonce, Payload::from(secret_key.as_bytes()))
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedKey {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
        })
    }

    /// Decrypt a private key using AES-256-GCM
    pub fn decrypt_key(&self, encrypted: &EncryptedKey) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, Payload::from(encrypted.ciphertext.as_ref()))
            .map_err(|e| anyhow::anyhow!("Decryption failed (wrong key or corrupted data): {}", e))?;

        String::from_utf8(plaintext).context("Decrypted key is not valid UTF-8")
    }

    /// Get the salt used for key derivation
    pub fn get_salt(&self) -> &[u8; 16] {
        &self.salt
    }

    /// Export encrypted key as hex string for storage
    pub fn export_encrypted(&self, secret_key: &str) -> Result<String> {
        let encrypted = self.encrypt_key(secret_key)?;

        // Format: <nonce_hex>:<ciphertext_hex>
        let nonce_hex = hex::encode(&encrypted.nonce);
        let ciphertext_hex = hex::encode(&encrypted.ciphertext);

        Ok(format!("{}:{}", nonce_hex, ciphertext_hex))
    }

    /// Import encrypted key from hex string
    pub fn import_encrypted(&self, encrypted_hex: &str) -> Result<EncryptedKey> {
        let parts: Vec<&str> = encrypted_hex.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid encrypted key format. Expected 'nonce:ciphertext'");
        }

        let nonce = hex::decode(parts[0]).context("Failed to decode nonce")?;
        let ciphertext = hex::decode(parts[1]).context("Failed to decode ciphertext")?;

        if nonce.len() != 12 {
            anyhow::bail!("Invalid nonce length: expected 12 bytes, got {}", nonce.len());
        }

        Ok(EncryptedKey { nonce, ciphertext })
    }

    /// Validate a secret key format (basic Stellar check).
    /// Must start with 'S' and be at least 56 characters.
    #[must_use]
    pub fn validate_secret_key(secret_key: &str) -> Result<()> {
        if !secret_key.starts_with('S') {
            anyhow::bail!("Secret key must start with 'S' (Stellar format)");
        }
        if secret_key.len() < 56 {
            anyhow::bail!("Secret key is too short");
        }
        Ok(())
    }

    /// Validate a public key format (basic Stellar check).
    /// Must start with 'G' and be at least 56 characters.
    #[must_use]
    pub fn validate_public_key(public_key: &str) -> Result<()> {
        if !public_key.starts_with('G') {
            anyhow::bail!("Public key must start with 'G' (Stellar format)");
        }
        if public_key.len() < 56 {
            anyhow::bail!("Public key is too short");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() -> Result<()> {
        let manager = KeyManager::from_password("my_secure_password")?;
        let secret_key = "SBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU";

        let encrypted = manager.encrypt_key(secret_key)?;
        let decrypted = manager.decrypt_key(&encrypted)?;

        assert_eq!(decrypted, secret_key);
        Ok(())
    }

    #[test]
    fn test_export_import_roundtrip() -> Result<()> {
        let manager = KeyManager::from_password("another_password")?;
        let secret_key = "SBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU";

        let exported = manager.export_encrypted(secret_key)?;
        let imported = manager.import_encrypted(&exported)?;
        let decrypted = manager.decrypt_key(&imported)?;

        assert_eq!(decrypted, secret_key);
        Ok(())
    }

    #[test]
    fn test_wrong_password_fails() -> Result<()> {
        let manager1 = KeyManager::from_password("password1")?;
        let secret_key = "SBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU";

        let encrypted = manager1.encrypt_key(secret_key)?;
        let salt1 = manager1.get_salt();

        // Manager2 must use same salt to test password difference
        let manager2 = KeyManager::from_password_with_salt("password2", salt1)?;
        let result = manager2.decrypt_key(&encrypted);

        // Should fail due to wrong password
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_password_with_same_salt_derives_same_key() -> Result<()> {
        let salt = [42u8; 16];
        let password = "test_password";

        let manager1 = KeyManager::from_password_with_salt(password, &salt)?;
        let manager2 = KeyManager::from_password_with_salt(password, &salt)?;

        // Same password and salt should derive identical keys
        assert_eq!(manager1.master_key, manager2.master_key);
        Ok(())
    }

    #[test]
    fn test_validate_secret_key() {
        assert!(KeyManager::validate_secret_key("SBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU").is_ok());
        assert!(KeyManager::validate_secret_key("GBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU").is_err());
        assert!(KeyManager::validate_secret_key("short").is_err());
    }

    #[test]
    fn test_validate_public_key() {
        assert!(KeyManager::validate_public_key("GBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU").is_ok());
        assert!(KeyManager::validate_public_key("SBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU").is_err());
    }

    #[test]
    fn test_encrypted_key_zeroizes_on_drop() {
        let manager = KeyManager::from_password("password").unwrap();
        let secret_key = "SBZXVMIRWXL5VZVKXWV2FGKYTQ5VV5VRNJYQVZKYWW3XYVYP3IXGKDU";
        let _encrypted = manager.encrypt_key(secret_key).unwrap();
        // When _encrypted goes out of scope, it should zeroize
        // (can't directly test zeroization, but ensures Drop runs without panic)
    }
}
