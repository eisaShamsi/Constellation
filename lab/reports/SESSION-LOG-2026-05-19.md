# Session Log — 2026-05-19

## Function in hand

**MIG-036 — Sight v7 (Form-Aligns-To-Purpose redesign).** Replacing
v6's per-note remap contract (every star gets a position inside a
tradition's grammar — within-quadrant angular jitter, hash-jittered
ladder slots, etc.) with a CATEGORICAL contract: each tradition
declares its cells; at universe view the cell IS the unit (density-
blob magnitude encodes population), drill-in expands the cell to a
stack of individual notes. Time gets dropped from the tradition list
and becomes its own dome (the Time Dome). Architect doc at
`lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md`.

This day's session log captures the full MIG-036 setup + the first
three build phases (P1 scaffolding, P2 rendering primitives, P3
universe-view dispatcher + first sample tradition).

---

## Pivot: MIG-029 → MIG-036

MIG-029 §ν.5 (per-note frontmatter wiring for the 9 tradition-kind
fields, shipped at 6b7eaff0) hit a wall in Boss test: Eisa's frontmatter
edits (`masadir_source: sunnah` on three notes) didn't move the stars
to the sunnah wedge. Three patch iterations failed:

- `732d2b4c` MIG-029 §ν.5-fix — opportunistic refill of missing
  cache rows (failed: cache rows existed but were stale)
- `feffbd4e` MIG-029 fix-2 — widened to STALE rows via
  `computed_at < modified*1000` check (failed: `note_meta.modified`
  itself wasn't updating)
- `d41acc50` MIG-029 fix-3 — always refill all cache rows on
  `get_layout` (still failed)

Root cause traced to `index_note`'s cache-hit short-circuit at
`search.rs:3004` returning early when file mtime matched, so the
write-time refresh of `note_meta` never fired. Carried as PJ-060
into v7 close-out — a focused mini-MIG (chunked UPDATE + Tauri
progress events).

Eisa: "Stop patching, and try to restructure the whole thing if
needed."

Architectural reframe followed. Two issues surfaced:

1. **Time vs. categorical conflict.** All four sample notes were
   created in April 2026; per the v6 sectoral layout that put them
   in the same temporal wedge regardless of their masādir source.
   Eisa flagged this: "I think the traditions quadrant will not work
   with the time (months) rim. It doesn't match, and it's illogical."

2. **Aristotelian shouldn't BE the time view.** "If Aristotelian is
   just to display the time, then why are we calling it this? Instead,
   the Traditions (including Aristotelian) should look at the
   knowledge-cognition lens based on their design. If I want to
   display time, I will call this 'the Time Dome'."

Decision: drop time from tradition rim; add the Time Dome as a
separate first-class dome.

Then the **hash-jitter problem**. My proposal to "use" the angular
axis inside Aristotelian's stratum rings by hash-jittering notes
triggered Eisa's hardest correction of the day:

> "You said: Angular = hash-jittered (no meaning — just visual
> spread within each stratum ring). We shouldn't design a
> non-meaningful function… What is the message here? That we are
> SMART? Each function should serve its core purpose, and its
> design and behavior should be aligned. I want to have this as
> a key rule in Constellation design."

---

## Commit: Form-Aligns-To-Purpose rule (`30f1d6a7`)

