use std::fs;
use std::path::PathBuf;

use crate::crypto;
use crate::models::AppConfig;

/// Dapatkan path config file: {config_dir}/dynatetyper/config.toml
pub fn config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dynatetyper");
    config_dir.join("config.toml")
}

/// Load config dari file. Otomatis deteksi apakah terenkripsi atau plain TOML.
pub fn load_config() -> AppConfig {
    let path = config_path();

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                // Deteksi: apakah file terenkripsi?
                let toml_content = if crypto::is_encrypted(&content) {
                    // File terenkripsi → decrypt dulu
                    match crypto::decrypt_string(&content) {
                        Ok(decrypted) => decrypted,
                        Err(e) => {
                            eprintln!("Error decrypting config: {}. Using default.", e);
                            return create_and_save_default();
                        }
                    }
                } else {
                    // File plain TOML
                    content
                };

                // Parse TOML
                match toml::from_str::<AppConfig>(&toml_content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Error parsing config TOML: {}. Using default.", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading config file: {}. Using default.", e);
            }
        }
    }

    create_and_save_default()
}

/// Buat default config dan simpan.
fn create_and_save_default() -> AppConfig {
    let default_config = AppConfig::default();
    let _ = save_config(&default_config);
    default_config
}

/// Simpan config ke file. Jika encrypt_config aktif, enkripsi dengan AES-256-GCM.
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();

    // Pastikan parent directory ada
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    // Serialize ke TOML
    let toml_string =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Enkripsi jika diaktifkan
    let file_content = if config.encrypt_config {
        // Pastikan key ada di keychain
        crypto::ensure_key_exists()?;
        // Encrypt TOML string
        crypto::encrypt_string(&toml_string)?
    } else {
        // Plain TOML
        toml_string
    };

    fs::write(&path, file_content).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
