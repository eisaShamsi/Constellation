# Session Log — 2026-07-14

## SS Three-Zone Cockpit /migration — BUILD (Phases 1–2 closed 2026-07-13)

**Function in hand:** the **Second Screen Knowledge Cockpit** — the conservative cut + the HEALTH and WHERE-lite zones (Boss-ruled Option C + conservative cut + A9 keep-lists-browsable; Plan approved as amended, `docs/SS-Cockpit-Migration-PLAN.md`).

**Concept (the horse):** the SS is a read-only Presenter Display answering three questions about the moment of work — WHERE am I, how HEALTHY is the corpus's link fabric, what to engage next (DECISION = the shipped lenses). Never a mirror of the main window.

### §0 — Pre-flight

**Predecessor → Replacement entries (Predecessor Lookup Rule — before any code; verified against orientation v3.48 + the Architect territory map):**

1. **Dashboard-note RO copy** — lives: `SecondScreenPage.svelte` dispatch #2 (:1177-1188, `dashboardNoteTab`, u13 receiver :986); sender `emitDashboardOpenNote` (`+layout.svelte:8311`). → Replacement: **NONE (cut)** — a home-dashboard note click with SS open falls through to `openNoteTab` in MAIN (the existing single-owner path). Boss-ruled (Move 1: read-only note copies).
2. **Dashboard-tag RO editor half** — lives: dispatch #3's `<NoteEditor>` (:1226, `dashboardSelectedNote`). → Replacement: NONE (cut); **the tag LIST half STAYS, clicks stay SS-local (A9)**.
3. **Index-term RO editor half** — lives: dispatch #4's `<NoteEditor>` (:1273, `indexSelectedNote`). → Replacement: NONE (cut); the term LIST half stays, SS-local (A9).
4. **Index-compare RO editor half** — lives: dispatch #5's `<NoteEditor>` (:1329). → Replacement: NONE (cut); the compare LIST halves stay, SS-local (A9).
5. **The 9 stub facets + facet tab bar** — live: `SecondScreenCockpit.svelte` `FACETS`/`activeTab`/`facetLabel` (:123-139), `.ck-tabs` (:174-188), the "wired in the next pass" stub (:200-205). → Replacement: the three ZONES (HEALTH §10 / WHERE §14 / DECISION = the lenses) — same component, Boss-approved via the Plan.
6. **The lens toggle** (Boss-validated 2026-07-11) — lives: nested in the `.ck-tabs` row (:179-187). → Replacement: **re-homed into the cockpit header chrome beside the Pin/Follow dial** (INV-10; same component). Never lost with the tab bar.
7. **The `!COCKPIT_ENABLED` editor-panels clone** — lives: dispatch #8b (:1563-1690) + `loadEditorPanelsData` (:472-542) + ep* panel state (:108-115). → Replacement: NONE (dead code cut). **A1:** the `{#if COCKPIT_ENABLED}` wrapper at :1553 is deleted too — the cockpit mounts unconditionally; the repointed flag gates ONLY the zone mounts. **A6/A7:** `:106-107` (`editorPanelsActive`/`Data`) + all FIVE `emitEditorPanels` senders (`+layout.svelte:572, :3400, :4966, :4977, :5008`) are the INV-2 lifeline — KEEP, verified present after every PART-A diff.
8. **The OrgChart clone** — lives: :1840-1855 + import :40. → Replacement: NONE (dead cut; the main-window OrgChart is the home).
9. **The Map companion** — lives: :1339-1425 + `mapCompanion*` state (:99-102) + u17 (:1036-1067) + `emitMapCompanion` senders (`+layout.svelte:7522/7528/7534/7541`) + vocabulary (`secondScreen.ts:361-376`). → Replacement: NONE (dead cut — Map disabled since MIG-038); **both channel ends deleted in the same commit** (the workspace-stall-class trap).
10. **The SS-local dashboard loaders + the 2s poll** — live: :204-293, :792-803 (never-rendered state). → Replacement: NONE (dead cut). **A6:** only the CALL EXPRESSIONS to `loadDashboardData()`/`refreshRecentLists()` are deleted from u5/u5b/u11 — the listeners, `loadAllData()`, and u11's `mainSidebarMode` assignment STAY.

