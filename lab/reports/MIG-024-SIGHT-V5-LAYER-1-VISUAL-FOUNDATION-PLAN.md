# MIG-024 — Sight v5 Layer 1 Visual Foundation — Plan

**Phase:** 2 of 4 (/migration discipline) · **Date:** 2026-05-12
**Status:** Draft for Eisa to approve once. Plan-Approval-Equals-Build-Approval; cascade is autonomous after approval.
**Architect:** `lab/reports/MIG-024-SIGHT-V5-LAYER-1-VISUAL-FOUNDATION-ARCHITECT.md`
**Reference contract:** `docs/Constellation-Sight-Concept-Paper-v3.1.md` §12.1.
**Reference visual:** `docs/Sight-vNext-MockB1-Toggle.svg`.
**§N audit folded:** `lab/reports/MIG-022-§N-FINAL-INTEGRATION-AUDIT.md` D-N1 = α (UPSERT) + D-N2 = a (inside MIG-024 §0).

---

## §0 · Locks from the Architect + §N audit

All decisions Eisa-locked 2026-05-12:

| ID | Lock | Notes |
|---|---|---|
| D-V1 | **Canvas 2D + D3-zoom** | v4's proven path |
| D-V2 | **Slide-in right side panel** | Matches Backlinks/Outgoing pattern |
| D-V3 | **Per-Universe / per-Library / per-Folder, user-toggleable in-canvas** | **Expansion**: adds a 3-button scope toggle to the chrome (U / L / F) alongside the 7-button mode toggle. Default = Universe. Persisted per-Universe via `appSettings.sight.lastScope`. Concept Paper v3.2 captures this at MIG-024 close-out per the inline-with-trigger rule. |
| D-V4 | **Per-note row × 1 layout cache** | Per-mode reprojection in JS at render time |
| D-V5 | **Hide v4 immediately at v5 ship** | `SIGHT_V4_ENABLED = false` flips in §1; v4 component stays on disk for cleanup MIG |
| D-V6 | **Giant Unsourced wedge + CTA** | "Classify some notes via Source Review →" handoff to right sidebar |
| D-V7 | **Hover-only, no labels toggle** | Side panel is the detail surface |
| D-N1 | **UPSERT in `index_note`** | Replaces DELETE+INSERT; trigger fires natively |
| D-N2 | **Fix inside MIG-024 §0** | First cluster, before visual-foundation work |

---

## §1 · Source material reviewed

- Architect doc §1–§11 (territory, invariants, work clusters, decisions, risks, test surface)
- Concept Paper v3.1 §5 (visual grammar) + §6 (seven modes) + §7 (four constants) + §10 (boundary lines) + §11 (perf budgets) + §12.1 (MIG-024 scope)
- Mock B1 SVG (current 7-button rev) — pixel-fidelity contract
- §N audit final report — D-N1 + D-N2 lock the §0 cluster
- `src/lib/sight/engine.ts` — current flag state (V2 false, V3 false, V4 true; V5 not yet declared)
- `src/lib/sight/` directory inventory: `calendar-rim.ts`, `community-territory.ts`, `density-cache.ts`, `engine.ts`, `layout-cache.ts`, `palette.ts`, `projection.ts`, `universe-health.ts`, `v3/`, `v4/`
- `src/lib/components/SkyView.svelte` — proven mount pattern Sight v5 mirrors
- `src-tauri/src/search.rs:3045-3054` — `index_note` (the §0 UPSERT remediation target)
- `src-tauri/src/cece/history.rs` — the trigger that needs `index_note` to fire UPSERT not DELETE+INSERT

---

## §2 · Risk register inheritance

All 10 risks from Architect §9 carry through to the Plan unchanged. Add:

- **R-11** (NEW from D-N1) — UPSERT change in `index_note` could break existing search-indexing path or re-trigger note_meta_a{i,d,u} downstream consumers in unexpected ways. **Mitigation:** §0's verification clause includes a full search-rebuild test on the trial Universe; FTS5 index integrity verified pre/post.
- **R-12** (NEW from D-V3) — Scope toggle introduces a second axis of state (mode × scope = 7 × 3 = 21 visual configurations). Mental-model risk for users + cache-key risk for the layout cache. **Mitigation:** scope filter applies BEFORE wedge computation as a row filter on `note_meta`; same per-mode azimuth dispatch then runs against the filtered set. Layout cache key bumps to `(library_set_hash, scope_kind, scope_id)`.
- **R-13** (NEW from D-V5) — v4 hidden immediately means no rollback target via the dock button. **Mitigation:** v4 component stays on disk; flipping `SIGHT_V4_ENABLED = true` in `engine.ts` + rebuild brings v4 back. Cleanup MIG retires only after Eisa confirms v5 stable across multiple sessions.

