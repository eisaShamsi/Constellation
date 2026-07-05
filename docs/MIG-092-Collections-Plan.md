# MIG-092 — Plan: Collections on the Search Hub (unifying Bookmarks)

**Date:** 2026-07-05 · **Phase:** 2 of 4 (Plan) · **Concept:** [Architect](MIG-092-Collections-Architect.md) — the horse §1, forms §3, invariants §6.
**Approval:** Boss "go" 2026-07-05 (both design forks ruled: unify Bookmarks; header-tab layout). Plan-Approval = Build-Approval → cascade autonomously, stop at **§6** (Boss test), §9 harness, and completion.

Each step = one commit + a verification clause. No note content is written anywhere in this migration.

---

1. **§1 Persistence + item model** — `workbench.json`→`collections.json`; `read/save_universe_workbench`→`read/save_universe_collections` (universe.rs); `workbench.rs`→`collections.rs` (module + `workbench_hydrate`→`collections_hydrate`, struct `WorkbenchRow`→`CollectionRow`); item model gains `type?: 'note'|'folder'|'search'` + inline `name/library_name` for non-note types; ensure-on-init writes `collections.json`; legacy `workbench.json` (if present) **migrate-then-retain** (`.migrated` suffix). Re-register commands in `lib.rs`.
   *Verify: cargo build + cargo test (round-trip incl. a folder-type + search-type member; missing file → empty; legacy file migrates once).*

2. **§2 Store: multi-set + Starred + Bookmarks migration** — rename `workbenchSets`→`collectionSets`, `WorkbenchItem`→`CollectionItem` (+`type`), `WorkbenchSet`→`Collection`; set-targeted mutations `createCollection(name)`/`renameCollection`/`deleteCollection`/`addToCollection(setId,item)`/`removeFromCollection(setId,key)`/`toggleDone`/`sweepDone`; keep `adoptIdentities`/`migratePath`. A pinned, **undeletable `starred`** collection + `addToStarred`/`removeFromStarred`/`isInStarred`. **One-time migration**: on load, if `bookmarks.json` has items and no `starred` collection exists yet, create `starred` and map each bookmark→`CollectionItem` (preserve `type`/`path`/`name`/`libraryName`; notes adopt `cid` on first hydration), then persist `collections.json`. Idempotent (guarded by the starred-exists check).
   *Verify: svelte-check; unit test — migrate {note, folder, search} → starred with 3 typed members; starred cannot be deleted; add/remove/rename/delete across two collections; re-run migration is a no-op.*

3. **§3 Mixed-member hydration** — frontend hydrate call passes only note-typed members' `cid`/`path` to `collections_hydrate`; folder/search members render from inline stored facts (no hydrate call, no "missing" diff). Rust `collections_hydrate` unchanged from the note-only read (already correct).
   *Verify: cargo test seeded note+folder+search keys — note gets live facts; a manual frontend trace confirms folder/search skip hydrate and render inline.*

4. **§4 Search Hub tab shell** — a tab strip under the input in `SearchHub.svelte` (Results | Collections), `dir`-aware + RTL, keyboard-reachable; the Collections tab renders the collection list + the active collection's members via `NoteList`/`NoteRow` (note rows hydrated, folder/search inline), hydrated on tab-open + liveness. Results tab unchanged.
   *Verify: svelte-check; tab toggles both ways; empty-state renders; a 100-member collection scrolls at budget; RTL mirrors the tab strip.*

5. **§5 Pick-up + management** — per-result "Add to Collection" affordance (hover icon on `.sh-item` + re-point the shared context-menu slot from `addToWorkbench` to an `addToCollection` set-picker submenu incl. "New collection…"); create/rename/delete/switch active collection in the Collections tab header; per-member remove / done / sweep-done; the ⭐ affordance quick-adds to Starred.
   *Verify: svelte-check; add from result row + context menu into a chosen collection; dedupe by path/cid; cap 100 respected; new-collection inline create works.*

6. **§6 Sidebar re-point** — the sidebar "Bookmarks" section renders the **Starred** collection (label key `sidebar.bookmarks` retained for continuity); the ⭐ `toggle-bookmark` command + `handleToggleBookmark` + `toggleBookmarkPath` + all context-menu bookmark actions call Starred add/remove; `isBookmarked`→`isInStarred`; bundle `bookmarks` load path folds into the migration.
   ***Boss test (tutorial):*** star a note (sidebar updates instantly) → open Search Hub → Collections tab → create "Task A" → search, add 3 results → create "Task B", add 2 → close Search Hub → restart the app → Starred + Task A + Task B all intact, each note-row shows live facts, and a folder that was bookmarked still opens from the sidebar. *(content-integrity harness `tests/mig-076/` runs here — §6 touches `+layout` wiring.)*

7. **§7 State chips** — reuse `workbenchChips` (`chips.ts`; due/unlinked/contested/forming) to narrow the active collection (note members only; client-side AND-intersection; zero IPC).
   *Verify: chip toggle performs ZERO `invoke()` (traced) and only narrows the shown set — reuse the pinned `tests/mig-090/chips.test.ts` (subset invariant).*

8. **§8 Liveness** — one debounced re-hydrate on the existing mutation events (`note-created`, `cascade:rewrote`, `cache-reconciled`, `screen:note-saved`); unlisten on destroy.
   *Verify: 10 s of typing elsewhere → ≤1 refresh after settle; a rename cascade → exactly one refresh, membership intact, no leaked listener.*

9. **§9 Retire Workbench + legacy write** — delete `WorkbenchView.svelte`, its overlay mount, dock button, palette entries, the Intent Bar, the `enabledFeatures.workbench` flag + Settings toggle; retire the `save_universe_bookmarks` write (read kept for migration only).
   *Verify: svelte-check; `grep -ri workbench src/` is clean except the retained `collections`-named files + `tests/mig-090`; content-integrity harness green; cargo build + npm run build.*

10. **§10 Close-out** — help topic + User Manual ×15 (define Collections + Starred = the former Bookmarks + the boundary story); session log + Orientation v-bump **same commit**; `/simplify` on the full diff; boot + typing re-measured on the 7,600-note universe vs baseline; **audit trio** in parallel (invariants / drift / migration-path: first-boot bookmarks→Starred, mid-migration interrupt, rollback via retained backup); staged Boss tutorial.

## Cost & risk
~10 commits, one focused session. Mostly repurposing tested machinery (hydrate/store/primitive verified reusable; store already multi-set). Risk concentrated in §2 (the Bookmarks migration — mitigated by idempotency + retained backup) and §4 (Search Hub layout + RTL). No content writes; no boot-path changes; harness at §6 and §9; audit trio at §10.
