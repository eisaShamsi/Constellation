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

---

## §98 — Inspector 360 back-nav button (compact mode, single-step)

**Boss decision (2026-04-29)**: option (A) — explicit back affordance inside the 360.3D function sidebar (compact mode). Wired as a small back-bar above the SVG that shows the previous note's name and navigates back when clicked.

**Scope chosen**: single-step back, compact mode only. Full-window mode unchanged (it auto-closes on node click in §93, so a back button there would be invisible). If Boss later wants full-window back-nav, the props are already there — only the mount-site call needs to opt in.

**State added** (`+layout.svelte:~390`):
- `inspector360PreviousPath: string | null = $state(null)`
- `inspector360PreviousName: string | null = $state(null)`

**Compact mount onNoteClick now saves before navigating**: when the user clicks an orbiting node, the current `sidebarTab.path` + `sidebarTab.name` are captured into the previous-state vars, then `openNoteTab` runs to navigate. The IPC `$effect` re-fires for the new note, the visualisation updates, and the back-bar appears showing "← {previousName}".

**onBack handler**: clears the previous-state vars and calls `openNoteTab` for the saved previous path. Single-step semantics — once back is clicked, the back state is empty until the next forward navigation. This matches the simple-back mental model (not a multi-hop history).

**Universe-switch reset extended**: the existing block that clears `mapEverOpened` / `orgChartEverOpened` / `inspector360EverOpened` / `inspector360Data` now also clears `inspector360PreviousPath` and `inspector360PreviousName`. So switching to a different Universe always resets back-state.

**Inspector360.svelte component changes**:
- Two new optional props: `previousNoteName?: string | null` and `onBack?: () => void`.
- When both are set AND in compact mode, a small back-bar renders above the SVG: `← {truncated previousNoteName}`. Subtle styling (light grey background, hover effect).
- Full-window mode does NOT render the back-bar regardless of props (kept the auto-close UX from §93). If we later want a full-window header back button, that's a follow-up.

**Mental model**:
- A → click B in inspector → on B, "← A" visible in compact widget.
- Click "← A" → on A, back-bar hidden.
- A → click B → click C → on C, "← B" visible (single-step, A is forgotten).
- Alt+Left and Alt+Right still work as the global multi-hop history; the inspector's back-bar is independent.

**Interaction with §97 layout**: the back-bar uses `align-self: stretch` so it fills the compact widget's available width regardless of the rs-tabs strip width. Padding `4px 8px`, font 0.78rem — small but readable.

**Build verification**: `npm run check` clean of new errors. The 1 remaining error in store.ts:1850 (LinkLifecycle missing 'fresh') is pre-existing.

**Rebuild required**: the §97 rebuild that completed at ~13:53 does NOT include this §98 change. A fresh `npm run tauri build` runs after this commit lands so the Boss test picks up both fixes together.

**Files changed**:
- `src/lib/components/Inspector360.svelte` (props + back-bar markup + CSS).
- `src/routes/+layout.svelte` (state + universe-switch reset + compact mount onNoteClick / onBack wiring).
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry).

---

## §99 — Multi-hop back-nav stack (Boss request)

**Boss-reported during Stage 1.7+1.9 retest (2026-04-29 ~14:25)**: width fix ✅, back button works ✅, but the single-step semantics are insufficient. The Boss wants the back chain to walk **all the way back to the original note**, not just one hop.

**Scope change**: replace the single-step `inspector360PreviousPath` / `inspector360PreviousName` pair with a stack of `{path, name}` entries. Each forward node-click pushes the current note onto the stack; each back click pops one entry.

**Implementation**:

- Replaced the two `$state` vars with a single `inspector360BackStack: $state<Array<{path: string; name: string}>>`. Initial value `[]`.
- The compact mount's `onNoteClick` now pushes `{path: sidebarTab.path, name: sidebarTab.name}` onto the stack via spread (`= [...stack, entry]`) for proper Svelte 5 reactivity.
- The compact mount's `onBack` pops the top entry via spread + pop (`const next = [...stack]; const target = next.pop()`), navigates to `target.path`, and reassigns the (now-shorter) stack.
- The `previousNoteName` prop reads `stack[stack.length - 1]?.name` so the back-bar always shows the immediate predecessor's name.
- Universe-switch reset clears the stack to `[]`.

