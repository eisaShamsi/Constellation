# Session Log — 2026-04-29

---

## §92 — MIG-003 main integration (fast-forward) + state-of-standing

**Operation**: `git merge --ff-only claude/frosty-stonebraker-75c9bf` from `main`, then `git push origin main`. No new commit on main; the seven MIG-003 commits from `claude/frosty-stonebraker-75c9bf` (`4f8e3c9 → 8cb80ac`) are now reachable from `origin/main`.

**Pre-state discovery**: yesterday's `SESSION-LOG-2026-04-28.md` "Handover to next session" snapshot claimed MIG-003 was closed and the working tree was clean. Investigation showed the actual state was different:

- `main` was at `6545b3e` (§91/§92 — MIG-008 + MIG-009 closed). MIG-003 commits had **never been merged** to `main`.
- Three closure documents lived in `main`'s working tree as **uncommitted/untracked** changes that were byte-identical to the committed versions on `claude/frosty-stonebraker-75c9bf`:
  - `M docs/CANONICAL-FILENAME-ARCHITECTURE.md` (+54 lines)
  - `M lab/reports/SESSION-LOG-2026-04-28.md` (+111 lines — the §85–§89 + handover sections)
  - `?? docs/Constellation Orientation & Onboarding v1.7.md` (entire file untracked)
- Tag `milestone/mig-003-closed` resolved to `25c65ef` (closure commit) but only on the side branch — reachable from `origin/claude/frosty-stonebraker-75c9bf`, **not** from `origin/main`.
- `git merge-base main claude/frosty-stonebraker-75c9bf = main's tip` — meaning `frosty` was strictly ahead of `main` by the seven MIG-003 commits, no divergent main-side work to merge.
- Numbering note: both lines independently re-used `§85–§91` after the §123→§85 reset of 2026-04-23. The frosty line is `§85=4f8e3c9 (MIG-003 Step 1) → §91=8cb80ac (MIG-003 handover)`. The main line was `§85=3f41efb (Orientation v1.0) → §91/§92=6545b3e (MIG-008/MIG-009)`. Linearized history after the FF interleaves both — readable but the §-sequence has duplicates.

**Cascade executed**:

1. `git stash push --include-untracked -m "pre-MIG-003-FF: stranded closure docs (byte-identical to frosty tip)"` — stash byte-identical doc changes so git would allow the FF to overwrite cleanly.
2. `git merge --ff-only claude/frosty-stonebraker-75c9bf` — `Updating 6545b3e..8cb80ac. Fast-forward.` Diff stat: 10 files, +3,080 / −153 lines. Brought in `lab/reports/MIG-003-HUMAN-NAME-FILENAMES.md` (+307), `src-tauri/src/mig003_step4.rs` (+378 new module), `src-tauri/src/libraries.rs` (+514 net), `src-tauri/src/search.rs` (+757 net), the three doc files, plus small touches in `embeddings.rs`, `lib.rs`, `sky_backfill.rs`.
3. `git stash drop stash@{0}` — incoming content was byte-identical to stashed; stash redundant. Pre-existing stashes left alone.
4. `git push origin main` — `6545b3e..8cb80ac main -> main` to remote. Tag `milestone/mig-003-closed` was already on origin (pushed with the side branch yesterday); now reachable from `origin/main` as well.

**Verification**:
- Working tree clean: `git status --short` empty.
- v1.7 orientation now tracked under git.
- `git rev-parse main origin/main HEAD` all = `8cb80ac`.
- `git merge-base --is-ancestor milestone/mig-003-closed main` returns true.

