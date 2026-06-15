# 02 — File Tree (Concept Paper)

> One of the Phase-1 core-spine functions. The sidebar tree is how a note is *found and opened* — it feeds the [Editor](01-Note-Editor.md) (the gate). Follows the template in [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3 and serves [00-Constellation](00-Constellation-Core-Concept-Paper.md).

## 1. Function in hand
The **File Tree** — the sidebar's library/folder/note browser. Renderer: `src/lib/components/FileTree.svelte` (a recursive `<svelte:self>` row component); hosted, wired, and given its context menu by `src/routes/+layout.svelte`. (Distinct from `src/lib/components/NotebookNavigator.svelte`, which is the *two-pane* folders/tags/properties browser used by the second screen / list mode — a separate function, not this one.)

## 2. Purpose
The ONE job: **present the on-disk hierarchy (Library → Folder → Note) and let the user open or organize a note.** It is the navigational entrance to **Observation** — you cannot connect, contend, or synthesize a note you cannot find and open. It also carries the first light-touch signals of the knowledge lifecycle into the periphery: a per-note **stage** emoji (🌱📖🔗✨) and a **maturity** left-border (sapling/evergreen/canonical/wilting), so the tree hints at *where a note sits in its life* without opening it. Justified: *File Over App* makes the `.md` tree the source of truth; the File Tree is the window onto that tree.

## 3. What it is NOT
- **Not** the Editor — it does not own or mutate note *content*; it opens a path and hands off to the gate.
- **Not** a derived-view computer — stage/maturity badges are *displayed* here but should be *maintained* upstream at write time (see §7; this is a live Rule-8 concern, not a clean state).
- **Not** the Library Manager — library-root rows offer create-only (New Note / Folder / Base); rename/delete of a *library* live elsewhere.
- **Not** the second-screen browser (`NotebookNavigator`) — that is a different two-pane component.

## 4. Wiring
- **Inputs (props/stores):** `entries: FileEntry[]` (the tree, from `read_library_tree`), `maturityMap` / `stageMap` (`SvelteMap` path→state), `$activeTab` / `$openTabs` / `$splitActive` (for the active-row highlight), `renamingPath` (inline-edit target), `color`/`libraryName`.
- **Inputs (IPC, via host):** `read_library_tree { path, maxDepth }` on library-expand; `scan_note_stages` + `compute_note_maturity` (fire-and-forget on expand) to populate the badge maps.
- **Outputs (callbacks → host):** `onNoteClick(path,name,_,e)` → `handleNoteClick` → `openNoteTab`; `onFolderClick`; `onContextMenu(entry,x,y)` → `handleContextMenu`; `onRenameComplete(oldPath,newName)` → `handleRenameComplete` (the rename cascade — writes disk + repoints links).
- **Outputs (writes):** none directly from `FileTree.svelte`; all disk/IPC writes happen in the host handlers it calls (rename, move, delete, create).
- **Consumers:** the Editor (receives the open), Sky View (`skyViewSelectedPath` set on click), the rename/move/delete handlers.
- **Connection to the Editor (the gate):** a row click calls `onNoteClick` → the host's `handleNoteClick` → `openNoteTab(...)`, which mounts the note in the Editor. The tree never reads or writes content — it only supplies a path to the gate. The active-row class mirrors `$activeTab`/`$openTabs`, so the gate is the source of truth for "what's open," not the tree.

## 5. Right-click / context menu
- **Has one — and it is SHARED, not hand-rolled.** `FileTree.svelte` only forwards the event (`oncontextmenu → onContextMenu(entry,x,y)`); the host builds items through `getContextMenuItems()` → **`buildContextMenu()`** (MIG-077 A3-R, `src/lib/components/contextMenuBuilder.ts`). All labels flow through `$t()`. This satisfies the core paper's §5 "one shared builder" contract.
- **Items per target kind** (from `getContextMenuItems`):
  - **Note:** Open · Open in new tab · Rename (inline, via `renamingPath`) · Move · Add tag · Copy path · Copy name · Suggest sources (`.md` only) · Delete.
  - **Folder:** New Note · New Folder · New Base · Rename (inline) · Move · Delete.
  - **Library root** (`isLibraryRoot`): New Note · New Folder · New Base only (create-only; rename/delete suppressed by design).
- **Reachable only by right-click:** Move, Add tag, Copy path/name, Suggest sources, New Folder/Base, and Delete have no other affordance on the tree row itself (left-click opens; double-click is not a distinct action here). Rename is also primarily right-click-reached (then inline).
- **Note (debt, not a gap):** *drag-and-drop reordering/move is NOT implemented on the sidebar tree* — there is no `draggable`/`ondrop` on `FileTree.svelte` rows (verified by grep). Moving is context-menu "Move" → folder-picker dialog. If the bring-up wants drag-move, that is new work, not a regression to restore.

## 6. Multilingual
- **`FileTree.svelte` itself has no hardcoded English chrome** — its only on-screen text is file/folder *names* (note content, rendered verbatim) and the rename `<input>` (no label). No `$t()` calls are needed in the component, and none are missing.
- **All context-menu labels are localized** via `$t()` in `buildContextMenu` across all 15 locales (ar de en es fa fr he hi ja ko pt ru tr ur zh).
- **RTL / direction:** the tree uses logical CSS properties throughout (`padding-inline-start`, `border-inline-start`, `text-align: start`, `margin-inline-end`), so rows mirror correctly in RTL. **Verify in bring-up:** that mixed-script note titles in the row get `dir="auto"`/`detectDir()` (the component renders the title in a plain `<span>` — per-title direction detection should be confirmed, see the standing MIG-014 §2F "NotePane badge dir=auto polish" follow-up).

## 7. Boot behavior
- **Runs at boot?** The tree *structure* does **not** walk the filesystem at boot — `read_library_tree` is called **lazily on library-expand** (a user action), which is correct and respects boot-perf. `enrichNodesBackground` was deliberately removed from boot.
- **Rule 8 status — MIXED, contains a violation to fix.** The tree hierarchy itself is a cheap on-demand read (OK). **But the stage emoji + maturity border are RECOMPUTED on read:** each library-expand fires `scan_note_stages` (which `scan_stages_recursive` re-reads every `.md`'s YAML from disk) and `compute_note_maturity`. These are read-time/expand-time recomputes of a derived view, not lookups of a persisted, trigger-maintained store — exactly the shape Rule 8 forbids. The right end-state is to persist stage + maturity write-time (note-save hook / trigger, like FTS5) and have the tree read them, the same way tags moved to `cache_boot_snapshot_graph`.
- **Cost:** tree read per library = one bounded directory walk to `maxDepth` (cheap; unmeasured here — *estimate* a few ms for a small library). The stage/maturity scans are **O(notes × file-read)** per expand — *unmeasured on a 7,600-note library; must be measured in bring-up* (a large library expand could stutter the badge fill).

## 8. Flag / gate & bring-up position
- **Gate today:** **none.** The File Tree has no `enabledFeatures.*` flag — it is part of the core sidebar and renders unconditionally. (Contrast: Sky View, Index, CCS, etc. are each behind `enabledFeatures.*` / `SIGHT_*_ENABLED`.) No new gate is needed to *enable* it; if a truly minimal shell ever wanted to hide it, that would be a new core-spine guard.
- **Bring-up phase:** **1 (Core spine)** — alongside the Editor. Depends on: the app shell, `$libraries`/`$libraryStats`, and `read_library_tree`. The stage/maturity enrichment depends on the Rule-8 fix above before it can be called "clean."

## 9. Budget
- **Boot budget:** zero boot-time filesystem walk for the tree (already met — lazy on expand). Library-expand should feel instant; the badge-enrichment scans must not block the expand (they are fire-and-forget today — keep it that way, but move them to persisted reads).
- **Interaction budget:** row click → Editor open within one tab-open (~1–3 ms disk read, per [01](01-Note-Editor.md) §7); expand/collapse instant; inline rename has no perceptible lag. No `invoke()` on hover or scroll.
- **Regression guard:** expand a large library (7,600+ notes) and confirm (a) the tree appears immediately and (b) badge fill does not stutter the row list; rename a note and assert the cascade repoints links (linked-probe-pair, per the Editor-Surface Gate §6).

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** every note in every library is findable, openable, and organizable from the tree.
- [ ] **Serves Constellation's core purpose:** it is the entrance to Observation — opening a note feeds the Editor (the gate); it adds no storage-PKM bloat.
- [ ] **Wires correctly to the Editor:** a row click opens exactly one tab in the Editor; the active-row class tracks `$activeTab`; the tree never reads/writes content.
- [ ] **Right-click present + correct (shared, not hand-rolled):** uses `buildContextMenu` (MIG-077); items per kind (note/folder/library-root) verified; the right-click-only actions (Move, Add tag, Copy, Suggest sources, Delete, New Folder/Base) all work.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** menu labels localize ×15; rows mirror in RTL (logical CSS verified); per-title `dir="auto"`/`detectDir()` confirmed on mixed-script titles.
- [ ] **Within budget:** large-library expand instant; badge fill does not stutter; no `invoke()` on hover/scroll.
- [ ] **Obeys Rule 8:** stage + maturity are **read from a persisted, write-time-maintained store**, not recomputed by `scan_note_stages` / `compute_note_maturity` on expand. *(Open — currently a violation.)*
- [ ] **Holds its invariants:** rename cascade repoints links (linked probe pair); move/delete route through the safe handlers; active-row reflects the gate's truth.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (badge-scan cost unmeasured; Rule-8 fix pending)**
Notes: Renderer is `FileTree.svelte`; menu + handlers in `+layout.svelte`. Two honest findings for the bring-up: (1) **Rule 8 violation** — stage/maturity badges recompute on each library-expand via `scan_note_stages` + `compute_note_maturity` instead of reading a persisted write-time store; this is the central fix. (2) **No drag-and-drop** on the sidebar tree (move is via the context-menu "Move" dialog) — confirm whether that is intended or a gap to fill. The shared right-click menu (MIG-077) and localized labels are already in good shape.