---

## §3 · Phase sequence — 7 clusters

Each cluster = 1 commit (or a small focused commit pair if the diff splits naturally). Verification clause per cluster is the gate; if the clause fails, fix before commit. Cascade per Plan-Approval-Equals-Build-Approval.

### §0 — Trigger-coverage UPSERT remediation (D-N1 + D-N2.a)

**What ships:**
- `src-tauri/src/search.rs::index_note` — replace `DELETE FROM note_meta WHERE path = ?; INSERT INTO note_meta (...) VALUES (...)` with `INSERT INTO note_meta (...) VALUES (...) ON CONFLICT(path) DO UPDATE SET col1 = excluded.col1, col2 = excluded.col2, ...`
- All `note_meta` columns named explicitly in the UPDATE clause (no `*`-style shortcuts; future-proof against schema additions)
- Existing `note_meta_a{i,d,u}` triggers continue to work (UPSERT fires `note_meta_au` correctly)
- **The new `note_state_history_au` trigger (MIG-022 §B.2) now catches every direct YAML edit via NotePane**, not just CECE classifier writes
- 1-2 new tests in `search.rs` test module: (a) UPSERT on existing path produces single `note_meta_au` event (not delete+insert); (b) note_state_history table receives an event when a note's epistemic field changes via `index_note`

**Files touched:**
- `src-tauri/src/search.rs` — `index_note` body
- `src-tauri/src/search.rs` test module (or `cece/history.rs` test module) — new tests

**Verification clause:**
- `cargo test --lib search` PASS (no FTS5 regression; UPSERT path tested)
- `cargo test --lib cece::history` PASS (existing 11 tests + 1-2 new ones)
- Manual: open the dev DB, edit a note's `held_by` field via NotePane, save, verify `note_state_history` table has a new row with the change.

**Estimated effort:** ½–1 day.

---

### §1 — Engine flag + module skeleton

**What ships:**
- `src/lib/sight/engine.ts` — `export const SIGHT_V5_ENABLED = false;` declared. **`SIGHT_V4_ENABLED` flips to `false` in this same edit per D-V5** (immediately at v5 ship). v5 ship = the §6 close-out commit, not §1; so for §1 itself, V5 stays false and V4 stays true. The flag flip happens in §6.
- `src/lib/sight/v5/` directory created.
- `src/lib/sight/v5/SightV5.svelte` — minimal stub component (renders a placeholder div labeled "Sight v5"); mounted via dock button when `SIGHT_V5_ENABLED === true`.
- `src/lib/sight/v5/types.ts` — TypeScript types for `SightV5Mode = 'R'|'L'|'T'|'C'|'S'|'A'|'P'`, `SightV5Scope = 'universe'|'library'|'folder'`, `Star`, `Wedge`, `LayoutCacheRow`.
- Dock button in `+layout.svelte` (or wherever the existing Sight dock entry lives) — conditional render: shows v4 button if V4 flag, shows v5 button if V5 flag, never both.
- `appSettings.sight` interface extended in `src/lib/libraries/store.ts`: `lastMode: SightV5Mode`, `lastScope: SightV5Scope`. Defaults: `R` and `universe`. `DEFAULT_SETTINGS` + `loadSettings` deep-merge updated (the V3-§11 AT RISK pattern — never miss this).

**Files touched:**
- `src/lib/sight/engine.ts`
- `src/lib/sight/v5/SightV5.svelte` (new)
- `src/lib/sight/v5/types.ts` (new)
- `src/routes/+layout.svelte` (dock button conditional)
- `src/lib/libraries/store.ts` (`appSettings.sight` interface + defaults + load)

**Verification clause:**
- `npm run check` PASS (no svelte-check errors).
- Dev build: dock has only the v4 button (V5 still gated false). No regression.

**Estimated effort:** ½ day.

---

