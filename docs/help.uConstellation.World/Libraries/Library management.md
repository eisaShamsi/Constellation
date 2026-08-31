---
aliases:
  - Libraries
  - Add library
  - Remove library
  - Manage libraries
  - Library switcher
description: Learn how to add, manage, and remove Markdown libraries in Constellation.
---

# Library management

Constellation is a multi-library reader that lets you work with multiple Markdown libraries simultaneously. You can add, browse, and remove libraries through the library management interface.

## Adding a library

There are two distinct operations:

1. **Create a new library** — Constellation builds a fresh library folder at a location you choose, then registers it.
2. **Link an existing library** — you point Constellation at a folder that's already on disk (your Obsidian vault, an old notes folder, a synced Dropbox directory, etc.), and Constellation registers it without modifying any files.

Both operations are reachable from the same set of surfaces:

- **Welcome screen** when no libraries are registered yet.
- **Sidebar toolbar** — the **+ Library** button.
- **Library Manager** screen — **Create new library** and **Link existing library** cards side by side.
- **Mission Control** (`Ctrl+P`) — *New library* and *Add library* commands.

### Create a new library

When you choose *New library* from any of those surfaces, the **Create dialog** opens. It contains:

- A **Location** field showing the parent folder where the library will be created. Click **Pick…** to open an OS folder picker (Explorer / Finder / Files) and choose any folder on your computer.
- A **Name** field pre-filled with *My Library* and pre-selected — start typing to replace it.
- **Cancel** and **Create** buttons.

The dialog blocks empty names, names with illegal characters (`\ / : * ? " < > |`), and parent-directory escapes (`..`) before the operation reaches your filesystem. If a folder with that name already exists at the location, Constellation tells you and the dialog stays open so you can rename and try again.

After Create, the new library appears in the sidebar with its own color, ready to receive notes.

### Link an existing library

When you choose *Link existing library*, an OS folder picker opens directly. Navigate to the folder you want to register and select it. Constellation indexes its Markdown files and displays them in the sidebar.

> [!tip]
> Linking does not copy or move any files. Constellation reads your library files directly from their original location. If you also use Obsidian, you can link an Obsidian vault and both apps will work on the same files.

---

## How a library looks

A Library is a first-class citizen in Constellation — more than a folder — so it carries its own **mark**. Everywhere a library appears, it shows as a small **library building** (a pediment over columns), tinted in the library's own colour:

- **In the sidebar file tree** — next to each library's name.
- **In the Move dialog** — the destination list shows each library as a bold, colour-tinted building row, so you can tell a whole library apart from an ordinary folder (folders keep the plain folder icon).
- **On the Dashboard** — each library card carries the same mark.

A **Linked Universe** (a peer Universe whose libraries are federated in) is marked differently — a small **planet-and-orbit** glyph — because it is a whole other Universe, not a library inside this one.

You can resize the library mark: **Style Setter → Library → Icon size**. This affects the library building glyph only; the toolbar icons and folder icons have their own size controls.

---

## Library dropdown

The library dropdown is a quick-access popup that appears above the sidebar footer.

### How to open

Click the **Constellation** button at the bottom of the left sidebar. The dropdown appears above it showing:

- **+ Open library** — Opens the folder picker to add a new library.
- **Library list** — All registered libraries with their color dot, name, and note count.
- **Manage libraries...** — Opens the full library management modal.

Press **Escape** or click outside the dropdown to close it.

> [!info]
> Constellation displays all libraries simultaneously. The dropdown serves as a management shortcut, not a library switcher.

---

## Library manager

The library manager is a modal dialog for managing all your registered libraries.

### How to open

1. Click the **Constellation** footer in the sidebar to open the library dropdown.
2. Click **Manage libraries...** at the bottom.

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

1. Open the library manager (sidebar footer > **Manage libraries...**).
2. Click the **trash icon** next to the library you want to remove.
3. A confirmation dialog appears: "Remove [library name] from Constellation? Your files won't be deleted."
4. Click **Remove** to confirm, or **Cancel** to keep the library.

### What happens on removal

When you remove a library, Constellation:

1. **Closes all open tabs** from that library.
2. **Stops the file watcher** for that library (no more live-reload for its files).
3. **Removes the library** from the registry and refreshes the sidebar.

The library's folder and files are never modified or deleted. You can re-add the same library at any time using the **+ Add library** option.

---

## Library colors

Each library is automatically assigned a distinct color when added. These colors appear in:

- The file explorer sidebar (color dot next to the library name)
- The library dropdown and library manager
- The sky view (nodes are colored by library)
- Tab labels in the editor

Library colors help you visually distinguish which library a note belongs to when working across multiple libraries.

---

## Keyboard shortcuts

| Action | Shortcut |
|--------|----------|
| Add library | Ctrl+Shift+O |
| Close library dropdown | Escape |
| Close library manager | Escape |
