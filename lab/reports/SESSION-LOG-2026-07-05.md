# Session Log — 2026-07-05 (MIG-092 · Collections on the Search Hub)

**Branch:** `main` · **Predecessor session:** MIG-091 close-out (tip `10dd275e`).

## MIG-092 — Collections on the Search Hub (unifying Bookmarks)

**Concept (the horse):** *A Collection is the working set of notes I'm forming a
task around — hand-picked from what a search surfaces, named, and kept alive
across sessions — so the notes I judged worth keeping outlive the query that
found them and stay actionable.* (The "hold" half of the ratified MIG-090 v2
Navigator horse; the rejected "translate" half — the Intent Bar — is deleted.)

**Boss decisions (2026-07-05):**
- **Unify Bookmarks** into Collections (Starred = the pinned favorites collection).
- **Header-tab** layout inside the Search Hub (Results | Collections).
- (Default) clean rename `workbench*` → `collections*`.

**Whole-entity fit:** vs Search Hub / Bases / File Explorer all gap-defined
(hand-picked static ≠ query ≠ hierarchy). The adversarial dedup pass caught an
existing **Bookmarks** feature (absent from the Boss's dedup list AND the v2
whole-entity map) → Boss ruled UNIFY. Docs: `docs/MIG-092-Collections-Architect.md`,
`docs/MIG-092-Collections-Plan.md`.

**Sequencing note:** plan §9a (retire the Workbench *surface*) is pulled forward
into §2 because the store rename requires its consumers gone first (tree stays
green at every commit). §9b (flag + Settings toggle + bookmarks-write retire +
final grep + dead `showWorkbench`/CSS sweep) stays at the end.

### §1 — Persistence rename — commit `3e6aba1b`
- `workbench.rs`→`collections.rs` (`WorkbenchRow`→`CollectionRow`, `workbench_hydrate`
  →`collections_hydrate`); `universe.rs` `read/save_universe_workbench`→`…_collections`
  writing `collections.json`; read adopts a legacy `workbench.json` once
  (idempotent + reversible via `workbench.json.migrated`); init writes +
  `files_to_move` updated; `lib.rs` module + command registration renamed.
- **Verify:** `cargo check` clean; `cargo test collections::` → 2 passed.

### §2 — Collections store (multi-set + Starred + Bookmarks migration) — commit `5f7cf376`
- New **pure** module `collectionsLogic.ts` (dependency-free reducers) + a thin
  `store.ts` writable/IPC shell over it. Item model gains `type?: note|folder|search`
  (+ inline `name/libraryName` for the non-note members unified from Bookmarks).
- `loadCollections` runs the one-time **Bookmarks→Starred** migration
  (idempotent; legacy `bookmarks.json` retained as backup).
- Retired the Workbench **surface** (deleted `WorkbenchView.svelte`; removed the
  `+layout` dock button, palette entries, context-menu `addToWorkbench` actions,
  overlay mount; re-pointed `loadCollections`/`migrateCollectionPath`).
- **Verify:** svelte-check **0 errors**; `tests/mig-092/collections.test.ts` **14/14**;
  full suite 221/222.

### Open / carried
- **Pre-existing failing test (NOT mine):** `tests/sight-v6/tradition-isolation.test.ts`
  (dome-position assertion, line 128) fails identically at the §1 tip with §2
  changes stashed. Sight is a disabled Wing — out of MIG-092 scope; tracked
  separately for a Boss ruling.
- **Remaining §2b/§9b cleanup:** dead `showWorkbench` state + reset-chain no-ops +
  `.workbench-overlay`/`.w-*` CSS + `enabledFeatures.workbench` flag + Settings
  toggle + `workbench.*` i18n keys + retire `save_universe_bookmarks` write.

### §3 — Mixed-member hydration — commit `14f95491`
- `collectionsLogic.ts`: `HydratedNoteRow`/`CollectionDisplayRow`; `noteHydrationKeys`
  (only note members → cids/paths); `buildDisplayRows` (live notes / missing /
  inline folder+search, order preserved). `store.ts`: `hydrateCollectionNotes`.
- **Verify:** tests/mig-092 16/16.

### §4 — Search-Hub Collections tab — commit `4e74b8b2`
- `CollectionsPanel.svelte`: active collection via NoteList/NoteRow; switch/
  create/rename/delete (Starred pinned); per-member open/done/remove + sweep;
  re-hydrate only on key-set change (untracked snapshot → no loop, no per-toggle
  IPC); empty-state; `L()` English fallback.
- `SearchHub.svelte`: Results | Collections tab strip (RTL, count badge).
- **Verify:** svelte-check 0 errors.

### §5 — Pick-up ("Add to collection ▸") — commit `24e13c74`
- `contextMenuBuilder.ts`: repurposed the `addToWorkbench` slot → an
  "Add to collection ▸" one-level submenu (each collection + "New collection…");
  `tl()` English fallback; not gated on isMarkdown.
- `+layout.svelte`: shared `wireCollectionPickup()` wired into the file-tree
  note menu AND the search-result menu; SearchHub gets `onRevealPath=revealInTree`.
- **Verify:** svelte-check 0 errors.

### §6 — Sidebar re-point (Bookmarks ≡ Starred) — commit `f74f8ca8`
- `store.ts`: `toggleStarred(item)` (path-aware, removes by existing key).
- `+layout.svelte`: sidebar "Bookmarks" renders the **Starred** collection
  (`starredItems`); ⭐ command + `handleToggleBookmark` + `toggleBookmarkPath`
  route to Starred; context `bookmarked` flag → `isInStarred`. Legacy bookmarks
  store now write-only-dead (removed in §9b).
- **Verify:** svelte-check 0 errors; no stray isBookmarked/addBookmark/removeBookmark;
  content-integrity harness `tests/mig-076` + `tests/mig-092` → **44/44**.

### → STOP for Boss test (Stage 1)
Frontend rebuilt (`npm run build`; "This collection is empty" confirmed in
`build/assets/screen-*.js`); release `constellation.exe` building. Stage-1 tutorial
covers: sidebar bookmarks survived as Starred; ⭐ round-trip; Collections tab
create/add-from-search/switch/rename/delete/done/remove; restart persistence.

### Remaining after Boss validation
§7 chips (reuse workbenchChips) · §8 liveness · §9b cleanup (dead `showWorkbench` +
`.workbench-overlay`/`.w-*` CSS + `enabledFeatures.workbench` flag + Settings toggle
+ retire `save_universe_bookmarks` write + legacy `bookmarks` store + `workbench.*`
i18n) · §10 close-out (help + User Manual ×15 + full 15-locale `collections.*` i18n +
Orientation v-bump + /simplify + boot re-measure + audit trio).
