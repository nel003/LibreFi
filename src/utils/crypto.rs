use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};

/// Encrypts a string (like JSON) and returns a byte vector containing [Nonce + Ciphertext]
pub fn encrypt_payload(json_payload: &str, secret_key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let key = Key::<Aes256Gcm>::from_slice(secret_key);
    let cipher = Aes256Gcm::new(key);

    // Generate a random 12-byte nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt the payload
    let ciphertext = cipher
        .encrypt(&nonce, json_payload.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Combine Nonce (12 bytes) + Ciphertext into a single vector
    let mut encrypted_data = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);

    Ok(encrypted_data)
}

/// Decrypts a byte vector containing [Nonce + Ciphertext] back into a String
pub fn decrypt_payload(encrypted_data: &[u8], secret_key: &[u8; 32]) -> Result<String, String> {
    if encrypted_data.len() < 12 {
        return Err("Data too short to contain nonce".to_string());
    }

    let key = Key::<Aes256Gcm>::from_slice(secret_key);
    let cipher = Aes256Gcm::new(key);

    // Split the data back into Nonce and Ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt the payload
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    // Convert bytes back to a UTF-8 String
    String::from_utf8(plaintext_bytes)
        .map_err(|e| format!("Invalid UTF-8 in decrypted data: {}", e))
}
