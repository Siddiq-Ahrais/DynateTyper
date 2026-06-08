import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ═══════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════
interface KeyEntry {
  id: string;
  keys: string[];
  interval_ms: number;
  duration_ms: number;
}

// ═══════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════
let isCapturing = false;
let capturedKeys: string[] = [];
let editingEntryId: string | null = null;
let editCapturedKeys: string[] | null = null;
let confirmCallback: (() => void) | null = null;

// Modifier keys constant
const MODIFIERS = ["Ctrl", "Shift", "Alt", "AltGr", "Win"];

// Global keyboard blocker — prevents browser from processing key events during capture
// This is critical: without it, physical keyboard presses activate focused buttons
// (e.g. Space/Enter on the capture button re-triggers toggleCapture and stops capture)
function blockKeyboardEvent(e: KeyboardEvent) {
  if (isCapturing || editCaptureMode) {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }
}

// ═══════════════════════════════════════════════════════
// DOM Elements
// ═══════════════════════════════════════════════════════
function $(id: string): HTMLElement {
  return document.getElementById(id)!;
}

// ═══════════════════════════════════════════════════════
// Key Badge Rendering
// ═══════════════════════════════════════════════════════
function renderKeyBadges(keys: string[], container: HTMLElement) {
  container.innerHTML = "";
  keys.forEach((key, i) => {
    const badge = document.createElement("span");
    badge.className = `key-badge ${MODIFIERS.includes(key) ? "modifier" : ""}`;
    badge.textContent = key;
    container.appendChild(badge);

    if (i < keys.length - 1) {
      const sep = document.createElement("span");
      sep.className = "key-badge-separator";
      sep.textContent = "+";
      container.appendChild(sep);
    }
  });
}

// ═══════════════════════════════════════════════════════
// Profile Management
// ═══════════════════════════════════════════════════════
async function loadProfiles() {
  const profiles = await invoke<string[]>("list_profiles");
  const active = await invoke<string>("get_active_profile");
  const select = $("profile-select") as HTMLSelectElement;

  select.innerHTML = "";
  profiles.forEach((name) => {
    const opt = document.createElement("option");
    opt.value = name;
    opt.textContent = name;
    if (name === active) opt.selected = true;
    select.appendChild(opt);
  });
}

async function switchProfile(name: string) {
  await invoke("switch_profile", { name });
  await loadEntries();
}

async function createProfile() {
  showModal("profile-modal");
  ($("new-profile-name") as HTMLInputElement).value = "";
  ($("new-profile-name") as HTMLInputElement).focus();
}

async function saveNewProfile() {
  const name = ($("new-profile-name") as HTMLInputElement).value.trim();
  if (!name) return;

  try {
    await invoke("create_profile", { name });
    await loadProfiles();
    await switchProfile(name);
    hideModal("profile-modal");
  } catch (e) {
    alert(e);
  }
}

async function deleteProfile() {
  const select = $("profile-select") as HTMLSelectElement;
  const name = select.value;

  showConfirm(
    "Hapus Profile",
    `Yakin ingin menghapus profile "${name}"? Semua key entries di dalamnya akan hilang.`,
    async () => {
      try {
        await invoke("delete_profile", { name });
        await loadProfiles();
        await loadEntries();
      } catch (e) {
        alert(e);
      }
    }
  );
}

// ═══════════════════════════════════════════════════════
// Key Capture
// ═══════════════════════════════════════════════════════
async function toggleCapture() {
  if (isCapturing) {
    await stopCapture();
  } else {
    await startCapture();
  }
}

async function startCapture() {
  try {
    await invoke("start_key_capture");
    isCapturing = true;
    capturedKeys = [];
    updateCaptureUI();

    // Remove focus from the capture button so physical key presses
    // (especially Space/Enter) don't re-trigger the button click
    (document.activeElement as HTMLElement)?.blur();
  } catch (e) {
    alert(e);
  }
}

async function stopCapture() {
  try {
    await invoke("stop_key_capture");
    isCapturing = false;
    updateCaptureUI();

    // Show add form jika ada key yang di-capture
    if (capturedKeys.length > 0) {
      $("add-key-form").classList.remove("hidden");

      // Check conflict
      const warning = await invoke<string | null>("check_key_conflict", {
        keys: capturedKeys,
      });
      const warningEl = $("conflict-warning");
      if (warning) {
        warningEl.textContent = warning;
        warningEl.classList.remove("hidden");
      } else {
        warningEl.classList.add("hidden");
      }
    }
  } catch (e) {
    alert(e);
  }
}

