---
aliases:
  - Vaults
  - Libraries
  - Add vault
  - Add library
  - Remove vault
  - Manage vaults
  - Vault switcher
description: Learn how to add, manage, and remove libraries in Constellation.
---

# Library management

Constellation is a multi-library platform that lets you work with multiple Markdown libraries simultaneously. You can add, browse, and remove libraries through the library management interface.

## Adding a library

There are several ways to add a library:

1. **Welcome screen**: Click **+ Add Vault** on the welcome screen when no libraries are registered.
2. **Sidebar**: Click **+ Add vault** in the sidebar when the file explorer shows "No vaults yet".
3. **Vault dropdown**: Click the **Constellation** footer in the sidebar, then click **+ Open vault**.
4. **Vault manager**: Open **Manage vaults** and click **+ Add vault** at the bottom.
5. **Command palette**: Press Ctrl+P and select **Add vault**.

All methods open a folder picker dialog. Navigate to any folder containing Markdown files and select it. Constellation will index the library and display its notes in the sidebar.

> [!tip]
> Constellation reads your library files directly from their original location. It does not copy or move any files.

---

## Vault dropdown

The vault dropdown is a quick-access popup that appears above the sidebar footer.

### How to open

Click the **Constellation** button at the bottom of the left sidebar. The dropdown appears above it showing:

- **+ Open vault** — Opens the folder picker to add a new library.
- **Vault list** — All registered libraries with their color dot, name, and note count.
- **Manage vaults...** — Opens the full library management modal.

Press **Escape** or click outside the dropdown to close it.

> [!info]
> Constellation displays all libraries simultaneously. The dropdown serves as a management shortcut, not a library switcher.

---

## Vault manager

The vault manager is a modal dialog for managing all your registered libraries.

### How to open

1. Click the **Constellation** footer in the sidebar to open the vault dropdown.
2. Click **Manage vaults...** at the bottom.

### What you can do

For each library, the manager shows:

| Column | Description |
|--------|-------------|
| Color dot | The library's assigned color (matches the sidebar and graph) |
| Name | The library folder name |
| Path | The full filesystem path (truncated if long) |
| Note count | Number of markdown notes in the library |

### Per-library actions

Each library has two action buttons:

- **Open folder** (folder icon) — Opens the library folder in your operating system's file explorer (File Explorer on Windows, Finder on macOS, Files on Linux).
- **Remove** (trash icon) — Removes the library from Constellation after confirmation.

---

## Removing a library

> [!warning]
> Removing a library from Constellation does **not** delete any files. Your library folder and all its contents remain untouched on disk.

### Steps to remove

1. Open the vault manager (sidebar footer > **Manage vaults...**).
2. Click the **trash icon** next to the library you want to remove.
3. A confirmation dialog appears: "Remove [vault name] from Constellation? Your files won't be deleted."
4. Click **Remove** to confirm, or **Cancel** to keep the library.

### What happens on removal

When you remove a library, Constellation:

1. **Closes all open tabs** from that library.
2. **Stops the file watcher** for that library (no more live-reload for its files).
3. **Removes the library** from the registry and refreshes the sidebar.

The library's folder and files are never modified or deleted. You can re-add the same library at any time using the **+ Add vault** option.

---

## Library colors

Each library is automatically assigned a distinct color when added. These colors appear in:

- The file explorer sidebar (color dot next to the library name)
- The vault dropdown and vault manager
- The Sky View (nodes are colored by library)
- Tab labels in the editor

Library colors help you visually distinguish which library a note belongs to when working across multiple libraries.

---

## Keyboard shortcuts

| Action | Shortcut |
|--------|----------|
| Add library | Ctrl+Shift+O |
| Close vault dropdown | Escape |
| Close vault manager | Escape |
