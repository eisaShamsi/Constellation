# Constellation — Orientation & Onboarding

**Version 1.1 | 2026-04-26**
**Author of facts: Eisa ALSHAMSI (project owner, designer, IT Boss).**
**Maintainer: Claude (consultant / engineer / SME).**

---

## 0. How to use this document

**This is the first document any new Claude session reads.** It exists so a fresh AI can get to architectural fluency in one read instead of rediscovering the project from `git log` + screenshots over several frustrating turns.

**Maintenance is a Standing Order** (`CLAUDE.md` Standing Order #6). Whenever a fact below changes — a phase ships, a rule is added, a doc-drift item is fixed, a migration closes — update this file in the same commit that lands the change. Bump the version when the structure changes; date-stamp every section that updates. **The filename always carries its version suffix**: `Constellation Orientation & Onboarding v1.0.md` → `... v1.1.md`. Rename the file in the same commit that bumps the version — never leave a versionless or stale-version filename in `docs/`.

**This document is grounded.** Every claim cites the authoritative source (file:line, commit hash, or session log section). When two project documents disagree, I name both and don't pick a winner unless code-reading resolves it. When I don't know something, I say so explicitly in §17.

**Hard rule for every reader (human or AI) of this file**: if you find this document contradicts the actual codebase or a more recent session log, **trust the code and the session log first**, then update this file in the same session. Do not let stale orientation rot in place.

### v1.1 changelog (vs v1.0)

- §3: corrected `+layout.svelte` line count (3873 → 6872; the 3873 figure was 2026-03-27 from LL-002, since nearly doubled). Added route tree — only 4 `.svelte` files; root `+page.svelte` is a 1-line stub.
- §3.4 (NEW): build / release / CSP / capabilities / second-window URL.
- §4: each shipped phase now annotated with Write-Time-Derivation status.
- §5: clarified — **the M-numbers are milestones, not module boundaries**. The Arabic Engine is 5 layers in 15 Rust files (not 10). The Lexical Bridge is 6 modules in `src-tauri/src/lexicon/`.
- §6: added the verbatim `cid_cn` rationale (from `canonical.rs:1178-1185`).
- §7.1 (NEW): the 11 editor-stack files enumerated; pre-cached Decoration objects pinpointed.
- §8: MIG status refreshed to 2026-04-26 close-of-day.
- §10.12 (NEW): filename-versioning convention (this doc's filename always carries `vX.Y`).
- §11: LL-019, LL-020, LL-021, LL-022, LL-023 expanded with verbatim rule-text (now read in full).
- §12: new drift entries — Rule-8 violation list (`bases.rs`, `dataview.rs`, `lenses.rs`, Constellation Map); audit-agent count inconsistency (7 in `lab/audit-agents.md`, 8 in NotePane spec, 14 in `docs/AUDIT-SYSTEM.md`); no frontend test harness.
- §17: dramatically reduced — bases/dataview/lenses/embeddings/importers/watcher, canonical, file_kinds, lib.rs, tauri.conf.json, capabilities, CI, LESSONS-LEARNED.md, earliest session log all now read.
- New persistent fact: every `#[tauri::command]` dispatch is stamped via `perf_trace::record(invoke.message.command())` wrapped around `generate_handler!` ([`lib.rs:233-432`](src-tauri/src/lib.rs:233)).

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
- **Stack**: Tauri v2 (Rust backend) + SvelteKit + Svelte 5 + SQLite (rusqlite, bundled) + ONNX Runtime (`ort`) + CodeMirror 6 + PIXI v8 + D3 v7
- **Languages supported at launch**: 15 — `ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh`
- **RTL languages first-class**: 4 — Arabic, Hebrew, Persian, Urdu
- **Platforms**: Windows, macOS, Linux desktop. CI ships Windows builds today (`.github/workflows/release.yml` runs on `windows-latest`). iOS/Android not shipping despite Cargo `cfg` exclusions.

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

- **Universe** = portable directory. Contains `.constellation/` subfolder with `universe.json`, `libraries.json`, `settings.json`, `bookmarks.json`, `workspaces.json`, `property-types.json`, `bases/`, `templates/`. Move it to another machine and the entire workspace follows.
- **Library** = first-class citizen with its own color/appearance/tags/links/index. Registered in `libraries.json`. Multiple libraries coexist in one Universe. Constellation reads them in place — never copies.
- **Folder ≠ Library**. Folders are organizational only.
- **Terminology**: use "Library" everywhere, **never** "vault" (except for Obsidian import compatibility).

### 2.1 Universe migration (legacy → current)

`universe.rs::migrate_legacy_data` ([universe.rs:1306-1390](src-tauri/src/universe.rs:1306)) moves a v1 layout to v2:

- **From**: flat `universe.json` / `vaults.json` / `settings.json` at universe root; registry stored at `app_data_dir/vaults.json`; nested `name/name/` notes layout.
- **To**: `.constellation/` subdirectory; `vaults.json` renamed to `libraries.json`; registry moved to `app_data_dir/universes.json` (UniverseRegistry with `entries` and `active_id`); flat notes layout (Universe root IS the library, Obsidian-style).

`check_migration_needed` runs at startup; the user is prompted before any move.

---

## 3. Architecture (one-page view)

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (SvelteKit / Svelte 5)                                │
│  src/routes/                                                    │
│    +layout.svelte (6872 lines — orchestrator, see §3.2)         │
│    +page.svelte (1 line — note viewing handled by layout)       │
│    libraries/+page.svelte (704 lines — library management)      │
│    skills/+page.svelte (219 lines — skills/onboarding)          │
│  Second window: static/screen.html (separate Tauri webview)     │
│  Editors: NotePane.svelte / FocusPane.svelte (§7)               │
│  Panels: Sky View (PIXI), Constellation Map (D3 sunburst),      │
│    Inspector 360, Tension, Sight, Lens, Bases, Tasks, Calendar, │
│    Backlinks, OutgoingLinks, IndexPanel, OrgChart, SearchHub    │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC (~120 commands, 32 Rust modules)                     │
│  - perf_trace (LL-021): every dispatch stamped at the boundary  │
│    via Box-typed closure wrapping generate_handler!             │
│  - 3 plugins: opener / process / updater                        │
│  - panic hook in run() writes constellation-crash.log           │
├─────────────────────────────────────────────────────────────────┤
│  Backend (Rust, src-tauri/src/, 32 modules)                     │
│  - libraries.rs — file I/O, link extraction, cascade walker     │
│  - search.rs — SQLite, FTS5, Living Link triggers,              │
│    sky_nodes/sky_links triggers (Rule 8)                        │
│  - cache.rs — boot snapshot, alias resolution                   │
│  - canonical.rs / file_kinds.rs — YYYYMMDDTHHMMSSZ_KIND_XXXX    │
│  - universe.rs — universe/cUniverse + legacy migration          │
│  - arabic/ (15 files) — 5-layer morphological engine            │
│  - lexicon/ (6 modules) — Lexical Bridge polylingual lemma graph│
│  - strata.rs / maturity.rs / tension.rs / provenance.rs /       │
│    inspector360.rs / map.rs / lens.rs / lenses.rs / review.rs / │
│    trails.rs / canvas.rs (CE Layer 1)                           │
│  - bases.rs — .base file CRUD (read-time)                       │
│  - dataview.rs — DQL queries (read-time)                        │
│  - importers.rs — 7 source formats (one-off, async)             │
│  - watcher.rs — notify-rs file watch (must be async)            │
│  - boot_bundle.rs / sky_backfill.rs / tasks.rs                  │
│  - embeddings.rs — ONNX multilingual-e5-small (write-time)      │
│  - embeds.rs / fts5_tokenizer.rs                                │
│  - perf_trace.rs — IPC arrival tracer                           │
│  - ai/mod.rs — multi-provider AI                                │
├─────────────────────────────────────────────────────────────────┤
│  Storage                                                         │
│  - .md files on disk (source of truth)                          │
│  - SQLite DB at <universe>/.constellation/search.db              │
│    Tables: note_meta, note_links, note_aliases, sky_nodes,      │
│    sky_links, notes_fts (custom 'constellation' tokenizer),     │
│    notes_vocab (fts5vocab), schema_versions                     │
│  - boot-perf.latest.json — per-boot scorecard                   │
│  - .meta.json sidecars for non-markdown files (canonical)       │
│  - .constellation/review-pulse.json — Phase 7 schedule state    │
│  - .constellation/arabic-overrides.json — L5 user overrides     │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 Key dependencies (versions)

| Layer | Package | Version | Purpose |
|---|---|---|---|
| Rust | `tauri` | 2.x with `protocol-asset` feature | App runtime |
| Rust | `rusqlite` | bundled | SQLite |
| Rust | `ort` | ONNX Runtime | Semantic embeddings |
| Rust | `tokenizers` | HuggingFace | Tokenizers (with `onig`) |
| Rust | `fst` | BurntSushi | Arabic generative index |
| Rust | `memmap2` | 0.9 (desktop only) | mmap baked Arabic FST |
| Rust | `notify` | File watcher | |
| JS | `svelte` | ^5.0 | UI framework (runes mode) |
| JS | `@sveltejs/kit` | ^2.9 | Routing |
| JS | `@codemirror/*` | 6.x (full set) | Editor |
| JS | `pixi.js` | ^8.17 | Sky View force graph (LL-019: import `pixi.js/unsafe-eval` first) |
| JS | `d3` | ^7.9 | Constellation Map sunburst |
| JS | `@xenova/transformers` | ^2.17 | Frontend ONNX (where applicable) |
| JS | `katex` / `mermaid` / `marked` / `dompurify` | latest | Math / diagrams / markdown / XSS |

Plugins: `tauri-plugin-opener`, `tauri-plugin-process`, `tauri-plugin-updater`.

### 3.2 The `+layout.svelte` reactivity load

`+layout.svelte` is the orchestrator. As of 2026-04-26 it is **6872 lines** (was 3873 lines on 2026-03-27 when LL-002 was earned). The pattern from LL-002 still holds: any store mutation can cascade across the whole file. Performance discipline lives in the rules — never store-mutate from `onDestroy` or hot paths; pre-cache derivations; use `*EverOpened` lazy-mount for heavy panels (LL-022).

`+page.svelte` is **a single-line comment** — the entire note-viewing UI is composed inside `+layout.svelte`. The `libraries/` and `skills/` routes are real pages.

### 3.3 Tauri command surface

[`lib.rs:233-432`](src-tauri/src/lib.rs:233) registers ~120 commands across 32 modules. The `invoke_handler` is wrapped in a Box-typed closure that records each dispatch via `perf_trace::record(invoke.message.command())` — the LL-021 IPC arrival tracer. **Every IPC entry is timestamped at the dispatcher** without per-command instrumentation.

Two Tauri v2 type-system subtleties worth recording (from LL-021):

1. `generate_handler!` must be bound via `Box<dyn Fn(Invoke<Wry>) -> bool + Send + Sync + 'static>` to pin the macro's `R: Runtime` generic at the binding site (otherwise Rust emits `E0282 cannot infer type`).
2. `invoke.message.command()` returns `&str`; call `perf_trace::record` *before* forwarding to `inner(invoke)` (which consumes `invoke` by value).

**[`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) is significantly stale** (last updated 2026-03-31; lists ~80 commands). Until refreshed, [`lib.rs:233-432`](src-tauri/src/lib.rs:233) is the authoritative command registry.

### 3.4 Build / Release / CSP / Windows / Capabilities

**Versions** (in sync, `0.3.4` at v1.1):
- [`package.json`](package.json) — `"version": "0.3.4"`
- [`src-tauri/tauri.conf.json:4`](src-tauri/tauri.conf.json:4) — `"version": "0.3.4"`
- `src-tauri/Cargo.toml` — bumped per release workflow

**`tauri.conf.json` highlights**:
- `productName: "Constellation"`, `identifier: "world.uconstellation.app"`
- Two windows defined:
  - `main` — 1200×800, title "Constellation"
  - `second-screen` — 1200×800, `url: "screen.html"`, `visible: false` at startup
- CSP: `default-src 'self'`; `script-src 'self' 'unsafe-inline'`; **no `unsafe-eval`** → LL-019 still applies (any library using `new Function()` for runtime codegen needs its `*/unsafe-eval` variant, like `pixi.js/unsafe-eval`).
- Asset protocol enabled, `allow: ["**/*"]`, `requireLiteralLeadingDot: false`.
- Updater enabled, endpoint = public Gist (`gist.githubusercontent.com/.../latest.json`); minisign pubkey embedded.

**Capabilities** ([`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json)) — applies to both `main` and `second-screen` windows. Permissions: `core:default`, window controls (set-title/create/close/set-focus/center/destroy), `core:webview:allow-create-webview-window`, `core:webview:allow-set-webview-zoom`, `opener:default`, `updater:default`, `process:allow-restart`.

**Second-window file**: [`static/screen.html`](static/screen.html) (built copy lives at `build/screen.html`).

**CI / release** ([`.github/workflows/release.yml`](.github/workflows/release.yml)) — `windows-latest` runner. Two triggers:
1. Tag push `v*` → build + release.
2. Manual `workflow_dispatch` with `bump: patch|minor|major` or `custom_version: 0.X.Y` → bumps `package.json` + `tauri.conf.json` + `Cargo.toml`, commits, tags, pushes, then runs `tauri-action`.

After release, the workflow downloads `latest.json` from the release assets and `gh gist edit` updates the public Gist that the in-app updater polls.

**No frontend test harness yet.** No vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`. Rust-side unit tests exist (cascade_walker_tests, canonical, file_kinds — see §11.3 / drift list).

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

11 phases. **All shipped** per `docs/cognitive-engine-roadmap.md` and session log §1188 ("🎉 LAYER 1 COMPLETE", 2026-04-03). Write-Time Derivation (Rule 8) status added per actual code review (2026-04-26):

| # | Name | Rust | Rule 8? |
|---|---|---|---|
| 1 | Typed Links | `libraries.rs` + `search.rs` (note_links + triggers) | ✅ Write-time |
| 2 | Knowledge Strata (8-level) | [`strata.rs`](src-tauri/src/strata.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1137`](src-tauri/src/search.rs:1137)) |
| 3 | Maturity Lifecycle | [`maturity.rs`](src-tauri/src/maturity.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1215`](src-tauri/src/search.rs:1215)) |
| 4 | Tension Detector | `tension.rs` | ⚠️ Partial — contradictions cached, structural gaps on read |
| 5 | Provenance Chain (isnad-inspired) | `provenance.rs` | ⚠️ Partial — frontmatter sources cached, traversals on read |
| 6 | Externalization (signal in word_count) | within `strata.rs` | ✅ Write-time |
| 7 | Review Pulse | `review.rs` | Hybrid — `.constellation/review-pulse.json` |
| 8 | Trails (named ordered sequences) | `trails.rs` | ✅ Write-time |
| 9 | **Multi-Lens Views** | `lenses.rs` | ❌ **Hybrid violation** — definitions write-time (`lenses.json`), results recomputed on read (`apply_lens` walks the tree) |
| 10/11 | Expression Forge / Sense-Making Canvas | `canvas.rs` | ✅ Write-time (JSON persisted) |
| 12 | 360° Inspector | `inspector360.rs` | N/A — pure read aggregation |

### 4.3 Layer 2 — AI Discovery (5 phases, 🔲 all not started)

12. Hidden Pattern Discovery (ghost links via semantic engine).
13. Blind Spot Detection.
14. Cross-Domain Insight Generation.
15. Socratic Challenger.
16. Worldview Synthesis.

**Local-LLM-first policy.** Cloud AI opt-in only. Existing infrastructure: `ai_send_message` Tauri command; embeddings via ONNX (Xenova `@xenova/transformers` on the JS side, `ort` on the Rust side).

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

**Syntax in shipped code**: `[[Target|type]]` (pipe-after-target for type). Older `[[type::Target|annotation]]` from `KNOWLEDGE-FORMULATION.md` is **NOT** the implementation — pipe wins per `CE-spec.md:90-97` and CE-TEST-RECORD's passing tests.

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

### 5.1 Engine architecture (verbatim from [`arabic/mod.rs:16-37`](src-tauri/src/arabic/mod.rs:16))

```
[L1 normalizer]        — tashkeel / tatweel removal, hamza variants,
                          language detection; preserves surface form
   ↓
[L2 protected list]    — ~20K proper nouns + loanwords (hash lookup)
   ↓
[L3 generative FST]    — rolling-hash + FST over all (root × pattern)
                          combinations
   ↓
[L4 disambiguator]     — ranks multiple analyses by corpus frequency
   ↓
[L5 user overrides]    — per-Universe learning layer
```

**5 logical layers, 15 physical Rust files** in `src-tauri/src/arabic/`:
`normalizer.rs` · `protected.rs` · `fst_index.rs` · `fst_bake.rs` · `generator.rs` · `patterns.rs` · `affixes.rs` · `roots.rs` · `disambiguate.rs` · `overrides.rs` · `types.rs` · `regression.rs` · `bench.rs` · `rss.rs` · `mod.rs`.

**Entry points** ([`arabic/mod.rs:129-564`](src-tauri/src/arabic/mod.rs:129)):
- `analyze(word) → AnalysisList` — full pipeline, no overrides
- `analyze_with_overrides(word, store?) → AnalysisList` — with L5
- `analyze_best(word) → Analysis` — single best pick (FTS5 tokenizer hook)
- `analyze_with_overrides_best(word, store?) → Analysis` — best + L5

**Tauri commands** (`arabic::overrides`): `read_arabic_overrides`, `add_arabic_override`, `remove_arabic_override`, `reindex_arabic_overrides`.

### 5.2 M-numbered milestones (NOT module boundaries)

The "M3-M14" series in session logs are **project milestones**, not architecture. The engine itself is 5 layers (above). All M-milestones shipped per `lab/reports/SESSION-LOG-2026-04-18.md`:

- M3 FST-backed generative index + M3-baker on-disk cache.
- M5 502-case regression corpus, 100% pass.
- M6 FTS5 routes Arabic stemming through `analyze_best`. Closes the flagship `وائل → "ائل"` mangle bug.
- M7 deterministic disambiguator (e.g., كاتب resolves to Noun reading every time).
- M8 + M8b + M8c — Layer 5 user overrides + ACTIVE_STORE registry + Settings UI.
- M9 bench harness — measured ~130k words/sec, ~7.6 MiB cache.
- M10 Lexical Bridge architecture (15-concept seed).
- M11-infra Lexical Bridge baker (binary cache, content-addressed).
- M11-data v1 (49-concept hand-curated production seed).
- **M11-data v2 Producer ✅ complete** — corpus reached **20,000 concepts** across **499 thematic shards** in `lab/m11-data/concepts/`. Verified by `wc -l lexicon_v1.tsv` = 20,015 lines (incl. header).
- M12 query expansion plumbing (`escape_fts_term`, `build_match_expr`, `expand_to_match_expr`).
- M12-detect language detection (15-language classifier).
- M12-bench (mean 5.2 µs, p99 15.8 µs — 60–600× under 1 ms budget).
- M13 multilingual result badge (`match_via` end-to-end).
- M14 lexical_search end-to-end bench gate.

### 5.3 Lexical Bridge

`src-tauri/src/lexicon/` — **6 modules**: `bake.rs`, `detect.rs`, `expansion.rs`, `fts.rs`, `graph.rs`, `parse.rs`.

A **polylingual lemma graph**, not a morphological tool: every lemma in any of the 15 supported languages can be looked up and yields its equivalents in any other language. Storage: `src-tauri/src/lexicon/data/lexicon_v1.tsv`. Built deterministically by `lab/m11-data/build.py` (Python 3) from 499 JSON shards.

**Coverage policy**: `en` + `ar` required per concept; target ≥8 of 15 languages. **No third-party sources** — all content is Constellation-original (WordNet / Wiktionary explicitly rejected per project policy in `lab/m11-data/README.md`).

### 5.4 Custom FTS5 tokenizer ('constellation')

[`src-tauri/src/fts5_tokenizer.rs`](src-tauri/src/fts5_tokenizer.rs) (479 lines). Wraps the Rust stemming pipeline: Arabic Light10 + Hebrew prefix stripping + Persian / Cyrillic / Devanagari / German / Spanish / Portuguese / French / Turkish / English stemmers + bigrams. Symmetric across `FTS5_TOKENIZE_DOCUMENT` (write) and `FTS5_TOKENIZE_QUERY` (read).

**Token emission** (`emit_word`, lines 425-478):
1. Primary token: stemmed form.
2. Bigram (colocated): `prev_stem \x1f cur_stem` (separator byte `0x1f` is unmatchable in user text).
3. Stopwords/length-filtered: emit nothing, break bigram chain.
4. Bigrams only form between tokens **in the same script** (prevents Arabic↔English bigram noise).

All Arabic-side morphology delegated to `crate::libraries::process_word_for_fts` → Arabic Engine `analyze_best()`.

---

## 6. Canonical Filename Architecture

`docs/CANONICAL-FILENAME-ARCHITECTURE.md`. Every file Constellation manages gets an immutable canonical filename:

```
YYYYMMDDTHHMMSSZ_KIND_XXXX.ext
e.g. 20260410T153045Z_NOTE_7F3A.md (user sees "Agriculture System")
```

**12 core kinds** (verbatim from [`file_kinds.rs:25-45`](src-tauri/src/file_kinds.rs:25)): `NOTE` · `BASE` · `TMPL` · `LINK` · `MARK` · `CLIP` · `IMG` · `AUD` · `VID` · `ATT` · `CANVAS` · `DRAW`. Auto-generated for unknown extensions (e.g. `.blend` → `BLEND`).

### 6.1 The `cid_cn` namespace (verbatim rationale)

From [`canonical.rs:1178-1185`](src-tauri/src/canonical.rs:1178):

> "Constellation's stable note identifier is stored under the namespaced property name `cid_cn:` (Constellation Node id) instead of the generic `cid:` so it can never collide with a pre-existing `cid:` property in a user's Obsidian vault."

Frontmatter contract:

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

`cid_cn` is **immutable** and equals the canonical filename stem. `title` and `aliases` are user-mutable. **Wikilinks always use titles** (`[[Agriculture System]]`), never canonical filenames.

`ensure_cid_cn()` injects `cid_cn` on first read; legacy `cid:` is migrated in-place.

**Resolution order**: title exact → aliases → original_filename → broken (red).

**Migration is opt-in.** Existing libraries with human filenames continue to work. **MIG-003** (human-name filenames overlay so canonical stems don't appear in OS file explorer) is queued — not started.

Implemented by:
- [`src-tauri/src/canonical.rs`](src-tauri/src/canonical.rs) — generation, injection, repair, ensure_cid_cn
- [`src-tauri/src/file_kinds.rs`](src-tauri/src/file_kinds.rs) — 3-layer kind classification (extension → markdown content → auto-generated). Has 4 unit tests.
- [`src-tauri/src/importers.rs`](src-tauri/src/importers.rs) — integrates classification + canonical naming into 7-format importer (Obsidian, Bear, Notion, Evernote ENEX, Markdown folders, HTML archives, Constellation backups).

---

## 7. Editor (NotePane / FocusPane)

**Two editors**:

- **`FocusPane.svelte`** — quick capture. Plain text. **No markdown parser, no syntax highlighting, no decorations.** Imports only `bidiPlugin` + base CM6. Fast capture, organize later.
- **`NotePane.svelte`** — full WYSIWYG-like editor on **CodeMirror 6**. Live preview decorations, callouts, code blocks, images, wikilinks, tables.

### 7.1 The shared editor stack

`src/lib/editor/` — 11 plugins, imported by every editor instance per the **Editor Parity Rule** (CLAUDE.md):

| File | Purpose |
|---|---|
| `activeEditor.ts` | Global active-editor registry |
| `bidiPlugin.ts` | Per-line bidi (RTL/LTR) + script-specific fonts |
| `calloutPlugin.ts` | Obsidian callouts. **FREEZE-PROOF** (LL-014): no `Decoration.replace` with cursor-inside |
| `completions.ts` | Wikilink / tag / slash / typed-link / shortcode autocomplete |
| `iconSets.ts` | Icon library loader |
| `lineDecoPlugin.ts` | Line-level decorations (separate plugin avoids mark/replace conflicts) |
| `livePreview.ts` | Core inline-render plugin. **Pre-cached module-level Decoration objects** at [livePreview.ts:155-181](src/lib/editor/livePreview.ts:155) (`replaceDeco`, `highlightDeco`, `hrDeco`, `blockquoteDeco`, `tagDeco`, HTML decos, 8 typed-link decos, 2 checkbox decos) |
| `markdownHighlight.ts` | `==highlight==` syntax via lezer extension |
| `shortcodeAutocomplete.ts` | `{shortcode}` autocomplete |
| `tableFormulas.ts` | Spreadsheet-like cell formulas |
| `tableUtils.ts` | Table parsing / formatting / row-col / sort |

### 7.2 Key NotePane spec rules ([`docs/NotePane-spec.md`](docs/NotePane-spec.md), top-principal)

- **Section 2.1 — The Editor Owns Its Content.** After mount, CM6 owns the document. Communication is **one-way**: `Editor → onchange(text) → Parent stores → Debounced save`. Never Parent → Editor.
- **Section 2.6 — No `$effect` for Editor State.** No `$effect` block reads or writes `value`/`editBody`. Only allowed `$effect`s: dir change (guarded by `prevDir`), font change (guarded by `prevFontKey`), and similar narrow surfaces. **Violating §2.6 caused BUG-015** (see §8.1).
- **PaperOnDesk (PoD) layout**: gray desk `#e8e8ec`, white paper `max-width: 1200px`, `padding: 48px`.
- **Auto-title format**: code generates canonical `YYYYMMDDTHHMMSSZ_NOTE_XXXX` filename + `title:` field in frontmatter.

### 7.3 Audit-agent count (clarification — was inconsistent in v1.0)

Three agent sets exist; "14 audit agents" merges them:

- **[`lab/audit-agents.md`](lab/audit-agents.md) — 7 agents** for eNotePane: PA / AA / MA / SCA / RA / UXA / CQA.
- **NotePane spec — 8 agents**: above + EA (Environment Auditor), added 2026-03-27 per [`SESSION-LOG-2026-03-27.md:67-72`](lab/reports/SESSION-LOG-2026-03-27.md:67) Step 4.
- **[`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) — 14 agents**: 8 above + LA / SIA / SA / DIA / CFS / OGA. (Verbatim list in `docs/AUDIT-SYSTEM.md`.)

Migrations use a different audit cohort: Phase 4 of `/migration` runs three parallel agents (Invariant Check, Drift Check, Migration Path) per [`.claude/skills/migration.md`](.claude/skills/migration.md).

---

## 8. Migrations (active state, 2026-04-26)

`/migration` is a four-phase workflow for changes that cross subsystem boundaries (schema, data flow, cross-surface invariants). Defined in [`.claude/skills/migration.md`](.claude/skills/migration.md). Phases: **Architect → Plan → Build → Audit**.

| ID | Plan doc | Status |
|---|---|---|
| **MIG-001** Sky View Write-Time Derivation | `lab/reports/MIG-001-SKYVIEW-WTD.md` | ✅ Closed. All 11 steps + Phase 4 audit. |
| **MIG-002** Enrichment Persistence (stratum/maturity/origin_type → sky_nodes) | `lab/reports/MIG-002-ENRICHMENT-PERSISTENCE.md` | ⏳ §1–§6 shipped + tested. §7–§10 pending. |
| **MIG-003** Human-name Filenames | (no plan doc yet) | 🔲 Not started. |
| **MIG-004** Alias-Aware Resolution | `lab/reports/MIG-004-ALIAS-AWARE-RESOLUTION.md` | ✅ Closed. 9/12 invariants verified. Audit deferrals 4B-1, 4B-2 → MIG-005. |
| **MIG-005** Alias-aware in-memory inbound consumers | `lab/reports/MIG-005-ALIAS-AWARE-INMEMORY.md` | ⏳ Steps 1–3 shipped (§121/§122/§123 — `map.rs` / `strata.rs` / `maturity.rs`). Tutorial paused after fabrication caught. Steps 4–8 pending. |
| **MIG-006** Wikilink Rename Cascade | `lab/reports/MIG-006-WIKILINK-CASCADE.md` | ⏳ §1 (oldName lift) ✅ verified. §2 (regex walker) ✅ shipped + 11 unit tests. §3 expanded shipped at `3c4732d` then **REVERTED at `5afe0c2`** (BUG-015). §3 redo + §4–§11 pending. |

### 8.1 The MIG-006 §3 / BUG-015 incident (reference for "what not to do")

- **§115** (commit `3c4732d`, 2026-04-25) shipped MIG-006 §3 expanded "open-editor coherence" — three coordinated changes plus a **value-prop → CM6 doc sync `$effect`** in NotePane that dispatched a doc-replace transaction whenever the parent's body prop changed.
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
- **Watcher must be async**: [`watcher.rs:19-38`](src-tauri/src/watcher.rs:19) inline-comment — installing a recursive filesystem watch is blocking I/O. Without `#[tauri::command(async)]`, every sync command runs on the WebView2 UI thread → Boot Criterion 2 fails.

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
5. **State-of-standing record before any pivot or major triage.** Write a snapshot in the day's session log under `§STATE-OF-STANDING` covering: verified-shipped, at-risk/uncommitted, known-broken, pending/not-started, doc drift.
6. **Maintain `docs/Constellation Orientation & Onboarding vX.Y.md`** *(this file)*. Filename always carries version suffix; rename in the same commit that bumps the version. When any architectural fact changes — phase ships, rule added, drift fixed, migration closes — update this file in the same commit. Version-bump on structural changes; date-stamp section updates.

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

### 10.12 Versioned filename for this orientation doc *(NEW in v1.1)*

This file's name **always** carries its version suffix: `Constellation Orientation & Onboarding vX.Y.md`. When bumping (1.0 → 1.1, 1.1 → 1.2, etc.), rename in the same commit. Never leave a versionless or stale-version filename in `docs/`. SO #6 enforces this.

---

## 11. Lessons Learned (LL-001 → LL-023, summary)

[`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) is the canonical doc. Brief mental index:

- **LL-001** Tauri IPC is the #1 perf killer. Zero IPC during typing.
- **LL-002** The `+layout.svelte` reactivity cascade. Direct mutation bypasses Svelte; never store-mutate from `onDestroy` or hot path. *(2026-03-27, file was 3873 lines. Today: 6872 lines.)*
- **LL-003** Build passing ≠ working app. User-test in the running Tauri app.
- **LL-004** CM6 widget event handling — use capture-phase `addEventListener` on editor DOM for widget clicks; CM6 destroys widget before `domEventHandlers` fires.
- **LL-005** `tauri dev` rewrites Cargo.toml. Use forwarding feature pattern: add `[features]` to app's Cargo.toml that forwards to dependency (`protocol-asset = ["tauri/protocol-asset"]`); add feature name to `build.features` in `tauri.conf.json`.
- **LL-006** Phase-by-phase with user GO/NO-GO. Small increments.
- **LL-007** Shared plugins in `src/lib/editor/` pay off (Phases 6–8 of eNotePane were fast).
- **LL-008** Session log is a lifeline.
- **LL-009** Derive state, don't duplicate it. `$derived` not `$state` for computable values.
- **LL-010** Merge iteration loops over visible ranges — multiple passes multiply GC pressure.
- **LL-011** Tauri v2 asset protocol — 4 things needed for image previews: protocol-asset Cargo feature, assetProtocol enable+scope in tauri.conf.json security, `http://asset.localhost` in CSP `img-src` AND `connect-src`, `https:` in CSP `img-src`.
- **LL-012** `posAtDOM` unreliable for replacement widgets. Use `posAtCoords({x, y})`.
- **LL-013** `getCursorColumn` pipe-counting bug — count pipes before cursor; if line starts with `|`, subtract 1.
- **LL-014** **Three Strikes** — fix from root, not patch. After 3 distinct attempts, stop and find root cause.
- **LL-015** Always test production before chasing dev-mode performance. Tauri v2 + Vite + WebView2 + DevTools dev mode adds **~37 s per IPC** on the test hardware. Production `.exe` boot was 1 s UI / 8 s fully responsive where dev mode showed 25 s / 136 s.
- **LL-016** Cache at the call site when callers are unknown. `load_all_libraries` was hit 50+ times per boot from many code paths; in-callee cache (keyed on active universe path, invalidated by `save_libraries` + `set_active_universe`) covers all known and future callers.
- **LL-017** When patching fails, spawn adversarial expert agents. Three personas in parallel (Obsidian-internals, Tauri/Rust systems, PKM generalist), each tasked to attack the proposed fix. Produces concrete numerical acceptance criteria.
- **LL-018** **Paint-First UI** — never gate first paint on IPC. `appReady = true` synchronously at top of `initializeApp`; data loads as fire-and-forget that populates reactive stores progressively.
- **LL-019** **PIXI v8 + Tauri CSP** — `import 'pixi.js/unsafe-eval'` as side-effect before any PIXI class. PIXI v8 generates WebGL shaders via `new Function(...)`; default Tauri CSP blocks it; PIXI catches the throw and leaves an empty black canvas with no visible error. **Never relax app-wide CSP** to fix a single library — use the library's no-eval variant.
- **LL-020** **Wall-vs-server-time** decides where the bottleneck is. Always instrument **both** frontend `performance.now()` around `invoke()` AND Rust per-phase `Instant::now()` checkpoints in the response. If `wall_ms >> sum(server_timings_ms)`, fix is reordering / de-paralleling. If `wall_ms ≈ sum(server_timings_ms)`, fix is in the Rust handler. **Corollary**: covering indexes on row-stores (SQLite is a row store; narrow projections on wide tables benefit 100–1000×).
- **LL-021** **Five-stamp IPC diagnostic** + `perf_trace` arrival tracer. Five stamps: `invoke_start_unix_ms`, `server_start_unix_ms`, per-phase `Instant::now()`, `server_return_unix_ms`, `client_recv_unix_ms`. Methodology: Stage 1 stamps → Stage 2 plausible patches (stop after 2 fail) → Stage 3 cheap falsifiers (`{#if false}` gates, JS heartbeat) → Stage 4 dispatcher tracer → Stage 5 named-culprit conversion. Closed Criterion 2 at `core_queue_ms = 4 ms`, `hydrated_ms = 811 ms`. **Keep every diagnostic instrument permanently.**
- **LL-022** Always-mounted UI = always-running IPC. `*EverOpened` lazy-mount pattern. CSS `display: none` hides a component but does NOT prevent its `onMount` / mount-time `$effect` from firing. Reset `*EverOpened` flags on context switch (Universe/account/vault).
- **LL-023** Don't regress working features. **4-step verification** after any guard/refactor: render path → event path → state path → data path. **Any `$effect` that overwrites user intent** (auto-close, auto-reset, auto-navigate) is high-risk. The Tier-1 panel-placement regression (Backlinks tab silently reset to Properties) is the case that earned this rule.

---

## 12. Documentation drift log (known stale documents)

When in doubt, **trust the code first, the most recent session log second, this orientation doc third**. Then fix the drift in the same commit.

| Doc | Drift |
|---|---|
| [`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) | Last updated 2026-03-31. Lists ~80 commands; actual registry in [`lib.rs:233-432`](src-tauri/src/lib.rs:233) is ~120. Missing newer commands (cache_boot_snapshot_*, sky_backfill, arabic::overrides::*, canonical::*, perf_trace::*, etc.). |
| [`docs/CE-spec.md`](docs/CE-spec.md) | Body sections + progress table at line 862-878 are stale. Body says Phases 4 + 7 + 12-16 not started; roadmap and code show 1–11 done. Trust the roadmap. |
| [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) | Says `cid` in frontmatter; code uses `cid_cn` namespace via `ensure_cid_cn` — see §6.1. |
| [`docs/Constellation-Editor-Spec.md`](docs/Constellation-Editor-Spec.md) | Describes a custom-built-from-scratch editor never built. CodeMirror 6 was used. Doc remains aspirational. |
| `lab/reports/MIG-006-WIKILINK-CASCADE.md:165-167` | The §3 plan I wrote falsely claimed an existing prop-change handler did `view.dispatch({changes:...})`. No such handler existed. §115 had to build it, which produced BUG-015. |
| Audit-agent count inconsistency | `lab/audit-agents.md` lists 7 (PA/AA/MA/SCA/RA/UXA/CQA); NotePane spec adds EA = 8; `docs/AUDIT-SYSTEM.md` adds LA/SIA/SA/DIA/CFS/OGA = 14. The "14" is the umbrella; `lab/audit-agents.md` has not been updated to match. |
| **CE Rule 8 audit-pending list** | `bases.rs` (read-time `query_base`), `dataview.rs` (read-time `execute_dataview_query`), `lenses.rs` (hybrid: definitions write-time, results read-time on `apply_lens`), `Constellation Map` (`map.rs::constellation_map_universe` walks filesystem on every open). Sky View is now write-time after MIG-001. |
| **No frontend test harness** | No vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`. Rust-side tests only: `cascade_walker_tests` (11 in libraries.rs), 6 in canonical.rs, 4 in file_kinds.rs. |
| **No help topic for Constellation Map** | Sky View has [`docs/help.uConstellation.World/Sky View/Sky View.md`](docs/help.uConstellation.World/Sky%20View/Sky%20View.md). Constellation Map has none. |
| Versioning drift | All three (`package.json`, `tauri.conf.json`, `Cargo.toml`) at 0.3.4 today after release.yml workflow synchronizes them. |

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
| Why Constellation exists / vision | [`docs/Constellation — Concept Paper.md`](docs/Constellation%20—%20Concept%20Paper.md) |
| Living Link philosophy + 8 properties + 7 types + 6 lifecycle stages | [`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md) |
| Cognitive Engine 16-phase spec | [`docs/CE-spec.md`](docs/CE-spec.md) + [`docs/cognitive-engine-roadmap.md`](docs/cognitive-engine-roadmap.md) |
| Canonical filename format + 12 kinds + import pipeline | [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) |
| NotePane (eNotePane) editor rules | [`docs/NotePane-spec.md`](docs/NotePane-spec.md) |
| Audit system (7 / 8 / 14 agents — see §7.3) | [`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) + [`lab/audit-agents.md`](lab/audit-agents.md) |
| Migration four-phase workflow | [`.claude/skills/migration.md`](.claude/skills/migration.md) |
| PCS protocol (commit/push/SO) | [`docs/PCS-PROTOCOL.md`](docs/PCS-PROTOCOL.md) |
| Working protocols / Tutorial Rule | [`docs/WORK-BEHAVIOR.md`](docs/WORK-BEHAVIOR.md) |
| Hard-won rules from real bugs | [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) (LL-001 → LL-023) |
| Migration plans | `lab/reports/MIG-NNN-*.md` |
| Active boot-perf budget + criteria | [`lab/boot-perf/BOOT-BUDGET.md`](lab/boot-perf/BOOT-BUDGET.md) |
| What's in flight today | `lab/reports/SESSION-LOG-{latest-date}.md` |
| Subsystem status snapshot | [`lab/reports/STATUS.md`](lab/reports/STATUS.md) |
| User-facing feature docs | `docs/help.uConstellation.World/<topic>/<topic>.md` |
| Master User Manual (English) | [`docs/User Manual.md`](docs/User%20Manual.md) |
| 14 translated User Manuals | `docs/help.{lang}/User Manual.md` |
| **Tauri command registry (authoritative)** | [`src-tauri/src/lib.rs:233-432`](src-tauri/src/lib.rs:233) (`generate_handler!` block) |
| Tauri config / windows / CSP | [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) |
| Window permissions | [`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json) |
| Release workflow (CI) | [`.github/workflows/release.yml`](.github/workflows/release.yml) |
| Bases (database views) MVP | [`docs/BASES_MVP_SPEC.md`](docs/BASES_MVP_SPEC.md) |
| eNotePane build history | `docs/eNotePane-development-record.md` + `lab/experiments/phase-N-*.md` |
| Forensic snapshots from incidents | `lab/forensics/` |

---

## 15. Session-start protocol (for any new Claude session)

Mandated by `docs/WORK-BEHAVIOR.md` §1 + Standing Order #6:

1. **`git pull origin main`** to sync from any other device/session.
2. **`git log --oneline -10`** to see recent work.
3. **Read `lab/reports/SESSION-LOG-{latest-date}.md`** — pick up where the last session left off. Look for `§STATE-OF-STANDING` blocks.
4. **Read THIS document** (`docs/Constellation Orientation & Onboarding vX.Y.md`) — get architectural fluency in one read.
5. **Read [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md)** — every rule was earned by a real bug.
6. **Read [`CLAUDE.md`](CLAUDE.md)** — top-principal rules + Working Agreement + Standing Orders.
7. **Read [`lab/reports/STATUS.md`](lab/reports/STATUS.md)** — one-page subsystem status index.
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

**Bump version (1.0 → 1.1, etc.)** when the section structure changes. **Rename the file in the same commit** — the filename always carries the version suffix per §10.12. Date-stamp every section that updates with the date of update.

The document **must remain readable in one pass.** If it grows past ~1500 lines, split into linked sub-documents kept in `docs/orientation/`.

---

## 17. What I (Claude) have NOT read in detail

This list is mandated by the BASIC RULE. It exists so future readers know exactly which claims in this doc are grounded in direct code inspection vs. design-doc inference. If you need certainty on a claim that touches an "unread" file, **read it before acting**.

**Source code I have NOT read in full** (as of v1.1):
- [`src-tauri/src/search.rs`](src-tauri/src/search.rs) (224 KB) — only sections via grep + alias plumbing + trigger blocks (lines 720-767, 867-1245).
- [`src-tauri/src/libraries.rs`](src-tauri/src/libraries.rs) (172 KB) — selected functions only; cascade walker tests confirmed.
- `src-tauri/src/cache.rs` — `read_sky_links_raw` + `cache_boot_snapshot_sky` confirmed; rest read at API level only.
- `src-tauri/src/canonical.rs` — `cid_cn` rationale + 12 kinds + first/last 100 lines read; middle sections by API.
- `src-tauri/src/universe.rs` — migration commands read; rest at API level.
- `src-tauri/src/arabic/*` — module doc + entry points read; per-layer internals at design-spec level.
- `src-tauri/src/lexicon/*` — module list read; internals at design-spec level.
- `src-tauri/src/inspector360.rs`, `lens.rs`, `boot_bundle.rs`, `sky_backfill.rs`, `tasks.rs`, `ai/mod.rs`, `embeds.rs` — registered in lib.rs, internals not read.
- Most Svelte components: `Inspector360`, `ConstellationMap`, `GraphMindView`, `ConstellationSight`, `ConstellationSight2`, `IndexPanel`, `OrgChart`, `SearchHub`, `SecondScreenPage`, `SettingsModal`, `BaseView`, `FullSkyView`, `SenseMakingCanvas`, `DashboardView`, `UniverseSetup`, `BacklinksPanel`, `OutgoingLinksPanel`, `PropertyEditor` — **none read in full**.
- `src/lib/libraries/store.ts` — sections only.
- `src/lib/i18n/*.json` (15 locales) — `en.json` top 30 lines sampled; others not read.
- `src/lib/editor/*.ts` extensions — module list + livePreview pre-cache pinpointed; per-plugin code at API level.
- `src/routes/+layout.svelte` (6872 lines) — only via LL-002 / LL-022 / LL-023 anecdotes + the 2026-03-27 cascade analysis. **Not read end-to-end.**
- `src/routes/libraries/+page.svelte` (704 lines) — not read.
- `src/routes/skills/+page.svelte` (219 lines) — not read.

**Docs I have NOT read in full**:
- `docs/User Manual.md` — only TOC + first 100 lines.
- 23 of 24 help topics — only headlines (Cognitive Engine + Sky View read in detail).
- All 14 translated User Manuals.
- `docs/BASES_MVP_SPEC.md` — only first 120 of ~330 lines.
- Binary docs (`docs/Constellation_Lens_Concept_Paper_Eisa.pdf`, `docs/GraphMind*.docx`, `docs/constellation_cognitive_engine_v2.1.pdf`) — not read.

**Session logs partially read** (full chronological set is 20 files, 2026-03-27 → 2026-04-25):
- 2026-03-27: top of file read — Phase 0 BLOCKING-001 cascade discovery.
- 2026-04-18 (1.46 MB): structural headlines only.
- 2026-04-19 (99 KB): structural headlines + first 100 lines.
- Most others: sampled by date headlines, not read end-to-end.

**Specifics I do NOT know**:
- The full set of badge-letter meanings beyond T = Title, C = Content, P = Property, S = Semantic.
- The full list of letters used as filter chips in the Constellation Map left panel.
- Whether `memmap2 = "0.9"` is wired through to the FST baker yet (M9 follow-on item).
- The exact alias/identifier collision tiebreak behavior under all edge cases.
- Whether there's a `panic-handler` plugin or just the `std::panic::set_hook` in `lib.rs::run()`.

**Future maintainers**: when you read one of the above and confirm a fact, update §17 to remove that item AND fold the verified fact into the relevant section above. Keep §17 honest.

---

*End of v1.1. Maintained per Standing Order #6.*
