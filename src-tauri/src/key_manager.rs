use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::models::KeyEntry;

/// Flag global untuk mengaktifkan/menonaktifkan capture
static CAPTURING: AtomicBool = AtomicBool::new(false);

/// Flag untuk memastikan listener thread hanya di-spawn sekali
static LISTENER_STARTED: AtomicBool = AtomicBool::new(false);

/// Flag global untuk running state
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Konversi rdev::Key ke nama string yang human-readable
fn key_to_string(key: Key) -> String {
    match key {
        // Modifier keys
        Key::ControlLeft | Key::ControlRight => "Ctrl".to_string(),
        Key::ShiftLeft | Key::ShiftRight => "Shift".to_string(),
        Key::Alt => "Alt".to_string(),
        Key::AltGr => "AltGr".to_string(),
        Key::MetaLeft | Key::MetaRight => "Win".to_string(),

        // Function keys
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),

        // Navigation & editing
        Key::Escape => "Esc".to_string(),
        Key::Return => "Enter".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Space => "Space".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::UpArrow => "Up".to_string(),
        Key::DownArrow => "Down".to_string(),
        Key::LeftArrow => "Left".to_string(),
        Key::RightArrow => "Right".to_string(),
        Key::Insert => "Insert".to_string(),

        // Toggle keys
        Key::CapsLock => "CapsLock".to_string(),
        Key::PrintScreen => "PrintScreen".to_string(),
        Key::ScrollLock => "ScrollLock".to_string(),
        Key::Pause => "Pause".to_string(),
        Key::NumLock => "NumLock".to_string(),

        // Number row (top row 0-9)
        Key::Num0 => "0".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),

        // Symbol keys
        Key::BackQuote => "`".to_string(),
        Key::Minus => "-".to_string(),
        Key::Equal => "=".to_string(),
        Key::LeftBracket => "[".to_string(),
        Key::RightBracket => "]".to_string(),
        Key::SemiColon => ";".to_string(),
        Key::Quote => "'".to_string(),
        Key::BackSlash => "\\".to_string(),
        Key::IntlBackslash => "IntlBackslash".to_string(),
        Key::Comma => ",".to_string(),
        Key::Dot => ".".to_string(),
        Key::Slash => "/".to_string(),

        // Alfanumerik (A-Z)
        Key::KeyA => "A".to_string(),
        Key::KeyB => "B".to_string(),
        Key::KeyC => "C".to_string(),
        Key::KeyD => "D".to_string(),
        Key::KeyE => "E".to_string(),
        Key::KeyF => "F".to_string(),
        Key::KeyG => "G".to_string(),
        Key::KeyH => "H".to_string(),
        Key::KeyI => "I".to_string(),
        Key::KeyJ => "J".to_string(),
        Key::KeyK => "K".to_string(),
        Key::KeyL => "L".to_string(),
        Key::KeyM => "M".to_string(),
        Key::KeyN => "N".to_string(),
        Key::KeyO => "O".to_string(),
        Key::KeyP => "P".to_string(),
        Key::KeyQ => "Q".to_string(),
        Key::KeyR => "R".to_string(),
        Key::KeyS => "S".to_string(),
        Key::KeyT => "T".to_string(),
        Key::KeyU => "U".to_string(),
        Key::KeyV => "V".to_string(),
        Key::KeyW => "W".to_string(),
        Key::KeyX => "X".to_string(),
        Key::KeyY => "Y".to_string(),
        Key::KeyZ => "Z".to_string(),

        // Numpad (keypad)
        Key::Kp0 => "Num0".to_string(),
        Key::Kp1 => "Num1".to_string(),
        Key::Kp2 => "Num2".to_string(),
        Key::Kp3 => "Num3".to_string(),
        Key::Kp4 => "Num4".to_string(),
        Key::Kp5 => "Num5".to_string(),
        Key::Kp6 => "Num6".to_string(),
        Key::Kp7 => "Num7".to_string(),
        Key::Kp8 => "Num8".to_string(),
        Key::Kp9 => "Num9".to_string(),
        Key::KpReturn => "NumEnter".to_string(),
        Key::KpMinus => "Num-".to_string(),
        Key::KpPlus => "Num+".to_string(),
        Key::KpMultiply => "Num*".to_string(),
        Key::KpDivide => "Num/".to_string(),
        Key::KpDelete => "NumDel".to_string(),

        // Misc
        Key::Function => "Fn".to_string(),

        // Unknown — fallback
        Key::Unknown(code) => format!("Key({})", code),
    }
}