**Binary-mtime parity restored**: the `target/release/constellation.exe` at `main`'s checkout was 6 days old (Apr 23 19:57, pre-MIG-005). The post-MIG-003 build (Apr 28 18:19, 54.5 MB) lived only at `frosty-stonebraker-75c9bf/src-tauri/target/release/`. Per `feedback_verify_binary_before_testing.md` the source ↔ binary contract was broken at `main`. Cascade:
- Copied `frosty/.../target/release/constellation.exe` → `main/.../target/release/constellation.exe` (cp -p preserves mtime).
- Copied `frosty/.../target/release/bundle/msi/Constellation_0.3.4_x64_en-US.msi` → main equivalent.
- Copied `frosty/.../target/release/bundle/nsis/Constellation_0.3.4_x64-setup.exe` → main equivalent.
- Frosty `target/` retained as build cache (Boss directive, Option 1+3 hybrid). Frosty source tree is now tip-equivalent to main but the worktree stays around so its incremental build state isn't lost.

**Stale leftover identified, not removed**: `E:/مشاريع كلاود/Constellation/.claude/worktrees/upbeat-proskuriakova/` is a directory with a single 1020-byte `.claude/settings.local.json` from Apr 20 — not a registered worktree. Remote branch `origin/claude/upbeat-proskuriakova` exists but is strictly behind main (M11-data v2 lexicon work, session-log §22–§26 era, all already merged to main long ago). Cleanup deferred pending Boss authorization (the housekeeping bundle of `frosty` + `upbeat` deletions is scoped but not run).

**Backlog after this PCS**:

- ✅ MIG-003 fully on main (closed yesterday on the side branch; integrated to main today).
- ✅ Tag `milestone/mig-003-closed` reachable from `origin/main`.
- ✅ Source ↔ binary contract restored at `main` checkout.
- ⏳ CE Phase 12 360° Inspector — Boss approved re-enable plan (Option C: compact sidebar + full-window). Cascading now (§93).
- ⏳ MIG-006 §3 redo — queued after Phase 12.
- ⏳ CE Phase 9 Multi-Lens Path B (Rule-8 compliant, MIG-010 scale) — queued after MIG-006 §3.
- ⏳ Frosty + upbeat worktree/branch cleanup — pending Boss authorization.
- ⏳ MIG-005 Steps 4–8.
- ⏳ MIG-002 Steps 7–10.
- ⏳ User Manual + 14 i18n updates for MIG-003 — small surface area, deferred.
- ⏳ `docs/IPC-CONTRACT.md` ~5 weeks stale — missing `constellation_sight_*` rename, MIG-003 cascade hooks, `cache_boot_snapshot_sky`.
- ⏳ Rename-collision popup UX (wanted feature record exists).

---

## §93 — CE Phase 12 re-enable: Inspector 360 (compact + full-window)

**Boss decision (2026-04-29)**: re-enable CE Phase 12 with Mount-Surface Option C — compact widget in right-sidebar tab AND full-window overlay via dock button. Sequencing approved as Option 2 (Phase 12 first → MIG-006 §3 redo → CE Phase 9 Path B).

**What landed** (all in `src/routes/+layout.svelte` and `src/lib/libraries/store.ts`, no Rust changes):

