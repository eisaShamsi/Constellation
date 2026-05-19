# MIG-036 — Sight v7 — Form-Aligns-To-Purpose Redesign

**Status**: Architect doc (Phase 1 of 4 /migration discipline).
**Date**: 2026-05-19
**Driver**: Form-Aligns-To-Purpose rule (CLAUDE.md 2026-05-19, commit `30f1d6a7`).
**Predecessor**: Sight v6.3 (commits `28c939af` → `feffbd4e`); v6 stays operational during v7 development via `SIGHT_V7_ENABLED` flag (B2 dual-mount pattern from MIG-024/025).

---

## §1 — Goal

Rebuild Sight's rendering pipeline so that every visual element a tradition exposes carries cognitive meaning. Eliminate the within-cell hash-jitter that v6 inherited (where notes inside masādir's sunnah quadrant, pramāṇa's anumāna quadrant, Shāṭibī's grid cells, etc. were positioned by month-fraction or hash even though those positions carry no analytical meaning under the tradition's own grammar).

The redesign is driven by Eisa's articulation of the rule and his specific design call: **Hybrid X+Y, zoom-driven**.

## §2 — The redesign in one sentence

Each cell of a categorical tradition (wedge / ring / step / grid-cell / band / cluster) renders as a **density element** at low zoom (one visual unit, magnitude encoding population) and as a **stack of individual notes** when the user zooms / drills into that specific cell. The angular / within-cell position is *never used* at any zoom level. The cell IS the unit at low zoom; the stack IS the unit at high zoom.

## §3 — What changes vs. Sight v6

| Sight v6 (current) | Sight v7 (this MIG) |
|---|---|
| Hash-jittered notes inside each categorical cell | Density-blob per cell at low zoom |
| Click a star = open its note | Click a cell = drill in; inside the stack, click a note = open it |
| Single zoom level | Two-level: universe (cells) ↔ cell (notes) |
| Tradition dropdown contains 24 traditions; "Aristotelian" is the default + uses stratum × time + calendar rim | Dropdown contains 24 traditions + 1 explicit "Time Dome" entry. Aristotelian becomes a pure-radial maturity tradition (stratum bands only); Time Dome is the explicit time view (stratum × time + calendar rim) |
| Calendar rim drawn under every tradition | Calendar rim drawn ONLY when the active entry is Time Dome |
| Renderer "primitive" is always the dome (radial × angular) | Renderer primitive is the tradition's natural geometry — dome for stratum × time, segmented dome for sectoral, vertical-tower for radial-only, etc. |
| MIG-029's cache invalidation bugs are user-visible | The same data path is correct under v7 because notes land in their categorical cell (with population density) rather than at within-cell positions that depend on stale frontmatter propagation |

## §4 — Invariants Sight v7 must preserve

I1. **Read-only with respect to source-of-truth.** Doesn't write `note_meta`, `note_links`, or `.md` files. (Carried from v6.)

I2. **Locale-reactive and theme-reactive.** Every label flows through `$t()`; chrome reads from `--sight-*` CSS variables. (Carried.)

I3. **Form-Aligns-To-Purpose.** Every visual primitive expresses its tradition's specific epistemic grammar and nothing else. Within-category positions don't encode anything when the tradition's grammar doesn't say they should.

I4. **Per-note interaction preserved.** The user can still click a specific note to open it. This happens in the drill-in (high-zoom) view, not at the universe-level density view.

I5. **Performance budget unchanged.** Initial render ≤16ms on a 7,636-note universe per Concept Paper §11.3.

I6. **MIG-026 contract preserved.** All 24 traditions still ship; user-definable plugin layer still works; full 15-locale localization still works.

I7. **No `invoke()` on keystroke hot path.** (CLAUDE.md Rule 1.)

I8. **Write-time derivation preserved.** Cache stays as the source of layout data; the rendering pipeline reads from the cache, doesn't recompute on read.

I9. **MIG-029 per-note frontmatter wiring preserved.** Frontmatter `masadir_source` / `pramana_kind` / etc. still drive which cell each note lands in. The cache invalidation bug from MIG-029 fix-1/2/3 stops being user-visible under v7 because:
- The user sees the cell's POPULATION (a density blob) not individual note positions.
- A correctly-classified-but-stale-cached note ends up in the WRONG cell (still a bug), but in v7 this manifests as "the population count in two cells is slightly off" rather than "my note is invisibly missing."
- Drill-in to a cell shows the stack via a *live* SELECT that includes frontmatter — so the drill-in view shows truth even if the universe-level density is stale.

I10. **Aristotelian's epistemic identity restored.** Aristotelian = maturity gradient (stratum bands), not "the default coordinate system." Time is a separate explicit view.

## §5 — Per-shape primitive redesign

