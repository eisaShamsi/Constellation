# MIG-012 — Index Search Engine: Search History + Semantic Search

**Date opened**: 2026-05-04
**Status**: Phase 1 (Architect) — **Boss design approval required before Phase 2**
**Owner**: Index closure work-stream (final phase after MIG-010 + MIG-011)
**Composes with**: MIG-010 (mentions bridge), MIG-011 (filter bridge), the existing `embeddings.rs` ONNX pipeline.

---

## §1 · Goal

Make the Index filter box a real **search engine** rather than a substring + bridge filter:

1. **Search history** — the filter remembers prior queries within the same Universe, surfaces them on focus / down-arrow / dropdown. The user doesn't retype "knowledge" every time they open the Index; they recall it from history.

2. **Semantic search** — beyond substring (current) and lexical-bridge (MIG-011), add **conceptual** search: typing "thinking" surfaces terms semantically related to thinking even when there's no lexical-bridge edge — "cognition", "reflection", "metacognition", "rumination". Powered by the ONNX embedding pipeline already shipped in `embeddings.rs`.

Together, the three layers give the user:
- **Substring** — literal text matching (instant).
- **Lexical** — translation equivalence via the M11 corpus (instant after debounced IPC).
- **Semantic** — conceptual neighborhood via embeddings (slower; cached).

Plus history surfaces the user's most-used queries on first focus.

---

## §2 · Boss design decisions required

These three are not "Architect picks; Boss reviews" — they're **real architectural forks** where the choice shapes weeks of future work. **Plan-Approval = Build-Approval cannot apply** until Boss picks.

### Decision Q1 — Embedding granularity: term-level OR note-level?

