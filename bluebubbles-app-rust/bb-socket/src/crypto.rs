//! AES-256-CBC encryption compatible with CryptoJS/OpenSSL format.
//!
//! The BlueBubbles server encrypts socket payloads using CryptoJS AES,
//! which uses the OpenSSL EVP_BytesToKey key derivation function with
//! MD5 hashing and a random 8-byte salt.
//!
//! Format of encrypted data (Base64-decoded):
//! ```text
//! "Salted__" (8 bytes) + salt (8 bytes) + ciphertext
//! ```

use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use md5::{Md5, Digest};
use bb_core::error::{BbError, BbResult};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

/// AES-256-CBC encryption/decryption compatible with CryptoJS format.
pub struct AesCrypto;

impl AesCrypto {
    /// Decrypt a CryptoJS-formatted Base64 ciphertext using the given password.
    ///
    /// The ciphertext format is: "Salted__" + 8-byte salt + encrypted data.
    /// Key and IV are derived using OpenSSL's EVP_BytesToKey with MD5.
    pub fn decrypt(password: &str, base64_ciphertext: &str) -> BbResult<String> {
        use base64::Engine;
        let raw = base64::engine::general_purpose::STANDARD
            .decode(base64_ciphertext)
            .map_err(|e| BbError::Crypto(format!("base64 decode failed: {e}")))?;

        if raw.len() < 16 {
            return Err(BbError::Crypto("ciphertext too short".into()));
        }

        // Verify "Salted__" prefix
        if &raw[..8] != b"Salted__" {
            return Err(BbError::Crypto("missing Salted__ prefix".into()));
        }

        let salt = &raw[8..16];
        let ciphertext = &raw[16..];

        // Derive key (32 bytes) and IV (16 bytes) using EVP_BytesToKey with MD5
        let (key, iv) = Self::evp_bytes_to_key(password.as_bytes(), salt);

        // Decrypt
        let mut buf = ciphertext.to_vec();
        let decryptor = Aes256CbcDec::new_from_slices(&key, &iv)
            .map_err(|e| BbError::Crypto(format!("cipher init failed: {e}")))?;

        let decrypted = decryptor
            .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
            .map_err(|e| BbError::Crypto(format!("decryption failed: {e}")))?;

        String::from_utf8(decrypted.to_vec())
            .map_err(|e| BbError::Crypto(format!("utf8 decode failed: {e}")))
    }

    /// Encrypt plaintext using CryptoJS-compatible AES-256-CBC.
    ///
    /// Returns a Base64-encoded string in the "Salted__" + salt + ciphertext format.
    pub fn encrypt(password: &str, plaintext: &str) -> BbResult<String> {
        use base64::Engine;
        use rand::RngCore;

        // Generate random 8-byte salt
        let mut salt = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut salt);

        let (key, iv) = Self::evp_bytes_to_key(password.as_bytes(), &salt);

        let encryptor = Aes256CbcEnc::new_from_slices(&key, &iv)
            .map_err(|e| BbError::Crypto(format!("cipher init failed: {e}")))?;

        let plaintext_bytes = plaintext.as_bytes();
        // Allocate buffer with space for PKCS7 padding (up to 16 extra bytes)
        let mut buf = vec![0u8; plaintext_bytes.len() + 16];
        buf[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

        let encrypted = encryptor
            .encrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(
                &mut buf,
                plaintext_bytes.len(),
            )
            .map_err(|e| BbError::Crypto(format!("encryption failed: {e}")))?;

        // Build output: "Salted__" + salt + ciphertext
        let mut output = Vec::with_capacity(16 + encrypted.len());
        output.extend_from_slice(b"Salted__");
        output.extend_from_slice(&salt);
        output.extend_from_slice(encrypted);

        Ok(base64::engine::general_purpose::STANDARD.encode(&output))
    }

    /// OpenSSL EVP_BytesToKey implementation using MD5.
    ///
    /// Derives a 32-byte key and 16-byte IV from password and salt.
    fn evp_bytes_to_key(password: &[u8], salt: &[u8]) -> ([u8; 32], [u8; 16]) {
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];

        // We need 48 bytes total (32 key + 16 IV)
        // Each MD5 round produces 16 bytes
        let mut derived = Vec::with_capacity(48);

        let mut prev_hash = Vec::new();
        while derived.len() < 48 {
            let mut hasher = Md5::new();
            if !prev_hash.is_empty() {
                hasher.update(&prev_hash);
            }
            hasher.update(password);
            hasher.update(salt);
            prev_hash = hasher.finalize().to_vec();
            derived.extend_from_slice(&prev_hash);
        }

        key.copy_from_slice(&derived[..32]);
        iv.copy_from_slice(&derived[32..48]);

        (key, iv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = "test-password-123";
        let plaintext = "Hello, BlueBubbles!";

        let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
        let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let result = AesCrypto::decrypt("pass", "not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        use base64::Engine;
        let short = base64::engine::general_purpose::STANDARD.encode(b"short");
        let result = AesCrypto::decrypt("pass", &short);
        assert!(result.is_err());
    }

    #[test]
    fn test_evp_bytes_to_key_deterministic() {
        let (key1, iv1) = AesCrypto::evp_bytes_to_key(b"password", b"saltsalt");
        let (key2, iv2) = AesCrypto::evp_bytes_to_key(b"password", b"saltsalt");
        assert_eq!(key1, key2);
        assert_eq!(iv1, iv2);
    }
}