### §2 — Layout cache (Rust + IPC)

**What ships:**
- New SQLite table `sight_v5_layout` in `init_db` migration (additive; idempotent CREATE):
  ```sql
  CREATE TABLE IF NOT EXISTS sight_v5_layout (
      note_path TEXT PRIMARY KEY,
      stratum INTEGER,
      maturity TEXT,
      confidence_alpha REAL,
      contested INTEGER NOT NULL DEFAULT 0,
      library_path TEXT,
      folder_path TEXT,
      created_month INTEGER,
      sources_primary TEXT,
      stage TEXT,
      acts_primary TEXT,
      dominant_link_type TEXT,
      universe_snapshot_hash TEXT NOT NULL,
      computed_at INTEGER NOT NULL
  );
  CREATE INDEX IF NOT EXISTS idx_sight_v5_layout_snapshot
    ON sight_v5_layout(universe_snapshot_hash);
  ```
- Write-time cache invalidation: SQLite trigger that deletes the matching `sight_v5_layout` row on any `UPDATE note_meta` (after the §0 UPSERT, this fires correctly for both classifier writes AND direct YAML edits).
- Backfill function `backfill_sight_v5_layout(conn)` — populates the table for all existing notes on first boot of the new build; idempotent via `schema_versions` sentinel `'mig024_sight_v5_layout_backfill_v1'`.
- 4 new IPCs in `src-tauri/src/sight_v5.rs` (new module):
  - `sight_v5_get_layout(scope_kind, scope_id) → Vec<LayoutCacheRow>` — returns all rows in the requested scope.
  - `sight_v5_get_universe_snapshot_hash() → String` — used by frontend to detect cache invalidation.
  - `sight_v5_get_link_set_for_notes(paths: Vec<String>) → Vec<LinkEdge>` — returns typed-link edges between the visible notes (for connector-line rendering).
  - `sight_v5_warm_cache() → Result<()>` — fired by `requestIdleCallback` after `boot:hydrated`; recomputes the cache if hash mismatch.
- `src-tauri/src/lib.rs` — register the 4 new IPCs.

**Files touched:**
- `src-tauri/src/search.rs` — init_db gets `ensure_sight_v5_layout_table` + `ensure_sight_v5_invalidation_trigger` + `backfill_sight_v5_layout` calls (mirrors the §B pattern)
- `src-tauri/src/sight_v5.rs` (new module) — IPCs + helpers
- `src-tauri/src/lib.rs` — IPC registrations
- `src-tauri/src/cece/mod.rs` or top-level — `pub mod sight_v5;`

**Verification clause:**
- `cargo test --lib sight_v5` PASS — at least 4 new tests: table created, trigger fires on `note_meta` UPDATE, backfill is idempotent + resumable, scope filter returns correct row sets.
- Manual: on dev DB, run backfill, verify row count matches `note_meta` count, verify rows have correct strata + library_path values.

**Estimated effort:** 2-3 days.

---

### §3 — Dome geometry + Canvas 2D render layers

**What ships:**
- `src/lib/sight/v5/dome.ts` — pure functions:
  - `stratumRadii(domeRadius: number): number[]` — returns 8 radii for L1 → L8 bands.
  - `calendarRimMonths(domeRadius: number, locale: string): MonthLabel[]` — returns 12 month label positions + locale-aware month names via `Intl.DateTimeFormat`.
  - `milkyWayEllipses(domeRadius: number): Ellipse[]` — returns 2 ellipse specs matching Mock B1 placement.
  - `radiusForStratum(stratum: number, domeRadius: number): number` — mode-invariant; the load-bearing constant.
- `src/lib/sight/v5/render.ts` — Canvas 2D + D3-zoom render pipeline:
  - `renderBaseLayer(ctx, dome, stars, milkyWay, calendarRim)` — drawn once per cache-warm cycle. Static.
  - `renderFocusOverlay(ctx, focusedStar, incidentEdges)` — drawn on hover/select state change. Sparse.
  - HTML overlays for month labels (NOT canvas-drawn text) per v3 invariant 12; `dir="auto"` for RTL.
  - D3-zoom for pan/zoom on the dome.