**Baseline (INV-7).** Durable prior measurements stand as the baseline: cold boot ~17s on the 7,600-note Universe (MIG-079, the SKY read); PJ-066's recompute/connect records. §0 adds the ONE sanctioned instrumentation line (A11): a timing eprintln in `kh_cache_recompute_blocking`, so recompute duration is measured from the next dev run onward and re-checked at §17. Runtime SS-open/typing observations are re-verified live at each staged Boss test on the real Universe.

**Out of scope guardrails re-stated:** the Estimation Map; any tasks index; the split-companion's fate; the Sky View companion trio; the `screen:state-request/response` exchange (BOTH ends stay).

### PART A — the conservative cut (§1–§3) — BUILT, awaiting Stage-1 Boss test

- **§0** `45d20b88` — pre-flight: Predecessor entries + Plan/Architect docs + the A11 timing eprintln.
- **§1** `fbb84c2e` (−747 lines) — the dead code: the `!COCKPIT_ENABLED` ep-clone + the A1 wrapper (the cockpit now mounts UNCONDITIONALLY; `COCKPIT_ENABLED` repointed as the zone rollback toggle, zero consumers until §10), the OrgChart clone, the Map companion (BOTH channel ends in one commit), the never-rendered SS-local dashboard state/loaders/2s-poll (A6: call expressions only — u5/u5b `loadAllData` + u11 `mainSidebarMode` follow KEPT), 15 dead imports, 74 dead CSS rules. KEEP-list verified (A7): the `editorPanelsActive`/`Data` lifeline, all FIVE `emitEditorPanels` senders, both ends of the workspace exchange. Gates: svelte-check 0 (warnings 319→267), vitest 341. **Safety inspection `wf_8b0a5104-6e8` (ran whole-app, 83 agents, 55 confirmed): ZERO in-diff findings.** The sweep's 5 pre-existing APP-KILLERs (incl. 2 NEW: `store.ts:2039` ensure_cid_cn destroys write-ahead recovery on manual reopen; `+layout.svelte:3436` dirty background models never flushed at app close) → to the PJ ledger at the Stage-1 checkpoint for Boss sequencing.
- **§2** `dd576058` (−180 lines) — the four RO note copies cut (A9: lists stay browsable as display-only rows; `screen:dashboard-open-note` deleted both ends; home-dashboard note click always opens in MAIN; adopt primitive → peekTab only; the workspace exchange untouched). Gates: svelte-check 0 (265 warnings), vitest 341.
- **§3** `67a886f5` — the 9 stub facets + `.ck-tabs` bar cut; the lens toggle re-homed to the cockpit header beside the Pin/Follow dial (INV-10); the DECISION zone = the lens graph at full glass. Gates: svelte-check 0, vitest 341.
- **G-binary (A14 absence-grep):** all six deletion markers absent from the fresh `build/` (map-companion, dash-note-companion, ck-facet-soon, "wired in the next pass", screen:dashboard-open-note, screen:map-companion). Release binary building.

### Stage-1 Boss test — ALL PASS (+ one remark → fixed + re-tested PASS)
- **Stage 1 (the clean cockpit / follow+Pin / no-lag / instant workspace save / dashboard-click-to-main): ALL PASS.**
- **Boss remark:** closing ALL main-window tabs left the SS frozen on the last note. **Diagnosed off the code** (the focus-sender $effect at `+layout.svelte:562` only fired when a note was active — the SS was never told the desk emptied; pre-existing since the Cockpit shipped). **Fixed:** the effect now emits `editorPanels {active:false}` on a genuinely-empty tab set → the SS drops to its designed empty state. Boss re-check on the running binary (11:24): **PASS.** Commit `<this>`.
- Stage-0 discipline note: the first fix build FAILED silently useful — `os error 5` (the running app locked the exe); the freshness check caught the stale 10:52 binary before any test instruction went out; rebuilt clean after the Boss closed the app.

