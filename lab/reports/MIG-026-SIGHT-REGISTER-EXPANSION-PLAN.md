# MIG-026 — Sight Register Expansion + User-Definable — Plan

**Status**: Plan phase (Migration Rule step 2 of 4). Awaiting Eisa approval → Build cascade.

**Authored**: 2026-05-17. Step-by-step build sequence with per-phase verification clauses. After Eisa approval, Plan-Approval-Equals-Build-Approval cascade kicks in: I build each phase, commit, push, rebuild .exe if user-testable, surface tutorial-style Boss test per Testing Instructions Rule, wait for PASS, move to next phase.

**Antecedent doc**: `lab/reports/MIG-026-SIGHT-REGISTER-EXPANSION-ARCHITECT.md` (522 lines, §5.5/§8 locked 2026-05-17).

---

## §1 — Pre-cascade contract

### Decisions locked (per Architect §9)

| Architectural choice | Locked value |
|---|---|
| Chip UI redesign | A3+A6 (family categorization + 4 favorites + dropdown hybrid) |
| Multi-shape architecture | B1 (discriminated union on TraditionModule) |
| Ring-band renderer | C1 (RingSpec interface) |
| Ladder renderer | D3 (spiral N-step — new path-math renderer) |
| Relational renderer | E3 (hub-and-spoke fixed layout) |
| User-defined storage | F2 (`<Universe>/.constellation/registers/`) — TBD if "registers" or "traditions" subdirectory; default per K1 rename → `traditions/` |
| Declarative schema | G1 (JSON Schema, fixed shape vocabulary) |
| Plugin loader | H1 (dynamic import in v6.3, Obsidian-trust + default-off + manual enable) |
| Translation cascade | I1 (English first commit, 14-locale follow-up) + I3 (user-defined ships English-only) |
| Disclosure layer | J3 (ⓘ opens manifest in editor) + J5 (scope strip under expanded chip) |
| Terminology reframe | K1 (full rename throughout: code + Concept Paper + i18n + help docs) |
| Persistence schema | L1 (open string for activeTradition) |

### 12 invariants (must not break across cascade)

Per Architect §2 — re-stated here for verification reference:
1. §11.3 cross-filter ≤16ms (perf gate)
2. §11.5 pip foveation threshold
3. §11.6 anchor-only tradition remap (mini-domes register-agnostic; starsDefault for minis)
4. §11.7 manifest + citation per tradition
5. §11.9 no persistent toggle bars
6. §11.10 tour re-availability
7. File over app
8. Local first / no telemetry
9. Every keystroke instant
10. Multilingual by design
11. Reversibility
12. No silent file modification (no `tradition_kind:` frontmatter without user action)

Plus 6 MIG-026 new invariants (religious-lineage rule, geometric-shape isolation, chip-list parity for curated+user-defined, CARE disclosure, plugin sandbox boundary, schema-version compat).

### Cascade discipline (per Standing Order #1 + Testing Instructions Rule)

- Each phase = one commit (or small sub-phase commit group). Tagged with `MIG-026 §<phase>`.
- After each user-testable phase: rebuild .exe (NSIS), surface Boss-test tutorial, wait for PASS.
- Pure-refactor phases (Phase 0 rename, Phase α architecture foundation): commit + push without Boss-test, but spot-check.
- Session log updated after each phase per SO #1.
- Orientation v-bump on (a) MIG-026 mid-cascade if a top-principal rule lands, (b) MIG-026 ship completion.
- If a Boss test FAILs, that phase opens a `§<phase>-fix-N` sub-step (same shape as §C.1-fix-1, §C.3-fix-1).
- Phase-completion summary at the end of each Boss-test cycle.

### Where MIG-026 lands its commits (per Working Agreement #2)

All commits on `main` of `E:\مشاريع كلاود\Constellation`. Branched work not used for this MIG (matches MIG-025 pattern).

### Phase summary

21 build-and-test cycles across 10 main phases (Phase 0 + α through μ) plus 11 sub-phases. Boss-test-required cycles ≈ 17; pure-refactor cycles ≈ 4. Pace: ~3–4 weeks of focused work.

---

## §2 — Phase 0 — K1 full rename ("register" → "tradition")

**Why first**: All subsequent phases build under the new namespace. Renaming after the build cascade would mean renaming hundreds of new files.

**Goal**: Rename all internal naming from "register"/"Register" to "tradition"/"Tradition" throughout the codebase + spec docs + i18n + help docs. Zero user-visible behavior change. Existing 5 traditions still work; chip still says "Aristotelian ●"; settings persist via migration.

**Files touched** (~40 files):