- **Import re-enabled** at `+layout.svelte:84` (`import Inspector360 from '$lib/components/Inspector360.svelte';` — was commented out as "CE Phase 12: disabled — revisit later").
- **State**: `showInspector360`, `inspector360EverOpened` (LL-022 lazy-mount sticky flag), `inspector360Data: Note360View | null`, `inspector360Loading`, `inspector360FetchTimer`, `inspector360RequestSeq` (stale-result guard).
- **Lazy-mount $effect**: `if (showInspector360) inspector360EverOpened = true;` — same pattern as `mapEverOpened` / `orgChartEverOpened`.
- **IPC fetch $effect** (the load-bearing one): runs whenever the compact tab is visible OR the full-window overlay is up AND a note is in focus (`sidebarTab.path` + `sidebarTab.libraryPath`). Debounced 200 ms; sequence number prevents stale-result writes when rapid navigation produces overlapping fetches; cleanup function clears the timer on $effect re-run. Calls `invoke('get_360_view', {libraryPath, notePath})` → populates `inspector360Data`.
- **fullPageActive derived** at `+layout.svelte:980` now OR's in `showInspector360`.
- **Dock button**: new ribbon button after the Constellation Sight (Lens) button, gated by `$appSettings.enabledFeatures?.inspector360 !== false` (default `true`). Reticle-style SVG (concentric circles + 4 spoke lines). Closes all other full-page modes on toggle-on.
- **Full-window overlay mount**: lazy-mounted via `{#if inspector360EverOpened}`, visibility via `class:inspector360-visible={showInspector360}`. Mirrors the `map-overlay` pattern. `<Inspector360 compact={false} data={inspector360Data} onNoteClick={...} onClose={() => showInspector360 = false} />`. The component already has 3 sub-viz modes (Atmospheric / Neural / Cosmic) selectable from its own header.
- **Right-sidebar compact tab**: new `inspector360` tab button after `links` in the rs-tabs strip, gated by `$appSettings.panelPlacements?.inspector360 ?? 'right-sidebar'`. Same reticle SVG at 12×12. Renders `<Inspector360 compact={true} data={inspector360Data} onNoteClick={...} />`.
- **Mutual-exclusivity**: `showInspector360 = false` added to all 12+ existing sites that close other full-page modes (sky-view dock, index dock, map dock, lens dock, search-hub dock, orgchart dock, knowledge-health dock, search command palette, global-tasks command palette, index command palette, expression-forge command palette, constellation-map command palette, sense-making-canvas command palette, search-hub return button, `handleTagClick`).
- **Universe switch reset**: `inspector360EverOpened = false; inspector360Data = null;` added to the `mapEverOpened = false; orgChartEverOpened = false;` reset block, alongside the LL-022 sticky-flag reset.
- **content-area class:content-hidden** now includes `showInspector360`, matching map / orgchart / lens / search-hub.
- **CSS**: `.inspector360-overlay` and `.inspector360-overlay.inspector360-visible` extend the existing `.index-overlay, .map-overlay, .orgchart-overlay` rules (one selector add each).
- **Type extensions** in `src/lib/libraries/store.ts`:
  - `PanelId` union gains `'inspector360'`.
  - `AppSettings.enabledFeatures` interface gains `inspector360: boolean`.
  - `DEFAULT_SETTINGS.enabledFeatures` gains `inspector360: true`.
  - `DEFAULT_SETTINGS.panelPlacements` gains `inspector360: 'right-sidebar'`.
  - `+layout.svelte:331` `rightSidebarTab` $state union extended with `'inspector360'`.

**i18n status**: all 15 locales already had `panels.inspector360` (line 712 in en.json) and `inspector360.*` namespace (line 1142 with title/noData/gaps/blindSpots/orphan/fragile/dimensions/context/depth/dueForReview/outbound/inbound/words/mode_atmospheric/mode_neural/mode_cosmic/link_supports/link_contradicts/link_causes/link_derives_from/link_generalizes/link_exemplifies/link_part_of). `ribbon.inspector360` does NOT exist anywhere (the dock button falls back to `inspector360.title`). Adding `ribbon.inspector360` to all 15 locales deferred to §94 (docs commit).

**Build verification**: `npm run check` ran clean of new errors after type extensions. The 1 remaining error in store.ts:1850 (LinkLifecycle missing 'fresh' property) pre-dates this cascade. 284 warnings (all pre-existing CSS-unused-selector / empty-ruleset).

**Perf gate — DEFERRED to user testing.** Reading `inspector360.rs::get_360_view` confirmed the orientation v1.7 §4.2 row 12 description: it walks the full library on every call (`scan_all_notes` recursive fs walk + parse), then runs O(n²) inbound link iteration over the in-memory map, then hits 4 helper functions (`get_review_for_note`, `get_trails_for_note`, `compute_provenance_for_note`, `compute_stratum_for_note`, `compute_maturity_for_note`). On a 7,600-note universe this is expected to be 1–3 s per call on a modern SSD. Mitigations baked in: debounced fetch (200 ms), sequence guard (no stale writes), lazy-mount (no cost until first open), inspector360Data persists between opens (subsequent opens show stale data while refetch runs in background — no flash to empty state). **What's not in scope here**: bringing the IPC into Rule-8 compliance (cached `note_360_view` table updated on triggers). That's MIG-010-scale work; will be re-considered after Boss tests perf.

