# Session Log — 2026-06-14

## §MIG-077 — App-Wide Right-Click Context Menus — Phase A continuation (A3 →)

**Function in hand: MIG-077 Phase A** — consolidating the 6 hand-rolled context menus onto
the shared `ContextMenu` (+ a separate `<ConfidencePicker>` for the one radio-group menu),
per the Boss-approved Option A. Resumed at **A3 (OrgChart)** per `HANDOVER-2026-06-14.md`.

**Session ritual:** `git pull origin main` (already up to date at `ed15d3ad`); orientation
**v2.79 read** (v2.79 preamble + §3 architecture); `HANDOVER-2026-06-14.md` read in full;
`MIG-077-RIGHTCLICK-CONTEXT-MENUS-PLAN.md` read in full. A0/A1/A2 are shipped + Boss-validated
(commits `eb3a246f`, `381a471e`, `a999c165`, `8aab9db2`). Plan approval = build approval —
cascading A3 → A4 → A5 → Phase B → Phase 4, stopping only at the [GATE] Boss tests.

### A3 — OrgChart node menu → shared `ContextMenu` — SHIPPED (awaiting Boss gate)

**Commit:** `<pending>` · svelte-check **0 errors** · `npm run build` + `cargo build --release`
(1m59s) green · bundle-confirmed.

- Replaced the inline `.oc-fs-ctx` menu (`OrgChart.svelte`) with `getOrgNodeMenuItems(node): MenuItem[]`
  rendered via the shared `<ContextMenu>` — the proven A1/A2 idiom. Open for a note; an
  Expand/Collapse toggle for a container.
- **Latent bug fixed.** The old render was `$t('contextMenu.open') || 'Open'`, but the bare
  `contextMenu.open` key **never existed** — and Constellation's custom `t()` returns the literal
  key string on a miss (active-locale → en → key), which is truthy, so the `|| 'Open'` fallback
  was dead and the menu button rendered the literal text **"contextMenu.open"**. A3 adds the
  proper key, so the label now reads correctly in every locale.
- **i18n ×15:** added `contextMenu.{open,expand,collapse}` natively to all 15 locales (derived from
  existing app vocabulary — `open` from `contextMenu.openLink`'s verb; `expand`/`collapse` from
  `sidebar.expandAll`/`collapseAll` minus the "all" quantifier — not invented). The canonical home
  for these (reused later by B1/B2/B5/B6). Replaces the previous reuse of `sidebar.expandAll`
  ("Expand all") which was wrong wording for a single-node toggle.
- **Dead code removed:** the now-unused `handleCtxAction` function + the `.oc-fs-ctx` CSS block.
- **Bundle proof:** `build/` contains `open:"Open",expand:"Expand",collapse:"Collapse"` in BOTH
  the main app chunk (`_app/immutable/chunks/D7b8oqZV.js`) and the second-screen bundle
  (`assets/screen-CMUc73KQ.js`) — fresh frontend embedded. Binary mtime 14:31 == build time.

### BOSS STEER (after the A3 thin gate): menus must be RICH + CONTEXTUAL

At the A3 gate Eisa rejected the faithful 1-item consolidation: *"What is the use of a right-click
with only one command? I want the full list, like: Delete, Rename, Move, etc."* (notes) and *"Same
thing!"* (containers) — MIG-077's origin observation #3. AskUserQuestion settled three decisions:
**(1)** note menu = FULL, build everything now; **(2)** container menu = RICH; **(3)** NOT
identical-everywhere but **contextual** — *"it should be contextual and adapt to each type of
function."* Plan updated with the ADDENDUM; the thin A3 (`2e95b04a`) stays in history, its wiring +
keys carry forward. Re-scoped: A3-R1 shared builder → A3-R2 OrgChart ready set+rename → A3-R3 Move
→ A3-R4 Add tag (each its own gate, staged).

### A3-R1/R2 — contextual rich menu + OrgChart — SHIPPED (awaiting Boss gate)

**Commit:** `<pending>` · svelte-check **0 errors / 315 warnings** (baseline) · `npm run build` +
`cargo build --release` (2m18s) green · bundle-confirmed · binary mtime 17:12.