**Mental model**:

- A → click B → stack `[A]`, on B, bar shows "← A".
- A → click B → click C → stack `[A, B]`, on C, bar shows "← B".
- … click back → pop B, stack `[A]`, on B, bar shows "← A".
- … click back → pop A, stack `[]`, on A, bar hidden.
- A → click B → click C → click back → click D from B → stack `[A, B]`, on D, bar shows "← B". (Stack rewrites correctly when the user re-branches.)

**Edge cases handled**:

- Empty-stack back click → no-op (button hidden anyway).
- Loop nav (A → B → A): stack `[A, B]`, bar shows "← B" — back unwinds correctly.
- Universe switch resets the stack to `[]`.
- Tab close / sidebar tab change keeps the stack (no clear) — opening the inspector tab again still shows the back chain. Acceptable for v1; if Boss wants stack to clear when inspector tab is left, that's a one-line follow-up.

**Stack growth**: unbounded by design — long traversals make a long stack, but each entry is just two strings, so memory is negligible. No cap added; if Boss wants one (e.g. "max 50"), trivial follow-up.

**Multi-hop doesn't replace Alt+Left/Alt+Right**: those still work as the global navigation history across all of Constellation (not just the inspector). The inspector's back stack is independent — it tracks only inspector-driven navigation. So Alt+Left still walks across all kinds of nav (wikilink clicks, tab clicks, inspector clicks); the inspector's "←" only walks inspector clicks.

**Build verification**: `npm run check` clean of new errors.

**Files changed**:
- `src/routes/+layout.svelte` (back-state replaced with stack; onNoteClick / onBack rewritten; universe-switch reset updated).
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry).

No `Inspector360.svelte` change — the component's prop contract (`previousNoteName?: string | null` + `onBack?: () => void`) didn't change. The mount site computes `previousNoteName` from the stack and provides the popping `onBack` handler. Component agnostic to single-step vs multi-hop semantics.

---

## §100 — Stage 2 Boss findings: tooltip + sizing + sector grouping + full-window back-nav

**Boss-reported during Stage 2 testing (2026-04-29 ~17:00)**, with the explicit reminder that long tutorials should be split into stages going forward. Acknowledged: the Stage 2 tutorial bundled 2.1–2.7 in one message, which violates `feedback_staged_tests.md`. Future stages go out one sub-stage at a time.

Five concrete findings, all addressed in this commit:

**Finding 1 — Dock button tooltip leaks the i18n key (`ribbon.inspector360`).** The chain `$t('ribbon.inspector360') || $t('inspector360.title') || '360° Inspector'` doesn't fall through because `$t()` returns the key name as a non-empty string when the key is missing. `ribbon.inspector360` was never added to any of the 15 locales (intentionally deferred to a future i18n round per §93 notes), so the chain returned "ribbon.inspector360" verbatim and rendered as the tooltip. **Fix**: dropped `$t('ribbon.inspector360')` from the fallback chain. The dock button now uses `$t('inspector360.title') || '360° Inspector'`, which resolves to the localised "360.3D" (or its translated equivalent) in all 15 locales. The English fallback `'360° Inspector'` is reachable only if every locale's `inspector360.title` is missing, which is not the case.

**Finding 2 — Visualisation does not fill the available canvas (2.1.5).** The `.i360-viz` rule had `max-width: 1400px; max-height: 900px;` which capped the SVG even on monitors much larger than that. Result: the visualisation rendered small in the centre of the dark canvas with vast empty space around it. **Fix**: removed both max-* constraints. The SVG now fills the canvas to 100% / 100% of `.i360-canvas`. Ring radii bumped slightly (`rings = [180, 290, 400]` from `[160, 270, 380]`) to occupy the additional space proportionally.

