# Session Log — 2026-05-05

## Context
Continuation of MIG-013 CTSE work. Yesterday (2026-05-04) shipped Phase 1 (parallel vocab bootstrap) and Phase 1.5 (batched ONNX inference for term embeddings). Today's session opened with Boss reporting the Phase 1.5 build was *worse* — stuck at "Building vocabulary…" with total=0 for 7+ minutes. The slow `SELECT path, body_text FROM note_meta` was identified as the freeze point (heartbeat thread spawned after the SQL bulk-fetch).

## Pivot — three-SME audit + Architect v2

Boss directive: "audit this fix as we did before. Create Subject Matter Experts agents who will work as X-Ray and Public Library techniques programmers."

Spawned three parallel SME agents (parallel-systems, library/IR, application-architecture). All three converged on the same finding: **the per-library term-embedding pipeline is solving the wrong problem**. The canonical IR pattern (LCSH/MeSH/AAT since 1909) is to embed the controlled vocabulary, not the patron's corpus. M11 Lexical Bridge is exactly that controlled vocabulary.

Synthesized into three options for Boss; Boss picked **Option A** ("Pivot to M11 Bridge Adapter") with the constraint **"don't touch the M11's ~20K concepts"**.

## Architect v2 + Plan

- `lab/reports/MIG-013-CTSE-ARCHITECT-v2.md` — bridge-adapter architecture; M11 zero-diff invariant; sub-engines remapped (Sampler dropped); §5 retires the `init_term_embeddings` / `term_embeddings` table / "Rebuild Term Embeddings" UI.
- `lab/reports/MIG-013-CTSE-PLAN.md` — five landable commits (1A–1E). Each verifies M11 zero-diff via `git diff src-tauri/src/lexicon/` returning empty.

Boss approved the Plan and the "commit the 30 MB" decision.

## §1A — Build-time concept-vector pipeline

Discarded the half-finished fix-10 edit in `embeddings.rs` (architect v2 §5) before starting clean.

### What landed
- `src-tauri/src/lib.rs` — visibility widened: `arabic`, `embeddings`, `lexicon` flipped from `mod` → `pub mod` (purely additive); new `pub mod bridge_vectors`.
- `src-tauri/src/embeddings.rs` — added `pub fn embed_passages_standalone(model_path, tokenizer_path, texts, intra_threads, batch_size)`. Builds its own ONNX session (no AppHandle) and chunks through the existing `run_embedding_batch` pipeline. No behavioral change to runtime engine.
- `src-tauri/src/bridge_vectors/mod.rs` — stub with `ASSET_MAGIC = b"CTSEBV01"` and `VECTOR_DIM = 384`. Layout fully documented inline; runtime loader lands in §1B.
- `src-tauri/build_assets/build_concept_vectors.rs` — offline `[[bin]]` that reads M11's TSV (read-only via `lexicon::parse`), picks one canonical surface form per concept (en > zh > es > fr > de > … fallback chain), embeds in batches of 128 with `intra_threads = available_parallelism()`, and writes the asset.
- `src-tauri/Cargo.toml` — `[[bin]] name = "build_concept_vectors"` target.

### Verification (§1A test gate, automated — no Boss test required at this phase)

| Check | Result |
|---|---|
| Concepts parsed from M11 TSV | **20,000** ✓ |
| Per-language coverage | En=20000 (100% English coverage; fallback chain never fired) |
| Embedding throughput | **1,008.5 passages/sec** on 24 threads |
| Embed time (20K concepts) | **19.8 s** |
| Total run time | **20.1 s** |
| Asset size | **29.6 MB** (target 30–35 MB) ✓ |
| Header magic | `CTSEBV01` ✓ |
| Header count / dim | 20000 / 384 ✓ |
| First concept id | `c:12-angry-men` (matches TSV sample) ✓ |
| All vectors L2-normalized | yes (per-vector `0.99 ≤ ‖v‖ ≤ 1.01` check passed) ✓ |
| **M11 zero-diff invariant** | `git diff src-tauri/src/lexicon/` → **empty** ✓ |
| Binary size baseline | 55.6 MB (release). Post-1A asset adds ~30 MB → ~85 MB est. final binary. |

### Known good state at end of §1A
- `cargo run --bin build_concept_vectors --release` succeeds cold and produces the asset deterministically.
- 23 pre-existing dead-code warnings in the lib (not introduced by §1A).
- Working tree before commit:
  - Modified: Cargo.toml, embeddings.rs, lib.rs
  - New: bridge_vectors/, build_assets/, lab/reports/MIG-013-CTSE-ARCHITECT-v2.md, lab/reports/MIG-013-CTSE-PLAN.md, lab/reports/SESSION-LOG-2026-05-05.md

## §1B — Runtime loader + bridge adapter

