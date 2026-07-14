# SS Three-Zone Cockpit /migration — Phase 2 PLAN (+ design-inspection amendments)

**Date:** 2026-07-13 · **Workflow:** `wf_4f1db1d8-5b3` (planner + 4-lens design-stage safety inspection) · **Scope:** Boss-ruled Option C + conservative cut.
**Status:** **Boss-APPROVED 2026-07-13 (as amended).** A9 resolved: **keep lists browsable** — surviving SS list clicks stay SS-LOCAL (no open-in-main re-point; §2's re-point clause is struck; the RO editor halves are still cut). Build cascade in progress.

---

# MIG — SS Three-Zone Cockpit — Phase 2 PLAN

**Working on:** the Second Screen Knowledge Cockpit — the conservative cut + the HEALTH and WHERE-lite zones (Boss-ruled Option C, 2026-07-13).
**Concept (the horse):** the SS is a read-only Presenter Display answering three questions about the moment of work — where am I (WHERE), how healthy is the corpus's link fabric (HEALTH), what to engage next (DECISION, already shipped as the three lenses).
**Grounding:** `docs/SS-Cockpit-Migration-ARCHITECT.md` (INV-1..INV-11 bind every step) · `docs/concept-papers/PJ-068-v3-SS-Honest-Audit-2026-07-13.md` · `docs/concept-papers/PJ-068-v2-Second-Screen-Knowledge-Cockpit-Concept-Paper.md`.
**Out of scope (Boss-ruled, do not touch):** the Estimation Map / time map; any tasks write-time index; the split-view companion's fate; the Sky View companion trio + the `handleSkyNodeClick` short-circuit (+layout.svelte:5051-5061); the dashboard/tree companion's fate; the `screen:open-note` channel's overall fate; the workspace `screen:state-request/response` exchange (BOTH ends stay in place this migration — the 2s-stall trap never opens).

**Standing gates used below** (named once, referenced per step):
- **G-svelte** = `npm run check` (svelte-check) clean.
- **G-vitest** = `npx vitest run` green.
- **G-cargo** = `cd src-tauri && cargo test` green + `cargo build --release` compiles.
- **G-binary** = `npm run build` FIRST, grep `build/` for a new string from the step, THEN `cargo build --release`; Boss-test binary mtime checked (Stage-0 rule).
- **G-inspect** = diff-scoped `safety-inspection` workflow over the step's changed files (mandatory for every step touching a write path, index/cache, IPC contract, or editor/SS lifecycle; exempt for i18n-only and docs-only steps).
- **G-Boss** = the Boss builds-and-passes gate: EVERY commit is preceded by a Boss pass (top standing order 2026-07-13, no exceptions). Per-step this is a short observable check named in the step; the three FULL staged tutorials fall at §5 (after PART A), §13 (after HEALTH), and before §19 (close).

---

## §0 — Pre-flight (process, no code)

- **Function-in-hand line** written; **Predecessor → Replacement entries** into `lab/reports/SESSION-LOG-2026-07-14.md` for every PART-A cut target (Predecessor Lookup Rule), verified against orientation — one entry each for: dashboard-note RO copy, dashboard-tag RO half, index-term RO half, index-compare RO half, the 9 stub facets + tab bar, the `!COCKPIT_ENABLED` clone, the OrgChart clone, the Map companion, the lens toggle (Predecessor: nested in `.ck-tabs`; Replacement: cockpit header chrome — same component, Boss-approved via this plan).
- **Baseline measurement** on the 7,600+ note Universe (INV-7): cold boot time; 10-char type-burst in NotePane with SS open (zero SS IPC — console trace); SS open→painted time; `recompute_link_stats_cache` duration (log timestamp); workspace-save duration with SS open.
- Commit: session-log only.

---

## PART A — The conservative cut

**Rollback posture:** each step is a pure-deletion commit, individually `git revert`-able; PART A as a whole reverts as a contiguous range.

### §1 — Cut the dead code: `!COCKPIT_ENABLED` clone + OrgChart clone + Map companion + dead SS-local dashboard state

**Files/what:**
- `src/lib/components/SecondScreenPage.svelte`: delete dispatch #8b (:1563-1690, the ep clone — sole renderer of all ep* state); `loadEditorPanelsData` (:472-542) + ep* state (:106-115) + its three call sites (u18 :1099, u2's arm inside :816-824, `onNoteMutation` :155); OrgChart branch (:1840-1855) + import (:40); Map companion branch (:1339-1425) + `mapCompanion*` state (:99-102) + u17 listener (:1036-1067) + `ConstellationMap` import (:38); the never-rendered SS-local dashboard state + `loadDashboardData` + `refreshRecentLists` + the **2-second poll** (:204-293, :792-803) + its call sites in u5/u5b (:893-907) and u11 (:980-984); the six unused write-path imports (:13-15 — keep `wasRecentlyWritten`).
- `src/lib/secondScreen.ts`: delete `MapCompanionData`/`emitMapCompanion`/`onMapCompanion` (:361-376).
- `src/routes/+layout.svelte`: delete the 4 `emitMapCompanion` sender branches (:7522, 7528, 7534, 7541) — **both ends of the map channel in this one commit**.
- `src/lib/cockpitFlag.ts:1-9`: repoint `COCKPIT_ENABLED`'s doc comment — it is now the **three-zone rollback toggle** for PARTS D/E (symbol name unchanged; retired at §19).

**Grep-by-name lifeline mitigation (Architect risk #2, INV-2) — the explicit KEEP list, verified present after the diff:** the `screen:editor-panels` channel (secondScreen.ts:392-398), `editorPanelsActive`/`editorPanelsData` state, the +layout.svelte:572 sender $effect + the :3400/:4966 `active:false` sends, u18's payload-adopt core (:1084-1101 minus the loader call), u2's content re-read (:816-824 minus the loader call) and `cockpitReload++` (:841), uCascade's bump (:853). "Editor panels" is the cockpit's misnamed lifeline — nothing matching that name is deleted wholesale.

**Verification:** G-svelte, G-vitest, G-inspect, G-binary. Behavior identical (all deleted code was flag-dead, unreachable, or rendered nowhere — Mapper A). G-Boss: open SS on the big Universe → cockpit renders; switch tabs in main → cockpit follows (INV-2); type-burst → zero SS IPC (INV-5); no 2s poll in the SS console. Protects INV-2, INV-3, INV-5, INV-6.

### §2 — Cut the four read-only note copies; re-point surviving list clicks; shrink the adopt primitive

**Files/what:**
- `SecondScreenPage.svelte`: delete dispatch #2 whole (:1177-1188 + `dashboardNoteTab` :86 + u13 receiver :986); delete the RO editor **halves only** of dispatch #3 (:1226 + `dashboardSelectedNote`), #4 (:1273 + `indexSelectedNote`), #5 (:1329) — the tag LIST half and the Index term/compare LIST halves stay (fate ruled later).
- Re-point the surviving lists' note-clicks to `sendNoteToMain` (`screen:open-in-main`, the existing INV-1-clean navigate vocabulary) so a click always does something — no new dead-ends.
- `adoptFreshDiskIntoSS`/`adoptCompanionTab` (:734-775): companion list shrinks to `peekTab` only (the masked Sky View peek survives untouched this migration); `externalChangeNoteModel`/`SINGLE_OWNERSHIP` imports stay (a NoteEditor mount survives — the all-or-consistent rule from Mapper C).
- `src/routes/+layout.svelte`: delete `emitDashboardOpenNote` (:8311) so the home DashboardView's `onNoteToScreen` falls through to `openNoteTab` in main (Mapper C:135); `emitDashboardTagSelected`/`emitIndexTermSelected`/`emitIndexCompare` senders STAY (lists live).
- `src/lib/secondScreen.ts`: delete only the dashboard-open-note vocabulary; tag/term/compare vocab stays.
- **Workspace-stall trap (Architect risk #1, INV-8):** this step deliberately does NOT touch the `screen:state-request/response` exchange or SS `openTabs` — both ends stay; binding rule recorded: if any future step removes the SS responder (:910-924), the main-side exchange (+layout:8720-8742) goes in the SAME commit.

**Verification:** G-svelte, G-vitest, G-inspect, G-binary. G-Boss: home-dashboard note click with SS open now opens in MAIN; tag list / term list / compare clicks navigate main via open-in-main; **workspace save with SS open completes instantly** (explicit stopwatch check — INV-8); rename a note in main → cockpit repaints (cascade path, INV-6); no SS write IPC anywhere (INV-1). Protects INV-1, INV-5, INV-6, INV-8.

### §3 — Cut the 9 stub facets + facet tab bar; re-home the lens toggle (INV-10)

**Files/what:** `src/lib/components/SecondScreenCockpit.svelte`: delete `FACETS`/`activeTab`/`facetLabel` (:123-139), the `.ck-tabs` bar (:174-188), the "wired in the next pass" stub (:200-205). KEEP: `DIALS` (:116-119), pin logic (:56-62), `fetchLinks` + path/nonce guard (:73-111), the four lens mounts (:190-199). **Re-home the `.ck-lens` toggle group into the cockpit header chrome** (beside the Pin/Follow dial) — it must ship in the same commit as the bar deletion, never lost (Architect risk #5). `settings.panels.panel*` i18n keys are NOT touched — they are main-window Settings rows (SettingsModal.svelte:2118-2129; INV-9, INV-3).

**Verification:** G-svelte, G-vitest, G-binary. G-Boss: Butterfly/Ledger/Orrery toggle visible in the new home and switches lenses end-to-end (SS request → main writes `noteGraphStyle` at +layout:3404 → broadcast → SS re-render — the single-writer loop, INV-6-of-Architect/settings); Settings → Panels rows intact in main. Protects INV-3, INV-9, INV-10 (discharged), settings single-writer.

### §4 — Fix-in-pass: Index note-click dead-end + per-key i18n prune + the missing `comparingTerms` key

**Files/what:**
- `+layout.svelte:6466-6479`: delete the SS branch of the Index note-click (it sends `screen:open-note` into the no-op receiver and returns — a live silent-failure). Post-fix: the note opens in MAIN. The `screen:open-note` channel itself and its other senders stay (fate = open ruling, PJ-ledger item).
- i18n ×15 (`ar de en es fa fr he hi ja ko pt ru tr ur zh`): prune ONLY keys with zero remaining consumers after a whole-`src/` grep per key — never by block, never from Mapper lists alone (Mapper C's die-with-cut list assumed a wider cut; e.g. `secondScreen.splitCompanion` STAYS because the split companion survives). Known-KEEP regardless: the whole `secondScreen.dashboard.*` subtree (main-window home dashboard, +layout:8304), `settings.panels.*`, `title/sendToScreen/loading/linked/selectNote/detailEmpty`, the `cockpit.*` block. Known already-dead candidates: `detail, grid, graph, graphView, linkedOn, linkedOff, skyviewMode, backlinksFor`.
- ADD the missing `secondScreen.comparingTerms` key ×15 (referenced at SecondScreenPage:1290, absent from en.json, surviving surface — fix-what-we-discover).

**Verification:** G-svelte, G-inspect (the dead-end fix touches a nav path), G-binary. G-Boss: with SS open, click a note in the Index panel → it opens in main (was: nothing anywhere); switch app language to Arabic → compare view shows the translated "comparing terms" string, RTL intact. Protects INV-9.

### §5 — BOSS LIVE-TEST STAGE 1 (full staged tutorial) + PART-A docs touch

Full tutorial-style regression of the cut (Testing Instructions Rule; staged, one stage sent at a time): SS open/close lifecycle + monitor gating (INV-3/INV-4 of Architect list); focus-follow + Pin dial; all three lenses + re-homed toggle; list-half navigation; workspace save/restore with SS open (INV-8); second-screen items of the Editor-Surface Gate Checklist item 7. After the Boss pass: session log, orientation-doc touch (Move-1 shipped), milestone tag `milestone/ss-cut`.

---

## PART B — The Rust cache keys (additive; no new tables, no triggers, no backfill)

**Rollback posture:** keys are additive rows in the existing `link_stats_cache` (search.rs:3593-3597) — old frontends ignore them; reverting B deletes the writer code and the stale rows are simply overwritten/ignored on the next recompute. Migration path: fresh DB → boot-if-empty (cache.rs:1559) triggers recompute; keys absent until then → tiles show "computing…" (honest state). Recompute is idempotent and rewritten wholesale — a mid-recompute interrupt self-heals on the next run.

**PJ-066 canon binding all of PART B (Architect risk #3, INV-11):** the two new queries run INSIDE the existing background `recompute_link_stats_cache` (search.rs:7919-7977) on its existing connection pattern — never a new sync command, never holding the writer lock for reads elsewhere; snapshot reads stay via `with_read_conn`; NO `COALESCE(col,expr)` in any WHERE on `note_meta` (the 22s full-scan landmine).

### §6 — Key 13: the contradiction PAIR-LIST

**Files/what:** `src-tauri/src/search.rs` — inside `recompute_link_stats_cache`, after the ccs registers (~:7732): one indexed query — `note_links WHERE link_type='contradicts' AND status='active'` via `idx_link_type` (:3478), joined to `note_meta` for source/target names+paths, top-50 by weight + total count — serialized as a new cache key (e.g. `contradiction_pairs`). Extend the `constellation_knowledge_health_snapshot` payload (:8080) to carry it, PLUS a `scope: "active-universe"` field (the Architect's required scope statement — recompute opens only the active universe's DB, :7994-7995).

**Verification:** G-cargo (new unit test: seed contradicts links → recompute → snapshot carries the named pairs + count), G-inspect. Recompute duration re-measured on the 7,600-note Universe vs §0 baseline — delta within noise (indexed query). `kh-snapshot-ready` (:8011) still emits. G-Boss: boot the big Universe + open SS + type-burst — no perceptible change (INV-7). Protects INV-7, INV-11.

### §7 — Key 14: orphan/fragile corpus counts

**Files/what:** same function — an index-only scan on `idx_note_meta_incoming_wc` (search.rs:3446-3448; orphan `incoming_count=0`, fragile `>=5`, MIG-084 §F.2 semantics) cached as `{orphans, fragile}`. The scan runs ONLY in the background recompute — never on SS open (the Architect's explicit no-walks-on-open ruling for this count).

**Verification:** G-cargo (unit test: seeded orphan/fragile notes → counts in snapshot), G-inspect, recompute-timing re-check. G-Boss: same boot/type/SS-open smoke as §6. Protects INV-7, INV-11.

### §8 — Fix-in-pass: retire the SS `collect_library_notes` filesystem walk

**Files/what:** `SecondScreenPage.svelte:608-628` (`loadAllData`) currently calls `collect_library_notes` per library (libraries.rs:5281 → recursive `read_dir` + 1KB read per file) to feed the cockpit's `resolveTarget` (SecondScreenCockpit.svelte:47-52). Replace the source with a `note_meta`-backed read: one async command over `with_read_conn` returning `(path, name, library)` for the universe (indexed, `idx_note_name_lower` :3400) — reuse an existing query surface if one exists (checked at build), else one new small command. Same `loadAllData` shape, source swapped; the 3s-debounced refresh wiring unchanged.

**Verification:** G-cargo, G-svelte, G-inspect. SS-open time re-measured vs §0 (must improve or hold; the fs walk is gone — INV-7). G-Boss: open a note with outgoing wikilinks → Butterfly resolves and draws the targets; create a new note in main → within ~3s the lens can resolve it (freshness wiring intact, INV-6). Protects INV-5, INV-6, INV-7, INV-11.

---

## PART C — Art-Director design pass (workflow, no code)

### §9 — The zones' design spec (Boss-ruling gate)

Run the Art Director & Team multi-agent workflow (the PJ-088 pattern; the Art Director owns UX/UI per the 2026-07-10 ruling) with a written brief containing:
- **The exact data contract** — HEALTH: the 12 existing aggregates (lifecycle, ccs cooling/contested/tiers/retired/living/load-bearing, by_type/by_confidence, stats) + the two §6/§7 keys; WHERE: the focus payload fields + O(1) `note_meta`/`sky_nodes` PK facts. Nothing else exists; the design may not assume more.
- **The forbidden sources** — `constellation_map_universe` (map.rs:463), `cache_boot_snapshot_sky` (cache.rs:849), `detect_tensions` per-library from SS, any effective-weight corpus scan (the 90-day idle proxy is the shipped vocabulary), any fs walk.
- **The laws** — read-only always (INV-1; clicks navigate main via `open-in-main` only); fixed spatial zones (the v2 §3 NASA/ISA-101 principle); Form-Aligns-To-Purpose (no filler dimensions); honest "computing…" empty states; the **active-universe scope label** on the HEALTH board; glanceable-across-the-room; theme-aware; RTL ×15; the dial + lens toggle already occupy the cockpit header.
- **Deliverable:** `docs/SS-Zones-Design-SPEC.md` — zone layout/arrangement, HEALTH tile taxonomy + the contradiction pair-list presentation, WHERE locator strip content, interaction map, empty/idle states, the complete new-string inventory (the i18n source of truth for §12/§15).

**Gate:** Boss approves the spec BEFORE any D/E code. Commit: the spec doc only.

---

## PART D — The HEALTH zone build (behind the repointed `COCKPIT_ENABLED` flag)

**Rollback posture:** flag off → the cockpit renders the PART-A end-state (lenses-only). One-line rollback until §19.

### §10 — HealthBoard data plumbing

**Files/what:** new `src/lib/components/cockpit/HealthBoard.svelte` (final path per spec); fetches `constellation_knowledge_health_snapshot` + `constellation_ccs_snapshot` (one async invoke each — both already async/read-conn) keyed on SS-open + the 3s-debounced library-changed reload; subscribes to `kh-snapshot-ready` (search.rs:8011) for refresh — **no polling, no per-focus fetch, zero keystroke IPC**. Flag-gated mount in the cockpit per the approved layout. Listener unlisten in `onDestroy` (Rule 4).

**Verification:** G-svelte, G-vitest, G-inspect, G-binary. Type-burst in main with SS open → zero SS IPC (console trace — INV-5); SS open triggers exactly two snapshot invokes (INV-7). G-Boss: board region renders with live aggregate numbers on the big Universe. Protects INV-1, INV-5, INV-7.

### §11 — Tiles, pair-list, navigation, honest states

**Files/what:** render per spec: lifecycle/cooling/contested/tiers/orphan/fragile tiles + the named contradiction pair rows. Pair/tile click-throughs navigate via `sendNoteToMain` (`screen:open-in-main`) ONLY (INV-1). "Computing…" state whenever a key is absent (pre-first-recompute). The **active-universe scope label** rendered from the §6 `scope` field (new INV-12).

**Verification:** G-svelte, G-binary, G-inspect. **Migration-path test:** clear the cache rows → boot → tiles show "computing…" → `kh-snapshot-ready` fires → tiles fill without reopening the SS. G-Boss: click a contradiction pair → the note opens in MAIN; nothing on the board edits anything. Protects INV-1, INV-7, INV-12.

### §12 — HEALTH i18n ×15 + RTL

All spec-inventory strings into all 15 locale files; `detectDir`/`dir` attributes per the SecondScreenPage:161 pattern; Arabic visual pass (labels, number alignment, chevrons flipped). **Verification:** G-svelte; per-locale key-presence check; G-Boss: switch to العربية → the whole board reads natively, RTL-correct (full-localization standing order). Protects INV-9.

### §13 — BOSS LIVE-TEST STAGE 2 (full staged tutorial) + measurement

Tutorial: what the HEALTH board is (the whole-corpus tension and living-link health view the main window cannot show), then click-by-click through every tile, the pair-list jump, the computing state (fresh-cache demo), the scope label, and the language switch. Re-run the §0 measurement suite — boot/typing/SS-open/recompute deltas logged (INV-7 verdict recorded pre-commit). Session log + orientation touch after the pass.

---

## PART E — The WHERE-lite zone build (same flag)

### §14 — Locator data + WhereStrip component

**Files/what:** new `src/lib/components/cockpit/WhereStrip.svelte` per spec. Data: the focus payload already carried on `screen:editor-panels` (name/path/library/libraryColor — the same wire, INV-2) + folder-chain derived purely from the path string (zero IPC) + one O(1) async command for per-note facts (`note_meta` PK: word_count/created/modified/in-out counts; `sky_nodes` PK: stratum/maturity) — new command only if the fields aren't already in hand from `get_note_review_status`/the payload (decided at build; prefer zero new IPC; if new: async + `with_read_conn`, INV-11). Fetch keyed on path-change + `reloadNonce` exactly like `fetchLinks` (SecondScreenCockpit:97-110) — zero keystroke IPC. Idle state when no focus. **Forbidden:** `constellation_map_universe`, the sky snapshot, the dead `screen:universe-switched` channel (zero senders — Mapper A; universe/library labels derive per focus payload + the debounced library-changed reload; if the approved spec demands a live universe chip beyond that, it is filed as a PJ, not built on the dead channel).

**Verification:** G-svelte, G-cargo (if a new command), G-inspect, G-binary. Type-burst → zero SS IPC; tab-switch → locator updates in one hop (INV-4-of-Architect latency). G-Boss: open notes in different libraries/folders → the strip shows the right chain, library color, stratum/maturity facts; Pin the dial → the locator pins with it. Protects INV-1, INV-2, INV-5, INV-7, INV-11.

### §15 — WHERE i18n ×15 + RTL

Same discipline as §12. **Verification:** G-svelte; Arabic/RTL visual pass; G-Boss quick check. Protects INV-9.

---

## PART F — Docs, audit, measurement, close

### §16 — Help + User Manual (×15)

Rewrite `docs/help.uConstellation.World/Second Screen/Second Screen.md`: delete/replace the stale sections inventoried by Mapper C:156 — §Editor Panels Companion (37-51), the mode table (54-66, retired-Navigator rows), **§Note Editing in Second Screen (147-156 — the false full-editing claims)**, §Dashboard Interaction (161-176), §Navigator (180-183), §Map (230-240), §Workspace (243-245), the facet-tabs line (266); update §Split (208-218) and §Index (221-227) to the surviving list-half behavior; add HEALTH + WHERE sections (read-only, scope label, computing state). Update `docs/User Manual.md` §9 + the Index "Second Screen Integration" section in all 15 manuals (`docs/help.{lang}/`). **Verification:** no manual claims the SS edits anything, in any language. Protects INV-9 + closes the false-claims defect named in Architect §1.5.

### §17 — Measurement close-out (INV-7)

Re-run the full §0 suite on the 7,600+ note Universe; before/after table into the session log: boot, typing latency, SS-open time, SS-open IPC count, recompute duration, workspace-save duration. **Pass criterion: no regression on any row; SS-open improves (the §1 poll + §8 walk are gone).** A regression blocks §19 until fixed.

### §18 — Phase-4 Audit + per-cycle safety sweep

Three parallel agents: (1) **invariant checker** — INV-1..INV-12 verified against the final diff; (2) **drift checker** (LL-023) — new guards/flags the system doesn't know about (the repointed flag, the "computing…" states, the re-pointed list clicks); (3) **migration-path checker** — first-boot empty cache, mid-recompute interrupt, flag-off rollback render, old-workspace `secondScreen` blob loads silently (INV-8), locale fallback for the new keys. PLUS the whole-app `safety-inspection` sweep (per-cycle cadence — this migration's close is the cycle boundary). **Every confirmed finding fixed before §19 (WA#6).** Run `/simplify` over the migration's final diff.

### §19 — BOSS LIVE-TEST STAGE 3 (close tutorial) → flag retirement → close

Stage-3 full tutorial: WHERE strip walk-through + whole-cockpit regression (dial, lenses, HEALTH, navigation, language switch, SS lifecycle, workspace save/restore) + the SS items of the Editor-Surface Gate Checklist. After the Boss pass, the close commit: retire the repointed flag (delete the else-paths — rollback thereafter is `git revert`, per the Architect's retire-at-Move-2-close), orientation vX bump, PJ-ledger reconciliation FIRST (SO#9 — close shipped items incl. PJ-068 arc; file surfaced items: `screen:open-note` fate, split-companion fate, Sky-View-companion fate, universe-switch signal hole, dashboard/tree companion fate, federated-cache scope), session log, MoCh, help-sync check, milestone tag `milestone/ss-three-zone` + ZIP backup.

---

## Architect-flagged risks → mitigation map

| Risk | Mitigation (step) |
|---|---|
| Workspace-save 2s stall | Exchange untouched this migration (both ends stay — §2); binding both-ends-one-commit rule recorded for any future responder cut; explicit save-stopwatch check in §2's Boss gate (INV-8). |
| Grep-by-name kills the cockpit lifeline | §1's written KEEP list (channel, editorPanelsActive/Data, +layout:572 sender, u18 core, cockpitReload bumps 841/853) verified present post-diff; follow-check in §1's Boss gate (INV-2). |
| Recompute touch (writer-adjacent) | §6/§7 run inside the existing background job, PJ-066 canon (async, `with_read_conn` reads, no COALESCE-in-WHERE, never the writer lock); recompute timing measured per step (INV-11, INV-7). |
| i18n key deletions | §4: per-key whole-`src/` consumer grep, never block deletion; named KEEP sets (`settings.panels.*`, `secondScreen.dashboard.*`); Mapper lists treated as candidates only (INV-9). |
| Lens toggle lost with the tab bar | §3 re-homes the toggle in the SAME commit as the bar deletion; end-to-end set-lens loop in §3's Boss gate (INV-10). |

## Rollback plan per part

- **PART A:** pure-deletion commits §1-§4, each individually revertable; range-revert restores the pre-cut SS wholesale.
- **PART B:** additive cache keys — old builds ignore them; revert deletes the writer, rows self-heal on next recompute; no schema, no backfill, no migration state.
- **PARTS D/E:** behind the repointed `COCKPIT_ENABLED` (three-zone rollback toggle); flag off = the PART-A lenses-only cockpit; flag retired only at §19 after the Stage-3 Boss pass.

## Invariants (carried from the Architect doc; bind the Audit)

- INV-1 SS never writes; mutations forward via screen:request-note-action; settings single-writer via screen:set-lens.
- INV-2 Cockpit lifeline intact: screen:editor-panels channel, editorPanelsActive/Data, +layout.svelte:572 sender, cockpitReload bumps (841/853).
- INV-3 Main-window right sidebar, settings.panels.* keys, panel choreography untouched.
- INV-4 Rust window lifecycle byte-identical (lib.rs:660-693; two-monitor gate; handshake).
- INV-5 Zero SS IPC per main-window keystroke (path+nonce guard, SecondScreenCockpit.svelte:97-110). *(Extends to the HealthBoard and WhereStrip fetches — §10/§14.)*
- INV-6 G3 freshness: lenses repaint on note-saved + cascade:rewrote; echo guard kept.
- INV-7 Rule 8: zones read write-time-derived data; no whole-universe walk on SS open; measure boot/typing/IPC on the 7,600-note Universe pre-commit. *(Measured at §0, §6-§8, §13, §17.)*
- INV-8 Legacy workspace secondScreen blobs load silently forever; saves never stall. *(This migration leaves both exchange ends in place — the trap never opens.)*
- INV-9 i18n ×15 + RTL; locale deletions consumer-checked per key (secondScreen.dashboard.*, settings.panels.* are main-window-shared).
- INV-10 Lens toggle (Boss-validated 2026-07-11) re-homed, never lost with the tab bar. *(Discharged at §3 — post-state: the toggle lives in the cockpit header chrome.)*
- INV-11 New/surviving IPC: async + read-conn; no COALESCE-in-WHERE on note_meta.
- **INV-12 (new, from the Architect's required scope statement):** the HEALTH board always displays its honest scope — link_stats_cache is active-universe-only (search.rs:7994) — until a later migration federates the cache per "It is ONE universe."

**Boss-test stage placement:** short observable G-Boss gate before EVERY commit (top standing order); full staged tutorials at §5 (after the cut), §13 (after HEALTH), §19 (close, incl. WHERE). Estimated span: 3-4 sessions (matches the Architect's Option-C estimate).

---

# BINDING AMENDMENTS (from the design-stage safety inspection — apply during Build; deduped from 21 confirmed hazards)

**A1 — The flag wrapper (HIGH; DS1-02 = H2 = I4-H1).** §1 must ALSO delete the `{#if COCKPIT_ENABLED}` wrapper at `SecondScreenPage.svelte:1553` (+ its `{/if}`) — the cockpit mounts unconditionally in the `editorPanelsActive` branch. The repointed flag gates ONLY the HealthBoard (§10) and WhereStrip (§14) mounts INSIDE the cockpit. §18's migration-path check adds: flag-off → the PART-A lenses-only cockpit renders (never a blank SS).

**A2 — Cache-key rollback poison (HIGH; DS1-01 = I4-H2).** A pre-B hardening commit (landed BEFORE §6, OUTSIDE PART-B's revert range): `recompute_link_stats_cache` ends with `DELETE FROM link_stats_cache WHERE stat_key NOT IN (<known-key list>)` (or max_age computed over the reader-consumed key set only) — so a reverted/retired key self-heals instead of poisoning `max_age` into a permanent recompute loop. PART-B's rollback claim is corrected accordingly.

**A3 — Federation scope (HIGH; DS1-04 = H1).** §8's `collect_library_notes` replacement MUST span the federation: active `note_meta` UNION ALL each attached `cuN.note_meta` (the proven ATTACH-reader pattern), falling back to active-only when `federation.ready == false`. Verification adds a federated clause (a cUniverse wikilink target still resolves in the lenses). "It is ONE universe" (2026-07-05) binds `resolveTarget`.

**A4 — New keys never gate readiness (MED; DS1-05 = PERF-2 = I4-H3).** The two new keys are payload-optional, NEVER added to `KH_CACHE_KEYS`/`CCS_CACHE_KEYS` (the MIG-074 precedent, search.rs:8119-8134). On finding a new key absent, the snapshot command calls `spawn_kh_cache_recompute(app, false)` while STILL returning `ready:true` with the old aggregates. §11 adds the REAL first-boot-after-update test: cache NON-empty (12 old keys, 2 new absent) → tiles show "computing…", a recompute is actually in flight, main-window KH dashboard unaffected.

**A5 — Contradiction pairs: no INNER JOIN (MED; DS1-03).** The pair-list reads `source_name/source_path/target_name(/target_path)` directly from `note_links` (target_path is NULLABLE — an unresolved target is still a real contradiction); any enrichment join is LEFT JOIN. The §6 unit test seeds a NULL-target_path contradicts row and asserts it appears.

**A6 — §1 surgical ranges (MED; DS1-06 = H3).** Delete ONLY the `loadDashboardData()`/`refreshRecentLists()` CALL EXPRESSIONS — the u5/u5b/u11 listeners, their `loadAllData()` calls, and u11's `mainSidebarMode` assignment STAY. The ep* state deletion range is `:108-115` (editorPanelsTab + the eight panel arrays); `:106-107` (`editorPanelsActive`/`editorPanelsData`) are the INV-2 lifeline — any diff touching those two identifiers outside the KEEP-list call sites FAILS the step. §1's Boss gate adds: create a note in main → the SS status-bar count increments within ~3s.

**A7 — The complete lifeline KEEP list (LOW; DS1-07).** All FIVE `emitEditorPanels` senders are on the KEEP list, verified present after §1 AND §2: `+layout.svelte:572, :3400, :4966, :4977 (SS-open handshake), :5008 (Send-to-Screen)`.

**A8 — HEALTH staleness contract (LOW; DS1-08).** §9's design brief states the staleness contract; the board carries a visible "as of <age>" cue; §10 adds one refresh key — a ≥60s-debounced snapshot re-read driven by the EXISTING `screen:note-saved` broadcast.

**A9 — List-halves click behavior (MED; H4) — BOSS FORK, see the approval question.** Re-pointing surviving list clicks to open-in-main round-trips through the main focus machinery → u18's mode reset REPLACES the list with the cockpit after one click (one-shot lists). Either (a) declare this interim shape explicitly in §2's Boss tutorial, or (b) keep list clicks SS-local (no open-in-main) until the zones supersede the lists.

**A10 — §8 index truth (MED; PERF-1).** Cite/reuse `idx_note_boot_snapshot ON note_meta(name, path, library_name)` (search.rs:3424) — NOT `idx_note_name_lower`. §8's verification adds an `EXPLAIN QUERY PLAN` assertion (covering-index scan, no note_meta row-store scan).

**A11 — Measurement honesty (LOW; PERF-3).** §0 gains ONE sanctioned instrumentation line (a timing eprintln in `kh_cache_recompute_blocking`) as the only code in pre-flight. §10/§13's IPC gate wording: "zero SS invokes DURING the burst window; exactly the known lens refetch (~1.5s debounce + 450ms) after it." §17 attributes the SS-open win to the `loadDashboardData` deletion, not the localStorage poll.

**A12 — resolveTarget O(1) (LOW; PERF-4).** §8 builds a `Map<foldedName, {path, libraryName}>` once per allNotes load; `resolveTarget` becomes `Map.get` (kills the O(N·M) per-paint scan).

**A13 — Help-drift inventory (LOW; I4-H4).** §16 adds `docs/help.uConstellation.World/Index/Index.md` ("## Second Screen" §, lines ~76-81 — the SS-side editor claim) + a repo-wide grep across `docs/User Manual.md` + `docs/help.*/` for SS-editing claims as the executable verification.

**A14 — G-binary for deletion steps (LOW; I4-H5).** Pure-deletion steps define G-binary as an ABSENCE grep with a named marker (§1: `map-companion`/OrgChart chunk absent; §2: the dash-note companion marker; §3: `ck-facet-soon` absent).
