# MIG-013 — CTSE Phase 1 Plan (Bridge Adapter)

**Date**: 2026-05-05
**Architect**: `MIG-013-CTSE-ARCHITECT-v2.md` (approved by Boss 2026-05-05, "Go for A. But don't touch the M11's ~20K concepts.")
**Hard constraint**: zero-line diff in `src-tauri/src/lexicon/`. Verified mechanically before each commit.

---

## §0 · Pre-flight (no commit)

Before Phase 1A lands, run a one-time scout in the worktree to lock the open questions from Architect §7:

| Question | How resolved | Recorded in |
|---|---|---|
| Bundle size impact | Build current binary, record size. Build with placeholder 30 MB asset, record delta. | This file's §6 verification log |
| Surface-form selection | Iterate M11 concepts: count how many have `en` lemma, how many don't. Decision: "English lemma first; fall back to first lemma in language priority en→ar→es→…→zh." | This file's §0 record |
| Slow-path threshold | Defer to Phase 1D — measured on Boss's library after backfill. Default 0.78 from e5 model card. | Plan §5 verification |
| Asset commit policy | Commit the .bin to repo. Rationale: changes only when M11 TSV changes (rare); avoids CI build asset distribution complexity. Re-evaluate if repo size becomes painful. | This file's §0 record |
| Backfill priority | TF-IDF descending. Most-distinguishing terms resolve first → search becomes useful before backfill completes. | Phase 1C |

**Pre-flight deliverables** (no commits, just session-log notes):
- Concept-coverage stats (X of 20K have `en`, Y have only non-`en`).
- Current binary size baseline.

---

## §1 · Phase 1A — Build-time concept-vector pipeline

**Goal**: produce `bridge_vectors/data/concept_vectors_v1.bin` from M11. No runtime change. No user-visible change.

### Files touched
- **NEW** `src-tauri/build_assets/build_concept_vectors.rs` — standalone binary target. Reads M11 in-process (calls `lexicon::LexiconGraph::get()`), iterates concepts, picks canonical surface form, runs e5 batched, writes the .bin.
- **NEW** `src-tauri/src/bridge_vectors/data/concept_vectors_v1.bin` — committed binary asset.
- **NEW** `src-tauri/src/bridge_vectors/mod.rs` — empty stub; populated in 1B. (Module must exist so the asset path resolves under `include_bytes!` later.)
- `src-tauri/Cargo.toml` — add `[[bin]] name = "build_concept_vectors"` target, dev-only.
- `src-tauri/src/lexicon/` — **untouched**. Verified by `git diff --stat src-tauri/src/lexicon/` returning empty.

### Algorithm
```rust
// build_concept_vectors.rs (offline binary)
fn main() -> Result<()> {
    let graph = lexicon::LexiconGraph::get();           // existing M11 API, read-only
    let concepts = graph.iter_concept_ids();             // returns ~20K ConceptId values
    let surface_forms: Vec<(ConceptId, String, Lang)> = concepts
        .map(|cid| pick_canonical_lemma(graph, cid))     // en → ar → es → … priority
        .collect();
    let engine = embeddings::ensure_engine()?;           // existing e5 ONNX runtime
    let vectors = surface_forms
        .chunks(128)
        .flat_map(|batch| {
            let texts: Vec<String> = batch.iter()
                .map(|(_, lemma, _)| format!("passage: {lemma}"))
                .collect();
            embeddings::run_embedding_batch(&engine, &texts).unwrap()
        })
        .collect();
    write_bin(BIN_PATH, &surface_forms, &vectors)?;      // header + id table + matrix
    Ok(())
}
```

### Verification (must pass before commit)
1. `cargo run --bin build_concept_vectors --release` produces `concept_vectors_v1.bin`.
2. File size within 30–35 MB.
3. Header magic `CTSEBV01`, count matches M11 concept count from the pre-flight scout, dim == 384.
4. Spot-check 10 random concepts: `c:book`, `c:knowledge`, etc. — vectors decode, L2 norm ≈ 1.0 ± 0.001.
5. **`git diff src-tauri/src/lexicon/` returns empty** (M11 untouched invariant).
6. Main app binary build still succeeds (`cargo tauri build` — not run, just `cargo build` for size).

