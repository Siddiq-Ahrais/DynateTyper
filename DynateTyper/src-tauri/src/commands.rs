use parking_lot::Mutex;
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::config;
use crate::conflict_detector;
use crate::key_manager;
use crate::models::{AppConfig, KeyEntry, Profile};

/// Shared state: AppConfig wrapped in Mutex
pub struct AppState {
    pub config: Mutex<AppConfig>,
}

// ─── Key Capture ────────────────────────────────────────

#[tauri::command]
pub fn start_key_capture(app_handle: AppHandle) -> Result<String, String> {
    if key_manager::is_capturing() {
        return Err("Capture sudah berjalan".to_string());
    }
    key_manager::start_capture(app_handle);
    Ok("Capture dimulai".to_string())
}

#[tauri::command]
pub fn stop_key_capture() -> Result<String, String> {
    key_manager::stop_capture();
    Ok("Capture dihentikan".to_string())
}

// ─── Key Entry CRUD ─────────────────────────────────────

#[tauri::command]
pub fn add_key_entry(
    state: State<'_, AppState>,
    keys: Vec<String>,
    interval_ms: u64,
    duration_ms: u64,
) -> Result<KeyEntry, String> {
    let mut config = state.config.lock();

    let profile = config
        .active_profile_mut()
        .ok_or("Profile aktif tidak ditemukan")?;

    // Enforce max 20 entries per profile
    if profile.entries.len() >= 20 {
        return Err("Maksimum 20 key/combo per profile telah tercapai".to_string());
    }

    // Validasi: max 3 modifier + 1 key
    let modifier_count = keys
        .iter()
        .filter(|k| matches!(k.as_str(), "Ctrl" | "Shift" | "Alt" | "AltGr" | "Win"))
        .count();
    let non_modifier_count = keys.len() - modifier_count;

    if modifier_count > 3 {
        return Err("Maksimum 3 modifier key".to_string());
    }
    if non_modifier_count > 1 {
        return Err("Hanya 1 key biasa yang diizinkan per combo".to_string());
    }
    if non_modifier_count == 0 && modifier_count == 0 {
        return Err("Minimal 1 key harus dipilih".to_string());
    }

    let entry = KeyEntry {
        id: Uuid::new_v4().to_string(),
        keys,
        interval_ms,
        duration_ms,
    };

    profile.entries.push(entry.clone());
    config::save_config(&config)?;

    Ok(entry)
}

#[tauri::command]
pub fn edit_key_entry(
    state: State<'_, AppState>,
    id: String,
    keys: Vec<String>,
    interval_ms: u64,
    duration_ms: u64,
) -> Result<KeyEntry, String> {
    let mut config = state.config.lock();

    let profile = config
        .active_profile_mut()
        .ok_or("Profile aktif tidak ditemukan")?;

    let entry = profile
        .entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or("Key entry tidak ditemukan")?;

    // Validasi modifier count
    let modifier_count = keys
        .iter()
        .filter(|k| matches!(k.as_str(), "Ctrl" | "Shift" | "Alt" | "AltGr" | "Win"))
        .count();
    let non_modifier_count = keys.len() - modifier_count;

    if modifier_count > 3 {
        return Err("Maksimum 3 modifier key".to_string());
    }
    if non_modifier_count > 1 {
        return Err("Hanya 1 key biasa yang diizinkan per combo".to_string());
    }

    entry.keys = keys;
    entry.interval_ms = interval_ms;
    entry.duration_ms = duration_ms;

    let updated = entry.clone();
    config::save_config(&config)?;

    Ok(updated)
}

#[tauri::command]
pub fn delete_key_entry(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let mut config = state.config.lock();

    let profile = config
        .active_profile_mut()
        .ok_or("Profile aktif tidak ditemukan")?;

    let len_before = profile.entries.len();
    profile.entries.retain(|e| e.id != id);

    if profile.entries.len() == len_before {
        return Err("Key entry tidak ditemukan".to_string());
    }

    config::save_config(&config)?;
    Ok("Key entry dihapus".to_string())
}