- SightV5.svelte mounts two `<canvas>` elements + DOM month labels + DOM zoom container. Reuses SkyView's flex-child mount pattern (close button in `+layout.svelte` header per the v4 lesson).
- Suwaidi palette tokens imported from existing `src/lib/sight/palette.ts` (Plan-phase audit per Architect §8 verifies this is reusable; if not, copy-modify into `v5/palette.ts`).

**Files touched:**
- `src/lib/sight/v5/dome.ts` (new)
- `src/lib/sight/v5/render.ts` (new)
- `src/lib/sight/v5/SightV5.svelte` (real implementation replacing §1 stub)
- `src/routes/+layout.svelte` (close-button row hosts a v5 close button)
- Possibly `src/lib/sight/palette.ts` (if audit reveals drift from Mock B1 tokens)

**Verification clause:**
- Stage-1-quality build: open SightV5 in dev (set V5 flag true locally), verify dome renders with 8 bands + calendar rim + Milky Way wash + close button works + Esc closes.
- No stars yet (those land in §5); the dome chrome alone should match Mock B1 within Suwaidi palette tolerance.
- `npm run check` PASS.

**Estimated effort:** 4-5 days.

---

### §4 — Seven mode toggles + per-mode wedge dispatch + scope toggle (D-V3)

**What ships:**
- `src/lib/sight/v5/modes.ts` — pure functions:
  - `azimuthForMode(mode: SightV5Mode, note: LayoutCacheRow, context: ModeContext): number` — returns angle in radians; one per mode (R/L/T/C/S/A/P).
  - `wedgeBucketsForMode(mode: SightV5Mode, notes: LayoutCacheRow[], context: ModeContext): WedgeBucket[]` — returns the wedge configuration for the active mode given the visible note set.
  - `wedgeColorForMode(mode, bucketKey)` — returns the (subtle) wedge background tint for the rim, if any (most modes will be no-tint).
- `src/lib/sight/v5/scope.ts` — pure functions:
  - `filterNotesByScope(rows: LayoutCacheRow[], scope: SightV5Scope, scopeId: string | null): LayoutCacheRow[]` — applies the user's chosen scope filter before wedge computation.
- Mode toggle bar in SightV5.svelte (R · L · T · C · S · A · P) — three states (active gold / ready / dimmed) per Mock B1; click triggers 600 ms ease angular animation.
- **Scope toggle bar (D-V3)** — 3 buttons U · L · F above or below the mode bar (placement TBD during render — TBD inside the cluster, NOT a new question to Eisa). Default = U. When user is currently focused on a Library or Folder (the active sidebar selection), L and F unlock and switch to that context's scope. Persisted to `appSettings.sight.lastScope`.
- Mode-switch animation: ~600 ms ease with `cubic-bezier(0.4, 0, 0.2, 1)`; interpolates `azimuthForMode(prevMode, ...)` → `azimuthForMode(newMode, ...)` per star.
- Empty-state handling: mode P with universe-wide < 5 % classified → giant Unsourced wedge with CTA "Classify some notes via Source Review →" (D-V6); CTA opens the right sidebar Source Review panel.

**Files touched:**
- `src/lib/sight/v5/modes.ts` (new)
- `src/lib/sight/v5/scope.ts` (new)
- `src/lib/sight/v5/SightV5.svelte` (mode + scope toggle bars; animation orchestration)
- `src/lib/libraries/store.ts` (potentially: helper to read current Library/Folder context for scope L/F unlock)

**Verification clause:**
- Each mode renders correct wedges on the trial Universe. R splits by Library, L by typed-link kind, T by month, C by confidence, S by stage, A by act, P by source family.
- Scope toggle: switching U → L on a focused Library hides notes outside that Library. Spatial memory preserved (same star at same stratum band before/after).
- Mode-switch animation runs smoothly at 60 FPS on Eisa's 7,636-note universe.
- `npm run check` PASS.

**Estimated effort:** 4-5 days (was 3-4; +1 for scope toggle).

---

### §5 — Stars + connectors + side panel + interactivity

**What ships:**
- Star rendering in `render.ts::renderBaseLayer`:
  - Position: `(radiusForStratum(note.stratum), azimuthForMode(mode, note, context))` → polar to Cartesian.
  - Size: maturity → seed (1.5 px) / sapling (2.5) / evergreen (3.5) / canonical (5) / wilting (2 grey).
  - Brightness: confidence → hypothesis (0.45) / evidence (0.7) / established (1.0).
  - Color: ink (`#1a1a1a`) by default; red (`#a83232`) for `contested === 1`.