| Option | Mechanics | Storage | Behavior |
|---|---|---|---|
| **Q1.A — Term-level embeddings** | Embed every Index term (50-100k vectors on a 7,600-note library). Store in a new `term_embeddings` table. Filter searches against term embeddings via cosine similarity. | ~50k × 384 dims × 4 bytes ≈ 75 MB on disk per Universe. Boot-time materialization or lazy-on-Index-open. | The filter result IS the semantic answer: typing "thinking" returns Index TERMS like "cognition", "reflection". Same shape as substring + lexical. |
| **Q1.B — Note-level embeddings** | Reuse the existing `note_embeddings` table (already shipped). Filter queries embed → cosine match against note vectors → collect the term-vocabulary of matching notes → surface those terms in the filter list. | Zero new storage. | The filter result is "TERMS appearing in NOTES that are semantically close." Indirect, but reuses what exists. May surface noise (terms that appear in semantically-relevant notes but aren't themselves the concept). |
| **Q1.C — Both, fused** | Q1.A primary; Q1.B as a recall-boost when term-level returns thin results. | ~75 MB + reuses note_embeddings. | Best quality, most complexity. Two embedding pipelines in lockstep. |

**Recommendation**: **Q1.A — term-level**. Reasoning:
- The Index is a vocabulary-browsing surface; semantic neighbors of a TERM are the right unit.
- Note-level embeddings produce noise — typing "thinking" might surface a term like "Wikipedia" because some "thinking" note happens to cite Wikipedia.
- 75 MB per Universe is acceptable on local-first (Boss's library is 7,600 notes; 50k terms is the upper bound).
- Materialization is one-time per Universe; future incremental updates are cheap (add a term → embed it once).

But I can be wrong. If Boss prefers **reuse over freshness**, Q1.B is the lighter path.

### Decision Q2 — When do embeddings happen?

| Option | Trigger | UX |
|---|---|---|
| **Q2.A — Boot-time materialization** | First Universe open after MIG-012 install runs an async embed-all job. Progress bar in the status bar. ~10-20 min on a 50k-term library, depending on hardware. Subsequent boots are zero-cost (cache present). | One-time wait at install. Then it's free forever (with incremental updates as new terms appear). |
| **Q2.B — Lazy-on-first-Index-open** | Same job fires the first time the user opens the Index after install. Same 10-20 min, in-panel progress. | First Index-open is slow; subsequent opens free. |
| **Q2.C — Lazy-on-first-semantic-query** | The job fires the first time the user toggles "Semantic search" on. | Most users never trigger the cost (if they don't enable semantic). Best for opt-in. |
| **Q2.D — On-the-fly per query** | Embed the query at search-time, do nearest-neighbor over an embeds index built lazily. No bulk pre-compute. | Each query takes ~1-5s (slow). Unacceptable for an interactive filter box. |

**Recommendation**: **Q2.C — lazy-on-first-semantic-query**. Reasoning:
- Semantic search is opt-in by default (a new Settings toggle, like the existing cross-language one).
- Users who don't enable it pay zero cost.
- Users who DO enable it accept a one-time wait.
- Boss can flip the toggle on a Sunday afternoon and let the index build while doing something else.
- Q2.A is also viable if Boss wants this to "just work" without thinking about it.

### Decision Q3 — Search history scope and storage?

| Option | Storage | Scope |
|---|---|---|
| **Q3.A — localStorage (per-Universe key)** | Browser localStorage, key `constellation.indexHistory.<universe-name>`. ~5KB cap easily holds 200+ entries. | Per-Universe. Cleared with browser data; not synced. |
| **Q3.B — SQLite (per-Universe `index_search_history` table)** | New table in the existing per-Universe `search.db`. | Per-Universe. Survives browser-data clears. Backed up with the rest of the Universe. |
| **Q3.C — SQLite + cross-Universe global** | Two tables: per-Universe + a global. Surface both with visual grouping. | The user's "frequent queries" surface across Universes; per-Universe history is the recent context. |

**Recommendation**: **Q3.B — SQLite per-Universe**. Reasoning:
- Constellation is local-first; SQLite is the right place.
- Per-Universe matches the rest of the app's data model.
- Survives browser-data clears (some users do this for privacy; their history shouldn't vanish).
- Cross-Universe (Q3.C) feels overengineered until a user reports needing it.

---

## §3 · Surface design (post-decisions)

Assuming Q1.A + Q2.C + Q3.B:

### Settings → Index — three new controls

- **`Expand mentions cross-language`** (existing, MIG-010 + MIG-011).
- **`Semantic search`** (NEW) — toggle. When on, the filter ALSO does cosine-similarity matching against term embeddings.
- **`Search history`** (NEW) — toggle. When on, recently-used queries surface in a dropdown on filter-box focus and via down-arrow.
- **`Clear search history`** (NEW button) — one-click clear, confirmable.

### Filter results presentation

When all three layers are active, results are shown in priority groups within the same flat list:

1. **Direct substring matches** (top, no badge — current).
2. **Lexical bridge matches** (`via {lemma}` badge — MIG-011).
3. **Semantic matches** (NEW: `≈ {your-query}` badge or some equivalent indicator).

Sort within each group: by frequency (existing behavior).

### Search history dropdown

- Triggered by focus on the filter box OR pressing down-arrow.
- Shows last ~20 queries, most recent first. Cap at 200 stored.
- Click an item → it pre-fills the filter.
- Right-click an item → "Remove" / "Clear all".
- Hidden when the user starts typing (substring/lexical/semantic results take over).

### IPC additions

- `init_term_embeddings(library_path)` — fires the lazy embed-all job. Returns a stream of progress events.
- `search_terms_semantic(query, top_k)` — embed query → cosine match → return top K terms with similarity scores.
- `read_index_history(limit)` / `write_index_history_entry(query)` / `clear_index_history()` — three IPCs for history management.

---

## §4 · Invariants that must not break

| # | Invariant | How verified |
|---|---|---|
| **S1** | All three feature toggles default OFF. Pre-MIG-012 behavior is byte-identical for users who never opt in. | Plan |
| **S2** | Each layer is independently togglable: semantic ON, history ON, lexical OFF is a valid state. | Plan |
| **S3** | Semantic embed-all is interruptible + resumable. Power loss mid-job → resume on next open. | Build |
| **S4** | Term embeddings are incrementally updated when new terms appear in `notes_vocab` (via FTS5 triggers). No need to rebuild. | Build + Audit |
| **S5** | The semantic IPC is debounced (≥300ms, same as MIG-011). | Build |
| **S6** | Search history IPC writes are non-blocking (deferred via `setTimeout` or similar). The filter never waits on history persistence. | Build |
| **S7** | RTL: dropdown layout flows correctly; semantic-match badge reads naturally in Arabic. | Build (Boss test) |
| **S8** | No new boot-perf regression. Term embeddings only load on first semantic query (per Q2.C). Boot stays free. | Audit |
| **S9** | Cooccurrence chip-strip and mentions list are unaffected (separate paths). | Audit |
| **S10** | i18n complete in 15 locales for all new Settings labels, dropdown affordances, badges. | Audit |
| **S11** | Settings → Clear search history requires confirmation (irreversible). | Build |
| **S12** | Semantic match scores are normalized 0-1; UI displays only "matched" / "didn't match" — not raw scores (opaque to users). | Build |

---

## §5 · Phased plan preview

Sketched (not committed until decisions land):

1. **Build.1** — `term_embeddings` table + incremental population path via FTS5 trigger.
2. **Build.2** — `init_term_embeddings` IPC + progress events.
3. **Build.3** — `search_terms_semantic` IPC + 4 unit tests.
4. **Build.4** — Frontend wrapper + debounce + cache (mirror MIG-011 pattern).
5. **Build.5** — Filter loop adds semantic match path + sort + badge.
6. **Build.6** — Search history table + 3 IPCs.
7. **Build.7** — Frontend history dropdown + Settings toggles.
8. **Build.8** — `/simplify` checkpoint.
9. **Audit** — Phase 4.

That's roughly 2-3 sessions of work for the build alone.

---

## §6 · Risk register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Term-level embedding quality is bad on stems vs. surface forms | Medium | The constellation tokenizer produces stems; embedding stems may compress meaning poorly. **Verify with a quality bench** before Build.5. If poor, fall back to surface-form embedding (more storage but better signal). |
| 50k embed job blocks the UI thread | Low | All embedding goes through Tauri background tasks. Frontend gets progress events; UI stays responsive. |
| Search history grows unbounded | Low | Cap at 200 entries; FIFO eviction. Settings → Clear History as escape valve. |
| Semantic matches feel noisy / unhelpful | Medium | Boss G-test gates the quality. If unacceptable, add a similarity threshold (e.g. only show top-K matches above 0.7 cosine). |
| Embeddings table version skew when models update | Low | Store model version with the embeddings; invalidate on model change. (Boss-deferred; revisit in a future MIG.) |

---

## §7 · Boss approval needed

**Three decisions, recommendations in §2:**

- **Q1**: term-level (A) vs note-level (B) vs hybrid (C) embeddings? **Recommended: A.**
- **Q2**: When does the embed-all job fire? Boot (A) / first Index open (B) / **first semantic query (C, recommended)** / per-query (D, rejected)?
- **Q3**: Search history storage? localStorage (A) / **SQLite per-Universe (B, recommended)** / SQLite + global (C)?

**Once Boss picks, the cascade through Plan + 8 Build steps + Audit is autonomous per Plan-Approval = Build-Approval.**

If Boss wants to override any of my recommendations, surface during this Phase. After Plan, scope is locked.

---

**Phase 1 closes here. Awaiting Boss decisions on Q1, Q2, Q3.**