### Commit message
```
MIG-013 §1A: bake M11 concept vectors at build time

Adds offline `build_concept_vectors` binary that runs e5 over the
~20K M11 concepts once and emits concept_vectors_v1.bin (~30 MB,
20K × 384 f32, L2-normalized + concept-id table).

No runtime change yet — the asset is loaded in §1B. M11 itself
is read-only here; lexicon/ has zero-line diff (verified).

Architect: lab/reports/MIG-013-CTSE-ARCHITECT-v2.md §3.3
Plan: lab/reports/MIG-013-CTSE-PLAN.md §1
```

---

## §2 · Phase 1B — Runtime asset loader + Bridge Adapter API

**Goal**: load the .bin at boot, expose `nearest_concept(query_vec) -> Option<(ConceptId, f32)>`. No callers yet. No user-visible change.

### Files touched
- `src-tauri/src/bridge_vectors/mod.rs` — public API: `load()`, `nearest_concept()`, `nearest_concepts_k()`.
- **NEW** `src-tauri/src/bridge_vectors/store.rs` — `ConceptVectorStore` struct; cosine k-NN over flat f32 matrix (SIMD-friendly inner loop, no external dep).
- **NEW** `src-tauri/src/bridge_vectors/asset.rs` — header parser; mmap on `cfg(not(any(target_os = "ios", target_os = "android")))` mirroring `Cargo.toml:60` pattern; `include_bytes!` fallback on mobile.
- **NEW** `src-tauri/src/ctse/mod.rs` — `resolve_term_to_concept(term, lang) -> Option<ConceptId>`. Fast-path via `lexicon::LexiconGraph::find_nodes`; slow-path embeds + calls `bridge_vectors::nearest_concept`.
- `src-tauri/src/lib.rs` — register module: `mod bridge_vectors; mod ctse;`.
- `src-tauri/src/lexicon/` — **untouched** (invariant verified).

### Verification
1. `cargo build --release` succeeds.
2. Unit test in `bridge_vectors/store.rs`: synthetic 100-concept × 8-dim store, query at known concept returns that concept with score 1.0.
3. Unit test in `ctse/mod.rs`: input "book" with `Lang::En` resolves via M11 fast path (no e5 call); input "garblefuxx" with `Lang::En` returns `None` from slow path (or an unrelated concept below threshold).
4. Boot a debug binary; first call to `bridge_vectors::nearest_concept` after boot completes in < 50 ms (mmap + cosine over 20K × 384).
5. **M11 zero-line diff** check.

### Commit message
```
MIG-013 §1B: bridge_vectors runtime loader + ctse adapter API

Adds bridge_vectors module (mmap loader for concept_vectors_v1.bin,
cosine k-NN over the 20K × 384 matrix) and ctse module (term→concept
resolver: M11 fast path, vector slow path).

No callers yet; wired in §1C. M11 untouched.
```

---

## §3 · Phase 1C — Schema, write-time hook, backfill

**Goal**: every saved note now populates `term_vocab.bridge_concept_id` for its terms. Existing libraries get a one-time progressive backfill in the status bar.

### Files touched
- `src-tauri/src/search.rs` — schema migration: `ALTER TABLE term_vocab ADD COLUMN bridge_concept_id TEXT;` + index `idx_term_vocab_bridge_concept_id`. Idempotent (check `PRAGMA table_info`).
- `src-tauri/src/embeddings.rs` — **delete** `init_term_embeddings`, `run_embedding_batch` callers, term_embeddings-related code. Keep `ensure_engine` (still needed for query embedding + slow-path resolution).
- **NEW** `src-tauri/src/ctse/hooks.rs` — `on_terms_seen(conn, terms: &[(String, Lang)])` called from `reindex_single_note`.
- `src-tauri/src/{libraries.rs|search.rs}` — wire `on_terms_seen` into the existing `reindex_single_note` write path. (Exact file determined during build; the hook is one call site.)
- **NEW** `src-tauri/src/ctse/backfill.rs` — progressive backfill task; checkpointed per batch of 500 terms; resumable; emits `ctse-backfill-progress` Tauri event.
- `src/lib/components/StatusBar.svelte` (or equivalent) — show backfill progress when active.
- `src/lib/libraries/store.ts` — listener for `ctse-backfill-progress`.
- `src-tauri/src/lexicon/` — **untouched**.

