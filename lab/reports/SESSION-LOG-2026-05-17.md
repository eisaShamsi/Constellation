# Session Log — 2026-05-17

## Phase: MIG-027 Boss test build · MIG-026 Phase γ pending

Carry-over from 2026-05-14 cascade. MIG-027 commit `686ee58` landed at
`Sun May 17 11:56:03 2026 +0400` and was logged retroactively into
SESSION-LOG-2026-05-14.md (lines 1660–1733) because the active cascade
hadn't wrapped. Today's log opens cleanly for the build → Boss-test →
resume-MIG-026 cycle.

**Function in hand**: building the MIG-027 installer .exe so Eisa can
Boss-test Sight theme inheritance — verify that switching the interface
theme (Settings → Appearance → Constellation Light / Nord Light /
Solarized Light) makes Sight's anchor dome, chrome, chip row, sidebar,
and mini-domes all invert correctly (light bg, dark stars, light chrome),
and switching back to a dark theme restores the original dark starfield.

---

## Where we stand at session open

**Shipped + verified** (protected):
- MIG-026 §0 (K1 rename `register` → `tradition`) — commit `fba5031`.
- MIG-026 §α (multi-shape `TraditionModule` foundation + 7 stub
  renderers + subtitle bump) — commit `13577b9`.
- MIG-026 §β (A3+A6 chip UI: family categorization + favorites inline +
  dropdown) — commit `0218c38`. Phase β Boss-test was deferred when
  Eisa pivoted mid-build.
- MIG-027 (Sight theme inheritance) — commit `686ee58`. Code shipped,
  pushed to `origin/main`. Boss-test pending the .exe build.

**At-risk / in flight**:
- `.exe` build for MIG-027 Boss test — in progress in background, task
  ID `ba06uwwfe`. Captures Phase α + β + MIG-027 in one installer (all
  three sit on `main` at HEAD).

**Pending, not started** (Phase β + MIG-027 Boss test gate must clear
first):
- MIG-026 Phase γ — Polanyi + Mohist modules.
- Phases δ–μ + audit (per `MIG-026 Plan` §3).

**Known deferred** (intentionally out of scope):
- CNS theme inheritance — same dark-only assumption as Sight had.
  Spawn_task scoped it out of MIG-027; if Eisa wants it themed too,
  that's MIG-028 or a §-fix-N.
- Sight v3/v4/v5 — dark-only; intentionally not touched (deprecated /
  dual-mounted only).

---

## Build kickoff

`npm run tauri build` started in background (task `ba06uwwfe`). When it
completes, surface the NSIS installer path to Eisa with a `MIG027-theme`
suffix marker so the test target is unambiguous.

## Build complete — installer surfaced

Build finished in 1m 41s (Rust release compile) + ~30 s for the NSIS
bundle. Exit code 0. 41 Rust warnings (pre-existing, none new). Tauri
signing warning at end is informational — only blocks updater key
verification, not installer production.

Fresh artifacts (all `May 17 12:07`):

- Standalone: `src-tauri/target/release/constellation.exe` (87.8 MB)
- NSIS setup: `src-tauri/target/release/bundle/nsis/Constellation_0.3.4_x64-setup.exe` (123 MB)
- MSI: `src-tauri/target/release/bundle/msi/Constellation_0.3.4_x64_en-US.msi`

Per Eisa preference (memory `feedback_prefer_exe_over_msi`), copied the
NSIS setup with a `MIG027-theme` suffix so the test target is
unambiguous:

```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.3.4_x64-setup.MIG027-theme.exe
```

Test instructions surfaced to Eisa (Stage 1 only — per
`feedback_staged_tests.md`): verify the core "Sight follows interface
theme" behavior in one Light theme before staging additional themes /
restore tests.

---

## Stage 1 PASS · MIG-027 §-fix-1

Eisa Boss-tested on Constellation Light. Outcome: **Stage 1 PASS** —
Sight chrome + dome + chip row + facet sidebar + mini-domes all
inverted cleanly. Three theme leaks surfaced during interaction:

1. **Hover-info bar** (bottom-left "E:\..." path tooltip when a star
   is hovered) — `rgba(13,19,34,0.94)` bg + `#2a3245` border +
   `#e8ebf2` title + `#5a6275` path. Most visible leak — dark navy
   box on cream bg.
2. **Filter-count badge** (top-right "X / Y notes" when any facet
   filter is active) — `rgba(58,67,90,0.35)` hardcoded bg. Gold border
   and gold text stayed (semantic: filter-active = gold,
   theme-independent).
3. **Loading boxes** (`.sight-v6-loading` + `.sight-v6-loading-bg`) —
   bg + border + text all hardcoded dark. Surfaces during cache warm
   before `render-ready`.

