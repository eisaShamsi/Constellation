<!-- Auto-generated forensic report (workflow wf_e9b4414d) + reconciled with the optimize-on-copy
timing test run by the orchestrator (notes_fts_data 55985->55951; copy vocab lookups 8ms->4ms; the
copy is FAST, so the live-DB ~20ms/lookup slowness is access-conditions on the 1.9GB index, NOT
fragmentation). MIG-086 is unaffected (suggest_related_notes reads notes_vocab, which is correct). -->

# Forensic Report — term_vocab Drift (Issue A) & FTS5 Fragmentation (Issue B)

**Date:** 2026-06-23 · **Universe:** Eisa Cognitive Knowledge (7,660 notes) · **DB:** `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db` (1.89 GB)

---

## Top-line for Eisa (plain language)

Two separate problems were found in the search engine's plumbing. **Neither is hurting what you see on screen today**, but both are real and both should be fixed.

- **Problem A** is a side-table called `term_vocab` — a private notebook the "find similar notes" feature keeps. It was supposed to mirror the real word index but was **never filled in for your existing 7,660 notes** — it only ever recorded words from notes you edited *while the app was running*. So it's about 8% complete and one of its counters (`doc_count`) is essentially blank. The good news: **nothing shows those wrong numbers to you** — the visible Index panel reads the correct live index. The only victim is the cross-language "≈ similar" suggestion row, which silently drops valid suggestions because its notebook is mostly empty.
- **Problem B** is that the main word index has accumulated **9 fragments** instead of being neatly compacted into one. This makes every search a little slower than it needs to be. It is **mild fragmentation, not severe** — a one-time "tidy up" (called `optimize`) will help, but it is a tune-up, not an emergency.

The cleanest fix for A is to **stop keeping the private notebook at all** and read the real index directly — which also happens to be exactly what Rule 8 (write-time derivation) tells us to do. B is a separate, standard background tidy-up.

---

## ISSUE A — term_vocab drift

### A.1 Root cause — definitively

It is **partial population**, *not* a key-namespace mismatch, *plus* a second independent defect: a **dead `doc_count` column**.

**The keys line up perfectly.** Both `term_vocab` and the live FTS5 dictionary `notes_vocab` store the *same* stemmed token form, byte-identical by construction — both are built from `process_word_for_fts` (`libraries.rs:3305`) over the same `note_meta.body_text`, via the shared `tokenize_to_vec` (`fts5_tokenizer.rs:434`). Empirically, **99.85%** of sampled `term_vocab` terms are found verbatim in `notes_vocab`; `knowledge`, the stem `knowledg`, diacritic Arabic (`عَسْكَرَ`), accented Latin (`étienne`), CJK, and Devanagari all key-match. **No namespace translation is needed.** This rules out the "mismatch" hypothesis decisively.

**Why it drifted — the specific gap.** `term_vocab` and `notes_vocab` are maintained by two unrelated mechanisms:

| | `notes_fts` / `notes_vocab` | `term_vocab` |
|---|---|---|
| Maintained by | **SQLite triggers** on `note_meta` (`search.rs:2935-2950`) — fire on *every* row write, including bulk first-boot indexing | **Rust hook** `ctse::hooks::on_note_indexed`, called *only* from `reindex_single_note` (`search.rs:8245`) |
| First population | Automatic — fires when the corpus is first walked into `note_meta` | **None — no bootstrap was ever built** |

The bulk indexer writes `note_meta` directly, which fires the FTS triggers (so `notes_vocab` is complete) but **never calls `reindex_single_note`** (so `term_vocab` stays empty). The one-time first-fill walk was explicitly **queued and then abandoned**: the comment at `search.rs:2808-2814` says *"MIG-013 §1A-§1C retired the prior bulk `populate_term_vocab` bootstrap… A first-fill walk… is queued for §1D"* — and §1D pivoted to query-time concept expansion instead (`lib.rs:502-512`). There is **no `INSERT INTO term_vocab … SELECT … FROM note_meta`** anywhere in the codebase. So `term_vocab` only ever gained the ~handful of stems from notes edited live since the table was created.