- Connector lines in `renderFocusOverlay`:
  - Faint at rest (~0.10–0.15 alpha) — drawn in base layer.
  - On hover/select: focused star's incident edges brighten to ~0.85 alpha — drawn in focus overlay.
  - Color per 9 typed-link kinds (Concept Paper §5.4 + supersedes slate-blue `#5B7A8A`).
- Hover/select state machine in SightV5.svelte:
  - Hit-testing on mousemove: nearest-star-within-radius lookup (use a quad-tree or simple radius-bucket; perf budget ≤ 16 ms per frame).
  - Hover → tooltip near cursor (note title + stratum + maturity + stage badge); other stars don't dim (per Concept Paper — keep chrome quiet).
  - Click → side panel slides in.
  - Click background or Esc → clear selection.
  - Click another star → reassign focus.
- `src/lib/sight/v5/SightV5SidePanel.svelte` — slide-in right panel:
  - Header: note title (with `dir="auto"`).
  - Body: strata badge + maturity + stage + sources (if any) + confidence summary + top-5 incident links.
  - Footer: "Open in editor" button — handoff to NotePane via the existing `openNote` helper (mirror Backlinks/Outgoing's call site).
  - Close button (top-right of panel).

**Files touched:**
- `src/lib/sight/v5/render.ts` (star + connector rendering)
- `src/lib/sight/v5/SightV5.svelte` (hover/select state machine; side panel mount)
- `src/lib/sight/v5/SightV5SidePanel.svelte` (new)

**Verification clause:**
- All ~7,636 stars render at correct positions on the trial Universe within perf budget (≤ 500 ms cold, ≤ 50 ms warm).
- Hover any star → tooltip appears in <16 ms; incident edges brighten visibly.
- Click a star → side panel slides in within 200 ms; "Open in editor" opens the note in NotePane.
- Esc clears selection cleanly.
- `npm run check` PASS.

**Estimated effort:** 3-4 days.

---

### §6 — i18n stub + help stub + dock button + Settings + V5 flag flip

**What ships:**
- New `sight.v5.*` i18n key block in `src/lib/i18n/en.json` + `src/lib/i18n/ar.json`:
  - Mode names (Regions, Link Types, Time, Confidence, Stages, Acts, Provenance) with one-line tooltips
  - Scope names (Universe, Library, Folder)
  - Side-panel labels (strata, maturity, stage, sources, confidence, "Open in editor")
  - Empty-state strings ("Classify some notes via Source Review →", "Available later", etc.)
  - Tooltip text for dimmed mode buttons
- Backfill the `sight.v5.*` block to 13 other locales via 5 parallel agents per the V3-§10.D pattern (Latin / Iberian-Slavic / Arabic-script-Hebrew / CJK / Turkish-Hindi).
- Help topic stub at `docs/help.uConstellation.World/Sight v5/Sight v5.md` — brief introduction (1-2 pages) covering the dome + 4 constants + 7 modes + scope toggle + how to interpret. NOT the comprehensive help (lands with MIG-027 / Layer 4).
- Help topic stub translated to 14 locales (parallel agents).
- User Manual stub `## 11. Sight v5` chapter in `docs/User Manual.md` — short overview pointing at the help topic for detail.
- Dock button label + tooltip via i18n.
- Settings → Sight section: scope-toggle behavior toggle (off = scope locked to U; on = user can toggle U/L/F in canvas) — default on.
- **`SIGHT_V4_ENABLED = false` + `SIGHT_V5_ENABLED = true`** in `src/lib/sight/engine.ts` (the v5 ship moment per D-V5).

**Files touched:**
- `src/lib/i18n/en.json` + `ar.json` (manual)
- `src/lib/i18n/{de,es,fr,pt,ru,fa,ur,he,ja,ko,zh,tr,hi}.json` (parallel agents)
- `docs/help.uConstellation.World/Sight v5/Sight v5.md` (new, EN)
- `docs/help.{de,es,fr,pt,ru,fa,ur,he,ja,ko,zh,tr,hi,ar}/Sight v5/Sight v5.md` (new, parallel agents)
- `docs/User Manual.md` (new chapter)
- `docs/help.{14 locales}/User Manual.md` (chapter translations, parallel agents)
- `src/lib/components/SettingsModal.svelte` (Sight section)
- `src/lib/sight/engine.ts` (V4 → false, V5 → true)
- `src/routes/+layout.svelte` (dock button label via $t())

**Verification clause:**
- `npm run check` PASS.
- Manual: switch UI to Spanish/German/Arabic, open Sight v5, verify chrome translates.
- All 15 locale JSON files parse-valid (no orphan/duplicate keys).
- Help topic + UM chapter discoverable on disk (PJ-049 acknowledged as separate concern — in-app help viewer is a future MIG).
- NSIS build mtime updated; v4 dock button gone; v5 dock button present.

**Estimated effort:** 2-3 days.

---

## §4 · Boss-Test Gate (after §6 lands)

Per the Testing Instructions Rule, this gate is articulated as a tutorial. Eisa hasn't seen Sight v5 before; every step has pre-state, action, post-state.

### Stage 0 — Verify the right binary is running

**Pre-state:** Eisa has just installed the new build.
**Action:** Open Constellation. Look at the dock (left rail).
**Expected post-state:** A Sight v5 button visible in the dock (NOT v4). Hover the dock button — tooltip should say "Sight v5".
**Failure cue:** If the tooltip says "Sight v4" or there's no Sight button, the wrong binary is installed — STOP and reinstall.

### Stage 1 — Open Sight v5 and read the dome (the ~5-second comprehension test)

**Pre-state:** Constellation is open on the trial Universe; you're looking at the file tree.
**Action:** Click the Sight v5 dock button.
**Expected post-state:** A full-screen circular dome appears. You see:
- 8 concentric rings (the strata bands) — labeled L1 Datum at the rim, L8 Worldview at the center, with intermediate labels.
- A 12-month calendar wraps the outside of the dome (current month subtly tinted).
- Stars (small dots) populate the dome at varying radial positions and angles.
- A toggle bar at the top of the dome with 7 buttons: R · L · T · C · S · A · P (Regions / Link Types / Time / Confidence / Stages / Acts / Provenance).
- A 3-button scope toggle (U / L / F = Universe / Library / Folder) somewhere in the chrome.
- A right-hand legend explains the encodings (size = maturity, brightness = confidence, etc.).

**The ~5-second test:** Without reading the legend, can you say what mode is active and roughly what kind of universe shape you're looking at? If yes — Layer 1 works. If no — flag what was confusing.

**Failure cue:** Dome doesn't render at all → render pipeline broken. Dome renders but no stars → §2 layout cache or §5 star rendering broken. Some stars but obviously wrong positions → §3 dome geometry or §4 mode dispatch broken.

### Stage 2 — Toggle modes and verify spatial memory survives

**Pre-state:** Sight v5 open; Regions (R) mode active.
**Action:** Pick a star you can identify (e.g., a bright canonical star at L7 Perspective). Note its position. Click L → T → C → S → A → P in sequence, watching the same star.
**Expected post-state:** That star sits at the same stratum band (same radial distance from center) in every mode. Only its angular position changes as the wedges re-cut. The 600 ms ease animation interpolates the angular move.
**Failure cue:** If the star jumps to a different stratum band when modes toggle, invariant I-11 (spatial memory) is broken — STOP and report.

### Stage 3 — Hover and select stars

**Pre-state:** Sight v5 open in any mode.
**Action:** Hover any star.
**Expected post-state:** A tooltip appears near the cursor showing the note's title, stratum, maturity, and stage. The faint connector lines from this star to its neighbors brighten to a clearly visible alpha; other lines stay faint. Tooltip and brightening should appear immediately (no perceptible lag).

**Action:** Click the star.
**Expected post-state:** A right-hand side panel slides in showing the note's full detail — title, strata badge, maturity, stage, sources (if classified), top 5 incident links, and an "Open in editor" button. The connector lines stay brightened (selection is persistent).

**Action:** Click the "Open in editor" button.
**Expected post-state:** The note opens in NotePane in the main content area (or as a new tab depending on existing app behavior). Sight v5 doesn't close — handoff is forward-only.

**Failure cue:** Tooltip lags >100 ms = perf issue. Side panel doesn't slide in = §5 broken. Open-in-editor opens the wrong note = §5 side-panel-handler broken.

### Stage 4 — Esc and clear selection

**Pre-state:** A star is selected; side panel is open.
**Action:** Press Esc.
**Expected post-state:** Selection clears, side panel slides out, all connector lines return to faint baseline.

**Action:** Click a star, then click empty dome background.
**Expected post-state:** Same as Esc — selection clears.

### Stage 5 — Mode P empty-state behavior

**Pre-state:** Sight v5 open; switch to mode P (Provenance).
**Expected post-state:** Since most of the trial Universe isn't yet classified, you should see a giant **Unsourced** wedge dominating the dome, with a CTA reading "Classify some notes via Source Review →". Click the CTA.
**Expected:** The right sidebar opens the Source Review panel (the existing CECE surface).

**Failure cue:** If mode P shows a normal wedge distribution with the few classified notes only and no Unsourced wedge, D-V6.α isn't honored. If the CTA doesn't open Source Review, the handoff is broken.

### Stage 6 — Scope toggle

**Pre-state:** Sight v5 open; Universe scope (U) active.
**Action:** In the file tree (or wherever you select a Library), pick a Library. Switch to Sight v5. Click L (Library) in the scope toggle.
**Expected post-state:** Only stars from that Library remain visible; the dome re-fits the smaller note set. Mode toggles still work; spatial memory still preserved within the filtered set.

**Action:** Click F (Folder) — should switch to the active Folder's notes if a Folder is selected; if no Folder is selected, button is dimmed.

**Failure cue:** Scope toggle has no effect = §4 scope filter broken. Star count doesn't visibly drop = filter not wired.

### Stage 7 — Close and reopen

**Pre-state:** Sight v5 open with some mode + scope active.
**Action:** Close Sight v5 (close button or back to file tree).
**Expected post-state:** Returns to whatever was on screen before Sight opened (file tree, NotePane, etc.).
**Action:** Reopen Sight v5 from the dock.
**Expected post-state:** Sight v5 reopens with the last-used mode + scope restored (per `appSettings.sight.lastMode` + `lastScope`).

**Bonus check:** Quit Constellation entirely and relaunch. Open Sight v5. Same persistence should hold across app restart.

---

## §5 · What was deferred

These were explicitly out of scope per Concept Paper v3.1 §12 + Eisa's MIG-024 authorization:

- **Layer 2 — diagnostic computations** → MIG-025
- **Layer 3 — recommendation engine** (Qwen3-1.7B + GBNF wiring; V3-§7.b) → MIG-026
- **Layer 4 — coaching mode** → MIG-027
- **`lenses.rs::apply_lens` deletion** → cleanup MIG
- **v4 component deletion from disk** → cleanup MIG (after Eisa confirms v5 stable)
- **Comprehensive help topic + UM chapter** → MIG-027 (Layer 4 ship)
- **In-app help viewer** → PJ-049 (separate)
- **Mock B1 SVG further evolution** → PJ-051 (housekeeping)
- **MIG-022 §B.5 + §B.6** (Sight v3 overlay UI + its i18n) → permanently contradicted; not on any roadmap
- **MIG-022 §N P2/P3 polish** (drift findings F2-F8) → future cleanup MIG

---

## §6 · What's next

After Eisa approves this Plan: **Phase 3 Build cascades autonomously through §0 → §1 → §2 → §3 → §4 → §5 → §6.** Each cluster commits when its verification clause passes; session log entry per commit.

**Stops only at:**
1. **Boss-Test Gate after §6 lands** — Eisa runs the 7-stage tutorial above; I fix what he finds inline; we re-run failed stages until PASS.
2. **Genuine architectural surprise** — an unmapped invariant, a contract break, a perf cliff. I stop and surface it. No ambush.

After Boss-Test PASS: **Phase 4 Audit** — three parallel agents (invariants / drift / migration-path) per the MIG-022 §N pattern. MIG-024 close-out commit + orientation v2.01 → v2.02 (or v2.10 — TBD at close-out per how MIG-022 §N close lands) + Concept Paper v3.1 → v3.2 (folding D-V3 scope-toggle expansion into Concept Paper §6.1) + Pending Jobs v1.12.

Then MIG-025 (Layer 2 diagnostic) opens.

---

**End of MIG-024 Plan.** Awaiting Eisa's single approval read.