Added the new top-principal rule to `CLAUDE.md` between
"Constraint as Design" and "Language-First by Design", verbatim
per Eisa's "Wording Approved" approval. Includes the special
application clause for traditions (every visual element under a
tradition must derive from its theory's structure — never filler
to occupy axes the chosen primitive affords but the cognitive
content doesn't fill).

Saved to persistent memory as `feedback_form_aligns_to_purpose.md`.

Canonical violation prevented: 2026-05-19 hash-jitter proposal.

---

## Commit: MIG-036 Architect (`b6dcbdef`)

`lab/reports/MIG-036-SIGHT-V7-ARCHITECT.md` — full Sight v7
architecture, ~243 lines. Covers:

- §1 Why v7 (the diagnosis: per-note positions don't carry signal)
- §2 What stays universal across all traditions
- §3 What each tradition declares (cells, not positions)
- §4 Hybrid X+Y zoom-driven model (density at universe view, stack
  at drill-in)
- §5 v7 contract (`TraditionModuleV7` — cellRegions + cellMembership)
- §6 Time Dome as separate module (opt-in calendar rim)
- §7 Drill-in interaction (P7 — cell click expands)
- §8 Schema reuse (no new IPCs, no new cache tables — v7 reuses
  v6's `LayoutCacheRow`)
- §9-§12 per-tradition redesign matrix (24 traditions × new contract)
- §13 Phases P1-P13 with verification clauses per phase

Plan-Approval = Build-Approval. Eisa: "I like the Hybrid X+Y concept.
Let's adapt it." Then "A" (cascade now in session).

---

## Commit: MIG-036 P1 (`22eb7cee`) — Scaffolding

B2 dual-mount per Eisa's prior pattern (v5→v6→v7 all coexisted at
their respective ship moments).

- `SIGHT_V7_ENABLED = false` in `engine.ts` (gated off so v6 stays
  default during dev)
- `src/lib/sight/v7/SightV7.svelte` — minimal placeholder mount
- `src-tauri/src/sight_v7.rs` — module shell, re-exports
  `LayoutCacheRow` from sight_v6 (Architect §8 — schema reuse)
- `src-tauri/src/lib.rs` — `pub mod sight_v7` parallel to sight_v6
- `src/routes/+layout.svelte` — v7 dock button + modal mount,
  visible only when flag is true

---

## Commit: MIG-036 P2 (`9a2497a0`) — Rendering primitives

Pure-function modules — no canvas calls, no IPC, no DOM. Tested in
isolation via vitest.

- `src/lib/sight/v7/density.ts` —
  - `computeDensityScale(populations)` log-scale anchor
  - `densityRadius(p, scale)` log-scaled magnitude → pixel radius
  - `densityOpacity(p, scale)` log-scaled → alpha [0.5, 1.0]
  - `cellDensity(id, label, p, scale)` composite struct

- `src/lib/sight/v7/stack.ts` —
  - `buildStack(rows, resolver, sort)` projects `LayoutCacheRow[]`
    to `StackRow[]`
  - `sortStack(stack, sort)` — 'stratum' | 'created' | 'modified'
    | 'title' | 'hash'
  - `filterCell(rows, predicate)` thin helper

- `tests/sight-v6/v7-density.test.ts` — 14 unit tests
- `tests/sight-v6/v7-stack.test.ts` — 10 unit tests
- `vitest.config.ts` — added v7 test files to the include list

Tests: 82/82 pass (58 existing + 24 new v7 tests).

---

## Commit: MIG-036 P3 (this commit) — Universe-view dispatcher

The first phase that produces something visible. Wires the P2
primitives + the new v7 tradition contract into a working render.

- `src/lib/sight/v7/types.ts` — TraditionModuleV7 contract
  - `cellRegions(layout): CellGeometry[]` (per-render-pass cell
    declaration; geometric centers + hit-test radii)
  - `cellMembership(row): string | null` (per-note predicate)
  - `showCalendarRim?: boolean` (only Time Dome returns true)
  - `useDensityView?: boolean` (default true; per-note for Time Dome)
  - `CellGeometry`, `TraditionLayoutV7`, `CellMembership` types

- `src/lib/sight/v7/traditions/masadir.ts` — first sample
  TraditionModuleV7, replacing v6's masadir.ts grammar:
  - 4 cell centers at compass East / South / West / North after
    +π/4 rotation offset (Qur'an / sunnah / ijmā' / qiyās)
  - CELL_RADIUS_FRAC = 0.55 (centers at 55% of dome radius)
  - CELL_HIT_RADIUS_FRAC = 0.35 (generous click target)
  - cellMembership preserves MIG-029 §ν.3 fallback to 'quran'
    for invalid / absent `masadir_source` frontmatter
  - `showCalendarRim: false`, `useDensityView: true`

- `src/lib/sight/v7/anchor-v7.ts` — universe-view dispatcher
  - Reuses v6's chrome helpers (`computeDomeLayout`,
    `stratumBandBoundaries`, stratum labels, `readChromePalette`)
    so the dome geometry stays pixel-identical to v6 across the
    ship-gate visual diff
  - Layers (back → front): bg → stratum rings → stratum labels
    → conditional calendar rim → per-cell density blobs → per-cell
    labels → hover/selection rings
  - Calls `tradition.cellRegions(layout)` once per render
  - Iterates rows once via `tradition.cellMembership(row)` to
    count populations per cell
  - Uses `density.ts::computeDensityScale + cellDensity` for the
    universe-wide magnitude → blob-radius mapping
  - Returns `CellHitTestV7[]` for the click pipeline
  - Exports `cellAtPoint(hitTests, px, py)` helper

- `src/lib/sight/v7/SightV7.svelte` — wired to the dispatcher
  - Replaces P1 placeholder with a real canvas mount
  - Fetches rows via `sight_v6_get_layout` IPC (same cache schema
    per Architect §8)
  - DPR-aware canvas sizing
  - Resize observer + locale-subscribe → repaint
  - Mouse hover → highlight ring (repaint on cell change)
  - Mouse click → `cellAtPoint` → console.log (P3 verification only;
    P7 wires the real drill-in)
  - Hardcodes `masadirV7` as the active tradition for P3
    (P9 adds the full dropdown)

- `src/lib/sight/engine.ts` — `SIGHT_V7_ENABLED = true` (was false)
  - P3 produces something testable, so the flag flips so Boss
    can see the v7 dock button in a release build
  - v6 stays reachable via its own dock button (B2 dual-mount)
  - P11 retires the flag entirely (always-on); flipping back to
    false rolls v7 out of the dock if a mid-cascade regression
    surfaces

### Verification

Per Architect §13 P3 verification clause: "masādir under v7 renders
4 wedges with density blobs sized by population count. Visual
screenshot Boss-test."

- Type-check: 3 errors total, all pre-existing (store.ts `fresh`
  lifecycle gap in the LinkLifecycle record + 2 PropertyEditor.svelte
  node-type union errors). Zero v7-related errors.
- Tests: 82/82 vitest pass (no regression from P2).
- Boss test: cf. surfacing message — open Constellation, click the
  v7 dock button (new icon next to v6's), confirm 4 cells render
  at compass NE/SE/SW/NW with the largest blob being the population
  bucket the test universe loads with.

### Open after P3

- **P4** — port the next batch of traditions to TraditionModuleV7
  (sectoral family: aristotelian, pramana, peirce, habermas, longino,
  mencian-sprouts, korean-songnihak, akan-wiredu)
- **P5** — remaining shapes (concentric, grid, ladder,
  horizontal-bands, binary-flow, relational, gradient, radial-tower)
- **P6** — Aristotelian rewrite (vertical-tower primitive) + Time
  Dome as new module
- **P7** — Cell view drill-in (click → expand → stack)
- **P8** — Mini-dome adaptation under v7
- **P9** — Dropdown reorganization (Time group + 24 traditions)
- **P10** — User Manual rewrite + 15-locale backfill
- **P11** — Flag retire + ship gate
- **P12** — 3-agent /migration audit
- **P13** — Close-out (orientation v2.18 + Pending Jobs v1.13 +
  milestone tag + ZIP backup)

### Carried PJs

- **PJ-059** — Sight per-note search/finder (v7 close-out scope)
- **PJ-060** — `index_note` cache-hit short-circuit fix (separate
  mini-MIG; affects MIG-029 frontmatter-driven movement which v7
  inherits)

---

## Boss test: MIG-036 P3 (commit `e1153fe8` + fix-1 `0a0bc58a`)

- **Stage 1**: PASS with two findings:
  - My docs said "right-side dock" — the dock is on the LEFT. My
    error, corrected in mental model.
  - qiyās blob at NORTH overlapped with the CONNECTION stratum
    label on the +y vertical axis.
- **Stage 2** (hover): PASS
- **Stage 3** (click): PASS (informational)

### Fix: MIG-036 P3-fix-1 (commit `0a0bc58a`)

Root cause: P3 inherited v6 masādir's +π/4 wedge-rotation offset
(originally from §θ-fix-1 to push wedge dividers off cardinal axes;
the wedge midpoints landed at E/S/W/N as a side effect). v6 was
fine because individual stars scattered across each wedge with hash
jitter, so stratum labels sat BESIDE the stars; v7 collapsed each
wedge to one density blob exactly on the cardinal axis.

Fix: cells now at diagonals (NE/SE/SW/NW per Concept Paper §4.1.3
original geometry, before v6's rotation). v7 doesn't draw wedge
dividers so the rotation hack is unnecessary. New rebuild
(MIG036-P3-fix1, unsuffixed at `src-tauri/target/release/bundle/nsis/`)
ready for Boss test — but pivot intervened (see below).

---

## Pivot: v7 cascade → v6.3 surgical edits (Eisa decision)

**Eisa's question (verbatim)**: "Why do we have to reinvent the
whole Sight function? We have already reached an advanced level in
v6. Why don't we change the necessary items? Waiting for your
honest answer."

Honest answer surfaced + accepted: v7 was over-engineered for the
three problems it set out to solve. The Form-Aligns-To-Purpose rule
is a *constraint on design*, not a justification for *rewriting
working code*. When Eisa said "Stop patching, and try to restructure
the whole thing if needed" in the MIG-029 cycle, I read "restructure"
as "rewrite the contract" — that was an overcorrection.

Working Agreement #5 (cross-check against proven methods) also
applied here in retrospect: mature visualization systems extend
their existing contract rather than rewriting it for cleanness. I
optimized for "cleanest possible new contract" when the right call
was "smallest disruption to a working system that fixes the problem."

**Eisa decision**: Path A — pivot to v6.3 surgical edits. v7 stays
on disk under `SIGHT_V7_ENABLED` (flag flipped back to false) as
the fallback if v6.3 hits a wall. If v6.3 succeeds, v7 modules
become candidates for retirement in a future MIG.

**Plan approved**: Phases 1 → 2 → 3 of v6.3 (MIG-037).

  - Phase 1: Time Dome added as new tradition (identity remap)
  - Phase 2: Calendar rim opt-in flag; Aristotelian re-frame to
    pure-radial (design call surfaced at Phase 2 boundary before
    code)
  - Phase 3: Density blobs at wedge midpoints. Reuses v7's
    `density.ts` as a pure-function primitive — the only piece of
    v7 work salvaged into v6.3 directly.

---

## Commit: MIG-037 P1 — Time Dome added (this commit)

The proper home for the time-aware view that v6 had implicitly
inherited inside Aristotelian. Per Eisa's direction: "If
Aristotelian is just to display the time, then why are we calling
it this? Instead, the Traditions (including Aristotelian) should
look at the knowledge-cognition lens based on their design. If I
want to display time, I will call this 'the Time Dome'."

### What ships

- `src/lib/sight/engine.ts` — `SIGHT_V7_ENABLED` flipped back to
  `false` (Eisa sub-decision approved). v7 dock button no longer
  appears; v7 module code stays on disk dormant.
- `src/lib/sight/v6/types.ts` — `'time-dome'` added to TraditionId
  union (25th curated tradition).
- `src/lib/sight/v6/traditions/timeDome.ts` — NEW. Identity-remap
  tradition module mirroring current Aristotelian behavior (stratum
  × time, calendar rim around outer edge). Phase 2 adds the
  `showCalendarRim: true` opt-in to differentiate it from
  Aristotelian (which will become pure-radial).
- `src/lib/sight/v6/traditions/index.ts` — import + REGISTRY entry
  + new `'time'` FamilyId + new FAMILIES `time` entry (placed FIRST
  in object iteration order so it surfaces at the top of the chip
  dropdown per Eisa's "Time group at top" direction; relies on JS
  insertion-order stability + the FAMILIES iteration pattern in
  traditionChip.svelte). Export added to bottom block.
- `src/lib/sight/v6/traditionChip.svelte` — `'time-dome'` added to
  CURATED_TRADITION_IDS.
- `src/lib/i18n/{15 locales}.json` — Time Dome i18n entries (name,
  tooltip, scope) + family label, propagated to all 15 locales with
  proper native translations (no transliterations) per the
  full-localization standing order.
- `scripts/add-time-dome-i18n.mjs` — NEW one-shot propagation
  script; kept on disk as a record of the i18n addition (similar
  to how migrations stay readable post-execution).

### Verification

- Type-check: 3 pre-existing errors, zero from Phase 1
  (1467 → 1468 files — the new timeDome.ts).
- i18n: en + ar spot-check confirms entries land cleanly with the
  correct native translations.
- Boss test (separate message after rebuild): open Sight, switch
  the tradition chip dropdown to the new "Time" group at the top,
  pick "Time Dome", confirm it renders identically to Aristotelian
  (same star positions, calendar rim around the outer edge). At
  this phase Time Dome is a visual twin of Aristotelian; Phase 2
  is what makes them architecturally distinct.

### What Phase 1 does NOT ship

- Calendar rim opt-in flag (Phase 2)
- Aristotelian pure-radial reframe (Phase 2)
- Density-mode opt-in for any tradition (Phase 3)
- Dropdown reorder (Time always at top regardless of object
  iteration) — relies on insertion-order convention for now;
  explicit reorder is a polish item if the convention fails

---

## EOD pivots — Sight v6.3 frozen, plugin externalization postponed

After Phase 1 PASS Boss test, three consecutive Eisa pivots:

### Pivot 1 — "Why reinvent Sight?"

Eisa challenged the v6.3 cascade itself: "Why have I created Constellation? What does it represent? Why am I going through all these complex operations? To prove what?"

The challenge anchored on the foundational doc (`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`) and the User Manual's opening: Constellation cultivates *wisdom* through the *living link system*; visualizations are downstream readers, not the core mission. After research-grounded design discussion (Aristotelian epistemology via Stanford EP + IEP), Eisa pivoted to:

### Pivot 2 — "Sight + Map become plugins, not core"

Decision: Sight and Constellation Map both get externalized from core. Plugin API is hybrid (JS plugins + core Rust IPCs). Community / distribution model deferred. Eisa: "No more Sight as a core plug-in. But… we are going to make it an external plug-in."

### Pivot 3 — "Postpone plugin work, audit first"

Within minutes of the plugin-extraction pivot, Eisa pivoted again: "No. I will postpone the Plug-in for now. Instead, I want to audit the Constellation application. To identify where we stand and to plan the way ahead."

All three pivots accepted. Sight v6.3 Phase 2 + Phase 3 frozen. Plugin extraction work not started. MIG-037 P1 (Time Dome) stays on `main` as shipped (Phase 1 only). MIG-036 v7 cascade stays on `main` as dormant fallback. No reverts.

---

## State-of-Application Audit (SO #5)

Snapshot generated 2026-05-19 from three parallel research agents reading: Orientation v2.17 (4568 lines, canonical state-of-architecture), Pending Jobs v1.12 (canonical PJ tracker), session logs of last 7 days, memory entries, git log of last 10 days, codebase grep for BUG-NNN markers.

### Headline state

**Constellation is in a mature, near-ship state.** 18 subsystems verified-shipped and Boss-validated with milestone tags. Codebase clean on `main` (zero uncommitted changes, no divergent feature branches). Two Sight cascades sit at coherent stopping points — both can be paused indefinitely without regression risk. Backlog: 48 open Pending Jobs, dominated by P2/P3 polish (not blockers). **One P1 surprise**: PJ-060 (the `index_note` cache short-circuit) blocks Sight v6.3 P2/P3 AND fixes MIG-029 frontmatter movement that's been failing for weeks. Single most-leveraged fix on the board.

### (a) Verified-shipped and protected

18 subsystems with closure MIGs + protective invariants:

| Subsystem | Closure | Protective invariant |
|---|---|---|
| Typed Links Architecture | MIG-001 | Write-time derivation; confidence levels; decay formula |
| Knowledge Strata (8-level) | MIG-001 | Write-time cache; sky_nodes triggers |
| Maturity Lifecycle | MIG-001 | Write-time derivation; state machine |
| Living Link Architecture P0-P5 | MIG-004 | 7 link types; 4 confidence levels; 8 properties |
| Trails | MIG-001 | Write-time derivation |
| Arabic Morphological Engine L1-L5 | MIG-010 | 5-layer; tashkeel/hamza normalization; L2 protected list (1196); L3 FST mmap; L4 corpus ranking; L5 per-Universe overrides |
| Alias-aware Wikilink Resolution | MIG-004 | 3-tier resolution (name → alias → lowercase) |
| Lexical Bridge (6 modules) | MIG-010 | FTS5 + custom constellation tokenizer; proper-noun protected list |
| 360° Inspector | MIG-010 | Stratification Matrix; 8×8 grid; dedup-by-path; hover-only labels |
| Sight v6.3 Traditions (24 curated + user-definable) | MIG-026 | channel-isolation invariant; per-shape boost; full 15-locale localization |
| Sight v6 Theme Inheritance | MIG-027 | Chrome/semantic color split; theme-aware CSS vars; dark fallback |
| Create-Dialog Standardization | MIG-008 | Modal entry point; 12 file kinds |
| Index Lexical Bridge | MIG-010 | Per-note cache; trigram overlap scoring |
| Index Filter Bridge | MIG-011 | Per-note derivation; category enumeration |
| Index Search Engine | MIG-012 | Full-text ranking; lexeme boosts |
| Note-Stage Taxonomy | MIG-014 | 7 discrete stages; dash-encoded frontmatter |
| Chunked v2 Sentinel Migration | MIG-015 | Status-bar UI; atomicity guarantees |
| File-Over-App Protocol | §10.8 | Read-only invariant; file ACID contract |

Plus today's additions: **MIG-037 P1** (Time Dome added, Phase 1 only), **CLAUDE.md** (Form-Aligns-To-Purpose rule added).

### (b) At-risk / in-flight / uncommitted

- **MIG-036 v7 cascade** — DORMANT. P1+P2+P3+P3-fix-1 commits on `main`; `SIGHT_V7_ENABLED = false`. Code isolated; salvage value: `density.ts` pure functions reused by v6.3 Phase 3 (which is itself frozen). Risk: low. Fallback if v6.3 is ever revisited.
- **MIG-037 v6.3** — Phase 1 SHIPPED, Phases 2+3 FROZEN per today's pivot. Phase 2 design decision (Aristotelian pure-radial reframe) never approved; Phase 3 (density blobs) never started.
- **MIG-029 frontmatter wiring** — BLOCKED. 6 commits attempted (§ν.1–§ν.6 + 3 fixes). Root cause identified: `index_note` cache-hit short-circuit at `search.rs:3004`. Carried as PJ-060.
- **Plugin extraction work** — DISCUSSED and POSTPONED. No code, no Architect doc.
- **Tree state**: clean. No uncommitted changes. No divergent branches.

### (c) Known-broken

6 items, all graceful-degradation or pre-existing deferrals:

| ID | Subsystem | Issue | Severity | Status |
|---|---|---|---|---|
| PJ-012 | Frontend TS | `LinkLifecycle.fresh` missing in union; `npm run check` error at `store.ts:2212` | P2 | Deferred post-CE |
| PJ-028 | Staging | `splitStage('-concept')` empty lifecycle (leading-dash edge) | P2 | Open, graceful |
| PJ-029 | Staging | Concept Paper §6.1 vs `commitStage` multi-dash drift | P2 | Open, policy pending |
| PJ-033 | Localization | NotePane stage badge `<span>` lacks `dir="auto"` (Arabic bidi edge) | P3 | Open, ~1-line fix |
| BUG-001 mem | Note Lifecycle | Phantom duplicate notes — root cause appears resolved, citation kept | Minor | Resolved |
| BUG-015 mem | Svelte | onDestroy corrupts target body — appears mitigated | Minor | Mitigated |

**No production blockers. No runtime crashes. No data-corruption bugs open.**

### (d) Pending but not started

**48 open Pending Jobs**, clustering into 7 categories:

| Category | Count | Priority range |
|---|---|---|
| Mini-MIGs | 4 | P1–P2 |
| Larger MIGs (Links / Map / Rule 8) | 11 | P1–P2 |
| Bug fixes (link panels / search / features) | 6 | P2–P3 |
| Docs (User Manual backfill) | 1 | P2 |
| Cleanup (schema / i18n / dead code) | 6 | P2–P3 |
| Rule 8 audit (persistence / caching) | 4 | P2 |
| MIG-014 §2F follow-ups (staging / localization) | 6 | P2–P3 |
| Carried forward (PJ-059 Sight search; PJ-060 cache short-circuit) | 2 | P1–P3 |

**PJ-060** (P1 blocker) is the critical finding: filed today, not yet in v1.12 tracker. Fixes the cache invalidation that's been blocking MIG-029 + would unblock Sight v6.3 P2/P3 if v6.3 is ever revisited.

Other notable open PJs by area:

- **Links surfaces**: PJ-005 (MIG-007 Links Settings tab), PJ-008 (Outgoing duplication), PJ-009 (Backlinks duplication), PJ-010 (Unlinked Mentions alias bleed) — all isolated single-file or small-MIG work
- **Map backlog (PJ-011)**: 3 bundled issues (D3 perf/memory, tooltip shows canonical filename not human title, search highlight missing)
- **i18n gaps**: PJ-041 (cataloger reasoning hardcoded EN), PJ-042 (`self_reported_confidence` enum bypasses i18n), PJ-043 (taxonomy labels en+ar only, missing 13 locales = ~3,300 translations)
- **In-app Help viewer (PJ-049)**: Help files in 15 locales exist, no UI surface to show them
- **Cleanup batch**: PJ-016 (drop `term_vocab.bridge_concept_id`), PJ-017 (drop `term_embeddings`), PJ-018 (drop `index.semanticSearchEnabled` flag), PJ-019 (drop `searchHub.concept` i18n) — all dead-code/dead-data, intentionally deferred 2-3 sessions for safety, now safe to batch
- **Build reliability (PJ-004)**: NSIS bundling `os error 32` when app is running — affects CI

### (e) Documentation drift

Items in orientation v2.17 §17 (canonical drift register):

| Item | Status |
|---|---|
| `store.ts:3483` TraditionId literal-union duplicate (should import from `types.ts`) | OPEN since v2.13 |
| Concept Paper §4.1.2 pramāṇa "NE/SE/SW/NW" stale notation | RESOLVED in v4.1 (`d3d09d89` 2026-05-14) — remove from §17 in v2.18 |
| Concept Paper §4.1.3 masādir same geometry drift | RESOLVED in v4.1 — remove from §17 in v2.18 |
| §8 Migrations table coverage gap | RESOLVED in v2.16 — already removed |
| PJ-055 plugin-label collision schema warning | OPEN since v2.15 |

**New drift items from today's work** (to add to §17 in v2.18):

- v7 cascade + MIG-037 P1 (added to source but orientation not yet bumped)
- Plugin externalization discussed (no decision committed; document as "considered + postponed")
- State-of-app audit run (this entry)
- Time Dome added as 25th tradition → Concept Paper v4.1 traditions table needs 1-row addition

**User Manual staleness** (PJ-014 + §17): not read in full since pre-MIG-026; full re-read against shipped v6.3 + v7 surfaces would itemize gaps. Deferred to a future User Manual rebuild MIG.

### Confidence notes from the audit

- High confidence: shipping states, today's pivots, drift register
- Moderate gap: actual extent of User Manual staleness (only TOC + opening verified)
- PJ-060 scope is inferred from context; no Architect doc exists yet

---

## End-of-day state, 2026-05-19

**Where Constellation is:** A mature 18-subsystem PKM application with 24-tradition Sight + theme inheritance + full living link architecture + complete Arabic engine, all shipping cleanly. Three Sight-redesign attempts today (v7, v6.3, plugin externalization) all paused at coherent stopping points. Backlog clustered into 7 work streams, dominated by polish, with one P1 fix (PJ-060) that has outsized leverage.

**What's pending Eisa decision:** the way-ahead — which work stream to pick up first, given the audit findings.

---

## Plugin pivot + CECE focus (post-audit)

After the audit, Eisa set a concrete direction (superseding the earlier "postpone plugin"):

1. **Version → 0.1.0** (`26fe4f43`) — JS configs were drifted to 0.3.4; aligned to Cargo.toml + Eisa's "Constellation will be v.0.1."
2. **Sight + Map disabled** (`57cd7638`, MIG-038) — Sight via `SIGHT_V6_ENABLED=false`; Map via `loadSettings` force-off. Code intact for later detachment.
3. **Constellation Wings chartered** (`57cd7638`) — `docs/Constellation Wings/Charter v0.1.md`. The External Plug-in subsystem becomes its own sub-project, DEFERRED until Eisa schedules. Captures the Tauri-single-binary-Rust constraint, the two-layer (isolate-now / load-later) model, the hybrid-API decision, cross-check sources.
4. **Plugin taxonomy clarified** — "Core Plug-in" = a main-LEFT-DOCK feature (like Sky View / CNS / Index), staying in the app. "External Plugin" = detached (Sight, Map → Wings). CECE is the first Core Plug-in.

### CECE Concept Paper v1.0 (this commit)

`docs/Constellation-CECE-Concept-Paper-v1.0.md` — NEW. Eisa: "write/update the CECE concept paper. And based on its core concept we will decide on the proper naming." Grounded in `epistemic-content-EN.md` (the 5-civilization foundation) + an as-built code map (cece/ + classifier/ + sources/).

**Core concept**: CECE classifies each note on two axes — *what kind of knowledge* (content-type: 5 branches) and *where it came from* (source: 11 sources) — making the **epistemic texture** of the universe visible.

**Key accuracy finding (honest)**: CECE ships as a **5-cataloger heuristic ensemble** (User-Authority, Structural, Linguistic, Graph, Semantic). The 6th — **Reasoning (local LLM, Qwen3-4B via llama.cpp)** — is **designed but NOT wired** (abstains on every note). Background auto-scan also not wired (manual-only). User-facing copy must not claim "AI/LLM classification" as shipped.

**Naming decision (pending Eisa)**: "Source Review" names only the source axis, undersells the content-type half. §10 of the paper lays out candidates (The Cataloger / Epistemic Lens / Provenance / Epistemic Content / Ways of Knowing) with my recommendation = **"The Cataloger"** (spans both axes, matches the engine's own cataloger architecture, warm + human). Awaiting Eisa's pick before finalizing the name + building the left-dock feature.

**Next after naming**: build the CECE left-dock Core Plug-in (dock button + full-page view reusing SourceReviewPanel + ClassifierScanProgressStrip); right-sidebar Source Review tab stays as-is until the dock view is done.

