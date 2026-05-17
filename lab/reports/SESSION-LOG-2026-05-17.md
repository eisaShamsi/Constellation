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

---