#[tauri::command]
pub fn clear_all_entries(state: State<'_, AppState>) -> Result<String, String> {
    let mut config = state.config.lock();

    let profile = config
        .active_profile_mut()
        .ok_or("Profile aktif tidak ditemukan")?;

    profile.entries.clear();
    config::save_config(&config)?;

    Ok("Semua key entries dihapus".to_string())
}

#[tauri::command]
pub fn get_entries(state: State<'_, AppState>) -> Result<Vec<KeyEntry>, String> {
    let config = state.config.lock();

    let profile = config
        .active_profile()
        .ok_or("Profile aktif tidak ditemukan")?;

    Ok(profile.entries.clone())
}

// ─── Conflict Detection ─────────────────────────────────

#[tauri::command]
pub fn check_key_conflict(keys: Vec<String>) -> Result<Option<String>, String> {
    Ok(conflict_detector::check_conflict(&keys))
}

// ─── Profile Management ─────────────────────────────────

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state.config.lock();
    Ok(config.profiles.iter().map(|p| p.name.clone()).collect())
}

#[tauri::command]
pub fn create_profile(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let mut config = state.config.lock();

    // Cek duplikat
    if config.profiles.iter().any(|p| p.name == name) {
        return Err(format!("Profile '{}' sudah ada", name));
    }

    config.profiles.push(Profile {
        name: name.clone(),
        entries: Vec::new(),
    });

    config::save_config(&config)?;
    Ok(format!("Profile '{}' dibuat", name))
}

#[tauri::command]
pub fn switch_profile(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let mut config = state.config.lock();

    // Pastikan profile ada
    if !config.profiles.iter().any(|p| p.name == name) {
        return Err(format!("Profile '{}' tidak ditemukan", name));
    }

    config.active_profile = name.clone();
    config::save_config(&config)?;

    Ok(format!("Switched ke profile '{}'", name))
}

#[tauri::command]
pub fn delete_profile(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let mut config = state.config.lock();

    // Tidak boleh hapus profile terakhir
    if config.profiles.len() <= 1 {
        return Err("Tidak bisa menghapus profile terakhir".to_string());
    }

    // Jika menghapus profile aktif, auto-switch ke profile lain
    if config.active_profile == name {
        let new_active = config
            .profiles
            .iter()
            .find(|p| p.name != name)
            .map(|p| p.name.clone())
            .ok_or("Tidak ada profile lain untuk di-switch")?;
        config.active_profile = new_active;
    }

    config.profiles.retain(|p| p.name != name);
    config::save_config(&config)?;

    Ok(format!("Profile '{}' dihapus", name))
}

#[tauri::command]
pub fn get_active_profile(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock();
    Ok(config.active_profile.clone())
}

// ─── Encryption Toggle ──────────────────────────────────

#[tauri::command]
pub fn toggle_encryption(state: State<'_, AppState>, enable: bool) -> Result<String, String> {
    let mut config = state.config.lock();

    if enable == config.encrypt_config {
        return Ok(if enable {
            "Enkripsi sudah aktif".to_string()
        } else {
            "Enkripsi sudah nonaktif".to_string()
        });
    }

    if enable {
        // Aktifkan: generate key di keychain, set flag, re-save encrypted
        crate::crypto::ensure_key_exists()?;
        config.encrypt_config = true;
        config::save_config(&config)?;
        Ok("Enkripsi config diaktifkan. Key disimpan di OS keychain.".to_string())
    } else {
        // Nonaktifkan: set flag dulu, re-save as plain TOML, hapus key dari keychain
        config.encrypt_config = false;
        config::save_config(&config)?;
        // Best-effort delete key dari keychain
        let _ = crate::crypto::delete_key_from_keychain();
        Ok("Enkripsi config dinonaktifkan. Key dihapus dari keychain.".to_string())
    }
}

#[tauri::command]
pub fn get_encryption_status(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.config.lock();
    Ok(config.encrypt_config)
}