**Finding 3 — Side panels and HUD undersized (2.3.13/14, 2.4.16).** Original sizes were tuned for the compact-view aesthetic that Boss chose to discard for full-window. **Fix**: doubled padding, font-size, and key dimensions across:
- `.i360-header`: padding 18px 32px (was 10px 20px), gap 16px.
- `.i360-header-icon`: 28px (was 18px).
- `.i360-header-label`: 16px (was 11px).
- `.i360-header-name`: 26px (was 18px).
- `.i360-mode-select`: 16px font, padding 8px 16px (was 12px / 5px 10px).
- `.i360-close`: 48px × 48px (was 36×36), font 28px (was 20px).
- `.i360-panel`: 18px font, padding 18px 22px, max-width 380px, line-height 1.7 (were 11px / 10px 14px / 200px / 1.8). Position offset 100px / 32px (was 60px / 20px).
- `.i360-panel-title`: 18px font, margin-bottom 10px (were 11px / 4px).
- `.i360-panel-item` gap 10px (was 6px). Item color rgba 0.6 (was 0.5) for legibility at scale.
- `.i360-dot`: 14px × 14px (was 8×8).
- `.i360-hud`: padding 18px 36px (was 10px 24px).
- `.i360-hud-item`: 18px font, gap 8px, color rgba 0.55 (were 11px / 4px / 0.4).
- `.i360-hud-left/-right` gap 28px (was 16px).
- Node radii bumped: depth 1 = 11px (was 9), depth 2 = 8px (was 7), depth 3 = 5px (was 4).

**Finding 4 — Nodes scattered without clear typed-link grouping (2.2.11, image 5).** The previous spread formula `(i - (n - 1) / 2) * 15` used a fixed 15° per-note step regardless of total count. A sector with 10 notes spread to 150°, overlapping neighbouring sectors. **Fix**: replaced with a normalised-and-bounded formula:
```javascript
const SECTOR_WIDTH = 50; // degrees per typed-link sector
const offset = n > 1 ? (i / (n - 1) - 0.5) * SECTOR_WIDTH : 0;
```
Each typed-link sector now occupies a fixed 50° angular range centred on its sector angle, regardless of how many notes it contains. High-density sectors pack more tightly within their 50° wedge instead of spilling into neighbours. Visually: nodes cluster cleanly at the seven sector positions (supports/contradicts/causes/derives-from/generalizes/exemplifies/part-of) instead of forming a continuous ring around the centre. This also makes the Cosmic Sphere mode's labelled sector wedges (the rim labels in the §93 implementation) actually align with the nodes they represent.

**Finding 5 — Full-window auto-closes on node click; Boss wants "Return to 360.3D" (2.5.17).** The §93 design auto-closed the inspector when an orbiting node was clicked — that's contrary to the back-nav model the Boss wants. **Fix**:
- Removed `showInspector360 = false;` from the full-window mount's `onNoteClick`. The full-window now stays open after node click, refetches the new note's view, and walks the back stack like compact mode.
- Added the back-stack push to the full-window mount's `onNoteClick` (same shape as compact).
- Wired the `onBack` and `previousNoteName` props to the full-window mount (same expressions as compact).
- Added a "Return to {previous}" button in the full-window header's left cluster, conditional on `previousNoteName && onBack`. Styled as a pill button (white-08 background, 16px font, hover state).
- The back stack is shared between compact and full-window — clicking a node in compact, then opening full-window, sees the same back history. Clicking back in full-window pops the same stack compact uses.

**One finding deferred (2.2.9 — viz modes look identical).** Boss observed Atmospheric Rings and Neural Web are visually indistinguishable. Speculation: the previous crowding obscured the differences. With Finding 4's tighter sector grouping, the modes may now show their distinct character (Atmospheric: rotating ellipses + per-link synaptic lines; Neural: organic web with second-order branching; Cosmic: orbital paths with sector labels). If they still look the same after the §100 build, that's a follow-up (likely a more aggressive design pass on each mode's signature look). Boss explicitly accepted this for now via "All Pass, considering my remarks".

**Build verification**: `npm run check` clean of new errors.

**Files changed**:
- `src/lib/components/Inspector360.svelte` (sector formula + full-window back button + sizing pass).
- `src/routes/+layout.svelte` (dock-button tooltip + full-window mount: back-nav props, onNoteClick refactor without auto-close).
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry).

