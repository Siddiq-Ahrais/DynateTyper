//! AES-256-GCM encryption/decryption untuk config file.
//! Kunci disimpan di OS keychain (Windows Credential Manager).
//! Tidak butuh akses admin/root.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

/// Nama service & user untuk keyring (Windows Credential Manager)
const KEYRING_SERVICE: &str = "com.dynatetyper.config";
const KEYRING_USER: &str = "encryption-key";

/// Magic header untuk mendeteksi file terenkripsi
/// Format file: MAGIC(8 bytes) + NONCE(12 bytes, base64) + ":" + CIPHERTEXT(base64)
const ENCRYPTED_MAGIC: &str = "DNTCRYPT";

/// Generate random AES-256 key (32 bytes) dan simpan ke OS keychain.
pub fn generate_and_store_key() -> Result<(), String> {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);

    let key_b64 = BASE64.encode(key_bytes);

    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Gagal membuat keyring entry: {}", e))?;

    entry
        .set_password(&key_b64)
        .map_err(|e| format!("Gagal menyimpan key ke OS keychain: {}", e))?;

    Ok(())
}

/// Ambil AES-256 key dari OS keychain.
fn get_key_from_keychain() -> Result<[u8; 32], String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Gagal membuat keyring entry: {}", e))?;

    let key_b64 = entry
        .get_password()
        .map_err(|e| format!("Gagal mengambil key dari OS keychain: {}", e))?;

    let key_bytes = BASE64
        .decode(&key_b64)
        .map_err(|e| format!("Key dari keychain tidak valid (base64): {}", e))?;

    if key_bytes.len() != 32 {
        return Err(format!(
            "Key dari keychain ukuran salah: {} bytes (expected 32)",
            key_bytes.len()
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

/// Pastikan key sudah ada di keychain, jika belum generate baru.
pub fn ensure_key_exists() -> Result<(), String> {
    match get_key_from_keychain() {
        Ok(_) => Ok(()),
        Err(_) => generate_and_store_key(),
    }
}

/// Hapus key dari OS keychain (saat user disable encryption).
pub fn delete_key_from_keychain() -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Gagal membuat keyring entry: {}", e))?;

    // Jika entry tidak ada, tidak apa-apa
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Gagal menghapus key dari keychain: {}", e)),
    }
}

/// Encrypt plaintext string → format: "DNTCRYPT<nonce_b64>:<ciphertext_b64>"
pub fn encrypt_string(plaintext: &str) -> Result<String, String> {
    let key_bytes = get_key_from_keychain()?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Gagal membuat cipher: {}", e))?;

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Gagal mengenkripsi: {}", e))?;

    // Format: MAGIC + nonce_b64 + ":" + ciphertext_b64
    let nonce_b64 = BASE64.encode(nonce_bytes);
    let cipher_b64 = BASE64.encode(ciphertext);

    Ok(format!("{}{}:{}", ENCRYPTED_MAGIC, nonce_b64, cipher_b64))
}

/// Decrypt dari format encrypted string → plaintext.
pub fn decrypt_string(encrypted: &str) -> Result<String, String> {
    // Validasi magic header
    if !encrypted.starts_with(ENCRYPTED_MAGIC) {
        return Err("File bukan format terenkripsi DynateTyper".to_string());
    }

    let data = &encrypted[ENCRYPTED_MAGIC.len()..];
    let parts: Vec<&str> = data.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Format file terenkripsi tidak valid".to_string());
    }

    let nonce_bytes = BASE64
        .decode(parts[0])
        .map_err(|e| format!("Nonce tidak valid (base64): {}", e))?;

    if nonce_bytes.len() != 12 {
        return Err(format!(
            "Nonce ukuran salah: {} bytes (expected 12)",
            nonce_bytes.len()
        ));
    }

    let ciphertext = BASE64
        .decode(parts[1])
        .map_err(|e| format!("Ciphertext tidak valid (base64): {}", e))?;

    let key_bytes = get_key_from_keychain()?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Gagal membuat cipher: {}", e))?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| {
            "Gagal mendekripsi config. Key di keychain mungkin sudah berubah.".to_string()
        })?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("Hasil dekripsi bukan UTF-8 valid: {}", e))
}

/// Cek apakah content adalah file terenkripsi.
pub fn is_encrypted(content: &str) -> bool {
    content.starts_with(ENCRYPTED_MAGIC)
}