| File | Rename |
|---|---|
| `src/lib/sight/v6/types.ts` | `RegisterId` → `TraditionId`; `RegisterModule` → `TraditionModule`; `RegisterLayout` → `TraditionLayout`. `SectorSpec` stays (it's about geometry, not the register concept). |
| `src/lib/sight/v6/registers/index.ts` | File rename → `src/lib/sight/v6/traditions/index.ts`. Folder rename `registers/` → `traditions/`. Exports renamed. |
| `src/lib/sight/v6/registers/aristotelian.ts` | File rename → `traditions/aristotelian.ts`. |
| `src/lib/sight/v6/registers/pramana.ts` | File rename → `traditions/pramana.ts`. |
| `src/lib/sight/v6/registers/masadir.ts` | File rename → `traditions/masadir.ts`. |
| `src/lib/sight/v6/registerChip.svelte` | File rename → `traditionChip.svelte`. Component class names + CSS class names updated: `.register-chip-root` → `.tradition-chip-root` etc. |
| `src/lib/sight/v6/anchor.ts` | All "register" references in comments + variable names → "tradition". `register?: TraditionModule` param renamed. `drawSectorDividers` keeps name (about geometry). |
| `src/lib/sight/v6/SightV6.svelte` | All imports + references renamed. `registerExtensionChips` $derived → `traditionExtensionChips`. `activeRegister` reads → `activeTradition`. `getRegisterById` → `getTraditionById`. |
| `src/lib/libraries/store.ts` | `activeRegister?:` field → `activeTradition?:`. Migration block ADDED: if `parsed.sight.activeRegister` exists, copy to `activeTradition`, delete `activeRegister`. Existing dignaga/ishraqi migration blocks updated to operate on `activeTradition`. |
| `docs/Constellation-Sight-Concept-Paper-v4.0.md` | All §4 references renamed: "register" → "tradition" throughout §4.1.x and §4.2.x. §2.5 "Register Chip" → "Tradition Chip". §7 mini-dome stipulation references updated. §11 invariants language updated. |
| `lab/reports/MIG-025-SIGHT-V6-PLAN.md` | "register" references in §C/§D updated. §D.1 (Dignāga) + §D.2 (Ishrāqī) SUPERSEDED-notes updated. |
| 15 × `src/lib/i18n/*.json` | If any "register" i18n keys exist, renamed. (Currently none — chip labels are English brand convention.) |
| `docs/help.{lang}/Constellation Sight/Constellation Sight.md` × 15 | "register" references updated (if any) → "tradition". |
| `docs/Constellation — Universal Orientation.md` | "register" references → "tradition". |
| `docs/Constellation Orientation & Onboarding v2.11.md` | Already references "MIG-026" + "register" mix; v2.12 captures the rename. |

**Migration block in store.ts** (NEW, runs in `applyParsedSettings`):
```typescript
// MIG-026 Phase 0 — K1 rename migration (register → tradition).
// Idempotent: if user persisted activeRegister, copy to activeTradition
// and delete activeRegister. Subsequent loads have no activeRegister so
// the branch is silent. saveSettings persists the rewrite.
if ('activeRegister' in sightSnapshotForRename) {
  appSettings.update((s) => {
    const nextSight = { ...s.sight } as Record<string, unknown>;
    if (!('activeTradition' in sightSnapshotForRename)) {
      nextSight.activeTradition = sightSnapshotForRename.activeRegister;
    }
    delete nextSight.activeRegister;
    return { ...s, sight: nextSight as typeof s.sight };
  });
  saveSettings();
}
```

**Behavior shipped**: zero user-visible change. Subtitle stays "v6.2 — Registers (Phase 3)" until Phase α changes it to "v6.3 — Traditions (Phase 1)". Chip layout unchanged. All 5 currently-shipping traditions still work. Settings persist via migration.

**Verification clause**:
- **Code**: `grep -r 'RegisterId\|RegisterModule\|registerChip\|activeRegister' src/` returns no matches; `npm run check` passes; `npm run tauri build` passes
- **User (light Boss test)**: open Sight on the rebuilt .exe; chip still says "Aristotelian ●"; switch traditions; confirm settings persist after quit+relaunch

**Risk**: massive rename touching ~40 files. Mitigation: TypeScript-aware editing tools (sed via Edit) + test build before pushing. If type-check fails, no commit lands.

**Boss-test required**: yes — light verification that the rename didn't break anything visible.

**Time estimate**: 1 day.

**Commit message anchor**: `MIG-026 §0 — K1 full rename (register → tradition)`

---

## §3 — Phase α — Architecture foundation

**Goal**: Extend `TraditionModule` interface to support multi-shape registers. Add shape discriminator (`shape: 'sectoral' | 'rings' | 'grid' | 'ladder' | 'relational' | 'cyclic-flow' | 'binary-flow' | 'gradient' | 'horizontal-bands'`) + shape-specific spec interfaces. Add private renderer helpers in anchor.ts for each new shape (stubs OK; per-shape rendering filled in by later phases).

**Files touched** (5 files):
- `src/lib/sight/v6/types.ts` — add `TraditionShape` union; add `RingSpec`, `LadderSpec`, `RelationalSpec`, `CyclicFlowSpec`, `BinaryFlowSpec`, `GradientSpec`, `HorizontalBandsSpec` interfaces; extend `TraditionModule` with `shape: TraditionShape` discriminator + per-shape optional fields
- `src/lib/sight/v6/anchor.ts` — add private helpers `drawRingBoundaries`, `drawLadderSteps`, `drawRelationalGraph`, `drawCyclicFlow`, `drawBinaryFlow`, `drawGradientFog`, `drawHorizontalBands`; dispatch from `renderAnchorDome` step 2.5 via switch on `register.shape`
- `src/lib/sight/v6/traditions/aristotelian.ts` — declare `shape: 'sectoral'`
- `src/lib/sight/v6/traditions/pramana.ts` — declare `shape: 'sectoral'`
- `src/lib/sight/v6/traditions/masadir.ts` — declare `shape: 'sectoral'`

**Behavior shipped**: zero user-visible change. Architecture is in place to support all 7 shapes; only sectoral is in active use. New shape renderers exist as no-op stubs.

**Verification clause**:
- **Code**: `npm run check` passes; `npm run tauri build` passes; all 5 currently-shipping traditions render identically to Phase 0 ship
- **User**: light spot-check — open Sight, switch all 5 traditions, confirm nothing changed visually

**Risk**: low. Architectural additions are non-breaking.

**Boss-test required**: no (pure refactor; light spot-check sufficient).

**Time estimate**: 1 day.

**Subtitle bump**: `v6.2 — Registers (Phase 3)` → `v6.3 — Traditions (Phase 1)` lands here.

**Commit message anchor**: `MIG-026 §α — Architecture foundation (multi-shape TraditionModule)`

---

## §4 — Phase β — A3+A6 chip UI redesign

**Goal**: Redesign chip UI to handle 24+ traditions via family categorization (A3) + 4 favorites anchored inline + dropdown for rest (A6). Persistence of favorites in `appSettings.sight.favoriteTraditions: TraditionId[]`.

**Files touched** (4 files):
- `src/lib/sight/v6/traditionChip.svelte` — full rewrite: chip displays 4 favorites inline + dropdown trigger; dropdown opens panel with family-collapsible accordion; each tradition entry shows name + scope-strip + pin/unpin toggle; clicking a tradition switches active + collapses dropdown
- `src/lib/libraries/store.ts` — add `favoriteTraditions?: TraditionId[]` field; default `['aristotelian', 'pramana', 'masadir', 'polanyi']`
- `src/lib/sight/v6/SightV6.svelte` — chip mount unchanged; pass `favoriteTraditions` from store
- `src/lib/sight/v6/traditions/index.ts` — add `FAMILIES: Record<FamilyId, TraditionId[]>` const declaring which family each tradition belongs to

**Behavior shipped**: chip now scales to 24+ traditions. User sees 4 favorite chips inline + a dropdown trigger; clicking the dropdown opens a panel grouped by family (Aristotelian / Indian Nyāya / Sunni Islamic uṣūl / Modern Western / Chinese pragmatist / East Asian Confucian / African philosophical / Latin American decolonial / Jewish Abrahamic). Each tradition can be pinned/unpinned as favorite. Currently only 5 traditions shipped so the dropdown shows just 1 non-favorite (Mohist).

**Verification clause** (Boss test):
- **Stage 1**: open Sight, see 4 favorite chips inline + dropdown trigger
- **Stage 2**: click dropdown, see Mohist sān biǎo in the "Chinese pragmatist" family group
- **Stage 3**: pin Mohist; chip becomes the 5th inline favorite; another favorite gets bumped to dropdown
- **Stage 4**: hover any tradition in dropdown → scope-strip visible
- **Stage 5**: persistence — pin/unpin survives quit+relaunch

**Risk**: medium. Significant UI rewrite; layout edge cases (small window, RTL); favorites persistence.

**Boss-test required**: yes.

**Time estimate**: 2 days.

**Commit message anchor**: `MIG-026 §β — A3+A6 chip UI (family categorization + favorites)`

---

## §5 — Phase γ — Polanyi + Mohist tradition modules

**Goal**: Build the 2 chip-placeholder traditions inherited from MIG-025 (originally planned for §C.5 Polanyi + §D.3 Mohist). Polanyi = gradient shape; Mohist = horizontal-bands shape.

**Files touched** (3 files):
- `src/lib/sight/v6/traditions/polanyi.ts` — NEW. shape: 'gradient'. `remapStarPosition` = identity (Polanyi doesn't redistribute; it changes opacity per-star). `gradientSpec(layout): GradientSpec` returns center=tacit (low opacity) + edge=explicit (high opacity).
- `src/lib/sight/v6/traditions/mohist-san-biao.ts` — NEW. shape: 'horizontal-bands'. `remapStarPosition` redistributes stars to 3 horizontal bands (top=本/root, middle=原/origin, bottom=用/use). `horizontalBandsSpec(layout)` returns 3 bands with labels.
- `src/lib/sight/v6/anchor.ts` — fill in `drawGradientFog` (Polanyi: opacity gradient over canvas) + `drawHorizontalBands` (Mohist: 3 horizontal lines + labels)

**Plus**: register both in `traditions/index.ts` REGISTRY.

**Behavior shipped**: chip now has 5 + 2 = 7 working traditions. Switching to Polanyi shows fog gradient (dense center, clear edges). Switching to Mohist shows 3 horizontal zones with stars distributed by hash (since no `mohist_zone` frontmatter yet).

**Verification clause** (Boss test):
- **Stage 1**: switch to Polanyi → fog gradient visible (dense at center, clear at edges)
- **Stage 2**: switch to Mohist → 3 horizontal bands visible with 本/原/用 labels
- **Stage 3**: mini-domes UNCHANGED for both (§11.6 isolation)
- **Stage 4**: switch back to Aristotelian → both chrome additions cleanly disappear

**Risk**: medium. Two new shape renderers in one phase.

**Boss-test required**: yes.

**Time estimate**: 1.5 days.

**Commit message anchor**: `MIG-026 §γ — Polanyi (gradient) + Mohist sān biǎo (horizontal-bands) modules`

---

## §6 — Phase δ — Modern Western family (5 traditions)

Split into 2 sub-phases for manageable Boss tests.

### §6.1 — Phase δ.1 — Peirce + Habermas (sectoral, 3-cell each)

**Files**: 2 NEW modules + REGISTRY update.

**Behavior**: switching to Peirce → 3 sectors (Firstness / Secondness / Thirdness). Switching to Habermas → 3 sectors (technical / practical / emancipatory). Both default all stars to first sector since no frontmatter opt-in yet.

**Verification clause** (Boss test): 2-stage cycle — each tradition switch, confirm 3-sector layout + labels + mini-isolation.

**Time**: 0.5 day.

### §6.2 — Phase δ.2 — Dewey + Husserl + Longino

**Files**: 3 NEW modules + REGISTRY update.

**Behavior**:
- Dewey → cyclic-flow shape: 5-segment ring (indeterminate / problem / hypothesis / reasoning / testing) with arrow flow
- Husserl → mixed shape: central disc (formal ontology) + 3 petals (material nature / animal nature / spirit)
- Longino → sectoral 4-cell (venues / uptake / public standards / tempered equality)

**Verification clause** (Boss test): 3-stage cycle.

**Time**: 1 day.

**Commit message anchor**: `MIG-026 §δ.1` and `MIG-026 §δ.2`

---

## §7 — Phase ε — Arabic Islamic family (3 traditions)

3 sub-phases — each registers a substantially different shape, so worth separate Boss tests.

### §7.1 — Phase ε.1 — Ibn Rushd burhān ladder (4 concentric rings)

**Files**: NEW `traditions/ibn-rushd-burhan.ts` + REGISTRY update + anchor.ts `drawRingBoundaries` implementation.

**Behavior**: switching to Ibn Rushd burhān → 4 concentric rings with labels (burhān innermost, jadal next, khaṭāba next, shiʿr outermost).

**Verification clause** (Boss test): switch → 4 ring boundaries + 4 ring-label arcs visible; stars distributed across rings by `burhan_kind` frontmatter (defaults to outermost shiʿr if absent).

**Time**: 1 day (first ring-shape implementation).

### §7.2 — Phase ε.2 — Shāṭibī maqāṣid (multi-tier grid 3×5)

**Files**: NEW `traditions/shatibi-maqasid.ts` + REGISTRY update + anchor.ts `drawMultiTierGrid` helper (extension of drawSectorDividers + drawRingBoundaries composition).

**Behavior**: switching to maqāṣid → 3 concentric rings × 5 angular sectors = 15-cell grid with labels (ḍarūriyyāt innermost ring; 5 sectors per ring: dīn / nafs / ʿaql / nasl / māl).

**Verification clause** (Boss test): 15 cells visible with proper labels; stars distributed across cells.

**Time**: 1 day.

### §7.3 — Phase ε.3 — Ibn Khaldūn ʿumrān (binary-flow 2-cell)

**Files**: NEW `traditions/ibn-khaldun-umran.ts` + REGISTRY update + anchor.ts `drawBinaryFlow` implementation.

**Behavior**: switching to Ibn Khaldūn → 2 horizontal bands (badawī below / ḥaḍarī above) with a cycle-arrow connecting them (cyclical-civilizational dynamic).

**Verification clause** (Boss test): 2 bands + cycle arrow visible.

**Time**: 0.5 day.

**Commit message anchors**: `MIG-026 §ε.1`, `MIG-026 §ε.2`, `MIG-026 §ε.3`

---

## §8 — Phase ζ — Jewish family (3 traditions; ζ.2 is the D3 spiral spike)

### §8.1 — Phase ζ.1 — PaRDeS (4 concentric rings)

**Files**: NEW `traditions/pardes.ts` + REGISTRY update.

**Behavior**: switching to PaRDeS → 4 concentric rings (peshaṭ outermost as surface, sod innermost as hidden core; remez and drash between).

**Verification clause** (Boss test): 4 ring boundaries with Hebrew transliterations.

**Time**: 0.5 day (reuses Phase ε.1 ring renderer).

### §8.2 — Phase ζ.2 — Maimonidean prophecy (D3 spiral, 11 steps) — NEW SPIRAL RENDERER

**Files**: NEW `traditions/maimonidean-prophecy.ts` + REGISTRY update + anchor.ts `drawLadderSteps` with `LadderShape: 'spiral'` variant.

**Spiral parametrization**:
- Spiral starts at center, spirals outward to outer rim
- 11 evenly-spaced step-marks along the spiral
- Equiangular spiral with golden ratio expansion: `r(θ) = a * exp(b * θ)` where `a` = small inner offset, `b` chosen so step 11 lands at outer rim
- Each step labeled with prophecy level name + brief tooltip

**Behavior**: switching to Maimonidean → spiral with 11 step marks; stars distributed across steps by `prophecy_level` frontmatter (defaults to step 1 if absent).

**Verification clause** (Boss test):
- **Stage 1**: switch → visible spiral path with 11 labeled steps
- **Stage 2**: spiral starts at center, ends at outer rim
- **Stage 3**: labels readable (rotated tangent to spiral or radial spokes — Plan picks during build)
- **Stage 4**: stars on spiral path

**Risk**: HIGH — first spiral renderer; visual quality may need iteration; label positioning is tricky. Mitigation: build minimal version first; iterate based on Boss feedback.

**Time**: 2 days (the spike; subsequent ladder traditions reuse this work).

### §8.3 — Phase ζ.3 — Talmudic 13 middot (D3 spiral, 13 steps OR toolkit chip-overlay)

**Plan-time decision**: ship as spiral by default (reuses ζ.2 spiral renderer). If 13 steps read as too cluttered in Boss test, fall back to Hillel's 7 middot (cleaner) or chip-overlay toolkit (each middah is a clickable chip applied to a passage).

**Files**: NEW `traditions/talmudic-middot.ts` + REGISTRY update.

**Behavior**: switching to Talmudic 13 → spiral with 13 step marks (or 7 if Hillel-fallback adopted, or chip overlay).

**Verification clause** (Boss test): 13-step spiral visible; if too cluttered, propose fallback in §ζ.3-fix-1.

**Time**: 0.5 day.

**Commit message anchors**: `MIG-026 §ζ.1`, `MIG-026 §ζ.2`, `MIG-026 §ζ.3`

---

## §9 — Phase η — East Asian family (3 traditions)

### §9.1 — Phase η.1 — Mencian 4 sprouts (sectoral 4-cell with optional center)

**Files**: NEW `traditions/mencian-sprouts.ts` + REGISTRY update.

**Behavior**: switching to Mencian → 4 sectors (compassion / shame / deference / right-wrong) with optional 5th central virtue (xìn) ringed in the middle.

**Time**: 0.5 day.

### §9.2 — Phase η.2 — Wang Yangming (binary-flow with central liángzhī)

**Files**: NEW `traditions/wang-yangming.ts` + REGISTRY update.

**Behavior**: switching to Wang Yangming → 2 hemispheres (zhī left / xíng right) unified by central liángzhī disc with bidirectional arrow.

**Time**: 0.5 day.

### §9.3 — Phase η.3 — Korean Sŏngnihak (2×2 grid)

**Files**: NEW `traditions/korean-songnihak.ts` + REGISTRY update.

**Behavior**: switching to Korean Sŏngnihak → 2×2 grid (li/qi vertical axis × four-emotions/seven-feelings horizontal axis).

**Time**: 0.5 day.

**Commit message anchors**: `MIG-026 §η.1`, `MIG-026 §η.2`, `MIG-026 §η.3`

---

## §10 — Phase θ — Latin American + African families (5 traditions; θ.1 + θ.5 are the E3 hub-and-spoke spike)

### §10.1 — Phase θ.1 — Mignolo pluriversal (E3 hub-and-spoke) — NEW RELATIONAL RENDERER

**Files**: NEW `traditions/mignolo-pluriversal.ts` + REGISTRY update + anchor.ts `drawRelationalGraph` with `RelationalShape: 'hub-and-spoke'` variant.

**Geometry**: central disc (modernity/totality) + N outer clusters (subaltern positions: e.g., Andean / Yoruba / Māori / Inuit / Aymara). Each cluster is a small bubble cluster connected to center by a line. Nodes/clusters labeled.

**Behavior**: switching to Mignolo → central disc + 4–6 outer clusters with labels and connecting lines.

**Verification clause** (Boss test):
- **Stage 1**: switch → central disc + outer clusters visible
- **Stage 2**: cluster lines drawn
- **Stage 3**: cluster labels readable

**Risk**: HIGH — first relational renderer. Mitigation: hub-and-spoke fixed layout (E3) avoids force-directed perf cost.

**Time**: 2 days (the spike).

### §10.2 — Phase θ.2 — Dussel transmodernity (binary-flow)

**Files**: NEW `traditions/dussel-transmodernity.ts` + REGISTRY update.

**Behavior**: 2 regions (inner disc = totality; outer ring = exteriority) with directional flow arrows.

**Time**: 0.5 day (reuses Phase ε.3 binary-flow renderer).

### §10.3 — Phase θ.3 — Maldonado-Torres (3 concentric rings)

**Files**: NEW `traditions/maldonado-torres.ts` + REGISTRY update.

**Behavior**: 3 concentric rings (coloniality of power outermost / knowledge middle / being innermost).

**Time**: 0.5 day (reuses ring renderer).

### §10.4 — Phase θ.4 — Akan Wiredu (sectoral 2-3 cell)

**Files**: NEW `traditions/akan-wiredu.ts` + REGISTRY update.

**Behavior**: 2 sectors (nokware verified / ìgbàgbọ́ testimonial) with optional 3rd inquiry-arrow between.

**Time**: 0.5 day.

### §10.5 — Phase θ.5 — Ibuanyidanda (E3 hub-and-spoke relational)

**Files**: NEW `traditions/ibuanyidanda.ts` + REGISTRY update.

**Behavior**: every node connected to central "missing link" hub. Uses same `drawRelationalGraph` as θ.1.

**Time**: 0.5 day (reuses θ.1 relational renderer).

**Commit message anchors**: `MIG-026 §θ.1`, `§θ.2`, `§θ.3`, `§θ.4`, `§θ.5`

---

## §11 — Phase ι — Disclosure layer + manifests

### §11.1 — Phase ι.1 — 24 English manifests in docs/registers/

(Note: filename keeps `registers/` for now since folder rename can be a follow-up — see Plan-time decision below.)

**Files**: 24 NEW manifest files in `docs/registers/<id>.md`. Each carries:
- `id`, `name`, `family`, `shape`
- `citation` (primary + modern, Chicago style — from Agent 1 research)
- `scope` (CARE-aligned: when to use, when not to use)
- `applicability` (concrete domains)
- `lineage` (intellectual history)
- `critique` (known critiques in scholarly literature — per Drabinski's "make ideology visible")
- `version: 1`, `changelog: 2026-MM-DD initial`

**Folder rename decision**: `docs/registers/` or `docs/traditions/`? K1 says full rename; default = `docs/traditions/`. But MIG-025 Plan §C.7 still mentions `docs/registers/`. Plan: rename to `docs/traditions/` to honor K1.

**Behavior shipped**: 24 manifests on disk, not yet wired to ⓘ chip (that's ι.2).

**Verification clause**: light Boss spot-check — open `docs/traditions/aristotelian.md` and `docs/traditions/maimonidean-prophecy.md`; confirm format consistent + scope-statements present.

**Time**: 2 days (substantial prose; ~300 words × 24 = ~7,200 words total).

### §11.2 — Phase ι.2 — ⓘ button (J3) + scope strip (J5)

**Files**:
- `src/lib/sight/v6/traditionChip.svelte` — add ⓘ icon per chip in dropdown; click opens manifest in NotePane (via existing `onOpenNote` callback or new `openManifest(id)` callback)
- Same file — add scope-strip rendering under each chip in dropdown panel
- `src/lib/sight/v6/SightV6.svelte` — wire `openManifest` callback to mount the manifest as a note

**Behavior**: each chip in dropdown shows scope-strip below name; ⓘ button opens full manifest in NotePane.

**Verification clause** (Boss test):
- **Stage 1**: open dropdown, see scope strips
- **Stage 2**: click ⓘ on pramāṇa, manifest opens in NotePane
- **Stage 3**: manifest is readable + citation present

**Time**: 1 day.

**Commit message anchors**: `MIG-026 §ι.1`, `MIG-026 §ι.2`

---

## §12 — Phase κ — User-definable tradition layer

### §12.1 — Phase κ.1 — Declarative JSON layer + JSON Schema + loader

**Files**:
- NEW `docs/traditions/schema/tradition.v1.schema.json` — JSON Schema for declarative user-defined traditions
- NEW `src/lib/sight/v6/traditions/userDefinedLoader.ts` — boot-time scanner of `<Universe>/.constellation/traditions/*.json`, parses + validates against schema + registers in REGISTRY
- `src/lib/sight/v6/traditions/index.ts` — REGISTRY extended to include user-defined entries
- `src/lib/sight/v6/SightV6.svelte` — call `userDefinedLoader.loadAll()` on mount

**Behavior**: users can author JSON tradition files at `<Universe>/.constellation/traditions/<id>.json` matching the schema; they appear in chip dropdown alongside curated traditions.

**Verification clause** (Boss test):
- **Stage 0**: Eisa creates `<Universe>/.constellation/traditions/test-tradition.json` with minimal schema content
- **Stage 1**: restart Constellation, open Sight, see `test-tradition` in chip dropdown
- **Stage 2**: switch to it, dome renders per declarative shape
- **Stage 3**: schema-version-mismatch test: change `schema_version: 1` → `schema_version: 99`, confirm warning + graceful skip

**Time**: 2 days.

### §12.2 — Phase κ.2 — TS plugin loader (H1 Obsidian-trust + default-off + manual enable)

**Files**:
- NEW `src/lib/sight/v6/traditions/pluginLoader.ts` — dynamic `import()` scanner of `<Universe>/.constellation/traditions/*.ts`; Obsidian-trust model
- `src/lib/libraries/store.ts` — add `appSettings.sight.enabledTraditionPlugins: string[]` (default empty, user opts in per-plugin)
- `src/lib/sight/v6/SightV6.svelte` — UI for plugin opt-in (notice on first detection: "Constellation found a tradition plugin file at <path>. Enable it? (yes/no)")
- Plus the warning UI for plugin-load errors

**Behavior**: users can write TS plugin tradition files; first time Constellation detects one, user is prompted; if enabled, plugin loads via `import()`; if it throws on load, warning logged + plugin skipped.

**Verification clause** (Boss test):
- **Stage 0**: Eisa writes a `<Universe>/.constellation/traditions/test-plugin.ts` with a minimal TraditionModule export
- **Stage 1**: open Sight, prompt appears
- **Stage 2**: enable, plugin loads, chip dropdown shows it
- **Stage 3**: error test — break the plugin syntactically; restart; warning visible; other traditions still work

**Risk**: HIGH — security model (Obsidian-trust = full trust once enabled); plugin can crash app on load. Mitigation: try/catch around dynamic import + warning logging.

**Time**: 2 days.

**Commit message anchors**: `MIG-026 §κ.1`, `MIG-026 §κ.2`

---

## §13 — Phase λ — Translation cascade

### §13.1 — Phase λ.1 — English baseline manifests + i18n keys ship in main MIG ship

(Already covered by Phase ι.1.)

### §13.2 — Phase λ.2 — 14-locale translation cascade (follow-up commit)

**Files**: 24 manifests × 14 locales = 336 NEW files at `docs/traditions/<lang>/<id>.md`.

**Each file**: AI-generated translation of the English manifest. Per §A.15 precedent — frontmatter `translation_status: AI-generated 2026-MM-DD — native-speaker review recommended`.

**Plus**: ~10–20 new i18n keys for chip-related strings (favorites pin/unpin tooltip, ⓘ button label, etc.) in 15 locale files.

**Behavior**: users in non-English locales see manifest content in their language; chip strings localized.

**Verification clause**: light Boss spot-check — open `docs/traditions/ar/pramana.md`, confirm Arabic translation present.

**Time**: 1 day (AI-generated; mostly time-budgeted on Write operations).

**Commit message anchor**: `MIG-026 §λ — translation cascade (336 files)`

---

## §14 — Phase μ — Ship gate

**Goal**: Final verification + Concept Paper bump + orientation bump + Boss-test ship-cycle.

### §14.1 — Channel-isolation test (per §C.9, deferred from MIG-025)

**Files**: NEW `tests/sight-v6/tradition-isolation.test.ts` — programmatically switch through all 24 traditions; assert mini-dome encodings (channel labels + spatial layouts) stay constant.

**Verification clause**: vitest green on all 24 traditions × 4 mini-dome channels = 96 assertions.

**Time**: 0.5 day.

### §14.2 — Performance test (per §11.3 16ms cross-filter)

**Files**: NEW `tests/sight-v6/tradition-perf.test.ts` — switch through all 24 traditions on a 7,600-note test universe; assert each switch ≤16ms.

**Verification clause**: vitest green; perf budget honored.

**Time**: 0.5 day.

### §14.3 — Concept Paper v4.0 → v4.1

**Files**: NEW `docs/Constellation-Sight-Concept-Paper-v4.1.md` (preserving v4.0 alongside per the versioning rule). Expansion: §4.1.5 through §4.1.24 (19 new tradition subsections) per the family structure. Plus §4.2 reduced to just §4.2.1 (Mohist sān biǎo) as the v1-preview survivor.

**Time**: 2 days (substantial scholarly prose).

### §14.4 — Boss-test cycle for the 24-tradition ship

**Verification clause** (Boss-test, multi-stage):
- **Stage 1**: chip shows favorites + dropdown; family categorization works
- **Stage 2**: switch through all 24 traditions sequentially; confirm each visual shape rendered correctly
- **Stage 3**: mini-domes UNCHANGED across all 24 switches (§11.6 isolation)
- **Stage 4**: ⓘ disclosure: open 4 random manifests, confirm scope/citation/critique present
- **Stage 5**: user-defined: authored declarative JSON tradition works; authored TS plugin tradition works (with consent flow)
- **Stage 6**: persistence: pin/unpin survives; active tradition survives
- **Stage 7**: performance: switches feel instant; mini-dome filter responses still ≤16ms

**Time**: 1 day Boss-test + ~1 day fix cycles.

### §14.5 — Orientation v-bump documenting MIG-026 ship

Orientation v2.11 → v2.12 (or higher, depending on how many bumps land mid-cascade). Documents the 24-tradition ship + the user-definable layer + the full rename + the new top-principal rules.

### §14.6 — Tag + backup per Backup Routine

Per CLAUDE.md backup routine: `git tag milestone/sight-v6.3-traditions-ship <commit>` + ZIP archive to `E:/Backups/Constellation/`.

**Commit message anchor**: `MIG-026 §μ — Sight v6.3 ship gate (24 traditions + user-definable + ladder/relational/gradient)`

---

## §15 — Phase decomposition summary

| Phase | Type | Boss-test | Time | Commits |
|---|---|---|---|---|
| 0 — K1 rename | refactor | light | 1d | 1 |
| α — architecture foundation | refactor | spot | 1d | 1 |
| β — chip UI redesign | feature | full | 2d | 1 |
| γ — Polanyi + Mohist | feature | full | 1.5d | 1 |
| δ.1 — Peirce + Habermas | feature | full | 0.5d | 1 |
| δ.2 — Dewey + Husserl + Longino | feature | full | 1d | 1 |
| ε.1 — Ibn Rushd burhān (rings spike) | feature | full | 1d | 1 |
| ε.2 — Shāṭibī maqāṣid (grid) | feature | full | 1d | 1 |
| ε.3 — Ibn Khaldūn ʿumrān | feature | full | 0.5d | 1 |
| ζ.1 — PaRDeS | feature | full | 0.5d | 1 |
| ζ.2 — Maimonidean (spiral spike) | feature | full | 2d | 1 |
| ζ.3 — Talmudic | feature | full | 0.5d | 1 |
| η.1 — Mencian | feature | full | 0.5d | 1 |
| η.2 — Wang Yangming | feature | full | 0.5d | 1 |
| η.3 — Korean Sŏngnihak | feature | full | 0.5d | 1 |
| θ.1 — Mignolo (relational spike) | feature | full | 2d | 1 |
| θ.2 — Dussel | feature | full | 0.5d | 1 |
| θ.3 — Maldonado-Torres | feature | full | 0.5d | 1 |
| θ.4 — Akan Wiredu | feature | full | 0.5d | 1 |
| θ.5 — Ibuanyidanda | feature | full | 0.5d | 1 |
| ι.1 — 24 manifests | docs | spot | 2d | 1 |
| ι.2 — ⓘ + scope strip | feature | full | 1d | 1 |
| κ.1 — declarative JSON layer | feature | full | 2d | 1 |
| κ.2 — TS plugin loader | feature | full | 2d | 1 |
| λ — translation cascade | docs | spot | 1d | 1 |
| μ — ship gate | tests + ship | full | 2d | 1–3 |

**Total**: 21 commits in the build path. Plus orientation bumps + session log entries throughout. Plus fix-N cycles for whatever Boss tests reveal.

**Total time**: ~25 days of focused work. Real calendar time is longer due to Boss-test waits.

---

## §16 — Risks recap (carried from Architect §5)

- **Perf** (24 traditions × per-star remap): benchmark in Phase μ.2; if regressed, may need optimization
- **D3 spiral renderer** (ζ.2): visual quality may need iteration
- **E3 relational renderer** (θ.1): hub-and-spoke layout for 4–6 outer clusters
- **κ.2 plugin loader**: Obsidian-trust security model — explicit consent, no sandbox
- **Concept Paper prose quality**: 19 new subsections × ~500 words = ~9,500 words of new scholarly prose

---

## §17 — What's not in this Plan (deferred)

- **Per-note frontmatter opt-in for tradition-kind fields** (`pramana_kind`, `masadir_source`, `burhan_kind`, etc.) — these need Rust-side extraction. Separate MIG (MIG-027?).
- **Wasm/QuickJS sandbox for TS plugin layer** — Plan ships Obsidian-trust H1. Wasm sandbox is MIG-028.
- **v4.1 polish** for individual traditions (per-quadrant radial-internal structure in pramāṇa; sub-sector annotations in masādir; emanation-rings labeling in Ishrāqī — but Ishrāqī is EXCLUDED).
- **CARE consent flow for Indigenous knowledge** (Indigenous traditions all excluded by religious-lineage rule, so non-issue).
- **Federation (cUniverse) implications for user-defined traditions** — user-defined live per-universe; federation question is "do federated universes' traditions appear in chip?" Deferred to a future MIG if Eisa wants.

---

**End of MIG-026 Plan doc.**

Awaiting Eisa approval. Per Plan-Approval-Equals-Build-Approval: once approved, I cascade through Phase 0 → Phase μ autonomously, stopping at user-testable verification clauses per the Testing Instructions Rule. Standing Order #1 + #6 honored throughout (session log + orientation bumps).

If Eisa wants to revise a phase's verification clause, file boundary, or split/merge a phase before approval, surface and re-Plan.
