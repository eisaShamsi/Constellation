# MIG-013 — Constellation Terms Scanning Engine (CTSE) — Architect v2

**Date**: 2026-05-05
**Supersedes**: `MIG-013-CTSE-ARCHITECT.md` (v1, kept as historical record)
**Status**: Phase 1 (Architect) — Boss directive 2026-05-05 after three-SME audit converged on Option A: pivot to M11 Bridge Adapter as the primary semantic-search path.
**Hard constraint** (Boss, 2026-05-05): **do not touch M11's ~20K concepts.** The `lexicon/` module — its TSV (`lexicon_v1.tsv`), FST cache, graph structure, public API, and runtime singleton — stays byte-identical. CTSE reads M11; it never writes to it.

---

## §1 · What changed since v1

v1 proposed embedding the user's per-library term vocabulary (~111K terms on Boss's 7,635-note library) at runtime. Three SME audits (parallel-systems, library/IR, application-architecture) independently concluded that pipeline solves the wrong problem:

- Library size scales linearly. 100K-note library = hours of bootstrap.
- The unit being embedded (user-corpus terms) is noisy long-tail; semantic precision is mediocre.
- LCSH/MeSH/AAT (the canonical IR pattern since 1909) embed the **controlled vocabulary** and map patrons' words to it. M11 is exactly that controlled vocabulary.

**v2 inverts the phase order.** Architect-v1 Phase 4 ("Bridge Adapter") becomes the new Phase 1. Per-term user-corpus embedding is removed from the roadmap entirely — `term_vocab` continues to exist (Index panel, mentions, statistics) but no row in it is ever embedded.

---

## §2 · Mission (revised)

Provide **constant-time semantic search regardless of library size** by embedding M11's ~20K controlled-vocabulary concepts **once at build time**, shipping the resulting vectors as a binary asset, and resolving user terms (and free-text queries) to their nearest M11 concept at runtime via a small in-process k-NN over a 20K × 384-f32 matrix (~30 MB).

**Why "regardless of library size"**: the vector matrix is fixed at compile time. A user with 100 notes and a user with 1M notes hit the same lookup cost (~ms). The per-library work reduces to a single column on `term_vocab` (`bridge_concept_id`) populated by exact-form lookup against M11's existing `find_nodes` — no ONNX inference per term, no per-library bootstrap.

---

## §3 · Architecture (revised, Boss constraint honored)

### §3.1 Module layout — M11 untouched

```
src-tauri/src/
├── lexicon/                          ← M11. UNTOUCHED.
│   ├── mod.rs                        (public API: equivalents, expand, lexicon_expand_for_filter)
│   ├── graph.rs                      (LexiconGraph, find_nodes, FST cache)
│   ├── parse.rs, fts.rs, detect.rs, expansion.rs
│   └── data/lexicon_v1.tsv           (the ~20K concepts — Boss said: don't touch)
│
├── bridge_vectors/                   ← NEW. CTSE Bridge Adapter sibling.
│   ├── mod.rs                        (public API: nearest_concept, nearest_concepts_k)
│   ├── store.rs                      (in-memory ConceptVectorStore: id-table + flat f32 matrix)
│   ├── asset.rs                      (include_bytes! the baked .bin; zero-copy header parse)
│   └── data/concept_vectors_v1.bin   (build-time output: 20K × 384 f32 + concept-id table)
│
├── embeddings.rs                     (e5 ONNX runtime — already exists; reused for query embedding)
└── ctse/                             ← NEW. Bridge Adapter logic + write-time hooks.
    ├── mod.rs                        (public API: resolve_term_to_concept, search_by_concept)
    └── hooks.rs                      (reindex_single_note hook: populate term_vocab.bridge_concept_id)
```

**M11 changes**: zero. Not one byte of `lexicon/` is touched by this migration. The Bridge Adapter calls `lexicon::LexiconGraph::get()` and `find_nodes(lang, lemma)` as a read-only consumer — same way IndexPanel and `read_term_mentions` already do.

### §3.2 The four CTSE sub-engines (revised)