### Backfill order
TF-IDF descending: highest-IDF terms (rarest, most-distinguishing) resolve first. SQL: `SELECT term FROM term_vocab WHERE bridge_concept_id IS NULL ORDER BY total_count ASC, term LIMIT ?` (rare = high IDF). Tunable; the Plan accepts this default.

### Cancellation / resume
- Each batch (500 terms) commits in one transaction.
- App close mid-fill → next boot picks up where the `WHERE bridge_concept_id IS NULL` query restarts.
- No "Rebuild" button anywhere — backfill is automatic and silent except for the status-bar strip.

### Verification (Boss-testable)
**Test tutorial — to be sent to Boss after this commit:**

> **What this is**: Constellation's semantic search now uses a curated 20,000-concept dictionary that ships with the app, instead of trying to learn vocabulary from your library. This means: (a) you don't wait for any "build vocabulary" phase — search is ready as soon as your terms are linked to the dictionary, (b) library size doesn't matter — 100 notes or 100,000 notes have the same search performance, (c) you don't see a "Rebuild" button anymore — the linking happens quietly in the status bar the first time you open a library.
>
> **Step 1 — Pre-state**: open Boss's library. Look at the status bar at the bottom of the window. You should see a faint "Linking terms… N / M" strip. M is the count of unique words in your library; N grows over a few minutes.
>
> **Step 2 — Action**: while the strip is still running, click any note and edit it normally. Type a few words. The strip should keep moving — editing must not slow it down or be slowed down by it.
>
> **Step 3 — Post-state**: when the strip disappears, open the Index panel. Each term is now silently tied to a dictionary concept (not visible yet — query path lands in §1D). The work is done; you'll never see this strip again for this library unless you add many new notes.
>
> **If you see this instead**: a frozen N (no progress for 30+ seconds) means the backfill stalled. Report it; do NOT close the app — the partial state is correct, but I want to inspect the stall point.

1. Boss confirms the strip moves smoothly to completion.
2. Editing during backfill is unaffected.
3. Spot-check via SQL: `SELECT term, bridge_concept_id FROM term_vocab WHERE bridge_concept_id IS NOT NULL LIMIT 20;` — known terms (book, knowledge, معرفة) resolve to plausible concept ids.
4. Closing the app mid-backfill and reopening: progress resumes from where it left off (rowcount strictly monotonic).
5. **M11 zero-line diff** check.

### Commit message
```
MIG-013 §1C: term_vocab.bridge_concept_id + write-time hook + backfill

Adds bridge_concept_id column to term_vocab, wires ctse::hooks into
reindex_single_note (every saved note resolves its terms to M11
concepts), and runs a one-time progressive backfill on existing
libraries (TF-IDF descending, checkpointed per 500 terms, resumable).

Removes init_term_embeddings and the term_embeddings table — the
per-library term-embedding pipeline is retired per Architect v2.

M11 untouched.
```

---

## §4 · Phase 1D — Query path + Settings UI cleanup

**Goal**: semantic-search input wired to `ctse::search_by_concept`. The old "Build / Rebuild Term Embeddings" Settings flow is removed.

### Files touched
- **NEW** `src-tauri/src/ctse/search.rs` — `search_by_concept(query: String, library_id: String, limit: u32) -> Vec<NoteHit>`. Embeds query (e5 `query:` prefix), runs `bridge_vectors::nearest_concepts_k(query_vec, k=10)`, joins `term_vocab` on `bridge_concept_id IN (...)`, returns notes via `notes_fts MATCH`.
- `src-tauri/src/lib.rs` — register Tauri command.
- `src/lib/components/SettingsModal.svelte` — **remove** "Rebuild Term Embeddings" button + confirmation dialog + progress UI for old pipeline. Remove `confirmDialog`, `termEmbedProgress` UI, related effects.
- `src/lib/libraries/store.ts` — remove `TermEmbedProgress` interface, `termEmbedProgress` writable. Keep `ctse-backfill-progress` listener from §1C.
- `src/lib/components/SearchPanel.svelte` (or wherever semantic search lives) — call `ctse::search_by_concept`.
- `src-tauri/src/lexicon/` — **untouched**.

### Verification (Boss-testable)
**Test tutorial — to be sent to Boss after this commit:**