**Loading-state UX**: not added in this commit. The component shows the "Open a note to see its 360° view" empty state when `data === null`. After `inspector360Data` is set once, subsequent fetches don't blank it out (sequence-guarded write); user sees stale data while next fetch resolves. First-open shows empty state for ~1–3 s. If Boss reports this as confusing during testing, a simple `<div class="inspector360-loading">…</div>` overlay can be added in a follow-up.

**Files changed**:
- `src/routes/+layout.svelte` (~22 distinct edits across state, $effects, derived, dock, overlays, sidebar tab, mutual-exclusivity, CSS)
- `src/lib/libraries/store.ts` (4 edits: PanelId union, enabledFeatures interface, enabledFeatures default, panelPlacements default)
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry)

**Pending after §93**:
- §94: /simplify pass on the §93 diff.
- §95: orientation v1.8 + help files + User Manual + `ribbon.inspector360` to all 15 locales.
- Boss tutorial test (separate from §95 since the test is the verification gate).
- Perf gate result feeds back into MIG-006 §3 redo prioritization.

---

## §94 — CE Phase 12 /simplify pass

Three reviewers (reuse / quality / efficiency) ran in parallel against the §93 diff. Findings + fixes:

**Reuse** — clean. No duplications, the new patterns (debounce + sequence guard, mutual-exclusivity inline resets, right-sidebar tab gating, panel-placement check) match conventions already in the file. No reusable helpers exist for the IPC fetch or the close-other-modes pattern; introducing them would be premature abstraction.

**Quality — must-fix #1: dead `inspector360Loading`.** Declared on line 390, written `true`/`false` in three places inside the IPC `$effect`, never read. No UI surface consumed it (Inspector360.svelte has no `loading` prop). Deleted the declaration and the three writes. Component shows its own "Open a note to see its 360° view" empty state during the first fetch — that's the Stage-1 UX. If first-load latency proves jarring during Boss testing, a dedicated loading prop on the component is a follow-up.

**Quality — nice-to-have: trim block comment.** The 6-line comment before the IPC `$effect` partly enumerated WHAT (when it runs — re-narrating the boolean guard) instead of just WHY. Trimmed to 3-line WHY-only ("Debounce 200 ms; sequence number discards stale results; last-key guard skips re-fetching same note").

**Quality — follow-up logged for §95+**: 12 mutual-exclusivity sites duplicate a "close other full-page modes" pattern. Lists differ slightly between sites, so it's not pure copy-paste, but extracting `closeAllFullPageExcept(keep)` is the right move before the next full-page surface lands. Tracked as a backlog item; not a §94 blocker.

**Efficiency — critical: $effect re-fired on every keystroke.** The original $effect read `sidebarTab?.path` directly. `sidebarTab = $derived($focusedTab)` and `$focusedTab` derives from `openTabs`, which `updateTabContent` (`store.ts:578`) replaces with `{ ...t, content: newContent }` on every save. So `sidebarTab` got a new identity each typing-debounce tick → the $effect re-ran → cleared and re-scheduled the 200 ms timer. The fetch was starved while typing (the timer kept resetting). **Fix**: read paths through `$derived` string values (`inspector360Path`, `inspector360LibPath`). Strings compare by value in Svelte 5's equality check, so identity-change of the parent `sidebarTab` no longer publishes to the $effect when the underlying strings haven't changed. **Required relocating both new `$derived` lines and the IPC $effect from line ~580 to immediately after `const sidebarTab = $derived($focusedTab)` at line ~1075** to satisfy TypeScript's TDZ check (the helpers can't reference `sidebarTab` before it is declared).