function updateCaptureUI() {
  const btn = $("btn-capture");
  const label = $("capture-label");
  const vk = $("visual-keyboard");

  if (isCapturing) {
    btn.classList.add("capturing");
    label.textContent = "Stop Capture";
    vk.classList.remove("hidden");
  } else {
    btn.classList.remove("capturing");
    label.textContent = "Mulai Capture";
    vk.classList.add("hidden");
    // Clear all active keys on visual keyboard
    clearVisualKeyboard();
  }
}

function clearVisualKeyboard() {
  document.querySelectorAll(".vk-key.active").forEach((el) => {
    el.classList.remove("active");
  });
  // Clear all pending flash timeouts
  keyFlashTimeouts.forEach((timeout) => clearTimeout(timeout));
  keyFlashTimeouts.clear();
}

// Track flash timeouts per key to ensure minimum visibility
const keyFlashTimeouts = new Map<string, number>();

function highlightKey(keyName: string, active: boolean) {
  // Find all matching keys (some keys like Shift/Ctrl appear twice)
  const keys = document.querySelectorAll(
    `.vk-key[data-key="${CSS.escape(keyName)}"]`
  );

  if (active) {
    // Clear any pending release timeout for this key
    const existing = keyFlashTimeouts.get(keyName);
    if (existing) {
      clearTimeout(existing);
      keyFlashTimeouts.delete(keyName);
    }
    keys.forEach((el) => el.classList.add("active"));

    // Show key pop-up animation
    showKeyPopup(keyName);
  } else {
    // Delay removal to ensure minimum 150ms visibility
    const timeout = window.setTimeout(() => {
      keys.forEach((el) => el.classList.remove("active"));
      keyFlashTimeouts.delete(keyName);
    }, 150);
    keyFlashTimeouts.set(keyName, timeout);
  }
}

// ═══════════════════════════════════════════════════════
// Key Pop-up Animation
// ═══════════════════════════════════════════════════════
function showKeyPopup(keyName: string) {
  const container = document.getElementById("key-popup-container");
  if (!container) return;

  const popup = document.createElement("div");
  popup.className = `key-popup ${MODIFIERS.includes(keyName) ? "modifier" : ""}`;
  popup.textContent = keyName;

  container.appendChild(popup);

  // Remove after animation completes
  popup.addEventListener("animationend", () => {
    popup.remove();
  });

  // Fallback removal
  setTimeout(() => {
    if (popup.parentElement) popup.remove();
  }, 1200);
}

// Track modifier keys pressed via visual keyboard clicks
const vkPressedModifiers: Set<string> = new Set();

function initVisualKeyboard() {
  // Tag modifier keys with a special class for green glow
  const modifierNames = ["Ctrl", "Shift", "Alt", "AltGr", "Win"];
  document.querySelectorAll(".vk-key").forEach((el) => {
    const keyEl = el as HTMLElement;
    const keyName = keyEl.dataset.key;
    if (keyName && modifierNames.includes(keyName)) {
      el.classList.add("modifier-key");
    }

    // ─── Click handler for visual keyboard keys ───
    if (keyName) {
      keyEl.style.cursor = "pointer";
      keyEl.addEventListener("mousedown", (e) => {
        e.preventDefault(); // Prevent focus loss
        if (!isCapturing && !editCaptureMode) return; // Only respond during capture

        if (MODIFIERS.includes(keyName)) {
          // Toggle modifier
          if (vkPressedModifiers.has(keyName)) {
            vkPressedModifiers.delete(keyName);
            highlightKey(keyName, false);
          } else {
            vkPressedModifiers.add(keyName);
            highlightKey(keyName, true);
          }
        } else {
          // Regular key: build combo from pressed modifiers + this key
          highlightKey(keyName, true);

          const combo: string[] = [...vkPressedModifiers, keyName];

          // Emit the same event as physical keyboard capture
          if (editCaptureMode) {
            editCapturedKeys = combo;
            renderKeyBadges(combo, $("edit-key-display"));
            invoke("stop_key_capture").then(() => {
              editCaptureMode = false;
              ($("btn-recapture") as HTMLButtonElement).textContent = "Re-capture Key";
              ($("btn-recapture") as HTMLButtonElement).classList.remove("capturing");
            });
          } else {
            onKeyCaptured(combo);
          }

          // Release visual state after a short delay
          setTimeout(() => {
            highlightKey(keyName, false);
            // Also release all modifiers
            vkPressedModifiers.forEach((mod) => highlightKey(mod, false));
            vkPressedModifiers.clear();
          }, 200);
        }
      });
    }
  });
}