### What landed
- `src-tauri/src/bridge_vectors/asset.rs` — `parse()` over `include_bytes!("data/concept_vectors_v1.bin")`. Parses header + concept-id table + matrix into an owned `Box<[f32]>` (deliberately copies — `include_bytes!` returns `&'static [u8]` at unspecified alignment, and reinterpreting unaligned bytes as `&[f32]` is UB on strict-alignment targets).
- `src-tauri/src/bridge_vectors/store.rs` — `ConceptVectorStore` with `nearest_concept` and `nearest_concepts_k` (cosine over flat row-major matrix; small-k uses a sorted Vec instead of BinaryHeap for cache friendliness).
- `src-tauri/src/bridge_vectors/mod.rs` — `pub fn get() -> &'static ConceptVectorStore` singleton via OnceLock.
- `src-tauri/src/ctse/mod.rs` — Bridge Adapter:
  - `resolve_term_pure(graph, store, embed_query, term, lang, threshold)` — pure dependency-injected core; closure invoked only when M11 fast path misses.
  - `resolve_term_to_concept(app, term, lang)` — Tauri-context wrapper; pulls singletons + delegates query embed to `embeddings::constellation_embed_text`.
  - `DEFAULT_THRESHOLD = 0.78` (initial guess; tunable in §1D).
- `src-tauri/src/lib.rs` — `pub mod ctse;` registration.

### Verification (§1B test gate, automated)

| Test | Result |
|---|---|
| `bridge_vectors::store::nearest_concept_returns_exact_match_with_score_one` | ok |
| `bridge_vectors::store::nearest_concept_rejects_wrong_dim` | ok |
| `bridge_vectors::store::nearest_concept_zero_query_returns_zero_score` | ok |
| `bridge_vectors::store::top_k_returns_descending_scores` | ok |
| `bridge_vectors::store::top_k_clamps_to_count` | ok |
| `bridge_vectors::asset::baked_asset_parses` (real 30 MB asset) | ok |
| `ctse::fast_path_resolves_known_lemma_without_calling_slow_path` | ok — `book/En` resolved via M11; panicking closure never fired |
| `ctse::slow_path_zero_query_returns_none_below_threshold` | ok |
| `ctse::slow_path_above_one_threshold_rejects_everything` | ok |
| `ctse::slow_path_zero_threshold_always_returns_some` | ok |
| **M11 zero-diff invariant** | `git diff src-tauri/src/lexicon/` → **empty** ✓ |
| `cargo build --release` (lib) | ok, 1m 31s, 0 new warnings |

### Key implementation notes for next session
- `LexiconGraph.nodes` is `pub Vec<LemmaNode>` and `LemmaNode.concept_id` is `pub String`. The fast path indexes directly: `graph.nodes[node_idx as usize].concept_id`. No new lexicon API surface required.
- The asset is parsed once via OnceLock and the matrix lives on the heap (~30 MB) for f32-alignment correctness. `include_bytes!` zero-copy was rejected as UB-prone.
- The `ctse::resolve_term_pure` test pattern (panicking closure on fast-path tests) is the right reuse target for §1C — verifies fast-path coverage without burning ONNX cycles.

## §1C — write-time hooks + slow-path backfill scaffold

### What landed
- **Schema migration** (`search.rs`): new `TERM_VOCAB_BRIDGE_SCHEMA_VERSION = 1` constant and `ensure_term_vocab_bridge_column()` helper. Gated via `schema_versions` row `('term_vocab_bridge', 1)`. Adds `bridge_concept_id TEXT` (nullable) plus `idx_term_vocab_bridge_concept_id` index. Idempotent — fresh DBs and existing DBs both end up with the column. Pattern mirrors the sky / note_meta MIG-002 / MIG-003 helpers.
- **Write-time hook** (`ctse/hooks.rs`, NEW):
  - `on_note_indexed(conn, path, old_body, new_body)` — tokenizes both bodies via `fts5_tokenizer::tokenize_to_vec`, computes signed per-term `(total_delta, doc_delta)`, applies INSERT/UPDATE to `term_vocab`, and fast-path-resolves M11 concept ids for newly-inserted terms only.
  - `on_note_deleted(conn, path, body)` — subtracts a deleted note's contributions; rows that drop to zero are kept as tombstones with `bridge_concept_id` preserved (revival on next save is free).
  - Stopword set is cached at module level via `OnceLock` (`crate::libraries::build_stopwords()`).
  - `BODY_CAP_BYTES = 1 MiB` (matches the prior `populate_term_vocab` precedent).
  - **No ONNX in the write path** — fast-path-only. Slow-path resolution is deferred to the backfill (§1C-4).
