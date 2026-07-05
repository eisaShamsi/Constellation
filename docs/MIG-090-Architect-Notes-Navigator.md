# MIG-090 — Architect: the Notes Navigator Restructure

> **DIRECTION SUPERSEDED (2026-07-05):** the Boss revoked Option B after §2 shipped — a table of notes is what *Bases* are for; the Navigator must be rewritten from its ORIGINAL two-pane paradigm, "smart, fast, launching from where others ended" (research-first, `wf_50227623-e2e`). §1 kept; §2 reverted (`d03e2fd0`). **The diagnosis in this document (§§1–5) remains valid and feeds the new concept** — only §6's options and the recommendation are superseded.

**Date:** 2026-07-05 · **Phase:** 1 of 4 (Architect) · **Trigger (Boss, verbatim):** *"I want to restructure the 'Note Navigator', either to merge it with the 'File Explorer', or redesign it to match Constellation philosophy. Today, it is slow and dumb, and it feels like a foreign body within the Constellation ecosystem."*
**Discovery:** workflow `wf_971f71e2-913` (4 mappers + adversarial verifier — all findings upheld; 3 precision corrections applied below). Every claim cites file:line.

---

## 1. The concept verdict (the horse that never existed)

**The Navigator has no concept.** The 32-paper bring-up set contains no paper for it — [02-file-tree.md](concept-papers/02-file-tree.md) explicitly carves it out as "a separate function, not this one." Under Concept-Before-Function, it is a carriage with no horse. Its lineage confirms it: the component is named after Obsidian's *Notebook Navigator* plugin — an **Apple Notes / Bear paradigm transplant** that answers the storage question ("what files exist, when were they touched") in a system whose every other surface answers a cognitive question. **That is the precise diagnosis of the Boss's "foreign body" feeling.** Its rows carry name / preview / tags / date / size — none of the four ratified questions (Development · Altitude · Origin · Connection).

## 2. "Slow" — the mechanism, measured in reads

- **Mount = the corpus, twice.** `collect_library_notes_with_metadata` ([libraries.rs:4986](../src-tauri/src/libraries.rs)) reads **every `.md`'s full content** per library for a 200-char preview + tags; canonical-named files are read **twice** (title pass + content pass; the "1KB only" docstring is false). The default `universe_notes` library's path == the Universe root, so its walk **re-reads every nested library's files**, then each library is walked again individually — **~15,000+ full file reads per mount** on the 7,600-note corpus (frontend dedupes the duplicate rows it causes).
- **The walk repeats after every batch action** (`refreshData` ignores the boot-snapshot shortcut). The SS mount never receives `initialTags`, so it always pays the full per-library tag scans as well.
- **Unvirtualized:** "All notes" renders 7,600 rows ≈ **70k+ DOM nodes** in a plain `{#each}` (self-documented "the list is un-virtualized") — Rule-3 violation on a surface whose whole job is listing the corpus.
- Batch-W (`d9f8bd80`) moved all this off the dispatch thread — it no longer *freezes*, it is still *slow by construction*.
- **Rule-8 verdict:** everything the list shows already sits in SQLite (`note_meta`: name/path/library/modified/word_count/tags_json/body_text + `created_at`; boot snapshot proves 7,600 rows read in low-millis). The fs walk re-derives what the index already persists.

## 3. "Dumb" — the defect inventory

1. **Folders mode has NEVER worked** *(static read — Reproduce-First: confirm on the running app before citing as fixed)*: the invoke passes `{ libraryPath }` but the Rust command's parameter is `path` ([NotebookNavigator.svelte:121](../src/lib/components/NotebookNavigator.svelte) vs [libraries.rs:286](../src-tauri/src/libraries.rs)) — the rejection is swallowed by `.catch(() => [])`, every subtree is empty, the folder pane is a flat library list. Every other caller passes `path` correctly.
2. **Fake affordances:** the "created" sort silently falls back to modified (`NoteWithMeta` has no created field); the pane divider shows `cursor: col-resize` with zero drag logic.
3. **SS single-click is a no-op** (mode `'second'` routes clicks to an `onNotePreview` prop the SS never passes); only double-click works.
4. **Batch tag corrupts inline-array YAML** (`tags: [a, b]` + string-splice `- tag` = invalid mixed YAML) via a native English-only `prompt()`, one unattested `write_note` per note — **a second write path around the Editor gate**.
5. **Zero mutation listeners** — no `listen()` anywhere in the component chain; it goes stale on any rename/delete/create elsewhere. This is why the Boss's own 2026-06-29 ruling parked its right-click: "a separate data domain that doesn't refresh on delete/move/rename — a data hazard… until the Navigator is reworked into a display over shared data" (+layout.svelte:6186-6190). **Task `task_fcc8396c` (re-enable RC) is contingent on this migration by that ruling.**
6. **A dead fossil:** `src/lib/navigator/data.svelte.ts` (253 lines, "Shared data layer for the Unified Navigator") has **zero importers** — an earlier unification attempt that was never wired. Cautionary shape: unification must land as a wired swap, not a parallel layer awaiting adoption.
7. i18n/RTL drift: hardcoded EN (`{n} notes` count, sort tooltips, property placeholders, the prompt), chevrons never flip in RTL.

## 4. The overlap matrix (what already exists, better)