/// Cek apakah key adalah modifier
fn is_modifier(key_name: &str) -> bool {
    matches!(key_name, "Ctrl" | "Shift" | "Alt" | "AltGr" | "Win")
}

/// Inisialisasi global keyboard listener.
/// Harus dipanggil saat app startup agar global shortcuts (F6/F7) langsung aktif.
/// Listener thread di-spawn sekali saja dan tetap hidup selama aplikasi berjalan.
pub fn init_listener(app_handle: AppHandle) {
    // Hanya spawn listener thread sekali
    if LISTENER_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        std::thread::spawn(move || {
            let pressed_modifiers: Arc<parking_lot::Mutex<Vec<String>>> =
                Arc::new(parking_lot::Mutex::new(Vec::new()));

            let mods = pressed_modifiers.clone();
            let app = app_handle.clone();

            let callback = move |event: Event| {
                if !CAPTURING.load(Ordering::SeqCst) {
                    // Saat tidak capturing, clear modifiers agar tidak stale
                    let mut m = mods.lock();
                    m.clear();

                    // ─── Global Shortcuts: F6 = Run, F7 = Stop ───
                    if let EventType::KeyPress(key) = event.event_type {
                        match key {
                            Key::F6 => {
                                let _ = app.emit("shortcut-run", "f6");
                            }
                            Key::F7 => {
                                let _ = app.emit("shortcut-stop", "f7");
                            }
                            _ => {}
                        }
                    }
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let key_name = key_to_string(key);

                        // Emit individual key press untuk visual keyboard
                        let _ = app.emit("key-pressed", key_name.clone());

                        if is_modifier(&key_name) {
                            let mut m = mods.lock();
                            if !m.contains(&key_name) {
                                m.push(key_name);
                            }
                        } else {
                            // Chord: modifiers + key
                            let mut combo: Vec<String> = {
                                let m = mods.lock();
                                m.clone()
                            };
                            combo.push(key_name);

                            // Emit ke frontend
                            let _ = app.emit("key-captured", combo);
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let key_name = key_to_string(key);

                        // Emit individual key release untuk visual keyboard
                        let _ = app.emit("key-released", key_name.clone());

                        if is_modifier(&key_name) {
                            let mut m = mods.lock();
                            m.retain(|k| k != &key_name);
                        }
                    }
                    _ => {}
                }
            };

            // rdev::listen blocks the thread forever — ini intentional
            if let Err(e) = listen(callback) {
                eprintln!("Error in rdev listener: {:?}", e);
                // Reset flag agar bisa dicoba lagi jika listener gagal
                LISTENER_STARTED.store(false, Ordering::SeqCst);
            }
        });
    }
}

/// Mulai key capture.
/// Start/stop hanya toggle flag CAPTURING untuk mengaktifkan/menonaktifkan event emission.
pub fn start_capture(app_handle: AppHandle) {
    CAPTURING.store(true, Ordering::SeqCst);
    // Pastikan listener sudah jalan (fallback jika init_listener belum dipanggil)
    init_listener(app_handle);
}

/// Stop key capture (hanya toggle flag, listener tetap hidup)
pub fn stop_capture() {
    CAPTURING.store(false, Ordering::SeqCst);
}

/// Cek apakah sedang capturing
pub fn is_capturing() -> bool {
    CAPTURING.load(Ordering::SeqCst)
}

// ═════════════════════════════════════════════════════════
// Key Running (Auto-Type)
// ═════════════════════════════════════════════════════════

