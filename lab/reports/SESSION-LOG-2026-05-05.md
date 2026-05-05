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

## State-of-standing — end of session 2026-05-05

### Shipped today (verified-shipped, protected)
- `5e1c0f1` MIG-013 §1A — 30 MB concept-vector asset baked at build time. 20,000 concepts × 384 f32, all L2-normalized, magic+count+dim verified. Build helper at `src-tauri/build_assets/build_concept_vectors.rs`. Throughput 1008 passages/sec on 24 threads. M11 zero-diff invariant: `git diff src-tauri/src/lexicon/` empty.
- `909e381` MIG-013 §1B — runtime asset loader (`bridge_vectors::asset` + `bridge_vectors::store`) and adapter (`ctse::resolve_term_pure` + `ctse::resolve_term_to_concept`). 10 tests all pass: 5 store unit + 1 asset round-trip + 4 adapter (real M11). Fast-path resolution of `book/En` confirmed (panicking-on-call slow-path closure never fired). Cosine k-NN over 20K × 384 matrix; small-k via sorted Vec.

### At-risk / in-flight / uncommitted
- **None.** Working tree is clean (only the two `lab/reports/` doc additions ride along with `lib.rs` visibility changes; all included in commits above).

### Known-broken / pre-existing
- Boss's library term_vocab population path. `populate_term_vocab` (embeddings.rs:867) was committed at `e87adeb` (Phase 1) but its only caller — `init_term_embeddings` — is being retired in §1C. This means after §1C, `term_vocab` is filled only incrementally via reindex hooks (not bulk-populated on first boot). Boss's existing 7,635-note library will start with an empty `term_vocab` after §1C lands. New saves grow it. This is consistent with CLAUDE.md Rule 8 (Write-Time Derivation) but means the §1D Boss-test will show progress only on edited notes, not as a one-shot bulk strip. **Recommendation**: surface this in §1D Boss-test tutorial and/or add a one-time "Index this library now" trigger.
- 23 pre-existing dead-code warnings in the lib (none introduced by §1A/§1B; visible in build logs).

### Pending (not started)
- **§1C** (Rust-side only — re-scoped from approved Plan, recorded here for transparency):
  1. `search.rs` schema migration: add `bridge_concept_id TEXT` column to `term_vocab`; gate via new `schema_versions` row `('term_vocab_bridge', 1)`. Pattern at search.rs:1047–1062 (sky module) is the reuse target.
  2. `ctse/hooks.rs` (new) — `pub fn on_note_indexed(conn, app, note_path, body_text)` — re-tokenize via `fts5_tokenizer::tokenize_to_vec`, upsert into `term_vocab` with incremental count maintenance, **fast-path-only** resolve `bridge_concept_id`. No ONNX in write path (gotcha #2 from §1C scout).
  3. Wire `on_note_indexed` into `reindex_single_note` (search.rs:3890) **post-COMMIT**.
  4. `ctse/backfill.rs` (new) — Tauri command `ctse_run_backfill(library_name)` that walks `WHERE bridge_concept_id IS NULL ORDER BY total_count ASC LIMIT 500`, slow-path-resolves per term, commits per batch, emits `ctse-backfill-progress` events. Resumable (idempotent NULL-walk). Sentinel `'-'` for "tried and failed" so successive runs don't re-attempt.
  5. Remove `init_term_embeddings` Tauri command (embeddings.rs:516–752), deregister from lib.rs:379, remove `term_embeddings` table from `init_db` (search.rs:1165).
  6. Remove `populate_term_vocab` (embeddings.rs:867) once its single caller is gone.
- **§1C frontend** (deferred to §1D for atomicity): SettingsModal "Rebuild Term Embeddings" button + `confirmDialog` + `termEmbedProgress` UI removal; `termEmbedProgress` writable store removal.
- **§1D**: auto-trigger backfill on app boot if NULL rows present; status-bar progress strip; `ctse_search_by_concept` Tauri command + frontend wiring; full Settings UI cleanup. **First Boss-testable gate** (cross-language semantic search).
- **§1E**: three-agent audit (invariants, drift, migration-path).

### Documentation drift
- `docs/Constellation Orientation & Onboarding v1.34.md` does not yet mention CTSE Bridge Adapter (the v1 user-corpus pipeline was never added either). Will be added when §1D lands and CTSE becomes user-visible — bumping to v1.35 in the same commit per Standing Order.
- User Manual + 14 translations: no §1A/§1B content needed yet (silent commits). Drafts queued for §1D.

### Open architectural questions for next session
1. **Per-note hook tokenization cost**: `tokenize_to_vec` on a 5000-word note runs in ~5 ms (acceptable). For pathological notes (>50K words), it could be slower. Add a body-cap (e.g., 1 MiB) per the Phase 1 precedent? Yes — recommend matching the existing `BODY_CAP_BYTES` constant from the original Phase 1 commit.
2. **Cancellation**: backfill needs a cancel flag. The existing `EmbeddingState.term_embed_cancel: AtomicBool` (embeddings.rs:22) is being orphaned by `init_term_embeddings` removal; reuse it as `ctse_backfill_cancel` instead of creating a parallel atomic.
3. **What happens to the `populate_term_vocab` source code?** Two options: (a) delete entirely (simplest, can be resurrected from git if needed for a future bulk-bootstrap feature); (b) keep as a private fn and add a new Tauri command `ctse_populate_term_vocab` that exposes it for an explicit Boss-triggered "Index this library now" button. **Recommendation: defer the choice to §1D when the Settings UI is being touched anyway.**

### How to resume next session
1. Read `lab/reports/MIG-013-CTSE-ARCHITECT-v2.md`, `MIG-013-CTSE-PLAN.md`, this session log.
2. Run `git log --oneline -6 main` and confirm `909e381` is the latest CTSE commit.
3. Verify M11 invariant still holds: `git diff src-tauri/src/lexicon/` should be empty.
4. Pick up §1C item 1 (schema migration via `schema_versions`).
5. Cascade through §1C → §1D → §1E with no mid-cascade approval; Boss test fires at §1D.

## Notes for next session if interrupted
- `cargo run --bin build_concept_vectors --release` is the canonical way to regenerate the asset if M11 TSV changes. Output lands at `src-tauri/src/bridge_vectors/data/concept_vectors_v1.bin`.
- The asset is committed to the repo (Boss-approved). It changes only when `lexicon_v1.tsv` does.
- The `embed_passages_standalone` helper is **the only public surface** added to `embeddings.rs`; runtime callers continue to use the private `ensure_engine` / `run_embedding_batch` path through `EmbeddingState`.