| Sub-engine | What it is | Where it lives | Touches M11? |
|---|---|---|---|
| **CTSE-1: Tokenizer** | `fts5_tokenizer::tokenize_to_vec` (already shipped, fix-8) | `fts5_tokenizer.rs` | No |
| **CTSE-2: Ledger** | `term_vocab` SQLite table (already shipped, fix-8) — gains one new column `bridge_concept_id TEXT NULLABLE` | `search.rs` schema | No |
| **CTSE-3: Bridge Vector Store** | 20K × 384-f32 in-memory matrix + concept-id table, baked at build time | `bridge_vectors/` | **Reads** M11 at build time; never writes |
| **CTSE-4: Bridge Adapter** | Resolves a term → concept_id (exact-form fast path via M11 `find_nodes`; vector-NN fallback for unknown terms) | `ctse/` | **Reads** M11 only |

(v1's "Sampler" sub-engine is dropped. Sampling solved a problem — embed a fraction of user terms — that no longer exists.)

### §3.3 Data flow

**Build time** (CI / `cargo build`, runs once per release):
1. A build helper (offline binary in `src-tauri/build_assets/build_concept_vectors.rs`) loads M11's parsed `LexiconGraph`, iterates all ~20K concepts, picks a canonical surface form per concept (English lemma if present, else first available language with `passage:` prefix per e5 model card), and runs the existing e5 ONNX pipeline batched.
2. Output: `concept_vectors_v1.bin` written to `bridge_vectors/data/`, layout:
   ```
   [magic: 8 bytes "CTSEBV01"]
   [count: u32 LE]                        ← number of concepts (e.g. 19,847)
   [dim: u32 LE]                          ← 384
   [concept_id_table: count × (u16 len + UTF-8 bytes)]
   [vector_matrix: count × dim × f32 LE]  ← row-major, L2-normalized
   ```
   Estimated size: ~30 MB (20K × 384 × 4 = 30.7 MB) + ~300 KB id table.
3. The build asset is committed to the repo (binary asset under git; or generated in CI and cached, decision deferred to Plan §3).

**Runtime, boot**: `bridge_vectors::asset::load()` runs once via `OnceLock`, parses the header, and exposes the matrix as a zero-copy `&'static [f32]` slice (or memmaps it on desktop targets — same pattern M9 uses for the FST cache, see `Cargo.toml:60`).

**Runtime, write path** (per-note save, via existing `reindex_single_note`):
1. Tokenize the note (CTSE-1, already happens).
2. For each new term landing in `term_vocab`, call `ctse::resolve_term_to_concept(term, detected_lang)`:
   - **Fast path**: `lexicon::LexiconGraph::find_nodes(lang, lemma)` — exact-form, microseconds. ~80% hit rate expected on lexicon-covered terms.
   - **Slow path**: if no node match, embed the term via e5 (same model already loaded), cosine-search the 20K matrix, take top-1 if score ≥ threshold (default 0.78, tunable). Returns `Some(concept_id)` or `None`.
3. Update `term_vocab.bridge_concept_id` for that term.

**Runtime, query path** (semantic search):
1. User types a query. Embed it once via e5 (`query:` prefix).
2. Cosine-search the 20K concept matrix → top-k concepts with scores.
3. Surface notes by joining `term_vocab` rows where `bridge_concept_id IN (top_k)` → `notes_fts` MATCH.

**Cost profile**:
- Boot: zero ONNX work. One mmap or `include_bytes!` slice.
- Per-note save: 1 e5 inference *only* for terms M11 doesn't have exact-form coverage for (long-tail proper nouns, code identifiers, etc.). Most notes incur zero ONNX work.
- Per-query: 1 e5 inference + 1 cosine pass over 20K × 384 floats (~5 ms on modern CPU, SIMD-friendly).

---

## §4 · Invariants that must not break

1. **M11 read-only**: `lexicon/` source files have a zero-line diff at the end of MIG-013. Verified mechanically by `git diff src-tauri/src/lexicon/` returning empty.
2. **Boot time unchanged**: the bridge-vector asset loads via mmap (desktop) or `include_bytes!` (mobile fallback) — no parse loop, no allocation of 30 MB on the heap. Same pattern as M9 FST mmap.
3. **No keystroke-path IPC**: query embedding only fires on debounced search submit (≥300 ms), never per-keystroke. (CLAUDE.md Rule 3.)
4. **Write-time derivation honored** (Rule 8): `term_vocab.bridge_concept_id` is populated by the same `reindex_single_note` hook that updates `notes_fts`. No `rebuild_*` command. First-time backfill on existing libraries runs progressively in the status bar, resumable.
5. **Cancellation safety**: backfill is checkpointed per batch; closing the app mid-fill leaves a partial-but-correct state, resume continues from `MAX(rowid)` of un-resolved rows.
6. **Bundle size**: ~30 MB asset addition is acceptable but must be measured against current binary size and reported in the Plan's verification clause.
7. **Mobile targets**: iOS/Android still build. The asset uses `include_bytes!` fallback (heap-resident) since mobile sandboxes routinely deny mmap (mirrors Cargo.toml `cfg(not(any(target_os = "ios", target_os = "android")))` pattern at line 60).
8. **No regression on existing M11 surfaces**: MIG-010 mentions expansion and MIG-011 filter expansion continue to work unchanged. Verified by smoke-testing `read_term_mentions` and `lexicon_expand_for_filter` after the migration.

---

## §5 · What is removed

- The half-finished fix-10 edit in `embeddings.rs` (heartbeat-before-fetch + streaming SQL) — discarded, not committed. SME consensus: it patches a problem that ceases to exist.
- v1 Phase 1 `populate_term_vocab` rayon parallel bootstrap — kept (it's still useful for the Ledger; the term-vocab table is consumed by Index panel and mentions). But its embed-loop downstream (`init_term_embeddings`) is removed.
- `init_term_embeddings`, `run_embedding_batch` (v1's Phase 1.5 batch ONNX), and the `term_embeddings` table (if present in schema) — all removed. The model loader (`ensure_engine`) stays — we still need e5 for query embedding and slow-path term resolution.
- The Settings modal "Build Term Embeddings" / "Rebuild" button + modal flow — replaced by a quiet status-bar progress for the one-time backfill of `term_vocab.bridge_concept_id`.

---

## §6 · Phased rollout

### Phase 1A — Build-asset pipeline (offline, no runtime changes)
- Add `build_assets/build_concept_vectors.rs` binary target.
- Generates `bridge_vectors/data/concept_vectors_v1.bin` from M11.
- Verification: file produced, header valid, sample 10 concepts decode to plausible vectors (norm ≈ 1.0).
- **No user-visible change**. Lands as a single commit; binary asset committed.

### Phase 1B — Runtime asset loader + adapter API
- New `bridge_vectors/` module with `load()`, `nearest_concept()`, `nearest_concepts_k()`.
- New `ctse/mod.rs` with `resolve_term_to_concept()`, `search_by_concept()` (skeleton, no callers yet).
- Verification: unit tests on synthetic 100-concept fixture; cosine-NN returns correct id; M11 exact-form fast path returns correct id without ONNX.
- **No user-visible change**.

### Phase 1C — Schema + write-time hook
- Add `bridge_concept_id TEXT` column to `term_vocab` via migration.
- Wire `ctse::hooks::on_term_seen` into `reindex_single_note`.
- One-time backfill: progressive, resumable, in the status bar.
- Verification: open Boss's library; backfill completes without freezing; spot-check 20 known terms have plausible concept ids.

### Phase 1D — Query path + UI cleanup
- Replace whatever frontend currently triggers term-embedding rebuild with a simple semantic-search input wired to `ctse::search_by_concept`.
- Remove the Settings modal "Rebuild term embeddings" button.
- Verification: type a multi-language query (Arabic + English); verify results surface notes joined via `bridge_concept_id`.

### Phase 1E — Audit
- Three agents in parallel: invariant checker, drift checker, migration-path checker (per `/migration` rule).

(Future Phase 2: cross-concept relationships using M11's existing edges — supports semantic-graph traversal beyond top-k. Deferred.)

---

## §7 · Open questions for the Plan

1. **Asset commit policy**: commit the 30 MB `.bin` to the repo, or generate in CI and cache? (Repo simplicity vs. clone size trade-off — recommend committing for now since it changes only when the M11 TSV does.)
2. **Slow-path threshold**: 0.78 cosine is a starting guess from e5 model card; tune empirically on Boss's library before locking.
3. **Surface-form selection** when a concept has many lemmas: English-first vs. concatenate top-N lemmas vs. embed each and average. The Plan picks one and verifies.
4. **Backfill prioritization**: TF-IDF descending (best-distinguishing terms resolve first → search becomes useful early) vs. rowid order. Recommend TF-IDF descending.

These get resolved in the Plan, not here.

---

## §8 · Closing

CTSE v2 is the architecture all three SMEs converged on. M11 stays untouched per Boss directive. The user-corpus embedding pipeline — and every fix-N attempt to make it scale — is retired. Bootstrap freezes become impossible because there is no per-library bootstrap to freeze.

**Approval needed**: this Architect doc, then a Plan doc breaking Phase 1A–1E into landable commits with verification clauses. No code lands until the Plan is approved.