- **Wire-in** (`search.rs::reindex_single_note` + `reindex_delete_note`): both wrappers now query `note_meta.body_text` once before the index_note write and once after, then call the appropriate hook with `old_body`/`new_body`. Hook errors are logged but never fail the reindex (term_vocab is a derived view; the file + note_meta are the sources of truth).
- **Slow-path backfill** (`ctse/backfill.rs`, NEW):
  - Tauri commands: `ctse_run_backfill`, `ctse_cancel_backfill`, `ctse_backfill_status`.
  - Walks `WHERE bridge_concept_id IS NULL ORDER BY total_count ASC, term LIMIT 500` (TF-IDF descending = rarest first; search becomes useful early in the backfill).
  - Per-term resolution via new `ctse::resolve_term_multilang(app, term)`: 15-language fast-path FST sweep, then slow-path e5 inference + cosine k-NN. Bigrams skip slow path.
  - Sentinel `'-'` for "tried and failed" so re-runs visit only genuinely-new NULL rows.
  - Resumable: each batch commits in its own transaction. App close mid-fill leaves a partial-but-correct state.
  - Cancellation reuses `EmbeddingState.term_embed_cancel: AtomicBool` (orphaned by §1C-5; per gotcha #2 of last session).
  - Emits `ctse-backfill-progress` events with `{processed, total, done, cancelled}`.
- **Shared multi-lang helper** (`ctse/mod.rs`):
  - New `pub fn fast_path_concept_id(graph, term)` — single multi-language lookup helper used by both the hook and the backfill. Returns `None` for bigrams without trying any language.
  - New `pub fn resolve_term_multilang(app, term)` — fast-path-then-slow-path resolver for the backfill.
- **Retired surfaces** (`embeddings.rs`):
  - Deleted `init_term_embeddings`, `cancel_term_embeddings`, `search_terms_semantic`, `term_embedding_status` Tauri commands.
  - Deleted `populate_term_vocab` (Phase 1 rayon bootstrap) and `blob_to_vec` (orphaned with `search_terms_semantic`).
  - Deleted `TermEmbedProgress` and `TermSimilarity` payload structs.
  - Removed orphaned `use std::sync::atomic::Ordering` and `Emitter` imports.
  - Kept: `EmbeddingState.term_embed_cancel` (reused by `ctse::backfill`), `vec_to_blob` (still used by `constellation_embed_notes`), `embed_passages_standalone` (used by §1A build helper), the runtime engine pipeline.
- **Schema cleanup** (`search.rs`): `term_embeddings` CREATE TABLE removed. The comment on `term_vocab` updated to reference §1C's write-time-derivation maintenance path. Existing DBs may still carry the old `term_embeddings` table — left in place (harmless dangling table). The MIG-013 §5 §retired-table-GC item is queued for a future cleanup pass.
- **lib.rs**: deregistered the four old commands; registered the three new `ctse::backfill::*` commands. The frontend store still references the old IPCs (`initTermEmbeddings`, etc.) — those throw at runtime if invoked, but §1D's Settings cleanup removes the call sites.

### Verification (§1C test gate, automated)

| Check | Result |
|---|---|
| `cargo build --release --lib` | ok, 1m 50s, 22 warnings (all pre-existing) |
| `cargo test --lib --tests ctse` | **9/9 ok** |
| · `ctse::tests::fast_path_resolves_known_lemma_without_calling_slow_path` | ok |
| · `ctse::tests::slow_path_zero_query_returns_none_below_threshold` | ok |
| · `ctse::tests::slow_path_above_one_threshold_rejects_everything` | ok |
| · `ctse::tests::slow_path_zero_threshold_always_returns_some` | ok |
| · `ctse::hooks::tests::first_index_inserts_and_fast_path_resolves` | ok |
| · `ctse::hooks::tests::idempotent_resave_yields_no_delta` | ok |
| · `ctse::hooks::tests::edit_applies_signed_delta` | ok |
| · `ctse::hooks::tests::delete_subtracts_and_tombstones` | ok |
| · `ctse::hooks::tests::bigram_tokens_stay_null_after_fast_path` | ok |
| `cargo test --lib --tests bridge_vectors` | **6/6 ok** (no regression) |
| **M11 zero-diff invariant** | `git diff src-tauri/src/lexicon/` → **empty** ✓ |

### Implementation notes for §1D
- `term_vocab` starts empty after §1C lands on Boss's existing library — `populate_term_vocab` is gone, and the write-time hook only grows the table on saves. §1D needs **either** an auto-trigger that walks `note_meta.body_text` on first boot when `term_vocab` is empty (re-fires `on_note_indexed` for each row) **or** an explicit "Index this library now" Settings action. Recommendation: do both — auto-fire on first boot (silent, status-bar strip), keep an explicit action for power users.
- The `ctse_run_backfill` Tauri command is registered but no auto-trigger fires it yet. §1D adds a boot-time check: if `ctse_backfill_status > 0` and a small flag (e.g., a settings or schema_versions row) shows we haven't auto-fired this Universe, dispatch the command.
- The frontend SettingsModal still has live calls to the four removed Tauri commands. Until §1D lands, toggling semantic search ON in Settings throws at runtime on the first IPC. Acceptable in a cascade — no Boss test happens between §1C and §1D.
- `ctse_search_by_concept` is the §1D query-path command. Pattern: embed query → top-k concept ids via cosine on the 20K matrix → SQL `JOIN term_vocab ON bridge_concept_id IN (...)` → `notes_fts MATCH` for snippets.

## State-of-standing — end of session 2026-05-05 (revised after §1C)

### Shipped today (verified-shipped, protected)
- `5e1c0f1` MIG-013 §1A — 30 MB concept-vector asset baked at build time. 20,000 concepts × 384 f32, all L2-normalized, magic+count+dim verified.
- `909e381` MIG-013 §1B — runtime asset loader + Bridge Adapter API + 10 tests.
- **(this session)** MIG-013 §1C — schema migration `term_vocab.bridge_concept_id`; write-time hook (`ctse/hooks.rs`); slow-path backfill (`ctse/backfill.rs`) with `ctse_run_backfill`/`ctse_cancel_backfill`/`ctse_backfill_status` Tauri commands; retirement of `init_term_embeddings` / `cancel_term_embeddings` / `search_terms_semantic` / `term_embedding_status` / `populate_term_vocab` / the `term_embeddings` CREATE TABLE. 9 ctse tests + 6 bridge_vectors tests green; M11 zero-diff invariant intact.

### At-risk / in-flight / uncommitted
- **Frontend SettingsModal still calls the four removed Tauri commands.** Toggling semantic search ON in Settings throws at runtime on the first IPC. Resolved by §1D's full Settings cleanup. No Boss test happens in this gap.
- **Existing libraries' `term_vocab` may have stale row data** from the prior `populate_term_vocab` bootstrap (rows with `doc_count = 0` because the bulk loader skipped that field). Cosmetic — backfill still works because the NULL filter on `bridge_concept_id` is the cursor. Ordering is by `total_count` which is correct.

### Known-broken / pre-existing
- 22 pre-existing dead-code warnings in the lib (none introduced by §1C). Two new warnings appeared and were resolved by removing the orphaned `Emitter` import.

### Pending (not started)
- **§1D** (next): auto-trigger first-fill on boot when `term_vocab` is empty (walks `note_meta.body_text` and re-fires `on_note_indexed`); auto-trigger `ctse_run_backfill` on boot when NULL rows exist; status-bar progress strip subscribed to `ctse-backfill-progress`; `ctse_search_by_concept` Tauri command + frontend wiring; full Settings UI cleanup (remove the four old IPCs' call sites + the `termEmbedProgress` writable store + the `confirmDialog` for "Rebuild Term Embeddings"); update help files + User Manual to describe Constellation Sight cross-language behavior. **First Boss-testable gate** (cross-language semantic search).
- **§1E**: three-agent audit (invariants, drift, migration-path) per Migration Rule §4.

### Documentation drift
- Orientation `v1.35` covers §1A + §1B. **Bumping to `v1.36` in this commit** per Standing Order #6 — covers §1C.
- User Manual + 14 translations: still no §1A/§1B/§1C content. Drafts queued for §1D when CTSE becomes user-visible (cross-language search Boss test).

### How to resume next session
1. Read `lab/reports/MIG-013-CTSE-ARCHITECT-v2.md`, `MIG-013-CTSE-PLAN.md`, this session log (especially the §1C section above).
2. `git log --oneline -8 main` and confirm the §1C commit is the latest.
3. M11 invariant: `git diff src-tauri/src/lexicon/` should be empty.
4. Pick up §1D — start with the boot-time auto-fire decision (where to gate the first-fill check; recommend a boolean column on `schema_versions` or a sentinel value).
5. Cascade through §1D → §1E. Boss test fires at §1D once cross-language search is wired and the first-fill has run.

## Notes for next session if interrupted
- `cargo run --bin build_concept_vectors --release` is the canonical way to regenerate the asset if M11 TSV changes. Output lands at `src-tauri/src/bridge_vectors/data/concept_vectors_v1.bin`.
- The asset is committed to the repo (Boss-approved). It changes only when `lexicon_v1.tsv` does.
- The `embed_passages_standalone` helper is **the only public surface** added to `embeddings.rs`; runtime callers continue to use the private `ensure_engine` / `run_embedding_batch` path through `EmbeddingState`.
- §1C's hook uses `note_meta.body_text` as the source of truth for tokenization (NOT the file on disk) — this is intentional, because `notes_fts` was populated from the same `body_text`, so the token namespaces stay in sync. If a future migration changes the body normalization pipeline, both surfaces must be re-derived together.
