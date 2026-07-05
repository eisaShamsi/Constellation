# MIG-092 — Architect: Collections on the Search Hub (unifying Bookmarks)

**Date:** 2026-07-05 · **Phase:** 1 of 4 (Architect) · **Branch:** `main`
**Lineage:** the "hold" half of the ratified [MIG-090 v2 Navigator horse](concept-papers/MIG-090-Notes-Navigator-v2-Concept-Paper.md). The Boss rejected the Workbench as "another search engine" (the *translate* half — the Intent Bar — duplicated the Search Hub). MIG-092 keeps the *hold* half and folds it into the surface that already owns search.
**Inputs:** understand-workflow `wf_c42d7e3f-89a` (5 parallel readers + 2 adversarial verifiers) · first-hand reads of `workbench.rs`, `universe.rs`, `store.ts`, `SearchHub.svelte` · WA#5 cross-check (Zotero Collections/Saved-Searches split; Obsidian Bookmarks unification).

---

## 1. The concept — the horse

> **A Collection is the working set of notes I'm forming a task around — hand-picked from what a search surfaces, named, and kept alive across sessions — so the notes I judged worth keeping outlive the query that found them and stay actionable.**

Each clause is load-bearing: **hand-picked** (not a query → not Bases) · **from search** (attaches to the Search Hub's *existing* engine → no rival search — this resolves the rejection) · **named + task-scoped** (many baskets, not one shelf) · **kept alive across sessions** (the held set the gap-analysis proved nothing owns) · **outlive the query** (search results evaporate; my judgement about them shouldn't).

The gap this fills (MIG-090 concept paper §2): *"the notes about X I was forming last week — the thread I dropped — what I'm working WITH right now — what needs my hand next."* Search demands a formulated query; Bases demand configuration; Reviewer decides by its own schedule; Tabs hold windows not intent; Recents are time-ordered noise. None hold *my* working set.

## 2. Whole-entity fit (the zero-duplication law)

| vs. | Their question | Collections' question | Verdict |
|---|---|---|---|
| **Search Hub** | Find any note by query (transient). | Keep the ones I picked (persistent). | ✅ gap-defined — Collections is the *persistence of judgement about* results |
| **Bases** | Auto-populate a table from criteria (dynamic). | Hand-pick static membership. | ✅ gap-defined — the Zotero *Collections vs Saved-Searches* split |
| **File Explorer** | Where notes live (hierarchy, transient multi-select). | Cross-cutting named sets (persistent). | ✅ gap-defined — a note lives in one folder, rides many collections |
| **Bookmarks** | One flat shelf of quick-access shortcuts. | Named working-sets of notes with work-state. | **UNIFIED** (Boss ruling 2026-07-05) |

**The Bookmarks finding + ruling.** The adversarial dedup pass caught an existing **Bookmarks** feature ([store.ts:926](../src/lib/libraries/store.ts:926); sidebar [+layout:6468](../src/routes/+layout.svelte:6468)) — a flat list that hand-picks `{note|folder|search}` into one shelf, storing name/path inline (no groups, no `done`, no live hydration, no cid-keying). It was in *neither* the Boss's dedup list *nor* the v2 concept paper's whole-entity map. Per the zero-duplication law it could not be designed past silently.

**Boss ruling: UNIFY.** Collections becomes THE hand-picked mechanism; the former Bookmarks folds in as a pinned **Starred** collection with two mounts — the Search Hub *Collections* tab (manage all collections) and the sidebar shortcut (quick nav to favorites). This *reduces* the entity's surface count instead of adding a tenth copy — the concept-paper §3 ideal ("the Navigator work reduces the entity's duplication instead of adding a tenth copy"). Industry precedent: Obsidian folded starred-files into Bookmarks; Zotero keeps hand-picked Collections distinct from dynamic Saved-Searches (= our Bases).

## 3. Chosen forms (Boss decisions 2026-07-05)

1. **Bookmarks → UNIFY** into Collections (Starred = the pinned favorites collection). Bigger scope (touches a shipped feature) but the cleaner single-mechanism entity.
2. **Layout → header tab** inside the full-page Search Hub: a tab strip under the input toggles the body **Results | Collections**. "Add to Collection" lives on result rows + the context menu; the Collections tab is view/manage.
3. **Naming → clean rename** `workbench*`→`collections*` (module, commands, JSON, store). Safe because the Workbench flag shipped **off** — no real universe has basket data. Legacy `workbench.json` is migrated-then-retained.

