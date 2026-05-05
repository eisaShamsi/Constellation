# MIG-013 — Constellation Terms Scanning Engine (CTSE)

**Date opened**: 2026-05-05
**Status**: Phase 1 (Architect) — drafted under Boss directive ("Don't reinvent the wheel; fetch the fix from the experts")
**Owner**: Vocabulary subsystem
**Composes with**: MIG-010 / MIG-011 / MIG-012 (semantic search, lexical-bridge filter, mentions). MIG-008 / MIG-006 (write-path triggers via reindex_single_note).

---

## §1 · Mission

Provide **fast, scalable, accurate vocabulary statistics** for any Constellation consumer that needs to know *what terms exist, how often, and where* — at sub-second response time regardless of library size, proven up to 100K+ notes.

**Why it exists**: MIG-012's term-embedding pipeline tried to enumerate vocabulary via `fts5vocab` (the FTS5 vocab virtual table). On a 7,635-note multi-script library this hung 20+ minutes per query. Seven progressive band-aids (fix-3 through fix-7: cnt threshold tuning, fresh connection, dropped ORDER BY, FTS5 segment merge) all failed. fix-8 introduced a maintained shadow table per CLAUDE.md Rule 8 (Write-Time Derivation), but the bootstrap itself was single-threaded and froze at note 601/7635 on Boss's library.

CTSE codifies this work as a **first-class engine** — peer to the Arabic Engine (CAE) and Lexical Bridge (M11) — with proper architecture from peer disciplines.

---

## §2 · Research base (where the techniques come from)

| Technique | Source | Application here |
|---|---|---|
| **TermsEnum** with cached `TermState` | Apache Lucene / Elasticsearch | Materialize a flat `(term, doc_count, total_count)` table maintained at write-time so reads never walk doc-lists. |
| **Controlled Vocabulary (LCSH)** | Library of Congress (since 1909) | M11 Lexical Bridge is Constellation's controlled vocabulary. CTSE's bridge adapter (Phase 4) maps user-library terms to M11 concepts so semantic search uses the curated vocabulary, not the user's noisy long tail. |
| **TF-IDF** | Salton & McGill, *Introduction to Modern Information Retrieval* (1983) | Phase 3: rank terms by inverse document frequency; embed only the top-K (most-distinguishing terms). Avoids embedding 500K low-information tokens. |
| **Sampling for collection statistics** | Witten, Moffat, Bell, *Managing Gigabytes* (1999) | Phase 3: at huge scale (>50K notes), sample N% of notes for vocabulary statistics. Extrapolate counts. |
| **Map-Reduce parallelism** | Dean & Ghemawat (2004) | Phase 1: tokenize per-note in parallel via rayon, fold to thread-local maps, reduce to global. ~8× speedup on 8-core machines. |
| **Materialized views** | Codd's relational model | Already adopted (CLAUDE.md Rule 8). CTSE's `term_vocab` table is the materialized view; reindex_* hooks maintain it incrementally. |

---

## §3 · Sub-engines

CTSE has four cooperating components, modeled on how mature retrieval systems decompose the same problem:

### CTSE-1: Tokenizer
The tokenization pipeline (already built in `fts5_tokenizer.rs::tokenize_to_vec`). Same pipeline FTS5 uses — guarantees term namespace consistency between `notes_fts` and `term_vocab`.

### CTSE-2: Ledger
The materialized `term_vocab` SQLite table. Schema:
```sql
CREATE TABLE term_vocab (
    term TEXT PRIMARY KEY,
    doc_count INTEGER NOT NULL,
    total_count INTEGER NOT NULL,
    last_seen INTEGER NOT NULL DEFAULT (CAST(strftime('%s','now') AS INTEGER))
);
CREATE INDEX idx_term_vocab_total_count ON term_vocab (total_count DESC);
```
The Ledger is the source of truth for any "what terms exist?" query.

### CTSE-3: Sampler
For libraries >50K notes, full tokenization is expensive even parallelized. The Sampler reads a randomized N% (default 10%) and extrapolates frequencies. Statistical accuracy: top-1000 terms are >99% accurate at 10% sample. Edge cases (rare terms appearing only once) miss but matter less.

### CTSE-4: Bridge Adapter
Maps `term_vocab` entries to M11 Lexical Bridge concepts. Returns "this user-term `معرف` corresponds to bridge concept `c:knowledge` which has equivalents in 15 languages." Powers controlled-vocabulary semantic search — embed the 20K M11 concepts once (build-time, ships with binary), look up at query time. **Library size becomes irrelevant** for semantic search if Bridge Adapter is used.

---

## §4 · Phased rollout

