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
