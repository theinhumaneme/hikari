use std::fs;

use openssl::{
    pkey::{Private, Public},
    rsa::{Padding, Rsa},
    symm::{decrypt, encrypt, Cipher},
};
use rand::{rngs::OsRng, RngCore};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("OpenSSL error: {0}")]
    OpenSSL(#[from] openssl::error::ErrorStack),
    #[error("Decryption error: {0}")]
    Decryption(String),
}

fn read_public_key(key_path: &str) -> Result<Rsa<Public>, CryptoError> {
    let key_data = fs::read(key_path)?;
    let rsa = Rsa::public_key_from_pem(&key_data)?;
    Ok(rsa)
}

fn read_private_key(key_path: &str) -> Result<Rsa<Private>, CryptoError> {
    let key_data = fs::read(key_path)?;
    let rsa = Rsa::private_key_from_pem(&key_data)?;
    Ok(rsa)
}

pub fn encrypt_json(
    input_path: String,
    output_path: String,
    public_key_path: String,
) -> Result<(), CryptoError> {
    // Read the JSON file
    let json_content = fs::read_to_string(input_path)?;
    let json_value: Value = serde_json::from_str(&json_content)?;
    let json_bytes = serde_json::to_vec(&json_value)?;

    // Generate a random AES key
    let mut aes_key = [0u8; 32];
    OsRng.fill_bytes(&mut aes_key);

    // Generate a random IV
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);

    // Encrypt the JSON data with AES
    let encrypted_data = encrypt(Cipher::aes_256_cbc(), &aes_key, Some(&iv), &json_bytes)?;

    // Read the public key
    let rsa = read_public_key(public_key_path.as_str())?;

    // Encrypt the AES key with RSA
    let mut encrypted_key = vec![0; rsa.size() as usize];
    let encrypted_key_len =
        rsa.public_encrypt(&aes_key, &mut encrypted_key, Padding::PKCS1_OAEP)?;
    encrypted_key.truncate(encrypted_key_len);

    // Combine everything into the final format
    let mut final_data = Vec::new();

    final_data.extend_from_slice(&(encrypted_key_len as u32).to_be_bytes());
    final_data.extend_from_slice(&encrypted_key);
    final_data.extend_from_slice(&iv);
    final_data.extend_from_slice(&encrypted_data);

    // Write to output file
    fs::write(output_path, final_data)?;

    Ok(())
}

pub fn decrypt_json(
    input_path: String,
    output_path: String,
    private_key_path: String,
) -> Result<(), CryptoError> {
    // Read the encrypted file
    let encrypted_data = fs::read(input_path)?;

    if encrypted_data.len() < 4 {
        return Err(CryptoError::Decryption("Invalid file format".to_string()));
    }

    // Read the key length
    let key_len = u32::from_be_bytes(encrypted_data[..4].try_into().unwrap()) as usize;

    if encrypted_data.len() < 4 + key_len + 16 {
        return Err(CryptoError::Decryption("Invalid file format".to_string()));
    }

    // Extract the encrypted key, IV, and data
    let encrypted_key = &encrypted_data[4..4 + key_len];
    let iv = &encrypted_data[4 + key_len..4 + key_len + 16];
    let encrypted_content = &encrypted_data[4 + key_len + 16..];

    // Read the private key
    let rsa = read_private_key(private_key_path.as_str())?;

    // Decrypt the AES key
    let mut aes_key = vec![0; rsa.size() as usize];
    let aes_key_len = rsa.private_decrypt(encrypted_key, &mut aes_key, Padding::PKCS1_OAEP)?;
    aes_key.truncate(aes_key_len);

    // Decrypt the content
    let decrypted_data = decrypt(Cipher::aes_256_cbc(), &aes_key, Some(iv), encrypted_content)?;

    // Parse and validate JSON
    let json_value: Value = serde_json::from_slice(&decrypted_data)?;

    // Write decrypted JSON to file
    fs::write(output_path, serde_json::to_string_pretty(&json_value)?)?;

    Ok(())
}