function onKeyCaptured(keys: string[]) {
  capturedKeys = keys;
  const display = $("captured-key-display");
  const container = $("captured-keys");

  display.classList.remove("hidden");
  renderKeyBadges(keys, container);
}

// ═══════════════════════════════════════════════════════
// Key Entry CRUD
// ═══════════════════════════════════════════════════════
async function addKeyEntry() {
  const interval = parseInt(($("input-interval") as HTMLInputElement).value);
  const duration = parseInt(($("input-duration") as HTMLInputElement).value);

  if (capturedKeys.length === 0) {
    alert("Capture key terlebih dahulu");
    return;
  }

  try {
    await invoke("add_key_entry", {
      keys: capturedKeys,
      intervalMs: interval,
      durationMs: duration,
    });

    // Reset
    capturedKeys = [];
    $("add-key-form").classList.add("hidden");
    $("captured-key-display").classList.add("hidden");
    $("conflict-warning").classList.add("hidden");
    ($("input-interval") as HTMLInputElement).value = "100";
    ($("input-duration") as HTMLInputElement).value = "0";

    await loadEntries();
  } catch (e) {
    alert(e);
  }
}

function cancelAdd() {
  capturedKeys = [];
  $("add-key-form").classList.add("hidden");
  $("captured-key-display").classList.add("hidden");
  $("conflict-warning").classList.add("hidden");
}

async function loadEntries() {
  try {
    const entries = await invoke<KeyEntry[]>("get_entries");
    renderEntries(entries);
  } catch (e) {
    console.error("Failed to load entries:", e);
  }
}

function renderEntries(entries: KeyEntry[]) {
  const list = $("entries-list");
  const countEl = $("entry-count");

  countEl.textContent = `${entries.length} / 20`;

  if (entries.length === 0) {
    list.innerHTML = `
      <div class="empty-state">
        <span class="empty-icon">📋</span>
        <p>Belum ada key yang didaftarkan.</p>
        <p class="empty-hint">Gunakan tombol "Mulai Capture" untuk menambahkan key.</p>
      </div>
    `;
    return;
  }

  list.innerHTML = "";
  entries.forEach((entry, index) => {
    const item = document.createElement("div");
    item.className = "entry-item";
    item.innerHTML = `
      <div class="entry-info">
        <span class="entry-number">${String(index + 1).padStart(2, "0")}</span>
        <div class="entry-keys key-badges" id="entry-keys-${entry.id}"></div>
        <div class="entry-params">
          <div class="entry-param">
            <span class="entry-param-label">Interval</span>
            <span class="entry-param-value">${entry.interval_ms}ms</span>
          </div>
          <div class="entry-param">
            <span class="entry-param-label">Durasi</span>
            <span class="entry-param-value">${entry.duration_ms === 0 ? "∞" : entry.duration_ms + "ms"}</span>
          </div>
        </div>
      </div>
      <div class="entry-actions">
        <button class="btn-icon" title="Edit" data-action="edit" data-id="${entry.id}">✏️</button>
        <button class="btn-icon danger" title="Hapus" data-action="delete" data-id="${entry.id}">🗑️</button>
      </div>
    `;

    list.appendChild(item);

    // Render key badges
    const keysContainer = document.getElementById(`entry-keys-${entry.id}`);
    if (keysContainer) {
      renderKeyBadges(entry.keys, keysContainer);
    }

    // Event listeners for action buttons
    item.querySelectorAll("[data-action]").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const target = e.currentTarget as HTMLElement;
        const action = target.dataset.action;
        const id = target.dataset.id!;

        if (action === "edit") {
          openEditModal(entries.find((en) => en.id === id)!);
        } else if (action === "delete") {
          deleteEntry(id);
        }
      });
    });
  });
}

async function deleteEntry(id: string) {
  showConfirm(
    "Hapus Key Entry",
    "Yakin ingin menghapus key entry ini?",
    async () => {
      try {
        await invoke("delete_key_entry", { id });
        await loadEntries();
      } catch (e) {
        alert(e);
      }
    }
  );
}

async function clearAllEntries() {
  showConfirm(
    "Clear All",
    "Yakin ingin menghapus semua key entries di profile ini? Tindakan ini tidak bisa dibatalkan.",
    async () => {
      try {
        await invoke("clear_all_entries");
        await loadEntries();
      } catch (e) {
        alert(e);
      }
    }
  );
}

