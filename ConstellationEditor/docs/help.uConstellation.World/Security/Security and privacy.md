---
aliases:
  - Security
  - Privacy
  - Lock screen
  - Library encryption
  - API key protection
description: Learn how Constellation protects your library data, locks your app, and secures API keys.
---

# Security and privacy

Constellation is a local-first desktop application. Your library data stays on your device and is never uploaded to external servers. All AI-related API calls are routed through the Tauri backend, meaning your API keys never touch the browser layer.

You can find security settings under **Settings > Security**.

## Security features

Constellation offers three optional security features you can enable independently:

| Feature | Purpose | Status |
|---------|---------|--------|
| [[#Library encryption]] | Encrypt cached library data at rest | Optional |
| [[#Lock on idle]] | Lock the app after inactivity | Optional |
| [[#Secure API key storage]] | Store API keys in the OS keyring | Optional |

All security features are **disabled by default** and can be toggled on or off at any time.

---

## Library encryption

> [!abstract] What it does
> When enabled, Constellation encrypts your cached library data at rest on the local device. This protects your data if someone gains access to your filesystem.

### How it works

- Constellation caches library metadata (file trees, search indices, tag maps) locally for performance.
- With library encryption enabled, this cached data is encrypted before being written to disk.
- Your original `.md` files in the library folder are **not modified** — Constellation is a reader and does not alter source files.
- Encryption uses the operating system's native secure storage facilities via the Tauri backend.

### How to enable

1. Open **Settings** (Ctrl+, or the gear icon in the sidebar).
2. Navigate to **Security**.
3. Toggle **Library encryption** to on.
4. The status badge will change from "Disabled" to "Enabled".

> [!tip]
> Library encryption protects cached data only. Your original library files are managed by your operating system's file-level permissions and disk encryption (e.g., BitLocker on Windows, FileVault on macOS).

---

## Lock on idle

> [!abstract] What it does
> When enabled, Constellation automatically locks after a period of inactivity. A full-screen lock overlay appears, requiring your security PIN to resume.

This feature is useful for shared workstations or when you step away from your computer and want to prevent casual access to your notes.

### Setting up lock on idle

1. Open **Settings > Security**.
2. Toggle **Lock on idle** to on.
3. If you haven't set a PIN yet, you'll be prompted to create one.
4. Enter a **4 to 8 digit PIN** and confirm it.
5. Choose your **idle timeout** from the dropdown:
   - 1 minute
   - 5 minutes (default)
   - 10 minutes
   - 15 minutes
   - 30 minutes
   - 60 minutes

### How the lock screen works

- After the configured idle timeout with no mouse, keyboard, or touch activity, a full-screen lock overlay appears.
- The lock screen covers the entire app and blocks all keyboard shortcuts.
- Enter your PIN and press Enter or click the arrow button to unlock.
- If you enter the wrong PIN, the input field shakes and an error message is displayed.
- After successful unlock, the idle timer resets automatically.

### Changing your PIN

1. Open **Settings > Security**.
2. While **Lock on idle** is enabled, click **Change PIN**.
3. Enter and confirm your new 4-8 digit PIN.
4. Click **Confirm** to save.

### Security considerations

- Your PIN is stored as a SHA-256 hash — the actual PIN is never saved in plain text.
- The PIN hash is stored in your local application settings (localStorage).
- The lock screen is a **convenience feature** designed to prevent casual access. It is not a substitute for operating system-level login security.
- If you forget your PIN, you can reset it by clearing Constellation's application data.

> [!tip]
> For maximum security, combine the lock screen with your operating system's screen lock (Win+L on Windows, Ctrl+Cmd+Q on macOS).

---

## Secure API key storage

> [!abstract] What it does
> When enabled, Constellation stores your AI provider API keys in the operating system's secure keyring (Windows Credential Manager, macOS Keychain, or Linux Secret Service) instead of keeping them only in memory.

### How it works

By default, API keys are:
- Held in memory during your session only.
- Passed directly to the Tauri Rust backend for API calls.
- Never written to browser localStorage or any file on disk.

With **Secure API key storage** enabled:
- API keys are additionally stored in the OS-level secure keyring.
- Keys persist securely between app sessions — you don't need to re-enter them.
- The OS keyring encrypts stored credentials using your system login credentials.

### Supported providers

| Provider | Requires API key | Supports secure storage |
|----------|-----------------|------------------------|
| OpenAI | Yes | Yes |
| Claude (Anthropic) | Yes | Yes |
| Google Gemini | Yes | Yes |
| Ollama (Local) | No | N/A |

### How to enable

1. Open **Settings > Security**.
2. Toggle **Secure API key storage** to on.
3. The status badge will change from "Disabled" to "Enabled".
4. Your existing API key (if configured in Settings > AI Provider) will be migrated to the secure keyring.

> [!tip]
> Ollama runs locally and does not require an API key, so secure storage is not applicable for Ollama users.

---

## Data handling

### What stays local

Constellation is designed as a local-first application:

- **Library files**: Read directly from your local filesystem. Never copied or uploaded.
- **Settings**: Stored in browser localStorage under the `constellation-settings` key.
- **Locale preference**: Stored in localStorage under the `constellation-locale` key.
- **Workspaces and bookmarks**: Stored in localStorage.
- **AI API keys**: Held in memory only (or in OS keyring if secure storage is enabled).

### What leaves your device

The only data that leaves your device is when you use the **AI Provider** features:

- Note content is sent to your configured AI provider (OpenAI, Anthropic, Google Gemini) for processing.
- All API calls are routed through the Tauri Rust backend — they do not pass through the browser.
- If you use **Ollama**, all AI processing happens locally on your machine.

### Network access

Constellation requires network access only for:

- AI provider API calls (if configured)
- Checking for app updates (if enabled)

No telemetry, analytics, or usage data is collected or transmitted.

---

## Reporting security issues

If you discover a security vulnerability in Constellation, please report it responsibly:

- Open an issue on [GitHub](https://github.com/uConstellation/constellation/issues) with the label `security`.
- Alternatively, email the developer directly.

Do not publicly disclose security vulnerabilities before they have been addressed.