**The empirical fingerprint confirms this exactly:**
- `term_vocab`: **538,813** rows vs `notes_vocab`: **6,498,791** rows — only **~8.3%** of the corpus vocabulary is present.
- Coverage is **monotonic in frequency**, not by script: cnt>500 → 84.6% present; cnt 51–500 → 64.4%; cnt 6–50 → 23.7%; cnt 2–5 → 10.0%; **cnt=1 → 5.8%**. (Latin 90.7% missing, Arabic 96.2% missing — broad, so not a script filter.) This monotonic gradient is the signature of a population captured early and never completed — the head of the distribution got in, the long tail did not.
- `knowledge`: `term_vocab.doc_count = 2` vs `notes_vocab.doc = 1933`. `church`: 0 vs 1,293.

**The second, independent defect — `doc_count` is dead.** Even where rows exist:
- `term_vocab.doc_count == notes_vocab.doc` for only **0.08%** (3 of 3,994) of overlapping terms.
- **99.46%** of all rows have `doc_count = 0`; **MAX(doc_count) = 11** in a universe where `notes_vocab.doc` reaches **7,599**.
- (By contrast `total_count` is *approximately* right — matches `notes_vocab.cnt` for 95.9% of the overlap — because it tracks the same corpus, just stale.)

So `term_vocab` is broken on **two axes**: partial population (the head only) and a non-maintained `doc_count`.

### A.2 Who is harmed

**No user-facing surface displays the wrong number.** Verified: the only reader of `term_vocab` outside the maintenance hook is `ctse::search::ctse_search_terms_by_concept` (`ctse/search.rs:224`), and it runs **`SELECT term FROM term_vocab WHERE term IN (...)`** — **membership only**, never the counts. Its result scores come from the M11 concept vector cosine, not `doc_count`. The `apply_delta` read of `total_count` (`hooks.rs:155`) is internal insert-vs-update bookkeeping, not surfaced. **The visible Index panel reads the correct source** — `read_index_entries` (`libraries.rs:3618`) and the TF-IDF probe (`libraries.rs:4188`) both `SELECT … FROM notes_vocab`, the always-correct FTS5 dictionary.

**But the membership test is itself broken by the empty table.** The cross-language "≈ similar" expansion (Index panel) filters concept-expanded stems down to those present in `term_vocab`. Because the table is ~92% empty, **valid suggestions are silently dropped** for any stem that exists in the corpus but was never saved-while-running. The user sees fewer similar-term suggestions than they should — a silent under-delivery, not a wrong number.

### A.3 Severity

**P2 — silent feature degradation, no data corruption, no wrong number on screen.** The "≈ similar" row under-delivers across languages; everything else is correct. Not P1 (nothing visibly wrong, no integrity risk). Not P3 (a real, broad functional gap in a shipped feature, and Rule-8-relevant architectural debt — exactly the "hand-rolled drift-prone derived index" Rule 8 warns against).

### A.4 Fix options

**(a) Fix maintenance + one-time reconcile back-fill** (the `tag_counts` reconcile pattern). Build the abandoned §1D first-fill: a stamped, resumable background walk of `note_meta.body_text` → `tokenize_to_vec` → seed `term_vocab` with correct `total_count` *and* `doc_count`.
- *Pro:* keeps the existing consumer untouched; fills the table the way it was always meant to be filled.
- *Con:* **resurrects exactly the surface Rule 8 names as the anti-pattern** — a hand-rolled derived index parallel to the FTS5 dictionary, which will drift again on any code path that writes `note_meta` without the hook (the bulk indexer, future bulk ops). Two maintenance mechanisms for one fact. More code, more boot work, more long-term risk.

