/// Daftar hotkey sistem kritis yang berpotensi konflik.
/// Jika user mendaftarkan key combo yang match, tampilkan peringatan.

/// Daftar system critical hotkeys
const CRITICAL_HOTKEYS: &[(&[&str], &str)] = &[
    (&["Alt", "F4"], "Menutup aplikasi aktif"),
    (&["Ctrl", "Alt", "Delete"], "Windows Security Screen"),
    (&["Alt", "Tab"], "Switch antar window"),
    (&["Ctrl", "Esc"], "Membuka Start Menu"),
    (&["Win", "L"], "Lock screen"),
    (&["Win", "D"], "Show desktop"),
    (&["Win", "E"], "Membuka File Explorer"),
    (&["Win", "R"], "Membuka Run dialog"),
    (&["Ctrl", "Shift", "Esc"], "Membuka Task Manager"),
    (&["Alt", "F8"], "Windows password entry (login screen)"),
    (&["Ctrl", "Alt", "Tab"], "Switch window (persistent)"),
    (&["Win", "Tab"], "Task View"),
    (&["PrintScreen"], "Screenshot"),
    (&["Alt", "PrintScreen"], "Screenshot window aktif"),
    (&["Ctrl", "C"], "Copy"),
    (&["Ctrl", "V"], "Paste"),
    (&["Ctrl", "X"], "Cut"),
    (&["Ctrl", "Z"], "Undo"),
    (&["Ctrl", "A"], "Select All"),
    (&["Ctrl", "S"], "Save"),
];

/// Cek apakah key combo konflik dengan hotkey sistem.
/// Mengembalikan pesan peringatan jika konflik, None jika aman.
pub fn check_conflict(keys: &[String]) -> Option<String> {
    // Normalisasi: sort keys untuk perbandingan
    let mut normalized: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
    normalized.sort();

    for (hotkey, description) in CRITICAL_HOTKEYS {
        let mut hotkey_sorted: Vec<String> = hotkey.iter().map(|k| k.to_string()).collect();
        hotkey_sorted.sort();

        if normalized == hotkey_sorted {
            let combo_str = keys.join("+");
            return Some(format!(
                "⚠️ Peringatan: '{}' adalah hotkey sistem kritis ({}). Mendaftarkan combo ini bisa mengganggu fungsi sistem.",
                combo_str, description
            ));
        }
    }

    None
}