---

## STATE-OF-STANDING (SO#5 — before the pivot to the APP-KILLERs)

**Boss ruling:** fix the 2 new APP-KILLERs first; the SS-Cockpit migration PAUSES at the Part-A checkpoint.

- **(a) Verified-shipped + protected:** SS-Cockpit §0–§3 + the empty-desk follow-up (`45d20b88`→`9535072f`), ALL Boss-validated on the running binary (Stage 1 + re-check). The SS is the clean read-only cockpit: lenses at full glass + Pin/Follow + header lens toggle + honest empty state; ~930 lines of duplication/dead code gone. Earlier this arc: PJ-089 (`2f43fc97`), PJ-090-by-cut (`4196a9d2`) — both Boss-validated.
- **(b) In-flight / paused:** the SS-Cockpit migration — Parts B–F NOT started (next step would be the A2 pre-B hardening commit + the two cache keys). Plan + amendments in `docs/SS-Cockpit-Migration-PLAN.md` (approved as amended); resume point = §6 (pre-B hardening first). The repointed `COCKPIT_ENABLED` flag has zero consumers by design until §10. Working tree clean.
- **(c) Known-broken (the pivot's cause):** the sweep's 5 pre-existing APP-KILLERs (`lab/reports/SWEEP-REGISTER-2026-07-14-wf_8b0a5104.md`): the 2 NEW = ensure_cid_cn destroys write-ahead recovery on manual reopen (`store.ts:2039`) + dirty background models never flushed at app close (`+layout.svelte:3436`); plus save_pulse non-atomic (=PJ-075), open_existing_universe active_path flip (`universe.rs:1099`), template-insert raw write bypass (`+layout.svelte:4767`).
- **(d) Pending not started:** Group-1 queue (PJ-098/093/086/099/085+073/074/083/087+075/076/077/094–097/100/101/072/002); the Cockpit Parts B–F; PJ-014 locale backfill; PJ-081 orientation-body refresh.
- **(e) Doc drift:** SS help + User Manual still describe the pre-cut SS (§16 of the Plan owns the rewrite — deferred with the migration); the sweep register is NOT yet mapped item-by-item into the ledger (the 5 app-killers are; the ~50 others map at the next full reconciliation — noted in v1.28's preamble).

---

## PJ-102 — the manual-reopen recovery APP-KILLER — FIXED (three-part arc) + the banner exits

**Function in hand:** the manual-reopen recovery path (`openNoteTab` × the write-ahead net × `ensure_cid_cn` × the freshness arbiter) + the save-health banner.

### The arc (each part driven by the Boss's live findings on a REAL locked file)
1. **PJ-102a** — Reproduce-First (Recipe S1 RED→GREEN): `ensure_cid_cn`'s disk-verbatim return no longer overwrites net-recovered content (adopt gated on `!extractCidCn(content)`). Adversarial review: SAFE 5/5.
2. **Boss live-test round 1: PASS — with two new wounds** (the banner self-healed while the file stayed locked; switching away and back lost the recovered line). Diagnosed off the code + journal: the recovered model was **born clean on content disk never had**.
3. **PJ-102b** — born-DIRTY + TRUE disk baseline (`markRecoveredFromNet`); `adoptDisk` phantom-event guard; the restore path gets the true baseline (the review's confirmed Q4 hole — a phantom event destroyed the PRESERVED net via clearWriteAhead-on-adopt); the `''`-sentinel Q5 corner fixed (baseline only from real disk bytes). Recipes S4/S5/S6/S7 RED→GREEN.
4. **PJ-102c (Boss-directed mid-arc)** — the two locked-file EXITS on the banner: **Save a copy** (verbatim sibling; fresh identity — cid stripped + title suffixed after the Boss's twin-tab remark; localized suffix ×15; NEW tab to avoid the failed-outgoing-flush abort) + **Discard…** (two-step inline confirm). Recipes S8/S9.
5. **Boss live-tests: ALL PASS** (reopen · switch-away/return · honest banner · Save-a-copy · Discard · the tab-label re-check).

**Gates:** Recipe S ×9 · vitest 350 · svelte-check 0 · 2 adversarial reviews (findings fixed in-pass, WA#6). Test artifacts cleaned from `Eisa Test/`; the lock job released.
**Close (SO#9):** Pending Jobs **v1.29** (PJ-102 done; **PJ-106 Arabic/RTL filed as the Boss-directed ► Next**); Orientation **v3.50**; help save-safety section updated (EN).

### NEXT — PJ-106: the Arabic/RTL typing & navigation overhaul (Boss-directed)
Boss (2026-07-14): Home/End misbehave; line/paragraph navigation and word/sentence/line/paragraph/page selection broken when typing Arabic; worse in bilingual notes. Editor-core (CM6 bidi) across NotePane+FocusPane → **/migration** with WA#5 prior-art research (CM6 bidi isolates, Obsidian/VS Code RTL). Starts next.

---

## PJ-106 — Arabic/RTL Typing & Navigation /migration — BUILD (Phases 1–2 closed 2026-07-14)

**Function in hand:** the editor's caret/selection/direction engine for RTL + bilingual notes (NotePane + FocusPane + ConflictMergeView), shared from `$lib/editor/`.
**Ruled:** Option B (full: direction + selection), A-first, LOGICAL (Word) arrows, per-paragraph Ctrl+Shift override stored as an invisible RLM/LRM mark. Plan `docs/PJ-106-RTL-Typing-PLAN.md` (approved as amended; 15 design-inspection hazards binding). Symptoms `lab/reports/PJ-106-RTL-Symptoms-BossReported.md` (Rounds 1–3).
**Reproduce-First:** the Boss's detailed live symptom reports (Rounds 2–3) ARE the on-demand reproduction; the visual defects are not headless-testable (jsdom false-passes — no layout), so the fix is verified by the staged Boss live-tests; the offset-pure direction logic is locked in `tests/pj-106/rtlDirection.test.ts`.

### Predecessor → Replacement (the direction heuristics re-pointed — all SAME PLACE, no relocation)
- **H2** `dirCompartment` editorAttributes.dir (`NotePane.svelte:450`) — Replacement: extended to carry BOTH editor + content attrs at the RESOLVED base (`dir==='rtl'?'rtl':'ltr'`); same place. **H3** `contentAttributes.dir:'auto'` (`:451`, the viewport-flip competitor) — CUT; folded into H2 at the resolved base (SI2-1). **H4** `bidiPlugin.resolveEditorDir` (`bidiPlugin.ts:59`) — Replacement: reads the DOM `dir` (now deterministic), the 'auto' viewport-scan branch REMOVED (H9). Reconfigure site (`NotePane.svelte:815`) updated to keep both attrs in sync. FocusPane H2 (`:183`) → `detectDir(value)` base + content attr (SI2-2). ConflictMergeView panes (`:52`) → content dir = the pane's resolved base (SI4-01). **H1** `detectDir(body)→noteDir` (`+layout.svelte:1631`) kept (never returns 'auto' — verified). **H5** toolbar `RTL_DETECT` — untouched (INV-5).

### §A1 — the headline direction fix (SHIPPED to test)
`EditorView.perLineTextDirection.of(true)` enabled (rollback lever `src/lib/editor/rtlFlag.ts` `RTL_MOTION_ENABLED`, in a compartment for NotePane; flag-gated add for Focus/merge — SI4-03: motion only, bidiPlugin's static rendering untouched) across all THREE editable surfaces; deterministic base direction replaces `dir='auto'`. This connects the already-rendered per-line `dir` to the caret/selection MOTION engine — the root cause of symptoms ①②③. Gates: svelte-check 0, vitest 354 (+4 rtlDirection). **Awaiting Stage-1 Boss live-test.** (§A2 same-transaction, §A3 empty-line inheritance, §A4 isolate ranges, §A5 logical-arrow keymap, then Part B selection + the Ctrl+Shift override — after the Boss validates the core.)

### §A1..§A5 continuation (Boss-validated + committed, into 2026-07-15)
- **§A1 direction fix — Boss PASS** (Stage-1). Committed `404f7139`.
- **§B0 (Boss-requested mid-arc)** — triple-click selects the line's TEXT, not the trailing newline. `EditorView.mouseSelectionStyle` override (`src/lib/editor/tripleClickLine.ts`), shared NotePane+FocusPane+merge (Editor Parity); supports drag/extend/multi. **Boss PASS.** Commit `a75868fd`. (Round-4 ruling recorded: triple-click=paragraph is the universal Win+Mac standard; sentence = Ctrl+click, Part B.)
- **§A3+§A2 — Enter on an RTL line puts the caret on the RIGHT** (Boss regression report + screenshot). Root cause read off code: `.cm-line{unicode-bidi:plaintext}` defaults an EMPTY line to LTR, and bidiPlugin only stamped `dir` when it DIFFERED from base → in an RTL-base note the empty line matched → no stamp. Fix: neutral RTL line ALWAYS stamped (§A3) + structural change (line-count) rebuilds synchronously so it lands same-frame (§A2). **Boss PASS.** Commit `60124045`. Safety-inspection (whole-app, ran instead of diff-scoped): editor-lifecycle/notemodel/reactivity = 0 confirmed; it also HIT the weekly agent limit (resets Jul 18) → solo the rest of the week. It surfaced an unrelated APP-KILLER (SS wikilink click consumes the WAB recovery net read-only) — logged for PJ triage.
- **§A5 — logical (Word-style) arrow keys** across bidi boundaries. `src/lib/editor/rtlMotion.ts` `logicalArrowKeymap(skipRanges?)`: ArrowLeft/Right move one LOGICAL char (forward resolved from the per-line base), Shift extends; lens-widget caret-trap (design-inspection H3) avoided via an INJECTED skip source scoped to these commands (NOT the global atomicRanges facet, which also feeds Backspace); injected so FocusPane skips livePreview (Rule 6). Folded with §A1 into ONE `rtlMotionCompartment`. `tests/pj-106/rtlMotion.test.ts` (7 offset-pure). svelte-check 0, vitest 11 (pj-106). **Boss PASS** incl. the Arabic↔Latin seam. Ctrl+←/→ word-hop left unchanged (no stock logical word command — flagged for Part B). Commit `a75868fd`… (see git). Symptoms ②(End/Home) & ③(trailing Latin) re-confirmed PASS by Boss.
- **Home-caret-invisible on imported notes — DIAGNOSED then PARKED (Boss "closed" 2026-07-15).** Drove the running app via computer-use + filesystem diff; trigger = the imported note's rich 16-field frontmatter (body byte-identical, proven innocent); ruled out body/heading/wrapping/callout/long-URL. Exact pixel mechanism not nailed (1.5px blink unresolvable in screenshots; release devtools disabled → needs an instrumented build). Full record: symptoms doc Round 6. **Polish-class; parked.** Test notes → scratchpad; Boss notes untouched.

**PJ ledger (SO#9):** file the parked caret bug as a new PJ-NNN (deferred, Boss-ruled) + the SS-wikilink-WAB app-killer surfaced by the inspection, at the next version bump. PJ-106 Part A core (direction + logical arrows + triple-click) shipped & Boss-validated; remaining: §A4 isolates (only if symptom ③ still shows — Boss re-confirmed it PASS, so §A4 may reduce to the callout-caret repro), then Part B (sentence/paragraph/page selection + the Right/Left-Ctrl+Shift paragraph override).