**Efficiency — high: no last-fetched-key dedup.** Clicking tab A → tab B → back to tab A re-fired `get_360_view` for A even though `inspector360Data` already held A's view. **Fix**: added `lastFetchedInspectorKey: string | null` (plain `let`, not `$state` — no reactive consumer needed). Compose `key = ${libPath}::${path}`. Skip fetch when `key === lastFetchedInspectorKey && inspector360Data` is truthy. Set `lastFetchedInspectorKey` only on successful fetch.

**Efficiency — medium: unnecessary null-write.** When `!shouldFetch`, the original code unconditionally wrote `inspector360Data = null`. On a layout-level reactive tick this could fire repeatedly even when `inspector360Data` was already null. Svelte 5 might still schedule downstream effect re-checks. **Fix**: guarded — only write `null` when the value is actually about to change.

**Efficiency — confirmed safe**: cleanup function correctly cancels in-flight timers on $effect re-run; sequence guard discards stale invoke results when the timer already fired before cleanup; `fullPageActive` adding one OR is negligible; the 12 mutual-exclusivity additions are each one synchronous boolean write — negligible.

**Build verification**: `npm run check` clean of new errors. The 1 remaining error (store.ts:1850 LinkLifecycle missing `'fresh'`) pre-dates this cascade.

**Files changed**:
- `src/routes/+layout.svelte` — IPC $effect relocated + rewritten with $derived key + last-fetched guard + null-write guard. `inspector360Loading` deleted. Block comment trimmed.
- `lab/reports/SESSION-LOG-2026-04-29.md` — this entry.

No store.ts changes.

---

## §95 — CE Phase 12 docs: orientation v1.8 + help + User Manual

Per Standing Order #2 (update help files + User Manual on user-facing changes) and Standing Order #6 (maintain orientation, write a NEW versioned file alongside existing ones).

**Orientation `Constellation Orientation & Onboarding v1.8.md`** written as a new file alongside v1.0..v1.7. Targeted edits from v1.7:

- Header bumped to **Version 1.8 | 2026-04-29**.
- Top-of-file "What changed in v1.8" note covers (1) MIG-003 fast-forward integration to main + binary-mtime parity restoration, (2) CE Phase 12 360° Inspector re-enabled with §93 wiring + §94 /simplify pass, (3) CE Phase 9 Multi-Lens approved for re-wire on Path B (queued after MIG-006 §3 redo).
- v1.7 changelog moved into a new `### v1.7 changelog (vs v1.6)` subsection at the head of the changelog list, including a clear note that v1.7's "MIG-003 closed" claim was correct only on the side branch — the main-line integration arrived in v1.8.
- §4.2 row 12 (Inspector 360) updated to mark **✅ enabled v1.8 §93** with the explicit Rule-8 violation acknowledged (1–3 s read-time aggregation per fetch, MIG-010 candidate).
- §17 unknowns gained two new entries: actual `get_360_view` latency on the 7,600-note Universe (not measured, awaiting Boss tutorial test) and whether the first-fetch empty-state UX feels jarring during the 1–3 s wait.
- File-end signature corrected from `End of v1.6` to `End of v1.8`.

**Help file `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md`** — new "Feature 12: 360° Inspector" section inserted before the existing "Coming Soon (Layer 2)" section. Covers what it is (synthesis surface for all CE features per-note), why it matters (gap detector via dimmed/dashed sectors for unused link types), how to use it (compact sidebar tab + full-window dock button with three viz modes), where the data comes from (each panel maps to a CE feature 1–9), what is being measured (1–3 s first-fetch on large libraries; cached after), and what the empty state means.

**User Manual `docs/User Manual.md`** — section 18.10 "360° Inspector" added (numbering follows the existing 18.x scheme inside chapter 21). Brief description with the same structural sections as other CE features (What it is / Why it matters / How to use it / Where you see it / Tips). Layer 1 feature count updated from "Nine tools" / "All nine" → "Ten tools" / "All ten" at lines 1193 and 1197.