// ═══════════════════════════════════════════════════════
// Edit Modal
// ═══════════════════════════════════════════════════════
function openEditModal(entry: KeyEntry) {
  editingEntryId = entry.id;
  editCapturedKeys = null;

  // Set values
  renderKeyBadges(entry.keys, $("edit-key-display"));
  ($("edit-interval") as HTMLInputElement).value = String(entry.interval_ms);
  ($("edit-duration") as HTMLInputElement).value = String(entry.duration_ms);

  showModal("edit-modal");
}

async function saveEdit() {
  if (!editingEntryId) return;

  const interval = parseInt(($("edit-interval") as HTMLInputElement).value);
  const duration = parseInt(($("edit-duration") as HTMLInputElement).value);

  // Use re-captured keys if available, otherwise get from current entry
  let keys: string[];
  if (editCapturedKeys) {
    keys = editCapturedKeys;
  } else {
    // Get current keys from the entry
    const entries = await invoke<KeyEntry[]>("get_entries");
    const current = entries.find((e) => e.id === editingEntryId);
    if (!current) return;
    keys = current.keys;
  }

  try {
    await invoke("edit_key_entry", {
      id: editingEntryId,
      keys,
      intervalMs: interval,
      durationMs: duration,
    });

    editingEntryId = null;
    editCapturedKeys = null;
    hideModal("edit-modal");
    await loadEntries();
  } catch (e) {
    alert(e);
  }
}

// Re-capture key for editing
let editCaptureMode = false;

async function startEditRecapture() {
  try {
    await invoke("start_key_capture");
    editCaptureMode = true;
    ($("btn-recapture") as HTMLButtonElement).textContent = "Tekan key...";
    ($("btn-recapture") as HTMLButtonElement).classList.add("capturing");

    // Remove focus so physical keys don't trigger button clicks
    (document.activeElement as HTMLElement)?.blur();
  } catch (e) {
    alert(e);
  }
}

// ═══════════════════════════════════════════════════════
// Encryption Toggle
// ═══════════════════════════════════════════════════════
async function loadEncryptionStatus() {
  try {
    const enabled = await invoke<boolean>("get_encryption_status");
    const toggle = $("toggle-encryption") as HTMLInputElement;
    const status = $("encryption-status");

    toggle.checked = enabled;
    status.textContent = enabled ? "Aktif" : "Nonaktif";
    status.classList.toggle("active", enabled);
  } catch (e) {
    console.error("Failed to load encryption status:", e);
  }
}

async function toggleEncryption() {
  const toggle = $("toggle-encryption") as HTMLInputElement;
  const enable = toggle.checked;

  // Konfirmasi sebelum toggle
  const message = enable
    ? "Aktifkan enkripsi AES-256 untuk config file? Kunci akan disimpan di OS keychain."
    : "Nonaktifkan enkripsi config? File akan disimpan sebagai plain text dan kunci dihapus dari keychain.";

  showConfirm(
    enable ? "Aktifkan Enkripsi" : "Nonaktifkan Enkripsi",
    message,
    async () => {
      try {
        const result = await invoke<string>("toggle_encryption", { enable });
        showEncryptionFeedback(result, "success");
        await loadEncryptionStatus();
      } catch (e) {
        // Revert toggle on error
        toggle.checked = !enable;
        showEncryptionFeedback(String(e), "error");
      }
    }
  );

  // If user cancels confirm, revert toggle
  const originalNo = $("btn-confirm-no").onclick;
  $("btn-confirm-no").onclick = () => {
    toggle.checked = !enable;
    confirmCallback = null;
    hideModal("confirm-modal");
    $("btn-confirm-no").onclick = originalNo;
  };
}

function showEncryptionFeedback(message: string, type: "success" | "error") {
  const el = $("encryption-feedback");
  el.textContent = message;
  el.className = `encryption-feedback ${type}`;
  el.classList.remove("hidden");

  // Auto-hide after 5 seconds
  setTimeout(() => {
    el.classList.add("hidden");
  }, 5000);
}

// ═══════════════════════════════════════════════════════
// Modal Utilities
// ═══════════════════════════════════════════════════════
function showModal(id: string) {
  $(id).classList.remove("hidden");
}

function hideModal(id: string) {
  $(id).classList.add("hidden");
}

function showConfirm(title: string, message: string, onConfirm: () => void) {
  $("confirm-title").textContent = title;
  $("confirm-message").textContent = message;
  confirmCallback = onConfirm;
  showModal("confirm-modal");
}