## 4. Heterogeneous membership (the unify complication)

`workbench_hydrate` reads `note_meta` — notes only. Bookmarks hold folders + saved searches too. Resolution: the item model gains `type?: 'note'|'folder'|'search'` (default `note`).
- **Note** members: membership-only (`cid`/`path`), **live-hydrated**, rename-proof (cid self-upgrade). The rich path.
- **Folder / search** members: keep today's inline `{name, path, libraryName}` behavior, **not** hydrated (no `note_meta` row). Rendered from stored facts, same as Bookmarks render them today.

No capability is dropped; the note case simply *gains* the rich machinery. `collections_hydrate` is called only for note members.

## 5. Predecessor Lookup (before any edit)

**DELETE** — standalone Workbench desk (`WorkbenchView.svelte`; overlay [+layout:7045](../src/routes/+layout.svelte:7045); dock button `:6210`; palette entries `:2253`; context-menu actions `:5370`/`:5405`) · the **Intent Bar** (no replacement — the Search Hub's search is the front door) · the `enabledFeatures.workbench` flag ([store.ts:3992](../src/lib/libraries/store.ts:3992), default `:4320`) + its Settings toggle.

**KEEP + repurpose in place** — `workbench.json` r/w ([universe.rs:1407](../src-tauri/src/universe.rs:1407)) → `collections.json` · `workbench_hydrate` ([workbench.rs:152](../src-tauri/src/workbench.rs:152), takes plain `cids[]`/`paths[]`, **zero search coupling** — verified) → `collections_hydrate` · `workbenchSets` store + mutations ([store.ts:972–1100](../src/lib/libraries/store.ts:972); `WorkbenchSet{id,name,…}` is **already multi-set**) · `NoteRow`/`NoteList` (the shared VirtualList primitive) · `workbenchChips.ts` (due/unlinked/contested/forming).

**RE-POINT (Bookmarks predecessors)** — `bookmarks` store + `add/remove/isBookmarked/save/loadBookmarks` ([store.ts:926–961](../src/lib/libraries/store.ts:926)) → Starred collection ops · `save/read_universe_bookmarks` (read kept for one-time migration; write retired) · sidebar Bookmarks section (`:6468`) → renders Starred · ⭐ `toggle-bookmark` cmd (`:2221`), `handleToggleBookmark` (`:4608`), `toggleBookmarkPath` (`:5305`), context-menu bookmark actions (`:5348/5355/5368/5403`), bundle `bookmarks` load (`:2414`) → Starred add/remove + migrate-on-load.

## 6. Invariants that must not break

- **No content writes** — Collections is membership-only; adding a note never touches its file → the Editor-Surface Gate is not *triggered*, but the content-integrity harness runs after §6 and §9 (they touch `+layout` wiring) as a belt-and-suspenders.
- **ONE-universe** — hydrate keys by `cid`/`path` across all libraries incl. federated child-universes; federated read-only note members are holdable + openable, mutations skip (the batch-bar precedent).
- **`cid_cn != ''` sentinel** — already guarded in `hydrate_sql`.
- **Boot untouched** — collections + Starred load post-paint, never on the boot path.
- **Write-Time Derivation (Rule 8)** — Collection membership is *user-authored*, not a derived view: no trigger, no `scan_*`; hydration is a cheap O(set) indexed lookup.
- **Perf** — zero `invoke()` on the keystroke hot path (search already debounced 300 ms; add-to-collection is a click); member list virtualized via `NoteList`.
- **Migration safety (NEW)** — bookmarks→Starred migration is **idempotent, resumable, reversible** (legacy `bookmarks.json` retained as backup); mixed members never break note-only hydration; the sidebar re-point never regresses folder/search opening.

## 7. Risk & effort

~10 commits, one focused session, **mostly repurposing tested machinery**. Risk a notch above the coexist alternative because it touches the shipped Bookmarks feature — mitigated by the retained-backup migration, harness runs at §6/§9, and the §10 audit trio (first-boot bookmarks→Starred, mid-migration interrupt, rollback). Risk concentrated in §2 (migration) and §4 (Search Hub layout + RTL). Stops for the Boss test at **§6**.
