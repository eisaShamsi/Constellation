# Constellation — Orientation & Onboarding

**Version 1.0 | 2026-04-26**
**Author of facts: Eisa ALSHAMSI (project owner, designer, IT Boss).**
**Maintainer: Claude (consultant / engineer / SME).**

---

## 0. How to use this document

**This is the first document any new Claude session reads.** It exists so a fresh AI can get to architectural fluency in one read instead of rediscovering the project from `git log` + screenshots over several frustrating turns.

**Maintenance is a Standing Order** (`CLAUDE.md` Standing Order #6). Whenever a fact below changes — a phase ships, a rule is added, a doc-drift item is fixed, a migration closes — update this file in the same commit that lands the change. Bump the version when the structure changes; date-stamp every section that updates.

**This document is grounded.** Every claim cites the authoritative source (file:line, commit hash, or session log section). When two project documents disagree, I name both and don't pick a winner unless code-reading resolves it. When I don't know something, I say so explicitly in §13.

**Hard rule for every reader (human or AI) of this file**: if you find this document contradicts the actual codebase or a more recent session log, **trust the code and the session log first**, then update this file in the same session. Do not let stale orientation rot in place.

---

## 1. What Constellation IS

**Constellation is a Personal Knowledge Formulation desktop application.**

The distinction is fundamental — it is **not** PKM (Personal Knowledge Management):

> Knowledge Management asks: "Where did I put that?"
> Knowledge Formulation asks: "What can I BUILD from what I know?"
> *(`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md:13-17`)*

It is built on **standard Markdown files** (`.md` + YAML frontmatter) on the user's local filesystem, with a portable Universe-config layer above. Local-first, no telemetry, no cloud, no account.

- **Author**: Eisa ALSHAMSI
- **License**: MIT
- **Repository**: `github.com/eisaShamsi/Constellation`
- **Stack**: Tauri v2 (Rust backend) + SvelteKit + Svelte 5 + SQLite (rusqlite, bundled) + ONNX Runtime (ort) + CodeMirror 6 + PIXI v8 + D3 v7
- **Languages supported at launch**: 15 — `ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh`
- **RTL languages first-class**: 4 — Arabic, Hebrew, Persian, Urdu
- **Platforms**: Windows, macOS, Linux desktop. (iOS/Android: Cargo.toml has `cfg(not(any(target_os="ios", target_os="android")))` exclusions for memmap2; mobile is target-aware in build but no shipping mobile app per Concept Paper.)

---

## 2. Universe / Library / Note hierarchy

Constellation has a **five-level knowledge hierarchy** *(per `CLAUDE.md` "Constellation Knowledge Hierarchy" section)*:

```
Universe (root, named by user, contains universe.json)
  └── cUniverse (child universe — federation of libraries)
       └── Library (self-contained knowledge base, like Obsidian vault)
            └── Folder (subdirectory inside a Library)
                 └── Note (single .md file with optional YAML frontmatter)
```

- **Universe** = portable directory. Contains `universe.json`, `libraries.json`, `settings.json`, `bookmarks.json`, `workspaces.json`, `property-types.json`, and `bases/`. Move it to another machine and the entire workspace follows.
- **Library** = first-class citizen with its own color/appearance/tags/links/index. Registered in `libraries.json`. Multiple libraries coexist in one Universe. Constellation reads them in place — never copies.
- **Folder ≠ Library**. Folders are organizational only.
- **Terminology**: use "Library" everywhere, **never** "vault" (except for Obsidian import compatibility).

---

## 3. Architecture (one-page view)

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (SvelteKit / Svelte 5)                                │
│  - +layout.svelte (3873+ lines, 77 $state, 17 $effect)          │
│  - NotePane.svelte (CM6 editor, livePreview, RTL bidi)          │
│  - FocusPane.svelte (plain-text capture, no parser)             │
│  - Sky View (PIXI v8 force graph)                               │
│  - Constellation Map (D3 sunburst)                              │
│  - Inspector 360, Tension, Sight, Lens, Bases, Tasks, Calendar  │
│  - SecondScreenPage (separate Tauri window — display only)      │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC (140+ commands, 31 Rust modules)                     │
│  - perf_trace (LL-021): every dispatch stamped at the boundary  │
│  - 5 plugins: opener / process / updater (no panic-handler-as-  │
│    plugin yet — std::panic::set_hook in run())                  │
├─────────────────────────────────────────────────────────────────┤
│  Backend (Rust, src-tauri/src/)                                 │
│  - libraries.rs (172 KB) — file I/O, link extraction            │
│  - search.rs (224 KB) — SQLite, FTS5, Living Link triggers      │
│  - cache.rs (36 KB) — boot snapshot, alias resolution           │
│  - canonical.rs (54 KB) — YYYYMMDDTHHMMSSZ_KIND_XXXX filenames  │
│  - universe.rs (61 KB) — universe/cUniverse management          │
│  - arabic/ (10 files, ~300 KB) — 5-layer morphological engine   │
│  - lexicon/ (8 files, ~140 KB) — Lexical Bridge / FST           │
│  - strata.rs / maturity.rs / tension.rs / provenance.rs /       │
│    inspector360.rs / map.rs / lens.rs / lenses.rs / review.rs / │
│    trails.rs / canvas.rs                                        │
│  - bases.rs (32 KB) — .base file CRUD                           │
│  - importers.rs (37 KB) — 7 source formats                      │
│  - file_kinds.rs (16 KB) — 12 core kinds + auto-generation      │
│  - watcher.rs (4.5 KB) — notify-rs file watch                   │
│  - boot_bundle.rs, sky_backfill.rs, dataview.rs, tasks.rs       │
│  - embeddings.rs / embeds.rs / fts5_tokenizer.rs                │
│  - ai/mod.rs (12.6 KB) — multi-provider AI                      │
├─────────────────────────────────────────────────────────────────┤
│  Storage                                                         │
│  - .md files on disk (source of truth)                          │
│  - SQLite DB at <universe>/.constellation/search.db              │
│    Tables: note_meta, note_links, note_aliases, sky_nodes,      │
│    sky_links, notes_fts (custom 'constellation' tokenizer),     │
│    schema_versions, plus Living Link state                      │
│  - boot-perf.latest.json — per-boot scorecard                   │
│  - .meta.json sidecars for non-markdown files (canonical)       │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 Key dependencies (versions)

| Layer | Package | Version | Purpose |
|---|---|---|---|
| Rust | `tauri` | 2.x with `protocol-asset` | App runtime |
| Rust | `rusqlite` | 0.31 (bundled) | SQLite |
| Rust | `ort` | 2.0.0-rc.12 | ONNX Runtime (semantic embeddings) |
| Rust | `tokenizers` | 0.20 (with `onig`) | HuggingFace tokenizers |
| Rust | `fst` | 0.4 | BurntSushi FST (Arabic generative index) |
| Rust | `memmap2` | 0.9 (desktop only) | mmap baked Arabic FST |
| Rust | `notify` | 7 | File watcher |
| Rust | `petgraph` | 0.6 | Graph algorithms |
| JS | `svelte` | ^5.0 | UI framework (runes mode) |
| JS | `@sveltejs/kit` | ^2.9 | Routing |
| JS | `@codemirror/*` | 6.x (full set) | Editor |
| JS | `pixi.js` | ^8.17 | Sky View force graph |
| JS | `d3` | ^7.9 | Constellation Map sunburst |
| JS | `@xenova/transformers` | ^2.17 | Frontend ONNX embeddings (where applicable) |
| JS | `katex` | ^0.16 | Math rendering |
| JS | `mermaid` | ^11.12 | Diagram rendering |
| JS | `marked` | ^17 | Markdown rendering (non-CM6 surfaces) |
| JS | `dompurify` | ^3.3 | XSS protection |

Plugins: `tauri-plugin-opener`, `tauri-plugin-process`, `tauri-plugin-updater`.

### 3.2 Versioning state

- `package.json`: `0.3.4`
- `src-tauri/Cargo.toml`: `0.1.0`
- **Drift acknowledged**. Pending bump to `0.4.0` (per `lab/reports/HANDOFF-2026-04-15.md` open items).

### 3.3 Tauri command surface

`src-tauri/src/lib.rs:256-428` registers **140+ commands** across 31 modules. The `invoke_handler` is wrapped in a closure that records each dispatch via `perf_trace::record(invoke.message.command())` — the LL-021 IPC arrival tracer. **Every IPC entry is timestamped at the dispatcher** without per-command instrumentation.

**`docs/IPC-CONTRACT.md` is significantly stale** (last updated 2026-03-31; lists ~50 commands of the 140+). Until refreshed, `lib.rs:256-428` is the authoritative command registry.

---

## 4. The Cognitive Engine (CE)

`docs/CE-spec.md` + `docs/cognitive-engine-roadmap.md` are the canonical specs. The CE is a **two-layer architecture**.

### 4.1 Seven epistemological foundations
*(from `docs/CE-spec.md:22-29`)*

1. Knowledge is not information — value is in connections, not storage.
2. Knowledge has a vertical dimension — 8-level hierarchy (Datum → Worldview).
3. Knowledge has a certainty dimension — `ilm al-yaqin → haqq al-yaqin`.
4. Knowledge is organized by immutable principles — non-contradiction, causality, hierarchy.
5. Knowledge has diverse sources — sensory, rational, transmitted, experimental, intuitive.
6. Knowledge exists on a spectrum — received (from authority) vs discovered (by user).
7. The essence of knowledge is understanding-generative apprehension — enables explain, predict, act.

### 4.2 Layer 1 — Structural Cognition (zero AI)

11 phases. **All shipped** per `docs/cognitive-engine-roadmap.md` and session log §1188 ("🎉 LAYER 1 COMPLETE", 2026-04-03).

| # | Name | Status |
|---|---|---|
| 1 | Typed Links | ✅ shipped (commit `d7edc6d`, 18 tests passed) |
| 2 | Knowledge Strata (8-level hierarchy) | ✅ shipped (`0f6d4bf`) |
| 3 | Maturity Lifecycle (seed/sapling/evergreen/canonical/wilting) | ✅ shipped (`5cf4283`) |
| 4 | Tension Detector (contradictions, orphans, gaps, SPOFs) | ⏳ Built (`88f8ddb`), pending large-library test (≥50 linked notes) |
| 5 | Provenance Chain (isnad-inspired) | ✅ shipped (`2de0c15`) |
| 6 | Externalization Engine (fleeting → literature → permanent → synthesis) | ✅ shipped (`87d21d7`) |
| 7 | Review Pulse (spaced resurfacing + staleness scan) | ✅ shipped (`b2bbed0`) per roadmap; CE-spec body says "not started" — **roadmap wins** |
| 8 | Trails (named ordered sequences) | ✅ shipped (`96d7f3e`) |
| 9 | Multi-Lens Views | ✅ shipped (`4b72c0c`) |
| 10 | Expression Forge (synthesis workspace) | ✅ shipped (`e6e4966`) |
| 11 | Sense-Making Canvas (Cynefin quadrants) | ✅ shipped (`bec8e3d`) |

### 4.3 Layer 2 — AI Discovery (5 phases, 🔲 all not started)

12. Hidden Pattern Discovery (ghost links via semantic engine).
13. Blind Spot Detection.
14. Cross-Domain Insight Generation.
15. Socratic Challenger.
16. Worldview Synthesis.

**Local-LLM-first policy.** Cloud AI opt-in only. Existing infrastructure: `ai_send_message` Tauri command; embeddings via ONNX (Xenova `@xenova/transformers`).

### 4.4 The Living Link Architecture (CE Layer 2 in some docs, P0–P5 phases)

`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` is the philosophy doc. Implementation status: **all P0–P5 shipped + user-validated** (per cognitive-engine-roadmap and User Manual Tutorials 1–11).

**8 link properties** (the "blood components"):

| # | Property | Question |
|---|---|---|
| 1 | Type | What KIND of relationship? |
| 2 | Direction | Which way does knowledge flow? |
| 3 | Annotation | WHY does this connection exist? |
| 4 | Weight | How significant? |
| 5 | Confidence | How certain? |
| 6 | Created | When? |
| 7 | Last Traversed | Still alive? |
| 8 | Traversal Count | How active? |

**7 typed link types** (cognitive vocabulary; default is `relates`/`associative`):
`supports` (blue) · `contradicts` (red) · `causes` (orange) · `exemplifies` (green) · `generalizes` (purple) · `derives-from` (gold) · `part-of` (gray).

**Syntax in shipped code**: `[[Target|type]]` (pipe-after-target for type) and `[[type::Target|annotation]]` is **NOT** the canonical syntax in the implementation — only in the older `KNOWLEDGE-FORMULATION.md` design doc. **Pipe wins** per `CE-spec.md:90-97` and CE-TEST-RECORD's passing tests.

**4 confidence levels**: `hypothesis` → `evidence` → `established` → `contested`. Auto-promotes by traversal count: ≥3 → evidence, ≥10 → established. Right-click any link to override.

**Tier visual** (from traversal count): emerging (×1–2) · established (×3–9) · load-bearing (×10+) · stale (≥90 d untouched).

**Decay formula** (display-only — raw `weight` column never modified):
```
effectiveWeight = rawWeight × exp(−ln(2) × daysSinceTraversal / halfLifeDays)
```
Default half-life: 60 days. User-tunable (Settings → Sky View & Links → Living Link Lifecycle).

**Storage**: dual-layer design (LINK files on disk + SQLite). **The on-disk LINK files layer was deliberately deferred** — implementation lives only in `note_links` SQLite table. Frontmatter aliases drive resolution.

**Archive = soft-delete.** Every operation is reversible. Archived rows are hidden everywhere, restored from Link Dashboard's Archived tab.

---

## 5. The Arabic Engine + Lexical Bridge

A native 5-layer morphological engine. **Not a port** — built from scratch, license-clean.

**Layer order in `arabic::analyze_with_overrides`**:
0. **User Override** (`UserOverride` from `<universe>/.constellation/arabic-overrides.json`).
1. Protected list (`protected_seed.tsv`, ~1196 hand-picked entries: proper nouns / places / loanwords / function words).
2. Generative FST (BurntSushi `fst`, ~32k keys at current seed; mmap-backed on desktop).
3. Surface heuristic (Light10 fallback for unknown words).
4. Disambiguator (deterministic rank: confidence → origin priority → POS → fewer affixes → alphabetic).

**M-numbered milestones — all ✅ shipped** (per `lab/reports/SESSION-LOG-2026-04-18.md` headlines):

- M3 FST-backed generative index + M3-baker on-disk cache.
- M5 502-case regression corpus, 100% pass.
- M6 FTS5 routes Arabic stemming through `analyze_best`. Closes the flagship `وائل → "ائل"` mangle bug.
- M7 deterministic disambiguator (e.g., كاتب resolves to Noun reading every time).
- M8 + M8b + M8c — Layer 0 user overrides + ACTIVE_STORE registry + Settings UI.
- M9 bench harness — measured ~130k words/sec, ~7.6 MiB cache. Aspirational targets ≥200k w/s, ≤10 MiB at 7K-root scale.
- M10 Lexical Bridge architecture (15-concept seed).
- M11-infra Lexical Bridge baker (binary cache, content-addressed).
- M11-data v1 (49-concept hand-curated production seed).
- **M11-data v2 Producer** ✅ complete — corpus reached ~20K-concept target across **499 thematic shards** in `lab/m11-data/concepts/`.
- M12 query expansion plumbing (`escape_fts_term`, `build_match_expr`, `expand_to_match_expr`).
- M12-detect language detection (15-language classifier).
- M12-bench (mean 5.2 µs, p99 15.8 µs — 60–600× under 1 ms budget).
- M13 multilingual result badge (`match_via` end-to-end).
- M14 lexical_search end-to-end bench gate.

**Pending Arabic Engine perf follow-ons** (queued from M9):
- String-intern `pattern_label` across forms.
- mmap FST bytes (already shipped per `Cargo.toml: memmap2 = "0.9"` — **needs verification**, may already be done).
- Trim per-call `Arc::clone` on `ACTIVE_STORE`.

**Pending M11-data follow-ons**:
- M11-data-synonyms (sense-tagged in-language near-equivalents).
- M11-data-domains (science / philosophy / arts / Islamic studies / medicine packs).

**Custom FTS5 tokenizer**: `constellation` tokenizer registered via FFI (`src-tauri/src/fts5_tokenizer.rs`, ~18.6 KB). All Arabic tokens flow through `analyze_best` at index time and at query time symmetrically.

---

## 6. Canonical Filename Architecture

`docs/CANONICAL-FILENAME-ARCHITECTURE.md`. Every file Constellation manages gets an immutable canonical filename:

```
YYYYMMDDTHHMMSSZ_KIND_XXXX.ext
e.g. 20260410T153045Z_NOTE_7F3A.md (user sees "Agriculture System")
```

**12 core kinds**: `NOTE` · `BASE` · `TMPL` · `LINK` · `IMG` · `AUD` · `VID` · `ATT` · `CANVAS` · `DRAW` · `MARK` · `CLIP`. Auto-generated for unknown extensions (e.g. `.blend` → `BLEND`).

**Frontmatter contract** (the spec doc says `cid`; the actual code uses `cid_cn` namespace, migrated by `ensure_cid_cn` in `src-tauri/src/canonical.rs`):

```yaml
---
title: Agriculture System
cid_cn: 20260410T153045Z_NOTE_7F3A
kind: note
created: 2026-04-10T15:30:45Z
aliases:
  - Agriculture System
---
```

`cid_cn` is **immutable**. `title` and `aliases` are user-mutable. **Wikilinks always use titles** (`[[Agriculture System]]`), never canonical filenames.

**Resolution order**: title exact → aliases → original_filename → broken (red).

**Migration is opt-in.** Existing libraries with human filenames continue to work. **MIG-003** (human-name filenames overlay so canonical stems don't appear in the OS file explorer) is queued — not started.

Implemented by:
- `src-tauri/src/canonical.rs` (54 KB)
- `src-tauri/src/file_kinds.rs` (16 KB) — kind classification engine (3 layers: extension → markdown content heuristics → auto-generation).
- `src-tauri/src/importers.rs` (37 KB) — integrates classification + canonical naming into 7-format importer.

**14-Agent Audit** completed during the canonical landing session (`SESSION-LOG-2026-04-08`).

---

## 7. Editor (NotePane / FocusPane)

**Two editors**:

- **`FocusPane.svelte`** — quick capture. Plain text. **No markdown parser, no syntax highlighting, no decorations.** Fast capture, organize later. Bidi via `bidiPlugin` (per-line script detection).
- **`NotePane.svelte`** — full WYSIWYG-like editor on **CodeMirror 6**. Live preview decorations, callouts, code blocks, images, wikilinks, tables.

**Key NotePane spec rules** (`docs/NotePane-spec.md`, top-principal):

- **Section 2.1 — The Editor Owns Its Content.** After mount, CM6 owns the document. Communication is **one-way**: `Editor → onchange(text) → Parent stores → Debounced save`. Never Parent → Editor.
- **Section 2.6 — No `$effect` for Editor State.** No `$effect` block reads or writes `value`/`editBody`. Only allowed `$effect`s: dir change (guarded by `prevDir`), font change (guarded by `prevFontKey`), and similar narrow surfaces.
- **PaperOnDesk (PoD) layout**: gray desk `#e8e8ec`, white paper `max-width: 1200px`, `padding: 48px`.
- **Auto-title format**: `CoNoteDDMMYYYY.HH:MM` (older — pre-canonical-filenames). Current code generates canonical `YYYYMMDDTHHMMSSZ_NOTE_XXXX` filename + `title:` field in frontmatter.
- **8 audit agents** (PA / AA / MA / SCA / RA / UXA / CQA / EA) defined in NotePane-spec; **expanded to 14** in `docs/AUDIT-SYSTEM.md` (adds LA / SIA / SA / DIA / CFS / OGA).
- **Build phases 0–8** for eNotePane (now NotePane). Each phase passed all 8 auditors + user testing.

**eNotePane → NotePane history**: built phase-by-phase 2026-03-26 → 2026-03-29, promoted to production, old NotePane archived (`docs/eNotePane-development-record.md:107-114`).

**Editor Parity Rule** (`CLAUDE.md`): all note views share the same CM6 extensions. Shared modules in `src/lib/editor/`: `livePreview`, `calloutPlugin`, `lineDecoPlugin`, `bidiPlugin`, `completions`, `markdownHighlight`, `tableUtils`, `tableFormulas`, `iconSets`, `shortcodeAutocomplete`, `activeEditor`. **Exception**: FocusPane is plain-text; never gets new editor features.

---

## 8. Migrations (active state, 2026-04-26)

`/migration` is a four-phase workflow for changes that cross subsystem boundaries (schema, data flow, cross-surface invariants). Defined in `.claude/skills/migration.md`. Phases: **Architect → Plan → Build → Audit**.

| ID | Plan doc | Status |
|---|---|---|
| **MIG-001** Sky View Write-Time Derivation | `lab/reports/MIG-001-SKYVIEW-WTD.md` | ✅ Closed. All 11 steps + Phase 4 audit. Release-run boot-perf trace not yet collected. |
| **MIG-002** Enrichment Persistence (stratum/maturity/origin_type → sky_nodes columns) | `lab/reports/MIG-002-ENRICHMENT-PERSISTENCE.md` | ⏳ §1–§6 shipped + tested. §7 (`enrichment_worker.rs` drain loop), §8 (derives-from triggers), §9 (frontend swap), §10 (audit) **pending**. |
| **MIG-003** Human-name Filenames | (no plan doc yet — design in `docs/CANONICAL-FILENAME-ARCHITECTURE.md`) | 🔲 Not started. User-flagged readability pain (canonical stems shown in Explorer). |
| **MIG-004** Alias-Aware Resolution | `lab/reports/MIG-004-ALIAS-AWARE-RESOLUTION.md` | ✅ Closed. 9/12 invariants verified. Audit deferrals 4B-1, 4B-2 → MIG-005. |
| **MIG-005** Alias-aware in-memory inbound consumers | `lab/reports/MIG-005-ALIAS-AWARE-INMEMORY.md` | ⏳ Phase 1 Architect done (§120). Build steps 1–3 shipped (§121/§122/§123 — map.rs / strata.rs / maturity.rs). Tutorial **paused** mid-rewrite after fabrication caught. Steps 4 (tension.rs), 5 (inspector360.rs), 6 (LinkDashboard.svelte), 7 (doc drift fix), 8 (Phase 4 audit) pending. |
| **MIG-006** Wikilink Rename Cascade | `lab/reports/MIG-006-WIKILINK-CASCADE.md` | ⏳ §1 (oldName lift) ✅ verified. §2 (regex walker) ✅ shipped + 11 unit tests. §3 expanded shipped at `3c4732d` then **REVERTED at `5afe0c2`** (BUG-015). §3 redo + §4–§11 pending. |

### 8.1 The MIG-006 §3 / BUG-015 incident (reference for "what not to do")

Worth describing because the lessons are top-tier:

- **§115** (commit `3c4732d`, 2026-04-25) shipped MIG-006 §3 expanded "open-editor coherence" — three coordinated changes plus a fourth piece: a **value-prop → CM6 doc sync `$effect`** in NotePane that dispatched a doc-replace transaction whenever the parent's `body = $derived(parseFrontmatter(tab.content)).body` changed.
- The `$effect` raced with the `{#key tab.id+'|'+tab.path}` `onDestroy` during ordinary tab navigation. Click source → click target → reactivity propagated `tab.content` to target's body → the OLD source NotePane's `value` prop changed → the `$effect` replaced its own CM6 doc with target's body BEFORE `{#key}` ran destroy → destroy's `doFlush()` then read the swapped doc → `handleFlush` wrote that swapped content to the OLD pane's `mountedFilePath`. Result: target file body overwritten with source body (or vice versa).
- **NotePane spec §2.6 explicitly forbade this pattern by name.** Had the spec been read before the change shipped, the bug would have been surfaced before commit. It wasn't.
- §116 (`5afe0c2`) reverted §115. §117 + §118 cleaned docs + recovered disk state (BUG-014 orphan `cid_cn` closed as collateral).
- **Lesson**: per the BASIC RULE and Working Agreement #4 (both top principal), every change touching write paths / lifecycle / reactivity / IPC contract MUST validate against the architecture before shipping. The MIG-006 §3 plan even documented a fictional "existing prop-change handler" that didn't exist in the code — the plan misled itself.

---

## 9. Boot performance — 5 ship-gate criteria

`lab/boot-perf/BOOT-BUDGET.md` defines the criteria. Test corpus: **trial Universe (7,600 notes, 16 libraries, 656k typed links, 4k images on Windows 11 NTFS)**.

| # | Criterion | Status |
|---|---|---|
| 1 | UI visible ≤ 2.5 s | ✅ ~1 s production |
| 2 | Fully responsive (`hydrated_ms`) ≤ 6 s | ✅ closed at **811 ms** after Round 7 (per LL-021 follow-up) |
| 3 | Idle RSS ≤ 350 MB | 🔲 Not measured |
| 4 | Stat-sweep 50 externally-modified files ≤ 3 s, non-blocking | 🔲 Not implemented |
| 5 | Kill-mid-index recovery (no duplicate notes, no WAL corruption) | 🔲 Not implemented |

**Permanent diagnostic instrumentation** (cheap, kept after Criterion 2 closure):
- **Five-stamp IPC diagnostic** (LL-021): `invoke_start_unix_ms` → `server_start_unix_ms` → per-phase `Instant::now()` → `server_return_unix_ms` → `client_recv_unix_ms`. Distinguishes queue / body / transport.
- **`perf_trace::TRACE_LOG`** in `src-tauri/src/perf_trace.rs` — wraps `generate_handler!` to stamp every IPC dispatch arrival. Read via `get_perf_trace_log` IPC.
- **JS heartbeat** (max-gap tracking from `boot:paint` to `boot:hydrated`).

### 9.1 What closed Criterion 2

The `perf_trace` arrival tracer (Round 6) showed `constellation_map_universe` was being dispatched twice (~17.2 s gap between the two), blocking `cache_boot_snapshot_core`. The fix (Round 7) was a single attribute change: `#[tauri::command]` → `#[tauri::command(async)]` on `constellation_map_universe`. `core_queue_ms` dropped from ~19.9 s to 4 ms; `hydrated_ms` to 811 ms. **5,100× reduction in queue.**

### 9.2 Other boot-perf primitives

- **Covering index**: `idx_note_boot_snapshot ON note_meta(name, path, library_name)` — 100–1000× speedup on the boot snapshot projection (LL-020 corollary).
- **Paint-first UI** (LL-018): `appReady = true` synchronously at top of `initializeApp`; data hydrates after.
- **`LIBRARIES_CACHE`** (LL-016): in-memory cache for `load_all_libraries` invalidated by `save_libraries` and `set_active_universe`.
- **Always-mounted UI lazy-mount** (LL-022): `*EverOpened` flag pattern for expensive overlays (Map, OrgChart). CSS `display:none` does NOT prevent `onMount` IPC.

---

## 10. Standing rules (top-principal hierarchy)

These are loaded by every session via `CLAUDE.md` + persistent memory. **Order matters** — earlier rules override later ones.

### 10.1 BASIC RULE — Don't Make Things Up *(top of all rules)*

If I don't have a clue or information, I say **"I don't know."** No invented file paths, line numbers, function names, badge taxonomies, prior-art summaries, or any factual claim. Confident filler is fabrication. **Fabrication is the worst class of error** — bugs are recoverable; trust isn't.

When tempted to add a "side note" / "for context" / "by the way" — every claim in it must be sourced. If any claim isn't, the entire side note is cut.

Canonical violation prevented: 2026-04-26 tutorial fabricated T/C/P badge meanings as "Theory/Concept/Proposition." Actual taxonomy (per project owner): T = Title, C = Content, P = Property. With S = Semantic. **Other badge letters are unknown to me.**

### 10.2 Working Agreement #1–#4 *(non-negotiable)*

1. **Do the work yourself.** Don't offload to the user. SQL queries, log greps, file inspection, build verification — all Claude's job. Only ask the user for: GUI interaction, design decisions, plan approval, release confirmation.
2. **One location: `E:\مشاريع كلاود\Constellation` on `main`.** No worktree-as-domain. Use absolute paths into the primary location.
3. **The user is a non-technical IT Boss.** Plain language. No internal jargon unless asked. Test instructions follow the Tutorial Rule.
4. **Validate every change against the entire architecture before shipping.** Spawn parallel agents for any change touching write paths / lifecycle / reactivity / IPC contracts. Speed never overrides preservation. The MIG-006 §3-expanded → BUG-015 incident is the canonical violation this rule prevents.

### 10.3 Standing Orders

1. Update `lab/reports/SESSION-LOG-YYYY-MM-DD.md` after every phase, step, or significant commit.
2. Update help files + User Manual + 14 translations on user-facing changes.
3. Session log is the safety net for context loss across sessions.
4. Run `/simplify` (code review) after each phase.
5. **State-of-standing record before any pivot or major triage.** When the user says "where do we stand?" or asks for backlog/inventory, write a snapshot in the day's session log under §STATE-OF-STANDING covering: verified-shipped, at-risk/uncommitted, known-broken, pending/not-started, doc drift.
6. **Maintain `docs/Constellation-Orientation-and-Onboarding.md`** *(this file)*. When any architectural fact changes — phase ships, rule added, drift fixed, migration closes — update this file in the same commit. Version-bump on structural changes; date-stamp section updates.

### 10.4 Testing Instructions Rule (Tutorial Rule, top principal)

Every test instruction is a tutorial. Define the feature first (what / why / why it matters — same paragraph as help-file/User-Manual content), then click-by-click walkthrough. Pre-state, action, post-state per step. Failure modes spelled out. Plain language only — no internal component names unless explicitly asked. The user is the Boss, not a developer in Claude's team.

### 10.5 Plan Approval = Build Approval (top principal)

Once the user approves a plan, Claude cascades through the build steps autonomously. Stops happen only at: user-testable verification clauses, genuine architectural surprise, plan completion. Asking per-sub-step approval wastes the user's time.

### 10.6 Migration Rule

Any change that touches schema / core data flow / cross-surface invariants / multiple subsystems goes through `/migration` four-phase workflow before any code is written. Single-file refactors and local bug fixes use `/simplify`. Rule of thumb: does it cross Rust ↔ Svelte, schema ↔ code, write path ↔ read path? If yes, `/migration`. If no, don't.

### 10.7 Performance Rules (CLAUDE.md, 8 rules)

1. **Every keystroke must be instant.** ViewPlugin `update()` must use a line-change guard for `selectionSet`. Pre-cache module-level `Decoration` objects.
2. **No `$effect` loops.** No `$effect` reads and writes the same reactive variable. Use `$derived` for computed values.
3. **No heavy work on the main thread.** Vault indexing, search, file I/O → Rust. CM6 syntax tree iteration: only `view.visibleRanges`. Debounce saves ≥1500 ms. **Zero `invoke()` on the keystroke hot path.**
4. **No memory leaks.** Every `setTimeout`/`setInterval`/`addEventListener`/`EditorView`/`listen()`/`requestAnimationFrame` → cleanup in `onDestroy`.
5. **Minimal DOM.** Hide with `display: none`, not removal. Avoid `:global()` cross-tree CSS. Use flex/grid, no JS positioning.
6. **No unnecessary imports.** No `@codemirror/language-data` in FocusPane (500 KB+). Tree-shake aggressively. Lazy-load heavy features.
7. **Test before commit.** Type 10 chars rapidly in NotePane + FocusPane after every change. Verify `$effect` doesn't loop.
8. **Write-Time Derivation.** Every computed view in Constellation is maintained at write time, not read time. Persist derived views; trigger or hook on the source-of-truth write path; reads are cheap lookups. **Hard constraint**: no new feature may regress boot time, typing latency, or IPC responsiveness on the 7,600-note universe.

### 10.8 Architecture principles (CLAUDE.md)

- **File Over App.** `.md` on disk is the source of truth. The app is a window. Never lock content in proprietary formats.
- **Local-First.** No telemetry, no cloud dependency. Sync is the user's choice (Git, Syncthing, iCloud).
- **Knowledge Formulation, not Management.** See §1.
- **The Living Link Architecture.** See §4.4.
- **Constraint as Design.** Don't add features just because you can. Every feature must justify its existence. FocusPane has no toolbar — that IS the design.
- **Language-First by Design.** Bidi support is an architectural feature, not an add-on.
- **Constellation Knowledge Hierarchy** (5 levels). See §2.

### 10.9 Don't (CLAUDE.md, hard "no" list)

- Don't use preview/screenshot tools unless essential.
- Don't add unnecessary abstractions.
- Don't use "vault" terminology in new code.
- Don't add a feature that makes the app slower.
- Don't commit code with known `$effect` loops.
- Don't import heavy libraries in FocusPane.
- Don't use `position: absolute` for layout — flexbox/grid.
- Don't write CSS magic pixel numbers without documenting why.
- **Don't patch the same bug more than three times** (LL-014). Find root cause.
- Don't create `Decoration.mark/replace/widget` inside a builder function — pre-cache at module level.
- Don't call `invoke()` from a CM6 ViewPlugin, an input event handler, or any synchronous hot path.
- **Don't duplicate working code by copy-pasting and adapting.** Extract into a shared component.
- **Additional screens are displays, not domains.** Second Screen mounts core components — never re-implements save/load/edit.

### 10.10 PCS Protocol

**P**ush + **C**ommit + **S**tanding Order. Every milestone runs the full sequence: verify build → commit (specific files, no `git add -A`) → push → milestone tag → ZIP archive → session log → help files → 14 translation updates → SO commit + push.

### 10.11 Backup routine

After milestones: `git tag milestone/<name> <commit>` + `git push origin --tags`. ZIP archive: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`. Restore via `git checkout milestone/<name>` or unzip.

---

## 11. Lessons Learned (LL-001 → LL-023, summary)

`docs/LESSONS-LEARNED.md` is the canonical doc. Brief mental index:

- **LL-001** Tauri IPC is the #1 perf killer. Zero IPC during typing.
- **LL-002** The `+layout.svelte` reactivity cascade. Direct mutation bypasses Svelte; never store-mutate from `onDestroy` or hot path.
- **LL-003** Build passing ≠ working app. User-test in the running Tauri app.
- **LL-004** CM6 widget event handling — use capture-phase `addEventListener` on editor DOM for widget clicks; CM6 destroys widget before `domEventHandlers` fires.
- **LL-005** `tauri dev` rewrites Cargo.toml. Use forwarding feature pattern.
- **LL-006** Phase-by-phase with user GO/NO-GO. Small increments.
- **LL-007** Shared plugins in `src/lib/editor/` pay off (Phases 6–8 of eNotePane were fast).
- **LL-008** Session log is a lifeline.
- **LL-009** Derive state, don't duplicate it. `$derived` not `$state` for computable values.
- **LL-010** Merge iteration loops over visible ranges.
- **LL-011** Tauri v2 asset protocol — 4 things needed for image previews.
- **LL-012** `posAtDOM` unreliable for replacement widgets. Use `posAtCoords`.
- **LL-013** `getCursorColumn` pipe-counting bug — count pipes before cursor.
- **LL-014** **Three Strikes** — fix from root, not patch. After 3 distinct attempts, stop and find root cause.
- **LL-015** Always test production before chasing dev-mode performance (~37 s/IPC dev overhead).
- **LL-016** Cache at the call site when callers are unknown.
- **LL-017** When patching fails, spawn adversarial expert agents.
- **LL-018** **Paint-First UI** — never gate first paint on IPC.
- **LL-019** **PIXI v8 + Tauri CSP** — `import 'pixi.js/unsafe-eval'` as side-effect before any PIXI class. Empty black canvas = `unsafe-eval` blocked silently.
- **LL-020** Wall-vs-server-time decides where bottleneck is. Plus covering index corollary for narrow projections.
- **LL-021** **Five-stamp IPC diagnostic** + `perf_trace` arrival tracer. Methodology: Stage 1 stamps → Stage 2 plausible patches (stop after 2 fail) → Stage 3 cheap falsifiers → Stage 4 dispatcher tracer → Stage 5 named-culprit conversion.
- **LL-022** Always-mounted UI = always-running IPC. `*EverOpened` lazy-mount pattern.
- **LL-023** Don't regress working features. 4-step verification: render path → event path → state path → data path. **Any `$effect` that overwrites user intent is high-risk.** Predicted MIG-006 §3 / BUG-015 by name.

---

## 12. Documentation drift log (known stale documents)

When in doubt, **trust the code first, the most recent session log second, this orientation doc third**. Then fix the drift in the same commit.

| Doc | Drift |
|---|---|
| `docs/IPC-CONTRACT.md` | Last updated 2026-03-31. Lists ~50 commands; actual registry in `lib.rs:256-428` is 140+. Missing newer commands (cache_boot_snapshot_*, compute_note_*, detect_tensions, get_360_view, constellation_map_*, constellation_link_*, formulation_analysis, perf_trace, etc.). |
| `docs/CE-spec.md` | Body sections + progress table at line 862-878 are stale. Body says Phases 4 + 7 + 12-16 not started; roadmap and code show 1–11 done. Trust the roadmap. |
| `docs/CANONICAL-FILENAME-ARCHITECTURE.md` | Says `cid` in frontmatter; code uses `cid_cn` namespace via `ensure_cid_cn`. Doc not updated. |
| `docs/Constellation-Editor-Spec.md` | Describes a custom-built-from-scratch editor never built. CodeMirror 6 was used. Doc remains aspirational. |
| `docs/Constellation — Concept Paper.md:127` | "Mobile apps — Not yet"; Cargo.toml has iOS/Android `cfg` exclusions for memmap2 — mobile is at least compile-aware. Status uncertain. |
| `lab/reports/MIG-006-WIKILINK-CASCADE.md:165-167` | The §3 plan I wrote falsely claimed an existing prop-change handler did `view.dispatch({changes:...})`. No such handler existed. §115 had to build it, which produced BUG-015. The plan misled itself. |

---

## 13. Outstanding bugs / cosmetic issues

| ID | Status |
|---|---|
| **BUG-013** open-editor cascade race | Open. Documented limitation: switch tabs before renaming a target whose source is visible. |
| **BUG-014** orphan `cid_cn` (collateral from BUG-012) | Closed §118 (2026-04-25). |
| **BUG-015** target-body corruption from §115 value-sync `$effect` | Vector removed from `main` at §116 (`5afe0c2`). Forensic snapshots in `lab/forensics/`. |
| Title-heading rename gap | `NoteEditor.handleTitleChange` does not call `updateLinksOnRename`. Only file-tree rename triggers cascade. Open. |
| "Auto-update links on rename" toggle wrongly placed under Sky View & Links | Should be Knowledge Management. Open. |
| Sidebar active-item highlight lag (~10 s after wikilink nav) | Open. |

---

## 14. Where to read what (index)

When you need to know about… read this:

| Topic | Source |
|---|---|
| Why Constellation exists / vision | `docs/Constellation — Concept Paper.md` |
| Living Link philosophy + 8 properties + 7 types + 6 lifecycle stages | `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` |
| Cognitive Engine 16-phase spec | `docs/CE-spec.md` + `docs/cognitive-engine-roadmap.md` |
| Canonical filename format + 12 kinds + import pipeline | `docs/CANONICAL-FILENAME-ARCHITECTURE.md` |
| NotePane (eNotePane) editor rules | `docs/NotePane-spec.md` |
| Audit system (14 agents) | `docs/AUDIT-SYSTEM.md` |
| PCS protocol (commit/push/SO) | `docs/PCS-PROTOCOL.md` |
| Working protocols / Tutorial Rule / Two-environment workflow | `docs/WORK-BEHAVIOR.md` |
| Hard-won rules from real bugs | `docs/LESSONS-LEARNED.md` (LL-001 → LL-023) |
| Migration plans | `lab/reports/MIG-NNN-*.md` |
| Active boot-perf budget + criteria | `lab/boot-perf/BOOT-BUDGET.md` |
| What's in flight today | `lab/reports/SESSION-LOG-{latest-date}.md` |
| Subsystem status snapshot | `lab/reports/STATUS.md` |
| User-facing feature docs | `docs/help.uConstellation.World/<topic>/<topic>.md` |
| Master User Manual (English) | `docs/User Manual.md` |
| 14 translated User Manuals | `docs/help.{lang}/User Manual.md` |
| Tauri command registry (authoritative) | `src-tauri/src/lib.rs:232-432` (`generate_handler!` block) |
| Bases (database views) MVP | `docs/BASES_MVP_SPEC.md` |
| eNotePane build history | `docs/eNotePane-development-record.md` + `lab/experiments/phase-N-*.md` |
| Forensic snapshots from incidents | `lab/forensics/` |

---

## 15. Session-start protocol (for any new Claude session)

Mandated by `docs/WORK-BEHAVIOR.md` §1 + Standing Order #6:

1. **`git pull origin main`** to sync from any other device/session.
2. **`git log --oneline -10`** to see recent work.
3. **Read `lab/reports/SESSION-LOG-{latest-date}.md`** — pick up where the last session left off. Look for `§STATE-OF-STANDING` blocks.
4. **Read THIS document** (`docs/Constellation-Orientation-and-Onboarding.md`) — get architectural fluency in one read.
5. **Read `docs/LESSONS-LEARNED.md`** — every rule was earned by a real bug.
6. **Read `CLAUDE.md`** — top-principal rules + Working Agreement + Standing Orders.
7. **Read `lab/reports/STATUS.md`** — one-page subsystem status index.
8. **Read memory files** at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\MEMORY.md` (and the linked entries) — cross-session feedback.

If any of those files contradict each other, ground in the code (`grep`) and update the stale doc in the same session.

---

## 16. Standing Order #6 (this document's maintenance contract)

When any of these change, update this document in the same commit:

- A new migration starts, ships a step, or closes.
- A new top-principal rule is added or an existing one changes wording.
- A new BUG-NNN is opened or closed.
- A doc-drift item from §12 is fixed (remove the row).
- A new Lessons-Learned LL-NNN is added.
- A boot-perf criterion changes or closes.
- A version bumps (`Cargo.toml`, `package.json`, `tauri.conf.json`).
- A subsystem ships a major feature (CE Phase, MIG, Living Link slice, Arabic Engine M-milestone, etc.).
- A new help topic ships or a topic is restructured.

**Bump version (1.0 → 1.1, etc.)** when the section structure changes. Date-stamp every section that updates with the date of update. Keep "Last updated" lines at section heads where helpful.

The document **must remain readable in one pass.** If it grows past ~1500 lines, split into linked sub-documents kept in `docs/orientation/`.

---

## 17. What I (Claude) have NOT read in detail

This list is mandated by the BASIC RULE. It exists so future readers know exactly which claims in this doc are grounded in direct code inspection vs. design-doc inference. If you need certainty on a claim that touches an "unread" file, **read it before acting**.

**Source code I have NOT read in full** (as of this version):
- `src-tauri/src/canonical.rs` (54 KB) — only the frontmatter rewriter section.
- `src-tauri/src/universe.rs` (61 KB) — not read.
- `src-tauri/src/search.rs` (224 KB) — only sections via grep + alias plumbing.
- `src-tauri/src/cache.rs` (36 KB) — only alias resolution + read_sky_links_raw.
- `src-tauri/src/libraries.rs` (172 KB) — selected functions only.
- `src-tauri/src/arabic/*` (~300 KB across 10 files) — design from M-milestone docs only.
- `src-tauri/src/lexicon/*` (~140 KB) — design from M10-M14 docs only.
- `src-tauri/src/inspector360.rs`, `lens.rs`, `lenses.rs`, `bases.rs`, `importers.rs`, `ai/mod.rs`, `embeddings.rs`, `embeds.rs`, `dataview.rs`, `file_kinds.rs`, `fts5_tokenizer.rs`, `tasks.rs`, `trails.rs`, `review.rs`, `canvas.rs`, `provenance.rs`, `boot_bundle.rs`, `sky_backfill.rs` — **none read**, only signatures from `lib.rs` registration.
- Most Svelte components: `Inspector360`, `ConstellationMap`, `GraphMindView`, `ConstellationSight`, `ConstellationSight2`, `IndexPanel`, `OrgChart`, `SearchHub`, `SecondScreenPage`, `SettingsModal`, `BaseView`, `FullSkyView`, `SenseMakingCanvas`, `DashboardView`, `UniverseSetup`, `BacklinksPanel`, `OutgoingLinksPanel`, `PropertyEditor` — **none read in full**.
- `src/lib/libraries/store.ts` — sections only.
- `src/lib/i18n/*.json` (15 locales) — none read.
- `src/lib/editor/*.ts` extensions — design from NotePane spec only.
- `tauri.conf.json` — not read.

**Docs I have NOT read in full**:
- `docs/User Manual.md` — only TOC + first 100 lines.
- 23 of 24 help topics — only headlines (Cognitive Engine read in detail).
- All 14 translated User Manuals.
- `docs/BASES_MVP_SPEC.md` — only first 120 of ~330 lines.
- `docs/Constellation_Lens_Concept_Paper_Eisa.pdf`, `docs/GraphMind*.docx`, `docs/constellation_cognitive_engine_v2.1.pdf` — binary formats not read.

**Session logs partially read**:
- `SESSION-LOG-2026-04-18.md` (1.46 MB) — only structural headlines.
- `SESSION-LOG-2026-04-19.md` (99 KB) — only structural headlines + first 100 lines.

**Specifics I do NOT know**:
- The full set of badge-letter meanings beyond T = Title, C = Content, P = Property, S = Semantic.
- The full list of letters used as filter chips in the Constellation Map left panel.
- Whether Cargo.toml's `memmap2 = "0.9"` is wired through to the FST baker yet (M9 follow-on item).
- The exact alias/identifier collision tiebreak behavior under all edge cases.

**Future maintainers**: when you read one of the above and confirm a fact, update §17 to remove that item AND fold the verified fact into the relevant section above. Keep §17 honest.

---

*End of v1.0. Maintained per Standing Order #6.*
