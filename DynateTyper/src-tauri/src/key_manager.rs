use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// Flag global untuk menghentikan capture
static CAPTURING: AtomicBool = AtomicBool::new(false);

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

/// Mulai key capture di background thread.
/// Emit event `key-captured` ke frontend setiap kali key ditekan.
/// Payload: JSON array of key names (modifiers + key as chord).
pub fn start_capture(app_handle: AppHandle) {
    CAPTURING.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        let pressed_modifiers: Arc<parking_lot::Mutex<Vec<String>>> =
            Arc::new(parking_lot::Mutex::new(Vec::new()));

        let mods = pressed_modifiers.clone();
        let app = app_handle.clone();

        let callback = move |event: Event| {
            if !CAPTURING.load(Ordering::SeqCst) {
                return;
            }

            match event.event_type {
                EventType::KeyPress(key) => {
                    let key_name = key_to_string(key);

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
                    if is_modifier(&key_name) {
                        let mut m = mods.lock();
                        m.retain(|k| k != &key_name);
                    }
                }
                _ => {}
            }
        };

        // rdev::listen blocks the thread
        if let Err(e) = listen(callback) {
            eprintln!("Error in rdev listener: {:?}", e);
        }
    });
}

/// Stop key capture
pub fn stop_capture() {
    CAPTURING.store(false, Ordering::SeqCst);
}

/// Cek apakah sedang capturing
pub fn is_capturing() -> bool {
    CAPTURING.load(Ordering::SeqCst)
}
