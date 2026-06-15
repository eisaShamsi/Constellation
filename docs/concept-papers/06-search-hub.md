# 06 — Search Hub (Concept Paper)

> Function #6, **Phase 2 (Search + Index)** in [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3. Attaches downstream of the Editor (the gate): it reads what the Editor wrote into the FTS5 index and routes the user back into a note for editing.

## 1. Function in hand
The **Search Hub** — `src/lib/components/SearchHub.svelte` — the full-page universal search surface (the magnifier dock button + `searchhub-overlay` in `src/routes/+layout.svelte`). Universal mode categorizes hits (titles / contents / tags / properties / wikilinks / semantic); advanced mode parses FTS5-style operators (`links to [[`, `mutual [[`, `mentions [[`, `orphans`, the typed-link operators `supports`/`contradicts`/`causes`/…, `#tag`, `key=value`, `in:scope`).

## 2. Purpose
Find any note in the Universe by what it *says*, what it's *about*, and how it *connects* — then open it. It is the **diagnostic instrument** of the Five Acts: primarily **Connection** (the operator syntax surfaces typed links — what supports/contradicts/causes what — and the `via {lemma}` cross-lingual badge surfaces hidden bridges), with **Observation** as the floor (locate the note at all). Justified: a knowledge base you can't interrogate is a write-only drawer; Search is how stored knowledge re-enters the user's thinking. Not a file finder — a query language over the cognitive vocabulary.

## 3. What it is NOT
- **Not** the index/term-browser (that is the Index panel, MIG-010/011/012 — a different surface).
- **Not** a writer — it never mutates a note; it reads the index and emits an open request.
- **Not** the owner of the index — it does not build or refresh FTS5; the Editor's save-time reindex does (Rule 8). Search is a pure reader.
- **Not** the Quick Switcher (title-only jump) and **not** Sky View / Map (those are graph surfaces).

## 4. Wiring
- **Inputs (props/stores):** `initialQuery`, `allNotes`, `linkCounts` (props from `+layout`); `appSettings` (reads `enabledFeatures?.semanticSearch` to decide whether to embed); search history (`readSearchHistory`); NSC summary headlines (`getSummariesFor`, cache-first/batched — zero per-row IPC).
- **Inputs (IPC):** `constellation_search_universal` (universal mode), `constellation_search` (advanced/structured mode), `embedText` (only if semantic enabled). All **read-only** queries against the persisted index.
- **Outputs (events/callbacks):** `onNoteClick` → `openNoteTab` in `+layout` (opens the note with highlight terms); `onResults(Set<noteId>)` → graph-view highlighting; `onClose`. It writes nothing to disk; it appends to local search **history** only.
- **Consumers:** the Editor/tab system (receives the open request), Sky View highlight (`searchHubMatchIds`).
- **Connection to the Editor (the gate):** strictly downstream. Search reads the FTS5 index that the **Editor's save → `constellation_search_reindex`** keeps current, and its only write-path action is to hand a `path` back to `openNoteTab` — i.e. it returns the user to the Editor. It never reaches around the Editor to touch a file.

## 5. Right-click / context menu
- **None.** Grep of `SearchHub.svelte` and its mount overlay for `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` returns **zero matches**. Result rows are plain `<button>`s; the only actions are left-click (open), Enter (open selected), arrow-key navigation, and the syntax-chip/history buttons.
- **Gap — flag.** A result row arguably should offer the conventional shared `<ContextMenu>` (MIG-077) actions a note row gets elsewhere: *Open in new tab / Open in split / Open in second screen / Copy wikilink / Reveal in file tree*. Today those are unreachable from a search result; shift/alt-click open-in-surface (available in the Editor) is also not wired here. **Bring-up action:** add the **shared** `<ContextMenu>` (not a hand-rolled menu) to result rows, mirroring the file-tree row menu. Enumerate exact items during bring-up; do not assume them (BASIC RULE).

## 6. Multilingual (by default)
- **Strong.** Root is `dir={$dir}`; the input is `dir="auto"`; every result name, snippet, `match_via` badge, history entry, and advanced-group header sets direction via `detectDir()` or `dir="auto"`, so mixed Arabic/Hebrew/CJK rows render correctly regardless of UI direction. RTL chevron flip is handled (`:global([dir="rtl"]) .sh-chevron`).
- **Localized operators:** advanced syntax is matched in the current locale via `getSearchOps()` + `canonicalizeSearchQuery` / `hasAdvancedSyntaxMultilingual` — the user types operators in their own language. Category/badge/chip strings flow through `$t()` (`searchHub.*`, `searchBadges.*`, `sidebar.*`), and those keys exist in **all 15 locale files** (ar de en es fa fr he hi ja ko pt ru tr ur zh — verified present).
- **One minor seam to verify in bring-up:** the result-`title` tooltips (e.g. `title="Incoming"` / `title="Outgoing"` on the link-direction arrows) and the `0̶` structured-badge glyph are **not** `$t()`-routed. Confirm whether those are user-visible English; if so, localize. (Low severity — verify in bring-up.)