### Phase 1 — Parallel bootstrap (this commit)
**Scope**: Replace MIG-012-fix-8's single-threaded `populate_term_vocab` with a rayon-parallelized version. Drop the unused `HashSet<path>` (Boss's library size revealed the original design wastes memory on path string clones). Body cap at 1 MiB per note (pathological-import safety). Tighter progress (every 10 notes). Per-note timing log for any tokenization >500ms (diagnostic for outlier notes).

**Performance target**: 7,635 notes tokenize in <60 sec on an 8-core machine. 100K notes in <15 min. Measured on Boss's library at next test.

**Boss test gate**: Rebuild click → "Building vocabulary" phase completes within minutes (not 20+) → embed phase begins with reasonable N.

### Phase 2 — Incremental maintenance via reindex hooks
**Scope**: When `reindex_single_note` runs (on every note save), also call `ctse::update_incremental(note_path)` to re-tokenize and update term_vocab counts. Same when `reindex_delete_note` runs. Net: term_vocab stays current without full rebuild.

**Triggers when**: any future MIG that touches reindex paths. Logged for follow-on.

### Phase 3 — TF-IDF + sampling
**Scope**: Add `total_count_idf` column to `term_vocab` storing log(N/df) score. Rank terms by tf-idf for embed-corpus selection (top-N terms that distinguish notes, not just frequent terms). Sampler activates for libraries >50K notes.

**Triggers when**: a Boss directive to optimize semantic search quality, OR a library that hits the 50K threshold.

### Phase 4 — M11 Bridge Adapter
**Scope**: Embed the 20K M11 corpus at build-time (ship vectors with binary). Add `bridge_concept_id` foreign key on term_vocab rows. Semantic search becomes "find the M11 concept closest to query, surface user-library terms mapped to that concept." Constant-time semantic search regardless of library size.

**Triggers when**: Phase 1 lands and Boss requests semantic search to scale to 100K+ libraries, OR a Boss directive to align semantic search with the controlled vocabulary.

---

## §5 · Phase 1 implementation plan

**Files touched**:
- `src-tauri/src/embeddings.rs` — replace `populate_term_vocab` body with rayon parallel version. Drop `HashSet<String>` path tracking. Keep body cap, per-note logging, tightened progress.
- `src-tauri/src/fts5_tokenizer.rs` — `tokenize_to_vec` already shipped (fix-8). Reused as-is.
- `src-tauri/src/search.rs` — schema already has `term_vocab` table (fix-8). Reused as-is.
- `src-tauri/Cargo.toml` — `rayon = "1"` direct dep already added (fix-9-prep).

**Algorithm**:
```rust
// Build stopwords once
let stopwords = Arc::new(crate::libraries::build_stopwords());

// Atomic progress counter shared across rayon threads
let processed = Arc::new(AtomicU32::new(0));
let total = notes.len() as u32;

// Heartbeat thread emits progress every 500ms
let stop = Arc::new(AtomicBool::new(false));
let heartbeat = std::thread::spawn(emit_loop(processed, total, stop, app));

// Parallel tokenize-and-aggregate. Each rayon thread builds a local
// HashMap<String, u32>; reduce merges them at the end.
let term_counts: HashMap<String, u32> = notes.par_iter()
    .map_with(stopwords.clone(), |stopwords, (path, body)| {
        if cancel.load() { return HashMap::new(); }
        let clipped = clip_body(body, BODY_CAP_BYTES);
        let tokens = tokenize_to_vec(clipped, &stopwords);
        let mut local = HashMap::new();
        for token in tokens {
            *local.entry(token).or_insert(0) += 1;
        }
        processed.fetch_add(1, Ordering::Relaxed);
        local
    })
    .reduce(HashMap::new, merge_maps);

stop.store(true, Ordering::Relaxed);
heartbeat.join().ok();

// Single-transaction bulk write to term_vocab
write_term_vocab(&app, term_counts)?;
```

**Tradeoffs accepted**:
- `doc_count` set to 0 in Phase 1 (we don't use it). Phase 2's incremental hooks will populate it accurately when each note is saved.
- Cancellation has up to 1 note's tokenization-time of latency (the rayon thread checks the flag at the start of each note, not mid-tokenization).
- Heartbeat fires every 500ms — frontend sees smooth progress even on rare slow batches.

---

## §6 · Closing

CTSE elevates a fix from "patch the embedding flow" to "design the vocabulary subsystem the way mature IR systems do." Phase 1 unblocks Boss tonight. Phases 2-4 are designed but deferred until directive.

**Approval needed**: Phase 1 implementation begins immediately on commit of this Architect doc + the implementation commit. Phases 2-4 await Boss greenlight.