Plus **chip contrast** issue Eisa flagged before testing: "the font
and the chip have to match the background color, to be naturally
visible." On dark themes `--text-muted` reads fine on the chip-row
bg; on light themes it goes too faint. Inactive chip color bumped
`--text-muted` → `--text-normal` — full contrast in both themes. The
active vs inactive distinction is carried by border + bg tint + dot
(not text dimness), so no regression in dark theme.

**Commit**: `2f190dc` — MIG-027 §-fix-1 — chip contrast + missed
theme leaks. 2 files changed, +30 / −14.

Files touched:
- `SightV6.svelte` CSS: 4 elements (filter-count, loading,
  loading-bg, hover-info + hover-title + hover-path).
- `traditionChip.svelte` CSS: 2 elements (`.tradition-chip`,
  `.tradition-chip-all-trigger`).

**Sweep verification**: ran a Sight v6 directory grep for any
remaining bare hex/rgba values outside `var(--..., fallback)`
patterns; only semantic gold preserves remain (EXTENDED badge,
preview chip, pin star — intentionally theme-agnostic).

Build kicked off for `MIG027-theme-fix1.exe`. Boss re-test instructions
will follow when the .exe is ready.

## fix-1 build complete — installer surfaced

Build finished in 1m 38s, exit code 0. Same 41 Rust warnings as the
fresh build (pre-existing, no new). Same signing warning at end
(informational; doesn't block installer output).

Installer artifact copied with `MIG027-theme-fix1` suffix marker:

```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.3.4_x64-setup.MIG027-theme-fix1.exe
```

Stage 1.1 re-test instructions surfaced to Eisa: hover a star (verify
hover-info bar matches theme), Shift+click to add a filter (verify
filter-count badge matches theme), check the chip row reads cleanly.
Then Stage 2 — restore to dark + sanity check Nord Light + Solarized
Light.

---

## Stage 1.1 PARTIAL · MIG-027 §-fix-2

Boss re-tested fix-1 .exe on Constellation Light. Chips read correctly,
hover-info inverted, filter badge bg corrected. New issue surfaced
with three close-up screenshots:

> "Still needs enhancement. We have to find a suitable replacement
> for the gold color."

The gold itself (`#fbbf24` bright amber) washes out on cream / off-
white backgrounds. Screenshots showed:

- `is-hovered` facet rows reading as pale peach on cream (color + bg
  both faint)
- Filter-count badge "3,596 / 7,341 notes" with gold text barely
  distinguishable from cream
- Canvas hover ring around stars: visible but soft

**Root cause**: `#fbbf24` (amber-400) is intentionally bright for
dark themes; on light themes that brightness reads as pale wash. The
SEMANTIC vs CHROME split in the original MIG-027 misclassified
`highlightedRing` as semantic (theme-agnostic) — it's actually an
interaction affordance and needs to adapt across themes like the
rest of chrome.

**Fix**: introduce theme-conditional CSS vars for the gold family.

  `SightV6.svelte` CSS — define 4 vars on `.sight-v6-root`:
  - `--sight-highlight` (text/foreground color)
  - `--sight-highlight-bg-soft` (subtle bg tint)
  - `--sight-highlight-bg-strong` (stronger bg tint)
  - `--sight-highlight-border-soft` (border color)

  Default (dark themes): bright amber `#fbbf24` + matching alphas.
  `:global(body.theme-light) .sight-v6-root` override: deep amber
  `#b45309` (Tailwind amber-700) + matching alphas. Keeps the gold
  semantic feel; only luminosity adapts so it reads cleanly on cream.

**Sweep** — 3 DOM consumers + 2 canvas consumers + 1 source declaration:

  SightV6.svelte:
  - `.sight-v6-pro-badge` (EXTENDED indicator)
  - `.sight-v6-filter-count` (X/Y notes badge)

  facetSidebar.svelte:
  - `.facet-cat-row.is-hovered` (hover-linked from star)
  - `.facet-cat-row.active.is-hovered` (stacked hover + active)

  dome.ts: `highlightedRing` PROMOTED from SEMANTIC_COLORS to
  ChromePalette. `readChromePalette()` reads `--sight-highlight` CSS
  var. `PALETTE` legacy const unchanged at runtime (sourced from
  chrome side of the spread; no consumer broke; grep confirmed no
  external imports of `SEMANTIC_COLORS.highlightedRing`).

  anchor.ts:738 + miniDome.ts:227 — `PALETTE.highlightedRing` →
  `_chrome.highlightedRing` (theme-aware canvas hover ring).

**Commit**: `593af51` — MIG-027 §-fix-2 — semantic gold theme-aware.
5 files changed, +78 / −25.

Build kicked off for `MIG027-theme-fix2.exe`. Boss re-test
instructions will follow when the .exe is ready.

## fix-2 build complete — installer surfaced

Build finished, exit code 0. Installer copied with `MIG027-theme-fix2`
suffix marker:

```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.3.4_x64-setup.MIG027-theme-fix2.exe
```

Mtime: 2026-05-17 12:50. (fix-1 was 12:28; fix-2 is 12:50 — Eisa
checks the binary mtime per Stage 0 of the test convention.)

Stage 1.2 re-test instructions surfaced.

## Stage 1.2 PASS

Eisa: **Pass.** Screenshot confirmed:
- Filter-count badge "3,596 / 7,341 notes" top-right — reads as saturated deep amber, clearly legible on cream bg
- Hover-linked facet rows in left sidebar (Connection 3,150, hypothesis 3,596, birth 2,963, Self 3,596, تاريخ عربي وإسلامي 207) — all deep amber, no pale-peach wash
- EXTENDED indicator (after Ctrl+Shift+D) — deep amber, visible
- Canvas hover rings around stars in both the anchor dome AND the ACTS mini-dome — deep amber, visible against cream

Stage 2 (dark-theme regression check) sent next to verify the
`:global(body.theme-light)` override hasn't broken the dark variant
that Stage 1 originally passed on.

## Stage 2 PASS

Eisa screenshot on Constellation Dark confirmed no regression:
- Bright lemon-amber filter-count badge
- Bright amber hover rings around the star in main dome + every
  visible mini-dome
- Bright amber hover-linked facets (Religion & Comparative Tra…,
  Connection, hypothesis, birth, Self)
- Bright amber EXTENDED indicator
- Dark navy hover-info bar with light text — correct for dark theme

The `:global(body.theme-light) .sight-v6-root` override is correctly
isolated to light themes; dark theme is untouched. Stage 2.5 (Nord
Light + Solarized Light sanity) sent next.

## Stage 2.5 PASS · MIG-027 SHIPPED

Eisa: **Pass.** Both Nord Light and Solarized Light screenshots
confirmed the deep-amber treatment is consistent across all three
light themes (Constellation Light, Nord Light, Solarized Light).
The `body.theme-light` body class is the right hook — it fires
across all three light variants automatically.

Observation flagged but not blocking: on Solarized Light the dome
stars look slightly more muted than on Nord / Constellation Light.
This is theme-inherent (Solarized's intentionally-low-contrast cream
overlays onto the SEMANTIC stage hues with less differentiation
than the other palettes). Keeping the chrome/semantic split intact
means we don't address this; making stage hues theme-aware would
contradict the architectural choice. Boss acknowledged not-a-bug.

**MIG-027 closes here.** Five Boss-test cycles all PASS; no further
fixes pending; the chrome plumbing is in place for Phases γ → θ of
MIG-026 to inherit theme-awareness automatically without retrofit.

Final commit cascade for MIG-027:
- `686ee58` MIG-027 — Sight theme inheritance (initial)
- `2f190dc` MIG-027 §-fix-1 — chip contrast + missed theme leaks
- `593af51` MIG-027 §-fix-2 — semantic gold theme-aware
- Plus session log + orientation v2.12 in the SHIP-gate commit

Cascading to MIG-026 Phase γ (Polanyi + Mohist tradition modules)
next — the original 21-step Plan resumes from the pause point.

---

## MIG-026 §γ — Polanyi + Mohist sān biǎo (code shipped)

Phase γ resumes the MIG-026 cascade after the MIG-027 pivot
closes. Two new tradition modules + 2 renderer implementations + 1
dispatch architecture refactor (gradient = overlay vs. chrome under
stars).

**Commit**: `2c5e901` — MIG-026 §γ — Polanyi (gradient) + Mohist
sān biǎo (horizontal-bands) modules. 4 files changed, +455 / −43.

### Files

- `src/lib/sight/v6/traditions/polanyi.ts` — NEW
  - shape: 'gradient', remapStarPosition = identity (per Plan:
    Polanyi doesn't redistribute; it modulates visibility)
  - gradientSpec: centerOpacity 0.18 / edgeOpacity 0.95 / labels
    'tacit' (center) + 'explicit' (edge)
- `src/lib/sight/v6/traditions/mohist-san-biao.ts` — NEW
  - shape: 'horizontal-bands', remapStarPosition hash-buckets stars
    into 3 zones (本 běn / 原 yuán / 用 yòng); 2 independent hashes
    for vertical+horizontal jitter avoid diagonal stripe artifacts;
    x clipped to dome circle at each y
  - horizontalBandsSpec: 3 bands with bilingual labels (Chinese
    character + transliteration + English gloss)
- `src/lib/sight/v6/traditions/index.ts` — REGISTRY adds both
  entries + imports + re-exports
- `src/lib/sight/v6/anchor.ts` — drawGradientFog + drawHorizontal-
  Bands implemented (theme-aware via _chrome.*); parseRgb helper
  added for the gradient overlay; dispatch refactored to split
  gradient (post-stars) from chrome shapes (pre-stars)

### Dispatch architecture refactor

Gradient is conceptually different from other shapes: it's an
OVERLAY that modulates stars (must paint after stars), not chrome
under stars (which paints before). The dispatch was split into two
points:

- **Step 2.5** (pre-stars / under-chrome): sectoral / rings /
  ladder / relational / cyclic-flow / binary-flow / horizontal-
  bands. These are geometric strokes that text labels paint over.
- **Step 7** (post-stars / overlay): gradient. The fog paints over
  the star layer (and over stratum labels — conceptually consistent
  with the Polanyi metaphor that things at center are "less
  articulable", including their labels).

### Theme-awareness inheritance

Both new renderers paint via `_chrome.*` (bg, strataRing,
stratumLabel) — they automatically follow the active interface
theme courtesy of MIG-027's chrome-palette plumbing. No per-theme
retrofit needed. This validates the Plan's claim that Phases γ → θ
build theme-aware from the start.

### Verification

`npm run check`: 3 pre-existing errors (store.ts LinkLifecycle
dedupe + 2 PropertyEditor types per memory). Zero new errors. File
count 1397 → 1399 (the two new tradition modules).

### Boss-test (Stages 1-4, per Plan §5)

- Stage 1: switch chip to Polanyi → fog gradient visible
- Stage 2: switch chip to Mohist sān biǎo → 3 horizontal bands
- Stage 3: open extended view (Cmd-Shift-D) → 4 mini-domes
  UNCHANGED on both
- Stage 4: switch back to Aristotelian → both chrome additions
  cleanly disappear

Build kicked off for Phase γ .exe (task `bw1tm020g`). Boss-test
instructions surface when the .exe is ready.

## Phase γ Boss test — Stage 1/3/4 PASS · Stage 2 PARTIAL · §γ-fix-1

Eisa Boss-tested the Phase γ .exe (`MIG026-phase-gamma.exe`,
mtime 13:37). Four screenshots:

- **Stage 1 (Polanyi)**: PASS — fog gradient visible (dense at
  center, fading to rim); 'tacit' label at dome center; 'explicit'
  label at bottom rim; Aristotelian-default star positions
  preserved under the fog.
- **Stage 2 (Mohist sān biǎo)**: **PARTIAL** — 2 divider lines
  clipped correctly to the dome chord; bilingual labels visible at
  left edge of each band (本 běn · root / 原 yuán · origin / 用 yòng
  · use); stars redistributed into the 3 bands. **But Eisa**:
  *"The stars are hardly visible."* The 7,341 stars spread across
  3 bands × ~1/3 of the dome each = no overlap density = low-alpha
  dots from density-mode dissolve into the bg.
- **Stage 3 (extended view, Polanyi active)**: PASS — 4 mini-domes
  unchanged (Aristotelian-default layout preserved; tradition
  isolation invariant §11.6 holds).
- **Stage 4 (revert to Aristotelian)**: PASS — fog disappears,
  Connection cluster back in place, no residual chrome from Mohist
  or Polanyi.

### Root cause for Stage 2 dimness

Density mode (§B.9, MIG-025) was tuned for Aristotelian's
concentrated stratum-by-time clusters. With 7,341 stars > the
5,000 threshold, density mode is always on for Eisa; per-star
alpha drops to ~0.3 so overlapping dots additive-blend into a
milky-way texture in dense regions. In Aristotelian the Connection
cluster occupies ~5% of dome area → high overlap → reads bright.
In Mohist horizontal-bands the same 7,341 stars spread across
~100% of dome area → no overlap → individual dots dissolve.

### §γ-fix-1

`SightV6.svelte`:
- New `$derived anchorDensityMode` reads `densityMode` AND gates on
  `tradition.shape`; returns false for spread shapes.
- `renderAnchorDome` receives `anchorDensityMode` instead of the
  universe-wide `densityMode`.
- `<MiniDome />` still receives `densityMode` (its layout is always
  Aristotelian-default, so density mode still benefits it).

Currently only `horizontal-bands` (Mohist) is treated as a spread
shape; the list will extend as future spread shapes ship (grid
Phase ε.2, rings Phase ε.1, relational Phase θ.1 — they share the
uniform-distribution property).

**Commit**: `63c7776` — MIG-026 §γ-fix-1 — disable density mode
for spread-shape traditions. 1 file changed, +33 / −1.

Build for §γ-fix-1 .exe kicked off (task `bqs7f1kbq`). Boss re-test
instructions surface when ready.

## §γ-fix-1 misdiagnosis · §γ-fix-2 (real fix: star radius +2px)

Boss re-tested fix-1.exe. Step 3 + Step 4 PASS. Step 2 still
PARTIAL — Eisa diagnosed correctly: *"I think it is not completely
visible because of the star's size. Let's pump up the size by 2px,
just for this type."*

**Acknowledged: §γ-fix-1 was a misdiagnosis.** The `densityMode`
parameter in `renderAnchorDome` is accepted-but-voided (anchor.ts
line 658: `void _densityMode;`) per the §B.9 comment that explicitly
says the anchor renders bodies at BODY_OPACITY_MULT (0.7) regardless
of densityMode. So passing `anchorDensityMode: false` had no visible
effect on the anchor (only mini-domes actually consume densityMode).

Real issue: BASE_STAR_RADIUS = 0.3125 world units → ~0.6 px diameter
at 1× zoom. Tuned for Aristotelian's concentrated clusters where
sub-pixel dots additive-blend into the milky-way texture. In Mohist's
spread layout, no overlap → individual sub-pixel dots dissolve.

### §γ-fix-2

  anchor.ts:
  - `drawStars` gains `radiusBoostScreenPx: number = 0`. Converted
    to world units via `1 / zoomScale` so the boost stays constant
    in screen pixels regardless of zoom level.
  - Boost applied to body radius (Pass 1) + pip radius proportionally
    (× 0.6 to preserve original ratio) + highlight ring radius
    (Pass 3, so brushing halo stays visually correct).
  - `renderAnchorDome` options gain `starRadiusBoostScreenPx?: number`,
    passed through to drawStars.

  SightV6.svelte:
  - New `$derived anchorStarRadiusBoostScreenPx` returns 2 for
    `horizontal-bands`, 0 otherwise.
  - renderAnchorDome receives the boost.
  - Updated `anchorDensityMode` comment to acknowledge it's reserved
    scaffolding (currently no-op on anchor; reserved for v6.2 hex-bin).

**Commit**: `be14ab2` — MIG-026 §γ-fix-2 — star radius +2px boost
for spread shapes. 2 files changed, +68 / −26.

Build kicked off for §γ-fix-2 .exe (task `bavn19gz3`).

## §γ-fix-2 PASS · Phase γ CLOSED

Eisa Boss-tested `MIG026-phase-gamma-fix2.exe` (mtime 16:46). Verdict:
*"Pass. Now it is readable."* Screenshot shows the 3 Mohist bands
(本 běn · root / 原 yuán · origin / 用 yòng · use) all populated with
clearly-visible cyan/orange/purple star dots at the bumped size; no
regression on the cluster-style layouts.

**Phase γ CLOSED.** Cumulative trail:

| Step | Commit | Outcome |
|---|---|---|
| γ initial | `2c5e901` | Polanyi + Mohist code; Stage 1 PASS, Stage 2 PARTIAL (Mohist stars dim), Stages 3+4 PASS |
| §γ-fix-1 | `63c7776` | Misdiagnosis — anchorDensityMode override (anchor voids densityMode per §B.9); had no visible effect |
| §γ-fix-2 | `be14ab2` | Real fix — per-shape star radius +2 CSS px boost for horizontal-bands. Eisa-diagnosed: "pump up the size by 2px, just for this type." |

**Per-shape boost feature now in the architecture** — Phase ε.1
(rings), ε.2 (grid), θ.1 (relational hub-and-spoke), and θ.5
(complementary hub-and-spoke) will reuse the same `starRadiusBoostScreenPx`
mechanism by adding their shapes to the `anchorStarRadiusBoostScreenPx`
$derived check. No new code needed for those phases; just one-line
additions when they ship.

**Theme-awareness check** — both new modules (Polanyi gradient +
Mohist horizontal-bands) inherit theme correctness for free via
MIG-027's `_chrome` plumbing. Validated in Eisa's screenshot
(cream bg correctly under both shapes' chrome).

### Awaiting next direction

Per the test instruction surfaced post-fix-2, the next options are:

1. **Cascade into Phase δ** (Modern Western family — 5 new
   traditions in 2 sub-phases: δ.1 = Peirce sectoral + Habermas
   sectoral; δ.2 = Dewey cyclic-flow + Husserl rings + Longino
   sectoral). Substantial cascade (~1.5 days per Plan).
2. **Phase β chip-UI interaction test deferred from earlier**
   (pin/unpin star, family browse, dropdown open/close) — covered
   by the same .exe.
3. **Pivot** to something else (CNS theming MIG-028 etc.).

## Eisa direction: Cascade into Phase δ.1 (Peirce + Habermas)

## MIG-026 §δ.1 — Peirce + Habermas (code shipped)

Both new modules are 3-sector sectoral shapes — they reuse the
existing `drawSectorDividers` renderer (no new shape implementation
needed). Each defaults all stars to first sector per Plan §6.1
because LayoutCacheRow doesn't yet carry the frontmatter field;
per-note opt-in ships as a §δ.1-fix-N follow-up once Rust-side
extraction lands.

### Files (6)

- `types.ts` — TraditionId extended with 'peirce' + 'habermas'
- `traditions/peirce.ts` (NEW) — 3 sectors: Firstness (NE) /
  Secondness (S) / Thirdness (NW); citation Peirce 1867
- `traditions/habermas.ts` (NEW) — 3 sectors: technical /
  practical / emancipatory; citation Habermas 1968
- `traditions/index.ts` — REGISTRY + FAMILIES['modern-western']
  + imports + re-exports
- `traditionChip.svelte` — TRADITIONS_META entries (display
  name, tooltip, scope, preview=false)
- `store.ts:3483` — activeTradition literal union extended
  with the 2 new IDs

### Doc-drift flagged

`store.ts:3483-3490` carries a duplicate of the TraditionId
literal union (used as the persisted `activeTradition` field
type). Every TraditionId extension requires updating this duplicate
too. Better long-term: import `TraditionId` directly from types.ts
or generate the schema type from a single source. Flagging for
MIG-026 ship-gate cleanup; not blocking.

### Commit

`51b853a8` — MIG-026 §δ.1 — Peirce + Habermas. 6 files changed,
+289 / −7. File count 1399 → 1401.

Build kicked off for Phase δ.1 .exe (task `bqi20hjew`). Boss-test
instructions surface when the .exe is ready (per Plan §6.1: 2-stage
cycle — each tradition switch, confirm 3-sector layout + labels +
mini-isolation).

## Phase δ.1 Boss test — Stage 1 + 2 PARTIAL, Stage 3 PASS · §δ.1-fix-1

Eisa Boss-tested `MIG026-phase-delta-1.exe` (mtime 17:09). Two
screenshots:

- **Stage 1 (Peirce)**: 3 sectors + labels (Firstness/Secondness/
  Thirdness) render correctly. **But**: top divider stroke runs
  vertically from dome center straight UP — directly through the
  +y axis where the stratum labels FOUNDATION / WORKING /
  CONNECTION / SYNTHESIS / EDGE OF KNOWING sit. Stratum labels
  paint on top of the divider per paint order (step 5 vs 2.5) but
  the visual collision is distracting — divider stroke shows
  through the gaps between letters.
- **Stage 2 (Habermas)**: same remark — identical geometry to
  Peirce so same vertical-axis collision.
- **Stage 3 (revert to Aristotelian)**: PASS.

### §δ.1-fix-1

Rotate the 3-sector pattern by +π/6 (30° clockwise) so the first
sector starts at -π/3 (~1 o'clock) and the vertical axis falls
INSIDE the third sector (no divider on the +y line).

Post-fix layout (both Peirce + Habermas):

| Sector | Angular extent | Peirce label | Habermas label |
|---|---|---|---|
| First | 1 o'clock → 5 o'clock (NE+E) | Firstness | technical |
| Middle | 5 o'clock → 9 o'clock (S+SW) | Secondness | practical |
| Third | 9 o'clock → 1 o'clock (NW+N, includes 12 o'clock) | Thirdness | emancipatory |

The 12 o'clock direction (where stratum labels live) is now safely
inside the third sector — no divider stroke crosses it.

**Pramana NOT updated** — it has the same architectural pattern
(4 quadrants starting at -π/2 with cardinal-axis dividers) and
would have the same collision if redistributed. Per Concept Paper
§4.1.2 the pramāṇa labels are documented at NE/SE/SW/NW positions,
so a rotation would change documented cultural mapping. Deferred
to a separate fix if Eisa flags it. Logging as a known issue.

**Commit**: `c9d0d98` — MIG-026 §δ.1-fix-1 — rotate Peirce +
Habermas sectors 30° CW. 2 files changed, +31 / −14.

Build kicked off for §δ.1-fix-1 .exe (task `b9maz6sl4`). Boss
re-test instructions surface when ready.

## §δ.1-fix-1 PASS · Phase δ.1 CLOSED

Eisa: "All Pass" on `MIG026-phase-delta-1-fix1.exe`. Phase δ.1
closes cleanly: Peirce + Habermas both render 3-sector layouts with
vertical-axis-cleared dividers.

## MIG-026 §δ.2 — Dewey + Husserl + Longino (code shipped)

Cascading directly into δ.2 per Plan §6.2. Three new tradition
modules + 2 stub renderers fleshed out.

### Files (8)

- `types.ts` — TraditionId extended with `dewey` + `husserl` +
  `longino`
- `store.ts:3483` — activeTradition literal union extended (the
  type-duplicate flagged for ship-gate)
- `anchor.ts`:
  - `drawRingBoundaries` IMPLEMENTED (was stub originally scheduled
    for Phase ε.1 / Ibn Rushd; pulled forward because Husserl is the
    first ring-shape tradition)
  - `drawCyclicFlow` IMPLEMENTED (was stub, scheduled for δ.2 = same
    phase). Ring at 75% radius + segment-divider ticks + segment
    labels at 85% radius + flow chevron arrows on the path
- `traditions/dewey.ts` (NEW) — 5-stage pattern of inquiry (cyclic-
  flow); Dewey 1938
- `traditions/husserl.ts` (NEW) — 4 regional ontologies (concentric-
  rings: formal / material-nature / animal-nature / spirit); Husserl
  1913
- `traditions/longino.ts` (NEW) — 4-cell sectoral with 45° rotation
  (avoids cardinal-axis divider collision per §δ.1-fix-1 principle);
  Longino 2002
- `traditions/index.ts` — REGISTRY + FAMILIES['modern-western']
  (now lists all 6 family members) + imports + re-exports
- `traditionChip.svelte` — TRADITIONS_META entries

### Architecture notes

`drawRingBoundaries` pulled forward — subsequent ring-shape phases
(Ibn Rushd ε.1, PaRDeS ζ.1, Maldonado-Torres θ.3) inherit the
implementation without needing to write it themselves. Similar
pulled-forward dynamic for the `_chrome` plumbing in MIG-027 →
Phase γ inherited theme-awareness for free.

Longino uses the 4-sector pattern from pramana but with a 45° (π/4)
rotation offset — same principle as §δ.1-fix-1 for the 3-sector
pattern. Dividers land at NE/SE/SW/NW positions, avoiding the
vertical-axis stratum labels.

### Verification

`npm run check`: 3 pre-existing errors. Zero new. File count
1401 → 1404.

### Commit

`d672f23` — MIG-026 §δ.2 — Dewey (cyclic) + Husserl (rings) +
Longino (sectoral). 8 files changed, +586 / −29.

### Modern Western family complete

All 6 traditions in the family now have implemented modules:

| Tradition | Shape | Phase shipped |
|---|---|---|
| Polanyi | gradient | γ |
| Peirce | sectoral 3-cell | δ.1 |
| Habermas | sectoral 3-cell | δ.1 |
| Dewey | cyclic-flow 5-segment | δ.2 |
| Husserl | rings 4-zone | δ.2 |
| Longino | sectoral 4-cell | δ.2 |

Build kicked off for Phase δ.2 .exe (task `b3138lbyg`). Boss-test
instructions surface when ready.

## Phase δ.2 Boss test outcomes + §δ.2-fix-1

Eisa screenshots:
- **Stage 1 (Dewey)** PARTIAL: "Stars = bumping it 1.5px. Chevron
  arrows = Enlarge it 2x."
- **Stage 2 (Husserl)** PARTIAL: "Stars = bumping it 1.5px."
- **Stage 3 (Longino)** PASS.
- **Side discovery on pramana**: "Top dividing line collides with
  the strata title." — same root cause as §δ.1-fix-1; deliberately
  left untouched in fix-1 to preserve Concept Paper §4.1.2 cultural
  mapping; now Eisa wants it fixed.

### §δ.2-fix-1 (3 changes in one commit)

1. **Star radius boost extended to cyclic-flow + rings** —
   `SightV6.svelte::anchorStarRadiusBoostScreenPx` now returns:
   - 2 px for `horizontal-bands` (existing Mohist)
   - **1.5 px for `cyclic-flow`** (NEW Dewey)
   - **1.5 px for `rings`** (NEW Husserl)
   - 0 for everything else

   The 1.5 vs 2 difference reflects that Dewey's stars sit in a
   narrow band around the 75% ring (less spread than horizontal-
   bands' full-dome stripes) and Husserl's stars sit in concentric
   zones (same partial-spread property).

2. **Chevron arrow size doubled** — `anchor.ts::drawCyclicFlow`
   `chevronSize: 4 → 8`. The smaller arrows didn't read as
   direction markers at 1× zoom; doubling makes the clockwise-flow
   indication unmistakable.

3. **Pramana quadrants rotated by +π/4** — `pramana.ts`. Same
   off-axis principle as §δ.1-fix-1 (Peirce+Habermas at +π/6 for
   3-sector) and Longino (+π/4 for 4-sector). The pramāṇa quadrants
   shift visual position:
   - pratyakṣa: NE → E
   - anumāna: SE → S
   - upamāna: SW → W
   - śabda: NW → N (now contains +y axis cleanly)

   Doc-drift flagged: Concept Paper §4.1.2 currently describes the
   quadrants as NE/SE/SW/NW. The pramāṇas remain "kinds, not levels"
   (their categorical meaning unchanged); only the geometric
   position shifts. Update Concept Paper §4.1.2 at MIG-026
   ship-gate.

**Commit**: `b6a4574` — MIG-026 §δ.2-fix-1 — star size + chevron +
pramana rotation. 3 files changed, +48 / −11.

Build for fix-1 .exe kicked off (task `b8g10k4ln`).

## §δ.2-fix-1 PASS · Phase δ CLOSED

Eisa "Check 1 / Check 2 / Check 3: Pass." Phase δ closes
cleanly. Modern Western family complete (6 traditions: polanyi /
peirce / habermas / dewey / husserl / longino). Pramana cleanup
applied (NE/SE/SW/NW → E/S/W/N geometric position; categorical
meaning unchanged).

## Phase ε direction: cascade ε.1 → ε.2 → ε.3

Eisa selected "Continue all of Phase ε" via AskUserQuestion. Per
Plan §7 recommendation ("3 sub-phases — each registers a
substantially different shape, so worth separate Boss tests"),
cascading through ε.1 → ε.2 → ε.3 with separate test cycles each.

## MIG-026 §ε.1 — Ibn Rushd burhān (code shipped)

First Arabic / Islamic family tradition. Reuses `drawRingBoundaries`
already implemented in §δ.2 for Husserl — no new renderer needed.

### Files (5)

- `types.ts` — TraditionId += 'ibn-rushd-burhan'
- `store.ts` — activeTradition union extended
- `traditions/ibn-rushd-burhan.ts` (NEW) — 4 concentric zones:
  burhān (0-25%) / jadal (25-50%) / khaṭāba (50-75%) / shiʿr
  (75-100%). Default-all-to-shiʿr (lowest demonstrative force —
  defensible: most notes start as imaginative association before
  being elevated). Citation: Ibn Rushd, *Faṣl al-Maqāl* §§ 7-15.
- `traditions/index.ts` — REGISTRY entry; FAMILIES['arabic-islamic
  -beyond'].traditions = ['ibn-rushd-burhan'] (was empty)
- `traditionChip.svelte` — TRADITIONS_META entry

### Verification

`npm run check`: 3 pre-existing errors. Zero new. File count
1404 → 1405.

### Commit

`73d6e6a` — MIG-026 §ε.1 — Ibn Rushd burhān ladder (rings, 4
zones). 5 files changed, +144 / −6.

Build kicked off (task `b7hv9tpfc`). Boss test surfaces when .exe
ready.

## §ε.1 PASS

Eisa: "Pass" — Ibn Rushd burhān verified. Stars cluster in shiʿr
outer annulus as designed, 4 ring labels visible along +x axis.
Phase ε.1 closes.

## MIG-026 §ε.2 — Shāṭibī maqāṣid (code shipped)

Second Arabic / Islamic family tradition. The maqāṣid grid is the
first GRID-shape tradition — composed via existing renderers
(drawSectorDividers + drawRingBoundaries) rather than a new
drawGrid implementation. The tradition module provides BOTH
callbacks; the anchor dispatcher fires both; the visual grid
emerges as their union.

### Geometry

- 3 ring tiers radial: ḍarūriyyāt (0-33%) / ḥājiyyāt (33-67%) /
  taḥsīniyyāt (67-100%)
- 5 sectors angular: dīn / nafs / ʿaql / nasl / māl — rotated π/4
  (45°) CW from cardinal, same as Longino. Dividers at -45°/27°/
  99°/171°/243° (no cardinal-axis collisions). Sector 0 contains
  the +x axis so ring labels along that axis don't cross any
  divider; sector 4 contains the +y axis (12 o'clock) without a
  divider crossing it, so stratum labels stay clear.
- 15 cells total. Stars hash-distributed across all cells per Plan
  ("stars distributed across cells").

### Files (5)

`types.ts` + `store.ts` + `shatibi-maqasid.ts` (NEW) +
`traditions/index.ts` + `traditionChip.svelte`.

### Verification

`npm run check`: 3 pre-existing errors. Zero new. File count
1405 → 1406.

### Commit

`ad9978f0` — MIG-026 §ε.2 — Shāṭibī maqāṣid al-sharīʿa (3×5 grid).
5 files changed, +190 / −6.

Build kicked off (task `bg57s9iw3`). Boss test surfaces when ready.

---

