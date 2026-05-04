# MIG-012 — Phase 2: Build Plan

**Companion to**: `MIG-012-INDEX-SEARCH-ENGINE-ARCHITECT.md`
**Phase**: 2 (Plan) — Boss-approved 2026-05-04: **Q1.A + Q2.C + Q3.B**.
**Build steps**: 8 commits + simplify checkpoint + Phase 4 audit.

---

## §0 · Reading guide

Two parallel tracks fold into a single linear cascade for sanity:

- **Track A (Build.1–5)**: Semantic search infrastructure → IPC → frontend wiring → badge.
- **Track B (Build.6–7)**: Search history table + IPCs → frontend dropdown.

Each step lands as one commit. Single Boss-test gate at **Build.7** (combined G test for both tracks). Build.8 is the simplify checkpoint.

The model is already shipped (`multilingual-e5-small`, 384 dims) — `embeddings.rs` exposes `run_embedding` and the `EmbeddingState`. Track A builds on top.

---

## §Build.1 — `term_embeddings` table + schema

**Surface**: `src-tauri/src/search.rs` (init_db).

- New table `term_embeddings (term TEXT PRIMARY KEY, embedding BLOB, dimensions INT, model_id TEXT, last_built INTEGER)`.
- Store version in `schema_versions.term_embeddings = 1`.
- Migration step (idempotent): create table if missing.

**Verification**: `cargo check` clean. Table appears on first boot of the new binary.

---

## §Build.2 — `init_term_embeddings` IPC + progress events

**Surface**: `src-tauri/src/embeddings.rs`.

- New `#[tauri::command] fn init_term_embeddings(app, force: Option<bool>) -> Result<(), String>`.
- Walks `notes_vocab` (the FTS5 vocabulary view) → for each term not yet embedded (or all when `force`), calls `run_embedding` with `passage: {term}` prefix → INSERT OR REPLACE into `term_embeddings`.
- Emits Tauri events `term-embedding-progress { processed, total }` every ~50 terms so the frontend can render a progress bar.
- Cancel-safe: a separate `cancel_term_embeddings` IPC sets a stop flag the worker checks per-term.
- Resumable: when re-fired, skips terms already in `term_embeddings` (via existence check).

**Verification**: invoke from DevTools → events fire → table populates → second invocation no-ops. Manual run on Boss's library expected to take ~10–20 min.

---

## §Build.3 — `search_terms_semantic` IPC + 4 unit tests

**Surface**: `src-tauri/src/embeddings.rs` (or a new `src-tauri/src/semantic_search.rs` if scope grows).

- New `#[tauri::command] fn search_terms_semantic(app, query: String, top_k: Option<u32>, threshold: Option<f32>) -> Result<Vec<TermSimilarity>, String>`.
- Embed the query with `query: ` prefix → load all term embeddings → cosine similarity → filter by threshold (default 0.7) → sort descending → take top_k (default 50) → return `Vec<{ term, score }>`.
- Tests: cosine math correctness; empty corpus returns empty; threshold filter works; top_k caps result.

**Verification**: 4/4 tests pass. Existing M13 + MIG-011 tests still pass.

---

## §Build.4 — Frontend wrapper + debounce + cache (mirrors MIG-011)

**Surface**: `src/lib/libraries/store.ts`, `src/lib/components/IndexPanel.svelte`.

- `store.ts`: `TermSimilarity { term: string, score: number }`; `searchTermsSemantic(query, topK?, threshold?): Promise<TermSimilarity[]>`.
- `IndexPanel.svelte`: new prop `semanticSearchEnabled?: boolean`. New state `semanticMatches: $state<Map<string, number>>(new Map())` (term → score). Debounced effect on `filterQuery` (300ms) calls IPC when toggle is on. Cache + cancel-token per MIG-011 pattern.

**Verification**: Manual: toggle on, type, observe IPC fires once per ~300ms pause. svelte-check clean.

---

## §Build.5 — Filter loop + semantic badge

**Surface**: `IndexPanel.svelte` — extend `filteredResult`.

- For each entry, after substring + bridge checks, if entry.term is in `semanticMatches` → include with `≈` indicator.
- Annotation Map gains a `semanticAnnotations: Map<term, score>`.
- Render: small ` ≈ ` chip after term name, distinct from `via {lemma}` (different style).
- Sort: direct → bridge → semantic, within frequency.

**Verification**: Boss-testable but rolled into Build.7 gate (combined with history).

---

## §Build.6 — Search history table + 3 IPCs

**Surface**: `src-tauri/src/search.rs` (init_db) + new IPC file.

- New table `index_search_history (id INTEGER PRIMARY KEY, query TEXT, last_used INTEGER, use_count INTEGER DEFAULT 1)`. Per-Universe (in the same `search.db`).
- Three IPCs:
  - `read_index_history(limit: Option<u32>) -> Result<Vec<HistoryEntry>>` — returns `{ query, last_used, use_count }`, ORDER BY last_used DESC LIMIT N (default 20).
  - `write_index_history_entry(query: String) -> Result<(), String>` — UPSERT (INSERT OR REPLACE bumping use_count + last_used). FIFO eviction at 200 rows.
  - `clear_index_history() -> Result<(), String>` — DELETE FROM table.

**Verification**: cargo check. Table populates on first call. Second call returns history. Clear empties.

---

## §Build.7 — Frontend history dropdown + Settings toggles (G gate)

**Surface**: `IndexPanel.svelte` (history dropdown), `SettingsModal.svelte` (toggles), `store.ts` (wrappers), 15 locales (new keys).

- Settings → Index gains:
  - `Semantic search` toggle (default off).
  - `Search history` toggle (default off).
  - `Clear search history` button (with confirm dialog).
- IndexPanel:
  - On filter-box focus + empty query: show history dropdown if toggle on.
  - Down-arrow cycles entries.
  - Click entry → fills filter box.
  - Right-click entry → "Remove entry" / "Clear all".
  - On query commit (Enter / blur with non-empty), call `writeIndexHistoryEntry`.
- AppSettings: `index.semanticSearchEnabled: boolean`, `index.searchHistoryEnabled: boolean`.
- i18n: new keys in 15 locales for Settings labels + dropdown affordances + semantic badge.

**Boss G test gate**: end-to-end verification.

---

## §Build.8 — `/simplify` checkpoint

Combined-lens review (per MIG-011 pattern). Address Tier 1 + worthwhile Tier 2.

---

## §Audit — Phase 4

`MIG-012-AUDIT.md`. 12 invariants S1–S12 from Architect doc verified.

---

## §X · Boss approval gates

| Gate | What | When |
|---|---|---|
| **G1** | Plan-Approval = Build-Approval (Boss already said "go A C B"). | NOW. |
| **G2** | End-to-end test (semantic + history both visible, working, RTL clean). | After Build.7. |
| **G3** | Closure. | After Audit. |

Cascading immediately. Stop only at G2.

---

**Phase 2 closes. Cascading to Build.1.**