**(b) Re-point the consumer to `notes_vocab`, retire `term_vocab`'s counts.** Change the one membership query to test against `notes_vocab` (same keys, no translation) and stop relying on `term_vocab` for presence. `term_vocab.doc_count`/`total_count` become dead and can be dropped; `term_embeddings` (the semantic vectors, separately keyed on the same raw term form) is the real asset and stays.
- *Pro:* **this is the Rule 8 canonical move** — read the write-time-derived source of truth (`notes_vocab`) directly; one source, always current, zero drift, zero new boot work. Removes a whole maintenance mechanism and a 538K-row table.
- *Con:* `notes_vocab` is a 6.5M-row b-tree; the `WHERE term IN (...)` membership probe must be confirmed fast enough (it's a PRIMARY-KEY-style lookup per term — should be fine, but measure). Touches the CTSE read path.

**(c) Leave as-is.** Defensible *only* on the "no wrong number is shown" finding — but **rejected** under fix-what-you-discover: the "≈ similar" degradation is a real, discovered functional defect, not cosmetic.

### A.5 Recommendation for A

**Option (b) — re-point to `notes_vocab`, retire `term_vocab`'s counts.** It is the Rule 8 answer read straight off the data: `notes_vocab` already holds correct, always-current `doc`/`cnt` for all 6.5M terms, with identical keys. Option (a) would rebuild the exact anti-pattern Rule 8 forbids and guarantee future drift. **Concept (the horse):** *"Does this expanded term actually occur in the user's library?"* — answerable directly from the live dictionary; no private ledger required.

**Migration Rule:** **Yes — `/migration`.** It changes a read-path contract across the Rust↔CTSE boundary and retires a table/columns (schema + write-path). Predecessor Lookup Rule fires (retiring `term_vocab.doc_count`/`total_count` and a consumer query). Architect should confirm: (1) the `notes_vocab` membership probe latency on 6.5M rows; (2) nothing else reads `term_vocab` counts (already verified: nothing does); (3) whether `term_vocab` the table can be dropped entirely or must persist for `term_embeddings`' sake (they are separate tables — confirm the join shape before dropping).

---

## ISSUE B — FTS5 fragmentation

### B.1 Is it actually fragmented? — Interpretation

**Yes, but only mildly.** Read from the authoritative source (distinct `segid` in `notes_fts_idx`, not the opaque `structure` blob):

- **9 segments**: `segid` ∈ {1,2,3,4,5,6,7, 12, 18}.
- One dominant segment **`segid=12` holds 46,663 of 46,703 idx pages** — essentially the whole index (the original bulk build).
- `segid=1–7` are **1 page each**; `segid=18` is 33 pages — small recent-edit tails FTS5 automerge hasn't folded in.
- Blocks are dense: 55,983 blocks, avg **4,002 B/block** — *not* the bloated-small-block signature of severe fragmentation.
- `docsize` rows (7,660) match `note_meta` exactly — the doc set is consistent.

**Why it fragmented:** this is a **read-mostly universe**. The active DB has **no FTS5 segment maintenance at all** — confirmed: the only `'optimize'` call is per-cUniverse in `federation_prewarm()` (`search.rs:7586`), never on the active DB; the active-DB `PRAGMA optimize` at `search.rs:3973` is the query-planner ANALYZE (`sqlite_stat1`), a *different* operation that does nothing to segments. The active index relies entirely on FTS5 automerge fed by note-edit writes; with few edits, automerge rarely fires, so the small tails accumulate. The code already flags this exact gap: `search.rs:7459-7461` — *"if the active index ever fragments on a read-mostly universe, add a background, segid-gated, idempotent `INSERT INTO notes_fts(notes_fts) VALUES('optimize')` on the active DB."*

**Honest caveat:** 9 segments is far from the "50+ segments" pathology the cUniverse comment predicts. An OR-of-N-terms query fans across ≤9 segments but 7 are single-page, so the real cost is the one big segment plus a handful of tiny seeks. Optimize collapses this to **1 segment**, removing 8 redundant doclist seeks per term — a **legitimate, low-risk win, but expect a modest improvement, not a 9× one** on this DB. The reported ~20ms fts5vocab and ~180ms single-term MATCH are consistent with reads fanning across these 9 segments *plus* traversing the 6.5M-row vocab b-tree — optimize helps the segment-fan component, not the b-tree size.

### B.2 Decisive check (for the orchestrator)

**Run `'optimize'` on a *copy* of `search.db` and time the same probes before/after** (fts5vocab lookup + single-term MATCH). This is the only way to confirm how much of the latency is fragmentation vs. raw vocab-size. Note: `'optimize'` requires the custom `constellation` tokenizer registered on the connection (`register_fts5_tokenizer`, `search.rs:7554`) before opening `notes_fts` — the read-only Python path cannot do it; the Rust harness must.

### B.3 Fix

A new **`fts_optimize_backfill`** module mirroring the existing background-maintenance pattern (canonical reference: `review_backfill.rs` — `AtomicBool` single-run guard, `thread::spawn`, dedicated `Connection::open` + `register_fts5_tokenizer` + `busy_timeout`, returns immediately), wired into `ensure_search_db_ready` alongside the other seven `maybe_schedule` calls (`search.rs:7715-7751`).

**Critical design difference from the other backfills:** the simple `is_stamped`-and-skip-forever pattern is **wrong** here, because fragmentation **recurs** as edits accumulate. Use a **segid-gated re-arm**: each boot, read distinct `segid` from `notes_fts_idx`; run `'optimize'` only when it exceeds a threshold (e.g. >8). This matches the `search.rs:7460` spec ("background, segid-gated, idempotent").
- **Bug to fix while here:** the prewarm probe reads `SELECT MAX(segid) FROM notes_fts_data` (`search.rs:7571`/`7591`) — that column **does not exist** on `_data`; it silently logs `-1` via `unwrap_or(-1)`. The correct source is **`notes_fts_idx.segid`**. The new module must use `_idx`, and the prewarm probe should be corrected to match.

**Boot-safety / measurement requirements:** non-blocking (`thread::spawn`, returns immediately); never on the keystroke path; dedicated connection so it never blocks the main connection; `busy_timeout` so it yields to live writes; `'optimize'` only when the segid gate trips (read-mostly universes optimize once then stay quiet). Measure boot time and the two probe latencies before/after on this 7,660-note DB; must not regress boot.

**Migration Rule:** This is a **borderline `/migration`**. It adds a write operation (`'optimize'` mutates the FTS shadow tables) on the boot path and a new `schema_versions`/gate interaction. It does *not* change schema shape or any read/write *contract*, and it precisely follows an existing, blessed pattern the code already specified. Recommendation: **lightweight `/migration`** (Architect + a short Plan with the before/after timing as the verification clause), because it writes to the index on boot — but it does not need the full four-phase weight that Issue A needs.

---

## Relationship between A and B + sequencing

**They are independent problems with independent root causes.** A is "a derived side-table was never populated" (a missing back-fill in CTSE). B is "the FTS5 segments were never compacted" (a missing maintenance task on the live index). They share only a neighborhood (both touch the FTS5 layer) and a moral (both are write-time-derivation hygiene that was deferred and never finished).

- **Does fixing B help A?** Only marginally and only under Option (a). If A were fixed by reading `notes_vocab` (Option b), B's optimize would speed up *that* read too — but that's a general search speedup, not an A-specific fix. They do not depend on each other.
- **Order:** **Do B first.** It is smaller, lower-risk, follows a pattern the code already specifies, speeds up *all* search (Index panel, SearchHub, and the future `notes_vocab`-based membership probe), and the decisive optimize-on-copy timing test gives a clean go/no-go before any code lands. Then do A — and A's Option (b) directly *benefits* from B having compacted the index it will now read.

---

## Recommended next actions (ordered)

1. **B-0 — Decisive timing test (simple, no `/migration`).** Orchestrator: copy `search.db`, register the tokenizer, time fts5vocab lookup + single-term MATCH, run `INSERT INTO notes_fts(notes_fts) VALUES('optimize')`, re-time. *Verification:* report before/after ms and the post-optimize distinct-`segid` (expect → 1). Decides whether B proceeds.

2. **B-1 — `fts_optimize_backfill` (lightweight `/migration`).** Build the segid-gated background optimize module per `review_backfill.rs`, wire into `ensure_search_db_ready`, and fix the `notes_fts_data`→`notes_fts_idx` segid-probe bug in `federation_prewarm`. *Verification:* on the 7,660-note DB, boot time not regressed; distinct `segid` drops below the gate after first run; the two probe latencies improve by the margin B-0 measured; second boot runs the gate, finds it satisfied, and does *not* re-optimize.

3. **A-1 — Re-point CTSE membership to `notes_vocab`, retire `term_vocab` counts (full `/migration`).** Architect (Predecessor Lookup entry for retiring `ctse_search_terms_by_concept`'s `term_vocab` query + `doc_count`/`total_count`; confirm `term_vocab` vs `term_embeddings` separation before dropping the table) → Plan → Build → Audit. *Verification:* the "≈ similar" row now returns cross-language suggestions for terms present in the corpus that were previously dropped (Boss-testable: open a note, check the ≈ similar suggestions are richer than before); membership-probe latency on `notes_vocab` measured and within budget; no read of `term_vocab` counts remains anywhere.

**One-line summary for Eisa:** Two deferred clean-ups surfaced — a private word-list that was never filled in (fix: stop keeping it, read the real index — `/migration`) and an un-compacted search index (fix: a one-time background tidy-up — small `/migration`). Do the tidy-up first; neither is breaking anything you can see today, but both are worth fixing properly.