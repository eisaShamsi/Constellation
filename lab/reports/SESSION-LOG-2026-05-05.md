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

## Open items (after §1A commit)
- §1B: bridge_vectors runtime loader + cosine k-NN + `ctse::resolve_term_to_concept`.
- §1C: `term_vocab.bridge_concept_id` + write-time hook + progressive backfill (first Boss-testable gate).
- §1D: query path + remove old "Rebuild Term Embeddings" Settings UI (second Boss-testable gate).
- §1E: three-agent audit per Migration Rule §4.

## Notes for next session if interrupted
- `cargo run --bin build_concept_vectors --release` is the canonical way to regenerate the asset if M11 TSV changes. Output lands at `src-tauri/src/bridge_vectors/data/concept_vectors_v1.bin`.
- The asset is committed to the repo (Boss-approved). It changes only when `lexicon_v1.tsv` does.
- The `embed_passages_standalone` helper is **the only public surface** added to `embeddings.rs`; runtime callers continue to use the private `ensure_engine` / `run_embedding_batch` path through `EmbeddingState`.
