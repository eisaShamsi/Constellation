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

### Boss test Stage 1 — findings + fixes
Binary built 20:05; Eisa tested. **Test 3 (create + add from search) PASS. Test 4
(restart persistence) PASS.** Two findings on the sidebar Bookmarks (now Starred):
- **Test 1** — the per-row library label repeated (all bookmarks in one library
  showed "Eisa Test" twice). Ruling: flat list, each row shows its location
  breadcrumb **cUniverse / library / folder** on the END side. Fixed — commit
  `b934d91d`: `bookmarkLocation()` (longest-prefix lib + `childUniverseLibPaths`
  federation lookup + relative folders); removed dead `.s-name/.s-meta/.s-lib-name`.
- **Test 2** — sidebar bookmark rows had no right-click menu (browser default
  showed). Fixed — same commit: `handleBookmarkContextMenu` → shared note menu
  (Open / Open-in-new-tab / Reveal / **Remove bookmark** / Add-to-collection ▸ / Copy).
Rebuilt frontend + release binary for Stage-1 re-test.

**Re-test round 2** (commit `b934d91d` binary): Re-test 1 (location breadcrumb) +
Re-test 2 (right-click menu) PASS. Two RTL/tooltip refinements → commit `b52f2d3f→next`:
- Bookmark row now takes its **own title's** direction (`dir={detectDir(title)}`):
  RTL title → fully-RTL row (⭐+name at reading start, location at reading end);
  LTR title → LTR row — independent of app language (Language-First).
- Dropped the native `title` tooltip (it overlapped the right-click menu).

### Side task (Boss-routed) — Sight-v6 test — commit `eb3d34b0`
`tests/sight-v6/tradition-isolation.test.ts` stale assertion: `time-dome` is a
THIRD intentional identity remap (MIG-037 P1) omitted from `identityIds`. Added
it (renderer correct). Full suite now **224/224**.

### §7 — State chips — commit `596adf97`
- Renamed `workbenchChips.ts`→`collectionChips.ts` (+ pinned test import). Four
  toggle chips (due/unlinked/contested/forming) NARROW the shown members via
  `filterByChips` over the SAME hydration read — **zero IPC** (hydration $effect
  depends on keySig/activeId only, never chips). Folder/search drop under any chip.
- **Verify:** svelte-check 0 errors; pinned subset test + collections → 22/22.

### §8 — Liveness — commit `19d33d17`
- CollectionsPanel debounced (500ms) re-hydrate on the existing mutation events
  (note-created / cascade:rewrote / cache-reconciled / screen:note-saved);
  listeners + timer cleaned on destroy (no leak). Display-only; membership never
  changes here. **Verify:** svelte-check 0 errors.

### §9b — Retire Workbench flag + legacy bookmarks store — commit `0a29f01c`
- +layout: removed the always-false `showWorkbench` + every ref (fullPageActive,
  2 guards, the guard $effect, Esc branch, ~15 reset-chain no-ops) + `.workbench-overlay`
  CSS. store.ts: removed `enabledFeatures.workbench` flag + the vestigial `bookmarks`
  store. SettingsModal: removed the dead "Workbench" toggle. Rust: removed
  `save_universe_bookmarks` (read kept for boot bundle + migration).
- **Verify:** svelte-check 0 errors (warnings **326→324**); cargo check clean;
  content-integrity + collections + chips → **50/50**.
- *(Deferred to §10: `workbench.*` i18n removal, batched with the `collections.*`
  15-locale additions.)*

### → STOP for Boss test (Stage 2)
Stage-2 binary building (frontend rebuilt; `cp-chip`/"This collection is empty"
confirmed in `build/_app/immutable/`). Stage-2 tutorial: multi-collection switch /
create / rename / delete; per-member done + Clear-done + remove; open-from-collection;
the four state chips; live auto-refresh on an edit; and a Settings check (no
"Workbench" toggle).

### Remaining
§10 close-out: help topic + User Manual ×15 + full 15-locale `collections.*`/context-menu
i18n (− `workbench.*`) + Orientation v-bump (same commit) + `/simplify` + boot re-measure
on the 7,600-note universe + audit trio (invariants / drift / migration-path).
