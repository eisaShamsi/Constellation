# MIG-039 — "The Cataloger" (CECE left-dock Core Plug-in) — Session Handover

**Written 2026-05-19 for a fresh session. Self-contained — assumes no memory of the prior conversation.**

---

## 0. Read-first + the stale-orientation warning

Before anything else:

1. Read `CLAUDE.md` (project rules — top-principal rules apply).
2. Read this whole handover.
3. Read `docs/Constellation-CECE-Concept-Paper-v1.0.md` (what CECE is + the naming rationale).
4. **⚠️ The orientation doc is STALE.** The highest version is `docs/Constellation Orientation & Onboarding v2.17.md`, which **predates all of 2026-05-19's work**. Do NOT trust it for current state. The accurate end-of-2026-05-19 state is in `lab/reports/SESSION-LOG-2026-05-19.md`. **A v2.18 orientation bump is outstanding SO #6 debt** (see §6) — strongly consider doing it before/with this build.

## 1. The goal (one line)

Build **"The Cataloger"** as a first-class **left-dock feature** in Constellation — its own dock button + full-page view — promoting the CECE classifier from a right-sidebar tab to a main dock surface, the same way Sky View / CNS / Index sit on the left dock.

## 2. Context: what is this + why

- **CECE** (Constellation Epistemic Content Engine) classifies each note on two axes: *what kind of knowledge* (content-type, 5 branches) and *where it came from* (source, 11 sources). Full detail: `docs/Constellation-CECE-Concept-Paper-v1.0.md`.
- Eisa's plugin taxonomy (2026-05-19): a **"Core Plug-in" = a feature on the main left dock that stays in the app** (Sky View, CNS, Index, and now CECE). This is the OPPOSITE of an **"External Plugin"** (Sight + Map, which are being detached to the deferred "Constellation Wings" sub-project — see `docs/Constellation Wings/Charter v0.1.md`).
- So "activate CECE as a core plug-in" = **give it a left-dock button + main view.** That is this MIG.

## 3. The name (DECIDED — do not re-litigate)

- **User-facing name: "The Cataloger"** (or "Cataloger"). Decided by Eisa 2026-05-19.
- **Internal engine name stays "CECE"** — Rust modules `cece/`, IPCs `classifier_*` / `sources_*`, code comments all keep "CECE" (same pattern as Sky View / Sight keeping internal names). Do NOT rename the engine.
- **Arabic: المُصنِّف** (*al-muṣannif*, "the classifier") — Eisa chose the *classifier* sense, not the cataloger/indexer sense.
- The other 13 locales follow the **classifier sense** (see Concept Paper §10 for per-locale candidates) — confirm final values during the i18n step.

## 4. What's already done (committed on `main`, do NOT redo)

| Commit | What |
|---|---|
| `feec7b12` | CECE name decided ("The Cataloger") in Concept Paper §10 |
| `9ab0d193` | CECE Concept Paper v1.0 written |
| `57cd7638` | MIG-038: Sight + Map disabled; Constellation Wings chartered (deferred) |
| `26fe4f43` | Version aligned to 0.1.0 (Constellation is v0.1) |
| `bb221fe4` | MIG-037 P1: Time Dome (Sight v6.3, now dormant since Sight is disabled) |

## 5. What NOT to touch