## 7. Boot behavior
- **Runs at boot?** The component does **not** run a query at boot — it mounts hidden and queries only on user input (≥300 ms debounce). At ~800 ms after paint, `+layout` fires `cache_mark_search_ready` (a readiness flag, not a walk).
- **Rule 8 status: ✅ reads-persisted.** Search hits the FTS5 / `note_meta` index kept current write-time by the Editor's reindex and the file watcher. `constellation_search`'s own comment records that the old behavior — a `constellation_search_init` walk of *every library on every first search* — was **removed**; the cold path now only opens the DB connection (cheap) and returns empty if the index is cold. No `scan_*` / `rebuild_*` on read. This is the canonical Rule-8-compliant reader.
- **Cost:** per-query ~5–50 ms (E, charter) — three FTS5 `MATCH` queries (universal mode) plus optional embedding similarity if semantic is on; rows capped at 200 painted (MIG-071 audit). `cache_mark_search_ready` ~800 ms deferred (E). No boot-time cost attributable to the Hub itself.

## 8. Flag / gate & bring-up position
- **Gate today: effectively NONE.** The MASTER charter lists the intended gate as `enabledFeatures.search`, but grep finds **no `enabledFeatures?.search` in code** — the dock button (`+layout` ~`:5187`) is **unconditional**, unlike the gated OrgChart/SkyView/Index/CCS buttons. Only the *semantic embedding layer inside* search is flag-gated (`enabledFeatures?.semanticSearch`). **Bring-up action:** add the `enabledFeatures.search` (or `safeBootMode`-aware) gate the charter assumes, so minimal mode can flip Search off.
- **Bring-up phase: 2 (Search + Index).** Depends on: a populated FTS5 index (Phase 1 Editor save-path reindex) + `cache_mark_search_ready`. Cannot return useful results before the index is warm.

## 9. Budget
- **Boot budget:** zero added boot work beyond the deferred `cache_mark_search_ready` (~800 ms, off the paint path). Must not regress `paint_ms` / `hydrated_ms`.
- **Interaction budget:** keystrokes instant — input handler does no `invoke()`; queries are debounced ≥300 ms and the previous timeout is cancelled (Rule 3 / IPC contract). Result paint capped at 200 rows. Target ≤50 ms perceived per query on a 7,600-note Universe.
- **Regression guard:** type a query burst (no input lag); run an advanced operator query (`mutual [[X`) and a semantic query on a large Universe — measure query latency before/after any change to `search.rs` or the store query path; verify no per-row IPC creeps into the result loop.

## 10. Acceptance checklist (the gate to "re-enabled")
- [ ] **Serves its purpose:** universal + advanced + semantic queries return correct hits; operators parse in a non-English locale; clicking a result opens that exact note with highlight terms.
- [ ] **Serves Constellation's core purpose:** the typed-link operators surface **Connection** (supports/contradicts/causes…); cross-lingual `via {lemma}` bridges work (see [00-Constellation](00-Constellation-Core-Concept-Paper.md)).
- [ ] **Wires correctly to the Editor:** open from a result lands in a tab; no path reaches around the Editor to mutate a file; `onResults` highlights the graph.
- [ ] **Right-click present + correct:** result rows expose the **shared** `<ContextMenu>` (MIG-077), not a hand-rolled menu, with at least open-in-new-tab / open-in-split / copy-wikilink / reveal-in-tree — **currently absent (§5 gap).**
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** all strings `$t()` in 15 locales; `dir`/`detectDir` correct for mixed scripts; the arrow `title=` tooltips localized (§6 seam closed).
- [ ] **Within budget:** no `invoke()` on the input path; ≥300 ms debounce with cancel; ≤200 rows painted; large-Universe query latency measured.
- [ ] **Obeys Rule 8:** reads the persisted FTS5 index only; no `scan_*`/`rebuild_*` on read; verified no boot-walk regression.
- [ ] **Holds its invariants:** read-only (never writes a note); history is local-only; advanced/universal mode selection is deterministic from the query.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (per-query cost estimated; not yet measured on a large Universe)**
Notes: Rule 8 is the clean case here — Search is a pure reader of the Editor-maintained FTS5 index, and the old boot-walk was already removed. Two real debts to close before re-enable: (1) **no shared context menu on result rows** (§5), and (2) **the `enabledFeatures.search` gate the charter assumes does not exist in code — the dock button is currently unconditional** (§8). One minor i18n seam (arrow `title=` tooltips) to verify. Per-query latency on a 7,600-note Universe is unmeasured — verify in bring-up.
