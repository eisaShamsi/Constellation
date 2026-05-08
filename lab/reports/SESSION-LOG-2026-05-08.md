# Session log — 2026-05-08

**Cascade**: MIG-019 §2G.3 redesign cascade → §2G.3a → §2G.3b → §2G.3c → §2G.3d → §2G.3e → §2G.3f → §2G.3g → §2G.3h → §2G.3i. Eight test rounds with Eisa, eight commits + pushes, eight orientation v-bumps inline (v1.58 → v1.65). Lens-style zoom architecture finalized at the end of the day, awaiting §2G.3i Boss verdict.

The work this session is the second half of the §2G visual redesign that started yesterday evening (2026-05-07) — see `SESSION-LOG-2026-05-07.md` for §2G.1 → §2G.3 plus the (X, Y, Z) per-mode grammar approval.

---

## Commits today (chronological)

| Commit | Phase | Visible to user |
|--------|-------|-----------------|
| `7b1ecd3` | §2G.3e | de-spoke (hash jitter + 6-iter wedge repulsion) + smaller rim font 15→12 px + placeholder text "MDS embedding" → "fetching layout" |
| `6156f9a` | §2G.3f | numbered rim (1, 2, 3 …) colored per library + library legend panel (left for LTR, right for RTL) + library color palette (golden-angle hue stride) + node size cap (max r=4) + node stroke (0.6 px ink) + stronger repulsion (MIN_DIST 6→9, iter 6→12) + sqrt radial mapping (uniform area density) + silent loading |
| `4c8e2dc` | §2G.3g | Pixi `autoDensity: true` + `resolution: devicePixelRatio` (browser zoom no longer breaks layout) + `chartContainer` wrapping all chart layers + wheel zoom + drag pan + Reset View button + close button bumped to z:100 |
| `8596bed` | §2G.3h | `pickStar` inverse-transform (hover/click hit accurate when zoomed/panned) + selection ring r+1 stroke 1.5 (was r+3 stroke 2 — ring now matches node) + cream backdrops on Universe Health + Universe-name (no star bleed-through) + edges incident-only with cap 50 + 1-hop neighbour rings + side panel z-index 5→50 + Esc cascade (clear → reset → close) + close button z-index 100→1000 + wheel via `addEventListener({passive:false})` |
| `1592663` | §2G.3i | **Lens-style zoom architecture**: `.sight-v3-zoom-wrapper` wraps canvas + chart overlays; ONE CSS transform scales them together. Pixi `chartContainer.scale` REMOVED (CSS does it all). + close button via `bind:this` + raw `addEventListener('click', ...)` (bypasses Svelte `onclick` after four failed rounds) + Reset View always visible (.reset-active class for prominent state) + tooltip moved OUTSIDE the lens wrapper (CSS `position: fixed` was breaking inside transformed parent) + tooltip z-index 50→1500 + ::first-line bold for note title + edges darkened from light gold to dark burnt-amber for visibility on cream |

All commits pushed to `origin/main`. Origin tracks `1592663` at end of day.

---

## Eisa-flagged bugs and how each round addressed them

| Round | Eisa report | §2G.3x fix |
|-------|-------------|-----------|
| §2G.3 | "Why are nodes clustered toward the rim? Fix the bleeding fonts. Universe name above the dome." | §2G.3c (label truncation, Universe-name header), §2G.3e (rank-percentile radius), then sqrt mapping in §2G.3f |
| §2G.3e | "Better! I like how it renders. But why the nodes form spokes inside each wedge?" | The spoke fix actually shipped IN §2G.3e (hash-based azimuth jitter) — Eisa was looking at §2G.3d output |
| §2G.3f | "Replace rim labels with numbers. Library legend on the side. Nodes colored per library. Size cap. Stroke. Repulsion." | §2G.3f shipped all of these |
| §2G.3g (test) | "Mouse hovering displaced. Universe Health overlap dome. Hit-testing broken. Close button doesn't work. Reset button invisible. Edges overwhelming." | §2G.3h addressed all but the close button and lens-zoom mismatch |
| §2G.3h (test) | "Close button STILL doesn't work. Whole-page lens zoom needed (everything scales together, not just the chart). Note title still not visible." | §2G.3i — full lens architecture redo + close via raw addEventListener |
| §2G.3i | (pending Eisa test) | TBD |

---

## State of standing (SO #5 record)