| Navigator capability | Better sibling | Evidence |
|---|---|---|
| Flat all-notes list + metadata | **Base table** (BaseTab.svelte, `src/lib/lens/`) | SQL over `note_meta` via `execute_lens` (federated UNION across cUniverse schemas — Rule-8 clean, zero walk); **virtualized**; Boss-validated at 7,684 rows |
| In-list search | **Base in-memory search** | zero-IPC, precomputed row blobs; Boss-PASSed |
| Letter jump | **Base letter rail** (nothing else has it) | multi-script, Boss-validated |
| Folder browsing | **File Explorer** | lazy per-library tree, refreshed on mutation, rename-in-place |
| Tag browsing | **TagsPanel** | `tag_counts` summary from SQLite (~ms, write-time maintained); the Navigator's own tag-tree builder is a duplicate implementation |
| Property search | **SearchHub** `properties` category (FTS5) | the Navigator's `search_by_property` is another full-content fs walk |
| Known-item jump | QuickSwitcher | boot-snapshot + FTS5, ms-class |
| Right-click | FileTree/Base menus (shared builder) | Navigator's is parked by Boss ruling |
| **Multi-select batch tag/move/delete** | **NOWHERE ELSE** — the only unique capability | needs a ruling: port or retire |

**Base gaps if it becomes the list:** the lens dimension registry lacks `note.tags` and `note.modified` (both columns exist in `note_meta`); lens scope has no folder/path filter; no multi-select/batch ops; engine returns all rows over IPC (needs engine-side LIMIT eventually).

## 5. Constraints & invariants (any option)

- **Three-site lockstep:** the `SidebarMode` type (secondScreen.ts:229), the main mode tab (+layout.svelte:6151, feature-flagged `enabledFeatures.notesNavigator`), the SS if-chain branch (SecondScreenPage.svelte:1671-1686). No dangling branch may remain.
- **No workspace burden:** `sidebarMode` is never persisted — removing/replacing the mode cannot break restore. The only residue is the 450px width-force dance, which dies with the button.
- **Boot budget:** the Navigator mounts lazily (not on today's boot path). The MERGE option is the boot-sensitive one — File Explorer IS on the paint path; a merged surface must read `note_meta`/boot-snapshot only, never pull a walk toward boot (protected criteria: UI ≤2.5 s / hydrated ≤6 s).
- **PJ-068 interlock:** the SS mount is already ruled REPLICATES ("retire, or redesign into something the main list can't show") but PJ-068 is PARKED ("nothing changes until reopened"). Both windows render the SAME component, so this migration necessarily touches the SS. Two lawful paths: **(a)** bundle PJ-068's Navigator-line ruling into MIG-090 with explicit Boss reopen of that one line; **(b)** freeze the SS branch as-is and dispose of it later with PJ-068. Needs a Boss ruling.
- **Batch ops, if ported:** must ride the shared write paths (the Editor gate / shared handlers), never the current YAML string-splice; batch-tag touches note content → Editor-Surface Gate territory.
- **Folder hierarchy:** MIG-078 Phase B (write-time `tree_node`/`folder_stats`) is planned, not shipped — any folder facet must coordinate with it, not add a second read-time tree source.
- **Deletion inventory (clean):** the 5-component chain (1,033 lines) + dead data layer (253) + `collect_library_notes_with_metadata` (single-consumer IPC + wrapper + `NoteWithMeta` type) can retire together; `scan_library_tags` / `search_by_property` / `read_library_tree` have other consumers and survive. `navigator.*` i18n keys ×15 mostly die; the sidebar-tab keys survive. Predecessor Lookup entries required before any edit.
- **Prior art (WA#5):** the namesake plugin solves this exact problem with *Constellation's own prescribed architecture* (persisted derived metadata + virtualized list, 100k-note scale); Logseq/Tana/Craft all render "all pages" from the database, never a file re-scan. The dominant pattern IS Rule 8.

## 6. The options

### Option A — Merge into the File Explorer (one navigation surface, two presentations)
Tree and list become two views of ONE data domain (note_meta/boot snapshot), sharing the tree's refresh ecosystem and menus. **Effort: HIGH** (touches the paint-path FileTree). **Risk: medium-high** (boot budget; FileTree is Boss-validated and load-bearing). Honest reading: the surfaces answer different questions (structure vs working-set); a literal merge mostly relocates the problem.

### Option B — Rebuild the list ON the Base engine (recommended)
The "list mode" becomes a built-in **All-Notes Base**: SQL over `note_meta` (Rule-8 clean, federated per the ONE-universe ruling), virtualized, searchable, letter-railed, safe-RC — all Boss-validated machinery. Work: add `note.tags` + `note.modified` lens dimensions (columns exist); optional folder/path scope filter (coordinate with MIG-078 B); port multi-select + batch actions onto the Base surface through shared write paths (or retire them — Boss ruling); wire the sidebar list tab to it; retire the SS branch (PJ-068 line); delete the old chain + fossil + single-consumer IPC. **The four questions come free as lens columns** (stage/maturity/links already in the registry) — the list finally answers a Constellation question: *"my notes as a working set with their epistemic standing."* **Effort: MEDIUM. Risk: low-medium** (additive engine dimensions + a swap behind the existing feature flag as rollback).

### Option C — Retire and distribute
Delete the tab; folder browse → FileTree; tags → TagsPanel; property search → SearchHub; batch ops → ruling (port to Base later or drop). **Effort: LOW. Risk: capability loss** (batch ops are genuinely unique); leaves the "working set" question unanswered.

**Recommendation: B, containing C's deletions** — it is the reuse rule applied (secure the winning: the Base stack), Rule 8 applied, the RC ruling satisfied (Base RC exists), and the PJ-068 Navigator line resolved in the same stroke. A one-line concept it would finally own: **"the corpus as a working set — every note, its standing, and what it needs next, one lens away."**

## 7. Open Boss rulings (block the Plan phase)
1. **Option A / B / C** (recommendation: B).
2. **Batch operations:** port multi-select batch tag/move/delete onto the new surface (through shared write paths), or retire them?
3. **The SS branch:** bundle the PJ-068 Navigator-line ruling into MIG-090 (retire the SS Navigator mount now), or freeze it for PJ-068's own reopening?
