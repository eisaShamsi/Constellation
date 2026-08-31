---
aliases:
  - Universes
  - Universe setup
  - Universe manager
  - Open existing universe
  - Linked universe
description: Learn how to create, open, and manage Universes in Constellation — your portable data containers for libraries, bases, settings, and more.
---

# Universe

A **Universe** is a portable, user-owned directory where Constellation stores all your data — libraries, bases, bookmarks, settings, workspaces, and property types. Unlike traditional app data folders hidden deep in your system, a Universe lives wherever you choose and can be moved between devices.

## Universe directory structure

```
My Universe/
├── universe.json          # Name, creation date, Linked Universe references
├── libraries.json         # Registered libraries
├── bases/                 # Base files (.base)
├── bookmarks.json         # Bookmarked notes
├── settings.json          # App settings
├── workspaces.json        # Workspace layouts
└── property-types.json    # Property type mappings
```

---

## First-launch setup wizard

When you launch Constellation for the first time, a setup wizard guides you through creating or opening a Universe.

### Step 0: Welcome

You are presented with two options:

| Option | Description |
|---|---|
| **Create New Universe** | Set up a fresh universe with a name and location of your choice. |
| **Open Existing Universe** | Open a universe folder you've already created on this or another device. |

> [!tip]
> If you've used Constellation on another computer and copied your Universe folder over, choose **Open Existing Universe** to reconnect it instantly.

### Step 1: Name and location

If you chose **Create New Universe**:

1. Enter a **Universe Name** (e.g., "My Research", "Work Notes").
2. Click **Choose Folder** to pick where the universe directory will be created.
3. Click **Next** to create the universe and proceed.

### Step 2: Add libraries and Linked Universes

After creating your universe, you can immediately connect your data:

- **Add Library** — Opens a folder picker. Select a Markdown library folder.
- **Add Child Universe** — Link another existing Universe as a Linked Universe (the button's label predates the current name). Its libraries become available to your new universe automatically.
- **Skip for now** — Proceed without adding anything. You can always add libraries and Linked Universes later.

Click **Finish** when you're done.

> [!tip]
> You can add as many libraries and Linked Universes as you like during setup. They can also be managed later through the Universe Manager and Library Manager.

---

## Opening an existing universe

To open a Universe that already exists on your filesystem:

1. On the welcome screen, click **Open Existing Universe**.
2. Navigate to the folder that contains `universe.json`.
3. Constellation reads the universe metadata and registers it automatically.

This is useful when:
- You copied a Universe folder from another device.
- You reinstalled Constellation and want to reconnect your data.
- A colleague shared their Universe folder with you.

---

## Universe Manager

Access the Universe Manager from the sidebar footer (click the universe name) to manage multiple universes.

| Action | Description |
|---|---|
| **Switch** | Activate a different universe. All libraries, settings, and caches reload. The window title and status bar update instantly. |
| **Create New** | Create an additional universe. |
| **Remove** | Remove a universe from the list (files are preserved on disk). |
| **Open Folder** | Open the universe directory in your file explorer. |
| **Add Child Universe** | Link another Universe as a Linked Universe to share its libraries (the button's label predates the current name). |

The active universe is highlighted with a **green badge** and green border, making it easy to identify at a glance.

### Switching performance

Switching universes is designed to be fast:

- Essential data (settings, bookmarks, workspaces) loads **in parallel**.
- The app UI becomes usable **immediately** after libraries load.
- File watchers start and cached views are brought up to date **in the background** while you browse.
- All previous state (tabs, trees, caches) is fully cleared before loading the new universe.

---

## Linked Universes (Universe of Universes)

A Universe can link other Universes as **Linked Universes** — peers whose libraries are federated in. When you add a Linked Universe, its libraries automatically become available in the universe that links it — no duplication needed.

**Example:**

- **Universe A** has libraries: L1, L2, L3
- **Universe B** has libraries: L4, L5
- **Universe X** links A and B as Linked Universes, plus its own library L6
- **Effective libraries for X:** L1, L2, L3, L4, L5, L6

If L7 is later added to Universe B, Universe X automatically sees it.

Linked Universe libraries are fully integrated across all features:
- **Sky View** — Linked Universe library notes appear with their own color and convex hull.
- **Search** — notes from Linked Universe libraries are included in search results.
- **Cross-library linking** — you can link between your own libraries and Linked Universe libraries using `[[wikilinks]]`.

> [!warning]
> Circular references are detected and prevented. If Universe A references B and B references A, each is only resolved once.

---

## Sidebar display

Linked Universes appear in the left sidebar **above** your libraries, separated by a thin divider line. Each Linked Universe is shown with a **globe icon** to visually distinguish it from libraries (which use a colored dot). The library count for each Linked Universe is displayed alongside its name.

This flat layout keeps things simple — no collapsible categories, just universes first, then libraries.

---

## Window title and status bar

The active universe name appears in two places:

- **Window title bar** — Displays as **Constellation - UniverseName**.
- **Status bar** (bottom-right corner) — Shows the universe name alongside library and note counts.

Both update immediately when you switch universes through the Universe Manager.

---

## Portability

Since a Universe is a self-contained directory, you can:

- **Move it** to a different drive or location.
- **Copy it** to another computer.
- **Back it up** with any file sync or backup tool.
- **Share it** with others (library paths will need to be re-registered on the new machine).

> [!tip]
> The only thing stored outside your Universe is a small registry file in the app data directory that remembers which Universes exist and which one was last active.

### Portable Universes

Constellation universes are fully portable. You can move a universe folder to any location — a different drive, USB stick, or another computer — and Constellation rewrites the universe's library list when you reopen it, so your notes and folders appear immediately. The search index is a separate store and still holds the old location, so click the repair button on the moved-universe bar; Constellation refuses to remove index entries when that many look wrong at once, so nothing is at risk even if you wait.

To move a universe:
1. Close Constellation
2. Move or copy the universe folder to the new location
3. Open Constellation → it does NOT announce the old path is gone — it quietly opens a different universe instead
4. Choose **Open Existing Universe** and point to the new location
5. All notes and libraries appear immediately — use the **“Repair the index — safe, keeps everything”** button on the bar that appears — it backs up first, rewrites the stored locations in one step, and keeps link ages, review rhythm and everything earned. Do NOT use a Full re-read for a move: it rebuilds from scratch and resets every link's birth date

The universe folder structure follows the Obsidian model: notes go directly in the root folder, configuration lives in `.constellation/`.

---

## RTL support

The Universe setup wizard and manager fully support right-to-left (RTL) languages including Arabic, Hebrew, Persian, and Urdu. The interface direction adapts automatically based on your language setting.

## Bringing a folder in — and when Constellation refuses

**Bring In a Library** takes a folder of notes that lives outside your universe and adds it as a
library, either **Copy in** (the original stays put) or **Move in** (the folder relocates). This is
how **One Universe, One Location** is kept: nothing is ever referenced in place from outside.

Bring In refuses, changing nothing, when:

- **the folder is a universe of its own** — open it from the universe switcher instead;
- **the folder sits inside another universe**, even one not listed on this computer. The refusal
  names that universe. Moving the folder would take content out of it, so move it from within that
  universe instead;
- **the folder is already a registered library** — the unification proposal relocates those safely,
  keeping their index intact;
- **Constellation cannot read its own library list** — momentarily locked or damaged. It refuses
  rather than guessing whether the folder is already registered. Usually temporary; try again.

### Moving between two drives

A *Move* across drives copies first, then removes the original — and **keeps the original** when the
folder contains a shortcut or junction that cannot be copied, naming the link and both locations. If
the copy fails part-way, the partial copy is removed, so a failed Bring In never leaves half a
library inside your universe.