/// Konversi nama key (string) ke rdev::Key untuk simulate
fn string_to_rdev_key(key_name: &str) -> Option<rdev::Key> {
    match key_name {
        // Modifiers
        "Ctrl" => Some(rdev::Key::ControlLeft),
        "Shift" => Some(rdev::Key::ShiftLeft),
        "Alt" => Some(rdev::Key::Alt),
        "AltGr" => Some(rdev::Key::AltGr),
        "Win" => Some(rdev::Key::MetaLeft),

        // Function keys
        "F1" => Some(rdev::Key::F1),
        "F2" => Some(rdev::Key::F2),
        "F3" => Some(rdev::Key::F3),
        "F4" => Some(rdev::Key::F4),
        "F5" => Some(rdev::Key::F5),
        "F6" => Some(rdev::Key::F6),
        "F7" => Some(rdev::Key::F7),
        "F8" => Some(rdev::Key::F8),
        "F9" => Some(rdev::Key::F9),
        "F10" => Some(rdev::Key::F10),
        "F11" => Some(rdev::Key::F11),
        "F12" => Some(rdev::Key::F12),

        // Navigation & editing
        "Esc" => Some(rdev::Key::Escape),
        "Enter" => Some(rdev::Key::Return),
        "Tab" => Some(rdev::Key::Tab),
        "Space" => Some(rdev::Key::Space),
        "Backspace" => Some(rdev::Key::Backspace),
        "Delete" => Some(rdev::Key::Delete),
        "Home" => Some(rdev::Key::Home),
        "End" => Some(rdev::Key::End),
        "PageUp" => Some(rdev::Key::PageUp),
        "PageDown" => Some(rdev::Key::PageDown),
        "Up" => Some(rdev::Key::UpArrow),
        "Down" => Some(rdev::Key::DownArrow),
        "Left" => Some(rdev::Key::LeftArrow),
        "Right" => Some(rdev::Key::RightArrow),
        "Insert" => Some(rdev::Key::Insert),

        // Toggle keys
        "CapsLock" => Some(rdev::Key::CapsLock),
        "PrintScreen" => Some(rdev::Key::PrintScreen),
        "ScrollLock" => Some(rdev::Key::ScrollLock),
        "Pause" => Some(rdev::Key::Pause),
        "NumLock" => Some(rdev::Key::NumLock),

        // Number row
        "0" => Some(rdev::Key::Num0),
        "1" => Some(rdev::Key::Num1),
        "2" => Some(rdev::Key::Num2),
        "3" => Some(rdev::Key::Num3),
        "4" => Some(rdev::Key::Num4),
        "5" => Some(rdev::Key::Num5),
        "6" => Some(rdev::Key::Num6),
        "7" => Some(rdev::Key::Num7),
        "8" => Some(rdev::Key::Num8),
        "9" => Some(rdev::Key::Num9),

        // Symbol keys
        "`" => Some(rdev::Key::BackQuote),
        "-" => Some(rdev::Key::Minus),
        "=" => Some(rdev::Key::Equal),
        "[" => Some(rdev::Key::LeftBracket),
        "]" => Some(rdev::Key::RightBracket),
        ";" => Some(rdev::Key::SemiColon),
        "'" => Some(rdev::Key::Quote),
        "\\" => Some(rdev::Key::BackSlash),
        "," => Some(rdev::Key::Comma),
        "." => Some(rdev::Key::Dot),
        "/" => Some(rdev::Key::Slash),

        // Alfanumerik (A-Z)
        "A" => Some(rdev::Key::KeyA),
        "B" => Some(rdev::Key::KeyB),
        "C" => Some(rdev::Key::KeyC),
        "D" => Some(rdev::Key::KeyD),
        "E" => Some(rdev::Key::KeyE),
        "F" => Some(rdev::Key::KeyF),
        "G" => Some(rdev::Key::KeyG),
        "H" => Some(rdev::Key::KeyH),
        "I" => Some(rdev::Key::KeyI),
        "J" => Some(rdev::Key::KeyJ),
        "K" => Some(rdev::Key::KeyK),
        "L" => Some(rdev::Key::KeyL),
        "M" => Some(rdev::Key::KeyM),
        "N" => Some(rdev::Key::KeyN),
        "O" => Some(rdev::Key::KeyO),
        "P" => Some(rdev::Key::KeyP),
        "Q" => Some(rdev::Key::KeyQ),
        "R" => Some(rdev::Key::KeyR),
        "S" => Some(rdev::Key::KeyS),
        "T" => Some(rdev::Key::KeyT),
        "U" => Some(rdev::Key::KeyU),
        "V" => Some(rdev::Key::KeyV),
        "W" => Some(rdev::Key::KeyW),
        "X" => Some(rdev::Key::KeyX),
        "Y" => Some(rdev::Key::KeyY),
        "Z" => Some(rdev::Key::KeyZ),

        // Numpad
        "Num0" => Some(rdev::Key::Kp0),
        "Num1" => Some(rdev::Key::Kp1),
        "Num2" => Some(rdev::Key::Kp2),
        "Num3" => Some(rdev::Key::Kp3),
        "Num4" => Some(rdev::Key::Kp4),
        "Num5" => Some(rdev::Key::Kp5),
        "Num6" => Some(rdev::Key::Kp6),
        "Num7" => Some(rdev::Key::Kp7),
        "Num8" => Some(rdev::Key::Kp8),
        "Num9" => Some(rdev::Key::Kp9),
        "NumEnter" => Some(rdev::Key::KpReturn),
        "Num-" => Some(rdev::Key::KpMinus),
        "Num+" => Some(rdev::Key::KpPlus),
        "Num*" => Some(rdev::Key::KpMultiply),
        "Num/" => Some(rdev::Key::KpDivide),
        "NumDel" => Some(rdev::Key::KpDelete),

        // Misc
        "Fn" => Some(rdev::Key::Function),

        _ => None,
    }
}