**Doc-debts logged for follow-up** (intentionally NOT addressed in §95):

- 14 translated User Manuals (`docs/help.{ar,de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}/`) — Inspector 360 entry not added to non-English locales. Same deferral pattern as MIG-003's 14-locale update; the Boss's normal i18n flow will pick this up.
- `ribbon.inspector360` not added to any of the 15 locale JSONs. The dock button's title falls back to `inspector360.title` (which exists in all 15 locales as "360.3D" or its localised equivalent), so this is not a user-visible regression — but adding the explicit ribbon-namespaced key is the right convention. Trivial when batched with the User Manual i18n round.
- The 12-site mutual-exclusivity inline-reset pattern (close-other-modes) — extract `closeAllFullPageExcept(keep)` helper before the next full-page surface lands. Logged in §94 entry; not blocking.
- Multi-Lens Views section (User Manual §18.9) currently describes Multi-Lens as if it works. It does not — `apply_lens` is dead code today. Correction deferred until CE Phase 9 Path B (MIG-010) ships; at that point §18.9 gets rewritten to reflect the real Rule-8-compliant behaviour.

**Files changed**:
- `docs/Constellation Orientation & Onboarding v1.8.md` (new file).
- `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` (Inspector 360 section added).
- `docs/User Manual.md` (§18.10 added + Layer 1 feature count updated from nine to ten).
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry).

---

## §96 — Stage 1 hotfix: missing inspector360 entry in tabVisible safety map

**Boss-reported during Stage 1.6 testing (2026-04-29 ~13:00)**: clicking the new 360° Inspector tab icon in the right sidebar tab strip routed the user to the **Properties** panel instead of the Inspector 360 panel. The new icon was rendered correctly; the click handler set `rightSidebarTab = 'inspector360'`; but immediately after, a safety $effect at `+layout.svelte:1255–1276` reset `rightSidebarTab` back to `'properties'`.

**Root cause**: that $effect maintains a `tabVisible: Record<string, boolean>` map listing every known sidebar tab and a corresponding `order` array used to find a fallback when the active tab is no longer visible. The map covered `properties / backlinks / tags / star / tasks / calendar / health / provenance / review / links` — **`inspector360` was not in either structure**. So when the user clicked the new tab, the $effect read `!tabVisible['inspector360']` (undefined → falsy → truthy negation), concluded the tab was invisible, and force-reset to the first visible tab. That first tab was `properties`, hence the symptom.

**Fix**: added `inspector360: inSidebar('inspector360')` to the `tabVisible` map (uses the existing helper that reads `panelPlacements?.inspector360 ?? 'right-sidebar'` — the default I'd already wired in store.ts) and appended `'inspector360'` to the `order` array.

**Working Agreement #4 self-audit**: this is the exact miss WA#4 is supposed to prevent. Before shipping §93 I walked the call graph for `showInspector360` (dock buttons, command palette, mutual-exclusivity sites, full-window overlay, content-area class) but did NOT walk the call graph for `rightSidebarTab`. The safety $effect at line 1255 is a write site for `rightSidebarTab` and should have been on the audit list. Reaffirming the rule from `feedback_walk_through_writes.md`: before any reactive-state change, enumerate every read AND every write of the affected variable.

**Build verification**: `npm run check` clean of new errors. The 1 remaining error in store.ts:1850 (LinkLifecycle missing 'fresh') is pre-existing.

**Files changed**:
- `src/routes/+layout.svelte` — added `inspector360` to `tabVisible` map (line 1269) and to `order` array (line 1273).
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry).

**Stage 1 retest gate**: Boss to retry the Stage 1 walkthrough after the §96 binary builds. If the click now routes to the Inspector 360 widget with the spherical visualisation, Stage 1 ✅. Stage 2 (full-window mode) follows.

---

## §96b — Stage 1 verified by Boss + two findings

Boss reran Stage 1 against the §96 binary (built 2026-04-29 13:36). All five checks PASS:

- Reticle icon appears in the right sidebar — ✅
- Clicking it shows the spherical visualisation — ✅
- First-open latency: **almost instantly** (much faster than the 1–3 s estimate I'd anchored from reading `inspector360.rs`)
- Navigating between notes updates the visualisation — ✅
- Clicking an orbiting dot navigates to that note — ✅

**Perf finding** (significant): the actual `get_360_view` cost on the 7,600-note Universe is far below my theoretical 1–3 s estimate. Possible explanations: (a) OS file cache warm from prior navigation, (b) the inbound-iteration O(n²) wasn't the bottleneck on this dataset, (c) my estimate was simply pessimistic. Whatever the cause, **MIG-010 (Rule-8-compliant cached `note_360_view` table) priority drops to LOW** based on Boss's lived experience. Keep the option in the backlog but stop treating it as a near-term blocker. The orientation v1.8 §17 unknown about "actual latency on the 7,600-note Universe" is now resolved — fold the answer into §17 in a future bump.

**Two findings to address**:

**1. Right-sidebar tab strip clipping** (Boss-attached screenshot): the new 360° Inspector tab icon at the right edge of the rs-tabs strip is partially cut off — the strip overflows the sidebar's content area at the default 340 px width. With 11 tabs (now: properties, backlinks, tags, star, tasks, calendar, health, provenance, review, links, inspector360), the default browser button padding plus the 16 px icon plus tab margins exceeds available width even at `flex: 1`. The 11th tab is rendered but visually clipped. Boss-clickable still (verified — Stage 1 worked), but the visual presentation is wrong. **Fix in §97 below.**

**2. Back-navigation from Inspector**: Boss wants to "return to the previous note" after clicking an orbiting dot in the inspector. The default Constellation navigation history (`navigateBack` / `navigateForward`, bound to Alt+Left and Alt+Right) DOES preserve the prior note — pressing Alt+Left from the new note reverts to the source note. But the request seems to be for an explicit in-context back affordance inside the Inspector itself, not a global keyboard shortcut. Three possible implementations: (A) a small back button in the Inspector full-window header (compact mode has no header to host one), (B) breadcrumbs showing the path A → B → C, (C) hold-Shift-click on a node to "open in new tab" instead of replacing. Decision deferred to Boss; logged for §98 candidate.

---

## §97 — rs-tabs strip overflow fix

**Boss-reported during Stage 1.7 retest (2026-04-29, screenshot showed partial-clip of the inspector reticle icon at the sidebar's right edge).**

**Root cause**: `.rs-tab` had `flex: 1` (allowing infinite shrink in theory) but inherited the browser's default `<button>` padding (~6–12 px each side) which set an effective minimum width per tab. Eleven tabs × (~16 px icon + ~16 px button padding) ≈ 350 px just for the tab strip, exceeding the 340 px sidebar default minus scrollbar. The 11th tab rendered past the sidebar's right edge.

**Fix** (single CSS rule edit in `+layout.svelte`):

- `.rs-tabs`: added `flex-wrap: wrap` so any future tab beyond the sidebar's capacity gracefully wraps to a second row instead of clipping.
- `.rs-tab`: replaced `flex: 1` with `flex: 1 1 28px; min-width: 24px;` — the explicit basis + min-width gives the wrap algorithm sensible breakpoints. Added `padding: 0` to remove default browser button padding.

**Effect**: at the default 340 px width, all 11 tabs fit on a single row with even spacing. If a future migration adds a 12th sidebar tab, or if the user resizes the sidebar narrower than ~290 px, the strip wraps to a second row instead of clipping. No JavaScript involved; pure CSS.

**Build verification**: `npm run check` clean of new errors (the 1 remaining error in store.ts:1850 LinkLifecycle missing 'fresh' is pre-existing).

**Files changed**:
- `src/routes/+layout.svelte` (`.rs-tabs` + `.rs-tab` CSS).
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry + Stage 1 verification + back-nav decision queue).