**Stage 2 retest plan** (split into sub-stages per Boss directive):
- 2A: dock-button tooltip + full-window opens + sized correctly.
- 2B: visualisation fills canvas + nodes grouped by typed-link sector.
- 2C: side panels + HUD legible at 2× scale.
- 2D: full-window back-nav (Return to N from full-window header).
- 2E (deferred): viz mode distinctness — only triaged if Boss flags it again after seeing the new sectoring.

---

## §101 — Multi-ring stacking within sector (Stage 2A finding)

**Stage 2A retest result (2026-04-29 ~17:50)**: dock-button tooltip ✅ (now shows "360.3D" instead of "ribbon.inspector360"); full-window opens cleanly ✅. **New finding from the retest screenshot**: high-count typed-link sectors (e.g. derives-from with 15+ depth-1 notes) pile up onto a single ring, creating cramped clusters where adjacent nodes overlap. Boss directive: "spread the nodes by group/type equally around the sphere".

**Root cause of the pile-up**: §100's spread formula:
```javascript
const offset = n > 1 ? (i / (n - 1) - 0.5) * SECTOR_WIDTH : 0;
const ring = note.depth <= 1 ? 0 : note.depth <= 2 ? 1 : 2;
```
When a sector has 15 depth-1 notes, all 15 land on `rings[0]` (radius 180), spread across 50° at the depth-1 ring → 3.3° between adjacent notes. With node radius 11 px on a 180-radius arc, adjacent nodes overlap by ~13 px (arc-length 9.4 px, diameters 22 px each).

**Fix**: when a sector exceeds `MAX_PER_RING = 8`, spill notes onto multiple rings WITHIN the same sector (still bounded to `SECTOR_WIDTH = 50°`, so they don't leak into neighbouring sectors). Notes are sorted by depth ascending, so closer notes (more relevant) occupy the inner ring first; overflow goes outward.

**New formula**:
```javascript
const sorted = [...noteList].sort((a, b) => a.depth - b.depth);
const numRings = Math.min(3, Math.max(1, Math.ceil(n / MAX_PER_RING)));
const perRing = Math.ceil(n / numRings);
for each note at index i:
  ringNum = Math.min(numRings - 1, Math.floor(i / perRing));
  indexInRing = i - ringNum * perRing;
  countInRing = Math.min(perRing, n - ringNum * perRing);
  offset = countInRing > 1 ? (indexInRing / (countInRing - 1) - 0.5) * SECTOR_WIDTH : 0;
  pos = polarToXY(cx, cy, info.angle + offset, rings[ringNum]);
```

**Effect on the screenshot example**: a sector with 15 notes now uses 2 rings (8 + 7), each ring spreading ~7 nodes across 50° → ~7° between adjacent nodes per ring. Adjacent-node arc length on 290-radius outer ring ≈ 35 px, well clear of the 22 px diameters. Sector occupies a 2-ring radial column in its angular wedge instead of a 1-ring overflow pile-up.

**Trade-off**: depth-based ring assignment is replaced with count-based ring assignment. The previous "depth-1 always on inner ring, depth-2 on middle, depth-3 on outer" semantic is lost in dense sectors — instead, the inner ring carries the LOWEST-DEPTH 8 nodes and outer rings carry the rest. Sparse sectors (≤8 notes total) still use the inner ring, so the typical case looks the same as before.

**Untyped links unchanged**: still scattered around the full circle on depth-based rings. They form the outer scatter ring and don't compete with typed sectors for sector-wedge real estate.

**SECTOR_MAP unchanged**: the asymmetric layout (supports↔contradicts at 180° opposition, causes↔derives-from at 180° opposition, etc.) is preserved. The 90° gap between contradicts (180°) and derives-from (270°) on the bottom-left is by design — opposing the 0–180° dense side. If Boss later wants equal angular distribution (all 7 sectors at 51.4° apart, losing the 180° oppositions), that's a separate redesign.

**Build verification**: `npm run check` clean of new errors.

**Files changed**:
- `src/lib/components/Inspector360.svelte` (the `allNodes = $derived.by(...)` block).
- `lab/reports/SESSION-LOG-2026-04-29.md` (this entry).

**Stage 2B retest will follow once the §101 binary builds.** Just visualisation/sectoring focus per the staged-tests rule — panels + HUD (2C) and back-nav (2D) come later.