Each of the 9 shape renderers gets a v7 primitive that aligns its form to its specific cognitive grammar. The dome chrome (stratum rings as a visual reference, plus the tradition-specific dividers) stays for shapes whose grammar genuinely uses radial+angular; collapses to a simpler primitive for shapes whose grammar uses fewer dimensions.

### 5.1 Sectoral (masādir, pramāṇa, Habermas, Peirce, Mencian-sprouts, Akan-Wiredu, Longino, Husserl in 4-zone variant)

- Each wedge renders as ONE density element centered in the wedge.
- Density encoding: blob diameter (or fill intensity) scaled to log(population + 1).
- Stratum bands remain visible behind the wedges as a context layer (a note in the sunnah wedge that's at Foundation stratum lands in the inner part of the sunnah wedge, but the within-wedge angular position is meaningless and not rendered).
- Drill-in: click a wedge → wedge expands to fill the dome → stack of notes in that wedge appears, sorted by stratum (inside-out). Now the user sees individual notes.

### 5.2 Concentric rings (Ibn Rushd burhān, PaRDeS, Maldonado-Torres, Husserl in concentric variant)

- Each ring renders as a single density band whose thickness or saturation encodes its population.
- No within-ring angular distinction.
- Drill-in: click a ring → ring expands → stack of notes in that ring, sorted by some meaningful sub-axis (likely stratum, falling back to creation date if stratum is identical).

### 5.3 Grid (Shāṭibī maqāṣid 3×5, Korean Sŏngnihak 2×2)

- Each grid cell renders as a single density unit (intensity-encoded square).
- Drill-in: click a cell → cell expands → stack of notes in that cell.

### 5.4 Ladder/spiral (Maimonidean prophecy 11 steps, Talmudic 13 middot, Dewey 5 stages)

- Each step renders as a single density bar along the ladder.
- The ladder geometry itself carries the sequential meaning; within-step position is meaningless.
- Drill-in: click a step → expand to stack.

### 5.5 Horizontal bands (Mohist sān biǎo)

- Each band renders as a single density bar across the dome's diameter at the band's vertical position.
- No within-band horizontal jitter.
- Drill-in: click a band → expand to stack.

### 5.6 Binary flow (Wang Yangming, Dussel transmodernity, Ibn Khaldūn ʿumrān)

- Each binary cell renders as a single density blob; the flow direction arrows are unchanged (they carry meaning about the binary's direction).
- Drill-in: click a cell → expand to stack.

### 5.7 Relational hub-and-spoke (Mignolo pluriversal, Ibuanyidanda)

- Hub renders as a labeled central element with its own density.
- Each spoke cluster renders as a single density blob at the cluster's natural position around the hub.
- The within-cluster jitter currently in v6 is **borderline** — cluster center is the cognitive position, and intra-cluster scatter is what made the cluster READ as a cluster. Two options:
  - **5.7.A** — collapse intra-cluster to density (one blob per cluster). Consistent with the rule.
  - **5.7.B** — keep intra-cluster jitter but document it as "visual cluster expression," not analytical position.
- Default: **5.7.A** per the rule. Drill-in shows the stack.

### 5.8 Gradient fog (Polanyi tacit ↔ explicit)

- No change. The gradient is opacity-encoded; positions stay at default-stratum positions; the gradient *is* the answer.
- Drill-in: not applicable (the answer is the gradient, not per-note).

### 5.9 Stratum × time (Time Dome — NEW)

- Both axes meaningful (radial = stratum, angular = creation month). Calendar rim visible.
- Notes render as individual dots (no density collapse) because at the universe level, the user is reading position-of-each-note, not population.
- This is the form-aligned use of the dome geometry.
- Click a note = open it (no drill-in level — already at the per-note view).

### 5.10 Aristotelian (REDESIGNED — pure radial)

- Vertical-tower primitive (NOT a dome): five horizontal slabs stacked top-to-bottom, one per stratum band (Foundation at top → Edge of Knowing at bottom, or reverse — TBD).
- Each slab renders as a single density bar whose width or intensity encodes its population.
- No angular axis at all. The geometric primitive matches the cognitive content.
- Drill-in: click a slab → expand to stack of notes in that stratum.

## §6 — Time Dome (new entry)

`src/lib/sight/v7/traditions/time-dome.ts` — new module. Listed in the chip dropdown as a top-level group separate from the 24 traditions:

```
Dropdown structure:
   Time
   └─ Time Dome

   Tradition (family-organized, 24 entries)
   ├─ Western classical → Aristotelian
   ├─ Indian Nyāya → pramāṇa
   ├─ Sunni Islamic uṣūl → masādir
   ├─ Arabic / Islamic beyond uṣūl → Ibn Rushd burhān, Shāṭibī maqāṣid, Ibn Khaldūn ʿumrān
   ├─ Modern Western → Polanyi, Peirce, Habermas, Dewey, Husserl, Longino
   ├─ Jewish (Abrahamic) → PaRDeS, Maimonidean prophecy, Talmudic 13 middot
   ├─ East Asian Confucian → Mencian sprouts, Wang Yangming, Korean Sŏngnihak
   ├─ Chinese pragmatist → Mohist sān biǎo
   ├─ African philosophical → Akan Wiredu, Ibuanyidanda
   └─ Latin American decolonial → Mignolo pluriversal, Dussel transmodernity, Maldonado-Torres
```

Default on first install: Time Dome (matches the v6.3 default behavior where the first thing a user sees is the stratum × time grammar — except now it's honestly named).

## §7 — Drill-down interaction

Two interaction modes:

### 7.1 Universe view (default)
- The dome renders the active tradition's per-cell density.
- Hover over a cell: tooltip shows cell name + population count.
- Click a cell: transitions to Cell view for that cell.
- Click empty space outside any cell: no-op (or "reset zoom" if zoomed).

### 7.2 Cell view (drilled-in)
- The clicked cell expands to fill the canvas.
- The cell's notes render as a stack — one row per note, with the note title visible.
- Sort order: by stratum (most foundational first), with creation date as tiebreaker.
- Click a note in the stack: opens it in NotePane (same behavior as v6's click-a-star).
- Click outside the expanded cell, or press Esc, or press the back button: returns to Universe view.

### 7.3 Mini-domes under v7
The 4 mini-domes (Confidence / Stage / Acts / Provenance) per Concept Paper §11 invariant 6 stay tradition-agnostic. Under v7's hybrid model, they show the universe-level density too (population in each Confidence level / Stage / Acts decile / Provenance sector). Drill-in works the same way for them.

## §8 — v6 → v7 migration path (B2 dual-mount)

Same pattern as MIG-025's v5 → v6:

1. v7 lives in `src/lib/sight/v7/` and `src-tauri/src/sight_v7.rs`, parallel to v6.
2. New flag: `SIGHT_V7_ENABLED = false` initially (engine.ts).
3. v7 develops to feature-parity. Boss-test cycle on the v7 branch.
4. Once Boss approves: `SIGHT_V7_ENABLED = true` ships. v6 stays on disk but unreachable.
5. v6 retirement MIG follows after a stabilization window (same pattern as MIG-028 retired v5).

Schema-wise: v7's layout cache can either reuse `sight_v6_layout` (since the underlying data is the same) or get its own `sight_v7_layout` table. Recommendation: **reuse** — the per-note frontmatter fields landed in MIG-029 are all that v7 needs; no new columns required. Cache invalidation is no longer a user-visible bug under v7 (per Invariant I9).

## §9 — Phase decomposition

Build phases follow `/migration` discipline. ASCII phase markers per the 2026-05-19 "ν vs v" notation correction.

| Phase | Scope | Verification | Time |
|---|---|---|---|
| **P1** Scaffolding | `src/lib/sight/v7/` directory + Rust `sight_v7.rs` + `SIGHT_V7_ENABLED` flag + minimal `SightV7.svelte` placeholder mount | Builds clean (cargo check + vite build); placeholder loads behind the flag | 0.5d |
| **P2** Rendering primitives | New `density.ts` (single-blob per cell with magnitude encoding) + `stack.ts` (notes-list view per cell) — both pure functions, no Svelte | Unit tests on synthetic data | 1d |
| **P3** Universe view dispatch | `anchor-v7.ts` — replaces v6's `anchor.ts` `renderAnchorDome`. Reads the tradition module's `cellRegions()` callback (NEW) + populates each cell with density via `density.ts` | Renders correctly for 1 sample tradition (masādir) | 1d |
| **P4** Per-shape redesign — sectoral, concentric, grid | 14 tradition modules' `cellRegions()` implementations (sectoral: 6, concentric: 4, grid: 2, others: 2) | Visual smoke test per family | 2d |
| **P5** Per-shape redesign — ladder, horizontal bands, binary flow, relational, gradient | 10 more tradition modules | Visual smoke test | 1.5d |
| **P6** Aristotelian rewrite + Time Dome new module | `traditions/aristotelian-v7.ts` (vertical-tower density) + `traditions/time-dome.ts` (stratum × time + calendar rim only here) | Smoke test both modes | 1d |
| **P7** Cell view (drill-in interaction) | Click handler in `SightV7.svelte` + stack-view component + back/Esc/outside-click exit | Drill-in flow works end-to-end | 1d |
| **P8** Mini-dome adaptation | 4 mini-domes (confidence/stage/acts/provenance) gain density rendering + drill-in | Smoke test | 1d |
| **P9** Dropdown reorganization | Time group at top + 24-tradition family list below. i18n keys for "Time" + "Time Dome" + "Time Dome (tooltip)" | UI smoke + i18n agent backfill (15 locales) | 1d |
| **P10** User Manual chapter rewrite | `docs/User Manual.md` §8 + 14 locale mirrors. Document the hybrid X+Y model, the drill-in interaction, the Time-vs-Tradition split | Boss reads + accepts | 0.5d |
| **P11** Flag flip + ship gate | `SIGHT_V7_ENABLED = true`, `SIGHT_V6_ENABLED = false`. Boss-test cycle on the full v7 build. | Boss PASS | 0.5d |
| **P12** Audit | 3-agent /migration audit (invariants + drift + migration-path) | 0 blockers | 0.5d |
| **P13** Close-out | Orientation v2.18, Pending Jobs v1.13, milestone tag `milestone/sight-v7-form-aligns-to-purpose`, ZIP backup | Tag + backup land | 0.5d |

**Total**: ~12 days of focused work. ~15-20 commits.

## §10 — Sight v6 retirement (MIG-037)

After v7 ships + stabilizes, MIG-037 retires v6's codebase via the established pattern (delete `src/lib/sight/v6/`, `sight_v6.rs`, the IPC handlers, the cache schema if v7 didn't reuse it, and the `SIGHT_V6_ENABLED` flag). Not part of this MIG; queued for after Boss validates v7.

## §11 — Risks

- **R1 — Density encoding is illegible at extreme scales.** A universe with 7K notes might have one cell with 4000 and another with 50; the visual range needs to convey that asymmetry without saturating. **Mitigation**: log-scale + minimum-floor for the density encoding.
- **R2 — Drill-in interaction surface complexity.** Two zoom levels = more state, more keyboard / gesture handling, more failure modes. **Mitigation**: P7 phase scoped explicitly to interaction polish.
- **R3 — Aristotelian's vertical-tower primitive looks visually disconnected from the rest of the traditions.** **Mitigation**: shared chrome (stratum bands as horizontal reference lines, same color palette). The tower is a primitive within Sight's visual language, not a foreign artifact.
- **R4 — Per-note frontmatter wiring's cache invalidation issue (MIG-029 fix-1/2/3) might still misclassify notes** — but Invariant I9 means the user-visible symptom changes from "note missing" to "wrong cell count by 1." Acceptable for v7 ship; track as a follow-up via the existing PJ-060 (`index_note` short-circuit).
- **R5 — Performance regression**. The density computation is O(N) per cell × cells (small constant), plus stack-view query on drill-in. Total O(N) per render. Same as v6. **Mitigation**: vitest perf test (Plan §14.2 pattern) verifies ≤16ms on 7,636 notes.

## §12 — What's NOT in MIG-036

- Sight per-note search/finder (PJ-059) — separate UX; could land as P7 polish but doesn't have to.
- `index_note` short-circuit fix (PJ-060) — affects the data path globally, not just Sight. Separate MIG.
- Wasm/QuickJS sandbox for plugin layer (MIG-033) — orthogonal security uplift.
- v4.1 per-tradition internal-structure polish (MIG-034) — superseded by v7's per-shape redesign.
- Federation cUniverse tradition behavior (MIG-035) — orthogonal.

## §13 — Verification clauses (gate per phase)

Each phase has a single observable verification clause:

- P1: `npm run build` + `cd src-tauri && cargo check` both green. `SightV7.svelte` placeholder mounts behind the flag.
- P2: vitest unit tests for `density.ts` + `stack.ts` pure functions pass.
- P3: masādir under v7 renders 4 wedges with density blobs sized by population count. Visual screenshot Boss-test.
- P4–P5: each tradition family renders correctly per its primitive. Visual smoke test.
- P6: Aristotelian vertical-tower + Time Dome stratum × time both work. Calendar rim only on Time Dome.
- P7: click a cell → expand → see stack of notes → click note → opens. Esc returns to universe view.
- P8: 4 mini-domes render under v7 with density + drill-in.
- P9: dropdown has Time group at top + 24 traditions family-organized below. 15-locale labels present.
- P10: User Manual chapter present in 15 locales.
- P11: `SIGHT_V7_ENABLED = true` + Boss-test PASS across all 25 entries (Time + 24 traditions) + drill-in on a representative sample.
- P12: 3-agent audit returns 0 P0/P1.
- P13: milestone tag + ZIP backup landed; orientation v2.18 + Pending Jobs v1.13 reflect the close.

---

**End of Architect doc.**

Plan-Approval = Build-Approval: Eisa's "let's adapt it" 2026-05-19 is the approval to cascade through P1 → P13. I cascade now and stop only at the verification clauses that require Boss input (P3 visual screenshot, P11 ship-gate Boss-test).