// ═══════════════════════════════════════════════════════
// Init
// ═══════════════════════════════════════════════════════
window.addEventListener("DOMContentLoaded", async () => {
  // Load initial data
  await loadProfiles();
  await loadEntries();
  await loadEncryptionStatus();

  // ─── Init Visual Keyboard ───
  initVisualKeyboard();

  // ─── Global keyboard blocker during capture mode ───
  // Prevents physical key presses from interacting with DOM elements
  // (buttons, inputs, etc.) while rdev captures them via the global hook.
  // Without this, pressing Space/Enter while capture button is focused
  // triggers toggleCapture() again, immediately stopping the capture.
  document.addEventListener("keydown", blockKeyboardEvent, true);
  document.addEventListener("keyup", blockKeyboardEvent, true);
  document.addEventListener("keypress", blockKeyboardEvent, true);

  // ─── Key Capture Event from Rust ───
  await listen<string[]>("key-captured", (event) => {
    if (editCaptureMode) {
      // Re-capture for edit modal
      editCapturedKeys = event.payload;
      renderKeyBadges(event.payload, $("edit-key-display"));

      // Auto-stop after capture
      invoke("stop_key_capture").then(() => {
        editCaptureMode = false;
        ($("btn-recapture") as HTMLButtonElement).textContent = "Re-capture Key";
        ($("btn-recapture") as HTMLButtonElement).classList.remove("capturing");
      });
    } else {
      onKeyCaptured(event.payload);
    }
  });

  // ─── Visual Keyboard Events ───
  await listen<string>("key-pressed", (event) => {
    highlightKey(event.payload, true);
  });

  await listen<string>("key-released", (event) => {
    highlightKey(event.payload, false);
  });

  // ─── Event Listeners ───

  // Capture button
  $("btn-capture").addEventListener("click", toggleCapture);

  // Add key
  $("btn-add-key").addEventListener("click", addKeyEntry);
  $("btn-cancel-add").addEventListener("click", cancelAdd);

  // Clear all
  $("btn-clear-all").addEventListener("click", clearAllEntries);

  // Profile
  ($("profile-select") as HTMLSelectElement).addEventListener(
    "change",
    (e) => {
      switchProfile((e.target as HTMLSelectElement).value);
    }
  );
  $("btn-create-profile").addEventListener("click", createProfile);
  $("btn-delete-profile").addEventListener("click", deleteProfile);

  // Profile modal
  $("btn-save-profile").addEventListener("click", saveNewProfile);
  $("btn-cancel-profile").addEventListener("click", () =>
    hideModal("profile-modal")
  );
  ($("new-profile-name") as HTMLInputElement).addEventListener(
    "keydown",
    (e) => {
      if (e.key === "Enter") saveNewProfile();
    }
  );

  // Edit modal
  $("btn-save-edit").addEventListener("click", saveEdit);
  $("btn-cancel-edit").addEventListener("click", () => {
    editingEntryId = null;
    editCapturedKeys = null;
    if (editCaptureMode) {
      invoke("stop_key_capture");
      editCaptureMode = false;
    }
    hideModal("edit-modal");
  });
  $("btn-recapture").addEventListener("click", startEditRecapture);

  // Encryption toggle
  $("toggle-encryption").addEventListener("change", toggleEncryption);

  // Confirm modal — use stopPropagation to prevent backdrop interference
  $("btn-confirm-yes").addEventListener("click", (e) => {
    e.stopPropagation();
    if (confirmCallback) {
      const cb = confirmCallback;
      confirmCallback = null;
      hideModal("confirm-modal");
      cb();
    } else {
      hideModal("confirm-modal");
    }
  });
  $("btn-confirm-no").addEventListener("click", (e) => {
    e.stopPropagation();
    confirmCallback = null;
    hideModal("confirm-modal");
  });

  // Close modals by clicking backdrop
  document.querySelectorAll(".modal-backdrop").forEach((backdrop) => {
    backdrop.addEventListener("click", (e) => {
      e.stopPropagation();
      const modal = backdrop.parentElement!;
      modal.classList.add("hidden");

      // Cleanup capture if in edit mode
      if (editCaptureMode) {
        invoke("stop_key_capture");
        editCaptureMode = false;
      }
      editingEntryId = null;
      editCapturedKeys = null;
      confirmCallback = null;
    });
  });

  // Prevent clicks inside modal-content from reaching backdrop
  document.querySelectorAll(".modal-content").forEach((content) => {
    content.addEventListener("click", (e) => {
      e.stopPropagation();
    });
  });
});
