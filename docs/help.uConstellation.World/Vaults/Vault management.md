---
aliases:
  - Vaults
  - Add vault
  - Remove vault
  - Manage vaults
  - Vault switcher
description: Learn how to add, manage, and remove Obsidian vaults in Constellation.
---

# Vault management

Constellation is a multi-vault reader that lets you work with multiple Obsidian vaults simultaneously. You can add, browse, and remove vaults through the vault management interface.

## Adding a vault

There are several ways to add an Obsidian vault:

1. **Welcome screen**: Click **+ Add Vault** on the welcome screen when no vaults are registered.
2. **Sidebar**: Click **+ Add vault** in the sidebar when the file explorer shows "No vaults yet".
3. **Vault dropdown**: Click the **Constellation** footer in the sidebar, then click **+ Open vault**.
4. **Vault manager**: Open **Manage vaults** and click **+ Add vault** at the bottom.
5. **Command palette**: Press Ctrl+P and select **Add vault**.

All methods open a folder picker dialog. Navigate to your Obsidian vault folder and select it. Constellation will index the vault and display its notes in the sidebar.

> [!tip]
> Constellation reads your vault files directly from their original location. It does not copy or move any files.

---

## Vault dropdown

The vault dropdown is a quick-access popup that appears above the sidebar footer.

### How to open

Click the **Constellation** button at the bottom of the left sidebar. The dropdown appears above it showing:

- **+ Open vault** — Opens the folder picker to add a new vault.
- **Vault list** — All registered vaults with their color dot, name, and note count.
- **Manage vaults...** — Opens the full vault management modal.

Press **Escape** or click outside the dropdown to close it.

> [!info]
> Unlike Obsidian, which switches between vaults one at a time, Constellation displays all vaults simultaneously. The dropdown serves as a management shortcut, not a vault switcher.

---

## Vault manager

The vault manager is a modal dialog for managing all your registered vaults.

### How to open

1. Click the **Constellation** footer in the sidebar to open the vault dropdown.
2. Click **Manage vaults...** at the bottom.

### What you can do

For each vault, the manager shows:

| Column | Description |
|--------|-------------|
| Color dot | The vault's assigned color (matches the sidebar and graph) |
| Name | The vault folder name |
| Path | The full filesystem path (truncated if long) |
| Note count | Number of markdown notes in the vault |

### Per-vault actions

Each vault has two action buttons:

- **Open folder** (folder icon) — Opens the vault folder in your operating system's file explorer (File Explorer on Windows, Finder on macOS, Files on Linux).
- **Remove** (trash icon) — Removes the vault from Constellation after confirmation.

---

## Removing a vault

> [!warning]
> Removing a vault from Constellation does **not** delete any files. Your Obsidian vault folder and all its contents remain untouched on disk.

### Steps to remove

1. Open the vault manager (sidebar footer > **Manage vaults...**).
2. Click the **trash icon** next to the vault you want to remove.
3. A confirmation dialog appears: "Remove [vault name] from Constellation? Your files won't be deleted."
4. Click **Remove** to confirm, or **Cancel** to keep the vault.

### What happens on removal

When you remove a vault, Constellation:

1. **Closes all open tabs** from that vault.
2. **Stops the file watcher** for that vault (no more live-reload for its files).
3. **Removes the vault** from the registry and refreshes the sidebar.

The vault's folder and files are never modified or deleted. You can re-add the same vault at any time using the **+ Add vault** option.

---

## Vault colors

Each vault is automatically assigned a distinct color when added. These colors appear in:

- The file explorer sidebar (color dot next to the vault name)
- The vault dropdown and vault manager
- The graph view (nodes are colored by vault)
- Tab labels in the editor

Vault colors help you visually distinguish which vault a note belongs to when working across multiple vaults.

---

## Keyboard shortcuts

| Action | Shortcut |
|--------|----------|
| Add vault | Ctrl+Shift+O |
| Close vault dropdown | Escape |
| Close vault manager | Escape |
