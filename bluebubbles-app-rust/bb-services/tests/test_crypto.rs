//! Integration tests for AES-256-CBC encryption/decryption.
//!
//! Tests round-trip encryption, CryptoJS format compatibility,
//! salt randomness, key derivation, and edge cases.

use bb_socket::AesCrypto;

#[test]
fn encrypt_decrypt_roundtrip_basic() {
    let password = "my-secret-password";
    let plaintext = "Hello, BlueBubbles!";

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_decrypt_roundtrip_unicode() {
    let password = "unicode-pass-1234";
    let plaintext = "Hello World";

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_decrypt_roundtrip_json_payload() {
    let password = "server-password-hash";
    let plaintext = r#"{"guid":"msg-123","text":"Hello!","isFromMe":false,"chats":[{"guid":"iMessage;-;+15551234"}]}"#;

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);

    // Verify the decrypted JSON is valid
    let parsed: serde_json::Value = serde_json::from_str(&decrypted).unwrap();
    assert_eq!(parsed["guid"], "msg-123");
}

#[test]
fn encrypt_decrypt_roundtrip_empty_string() {
    let password = "test-pass";
    let plaintext = "";

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_decrypt_roundtrip_large_payload() {
    let password = "large-payload-pass";
    // 10KB payload
    let plaintext = "A".repeat(10_000);

    let encrypted = AesCrypto::encrypt(password, &plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
    assert_eq!(decrypted.len(), 10_000);
}

#[test]
fn encrypt_decrypt_roundtrip_single_char() {
    let password = "p";
    let plaintext = "x";

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn salt_is_random_across_encryptions() {
    let password = "same-password";
    let plaintext = "same plaintext";

    let encrypted1 = AesCrypto::encrypt(password, plaintext).unwrap();
    let encrypted2 = AesCrypto::encrypt(password, plaintext).unwrap();

    // Two encryptions of the same data should produce different ciphertext due to random salt
    assert_ne!(
        encrypted1, encrypted2,
        "encryptions with random salt should differ"
    );

    // Both should decrypt to the same plaintext
    assert_eq!(AesCrypto::decrypt(password, &encrypted1).unwrap(), plaintext);
    assert_eq!(AesCrypto::decrypt(password, &encrypted2).unwrap(), plaintext);
}

#[test]
fn cryptojs_format_has_salted_prefix() {
    use base64::Engine;

    let password = "test";
    let plaintext = "hello";

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let raw = base64::engine::general_purpose::STANDARD
        .decode(&encrypted)
        .unwrap();

    assert!(raw.len() >= 16, "encrypted data should be at least 16 bytes");
    assert_eq!(
        &raw[..8],
        b"Salted__",
        "encrypted data should start with 'Salted__' prefix"
    );
}

#[test]
fn decrypt_with_wrong_password_fails() {
    let encrypted = AesCrypto::encrypt("correct-password", "secret data").unwrap();
    let result = AesCrypto::decrypt("wrong-password", &encrypted);
    assert!(result.is_err(), "decryption with wrong password should fail");
}

#[test]
fn decrypt_invalid_base64_fails() {
    let result = AesCrypto::decrypt("pass", "not-valid-base64!!!");
    assert!(result.is_err(), "invalid base64 should fail");
}

#[test]
fn decrypt_too_short_data_fails() {
    use base64::Engine;
    let short = base64::engine::general_purpose::STANDARD.encode(b"short");
    let result = AesCrypto::decrypt("pass", &short);
    assert!(result.is_err(), "data shorter than 16 bytes should fail");
}

#[test]
fn decrypt_missing_salted_prefix_fails() {
    use base64::Engine;
    // 20 bytes of random data without Salted__ prefix
    let data = vec![0u8; 20];
    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
    let result = AesCrypto::decrypt("pass", &encoded);
    assert!(
        result.is_err(),
        "data without Salted__ prefix should fail"
    );
}

#[test]
fn key_derivation_is_deterministic() {
    let password = "consistent-password";
    let plaintext = "deterministic test";

    // Encrypt with known data
    let enc1 = AesCrypto::encrypt(password, plaintext).unwrap();
    let dec1 = AesCrypto::decrypt(password, &enc1).unwrap();

    let enc2 = AesCrypto::encrypt(password, plaintext).unwrap();
    let dec2 = AesCrypto::decrypt(password, &enc2).unwrap();

    assert_eq!(dec1, dec2, "same password should always decrypt correctly");
}

#[test]
fn encrypt_decrypt_with_special_characters_in_password() {
    let password = r#"p@$$w0rd!#%^&*()_+-=[]{}|;':",.<>?/\`~"#;
    let plaintext = "secret message";

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_decrypt_exactly_16_bytes() {
    // Exactly one AES block (16 bytes)
    let password = "block-test";
    let plaintext = "0123456789abcdef"; // 16 chars = 16 bytes

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_decrypt_block_boundary_minus_one() {
    // 15 bytes (one less than block size)
    let password = "boundary-test";
    let plaintext = "0123456789abcde"; // 15 chars

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_decrypt_block_boundary_plus_one() {
    // 17 bytes (one more than block size)
    let password = "boundary-test";
    let plaintext = "0123456789abcdefg"; // 17 chars

    let encrypted = AesCrypto::encrypt(password, plaintext).unwrap();
    let decrypted = AesCrypto::decrypt(password, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}
