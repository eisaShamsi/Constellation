# MIG-091 — Empower the File Explorer (Architect + Plan)

**Date:** 2026-07-05 · **Phases 1–2 of `/migration`** · **Boss-approved direction (2026-07-05):** the resolution of the MIG-090 Navigator question — *don't build a separate surface; give the file tree the file-management muscle the old two-pane Navigator had, and let it own the whole file-system job.*

**Concept (the horse):** *organize my files in bulk, and find my way through a large tree fast — without ever leaving the file system.* Everything here is file-system-bounded: names, folders, move/delete/tag. No content, no meaning, no cognition (those stay with Search Hub, the Tags panel, and Bases). This is the Note Navigator's keepable half, moved to its rightful home.

---

## The File Explorer today (`FileTree.svelte`, 252 lines — verified)

Already strong: recursive tree (native `<details>` folders + `<button>` notes); **inline rename**; the full shared **right-click** menu (open, move, tag, copy, delete, new note/folder/base, reveal, style); stage-emoji + maturity-border decorations; **live refresh on mutation**; sort cycle (name-asc/desc, modified-desc/asc). Lazy per-library load (`read_library_tree`, maxDepth on expand).

What it lacks — exactly the Navigator's keepable muscle:
- **No multi-select, no batch operations** — one file at a time only.
- **No filter box** to narrow a large library by name as you type.
- **Sort is name/modified only** — no created, no size. *(Verified: `FileEntry` carries `modified` but not `created`/`size`.)*
- **Not virtualized.**

## Predecessor → Replacement (Predecessor Lookup Rule)

- **Old two-pane `NotebookNavigator`** (`NotebookNavigator.svelte` + `NavBrowserPane`/`NavFileList`/`NavFileItem`/`NavBatchBar`, 1,033 lines; sidebar `list` mode, flag `enabledFeatures.notesNavigator`) → its keepable capabilities (multi-select, batch move/delete/tag, name filter, richer sort) **move into `FileTree.svelte` + the tree host in `+layout`**. Its facets that duplicated other organs (tags→Tags panel, properties→Search Hub) do **not** move. The old component + its flag + `collect_library_notes_with_metadata` + `search_by_property` **retire at the end**, once the empowered tree is Boss-validated — add-first, remove-after-proof.

## The four phases (each = one commit + verification clause)

### Phase A — Filter box + richer sort *(pure-additive; no write paths)*
- **Sort:** extend the existing `cycleSortOrder` set with **created** and **size**. Dependency: add `created: Option<u64>` + `size: Option<u64>` to the Rust `FileEntry` + `read_library_tree` (`fs::metadata` is already read for `modified` — `.created()` and `.len()` are free there) and to the TS interface; `sortEntries` gains the two cases. Small, additive, no behavior change until selected.
- **Filter:** a type-to-filter field atop the tree that narrows to entries whose **name** (or `display_title`) matches, **keeping ancestor folders** of any match so the hits stay reachable. Name-only — never content (that line stays Search Hub's). Client-side over the already-loaded tree; instant.
- **Verify:** svelte-check 0; sort by created/size orders correctly; filtering a big library narrows to matches with their folders, clears cleanly. **Boss test (Phase A).**

### Phase B — Multi-select *(interaction only; still no writes)*
- A selection `Set<path>` (host-owned, passed into the recursive `FileTree`). **Ctrl/⌘-click** toggles a row; **Shift-click** selects the range since the anchor (over the flattened visible order); plain click keeps today's open-note behavior. Selected rows get a check/highlight; a live count shows. Covers files **and** folders. Escape / click-empty clears.
- **Verify:** svelte-check 0; multi-select + range-select behave; plain click still opens a note; no write path touched yet.

### Phase C — The batch bar *(the crown jewel; touches write paths → full care)*
- With ≥1 selected, a bar offers **Move · Delete · Add tag**, each **looping the existing gated single-item handlers** over the selection — never a hand-rolled write:
  - **Move** → `moveItem(path, targetFolder)` per item (one folder picker for the whole selection).
  - **Delete** → `deleteWithSetting(path)` per item (trash-backed) behind one styled confirm with a plural-aware count.
  - **Add tag** → `addTagToNote(path, tag)` per item (the **same gated property-write the single-note Add-tag dialog uses** — `+layout:8279`), one tag prompt for the whole selection. **No YAML string-splice** — that corruption class (old NavBatchBar) stays dead.
- WA#4: a batch is N sequential gated writes; each already carries its own gate + tree-refresh. The batch wraps them with one confirmation, one progress pass, and one final refresh; selection clears on completion. Because **Add tag touches note content**, Phase C runs the content-integrity harness (`tests/mig-076/`) + the relevant Editor-Surface Gate items before the Boss test.
- **Verify:** svelte-check 0; harness green; **Boss test (Phase C)** — select several disposable notes, batch-tag / batch-move / batch-delete, confirm each lands and the tree refreshes; batch-tag an *open* note and confirm no corruption.

### Phase D — Virtualize the tree *(perf; REPRODUCE-FIRST — likely deferred)*
- The tree renders all loaded nodes in the DOM (native `<details>` only hides visually). Virtualizing a **nested** tree means flattening visible nodes + replacing native `<details>` with JS-controlled windowing over `VirtualList` — significant rework.
- **Gate:** MEASURE first on the 7,600-note universe with a realistic expansion. Build only if the tree is actually slow; otherwise **defer** to its own follow-up (no speculative rework). No felt tree-slowness has been reported.

## Invariants & risks
- **Write-path safety (Phase C):** batch = only the proven gated handlers, looped — no new write path, no YAML splice; the harness gates the content-touching verb.
- **Boot budget:** the tree is on the paint path — Phases A/B add only client-side interaction/filter; Phase A's Rust field add is two `metadata` reads already performed (zero new IO). No boot regression.
- **Federation:** the tree already shows cUniverse libraries read-only; batch **write** verbs must be **disabled on read-only cUniverse members** (the federated-write blocker, `load_libraries` non-recursive scoping). Confirm in Phase C.
- **i18n ×15:** new strings — filter placeholder, created/size sort tooltips, batch bar labels + confirm — localized; reuse existing slugs where they exist (move/delete/addTag already localized).
- **RTL:** filter input `dir=auto`; selection/batch-bar use logical properties.

## Process
`/migration`: this doc (Architect+Plan) → Boss approval → cascade with Boss tests at **Phase A**, **Phase C**, and completion. `/simplify` on the final diff; then the 3-agent audit (invariants · drift · migration path). Old Navigator retired + docs/manual/orientation close-out in the final commit.

## Cost
~4–5 commits (A, B, C, retire+close-out; D gated on measurement). Phase C is the care-point (write paths + harness). No content writes before Phase C; flag-free (the tree is always on).
