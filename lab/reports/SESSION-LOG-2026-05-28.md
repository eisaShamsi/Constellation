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

---

## Block 3 (afternoon) — MIG-060 Boss-test sub-fixes + Federation Audit

Boss-test moved beyond Stage 2 (360.3D) and hit successive surface-focus gaps. Plan had treated all three surfaces uniformly; reality required three custom focus mechanisms.

### MIG-060 sub-fixes

| Commit | What |
|---|---|
| `5114ce88` (§C-fix) | CNS focus: `focusNoteId?: string` prop on `ConstellationSight2.svelte`; onMount after `fitToScreen()` finds matching SimNode, sets `selectedNode`, pans canvas. `pendingCnsFocusPath` $state in `+layout.svelte` set before `toggleLens()`, cleared in `onClose` / `onNoteClick`. |
| `99ae76fb` (diag) | Temporary `diag_log_line` tracing — revealed `focusNode lookup: NO MATCH` for "Eisa ALSHAMSI" because the note isn't in CNS's gravity well (orphan, not in linked subgraph). Reverted in next commit. |
| `1ce715ed` (§C-fix-2) | Orphan-hide: new `skyNodePathSet` writable store in `libraries/store.ts` mirrored from `skyNodes` via `$effect`. LensBlockWidget skips CNS button render when row's note isn't in the set. Removed diagnostic traces. |
| `16b31c57` (doc) | Boss-test doc correction: Settings path is **Core Plug-Ins** (not "Features" — fabricated path was a BASIC-RULE violation). Clarified CNS = Constellation Nervous System (live core surface), distinct from retired Constellation Sight (the dome view, future plugin per MIG-038). |
| `e7baaadb` (§C-fix-3) | Cataloger focus: `case 'cataloger':` branch dispatches `constellation:classify-and-show` (existing event) one rAF after `showCataloger = true`. SourceReviewPanel's listener (already in place from MIG-039) picks up the path and focuses on that note's classification card. |

### Boss-test final verdict

| Stage | Result |
|---|---|
| 1 — Visual (3 icons appear) | ✓ pass |
| 2 — 360.3D gesture | ✓ pass (focusedTab auto-read) |
| 3 — CNS gesture | ✓ pass on Eisa Cognitive Knowledge universe (single-universe, no federation gap) |
| Check A — orphan-hide | ✓ pass (CNS icon hidden on orphan rows) |
| 4 — Cataloger gesture | ✓ pass on linked note; ✗ for orphan/federated note with FK error (pre-existing Cataloger federation gap) |
| 5 — RTL parity | ✓ already passed in Stage 1 |