- **The right-sidebar Source Review tab** — stays as-is until the new dock view is finished (Eisa's instruction). It's the per-note review surface; the dock view is the universe-wide home. Don't remove or move it in this MIG.
- **The CECE engine internal name** — stays "CECE."
- **Sight + Map** — disabled (`SIGHT_V6_ENABLED=false` in `src/lib/sight/engine.ts`; `constellationMap` force-off in `store.ts loadSettings`). Leave them; they go to Wings later.

## 6. Accuracy caveats (CRITICAL — do not misrepresent in UI/help copy)

- **The local-LLM "Reasoning" cataloger is DESIGNED BUT NOT WIRED.** It abstains on every note. CECE currently ships as a **5-cataloger heuristic ensemble** (User-Authority, Structural, Linguistic, Graph, Semantic). **Do NOT label The Cataloger as "AI classification" or "local LLM"** in any user-facing string. It is a heuristic classifier today.
- **Background auto-scan is NOT wired.** Scans are **manual-only** (`classifier_scan_start` / per-note `classifier_suggest_for_note`). Don't write UI copy implying automatic background classification.

## 7. The build plan (the MIG)

The pattern to mirror is **OrgChart** (a panel-style full-page dock feature). Study its wiring in `src/routes/+layout.svelte` first by grepping `showOrgChart` — it touches ~18 sites you'll replicate for `showCataloger`.

### Components (reuse — don't rebuild)
- `src/lib/components/SourceReviewPanel.svelte` — the suggestion queue (self-contained; fetches via its own IPCs).
- `src/lib/components/ClassifierScanProgressStrip.svelte` — scan progress + controls.
- `src/lib/components/ProvenancePanel.svelte` — optional, provenance display.

### Step 1 — New `src/lib/components/CatalogerView.svelte`
A full-page view component: a header titled "The Cataloger" + the `ClassifierScanProgressStrip` (scan controls) + the `SourceReviewPanel` (the queue), laid out in a content-area max-width container. Keep it self-contained; pass through any `onOpenNote` callback the panel needs.

### Step 2 — `src/lib/libraries/store.ts`
- Add `cece: boolean` to the `enabledFeatures` type (near line ~3404, alongside `constellationSightV6`).
- Add `cece: true` to `DEFAULT_SETTINGS.enabledFeatures` (near line ~3698) — default ON (it's a core feature).
- (Do NOT add a force-off — that was only for Map's disable.)

### Step 3 — `src/routes/+layout.svelte` (the ~18 sites; mirror `showOrgChart`)
- **State**: `let showCataloger = $state(false);` (near the other `show*` declarations, ~line 388–461).
- **`fullPageActive` derived** (~line 1064): add `|| showCataloger`.
- **Dock button**: insert after the Index dock button (~line 4427), gated `{#if $appSettings.enabledFeatures?.cece !== false}`. On click: `showCataloger = !showCataloger;` + set the other full-page flags false (copy the close-others list from a sibling button). Title `{$t('ribbon.cataloger')}`. Pick a classifier/sort icon (e.g., a funnel or layered-squares SVG).
- **Overlay mount**: mirror the `.orgchart-overlay` block (~line 5124). Add `<div class="cataloger-overlay" class:cataloger-visible={showCataloger}>` wrapping `<CatalogerView ... />`. Add matching CSS (copy `.orgchart-overlay` / `.orgchart-visible` rules).
- **Close-others**: add `showCataloger = false;` to the shared reset block (~line 3633), the escape-key handler (~line 2955: `if (showCataloger) { showCataloger = false; return; }`), the command-palette nav entries, and the other dock buttons' close-others lists. Grep every `showOrgChart = false` site and add `showCataloger = false` beside it.
- **Command palette**: add a `{ id: 'cataloger', name: $t('commands.cataloger'), icon: '🗂️', action: () => { showCataloger = !showCataloger; /* close others */ }, category: 'View' }` entry (~line 1747 area).
- Optional: `everOpened` `$effect` (~line 600) if lazy-mount is wanted.

### Step 4 — i18n (all 15 locales — full-localization standing order)
- `ribbon.cataloger` (dock tooltip) + `commands.cataloger` (palette) + any CatalogerView header strings.
- **en** = "The Cataloger". **ar** = "المُصنِّف". Other 13 = classifier sense (Concept Paper §10 lists candidates). Use a script (see `scripts/add-time-dome-i18n.mjs` as a template) to apply all 15 in one pass.

### Step 5 — Verify
- `npx svelte-check --tsconfig ./tsconfig.json --threshold error` — confirm no NEW errors (3 pre-existing are OK: store.ts `fresh` + 2 PropertyEditor node-type).
- `npm run tauri build` (background) — the signing step fails with exit 1 (no `TAURI_SIGNING_PRIVATE_KEY`) but the NSIS .exe is still produced at `src-tauri/target/release/bundle/nsis/Constellation_0.1.0_x64-setup.exe`. Copy it with a `MIG039-cataloger` suffix per Eisa convention.
- Boss test: a TUTORIAL (per the Testing Instructions Rule — define the feature, then click-by-click). Open The Cataloger from the left dock → see the queue + scan controls → run a manual scan → approve a suggestion → confirm it writes to the note's frontmatter. Confirm the right-sidebar Source Review tab still works unchanged.

## 8. Standing orders that apply

- **Full localization** (top-principal): all 15 locales, right native terms.
- **Testing Instructions Rule**: the Boss test must read as a tutorial.
- **Plan Approval = Build Approval**: Eisa approved this MIG-039 plan + the name; cascade the build, stop only at the Boss-test verification.
- **SO #1**: log progress to `lab/reports/SESSION-LOG-2026-05-20.md` (new day) as you go.
- **SO #6 (OUTSTANDING DEBT)**: orientation is at v2.17, stale since before 2026-05-19. A **v2.18 bump** is owed, capturing: MIG-037 P1 (Time Dome), MIG-038 (Sight+Map disabled + Wings charter), version 0.1.0, CECE Concept Paper + "The Cataloger" naming, and (when done) MIG-039. Address this early.

## 9. Suggested kickoff prompt (what Eisa pastes into the new session)

> Pick up **MIG-039 — build "The Cataloger"** (the CECE left-dock Core Plug-in). Read `lab/reports/MIG-039-CATALOGER-HANDOVER.md` for the full self-contained plan, then cascade the build per its §7, stopping at the Boss-test verification. Also handle the outstanding orientation v2.18 bump (§8) capturing 2026-05-19's work.

---

*Handover prepared 2026-05-19. All referenced commits are on `main`; the tree is clean. Nothing is half-done.*
