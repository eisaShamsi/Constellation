# Session log — 2026-05-28

## Block 1 (early morning) — MIG-058 + MIG-059 resolution + §L PCS

The block opened on 2026-05-27 evening's hard wall: federated search was stuck at 13-25 seconds across 8 iterations (§K.1 → §K.2 → §K.3 → Diagnostic v2 → Option C → Option E → Option F → Option G). Eisa's directive at the end of 2026-05-27 was the breakthrough impulse: *"It is not in my doctrine to accept any limitation! Think again!"* — followed by *"If this didn't work, stop patching and try to solve it once and for all."*

That re-framing forced an honest re-read of Eisa's diagnostic data. The Option G boss-test had shown: FTS5 segment merge ran (39s first boot, 0ms second), search quality improved per Eisa's own observation, but timing didn't move from ~16s. Combined with the empirical fact that the cost scaled with **result count (30 rows)** not segment count, the SQL itself had to be examined column by column. The only thing in the SQL doing per-row work via the custom Arabic-normalizing tokenizer was `snippet(notes_fts, 1, '<mark>', '</mark>', '...', 40)`.

Option H bypassed FTS5's native `snippet()` in federated mode and synthesized snippets in Rust from raw `body_text`. Eisa's boss-test result:

- Stage 2 (paste `الرباط`): **almost instantly** (was 16-25s).
- Stage 2b (`الربا`): **under a second**.
- Stage 3 (Arabic slow-typing): **no truncation; full word lands**.

Both MIGs closed in one shot. MIG-058 (truncation) resolved as a side effect — proving the earlier hypothesis that the input dropouts were caused by IPC blocking during slow async searches, not by Svelte / IME-layer behavior.

## The 8-iteration arc, captured

Every option pruned a hypothesis with evidence. None were wasted:

| Option | Hypothesis | Result | Lesson |
|---|---|---|---|
| §K.1 | Tokenizer not registered on federation Connection | Shipped, no behavior change | Necessary but not sufficient |
| §K.2 | UNION ALL with bm25/snippet was the bug | Dropped both, ordered by modified DESC | Functional but lost BM25 ranking |
| §K.3 | Per-cUniverse standalone Connection enables bm25 | Shipped, but standalone Connections were 15-25× slower than active | Cold FTS5 segment pages |
| §K.3.A diagnostic | Tokenizer / token mismatch / FTS5 schema differs | All probes equal; ruled out | Data-driven |
| Option C | Per-schema queries on warm federated_conn instead of standalone | Verified bm25 works in single-schema attached query; 13s baseline | Architecture correct, perf still bad |
| Option E | PRAGMAs (mmap_size, cache_size) on federated_conn | 18s — REGRESSED. Reverted. | mmap on ATTACH bypasses libraryStats-warmed OS cache |
| Option F | Pre-warm OS page cache via MATCH on throwaway Connection | Returned 0 matches (stopword filter stripped tokens), 16s | Need to verify the warm-up actually warms |
| Option G | FTS5 segment merge (`INSERT INTO notes_fts(notes_fts) VALUES('optimize')`) | 39s first run, 0ms idempotent. Quality improved, timing didn't. | Fragmentation wasn't the dominant cost |
| **Option H** | **Bypass FTS5 `snippet()` — tokenizer pass per row** | **< 1 second. Done.** | **snippet() with custom tokenizer was the bottleneck** |

## Commits that landed today

| # | Hash | Title | Layer |
|---|---|---|---|
| 1 | `c426af7e` | MIG-058 + MIG-059 — Option H: bypass FTS5 snippet() in federated mode | Backend |
| 2 | (this commit) | docs(MIG-058+MIG-059 §L PCS): orientation v2.39 + MoCh + final-state docs | Docs |

Plus the predecessor work from 2026-05-27 evening that's still part of the federation-perf MIG: Option G (`4cbdd56a`), Option F (`ab666eca`), Option E + revert (`912715b9`, ...), Option C (`fb83797e`), Diagnostic v2 (`72927a7f`), and earlier.

## Test counts

- 840/840 lib tests pass (4 option_c_* + 836 pre-existing).
- 47/47 federation tests still pass.
- 84/84 lens tests still pass.

## What ships in §L PCS

1. Final-state docs for MIG-058 + MIG-059.
2. Orientation v2.39 preamble closing the chapter.
3. MoCh-2026-05-28 entry for the conversational arc.
4. Milestone git tag `milestone/mig-058-mig-059-resolved`.
5. ZIP backup.
6. Updated user-facing help docs across 15 locales (federation no longer has the speed caveat).

## What's next after PCS

Back to the Constellation Base roadmap from Concept Paper v1.4. The federation work (MIG-056 through MIG-059) was a 4-MIG detour triggered by MIG-055 §I Stage 5's federation gap. The next trunk MIG is **Phase 1.5 — Host-Note Assemblage + Open-in-360.3D + Open-in-CNS + Open-in-Cataloger gestures**.

## Lessons for future MIGs

1. **When data plateaus across 3+ iterations, dig deeper before iterating again.** Options C-G all hit ~13-16s. That plateau was the signal that I was iterating around the wrong dimension. The cost scaled with result count (30 rows), not with anything I was changing. Reading the SQL column-by-column for per-row work was the right next move, and I should have done it earlier.

2. **`snippet()` with custom tokenizers is expensive.** SQLite FTS5's native `snippet()` re-tokenizes each matched row's column to find marker positions. With a custom Arabic-normalizing tokenizer, that's ~500ms per row × 30 rows = 15 seconds. Rust-side substring snippet is microseconds. Future federated-search work should default to Rust-side snippet generation unless there's a specific reason to prefer FTS5 native.

3. **Eisa's "no doctrine of limitation" is the right operating principle.** Each "let's accept this and document" framing I tried got pushed back. The actual fix existed; we just hadn't found it yet.

4. **Diagnostic v2 was load-bearing.** The data Eisa pasted (per-branch timings, sqlite_stat1 contents, EXPLAIN QUERY PLAN, keystroke event log) ruled out hypothesis after hypothesis. Without it, I would have shipped Option E (which regressed) as a guess.
