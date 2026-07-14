---
aliases:
  - Notes Management
  - Sidebar
  - File Explorer
  - Sky View
  - OrgChart
  - Organization Chart
  - File tree
  - PiP overlay
  - Picture-in-Picture
description: The Notes Management sidebar unifies the File Explorer (with filter, sort, multi-select, and batch operations) and Sky View (OrgChart) into a single tabbed panel for browsing and organizing your knowledge base.
---

# Notes Management

The Notes Management sidebar is the primary way to browse and organize your notes in Constellation. It unifies the **File Explorer** — a folder tree with filter, sort, multi-select, and batch operations — and the **Organization Chart (Sky View)** into a single sidebar panel with mode tabs.

> The former dual-pane "Notes Navigator" (List mode) has been retired; its file-management strengths (filter, richer sort, multi-select, batch) now live directly in the File Explorer, and its facet browsing is served by the Tags panel and Search Hub.

## Elements toolbar

The top row of the sidebar always shows the **Elements toolbar** with quick-action buttons:

| Button | Action |
|--------|--------|
| **New Note** | Open the Create dialog to make a new note |
| **New Base** | Open the Create dialog to make a new base (structured data note) |
| **New Folder** | Open the Create dialog to make a new folder |
| **New Library** | Open the Create dialog to make a new library |

These buttons are always visible regardless of which mode tab is active.

---

## The Create dialog

Every "create" affordance in Constellation — the toolbar buttons above, the right-click menu on a folder or library, and the "+ New …" command-palette entries — opens the same modal dialog. This is intentional: you name the new item **before** it lands on disk, and validation happens upfront, so the typical operating-system create flow is preserved across the whole app.

### What you see

When you click any "create" affordance, a dialog appears with these fields:

| Field | Behavior |
|-------|----------|
| **Title** | Identifies what's being created — *New Note*, *New Folder*, *New Base*, or *New Library*. |
| **Location** | The parent folder. **Read-only** when you invoked the create from a context that already knows the location (right-clicking a folder, or the toolbar button which uses the active library's root). **Pickable** for *New Library* — a *Pick…* button next to the field opens an OS folder picker so you can choose where the library lives. **Hidden** for *New Base* in the workspace (workspace bases always live in the workspace folder; there is no location to pick). |
| **Name** | Pre-filled with a sensible default (e.g. *New Folder*, *Untitled*, *Untitled Base*, *My Library*) and pre-selected so a single keystroke replaces it. Type the name you actually want. |
| **Libraries to query** | *(New Base only, workspace flow)* A multi-select list of libraries the base will read from. *All libraries* is the default. |
| **Create** / **Cancel** | Buttons at the bottom. *Create* commits; *Cancel* closes without side effect. |

### Keyboard

| Key | Behavior |
|-----|----------|
| **Enter** | Same as clicking *Create*. |
| **Escape** | Same as clicking *Cancel*. Closes the dialog without creating anything. |
| **Click outside the dialog** | Same as Escape. |
| **Tab** | Moves focus between Location, Name, Libraries, Cancel, Create. |

The Name field has focus when the dialog opens, with the default name pre-selected — so you can start typing immediately.

### Validation

The dialog won't let you click *Create* with a name that won't work on disk:

- **Empty name** — *Create* is disabled. The dialog shows "Name cannot be empty."
- **Illegal characters** — `\`, `/`, `:`, `*`, `?`, `"`, `<`, `>`, `|` are blocked at the input. The dialog shows the full character list inline. *Create* is disabled until you remove them.
- **A note title already exists** *(anywhere in the universe)* — clicking *Create* opens the **collision dialog** (Change name / Overwrite / Cancel), described under *When a name already exists* below, rather than an inline error. For **folders**, a name that already exists at the location is reported back into the dialog's error region, and the dialog stays open so you can rename and try again.

> [!info]
> The dialog also blocks names containing `..` (parent-directory escapes) — this is enforced both in the dialog and in the create operation itself, so a slip of the keyboard cannot accidentally place a folder outside the location you picked.

### Right-click affordances in the sidebar

Right-clicking surfaces the same dialog from these surfaces:

| Right-click on | Menu shows | What happens |
|----------------|------------|--------------|
| **A note in the file tree** | Open · Open in new tab · Rename · Move · Add tag · Copy path · Copy name · Reveal in tree · Suggest sources · Delete | Opens, renames, moves (universe-wide folder picker), adds a tag to frontmatter, copies path/name to clipboard, reveals in tree, classifies, or deletes the note. |
| **A folder in the file tree** | New Note · New Folder · New Base · Rename · Move · Delete | The first three open the Create dialog with that folder pre-filled as Location. Move opens a universe-wide folder picker. |
| **A library row** (the universe-notes header, an own library, or a child-universe library) | New Note · New Folder · New Base | Opens the Create dialog with that library's root pre-filled as Location. Library rows do not offer Rename or Delete here — those operations live in the Library Manager. |

### Renaming a note updates every link to it

When you rename a note — either from the **file tree** (right-click → Rename) or by **editing its title** at the top of the page — Constellation rewrites every `[[wikilink]]` that points to it, across the whole library, to the new name. You don't fix links by hand, and links never silently break.

While those links are being updated you'll see a brief read-only **"Updating links…"** overlay on the affected note(s); the editor accepts no typing for that moment, so nothing is lost while the note reloads. On a small library this is near-instant; on a very large library it may take a second or two. The note that previously linked to the old name still resolves afterward, because the old title is kept as an **alias** on the renamed note.

### When a name already exists — the collision dialog

Constellation keeps every note title unique **across your whole universe** — every library, and every linked child universe. Unique titles are what let `[[wikilinks]]` resolve to exactly one note; two notes sharing a title would make a link ambiguous.

So when you **create** a note with a name you typed, or **rename** a note, and that name already belongs to another note *anywhere* in the universe, a dialog appears instead of silently changing your name or failing:

- **Header** — "A note named *Foo* already exists", with a line beneath showing **which library** it already lives in (e.g. "Already in: History"). That can be a *different* library than the one you're working in — the check spans the entire universe, child universes included.
- **Change name** — a box is pre-filled with a free suggestion (e.g. *Foo 1*). Edit it if you like, then confirm to create or rename under that name instead.
- **Overwrite** — replaces the existing note. The displaced note is **moved to its library's `.trash`** first, so it stays recoverable — never hard-deleted. If a same-named note is already in that `.trash`, the new one is filed alongside it with a numeric suffix (*Foo 1*, *Foo 2*), so trashing never overwrites an earlier discarded copy.
- **Cancel** (button, *Escape*, or click outside) — closes the dialog and does nothing.

This applies only to a name you **type**. Quick Capture, which auto-names notes, keeps its own automatic numbering and never interrupts you. Folders are not affected — only notes carry titles that links resolve against.

### Deleting notes is recoverable

When you delete a note or folder — right-click → **Delete** in the file tree, or the multi-select **Delete** in the File Explorer's batch bar — where it goes is governed by **Settings → Universe & Libraries → "Deleted files"**:

- **System trash** *(the default)* — the OS Recycle Bin / Trash, recoverable through your operating system.
- **`.trash` folder** — a hidden `.trash` kept either **inside the note's own library** or **at the universe root** (a sub-option appears when you pick this). Recoverable on disk; the `.trash` folder is hidden from your tree and search.

Either way the note disappears from the tree and search immediately but is **not** erased. There is deliberately **no "permanently delete"** choice — routine deletes are always reversible. (A displaced note from an Overwrite, or a same-named note already in `.trash`, is kept with a numeric suffix so nothing in the trash is ever clobbered.)

### Templates apply uniformly

When you create a new note, Constellation looks up any folder template configured for the parent folder and applies it. **This now happens regardless of how you invoked the create** — the toolbar button, the right-click menu, and the command palette all run the same path. Earlier versions skipped templates on the right-click path; that inconsistency is fixed.

---

## Mode tabs

The second row contains the mode tabs. Click a tab to switch how your notes are displayed:

| Tab | Icon | Description |
|-----|------|-------------|
| **Tree** | Folder tree icon | File Explorer — browse, filter, sort, multi-select, and batch-organize your notes and folders |
| **OrgChart** | Tree diagram icon | Sky View — interactive tree-list hierarchy visualization |

Your selection and scroll position are preserved when switching between tabs.

> [!note]
> The Sky View ribbon icon has been removed from the left dock. Sky View (OrgChart) is now accessible via the OrgChart mode tab in this sidebar. Sky View (the full 3D graph) is still available via `Ctrl+G` or Mission Control.

---

## Adaptive sidebar width

The sidebar automatically adjusts its width to fit the longest library or child universe name visible in the current view. This ensures all names are fully readable without manual resizing.

---

## Child universe grouping

Across all three modes, content is organized with consistent grouping:

1. **Child universes first** — each child universe appears as a collapsible group with its libraries nested inside
2. **Own libraries below** — the parent universe's own libraries appear below a visual separator

This grouping is consistent across Tree and OrgChart modes, so switching tabs does not change the structural hierarchy.

---

## Cross-mode selection sync

Clicking a child universe, library, folder, or note in any sidebar mode highlights the corresponding nodes in the Sky View graph (if Sky View is open). This bidirectional sync helps you maintain spatial awareness as you browse your knowledge base in different modes.

---

## Picture-in-Picture (PiP) overlay

When Sky View is open and you click a child universe, library, or folder in the sidebar, a **Picture-in-Picture (PiP)** window appears as a resizable overlay on top of the main Sky View graph.

### What the PiP shows

| Feature | Description |
|---------|-------------|
| **Filtered sub-graph** | Only nodes belonging to the selected scope (universe, library, or folder) |
| **Filtered legend** | Its own color legend showing only the relevant entries |
| **Resizable** | Drag the edges or corners to resize the PiP window |
| **Repositionable** | Drag the title bar to move the PiP anywhere on screen |

The PiP provides a focused view of a subset of your graph without losing the full Sky View context behind it.

---

## Tree mode (File Explorer)

The file tree for browsing **and organizing** your notes and folders. It carries full file-management strength for large libraries.

### Filter by name

A filter box sits at the top of the tree. Type any fragment of a note or folder name (in any language) and the tree narrows to matches, automatically opening the folders that contain them. The filter searches **every** library — collapsed ones are loaded and revealed, then restored to how you had them when you clear the filter. It matches **names only**, never note contents (searching *inside* notes is Search Hub's job).

### Sort

The sort button cycles through eight orders: **Name** (A→Z / Z→A), **Modified** (newest / oldest), **Created** (newest / oldest), and **Size** (largest / smallest). Folders always stay on top; hover the button to see the current sort.

### Multi-select

- **Ctrl-click** (⌘-click on Mac) — add or remove a note or folder from the selection
- **Shift-click** — select a whole range
- **Plain click** — still just opens the note; the selection stays until you press **Escape** or clear it
- Selected rows are highlighted with an accent bar; both notes and folders can be selected

### Batch operations

With items selected, a bar appears at the bottom of the sidebar showing the count:

- **Add tag** — add a tag to every selected note
- **Move** — move the selection into one folder (a universe-wide picker)
- **Delete** — delete the selection (trash-backed, with a count confirmation)

Every batch action runs through the same safe, gated operations a single note uses — so batch-tagging never corrupts a note. Notes from linked child-universes (read-only) are skipped automatically.

### The basics

- Expand and collapse folders by clicking or using arrow keys
- Right-click for a contextual menu — notes get Open, Open in new tab, Rename, Move, Add tag, Copy path/name, Reveal in tree, Suggest sources, Delete; folders get New note, New folder, New base, Rename, Move, Delete; library roots get New note, New folder, New base
- Drag and drop to move notes between folders
- **Move** opens a universe-wide folder picker spanning all libraries — search or scroll, double-click to move instantly
- Folders and notes are grouped by child universe membership

---

## OrgChart mode (Sky View)

An interactive tree-list visualization of your entire knowledge base hierarchy.

### Hierarchy sources

Switch the hierarchy source using the dropdown:

| Source | What it shows |
|--------|---------------|
| **Folders** | Library > Folder > Subfolder > Note (default) |
| **Tags** | Tag taxonomy tree (nested tags like `#science/physics` become branches) |
| **MOC Links** | Hub notes with 5+ outgoing links and their targets |
| **Parent Property** | Notes with `parent: [[X]]` in frontmatter form a chain |

### Interactions

| Action | Effect |
|--------|--------|
| **Click a folder** | Expand or collapse its children |
| **Click a note** | Opens it in the editor |
| **Right-click a note** | Context menu: Open, Open in new tab, Rename, Move, Add tag, Copy path, Copy name, Reveal in tree, Suggest sources, Delete |
| **Right-click a folder** | Context menu: New note, New folder, New base, Expand/Collapse, Rename, Move, Reveal in tree, Delete |
| **Right-click a library** | Context menu: New note, New folder, New base, Expand/Collapse |
| **Double-click a folder** | Drill down — re-roots the chart at that folder |
| **Breadcrumb trail** | Navigate back up after drilling down |
| **Ctrl+F** | Search — highlights the path from root to matching nodes |

---

## Keyboard navigation

| Key | Action |
|-----|--------|
| **Arrow Up/Down** | Navigate the file list or tree |
| **Arrow Left/Right** | Collapse/expand folders in Tree and OrgChart modes |
| **Enter** | Open the focused note |
| **Ctrl/⌘-click** | Add or remove a note or folder from the multi-selection (Tree mode) |
| **Shift-click** | Select a range (Tree mode) |
| **Escape** | Clear the multi-selection |

## Your work is protected — saving, external edits, and renames

Constellation is built so that **no ordinary action can silently lose the text you've typed.** Three things you may see, and what they mean:

- **The save-health banner ("Couldn't save … — your edit is safe and will retry").** If a note's file can't be written for a moment — usually because a sync tool (Syncthing, OneDrive, iCloud) or antivirus is briefly holding it — Constellation keeps your edit in memory and in a crash-safe recovery net, shows this banner, and **keeps retrying automatically** until the file frees up. Your unsaved text is never lost: it survives closing and reopening the note, switching between notes, and even restarting the app, and the banner stays visible until a real save lands. Besides **Retry now**, the banner offers two explicit exits when a file stays stuck:
  - **Save a copy** — writes your unsaved version to a new note right next to the original (named "… (recovered copy)" in your language) and opens it in a new tab. Your work is durably on disk; the stuck original keeps retrying and its banner stays until it frees up.
  - **Discard…** — deliberately drops your unsaved change and keeps the file exactly as it is on disk. The button first turns into **"Really discard?"** so a stray click can't lose anything; click it again to confirm. This is the only way unsaved work is ever dropped — always your explicit choice, never the app's.

- **Renaming a note updates its links safely.** When you rename a note, Constellation automatically rewrites every `[[wikilink]]` that points to it. If one of the notes being updated happens to be open with unsaved edits *and* its file is momentarily locked, Constellation **skips updating that one note this time** to protect your unsaved work — its link still resolves via the old name and catches up on the next save. Every other note updates normally, and the app never freezes.

- **When a note is edited outside Constellation while it's open.** If the same note is changed by another program or a sync tool while you have it open, Constellation adopts the outside change if you have no unsaved edits; if you *do* have unsaved edits, it keeps **both** — your version stays on screen and the outside version is saved as a separate `.conflict` side-copy. A banner offers **Merge…** (a side-by-side view where you reconcile the two, with a "Copy to mine" button per difference) and **Show copy** (reveals the side-copy in your file explorer). Nothing is ever overwritten without your choice.
