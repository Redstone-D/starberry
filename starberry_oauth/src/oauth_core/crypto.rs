//! Cryptographic utilities for OAuth (PKCE, HMAC, AES-GCM, RSA) using `ring`.

use ring::{digest, hmac, aead, signature, error::Unspecified};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

/// Generate a PKCE code challenge from the given verifier using SHA-256 and base64url (no padding).
pub fn pkce_code_challenge(verifier: &str) -> String {
    // SHA-256 hash of the verifier
    let hash = digest::digest(&digest::SHA256, verifier.as_bytes());
    // Base64URL encode without padding
    URL_SAFE_NO_PAD.encode(hash.as_ref())
}

/// Create an HMAC tag for the given data using the provided secret key.
pub fn hmac_sign(key: &[u8], data: &[u8]) -> Vec<u8> {
    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key);
    let tag = hmac::sign(&s_key, data);
    tag.as_ref().to_vec()
}

/// Verify an HMAC tag for the given data and key.
pub fn hmac_verify(key: &[u8], data: &[u8], tag: &[u8]) -> bool {
    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::verify(&s_key, data, tag).is_ok()
}

/// Encrypt plaintext using AES-256-GCM. Returns ciphertext with nonce prefix.
pub fn aes_gcm_encrypt(key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, Unspecified> {
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)?;
    let sealing_key = aead::LessSafeKey::new(unbound_key);
    let mut in_out = plaintext.to_vec();
    sealing_key.seal_in_place_append_tag(
        aead::Nonce::try_assume_unique_for_key(nonce)?,
        aead::Aad::from(aad),
        &mut in_out,
    )?;
    Ok([nonce, &in_out].concat())
}

/// Decrypt ciphertext produced by `aes_gcm_encrypt`, expects nonce prefix.
pub fn aes_gcm_decrypt(key: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, Unspecified> {
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)?;
    let opening_key = aead::LessSafeKey::new(unbound_key);
    let nonce_len = aead::AES_256_GCM.nonce_len();
    let (nonce_bytes, encrypted) = ciphertext.split_at(nonce_len);
    let mut in_out = encrypted.to_vec();
    let plaintext = opening_key.open_in_place(
        aead::Nonce::try_assume_unique_for_key(nonce_bytes)?,
        aead::Aad::from(aad),
        &mut in_out,
    )?;
    Ok(plaintext.to_vec())
}

/// Sign data with an RSA private key (PKCS#1 v1.5 with SHA-256).
pub fn rsa_sign(private_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Parse DER-encoded private key
    let key_pair = signature::RsaKeyPair::from_der(private_key_der)?;
    let mut buf = vec![0; key_pair.public_modulus_len()];
    let rng = ring::rand::SystemRandom::new();
    key_pair.sign(&signature::RSA_PKCS1_SHA256, &rng, data, &mut buf)?;
    Ok(buf)
}

/// Verify data signed with an RSA public key (PKCS#1 v1.5 with SHA-256).
pub fn rsa_verify(public_key_der: &[u8], data: &[u8], sig: &[u8]) -> bool {
    let public_key = signature::UnparsedPublicKey::new(&signature::RSA_PKCS1_2048_8192_SHA256, public_key_der);
    public_key.verify(data, sig).is_ok()
} 