- **NEW `src/lib/components/contextMenuBuilder.ts`** — `buildContextMenu(target, actions)`: ONE
  shared source, contextual output by **(object kind × surface capability)**. An item appears only
  when its callback is provided AND fits the kind; group-based separators stay clean regardless of
  which items are present. This IS the "contextual" mechanism (each surface passes the callbacks it
  can fulfil). Reused later by the file tree, List-mode, Search, Sky View.
- **NEW `src/lib/components/RenameDialog.svelte`** — a small reusable rename dialog (RTL-aware via
  `detectDir`; reuses `actions.rename`/`dialogs.cancel` — no new strings). Full-page surfaces have
  no inline tree row, so they rename through this → the host's existing `handleRenameComplete`
  (rename + wikilink cascade + collision dialog all reused).
- **OrgChart** now builds its menu via `buildContextMenu` and emits a single `onNodeMenuAction(action,
  target)` to `+layout`'s new `handleOrgNodeMenuAction` (every op reuses an existing handler —
  openNoteTab / clipboard / handleSuggestSourcesForNote / confirmDelete / handleCreate* /
  handleRenameComplete; Expand/Collapse stays OrgChart-local). `libIdForPath` = longest-prefix lib
  lookup (correct with nested libraries).
- **R2 menu:** notes → Open · Open in new tab · Rename · Copy path · Copy name · Suggest sources ·
  Delete. Folders → New Note/Folder/Base · Expand/Collapse · Rename · Delete. Libraries → New
  Note/Folder/Base · Expand/Collapse. (**Move + Add tag** = A3-R3/R4.)
- **Reveal in tree OMITTED — it is dead app-wide.** Repo-wide grep: `constellation:reveal-in-tree`
  is dispatched (editor breadcrumb, `+layout:6333`) but **has no listener anywhere**. Shipping it in
  the menu would be a dead item — against Eisa's "useful menus." Flagged as a separate fix
  (`spawn_task` task_bd6d4802); will be added to the menus once the listener exists.
- **i18n ×15:** `contextMenu.{openInNewTab,move,addTag}` added natively (move = `tableToolbar.move`
  verbatim; addTag = `addProperty` grammar with the native "tag" noun; openInNewTab = standard
  localized phrasing). Bundle-confirmed (`openInNewTab:"Open in new tab"`, `addTag:"إضافة وسم"`).
- **Known minor (logged):** deleting/creating from the full-page chart doesn't auto-refresh the
  chart (the node lingers until reopened) — cosmetic; the file op itself is correct. Follow-up.

### task_bd6d4802 — "Reveal in file tree" listener (was dead app-wide) — SHIPPED

Interleaved on Boss's action of the chip I spawned during A3-R2 (the menu omitted Reveal-in-tree
because it was dead). Root cause: `constellation:reveal-in-tree` was **dispatched** (editor breadcrumb,
NotePane → `+layout:6333`) but **had no listener anywhere in src**. Fix:

- **`+layout` `revealInTree(path)` + listener** (`onMount`, cleaned up on destroy): switch to tree
  mode → longest-prefix library match over `$libraryStats` → expand the child universe (if nested)
  and the library (`toggleLibrary` lazy-loads the tree) → after `tick()` + 2 frames, find the row by
  `[data-tree-path]`, DOM-open every ancestor `<details>` (persists — Svelte only applies `open` at
  creation), `scrollIntoView({block:'center'})`, brief outline flash (1.6 s).
- **`FileTree`**: `data-tree-path={entry.path}` added to the note button + folder summary (rows were
  previously un-locatable by path).
- **Now functional everywhere:** the editor breadcrumb (⋯) "Reveal in file tree" works, AND
  "Reveal in tree" is added back to the OrgChart contextual menu (notes + folders), no longer omitted.
- svelte-check 0 errors / 315 warnings; build green; binary 17:28; `data-tree-path` + `reveal-in-tree`
  bundle-confirmed.

**Boss GATE — A3-R2 OrgChart rich menu + reveal-in-tree: PASS** (2026-06-14).

### A3-R3 — Move (destination-folder picker) — SHIPPED (awaiting Boss gate)