/// Cek apakah key adalah modifier (digunakan oleh simulate)
fn is_modifier_key(key_name: &str) -> bool {
    matches!(key_name, "Ctrl" | "Shift" | "Alt" | "AltGr" | "Win")
}

/// Simulate satu key combo (press modifiers, press key, release key, release modifiers)
fn simulate_combo(keys: &[String]) {
    use rdev::simulate;
    use std::thread::sleep;
    use std::time::Duration;

    let mut modifier_rdev_keys: Vec<rdev::Key> = Vec::new();
    let mut regular_rdev_keys: Vec<rdev::Key> = Vec::new();

    for key_name in keys {
        if let Some(rdev_key) = string_to_rdev_key(key_name) {
            if is_modifier_key(key_name) {
                modifier_rdev_keys.push(rdev_key);
            } else {
                regular_rdev_keys.push(rdev_key);
            }
        }
    }

    // Press modifiers
    for &mod_key in &modifier_rdev_keys {
        let _ = simulate(&EventType::KeyPress(mod_key));
        sleep(Duration::from_millis(10));
    }

    // Press & release regular keys
    for &reg_key in &regular_rdev_keys {
        let _ = simulate(&EventType::KeyPress(reg_key));
        sleep(Duration::from_millis(10));
        let _ = simulate(&EventType::KeyRelease(reg_key));
        sleep(Duration::from_millis(10));
    }

    // Release modifiers (reverse order)
    for &mod_key in modifier_rdev_keys.iter().rev() {
        let _ = simulate(&EventType::KeyRelease(mod_key));
        sleep(Duration::from_millis(10));
    }
}

/// Mulai running semua key entries
/// Setiap entry dijalankan di thread terpisah dengan interval & duration masing-masing
pub fn start_running(entries: Vec<KeyEntry>, app_handle: AppHandle) {
    RUNNING.store(true, Ordering::SeqCst);

    let _ = app_handle.emit("running-status", "started");

    for entry in entries {
        let app = app_handle.clone();
        std::thread::spawn(move || {
            let start_time = std::time::Instant::now();
            let interval = std::time::Duration::from_millis(entry.interval_ms);
            let has_duration = entry.duration_ms > 0;
            let duration = std::time::Duration::from_millis(entry.duration_ms);

            loop {
                // Check if we should stop
                if !RUNNING.load(Ordering::SeqCst) {
                    break;
                }

                // Check duration
                if has_duration && start_time.elapsed() >= duration {
                    break;
                }

                // Simulate the key combo
                simulate_combo(&entry.keys);

                // Wait for interval
                std::thread::sleep(interval);
            }

            let _ = app.emit("entry-finished", entry.id.clone());
        });
    }
}

/// Stop running
pub fn stop_running() {
    RUNNING.store(false, Ordering::SeqCst);
}

/// Cek apakah sedang running
pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}