> **What this is**: the search input can now find notes that share *meaning* with your query, not just notes containing the literal words. If you search "knowledge", a note that says "معرفة" (Arabic for knowledge) shows up because both link to the same dictionary concept. The dictionary is fixed and ships with the app — no per-library learning, no rebuild, no waiting.
>
> **Step 1 — Pre-state**: open the search panel (location TBD when SearchPanel is wired). Make sure the §1C backfill has finished (status bar has no "Linking terms" strip).
> **Step 2 — Action**: type the English word "knowledge". Wait 300 ms (debounce).
> **Step 3 — Expected**: results include notes that contain "knowledge", "معرفة", "wissen" (German), "savoir" (French) — anything mapped to the same concept. Each result should still show the actual matched term in context.
> **Step 4 — Cross-language test**: search "معرفة" directly. Same notes should appear, regardless of which surface form was queried.
>
> **If you see this instead**: only English notes appear → the cross-language join didn't fire; report.

1. Boss confirms cross-language results.
2. SearchPanel typing has zero perceptible lag (Rule 1).
3. Settings modal no longer shows "Rebuild Term Embeddings" anywhere.
4. Cosine threshold sanity: tune from 0.78 if Boss reports irrelevant results; record final value in session log.
5. **M11 zero-line diff** check.

### Commit message
```
MIG-013 §1D: query path via concept-NN; remove old term-embedding UI

Adds ctse::search_by_concept Tauri command (embed query → top-k
M11 concepts → join term_vocab.bridge_concept_id → notes_fts).
Cross-language results work because every concept ships with
multilingual surface forms.

Removes the Settings modal "Rebuild Term Embeddings" flow and its
frontend store. CTSE is now invisible — search "just works".

M11 untouched.
```

---

## §5 · Phase 1E — Audit (per Migration Rule §4)

Three parallel agents on the cumulative diff `MIG-013 §1A..§1D`:

1. **Invariant checker**: verify CLAUDE.md Rules 1-8 not violated; `lexicon/` zero-diff; no `invoke()` on keystroke path; backfill is resumable and idempotent.
2. **Drift checker** (LL-023 lens): are there guards anywhere in the system the migration didn't update? E.g., does any frontend code still expect `termEmbedProgress` to exist? Does the help file mention "Rebuild Term Embeddings"?
3. **Migration-path checker**: simulate (a) first-boot fresh install on Boss's library, (b) schema mismatch with old DB lacking `bridge_concept_id`, (c) backfill interrupted mid-batch, (d) rollback (revert all 4 commits — does the app still boot? `term_vocab` survives without the column?).

Findings go into `lab/reports/MIG-013-CTSE-AUDIT.md`. Any P0 findings are fixed before the audit closes.

---

## §6 · Verification log (filled in as we go)

| Phase | Commit hash | Boss tested? | Result | Notes |
|---|---|---|---|---|
| 1A | _pending_ | n/a (silent) | _pending_ | _bin size measured: pending_ |
| 1B | _pending_ | n/a (silent) | _pending_ | |
| 1C | _pending_ | yes | _pending_ | _backfill duration on 7635 notes: pending_ |
| 1D | _pending_ | yes | _pending_ | _final cosine threshold: pending_ |
| 1E | _pending_ | n/a | _pending_ | _audit findings: pending_ |

---

## §7 · Rollback plan

If anything in 1A–1D blocks Boss's day:
1. `git revert` the offending commit(s) — they're independent enough that 1D revert leaves 1A–1C usable (just no query path).
2. The `bridge_concept_id` column on `term_vocab` is nullable and unused without the query path — leaving it in a reverted state is harmless.
3. Old "Rebuild Term Embeddings" UI does NOT come back on revert (it was retired in 1D and 1C cleared the underlying code). Acceptable: the v1 UI was the source of the freeze — restoring it is a regression we don't want anyway.

---

## §8 · Approval gate

This Plan needs explicit Boss approval before §1A lands. Once approved, per the Plan-Approval-Equals-Build-Approval rule (CLAUDE.md), I cascade through 1A → 1B → 1C → 1D → 1E without per-step approval, stopping only at:
- The 1C and 1D Boss-testable verification clauses.
- Any genuine architectural surprise.
- 1E completion (final summary + next decision).
