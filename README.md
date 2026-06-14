# ⌨ DynateTyper

**ver 0.1 beta**

> Lightweight key automation manager built with Tauri 2 — capture physical keys, define repeat intervals, and run them on demand.

---

## ✨ Features

- **Key Capture** — Press any physical key (or modifier combo up to 3 modifiers + 1 key) and register it instantly via a visual on-screen keyboard.
- **Auto-Type Engine** — Run all registered key entries simultaneously with configurable per-key interval and optional duration limit.
- **Profile System** — Create, switch, and delete named profiles to organize different key sets.
- **Conflict Detection** — Warns you when a captured key/combo clashes with an existing entry.
- **AES-256-GCM Encryption** — Config files are encrypted at rest; encryption key is stored in the OS keychain (Windows Credential Manager).
- **Global Shortcuts** — `F6` to Run, `F7` to Stop — works even when the window is not focused.
- **TOML Config** — Human-readable configuration, persisted to the standard app-data directory.

---

## 🛠 Tech Stack

| Layer     | Technology                          |
|-----------|-------------------------------------|
| Runtime   | [Tauri 2](https://v2.tauri.app/)    |
| Backend   | Rust (enigo, rdev, aes-gcm, keyring)|
| Frontend  | Vanilla HTML + TypeScript + CSS     |
| Bundler   | Vite                                |

---

## 📦 Prerequisites

- [Node.js](https://nodejs.org/) ≥ 18
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS
  - Windows: WebView2, Visual Studio Build Tools

---

## 🚀 Getting Started

```bash
# Clone the repository
git clone https://github.com/Siddiq-Ahrais/DynateTyper.git
cd DynateTyper

# Install frontend dependencies
npm install

# Run in development mode (opens Tauri window with hot-reload)
npm run tauri dev

# Build production installer
npm run tauri build
```

---

## ⌨ Keyboard Shortcuts

| Shortcut | Action                        |
|----------|-------------------------------|
| `F6`     | Run all registered key entries|
| `F7`     | Stop running                  |

---

## 📂 Project Structure

```
DynateTyper/
├── index.html              # Main UI (capture, entries table, modals)
├── src/
│   ├── main.ts             # Frontend logic & Tauri IPC calls
│   └── styles.css          # UI styling
├── src-tauri/
│   ├── tauri.conf.json     # Tauri app configuration
│   ├── Cargo.toml          # Rust dependencies
│   └── src/
│       ├── lib.rs          # App entry point & command registration
│       ├── commands.rs     # Tauri command handlers (IPC bridge)
│       ├── config.rs       # TOML config load/save
│       ├── conflict_detector.rs  # Key conflict detection
│       ├── crypto.rs       # AES-256-GCM encryption + keyring
│       ├── key_manager.rs  # Key capture & auto-type engine
│       └── models.rs       # Data models (KeyEntry, Profile, etc.)
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## 📝 License

This project is currently unlicensed. All rights reserved.
