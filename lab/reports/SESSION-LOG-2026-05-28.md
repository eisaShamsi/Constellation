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

---

## Block 2 (mid-morning) — MIG-060 Phase 1.5 §A-§F (threading gestures)

After MIG-058+059 closed with §L PCS, Eisa confirmed the roadmap return: *"PCS + Orientation > back to the remaining Constellation Base, right?"* → proceed.

The next trunk MIG is **MIG-060 — Constellation Base Phase 1.5: Host-Note Threading Gestures**. Each lens row gets three small icon buttons on its trailing edge that open the host note in 360.3D / CNS / Cataloger — the deep-read surfaces that previously required a dock-click after the note opened.

### Architect + Plan landed in §211adceb (yesterday)

Locked design:
- Single custom event `constellation:open-note-in-surface` with `detail.surface` discriminator.
- UI: 3 inline buttons per row; 12px icons; always visible (CNS only gated by user feature flag).
- Navigation: open host note → `await tick()` → flip target surface flag (exclusive-surface clear pattern).
- 7-step Plan (§A i18n → §B widget → §C listener → §D CSS → §E tests → §F Boss-test → §G PCS).

### Build cascade (today)

| § | Commit | What shipped |
|---|--------|-------------|
| A | `8e76f545` | 45 new i18n keys (15 locales × 3 tooltip strings). Native equivalents per Eisa's full-localization rule. |
| B | `f8e374c8` | `LensBlockWidget._renderRow` — three buttons per row with stopPropagation + CustomEvent dispatch. CNS gated by `enabledFeatures.constellationSight !== false`. |
| C | `a8420ab0` | `+layout.svelte` listener — opens host note (`await openNoteTab`), then `await tick()`, then flips the requested surface flag in an exclusive-surface clear. Imports `tick`. |
| D | `49ac3da6` | CSS for `.cm-lens-row-actions` + per-surface hover hues (purple/cyan/orange). `marginInlineStart:auto` auto-flips LTR↔RTL. |
| E | `b5e35112` | 52 vitest tests pass (45 i18n parity + 6 surface-discriminator + 1 sanity guard). New `tests/mig-060/` directory + `test:mig-060` npm script. DOM-render tests deferred to §F Boss-test per scope-vs-effort. |
| F | `77f917dc` | `docs/MIG-060-BOSS-TEST.md` — 5-stage tutorial per Testing Instructions Rule. Eisa runs this next. |

### Verification status

- svelte-check: only the 3 pre-existing errors (no new ones introduced).
- Vitest: 52/52 pass on the new test suite.
- Vite frontend build: clean in 1m 53s.

### Awaiting Boss-test

Stage 1-5 of `docs/MIG-060-BOSS-TEST.md`. Per Eisa's staged-tests rule, Claude will surface Stage 1 in chat first, wait for findings, then proceed.

### What's next after Boss-test

§G PCS — orientation v2.40 + MoCh + 15-locale help-doc updates + milestone tag `milestone/mig-060-base-phase-1.5-shipped` + ZIP backup.

After MIG-060 closes, the Constellation Base roadmap continues:
- Phase 2 — Living Link Columns (separate MIG).
- Phase 2.5+ — Bridges (360.3D / CNS / Cataloger as lens DIMENSIONS, not just gestures).