### Verified shipped + protected
- §2G.1 visual spec doc (committed yesterday, `bfb8aba`)
- §2G.2 pure helpers polar.ts/modes.ts/regions.ts (`b1a2477`)
- §2G.3 → §2G.3c rolled commit (yesterday, `7d6fcf6`)
- §2G.3d (X, Y, Z) per-mode grammar (yesterday, `467b9f2`)
- §2G.3e → §2G.3i: today's five commits, all pushed

### At-risk / pending Boss test
- §2G.3i (the lens architecture) — built, installer ready at `~13:17`, awaiting test result. Two critical fixes: lens zoom (everything scales together) + close button via raw addEventListener.

### Known-broken (until §2G.3i Boss test passes)
- Close button (four rounds of failed Svelte `onclick` — now using raw DOM addEventListener as last-resort)
- Possibly: hit-testing under non-default zoom/pan (refactored in §2G.3i; algebra verified before commit)

### Pending but not started
- §2G.4: Mode toggle UI + 600 ms eased migration animation
- §2G.5: Mode persistence (`appSettings.sight.lastMode`)
- §2G.6: 3-agent audit + tag milestone + orientation v1.66 + i18n keys for mode names + close-out
- MIG-020 (layer peeling + v2 retire)

### Documentation drift (this PCS resolves)
- Help: master `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` was a v2-era doc with a "🚧 being rebuilt" banner. PCS updates it for v3 polar layout.
- User Manual §8 (Constellation Sight) was v2-era (gravity-well graph). PCS rewrites for v3.
- Translations (`docs/help.{lang}/Constellation Sight/`) deferred to §2G.6 (close-out batch).
- MoCh due (last `MoCh-2026-05-07-2200.md`; today is ~5 hours of direct chat).

### Orientation chain
- v1.58 → v1.65 (8 bumps today, all inline with the trigger commit per the inline-orientation rule).
- Each preserves prior versions in `docs/` per SO #6.

---

## What §2G.3 → §2G.3i taught us

1. **Architectural pivots come at higher altitudes than the implementation guesses.** Eisa's "lens" framing for zoom (everything scales together) was a single sentence that invalidated three rounds of "chart zooms while overlays stay anchored" implementation. The fix wasn't more CSS knobs — it was rethinking what the user expected. Always ask "what's your mental model?" before optimizing.

2. **Svelte 5 `onclick={fn}` can fail silently in some contexts and we don't know why.** Four rounds of escalating defensive fixes (lambda, stopPropagation, z-index 100 → 1000, type="button", explicit invocation) all kept hover-style firing while click events vanished. `bind:this` + raw `addEventListener('click', fn)` is the bulletproof escape hatch. When the framework binding fails inexplicably, drop to raw DOM.

3. **CSS `position: fixed` is broken inside `transform`-ed ancestors.** Spec quirk — fixed becomes relative to nearest transformed ancestor instead of the viewport. The tooltip stayed broken until I moved it OUTSIDE the lens wrapper.

4. **Per-step orientation v-bump caught its first violation today**. Earlier in the §2G.1 → §2G.3c rollout I batched the v1.58 → v1.59 update into a separate "SO catch-up" commit. Per `feedback_orientation_inline_with_commit.md` (top principal), that's a violation. After Eisa's "Have you SO's?" check-in last night, every commit since has carried its own orientation v-bump inline. v1.59 (catch-up) → v1.60 (§2G.3d inline) → v1.61 → v1.62 → v1.63 → v1.64 → v1.65 (§2G.3i inline) — clean discipline.

5. **The (X, Y, Z) grammar is the right architectural anchor.** Eisa's pivot from "switchable rim axis" to "per-mode (X, Y, Z) where each mode picks its own azimuth/radius/magnitude" elevated Sight from a chart with one toggle into a multi-instrument cognitive lens. The visual spec at v1.0 vs v1.1 captures this.

---

## Next session

§2G.3i Boss test verdict. If PASS:
- §2G.4: mode toggle UI (top-right 6-button bar: R · L · T · C · S · A) with 600 ms eased migration animation. R/L/T light up, C/S/A dimmed "available later".
- §2G.5: persist `appSettings.sight.lastMode` per Universe.
- §2G.6: 3-agent audit + tag MIG-019 milestone + orientation v1.66 + i18n keys for mode names + close-out.

If FAIL on close button or lens zoom: another round.
