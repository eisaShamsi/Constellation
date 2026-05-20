# Session Log — 2026-05-20

## Function in hand
**The Cataloger** — promoting CECE (the Constellation Epistemic Content Engine) from a right-sidebar "Source Review" tab to a first-class **left-dock Core Plug-in** with its own dock button + full-page view. This is **MIG-039** (handover: `lab/reports/MIG-039-CATALOGER-HANDOVER.md`).

## Pre-build checklist (done)
- `git pull origin main` → already up to date.
- Read CLAUDE.md, the MIG-039 handover, the CECE Concept Paper v1.0, LESSONS-LEARNED.md.
- **SO #8 cross-check**: confirmed MIG-039 is NOT stale. No `showCataloger` exists anywhere in `src/`; orientation v2.18 body + preamble both confirm CECE is still the right-sidebar Source Review panel with the left-dock build pending as MIG-039.
- Studied the OrgChart wiring (the pattern to mirror), the two reuse components, and the `enabledFeatures` type/defaults.

## Architectural impact analysis (Working Agreement #4) — what wires this touches, what it cuts
**Purely additive. Cuts nothing.**
- New `CatalogerView.svelte` — touches no existing code.
- `store.ts` — `+ cece: boolean` in `enabledFeatures` type; `+ cece: true` default. Settings-load spread (`...DEFAULT_SETTINGS.enabledFeatures`) back-fills existing users → default ON. No force-off (unlike Map).
- `+layout.svelte` — `+ showCataloger` state, `+ catalogerEverOpened` lazy-mount flag (LL-022) + effect, universe-switch reset, `fullPageActive` term, `content-hidden` term, dock button (gated `enabledFeatures.cece !== false`), lazy-mounted overlay, command-palette entry, escape branch, and `showCataloger = false` beside every sibling-close site.
- CSS — `.cataloger-overlay` appended to the shared overlay selector (no duplicated rules).
- i18n — `ribbon.cataloger`, `commands.cataloger`, `cataloger.title`, `cataloger.tagline` × 15 locales.
- **No IPC contract change, no schema change, no write-path change.** New `$effect` reads `showCataloger` / writes `catalogerEverOpened` (different vars) → no loop (Rule 2).
- **Right-sidebar Source Review tab untouched** (Concept Paper §10 / handover §5). Two SourceReviewPanel instances coexist safely (each fetches its own queue; both idempotently refresh on the shared `constellation:classify-and-show` window event).

### Deviation from handover §7 (flagged)
The handover assumed `ClassifierScanProgressStrip` carries "scan controls." It does not — it's a passive progress strip (shows progress + Cancel only while a scan runs). The whole-library scan is started via `classifier_scan_start` (today only from SettingsModal). So `CatalogerView` includes its own **"Scan library" button** mirroring `SettingsModal.startClassifierScan`, so the Boss can run a manual universe-wide scan from the dock view. Within the plan's intent (universe-wide home + scan controls).

### Accuracy guardrails (Concept Paper §5/§6, handover §6)
- The local-LLM "Reasoning" cataloger is DESIGNED-but-NOT-WIRED → CECE ships as a 5-cataloger heuristic ensemble. **No UI copy calls The Cataloger "AI" / "LLM".**
- Background auto-scan is not wired → scans are manual-only. **No UI copy implies automatic background classification.**

## Build progress
All code complete; NOT yet committed (per handover: stop at Boss-test verification; commit + orientation v2.19 bump happen "when it ships" = after Boss validates).

