use serde::{Deserialize, Serialize};

/// Satu key/combo yang didaftarkan pengguna.
/// Mendukung hingga 3 modifier + 1 key (chord, bukan sequential).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    /// UUID unik untuk identifikasi
    pub id: String,
    /// List key names, e.g. ["Ctrl", "Shift", "F5"]
    /// Modifier keys come first, regular key last
    pub keys: Vec<String>,
    /// Interval typing dalam milidetik
    pub interval_ms: u64,
    /// Durasi total dalam milidetik (optional, 0 = tak terbatas)
    pub duration_ms: u64,
}

/// Profile — kumpulan KeyEntry, maksimum 20 entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Nama profile (bebas, user-defined)
    pub name: String,
    /// Daftar key entries, max 20
    pub entries: Vec<KeyEntry>,
}

/// Root config yang disimpan sebagai TOML file.
/// Saat encrypt_config aktif, file dienkripsi AES-256-GCM
/// dengan kunci dari OS keychain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Nama profile yang sedang aktif
    pub active_profile: String,
    /// Apakah config file dienkripsi (optional, toggleable)
    #[serde(default)]
    pub encrypt_config: bool,
    /// Semua profiles
    pub profiles: Vec<Profile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_profile: "Default".to_string(),
            encrypt_config: false,
            profiles: vec![Profile {
                name: "Default".to_string(),
                entries: Vec::new(),
            }],
        }
    }
}

impl AppConfig {
    /// Cari profile berdasarkan nama (mutable reference)
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|p| p.name == name)
    }

    /// Cari profile berdasarkan nama (immutable reference)
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Get active profile (mutable)
    pub fn active_profile_mut(&mut self) -> Option<&mut Profile> {
        let name = self.active_profile.clone();
        self.get_profile_mut(&name)
    }

    /// Get active profile (immutable)
    pub fn active_profile(&self) -> Option<&Profile> {
        self.get_profile(&self.active_profile)
    }
}