`moveItem(source, target)` existed but was drag-drop-only (NotebookNavigator + OrgChart internal);
right-click Move needed a picker. The Rust `move_item` already guards: both paths must be inside a
library (a note can't escape the universe) and it errors on a destination name collision.

- **NEW `MoveDialog.svelte`** — a searchable, indented folder list scoped to the source's own library
  (move_item rejects cross-library anyway). Single-select + Move; double-click a folder = move. The
  collision error from `move_item` is shown **inline** (dialog stays open). RTL-safe. No new i18n
  (reuses `contextMenu.move` / `layout.search` / `dialogs.cancel`).
- **`+layout`**: `openMoveDialog(path, name)` loads the source library's full-depth tree, flattens to
  folders, and **excludes** the source itself + its descendants (can't move into own subtree) + its
  current parent (a no-op). `handleMoveConfirm` → `moveItem` → `refreshLibraryTree` + `loadAllStats`.
- **Menu**: `move` added to notes + folders in the OrgChart contextual menu.
- svelte-check 0 errors / 315 warnings; build green; binary 18:41; MoveDialog bundle-confirmed.

**Boss GATE — A3-R3 same-library move: directional pass, but "I want to move across the WHOLE
universe" (cross-library).** Reworked to universe-wide:

- **Two correctness gaps found + fixed in Rust** (`move_item` previously did NOT reindex — a latent
  bug for ALL moves incl. drag-drop): after a move the FTS/links index kept the OLD path, and a
  cross-library move would index the note under the wrong library. `move_item` now **reindexes on
  move** (mirrors `rename_item` Step 6): drop the old entry + add the moved note(s) under the
  destination library. Folder move walks every `.md` descendant (`collect_md_paths`) and reindexes
  each at its new path.
- **NEW IPC `list_universe_folders`** — a lightweight Rust-side walk returning folders ONLY across
  every library + federated child universe (skips dot-folders). Keeps the heavy enumeration in Rust
  (Rule 3) — the frontend never reads thousands of note rows just to build the picker.
- **MoveDialog now universe-wide**: folders grouped under each library root (library-root rows are
  bold with a universe glyph), a `loading` state while the IPC runs, exclusions applied across all
  libraries (source + descendants + current parent). `handleMoveConfirm` refreshes BOTH the source
  and the (possibly different) target library tree.
- svelte-check 0 errors / 315 warnings; Rust release clean; `npm run build` → `cargo build --release`
  (re-embed); binary 19:19; `list_universe_folders` bundle-confirmed.

**CRASH (Boss test of universe-wide Move) — diagnosed + fixed (Reproduce-First).** The Move dialog
hung on "…" and the app crashed. No fresh `constellation-crash.log` (both on disk were stale —
05-26 / 04-08) → an **OOM/abort, not a catchable panic**. `find` over the user's tree reported a
real **filesystem junction loop**. Root cause (read off the proven walker, not guessed): my new
`collect_folders` / `collect_md_paths` used the link-FOLLOWING `path.is_dir()` with **no symlink
skip**, while the proven `read_dir_recursive` has an explicit *"Skip symlinks to prevent circular
recursion"* guard. Following a directory junction loop → exponential folder visits → OOM crash; the
walk never returned, so the picker hung on "…". **Fix:** both walkers now skip symlinks
(`entry.file_type().is_symlink()`) BEFORE touching the path — mirrors `read_dir_recursive` exactly.
Rust release rebuilt; binary 20:22. (The dialog was always cancellable via Escape/click-outside —
the lock was the Rust walk, not the UI.)

**STILL CRASHED on 20:22 → REPRODUCED → REAL root cause found (the symlink fix was treating the
wrong thing).** Per LL-014 (three-strike) I stopped guessing and reproduced against Eisa's ACTUAL
universe (`Eisa Cognitive Knowledge`, 19 libraries) with a Node script mirroring the walk:
- The walk is **fast (≤12ms/library) and loop-free** — 130 folders total, **0 reparse points, 0
  cycles**. So `collect_folders` was never the cause; the symlink guards (still kept — correct
  defensive code for other machines) were irrelevant here.
- A second repro built the picker's entry list and found **1 DUPLICATE path**:
  `…/Eisa Cognitive Knowledge/New Library Test`. That library is **nested inside the universe root**,
  and the `universe_notes` library's path **IS** the universe root — so it appeared **twice** (once
  from the root's walk, once as its own library root). The dialog's keyed `{#each shown as f
  (f.path)}` **crashes Svelte on a duplicate key** → "froze and crash." The Node walk-repro missed
  it because it didn't render; the second repro (entry-build) caught it.
- **Fix:** `openMoveDialog` **dedupes entries by normalized path** (Set, keep first). Crash gone;
  the nested library now shows once.

### File-tree right-click → shared rich menu (Boss: "why can't I move a note from the file tree?")

The file tree still used its old menu (Rename / Suggest / Delete). Routed `getContextMenuItems`
through the shared `buildContextMenu`: notes gain **Open · Open in new tab · Move · Copy path · Copy
name** alongside Rename / Suggest (md) / Delete; folders gain **Move** + the create set; library
roots unchanged (create-only). **Rename stays INLINE** (the tree's native `renamingPath` edit, not
the dialog — the per-surface contextual difference); Reveal-in-tree omitted (this IS the tree).
- svelte-check 0 errors / 315 warnings; `npm run build` → `cargo build --release`; binary 20:44.

**Boss GATE — file-tree Move PASS; OrgChart Move worked but left the chart STALE** (the move
happened on disk + showed in the file tree, but the OC kept the old node; acting on it failed with
"doesn't exist"). Plus a general "some lag" observation.

### A3-R3 follow-up — OrgChart staleness fix

The fullscreen OrgChart cached `constellation_map_universe` into `mapRoot` and never reloaded.
- **OrgChart** gains a `refreshKey` prop + `reloadFullscreenData()` that re-fetches the tree
  **preserving the user's expand set + pan/zoom** (a move no longer collapses the chart); stale
  expanded paths in the Set are harmless.
- **+layout** `markOrgChartDirty()`: bumps `refreshKey` if the chart is open (reload in place), else
  sets `orgChartDirty` for the next open. Called from `handleMoveConfirm`, `handleDeleteConfirm`,
  `handleRenameComplete` (rename suppresses the watcher, so it needs the explicit mark), AND the
  `library-changed` watcher (catch-all for create + external). On dock **re-open**, reload only if
  `orgChartDirty` — so an unchanged re-open does NOT trigger a redundant whole-universe reload (lag-
  conscious).
- svelte-check 0 errors / 315 warnings; build green; binary 21:50.
- **Lag:** asked Eisa to pinpoint (opening Move / opening the chart / the menu / moving). The OC
  reload is now gated to actual changes to avoid redundant heavy reloads; further perf work pending
  a specific locus.

**Boss GATE — staleness fix + file-tree Move: (pending re-test).** Lag loci named: chart-open + the
right-click menu + general. Diagnosis: general lag is almost certainly CPU contention from my
back-to-back release builds ON Eisa's machine (each `cargo build --release` ~2min at full CPU) —
process fix: build then go idle for tests. Chart-open heaviness (`constellation_map_universe` full
depth every open) is pre-existing, not a MIG-077 regression. Spawned a measured perf task
(`task_a75e6a23`) for chart-open + menu render. Eisa chose: **continue A3-R4**, perf as its own task.

### A3-R4 — Add tag (content-integrity-safe) — SHIPPED (awaiting Boss gate)

The last A3 action; writes a note's frontmatter `tags:` — the content-integrity class (MIG-076), so
the write path is the careful part:
- **OPEN note** → goes through the **model** (`composeNoteModel` to read current content,
  `addTagToProps`, then `saveTabContent` — the SAME identity-guarded path PropertyEditor uses).
  Preserves unsaved body edits; a path mismatch is refused. A disk write behind an open model is
  **never** done (the model would later overwrite it and lose the tag).
- **CLOSED note** → gated `writeNote(…, 'add_tag')` (the WriteGate path, self-attesting + journaled)
  + `reindexNote`.
- `addTagToProps` mirrors PropertyEditor's list semantics (dedup case-insensitive; `value =
  items.join(', ')`; strips a leading `#`). Input via the reused single-input `RenameDialog`
  (title "Add tag — {note}"). Menu item wired on notes in OrgChart + the file tree.
- svelte-check 0 errors / 315 warnings; build green; binary 22:29; `add_tag` origin bundle-confirmed.
- **Gate must exercise the Editor-Surface checklist**: closed-note add, OPEN-note add with unsaved
  edits (no loss), existing-tags preserved, RTL tag.