- **Step 1 — `src/lib/components/CatalogerView.svelte`** (new): full-page view = "The Cataloger" header + tagline + a self-contained "Scan library" button (`classifier_scan_start`, mirrors `SettingsModal.startClassifierScan`, syncs disabled-state off the `classifier:scan` event lifecycle) + `ClassifierScanProgressStrip` + library-wide `SourceReviewPanel` (no `activeNotePath`). Bounded flex-column layout so SRP's `height:100%` internal scroll works. `onNoteClick` + `onClose` props.
- **Step 2 — `store.ts`**: `+ cece: boolean` in `enabledFeatures` type + `+ cece: true` default. Distinct from the existing `appSettings.cece` engine-settings object. Settings-load spread back-fills existing users → ON.
- **Step 3 — `+layout.svelte`** (25 sites, mirrors `showOrgChart`): import, `showCataloger` state, `catalogerEverOpened` lazy-mount flag + effect, universe-switch reset (LL-022), `fullPageActive` term, `content-hidden` term, dock button (after Index, gated `enabledFeatures.cece !== false`, layers SVG icon), lazy-mounted `.cataloger-overlay`, command-palette entry, escape branch, shared `.cataloger-overlay` CSS selector, and `showCataloger = false` added to all 12 sibling close-lists (one `replace_all`) + the 3 dock buttons that don't close OrgChart (Sky View / Index / Map).
- **Step 4 — i18n (15 locales)** via `scripts/add-cataloger-i18n.mjs`: `ribbon.cataloger`, `commands.cataloger`, `cataloger.title`, `cataloger.tagline`. en="The Cataloger", ar="المُصنِّف", other 13 = classifier-sense per Concept Paper §10. Clean diffs (8 lines/file, no reformat).
- **Step 5 — verify**: `svelte-check` → **3 errors, all pre-existing** (store.ts `fresh` + 2× PropertyEditor node-type); **0 new errors**. Release build running.

### Settings → Plug-Ins card (added on Eisa's request, 2026-05-20)
- Added a `{ id: 'cece', ... }` card to the **Discovery** group in `SettingsModal.featureGroups` (right after Index). `id: 'cece'` → `toggleFeature` writes `enabledFeatures.cece`; `getFeatureEnabled` defaults it ON. Reuses the already-localized `cataloger.title` / `cataloger.tagline` keys (no new i18n). Toggling it OFF hides the dock button (gate `enabledFeatures?.cece !== false`). Requires a rebuild so the Boss-test binary includes the card.

## Boss test round 1 (2026-05-20) — Steps 1-4 PASS; Step 5 blocked by a memory leak; layout wrong
Eisa's findings:
1. Steps 1-4 (dock button, tooltip, open full-page, Esc/reopen, Settings card toggle) **PASS**.
2. **Severe memory leak / unresponsiveness** on Cataloger boot + operation (had to force-close). Eisa adds: *the same leak already existed in the right-sidebar CECE* — so it's a pre-existing `SourceReviewPanel` bug, not the new wiring.
3. **Layout wrong**: the view rendered as a narrow centered column ("a window within a window") instead of using the full dock width.

### Root cause (diagnosed, not guessed)
`SourceReviewPanel` renders the **entire** pending queue with **no virtualization / no cap** (`{#each filteredQueue}` over all ~4,475 rows). Each card runs `parseComposite` (a `JSON.parse`) ≥2× (template + `cardNeedsUserCall`), and `filterCounts` + `splitAwareSkipCount` + `filteredQueue` each iterate the **full** queue parsing every blob. Opening the dock view (which loads the whole universe queue) → tens of thousands of DOM nodes + ~18k JSON parses in one synchronous pass → freeze. During a scan, `scheduleQueueReload` reloads every 1.5s on a growing queue → progressive slowdown. This is the CLAUDE.md **Rule 3** violation ("virtualize every list that can exceed 50 items"). It lived in the shared panel, hence the right-sidebar leak too.

### Fixes (in the shared `SourceReviewPanel` — cures both surfaces)
- **Memoize `parseComposite`** — component-local `Map` keyed by the immutable `composite_json` string; each blob parsed once, reused across counts/skip/filter/render and across scan reloads.
- **Cap the rendered cards** — `RENDER_BATCH = 80`; `visibleQueue = filteredQueue.slice(0, renderCap)`; render `visibleQueue`; **"Show more"** footer (+80) with a localized "Showing N of M" note; reset to one batch on filter change. Filter counts + Approve All still operate on the full queue — only the DOM is bounded.
- **i18n** — `cece.queueShowMore` + `cece.queueShowingCount` added to all 15 locales (via the script).
- **Layout (`CatalogerView`)** — removed the 880px centered column; the view now fills the full dock width.