MIG-060 ships. The Cataloger FK error during Stage 4 = pre-existing federation gap (not MIG-060's bug) → triggered the audit.

### Federation Audit (Block 3 continued)

Eisa requested broader scope: *"I also want to check how the remaining functions/core plugins are handling Universes with cUniverse(s) included, like mine."*

Four parallel exploration agents surveyed the codebase:
- Agent 1 — graph surfaces (CNS, Sky View, Map, 360.3D)
- Agent 2 — Cataloger / Classifier / NSC backends
- Agent 3 — sidebar panels (Backlinks, Outgoing, Mentions, Tags, Bookmarks, Five Acts, Bases)
- Agent 4 — search / index / dock surfaces (libraryStats, Search, Lens, Index, Knowledge Health, Federation Warnings, etc.)

Findings doc: `docs/MIG-061-federation-audit-findings.md`. Summary:

- ✓ Federated (4 surfaces): libraryStats, Search Hub, Lens execution, Federation Warnings popup.
- N/A by design (5): 360.3D, Bookmarks, Global Tasks, Expression Forge / Sense-Making Canvas / Dashboard.
- ◑ Partial (1): Org Chart (`constellation_map_universe`) — tree includes cUniverses; alias_map parent-only.
- ✗ Broken (14): CNS, Sky View, Backlinks, Outgoing, Unlinked Mentions, Tag Browser, Five Acts sidebar, Workspace Bases, The Cataloger, Classifier (scan + single-note), NSC Backfill, Index panel (entries + mentions), Knowledge Health, right-sidebar previews.

Four root-cause patterns:
1. **P1** — `cache_boot_snapshot_sky` not federated → 4 surfaces (CNS, Sky View, Backlinks, Outgoing).
2. **P2** — Backend uses bare `state.db` instead of `state.federated_conn` → 6 surfaces.
3. **P3** — Hardcoded `{active_universe}` filesystem paths → 3 surfaces.
4. **P4** — FK constraints to parent's `note_meta` → compounds P2 for write paths.

Three scope options surfaced to Eisa (A: mega-MIG / B: 4 pattern-MIGs recommended / C: just MIG-061 CNS). Awaiting Boss decision.

### What ships in this commit set (combined MIG-060 §G + Audit PCS)

- Orientation v2.40 capturing both MIG-060 close and audit findings.
- This session-log update (Block 3).
- Findings doc `docs/MIG-061-federation-audit-findings.md`.
- Milestone tag `milestone/mig-060-base-phase-1.5-shipped`.
- 15-locale help docs + MoCh deferred to MIG-061+ PCS (will batch together when the first federation fix ships).

---

## Block 4 (evening) — MIG-061 P1 federation fix shipped

The first of the four federation-fix MIGs (Option B locked in Block 3). Federated `cache_boot_snapshot_sky` AND `cache_boot_snapshot_graph` — the two boot-snapshot IPCs that feed CNS, Sky View, Backlinks, Outgoing Links, and (as side effect of §M's graph federation) Tag Browser.

### Architect + Plan committed (2783d622, f35e0b7e)

Four-question Boss lock per Architect §8:
- Q1: Rust per-schema loop+merge (Option 2)
- Q2: id=lower(name) tolerated; path disambiguates (Option C)
- Q3: per-schema link isolation (Option A — strict reading of Boss principle, departed from §5 recommendation of Option B)
- Q4: all-or-nothing readiness (Option A — departed from §5 recommendation of partial federation)

Wait — Q3 was originally locked Option B; Eisa later (§L) corrected to Option A based on his "no merge" principle. Documented in the Architect §8 update.

### Cascade (17 commits)

| § | Commit |
|---|---|
| A | `6b5173fa` get_federated_schemas |
| B | `76f9f826` read_sky_nodes_raw_in_schema |
| C | `dc43e753` read_sky_links_raw_in_schema |
| D | `4df77b6d` is_federated_sky_ready (Q4 all-or-nothing) |
| E | `ade3a010` cache_boot_snapshot_sky federation loop |
| G | `1d755755` 8 unit tests |
| H | `e69417af` Boss-test doc |
| J | `e05be00a` federation:ready event emit |
| J.2 | `1a500cf4` listener-order fix + defensive re-invoke |
| (diag) | `99ae76fb` `1782a123` diagnostic tracing (later removed in §O) |
| K | `617f4302` **stratum column-type latent bug fix** |
| L | `c62f8c53` Q3 → Option A per-schema isolation (Boss principle correction) |
| M | `7f648a55` federate cache_boot_snapshot_graph |
| N | `0c6f7661` listener re-fetches graph too |
| O | `25562627` remove diagnostic tracing |
| P+Q | `3b823085` audit follow-ups (D4 guard + 2 unit tests) |

### Boss-test marathon — 6 stages, 8 binary rebuilds

Stage 2 (federated count) failed THREE consecutive times before §K surfaced the real bug:
- Build 1: §A-§G shipped → CNS still 987 nodes → §J added event emit.
- Build 2: §J + §J.2 → still 987 → diagnostic tracing added.
- Build 3: §J.3 trace → revealed `Invalid column type Text at index: 4, name: stratum` → §K fix.
- Build 4: §K → CNS shows **8 751 nodes · 233 286 links** ✓ pass.

Stages 4-5 (Backlinks/Outgoing) failed once after Stage 2 passed → §M federated _graph too → Stage 4 = 104 backlinks, Stage 5 = 101 outgoing links ✓.

### Two technical inflection points

1. **§K (stratum column-type bug)** — Pre-MIG-061 latent bug surfaced by the diagnostic trace. `cache_boot_snapshot_sky` has been silently failing in production since the original sky_nodes code was written. Frontend silently fell back to `buildSkyData` (legacy non-federated path). My MIG-061 federation worked correctly but production never reached the federation code because the SQL row-read crashed first.

2. **§L (Eisa's principle correction)** — Boss intervened with the structural insight: *"The Federation should be simple. The app shouldn't reinvent the wheel; the wheel is already there!"* Architect's Q3 lock (Option B = federated link resolution) was corrected to Option A (per-schema isolation). The merge logic in §E was refactored to per-schema loops. Standalone-A behaves identically to A-as-cUniverse-of-B.

### Audit (3 parallel agents)

- **Invariant-check: 8/8 UPHELD** (7 documented + new INV-K for flexible-stratum-read).
- **Drift detection:**
  - D4 MEDIUM — empty-overwrite race in listener (fixed in §P).
  - D5 LOW — no direct unit tests for §M (fixed in §Q).
  - D6 MEDIUM-positive — §M closes Tag Browser as side effect.
- **Migration paths: 6/7 PASS** (S1/S2/S3/S5/S6/S7 PASS or degrade gracefully). S4 (pre-MIG-061 rollback after MIG-061 write) FAILs but is pre-existing risk.

### What ships in this commit set (§R PCS)

- Orientation v2.41 (this version) captures MIG-061 close.
- Session log Block 4 (this section).
- Milestone tag `milestone/mig-061-cns-federation-shipped`.
- ZIP backup.
- 15-locale help-doc updates deferred to MIG-062 batch (Eisa fatigue: 6+ hours of debugging today).

### What's next

- MIG-062 (P3): Federate `list_five_acts_notes` + `list_workspace_bases` filesystem walks. (Tag Browser was P3 originally but is now closed by §M.) Small MIG, ~3 commits.
- MIG-063 (P2 read): Federate Index entries + Index mentions + Unlinked Mentions + Knowledge Health + right-sidebar previews. ~5 commits.
- MIG-064 (P2+P4 write): Federate Cataloger + Classifier + NSC + their FK constraint. Architect needs to resolve the schema-design question first.

### Two pending polish items (PJs logged)

- PJ-NNN-A: Sky View node size scale for federated view.
- PJ-NNN-B: CNS gravity well full-canvas layout when window is maximized.