`svelte-check`: still 3 pre-existing errors, 0 new. Rebuilding for Boss test round 2.

## Boss test round 2 (2026-05-20) — ALL 5 STEPS PASS
Eisa: Steps 1-5 all PASS; *"you nailed the memory leak."* Leak gone, full-width layout correct, scan runs responsively, approve writes frontmatter, right-sidebar Source Review unchanged.

### New finding (round 2): "Hide reasoning" chevron does nothing
- **Root cause (pre-existing, not introduced by MIG-039):** the trail toggle only tracked an `expandedTrails` ("force-open") set. Reasoning trails are **open by default** for a card when either (a) the user set `reasoningTrailVisibility: 'always'` in Settings → Intelligence → CECE, or (b) trust-cal is active (<50 reviews). On a default-open card, clicking the chevron added the path to the *expanded* set (a no-op — already open) with no way to force it *closed*. **Eisa correctly identified the 'always' Settings choice as the likely cause** (I had wrongly assumed trust-cal without checking — corrected).
- **Fix (`SourceReviewPanel`):** added a `collapsedTrails` ("force-closed") set; `toggleTrail(notePath, currentlyOpen)` now flips based on the card's current visible state; `isTrailOpen` checks `collapsedTrails` **first**, so an explicit per-card click overrides any global default-open ('always' or trust-cal). Correct for all 3 prefs + trust-cal. `svelte-check`: 3 pre-existing, 0 new. Rebuilding for a quick chevron re-verify.

## Chevron re-verify (round 3) — PASS
Eisa: "All pass." Collapse/expand works per-card with "Always show" set; cards collapse independently. Eisa correctly noted the open-by-default came from his `reasoningTrailVisibility: 'always'` Settings choice (I'd wrongly assumed trust-cal — corrected); the fix covers both because the explicit-collapse check runs first.

## Eisa direction (2026-05-20, end of session)
1. **Question answered — disambiguation bug.** Card split on BOTH axes: picking the Source chip (`cece_resolve_disambiguation`) writes the source, clears the whole suggestion, and co-writes the other axis only if it was *settled* (`extract_other_axis_settled` → None for a split axis). So **Content type is left unclassified + the card vanishes** before the user picks it. Real bug. Fix is cross-subsystem (backend keeps the suggestion until both split axes resolve + frontend keeps the card). **Deferred to the next increment.**
2. **NSC chartered.** Eisa wants a **"Note Summary Creator (NSC)"** subsystem (NOT a plain excerpt): use the note's own summary if present, else **summarize the whole note**, **language-agnostic**. Framed as a Constellation differentiator. Design next (Architect pass — local-first; e5-small embeddings enable extractive summarization without the unwired LLM).
3. **Sequencing**: **secure the Cataloger now** (this commit), then build NSC + the disambiguation fix.

## Commit (secure the Cataloger) — 2026-05-20
ONE commit: `CatalogerView.svelte` (new) + `+layout.svelte` (dock wiring) + `store.ts` (`enabledFeatures.cece`) + `SettingsModal.svelte` (Plug-Ins card) + `SourceReviewPanel.svelte` (leak fix: memoize + render cap; chevron fix) + 15 i18n locales + `scripts/add-cataloger-i18n.mjs` + orientation **v2.19** (new file) + this session log.

## Open / next (after this commit)
- **NSC** (Note Summary Creator) — Architect/Concept pass, then build. Differentiator.
- **Disambiguation both-axes-split bug** — backend (`cece_resolve_disambiguation`) + frontend fix; allocate a PJ-NNN.
- **Help-doc + User Manual sync** for the Cataloger + NSC — deferred to land WITH the NSC increment (avoid doing the 15-language pass twice, since NSC will change the Cataloger card help).
- Orientation **body** (§4.x CECE section + left-dock feature list) still describes the pre-Cataloger state — update with the NSC increment (noted in the v2.19 preamble).
