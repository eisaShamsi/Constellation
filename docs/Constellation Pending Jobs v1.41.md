# Constellation Pending Jobs

**Version 1.41 | 2026-07-21**

> **What changed in v1.41** (**Boss ruling: "Let's focus on the Template Studio/engine. For me, it is priority one now." MIG-103 opened; §1 save-side + D2 BUILT & Boss-validated. Five research passes, four of my framings corrected. SO#9. Ultracode**):
> 
> ### ★ THE RE-PRIORITIZED BACKLOG
> 
> **► NEXT ACTION — MIG-103 D3** (a folder/kind carrying its own default template: off by default, opt-in per context, deepest-wins, only onto a still-empty note), **then D4** (apply a template to a note already started — promoted by the Boss's PKM/PKF correction to *the bridge between capture and composition*), **then §1's use-side** (each of the three kinds doing its distinct act) and the confirmed interaction model (destination propose+show, library picker when nothing is open, mixing heads-up).
> 
> **CLOSED THIS JOB:**
> - **MIG-103 §1 save-side — Boss-validated 2026-07-21.** The Boss's THREE kinds (whole note · frontmatter · snippet), each stamping `template_kind:`; title-confirm prompt on save; snippet extent is the user's choice (*My selection* vs *Whole note*, offered only when a selection exists). 13 Rust tests.
> - **MIG-103 D2 — the template door. Boss-validated 2026-07-21** (Steps 1–6). A quiet "Start from a template…" inside an empty note, gone on the first keystroke; applies the template to THAT note through the model (one undoable action), frontmatter merged additively — never a YAML dump.
> - **`create_note` frontmatter-merge defect — FIXED.** It trimmed every line (destroying nested-YAML indentation on EVERY template instantiation) and its identity filter matched trimmed lines, so a NESTED `title:` was falsely dropped. Red-first.
> - **MIG-103 D1 — no work needed.** New Note already opens blank and instant; the Boss's ruling confirms current behaviour.
> 
> **NEWLY FILED:**
> - **PJ-133** *(APP-KILLER · pre-existing)* — **"Discard my changes" silently un-discards.** `discardFailedSave` (`store.ts:506`) never clears the dirty model, and is the only `reloadTabsFromDisk` caller with neither `markCascading` nor `markReseeding`. After a failed save, clicking Discard removes the banner and the write-ahead net but leaves the discarded text in the model — the next departure flush (tab switch / closeTab / app close) durably writes it back over the disk copy the user explicitly chose to keep. No error, and the recovery net was erased by the same function. **Found by the 2026-07-21 inspection; NOT from this job's changes.**
> - **PJ-134** *(APP-KILLER · pre-existing)* — **`ensure_universe_notes_folder` swallows a libraries.json read/parse failure** (`universe.rs:384`): `.ok().and_then(...ok()).unwrap_or_default()` turns ANY failure into an EMPTY library list. **Found by the same inspection; NOT from this job's changes.**
> - **PJ-132** *(LOW · tooling)* — **the Sight v6 perf benchmarks flake under parallel load.** Three wall-clock tests (32 ms and 16 ms thresholds) slip 2–4 ms when 45 test files run together. **Verified pre-existing** (they fail on clean `main`). A suite that flakes trains us to ignore it — either widen the thresholds, mark them serial, or move them out of the default run.
> 
> **STILL OPEN:** the whole **PJ-130** safety backlog — **the 2026-07-21 whole-app sweep returned 60 confirmed (4 APP-KILLER · 24 HIGH · 20 MED · 12 LOW), a superset of the earlier 37; PJ-130's scope grew and needs re-triage against HEAD before the batches are worked.** (Earlier triage: 30 live defects: 1 APP-KILLER `store.ts:2298` second-screen dirty-birth, 18 HIGH, 11 MED — plan at `docs/PJ-130-Safety-Backlog-Remediation-Plan.md`, **Batch 1 built but Boss-untested**) · **PJ-124** (the inspection ignores `args.files`) · PJ-125/126/127/128/129/131.
> 
> **MIG-103 remaining phases:** §2 request path (+ the Arabic skeletons from R3, 10 items awaiting Boss validation) · §3 the Studio as a core-plugin app-within-app + the visual gallery · §4 recognition · §5 the manuscript builder (proposer, NOT a wizard) · §6 the Studio's own style setter · §7 tending.
> 
> ---

**Version 1.40 | 2026-07-20**

> **What changed in v1.40** (**MIG-101 opened + Phase A SHIPPED & Boss-validated. THREE APP-KILLERs fixed (one pre-existing-shipping, one mine, one pre-existing silent-data-loss). Whole-app safety inspection run: 37 confirmed. SO#9. Ultracode**):
> 
> ### ★ THE RE-PRIORITIZED BACKLOG
> 
> **► NEXT ACTION — MIG-101 Phase B** (the observation engine: persisted, write-time, cheap; conservation check; resumable back-fill). Plan: `docs/MIG-101-Shape-Graduation-Quiet-Signal-Plan.md`. **Boss ruled the sequencing: shape + signal FIRST, Qusasah as its own migration (proposed MIG-102).**
> 
> **CLOSED THIS JOB:**
> - **MIG-101 Phase A — SHIPPED + Boss-validated 2026-07-20** (Steps 2–6 pass, incl. the set-shape → type → switch-away-and-back durability check). `shape.rs` + `shape_history` + undo stack + menu + i18n ×15. **Shape goes THROUGH THE MODEL, never to disk** (single content ownership). **Undo CONSUMES a step** rather than appending its inverse.
> - **APP-KILLER — `update_frontmatter_property` CRLF/trailing-newline corruption: FIXED.** PRE-EXISTING and shipping (Bases cell editing). Every property edit silently converted CRLF→LF throughout the whole file and stripped the trailing newline. Fixed structurally (byte-offset splice; the body is never split). 9 red-first tests.
> - **APP-KILLER — `applyShape` missing the read-only guard: FIXED.** MINE; caught by the safety inspection. Read-only hosts could durably write a stale compose over the live note, and a CLEAN receiving model ADOPTS rather than conflicts → silent revert. Guard + hidden menu + a structural test proven red.
> - **APP-KILLER — `serializeLine` had no `nested-object-list` branch: FIXED.** PRE-EXISTING silent user-data loss: editing one `ikhtilāf` row flattened the whole block to a scalar and **every structured row vanished from the `.md` on reopen**. The legacy serializer handled it; the G4 swap dropped it. Red→green (`tests/g4/nestedObjectListRoundtrip.test.ts`).
> - **The Rust suite was already RED on `main`** — two `cache::` failures from fixtures one column (`created`) behind the production `note_links` schema. Fixed; suite green (1077/0).
> 
> **NEWLY FILED:**
> - **PJ-130** *(HIGH · from the whole-app inspection)* — **the remaining 34 confirmed findings.** The sweep returned **37 confirmed: 3 APP-KILLER, 19 HIGH, 12 MED, 3 LOW** across 15 files (`store.ts` ×7, `NoteEditor.svelte` ×6, `libraries.rs` ×6, `search.rs` ×3, `yamlDoc.ts` ×2 …). Classes: silent-data-loss ×8, index-divergence ×7, content-corruption ×4, content-loss ×4, cross-window-clobber ×3, false-success ×3, freeze-hang ×3. **Three app-killers were fixed this job; the rest are UNTRIAGED and need a Boss ruling on batching.** Full record: `tasks/wiexw8j3l.output` (260 KB), run `wf_5ff846df-00c`.
> - **PJ-131** *(LOW · discipline)* — **schema-version bump as a checklist item.** Adding `undone` without bumping `SHAPE_SCHEMA_VERSION` meant the ALTER never ran on an already-stamped table and undo went SILENTLY inert. Fixed here by making every entry point *upgrade if behind* rather than *bail if not current*; the general lesson (any table-shape change bumps its version in the same edit) should be a checklist line wherever module schemas are documented.
> 
> **STILL OPEN from v1.39:** APP-KILLER #2 `store.ts:2298` second-screen dirty-birth (**re-confirmed by this sweep**) · the cascade-window race · **PJ-125** (template-insert — needs a Boss design ruling) · **PJ-124** (**re-confirmed: the inspection ignored `args.files` and ran whole-app again**) · PJ-126 · PJ-127/128/129.
> 
> **Boss rulings still awaiting:** orphan phantom `.md` files from pre-fix folder moves · whether `WRITE_GATE_ENFORCE` should be flipped on for identity verdicts · batch-move's swallowed sibling failures · **whether "The Constellation Way" belongs in `CLAUDE.md`**.
> 
> **Then Group 1:** PJ-110 · PJ-117 · PJ-118 · PJ-119 · PJ-115 · PJ-104(→072; +PJ-120) · PJ-105 · PJ-098/093/086/099/085+073/074/083/087+075/076/077/094–097/100/101/111/112/002 · PJ-124/126/127/128/129/130/131. **Group 3:** PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
> 
> ---

**Version 1.39 | 2026-07-18**

> **What changed in v1.39** (**Boss ruling: app-killers first, then PJ-126. Remediation #1 (moveItem repath) SHIPPED + Boss-validated, with the ⋯-menu five-dead-commands fix riding the same build. gate_rename DOWNGRADED on adversarial re-verify. PJ-127/128/129 filed. SO#9. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — APP-KILLER #2: template-insert stale write (PJ-125).** Deterministic, CONFIRMED at HEAD: the `.note-pane` selector matches nothing, so the only reachable branch writes stale `tab.content` over disk (losing typing since the last flush), watcher-suppressed so no adopt heals it. **Needs a Boss design ruling first**: the cursor-insert branch has NEVER run, so "insert template" must be designed, not restored. **Then #3/#4** (cascade-window + second-screen dirty-birth — races; build the on-screen Note-Identity/Write-Trace instrumentation strip + the diagnostics-gated artificial cascade delay FIRST, per Reproduce-First). **Then PJ-126** (the content-bearing tooltip sweep — Boss-ordered after the app-killers). **Then PJ-114 Phase-1 §4–§10.**
>
> **CLOSED THIS JOB:**
> - **APP-KILLER #1 — moveItem folder-move repath: FIXED, Boss-validated 2026-07-18.** `store.ts` descendant branch now derives from the Rust-returned `newPath` (was `targetFolder + relative`, dropping the moved folder's own segment — wrong 100% of the time; phantom-note forking, or silent overwrite of a same-basename note since `WRITE_GATE_ENFORCE=false`). Red-first tests `tests/pj-127/moveItemRepath.test.ts` (5/7 red → 7/7 green). Boss live recipe passed: correct Copy-path, both phrases in the real file, no phantom in `Beta\`, no duplicate tab.
> - **⋯-menu five dead commands: FIXED, Boss-validated 2026-07-18.** Three mechanisms (host-shadowed file-ops · phantom `delete-note` event · `addProperty` window/document mismatch). Delete now uses the tree's exact confirm+trash flow.
>
> **NEWLY FILED:**
> - **PJ-127** *(MED · same-function gap)* — **`moveItem` has no pre-move flush.** `renameItem` wraps its invoke in markCascading + flush; `moveItem` does not, so a debounce firing during the awaited `move_item` IPC composes at a path that no longer exists. Surfaced by the Phase-1 architect; NOT reproduced (Reproduce-First gate: instrument before fixing). Test dir `tests/pj-127/` already carries the repath tests.
> - **PJ-128** *(LOW→MED · hardening)* — **`gate_rename` dest-exists check** (DOWNGRADED from app-killer): the locked region has no `new.exists()` check (`write_gate.rs:566-600`; sibling `gate_rmw_rename` has one at `:684-687`). Consequence refuted today (`move_item` checks upstream at `libraries.rs:1713`), residual = check-to-rename race + any future caller. Add the in-lock check.
> - **PJ-129** *(LOW · tooling)* — **phantom-event guard.** Third instance of a `constellation:*` CustomEvent dispatched with no listener (delete-note, add-property, and the recorded reveal-in-tree precedent). A vitest that greps dispatch sites and asserts a matching `addEventListener` exists would kill the class.
>
> **Open items needing a Boss ruling (from the Phase-1 architect, not silently parked):** orphan phantom `.md` files already created by pre-fix folder moves (detect-and-report vs leave alone) · whether `WRITE_GATE_ENFORCE` should be flipped on for identity verdicts specifically · batch-move's per-item `catch` swallowing a failed sibling.
>
> **Then Group 1:** PJ-110 · PJ-117 · PJ-118 · PJ-119 · PJ-115 · PJ-104(→072; +PJ-120) · PJ-105 · PJ-098 *(re-confirmed live — OrgChart raw `invoke('move_item')`)* /093/086*(re-examine)*/099/085+073/074/083/087+075/076/077/094–097/100/101/111/112/002 · PJ-124 + PJ-126 + PJ-127/128/129. **Group 3:** PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> ---

> **What changed in v1.38** (**The app-drawn link tooltip + its Style-Setter element SHIPPED and Boss-validated across four test rounds. PJ-126 filed. SO#9. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-114 Phase-1 §4** (NotePane Living-Link): the **confidence badge + row-density setting**, folding in the `LinkStateChips` extraction. **Gate: `hypothesis` — the majority tier — must render QUIET, not noise.** Then **§5** (BLOCKING indexer fix — it wipes confidence, resets `created`, resurrects archived links), **§6** (inspector popover), **§7–§10** (the write half). Plan: `docs/concept-papers/PJ-114-NotePane-LivingLink-P1-Plan.md`.
>
> **CLOSED THIS JOB:**
> - **The app-drawn link tooltip — DONE, Boss-validated 2026-07-18** (four rounds: localization → placement/size → the CSS-comment regression → language-awareness → Suggested Connections). `linkTip.ts` (window-singleton, `data-linktip` contract, delegated), `pLinkTip` in Style Setter → Panels (8 controls, reusing the shared `--tooltip-shadow`), direction by **`detectDir` dominance** on both the box and the seven content spans. The two link panels and Suggested Connections now use **no native tooltips**. New `backlinksPanel.linkIt` ×15 (a hardcoded English string found in passing) + `aria-label` for the icon-only button. Guarded by `tests/pj-114/linkTipCss.test.ts` (CSS integrity, red→green verified).
>
> **NEWLY FILED:**
> - **PJ-126** *(MED · consistency)* — **the content-bearing native-tooltip sweep.** A survey found **295 `title=` sites** app-wide (`+layout.svelte` 64, `NotePane` 22, `SourceReviewPanel` 17, `TableToolbar` 16, `CalloutTypesEditor` 12, `GraphMindView` 10 …). Most are legitimately chrome (toolbar/icon buttons) and should stay native. The ones that matter show **note content** — names, snippets, summaries, annotations — because those truncate and need direction to follow the content. The Boss has now reported this inconsistency **twice** (the row hint, then Suggested Connections), so the remaining content-bearing sites should be converted in ONE pass rather than discovered one at a time. Boss recommendation given; awaiting the go. **Open.**
>
> **Then PJ-110** (recovery-net durability). **Then Group 1:** PJ-125 · PJ-117 · PJ-118 · PJ-119 · PJ-115 · PJ-104(→072; +PJ-120) · PJ-105 · PJ-098/093/086*(re-examine)*/099/085+073/074/083/087+075/076/077/094–097/100/101/111/112/002 · PJ-124 + PJ-126 *(tooling/consistency)*. **Group 3:** PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> **⚠️ STILL AWAITING BOSS SEQUENCING RULING —** two whole-app sweeps confirmed **5 APP-KILLERs**, all pre-existing, none from this work: `store.ts:3539` (folder-move repaths open tabs one directory too high → later saves create a phantom note absorbing every edit) · `+layout.svelte:6356` (cascade guards from a pre-walk snapshot vs a live force-adopt) · `store.ts:2277` (second screen born dirty, durably writes the stale snapshot) · **PJ-125** (template-insert stale write) · `write_gate.rs:566` (`gate_rename` has no destination-exists check inside the locked region; Windows `fs::rename` replaces silently). **Question on the table: finish PJ-114 Phase 1 (§4–§10) first, or divert to the app-killers now?**
>
> ---

> **What changed in v1.37** (**PJ-114 Phase-1 §3b SHIPPED + Boss-validated — the living-link chip is localized from one source of truth, and Constellation now draws its tooltip. §3b closed; the ► Next action advances to §4. Two new items filed. SO#9. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-114 Phase-1 §4** (NotePane Living-Link, in build): the **confidence badge + row-density setting**, folding in the `LinkStateChips` component extraction (the badge touches that exact markup — one pass, one pixel-identical verification). `AppSettings.linkLifecycle` gains `density: 'minimal'|'rich'`, control in Settings → Files after the half-life slider, reusing `updateLinkLifecycle()` and the four `--confidence-*` vars from `ConfidencePicker` (no new colour tokens). **Verification gate: `hypothesis` — the majority tier — must render QUIET, not noise; if it shouts, the step isn't done.** Then **§5** (the BLOCKING indexer fix — it wipes confidence, resets `created`, and resurrects archived links), **§6** (per-link inspector popover), **§7–§10** (the write half). Plan: `docs/concept-papers/PJ-114-NotePane-LivingLink-P1-Plan.md`.
>
> **CLOSED THIS JOB:**
> - **PJ-114 Phase-1 §3b — DONE, Boss-validated 2026-07-18.** Shared localized link-state display (`src/lib/links/linkDisplay.ts`) + the app-drawn chip tooltip (`src/lib/links/linkTip.ts`). Delivered with **ZERO new i18n keys** by reusing `plurals.walks` + `ccs.tier.*` (already complete in all 15 locales, already on screen in the CCS panel) instead of the ~8×15 the plan estimated — a second translation of the same concepts would have drifted against CCSView. Three pre-existing defects fixed in-pass (custom-type slug vs label on the same row · `-1d ago` future timestamps · the mis-measuring bidi isolates). A **third** copy of the tooltip was found inside NotePane (`livePreview.ts`) by grepping the built bundle. 55 new tests incl. all-15-locale i18n parity.
>
> **NEWLY FILED:**
> - **PJ-124** *(tooling · MED)* — **the `safety-inspection` workflow ignores its `files` argument.** Three invocations in one session (args as an object, as a JSON string, and with `mode`/`scope` keys) each ran the **whole-app** sweep instead of the diff-scoped hunt the per-build standing order calls for. Cost: two unintended full sweeps (~17M subagent tokens). The per-build cadence in CLAUDE.md is effectively unavailable until the arg handling in `.claude/workflows/safety-inspection.js` is fixed. **Open.**
> - **PJ-125** *(APP-KILLER · content-loss)* — **template-insert writes the STALE store copy over disk.** `+layout.svelte:4798` selects `.note-pane .cm-editor`, but **no element in `src/` carries `note-pane`** (CLAUDE.md itself records the class as `.pane`; NotePane's root is `.e-desk`). So the `if (pane)` branch is unreachable and the "fallback" at `:4813-4817` is the ONLY path — it writes `tab.content`, which the autosave deliberately never updates ("Do NOT update the store during autosave"), discarding every keystroke since the last flush. `gate_write` watcher-suppresses the path so no adopt heals it, and the still-dirty model then overwrites the inserted template. Silent both ways. Found by the 2026-07-18 whole-app sweep. **Open · Charter · Group 1.**
>
> **Then PJ-110** (recovery-net durability). **Then Group 1:** PJ-125 · PJ-117 · PJ-118 · PJ-119 · PJ-115 · PJ-104(→072; +PJ-120) · PJ-105 · PJ-098/093/086*(re-examine)*/099/085+073/074/083/087+075/076/077/094–097/100/101/111/112/002 · PJ-124 *(tooling)*. **Group 3:** PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> **⚠️ AWAITING BOSS SEQUENCING RULING —** two whole-app sweeps this session confirmed **5 APP-KILLERs**, all pre-existing, none from §3b: `store.ts:3539` (folder-move repaths open tabs one directory too high → later saves create a phantom note that absorbs every edit) · `+layout.svelte:6356` (cascade guards built from a pre-walk snapshot while the reload force-adopts against live `openTabs`) · `store.ts:2277` (second screen born dirty from the shared write-ahead net, durably writes the stale snapshot) · **PJ-125** (above) · `write_gate.rs:566` (`gate_rename` has no destination-exists check inside the locked region; Windows `fs::rename` replaces silently). Question on the table: finish PJ-114 Phase 1 (§4–§10) first, or divert to the app-killers now?
>
> ---

> **What changed in v1.36** (**PJ-114 transformed: what began as "a right-click menu for Focus mode" became the elevation of NotePane to a TOP-PRINCIPAL — the GATE to Knowledge Cognition — with a capability audit, a Five-Acts vision + roadmap, and an approved Phase-1 `/migration` now in build. FM+'s foundation shipped; its MENU is paused pending the NotePane cross-check. 3 Boss-reported UI fixes shipped; 2,550 lines of unmounted dead code deleted. SO#9. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-114 Phase-1 §3b** (NotePane Living-Link, in build): extract the shared **localized** traversal helper — `fmtTraversed` is byte-identical in both link panels **and** hardcoded English, and the chip tooltip is a hardcoded English template (~8 keys ×15 locales). Then **§4** (confidence badge + density setting, folding in the `LinkStateChips` extraction), **§5** (the BLOCKING indexer fix), **§6** (per-link inspector), **§7–§10** (the write half: body-token rewrite engine → re-type → annotate → supersedes). Plan: `docs/concept-papers/PJ-114-NotePane-LivingLink-P1-Plan.md`.
>
> **Then PJ-110** (recovery-net durability — the prior Group-1 top). **Then Group 1:** PJ-117 · PJ-118 · PJ-119 · PJ-115 · PJ-104(→072; +PJ-120) · PJ-105 · PJ-098/093/086*(re-examine)*/099/085+073/074/083/087+075/076/077/094–097/100/101/111/112/002. **Group 3:** PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> ### ★ NEW TOP-PRINCIPAL (Boss, 2026-07-18)
> **"NotePane is the GATE to the knowledge cognition, it is the key to one's knowledge. If Constellation is going to be a unique, powerful, and smart PKM/PKF system, it will be through the NotePane capabilities. It is the starting point of one's cognitive journey, so we have to make sure it is well-equipped and well-instrumented."** → NotePane is the instrument of thought; every other surface only *reads* what it authored. Docs: `PJ-114-NotePane-Capability-Audit.md`, `PJ-114-NotePane-Cognitive-Gate-Vision.md`.
>
> ### CLOSED since v1.35
> - **PJ-116 — DONE/CLOSED 2026-07-18, Boss-validated, commit `455a7930`.** A title typed in Focus mode was silently discarded (FocusPane fired `ontitlechange`; `+layout` never wired it). Now routed to `handleRenameComplete` — renames the file AND heals every `[[link]]` universe-wide. The same commit landed the **parked sweep-#3** cascade-freeze protection (no longer uncommitted).
> - **⋯ note-menu RTL truncation — CLOSED, Boss-validated, commits `8ee48a86` + `cc35524d`.** Clipped by `.e-desk`'s `overflow-x:hidden`; made fixed-position with measure-and-clamp. A second Boss report (Latin UI + Arabic note) disproved viewport-overflow anchoring — with the sidebar open the menu spilled over the file tree. Correct rule: **anchor by the NOTE's direction, always open into the note.** Verified across all 4 UI×note-direction combinations.
> - **Caret continuity NotePane ↔ Focus — CLOSED, Boss-validated, commit `ca6ce954`.** Entering Focus dropped the caret to position 0; leaving restored the *pre-Focus* position. Root cause (found by on-screen instrumentation after two failed guesses): the app already has a per-tab cursor memory (`tab.cursorPos` → `initialCursorPos`, `NotePane.svelte:848-851`) and the fix had added a **second, competing** one that was overwritten. Fix reuses the existing mechanism.
>
> ### IN PROGRESS
> - **PJ-114 Phase 1 — "NotePane owns the living link"** (`/migration`, Boss-approved, ONE continuous migration). Boss rulings: **body-text is the write** (File-Over-App; no LINK files are ever written — the docs' dual-layer model does not exist) · identity `(source_path, folded target, link_type)` · inspector = popover · density = enum setting (**minimal by default, richer by choice**) · "stretch it" scope (+`supersedes` +confidence badge) · controls at three depths (**adjusted:** Backlinks rows are read-only for writes — their `sourcePath` is another, usually closed note). **Shipped:** §1 (deleted 4 unmounted components, 2,550 lines, `e88cafb8`), §2 (align-buttons reproduce — they work; a different defect surfaced → PJ-122), §3a (read-widening: `created` now reaches the UI, `6c810836`). **Remaining:** §3b, §4–§10.
>
> ### PAUSED / REFRAMED
> - **PJ-114 FM+ MENU (§1.3+) — PAUSED.** v1 (copy-link-text/copy-path + clipboard) **Boss-REJECTED**: strictly poorer than the native webview menu. Reframed by a research dig (`wf_b5c67f60-646`, `PJ-114-FM-Plus-Smart-Menu.md`) as a **knowledge** menu (native owns the clipboard; FM+ owns knowledge formulation). Then Boss ruled: *"it is the NotePane that should have the full living-link, so we are not going to replicate it in the FM"* → **enrich NotePane FIRST, cross-check FM+ AFTER.** Rejected menu code reverted; the shipped FM+ foundation stays (**flag + persisted `focusModePlus` setting** `22a9de41`, **footer toggle** with localized mark ×15 `11ace3fe`, **shared wikilink finder** `2772e2be`). Resume = the FM+ cross-check once Phase 1 lands.
>
> ### LIVE BUGS FOUND BY THE PHASE-1 ARCHITECT/PLAN (scheduled INSIDE the migration — not separate PJs)
> - **Indexer wipes living-link state** (`search.rs:6058/6107/6145`): the preserve gate ignores `confidence` and the re-INSERT hardcodes `'hypothesis'`, `created = now`, `status = 'active'` → editing an annotation **wipes a user-set confidence**, **resets `created`**, and **resurrects archived links** (so `archiveLink` does not survive a save — breaking `supersedes` at its root). → **§5, BLOCKING**, with `safety-inspection`.
> - **"Remove link" strips EVERY link on the line**; right-clicking the 2nd link acts on the 1st (5 first-match-on-the-line regexes, `NotePane.svelte:1141/1280`). → fixed in **§7**.
> - **Duplicate identical tokens already lose data**: repeated `[[type::target]]` collapse to one row (UNIQUE key, no occurrence index) — the 2nd token's annotation is silently dropped **today**.
>
> ### NEWLY FILED
> - **PJ-121** *(feature · Boss-requested)* — **render markdown tables as real tables.** Live preview has NO markdown-table renderer (its only table code is the Bases/Lens `_renderTable`); `Insert→Table` inserts raw pipes (`NotePane.svelte:1238`) and `TableToolbar` manipulates the RAW markdown. Boss: *"Can we have a full working table that resembles a real one?"* Needs a live-preview table widget with markdown as source of truth; Editor-Parity + perf rules. **Own concept paper + `/migration`.**
> - **PJ-122** *(MED · editor UX)* — **text-align inserts raw HTML.** `setLineAlignment` (`NotePane.svelte:1061`) wraps the line in `<div style="text-align: …">`, which is visible as literal code while the cursor is on that line, only renders after the cursor leaves, and makes the line hard to click back into. Question the HTML-in-markdown storage itself. Same family as PJ-121 — **design them together.**
> - **PJ-123** *(HIGH · clobber-class)* — **`BacklinksPanel` "Link it" raw read-modify-write.** `linkMention` (~`BacklinksPanel.svelte:190`) does `readNote` → `write_note` on a note that may be **open and dirty**, wrapped in `catch {}` — the same class as the `b6310479`/`baae4533` clobber sweeps. Out of scope for PJ-114; **fix separately.**
>
> ### PROCESS NOTE (logged, non-negotiable)
> **Reproduce-First was violated and re-learned.** Two caret fixes shipped on *plausible mechanisms* without observing the failure; both failed live and cost two Boss test cycles before the Boss said *"Enough guessing."* On-screen instrumentation (release builds disable devtools) found the true cause in ONE run. Also: **§0.2's "proven against the main editor" claim was false** — it refactored `CodeMirrorEditor.svelte`, which nothing mounts (caught by §1). Go to instrumentation FIRST.
>
> ---
>
> *(Prior preambles v1.0–v1.35 + full history follow below; also durable in the versioned files.)*
> ---

**Version 1.35 | 2026-07-17**

> **What changed in v1.35** (**PJ-106 CYCLE CLOSED — the per-cycle whole-app `safety-inspection` sweep ran (a day early, at Boss direction) and the §B4 post-gate edge was fixed. The sweep confirmed 62 (3 APP-KILLER · 11 HIGH · 38 MED · 10 LOW); the 3 reachable app-killers were fixed + Boss-validated live + committed; the 3rd app-killer was found NOT user-reachable and reframed into a new feature.** Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-114 (Focus-mode right-click menu)** *(Boss-directed 2026-07-17, "finish #1+#4, then design Focus-RC")* — the complete right-click context list for Focus mode (link→Rename etc.), which also makes the parked sweep-#3 protection reachable + testable. Design via the Art Director & Team + the banked Obsidian RC targets. **Then PJ-110** (recovery-net durability — the prior Group-1 top). Then the Group-1 queue below.
>
> **Then Group 1:** PJ-117 (adopt stale-snapshot TOCTOU · HIGH) · PJ-118 (ConflictMergeView stale-generation · HIGH) · PJ-119 (CE malformed-frontmatter classification loss · HIGH) · PJ-115 (reloadTabsFromDisk case/NFC skip · MED) · PJ-104(→072, univ active_path; + PJ-120 libraries.json registry-wipe) · PJ-105 · PJ-098/093/086*(re-examine)*/099/085+073/074/083/087+075/076/077/094–097/100/101/111/112/002. **Group 3:** PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> ### CLOSED since v1.34
> - **PJ-106 — CYCLE CLOSED 2026-07-17.** The per-cycle whole-app sweep (`wf_776dbce6-a50`, 82 agents, 62 confirmed; register `lab/reports/SWEEP-REGISTER-2026-07-18-wf_776dbce6-a50.md`) = the migration's C3 cycle boundary. **§B4 post-gate: the toolbar-Ctrl+Shift-click disarm edge was CONFIRMED REACHABLE and FIXED** (sweep #4, commit `b6310479`). All PJ-106 close ritual now complete.
> - **Sweep #2 (APP-KILLER) — CLOSED, Boss-validated live, commit `317b2512`.** `loadTabHistoryEntry` (Alt+←/→) lacked the B1 one-path-one-tab dedup and bypassed `resolveNoteContent` → a second NoteModel per file (silent clobber of saved edits) + no net recovery. Fixed to mirror `openNoteTab` (dedup-switch + resolveNoteContent + born-dirty-on-recovery). RED→GREEN `historyNavDedup.test.ts`; adversarial review "ships as-is." (Supersedes the 2026-07-14 register's `store.ts:1283` HIGH; sibling of PJ-099.)
> - **Sweep #1 (APP-KILLER, NEW) — CLOSED, Boss-validated live, commit `baae4533`.** PropertyEditor's onDestroy flush used LIVE tabId/filePath (incoming note) with stale editableProps (outgoing note) → spliced note A's frontmatter onto note B (BUG-023 class via the props channel). Fixed with a mount-time identity snapshot (mirrors NotePane's `mountedFilePath`).
> - **Sweep #4 — CLOSED, Boss-validated live, commit `b6310479`.** The §B4 paragraph-direction gesture flipped on a Ctrl+Shift+toolbar-click (contentDOM-scoped disarm never saw focus-preserving chrome). Two-part fix: a window-level capture-phase mousedown+wheel disarm belt (ViewPlugin, torn down with the editor) + IGNORE OS key auto-repeat (`e.repeat`) — the root cause the Boss's before/after repro exposed (held-modifier auto-repeat re-armed the gesture right after the click-cancel).
>
> ### PARKED / REFRAMED
> - **Sweep #3 (was APP-KILLER) — NOT user-reachable → REFRAMED into PJ-114.** The FocusPane readOnly-during-cascade fix defends a scenario that can't happen today: Focus mode HIDES the tree/tabs (`body.focus-active`) and auto-exits on nav, so no rename cascade can fire while a note is in Focus. It becomes reachable ONLY once the Focus-mode right-click menu (link→Rename) ships. **Code parked UNCOMMITTED** (FocusPane.svelte, CascadeFreezeOverlay.svelte, +layout.svelte) to ship WITH PJ-114. (SO#8 + Reproduce-First win: the sweep's "tree visible beside FocusPane" premise was wrong; caught before shipping an untestable fix.)
>
> ### NEWLY FILED
> - **PJ-114** *(feature · Boss-directed ► NEXT)* — **Focus-mode right-click context menu (complete RC list).** Concept: a right-click in Focus gives the essential actions (incl. operating on the `[[link]]` under the cursor — e.g. Rename its target) without leaving Focus. Subsumes the parked sweep-#3 read-only-during-cascade protection. Touches the *Focus = minimal/parser-free* principle → concept-first + Art-Director design; reuse the banked Obsidian RC menus (Note/Folder/Link/editor-empty). Folds in **PJ-116**.
> - **PJ-115** *(MED · rename-cascade)* — `reloadTabsFromDisk` matches rewritten paths with an exact case-sensitive `t.path === fp` while the cascade's exclude belt NFC+lowercase-normalizes; an open backlink tab whose recorded casing differs is silently skipped → stale `[[link]]` reverted on next save. Fix: normalize the compare (match the belt). (`store.ts:790`; diff-inspection #10 / whole-app #49.)
> - **PJ-116** *(LOW · folds into PJ-114)* — FocusPane never wires `ontitlechange`, so a title typed in Focus mode is silently discarded (no rename, no error). (`+layout.svelte` FocusPane render; diff-inspection #12.)
> - **PJ-117** *(HIGH · cross-note-bleed)* — `adoptExternalChangeIntoTabs` captures `get(openTabs)` BEFORE its awaited reads then loops the stale snapshot; an in-place nav during the await + no path-guard on `adoptDisk` → the OLD note's frontmatter is adopted into the NEW note's model → a frankenstein durable write (cid swap). Fix: re-get after the reads (as `SecondScreenPage.adoptFreshDiskIntoSS:564` does) + an expectPath guard on adoptDisk. (`store.ts:880`; whole-app #5.)
> - **PJ-118** *(HIGH · cross-note-bleed)* — `ConflictMergeView`'s rebuild `$effect` has no stale-generation guard; a build for conflict A can overwrite `mergeView` after `target` switched to B → `saveMerged` writes A's content into note B's `.md`. Fix: capture/token compare after each await + destroy the superseded view. (`ConflictMergeView.svelte:144`; whole-app #13.)
> - **PJ-119** *(HIGH · CE false-success)* — on a note with malformed/unclosed frontmatter, `rewrite_frontmatter_sources/content_type` return content unchanged yet the callers write the ids into `note_meta` and return Ok → the classification never lands on disk and is wiped on the next reindex. (`src-tauri/src/sources/mod.rs:468`; whole-app #4.)
> - **PJ-120** *(HIGH → folds into PJ-072)* — `ensure_universe_notes_folder` swallows a `libraries.json` read/parse failure and REWRITES it with only the auto `universe_notes` entry → every other library registration silently destroyed (no G6 backup, runs every boot/switch). Fix: fail-closed load (as `libraries.rs::load_libraries` does). (`src-tauri/src/universe.rs:384`; whole-app #12.)
> - *(The remaining ~55 sweep findings — MED/LOW — re-confirm existing backlog: PJ-075/087 (save_pulse non-atomic), PJ-073/085 (yamlDoc), PJ-074 (folder-rename), PJ-099 (loadTabHistoryEntry await-window), PJ-100/101, etc. Full per-finding register: `lab/reports/SWEEP-REGISTER-2026-07-18-wf_776dbce6-a50.md`.)*
>
> ---
>
> *(Prior preambles v1.0–v1.34 + full history follow below; also durable in the versioned files.)*
> ---

**Version 1.34 | 2026-07-17**

> **What changed in v1.34** (**PJ-103 — the app-close data-loss APP-KILLER — CLOSED, fixed + Boss-validated live; the Reproduce-First arc REFUTED the filed mechanism and uncovered a DEEPER one: the graceful close cut off the final disk write AND the localStorage recovery net was proven NON-DURABLE (a WebView2 leveldb log-orphan wiped a whole session's net on reopen — witnessed live, evidence preserved). The close handshake now flushes dirty models to DISK. 4 new PJs filed. SO#9. Ultracode**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — Jul 18, 4am: the per-cycle whole-app `safety-inspection` sweep** — now fires AUTOMATICALLY (scheduled task `pj106-cycle-close-sweep-jul18`, one-time, register-only: it commits/fixes nothing; the live session fixes every confirmed finding per WA#6). It remains the PJ-106 C3 cycle boundary + the §B4 post-gate + the toolbar-Ctrl+Shift-click disarm edge. **Then PJ-110** (the recovery net's localStorage backing is not durable — the new Group-1 top; the hard-kill crash-recovery class PJ-108 protected still rides on it).
>
> **Then Group 1:** PJ-104 (→ PJ-072; fresh evidence 2026-07-16 — the app booted into كون عيسى twice while the last-used universe was Eisa Cognitive Knowledge, timestamps in the session log) · PJ-105 · PJ-098/093/086*(re-examine first — see notes)*/099/085+073/074/083/087+075/076/077/094–097/100/101/111/112/002. **Group 3:** the PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> ### CLOSED since v1.33
> - **PJ-103 — CLOSED 2026-07-17 (APP-KILLER, Boss-validated live; open since v1.28).** The Reproduce-First arc (the whole point of the rule): **(1) the FILED mechanism was REFUTED live** — "switch-away drops the outgoing save via the staleness guard" did NOT fire: two Boss-executed fast switches (paste+instant click; type+pre-aimed instant click) both persisted the outgoing note. **(2) The REAL loss fired on the close instant** — type + click ✕ inside the 1.5s debounce: the Boss's `MARKER-THREE` never reached disk (the beforeunload flush DID fire — settling lib.rs's old "unproven" comment — its synchronous net-stash landed in localStorage at 19:13:46, but the async disk write was cut by `win.destroy()`). **(3) The recovery net then FAILED cross-session — TOTAL SILENT LOSS**: on reopen, Chromium leveldb reused a stale MANIFEST, recovered an old log, and DELETED the session's `000003.log` (holding the stash) as an orphan — leveldb's own LOG file confesses (`Delete type=0 #3`); the restore honestly served stale disk (journal: `session_restore_begin 2 tabs → 2/2 restored`, 19:18:29). Evidence: `lab/reports/pj103-evidence-000003.log`. **The fix (Boss ruling: up to 5s, instant when clean):** the `session:final-flush` handshake now runs `persistSessionNow` → **`flushAllForAppClose`** (every dirty model durably to its .md via the bounded gate + a re-pass for keystrokes typed during the hold + a `final_flush_residual_dirty` journal marker + AWAITED FTS reindex of the flushed notes) → ack; Rust cap 700ms → **5000ms** with an honest `final_flush_no_ack_5s` marker; the listener registers at the TOP of onMount (a boot-window close acks instantly); **per-id save serialization** at the one gate (`noteSession.save` chains same-id saves newest-last, sync-prefix contract preserved via the fast path — 2 recipes caught my first version breaking it); the updater `relaunch()` path now flushes+persists first. **Stand-in adversarial review** (`wf_5bb5c713`, 4 refute lenses, 12 findings, all fixed or filed — the §B4 precedent while `safety-inspection` is rate-limited until Jul 18). Gates: vitest 427 · svelte-check 0 · cargo clean. **Boss test: the MARKER-FOUR gesture landed on disk at the close instant + clean-close stays instant + typing burst clean — PASS.** Commit `<this>`.
>
> ### NEWLY FILED
> - **PJ-110** *(HIGH → Group-1 top after the sweep · crash-recovery durability)* — **the write-ahead net's persistent layer (localStorage `constellation-wab`) is NOT durable across sessions**: proven live 2026-07-16 — a WebView2/Chromium leveldb MANIFEST/log inconsistency ("Creating DB since it was missing" at 18:56, then `Delete type=0 #3` at 19:16) silently discarded an entire session's net records. The graceful close no longer depends on it (PJ-103's fix), but the HARD-KILL recovery class (PJ-102/PJ-108's whole arc) still does. Fix direction: move the net's persistence to a Rust-side file via the hardened `atomic_write` (app-data or per-universe `.constellation/`), localStorage demoted to a same-session cache. Needs its own migration (write-path + boot-recovery + PJ-108 preserveNet semantics cross the boundary). Evidence: `lab/reports/pj103-evidence-000003.log` + SESSION-LOG-2026-07-16 session 2.
- **PJ-111** *(MED · design-needed)* — `flushOutgoing` (the nav/close flush primitive) carries NO `isCascading`/`isReseeding` gate, unlike every editor-side write path — a model dirtied mid-cascade from a non-frozen surface and flushed at close/nav could write its pre-cascade body over the walker's rewrite (review finding, PLAUSIBLE, narrow). Adding the gate naively changes nav-abort semantics (a skipped flush must not let openNoteTab-reuse proceed to replace the model) — needs the design pass, not a drive-by.
- **PJ-112** *(MED · residual exit paths)* — process-exit paths that BYPASS the close-flush handshake: **OS shutdown/logoff** (tao 0.34.6 does not route `WM_QUERYENDSESSION`/`WM_ENDSESSION` through CloseRequested — framework-level; the handshake never runs) and any future `exit()` caller. The updater `relaunch()` is closed (this commit). Direction: a periodic dirty-model → net policy or a tao upstream watch; document the residual in the manual meanwhile.
- **PJ-113** *(LOW)* — close-time flushed notes get an AWAITED FTS reindex (this commit) but embeddings stay fire-and-forget → bounded semantic-search staleness for those notes until their next save. Fold into PJ-110/Rule-8 work or accept.
>
> ### NOTES / CORRECTIONS
> - **PJ-086 — RE-EXAMINE before working** *(SO#8 flag)*: filed as "switchTab never flushes the outgoing dirty model → last ≤1.5s lost on quit". The PJ-103 live arc proved the outgoing pane's TEARDOWN flush persists a plain tab switch (2/2), and the close-flush now sweeps all dirty models at quit anyway. The residual (if any) is hard-kill-only + split-mode edges. Cross-check live before any build.
> - **The 2026-07-14 sweep register's PJ-103 mechanism claim** ("the staleness guard drops the teardown flush; model dirty in RAM for the rest of the session") is CORRECTED by the live arc above — the register stays as the historical record; this entry is the correction. Its "~30s" bound derived from the idle-save belt, which never fires for the claimed scenario; the REAL loss bound was the sub-debounce tail (≤1.5s of typing) + the net's fragility (unbounded).
> - **Charter drift FIXED (this commit):** the 2026-07-14 whole-app sweep register was never appended to the Charter although PJ-102–105 carried "Open · Charter" — the Charter now references both registers (2026-07-14 + this close's stand-in review).
>
---

*(Prior preambles v1.0–v1.33 + full history follow below; also durable in the versioned files.)*
**Version 1.33 | 2026-07-16**

> **What changed in v1.33** (**PJ-106 — the Arabic/RTL typing & navigation /migration — CLOSED** (pending only the Jul-18 per-cycle sweep): C2 docs shipped ×15, the Phase-4 audit ran (one drift FAIL → fixed same-pass, WA#6), §A4 closed SUBSUMED via the live Reproduce-First gate, and the Boss's callout split-box report fixed the same day. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — Jul 18, 4am: the per-cycle whole-app `safety-inspection` sweep** (the PJ-106 C3 cycle boundary + the §B4 post-gate + the audit's toolbar-Ctrl+Shift-click mousedown-disarm edge to check). **Then PJ-103** (app-close never flushes dirty background models — the next Group-1 APP-KILLER).
>
> **Then Group 1:** PJ-104 (→ PJ-072) · PJ-105 · PJ-098/093/086/099/085+073/074/083/087+075/076/077/094–097/100/101/002. **Group 3:** the PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> ### CLOSED since v1.32
> - **PJ-106 — CLOSED 2026-07-16** (the full arc: Part A §A1/§B0/§A3/§A2/§A5 + Part B §B1/§B2/§B3/§B4, every increment Boss-live-validated; open since v1.29). The close work: **C2** — NEW help topic "Writing in Arabic and Mixed Scripts" ×15 locales (`wf_e588176b`, 14 native translations; every drifted manual's RTL section found + extended in place) + English User Manual §18 + **LL-034** (bidi has TWO engines — render fixes without motion are half the recipe). **C3 audit** (`wf_97aab837`): all 6 INVs HOLD · migration-path 5/5 PASS (flag-off contract pinned: `RTL_MOTION_ENABLED=false` strips motion/keymaps/gesture only; §B4 marks persist as bytes, render-honored) · drift **FAIL → FIXED**: a §B4 mark before `[!` severs a callout (7 mark-blind parsers) → callout HEADERS in §B4's skip list; + the Boss's independent report ("why is the callout formatted this way?") fixed at the root — `detectLineDir` strips the `[!type]` token so an Arabic-titled callout header renders RTL as one coherent box. **§A4 — SUBSUMED**: the Reproduce-First gate on the live app ("callout caret pass") fired no repro → not built, per the standing rule. **C1 (CM6 bump) — dropped as unneeded** (no API need surfaced; a bump without need violates Constraint-as-Design). Gates at close: vitest 427, svelte-check 0. Commit `<this>`.
>
> ### NEWLY FILED
> - **PJ-109** *(LOW · polish)* — A5's optional **Mod-ArrowLeft/Right Windows word-hop** (`cursorGroupBackward`/`cursorGroupForwardWin`) was planned as optional and never landed; plain logical arrows shipped. Bind if the Boss ever asks for Word's Ctrl+arrow word-jump semantics on bidi text.
> - *(Jul-18 sweep scope note, not a PJ: the §B4 gesture's arm survives a mousedown OUTSIDE the editor — a focus-preserving toolbar Ctrl+Shift+click could fire the flip on release; consider a window-level disarm.)*

---

*(Prior preambles v1.0–v1.32 + full history follow below; also durable in the versioned files.)*

**Version 1.32 | 2026-07-16**

> **What changed in v1.32** (**PJ-106 §B4 — the Right/Left-Ctrl+Shift paragraph direction switch — SHIPPED + Boss-validated (Boss directed proceeding ahead of the Jul-18 inspection reset; a 24-agent adversarial review stood in and confirmed 16 findings, 15 fixed pre-commit, the 16th resolved by Boss ruling). PART B IS COMPLETE.** SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — the PJ-106 CLOSE steps:** **C2** (docs/help/User Manual ×15 for the whole PJ-106 arc — Part A + B1–B4) can run now; **C3** (Phase-4 audit + the per-cycle whole-app sweep) waits for the **Jul 18 4am** `safety-inspection` reset — that sweep doubles as the §B4 post-gate confirmation; **C1** (CM6 minor bump) optional; **§A4** stays Reproduce-First-gated (build ONLY if the Arabic-callout caret repro fires live). **Then PJ-103** (app-close never flushes dirty background models — the next Group-1 APP-KILLER).
>
> **Then Group 1:** PJ-104 (→ PJ-072) · PJ-105 · PJ-098/093/086/099/085+073/074/083/087+075/076/077/094–097/100/101/002. **Group 3:** the PAUSED SS-Cockpit Parts B–F. Groups 2/4/5 unchanged.
>
> ### CLOSED since v1.31
> - **PJ-106 §B4 — DONE 2026-07-16 (Boss-validated live; Part B thereby COMPLETE: §B1/§B2/§B3/§B4).** Right-Ctrl+Shift → the caret's blank-delimited paragraph 100% RTL; Left-Ctrl+Shift → 100% LTR; persisted as an invisible RLM/LRM at each content line's CONTENT START (after list/quote/heading markers — plain-text, portable, File-Over-App). Fires on press-and-RELEASE with no intervening key (the Windows modifier-hotkey convention), Ctrl side decides, `KeyboardEvent.code` via **`domEventObservers`** — the review's **APP-KILLER catch**: with plain `domEventHandlers`, keymap-consumed chords (Ctrl+Shift+S!) never reach the disarm and their release would have force-flipped the paragraph. NEW `src/lib/editor/paragraphDir.ts` (pure change-computer, exported for tests) + `bidiPlugin.ts` mark-precedence (structured `BLOCK_PREFIX_RE` strip — a `- [x]` checkbox's x is never the first strong char) + content-based same-frame rebuild (undo/redo/paste/adopt all covered). **The review-earned guards:** doc-leading YAML frontmatter untouchable + caret-in-frontmatter no-ops (the merge view edits the FULL file — a marked `title:` key = silent metadata loss); content-leading `#tag` lines skipped (a mark before # kills the tag in index/tasks/Obsidian); CommonMark-aware fence parity (opener char+length; quoted/listed fences; indented-code shapes never marked); `[[note#heading]]`/`![[note#heading]]` identity normalized against marks (livePreview + store `extractHeadings` + Rust `get_note_headings`); caret maps AFTER the mark (assoc 1); `isolateHistory` one-undo-step; AltGr belt; digit-only lines are content; link-ref/footnote definitions skipped. **Boss test:** flips/returns/undo/persistence-across-restart PASS; Ctrl+Shift+S/L don't flip PASS; **Step-6 OS ruling: Boss switches language via Win+Space — no collision, already-written paragraphs stay put → ships as-is.** Gates: 30 §B4 recipes, vitest 425, svelte-check 0. Review `wf_34d75a00` (24 agents, 4 refute-first lenses + per-finding skeptics). Commit `<this>`.
>
> ### NEWLY FILED / NOTES
> - **Documented limitation (LOW, accepted):** a line *starting* with `_emphasis_` renders literal underscores in some EXTERNAL renderers when marked (CommonMark `_` flanking vs a preceding Cf char). In-app rendering unaffected.
> - **§B4 post-gate:** the Jul-18 per-cycle sweep (C3) re-inspects the B4 diff with the automated `safety-inspection` — the standing promise made when the Boss directed proceeding early.

---

*(Prior preambles v1.0–v1.31 + full history follow below; also durable in the versioned files.)*

**Version 1.31 | 2026-07-16**

> **What changed in v1.31** (**PJ-108 — the read-only-second-screen recovery-net destroyer APP-KILLER — FIXED + proven by a LIVE crash-recovery Boss test; PJ-106 Part-B SELECTION half SHIPPED (§B1/§B2/§B3, each Boss-validated); §B4 DEFERRED to Jul 18 by Boss ruling.** SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-103** *(APP-KILLER · app close never flushes dirty background models, `+layout.svelte` final-flush listener)* — the next Group-1 item now that PJ-108 is closed. **OR PJ-106 §B4** (the Right-Ctrl+Shift → paragraph 100% RTL / Left-Ctrl+Shift → 100% LTR override via an invisible RLM/LRM mark — the one Part-B step that WRITES to the note) **when the `safety-inspection` weekly limit resets Jul 18 4am** — Boss's call at the next session. §B4 pre-work banked: WebView2 spot-checks needed for `KeyboardEvent.code` Left/Right-Ctrl detection + the OS-eats-Ctrl+Shift fallback (Plan §B4 + amendment SI4-06).
>
> **Then Group 1:** PJ-104 (universe active_path → PJ-072) · PJ-105 (template raw-write) · PJ-098/093/086/099/085+073/074/083/087+075/076/077/094–097/100/101/002. **Group 3:** the PAUSED SS-Cockpit Parts B–F (resume Plan §6). Groups 2/4/5 unchanged.
>
> ### CLOSED since v1.30
> - **PJ-108 — DONE 2026-07-16 (APP-KILLER, Boss-validated LIVE).** The bug (verified against live code): the SS is a display-only window with its OWN `openTabs` but a SHARED localStorage recovery net — every SS note-open ran `openNoteTab → resolveNoteContent`, which CONSUMED the net (`clearWriteAhead`) with no writable editor to re-stash it → an unsaved save-failed note's ONLY recovery copy silently destroyed. **Fix — Solve-the-Class:** a store-level `displayOnlyWindow` flag (set once at SS init) makes EVERY `openNoteTab` in that window default `preserveNet` — covering all three traced doors at once (Dashboard note-click `SecondScreenPage:1261`, split-companion task file-link `TasksPanel:87`, workspace-restore replay `:717` — the last found DEAD in the shipped app: `sendWorkspaceRestore` has zero callers) plus any future SS call site; + `handleLinkClick` passes `readOnly` as `preserveNet` (belt for main-window read-only mounts) and never CREATES a note from a read-only display. Adversarial completeness review refuted `closeTab`/`switchTab`/peek/adopt as SAFE (no net mutation). **Reproduce-First:** Recipe RO (`tests/mig-076/readonlyLinkPreservesNet.test.ts`, 5 tests, RO2 RED pre-fix). vitest 395 · svelte-check 0. **Boss live test (the full crash-recovery arc):** real write-lock → red banner → SS task-link click (the vulnerable door) → force-kill with disk verified edit-free → reopen recovered the edit on screen → durable save landed it on disk. Commit `<this>`.
> - **PJ-106 Part B — SELECTION HALF SHIPPED + Boss-validated 2026-07-15** (3 commits, each after a Boss live PASS): **§B1** paragraph navigation Ctrl+↑/↓ (+Shift extend), Word convention, direction-blind pure-offset `paragraphNav.ts` (`c0d668fc`) · **§B2** select line Ctrl+L / paragraph block Ctrl+Shift+L, both TEXT-only per the §B0 rule, incl. overriding CM6's `Alt-l` trailing-newline RTL bug; select-page/select-all verified pre-existing (`10abf799`) · **§B3** select sentence Ctrl+click / Ctrl+Shift+S via `Intl.Segmenter` (UAX #29: breaks ؟ ۔ ! . — NOT ؛; no decimal false-break; Boss-confirmed the keyboard command fires on the Arabic layout) (`53b22c07`). All three surfaces (NotePane/FocusPane/ConflictMergeView) behind `RTL_MOTION_ENABLED`; 40 pj-106 tests. **The Boss's Round-1 selection list is fully covered** (word/sentence/line/paragraph/page/all). **PJ-106 stays OPEN for §B4** (deferred above), **§A4** (Reproduce-First-gated on the callout-caret repro), and the C1/C2/C3 close steps (CM6 bump · docs/help ×15 for the new commands · Phase-4 audit + per-cycle sweep).
>
> ### NEWLY FILED
> - *(none)* — the PJ-108 completeness review surfaced no new items; its one advisory (a future custom `onLinkClick` prop would bypass the NoteEditor belt) is already fully mitigated by the window-level flag, which sits below any link handler.

---

*(Prior preambles v1.0–v1.30 + full history follow below; also durable in the versioned files.)*

**Version 1.30 | 2026-07-15**

> **What changed in v1.30** (**PJ-106 Part-A CORE shipped + Boss-validated (direction + logical arrows + triple-click); the imported-note Home-caret bug diagnosed → PARKED (Boss "closed"); one new APP-KILLER filed from the safety sweep.** Solo the rest of the week — the `safety-inspection` workflow hit its weekly agent limit, resets Jul 18. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-106 Part B** *(in-flight · Boss actively working it)* — the Arabic/RTL **selection + paragraph-direction** half, built on the landed Part-A core. Remaining: **§A4** isolate ranges (symptoms ②/③ re-confirmed PASS by the Boss, so §A4 may reduce to the callout-caret repro only — Reproduce-First-gated); **Part B** — select sentence (Ctrl+click, Intl.Segmenter Arabic terminators) / paragraph / page, and the **Right-Ctrl+Shift → paragraph 100% RTL, Left-Ctrl+Shift → 100% LTR** override via an invisible RLM/LRM mark (KeyboardEvent.code Left/Right detection). Plan `docs/PJ-106-RTL-Typing-PLAN.md` §B; symptoms doc Rounds 1–6.
>
> **Then Group 1 (safety):** 1. **PJ-108** *(NEW · APP-KILLER)*. 2. **PJ-103** (app-close never flushes dirty background models). 3. **PJ-104** (universe active_path outside registry → PJ-072). 4. **PJ-105** (template-insert raw-write bypass). 5. PJ-098/093/086/099/085+073/074/083/087+075/076/077/094–097/100/101/002. **Group 3:** the PAUSED SS-Cockpit Parts B–F (resume Plan §6). Groups 2/4/5 unchanged.
>
> ### CLOSED / ADVANCED since v1.29
> - **PJ-106 Part-A CORE — SHIPPED + Boss-validated 2026-07-14/15** (4 commits, each after a Boss live-test PASS): **§A1** per-line direction → caret engine (`perLineTextDirection` + deterministic base replacing `dir='auto'`, all 3 editable surfaces; rollback lever `rtlFlag.ts`); **§B0** triple-click selects the line's TEXT not the trailing newline (`tripleClickLine.ts`); **§A3+§A2** Enter on an RTL line puts the caret on the RIGHT (empty/neutral line always stamped `dir='rtl'` + structural-change synchronous rebuild); **§A5** logical (Word-style) arrows across bidi boundaries (`rtlMotion.ts`, lens-widget caret-trap avoided via a scoped injected skip source, Rule-6-safe for Focus). Gates: svelte-check 0, vitest 365 (+11 pj-106). Symptoms ①②③ Boss-confirmed resolved. **PJ-106 stays OPEN for Part B (above).**
>
> ### NEWLY FILED
> - **PJ-107** *(NEW · LOW/polish · PARKED by Boss "closed" 2026-07-15)* — **imported Arabic notes render the Home caret INVISIBLE.** Home is functional (type/navigate/select work) — only the 1.5px blinking BAR isn't painted; End fine; Latin fine; created-in-app notes fine. **Trigger diagnosed** (drove the release app via computer-use + filesystem diff): the imported note's rich **16-field Obsidian frontmatter** (body is byte-identical — the CM6 doc is body-only, `NoteEditor.svelte:471` — proven innocent by the Boss's own paste test). Ruled out: body/heading/wrapping/callout/141-char-URL. **Exact pixel mechanism NOT nailed** — the caret can't be resolved in screenshots and the release binary has devtools disabled; naming it needs an **instrumented/dev build** reading `coordsAtPos` at the Home position (Reproduce-First; no guessing). Full record: `lab/reports/PJ-106-RTL-Symptoms-BossReported.md` Round 6.
> - **PJ-108** *(NEW · APP-KILLER · from the safety sweep `wf_63ab538f`)* — **a second-screen `[[wikilink]]` click silently destroys the crash-recovery net of an unsaved, save-failed note.** `NoteEditor.handleLinkClick` is the one handler with no `readOnly` belt → an SS wikilink click (or SS workspace restore) runs `openNoteTab → resolveNoteContent`, which **consumes + `clearWriteAhead`s** the shared-localStorage write-ahead buffer; but the SS mounts read-only, so its teardown flush (`handleFlush`/`handleSave`) early-returns → the net is gone and never re-stashed, disk still holds the pre-edit body, nothing surfaced. **Fix: read-only hosts open with `preserveNet` (or never let a read-only surface reach the consuming `openNoteTab`).**
>
> ---
>
> *(Prior preambles v1.0–v1.29 + full history follow below; also durable in the versioned files.)*

**Version 1.29 | 2026-07-14**

> **What changed in v1.29** (**PJ-102 — the manual-reopen recovery APP-KILLER — FIXED as a three-part arc, all Boss-live-tested on a REAL locked file.** The live test itself (a write-blocking lock I held on the note while the Boss typed/closed/reopened/switched) surfaced two deeper defects mid-arc, both fixed in-pass (WA#6): **(a)** `ensure_cid_cn` swapped net-recovered content back to stale disk after the net was consumed → the adopt is now gated on the in-hand content LACKING a cid_cn (Recipe S1); **(b)** the recovered model was born "clean" on content disk never had — the lie behind the Boss-hit switch-away vanish + the false-healing banner → a net-recovered open is now **born DIRTY with the TRUE disk baseline** (`markRecoveredFromNet`), `adoptDisk` gained the **phantom-event guard** (`disk === baseline` → refuse), and the session-restore path gets the true baseline too (the adversarial review's confirmed Q4 hole — a phantom event was destroying the PRESERVED net via clearWriteAhead-on-adopt); **(c)** Boss-requested mid-arc: the banner's two explicit **locked-file exits** — **Save a copy** (verbatim sibling copy, fresh identity: cid stripped + title suffixed so the TAB distinguishes it [Boss remark], localized suffix ×15, opens in a NEW tab) and **Discard…** (two-step inline confirm; the deliberate counterpart of the silent discard this arc eliminated). **Recipe S (9 tests, RED→GREEN)**; vitest 350; svelte-check 0; adversarial reviews ×2 (predicate SAFE 5/5; born-dirty SAFE with Q4/Q5 confirmed → fixed in-pass). **Boss live-tests: recovery-reopen PASS · switch-away/return PASS · honest banner PASS · Save-a-copy PASS · Discard PASS · tab-label re-check PASS.** Commit `<this>`. Orientation v3.50. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-106** *(NEW · Boss-directed 2026-07-14)* — **the Arabic/RTL typing & navigation overhaul.** Boss: "fix the Arabic (RTL) typing logic — Home/End are not acting as they should; I couldn't navigate through a line or a paragraph, or select a word, a sentence, a line, a whole paragraph, or a whole page. I am struggling to write, and it is even worse if it is bilingual (Arabic note containing Latin characters)." Scope: the CM6 editor core (bidiPlugin, cursor/selection motion, Home/End/word/paragraph/page commands) across NotePane + FocusPane — Language-First is an architecture principle; this is core, not polish. **Needs the full `/migration`** (editor-core + cross-surface; Architect with WA#5 prior-art research — CM6 bidi docs, Obsidian/VS Code RTL behavior — then Boss picks, design-stage inspection, build, staged Boss tests on real bilingual notes).
>
> **Then Group 1:** 1. **PJ-103** (app-close never flushes dirty background models — APP-KILLER). 2. **PJ-104** (universe active_path outside the registry path → fold into PJ-072). 3. **PJ-105** (template-insert raw-write bypass). 4. PJ-098/093/086/099/085+073/074/083/087+075/076/077/094–097/100/101/002. **Group 3:** the PAUSED SS-Cockpit Parts B–F (resume at Plan §6). Groups 2/4/5 unchanged.
>
> ### CLOSED since v1.28
> - **PJ-102 — DONE 2026-07-14** (see the preamble; the arc: predicate fix + born-dirty/true-baseline/phantom-guard + restore-path baseline + the two banner exits + the copy retitle). **Residuals noted, LOW:** a genuine external edit landing on a restored-from-net CLEAN tab still adopts (net preserved-then... rare²; the restore contract stays born-clean per Gate #8); a disk-unreachable-at-open recovery may raise one spurious-but-safe `.conflict` sidecar. Both recorded in the Charter register.
>
> ### NEWLY FILED
> - **PJ-106** — the Arabic/RTL typing & navigation overhaul (Boss-directed). **Open · ► Next action · /migration.**
>
> ---
>
> *(Prior preambles v1.0–v1.28 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.28.md`.)*

**Version 1.28 | 2026-07-14**

> **What changed in v1.28** (**SS-Cockpit /migration Part A SHIPPED + Boss-validated; then PAUSED by Boss ruling — the sweep's 2 NEW APP-KILLERs jump the queue.** Part A (`45d20b88`→`9535072f`): the conservative cut — the dead ep-clone/OrgChart/Map companions, the 4 RO note copies (A9 lists stay browsable), the 9 stub facets + tab bar (lens toggle re-homed, INV-10), the A1 flag repoint, ~930 lines gone; + the empty-desk follow-up (the SS now empties when the last main tab closes — Boss Stage-1 remark, fixed same-pass). All gates green; Stage-1 + re-check Boss-tested PASS. The §1-boundary whole-app sweep (`wf_8b0a5104-6e8`, 83 agents, 55 confirmed — full register `lab/reports/SWEEP-REGISTER-2026-07-14-wf_8b0a5104.md`; the Part-A diff itself: ZERO findings) confirmed 5 pre-existing APP-KILLERs → the 2 NEW filed below; **Boss ruled: fix them BEFORE the Cockpit zones.** SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-102** *(NEW · APP-KILLER)* — **manual reopen destroys the crash/failed-save recovery copy.** `openNoteTab` consumes the identity-proven write-ahead net via `resolveNoteContent` (no preserveNet), then the `ensure_cid_cn` step (`store.ts:2039`; Rust `canonical.rs:1226-1227` returns FULL DISK content when cid_cn already exists) replaces the recovered content with stale disk — screen, net, and disk all lose the user's last edits, zero surfacing, 100% reproducible whenever net ≠ disk on manual open. The app's own documented recovery route (closeTab's contract) is structurally dead. **Reproduce-First, then fix (likely: preserve-net on manual open + never let ensure_cid_cn override recovered content — it only needs to inject a missing cid).**
>
> **Then — PJ-103** *(NEW · APP-KILLER)* — **app close never flushes dirty BACKGROUND models.** The MIG-100 graceful-close handshake (`session:final-flush`, `+layout.svelte:3436`) persists only session.json; a note edited then switched away from holds its only current copy in the in-memory model (the outgoing pane's teardown flush is dropped by the staleness guard) → up to ~30s of typing vanishes at quit. Universe-switch DOES sweep (`flushAllDirtyTabs`) — app close is the one departure with no sweep. **Fix locus: `flushAllDirtyTabs('final_flush')` inside the final-flush listener before the ack (+ the same for any OS-kill-resistant net gap).**
>
> **Group 1 — Safety & correctness** *(after PJ-102/103)*
> 1. **PJ-104** *(NEW · APP-KILLER-class)* — `open_existing_universe`/`link_library_as_universe` flip `UniverseState.active_path` directly without the registry write path (`universe.rs:1099`/`:1031-1036`/`:1183-1185`) — the PJ-072 registry-mystery's likely mechanism. Fold into the PJ-072 investigation.
> 2. **PJ-105** *(NEW · HIGH)* — template-insert fallback writes `tab.content + template` via raw `invoke('write_note')` when no active CM6 DOM node is found (`+layout.svelte:4767`) — bypasses the single-ownership model (a stale-content clobber vector). Route through the model or drop the fallback.
> 3. **PJ-098** — OrgChart drag-drop raw move_item (HIGH). 4. **PJ-093** — reindex-skip when db None. 5. **PJ-086** — switchTab flush gap. 6. **PJ-099** — loadTabHistoryEntry await-window re-dirty. 7. **PJ-085+073** — frontmatter/YAML (the sweep re-confirmed `yamlDoc.ts:150` ikhtilāf collapse as HIGH). 8. **PJ-074/083/087+075/076/077** (the sweep re-confirmed save_pulse=PJ-075, collect_library_notes=PJ-077). 9. **PJ-094–097/100/101/072/002.** *(The sweep's ~50 non-app-killer confirmations map item-by-item at the next full reconciliation — register linked above; most re-confirm existing PJs.)*
>
> **Group 3 — THE PAUSED MIGRATION:** **SS-Cockpit Parts B–F** (the pre-B A2 hardening → the 2 cache keys → Art-Director design → the HEALTH tension board → WHERE-lite → docs/audit/close). Plan approved as amended (`docs/SS-Cockpit-Migration-PLAN.md`); resume at §6 after PJ-102/103. *(This is a Boss-directed pause, not a park — resume is the standing next feature work.)*
>
> **Groups 2/4/5** — unchanged (PJ-084/080/078/079/069-remainder · PJ-067 · MIG-096/088 · Backup & Recovery · polish/docs).
>
> ### CLOSED since v1.27
> - **SS-Cockpit /migration PART A — SHIPPED + Boss-validated 2026-07-14** (§0 `45d20b88`, §1 `fbb84c2e` −747, §2 `dd576058` −180, §3 `67a886f5`, empty-desk `9535072f`). The SS is the clean read-only cockpit (lenses full-glass + Pin/Follow + header lens toggle + honest empty state). Zero in-diff sweep findings. PJ-068's cut half is thereby DONE; the zones half continues as the paused Parts B–F.
>
> ### NEWLY FILED — PJ-102 → PJ-105 (from the §1-boundary whole-app sweep)
> - **PJ-102** *(APP-KILLER)* — reopen destroys the write-ahead recovery copy (`store.ts:2039` + `canonical.rs:1226`). **Open · Charter · ► Next action.**
> - **PJ-103** *(APP-KILLER)* — app close never flushes dirty background models (`+layout.svelte:3436`). **Open · Charter · second.**
> - **PJ-104** — universe active_path flipped outside the registry write path (`universe.rs:1099`) → fold into PJ-072. **Open · Charter.**
> - **PJ-105** — template-insert raw-write bypass (`+layout.svelte:4767`). **Open · Charter.**
>
> ---
>
> *(Prior preambles v1.0–v1.27 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.27.md`.)*

**Version 1.27 | 2026-07-13**

> **What changed in v1.27** (**PJ-090 — the SS Tasks-panel cross-window clobber — RESOLVED BY CUT, not by fixing the toggle.** SO#8 caught a scope error: the ledger's "SS Tasks-panel onToggle clobber" rested on code that the read-only Knowledge-Cockpit migration (PJ-068 v2) had already superseded — the Boss's own screenshot showed the current SS Cockpit, whose Tasks facet is a STUB ("wired in the next pass"), so a task cannot be toggled on the default SS at all; the only reachable SS toggle was the split-companion mode (split-view only). The Boss then reframed it as a CONCEPT question and convened the Art-Director-&-Team honest audit (`wf_043756ee-352`, 9 agents). **Verdict: the SS Cockpit is ~90% stubbed duplication of the main window's right sidebar (same i18n keys); only "Links" (the note-graph lenses) + the Pin dial are genuine complements.** The Tasks toggle is a Display-Not-Domain breach; the earlier cross-window-broadcast fix would "make an illegal write work." **Boss ruling: CUT the toggle (discard the broadcast fix).** DONE — `TasksPanel` gained a `readOnly` prop; both SS mounts are now `readOnly` (Display-not-Domain); the `toggleTaskReconciled` import + the two onToggle write handlers removed; the broadcast fix + Recipe R reverted. svelte-check 0, vitest 341. **Boss live-tested (main-window toggle still works; SS checkboxes read-only) — PASS.** Commit `<this>`. **PLUS — Boss ruling: re-conceive the SS as the full THREE-ZONE COCKPIT via `/migration`** (the ► Next major work). SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — the Second-Screen THREE-ZONE COCKPIT `/migration`** *(Boss-directed feature, 2026-07-13)*. Re-conceive the read-only SS around the ratified "Presenter Display" concept: **Move 1** — cut the 9 stub mirror facets + the tab bar + the (now read-only) Tasks facet + the dead/legacy companions (editor-panels `!COCKPIT_ENABLED` clone, OrgChart clone, disabled-Map companion, read-only note copies); keep the note-graph lenses + the Pin dial. **Move 2** — build the three zones WHERE (you-are-here Universe locator) / HEALTH (whole-corpus tension & living-link health board — the verified gap; tasks fold in as an urgency SIGNAL, never a list) / DECISION (the graph + connection space); the time-map is the finale. Direction ruled: **full three-zone cockpit** (2 of 3 reviewers). Design doc: `docs/concept-papers/PJ-068-v3-SS-Honest-Audit-2026-07-13.md` (+ PJ-068 / PJ-068-v2 concept papers). Crosses SS↔main sync + a write path → full `/migration` (Architect → Boss picks/ratifies → design-stage inspection → Build → Audit). *(This continues/closes PJ-068.)*
>
> **Group 1 — Safety & correctness** *(resume after / alongside the SS migration)*
> 1. **PJ-098** *(HIGH)* — OrgChart drag-drop raw `invoke('move_item')` bypasses the `moveItem` wrapper → stale open-tab path + aux-state divergence (`OrgChart.svelte:254`). *(Was the ► Next before the SS pivot.)*
> 2. **PJ-093** — reindex silently skipped when `state.db` is None + reindex-error swallow.
> 3. **PJ-086** — switchTab flush gap (HIGH). 4. **PJ-099** *(MED)* — `loadTabHistoryEntry` post-flush await-window re-dirty (`store.ts:1295`). 5. **PJ-085 + PJ-073** — frontmatter/YAML round-trip (HIGH; `yamlDoc.ts:150` nested-object-list ikhtilāf). 6. **PJ-074** — durable rename + folder-rename descendant cascade + link archive/unarchive TARGET aggregate. 7. **PJ-083** — cascade sync-clear hazard. 8. **PJ-087 + PJ-075/076** — persisted-JSON non-atomic. 9. **PJ-077** — sync-walk → async. 10. **PJ-094/095/096/097 + PJ-100/PJ-101**. 11. **PJ-072/002**.
>
> **Group 2/3** — unchanged (PJ-084/080/078/079/069-remainder · PJ-067 · MIG-096/088 · Backup & Recovery). **PJ-068 is now the active SS migration above (no longer "parked").**
>
> ### CLOSED since v1.26
> - **PJ-090 — SS Tasks-panel cross-window clobber — RESOLVED BY CUT 2026-07-13.** Not fixed — the toggle was CUT (Display-not-Domain). The Boss's screenshot + SO#8 revealed the ledger's premise rested on stale code (the read-only Cockpit's Tasks facet is a stub; the default SS can't toggle tasks; only the split-view split-companion mode could). The Art-Director-&-Team honest audit (`wf_043756ee-352`) ruled the SS Tasks toggle a Display-not-Domain duplication, and the cross-window-broadcast fix (adversarially-safe but conceptually wrong — it "makes an illegal write work") was **reverted**. Fix: `TasksPanel` `readOnly` prop; both SS mounts read-only; write handlers + import removed. svelte-check 0, vitest 341, Boss-tested PASS. Commit `<this>`. Orientation v3.48. **Lesson: SO#8 must verify a PJ against the RUNNING SS structure (the orientation Second-Screen §), not just the presence of the code — the toggle code existed but was unreachable in the current Cockpit. And: Concept-before-Function — the honest answer to a "fix the SS X" item can be "cut X, it violates the SS's read-only concept."**
>
> ### PROCESS NOTE
> - **The Art Director & Team own UX/UI design (Boss ruling 2026-07-10) — honored:** the SS concept question was answered by a multi-agent Art-Director workflow (census → 3 options → 3 adversarial honesty judges → synthesis), not hand-iterated solo. Its honest verdict (SS = 90% stubbed duplication) drove the CUT ruling + the three-zone-cockpit direction.
>
> ---
>
> *(Prior preambles v1.0–v1.26 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.26.md`.)*

**Version 1.26 | 2026-07-13**

> **What changed in v1.26** (**PJ-089 — the Index-panel preview two-writable-model silent clobber — FIXED (read-only peek).** The Index split-pane preview mounted a WRITABLE `NoteEditor` on a standalone `index_preview_*` tab whose unique id keyed a SECOND single-ownership NoteModel for the same path → two independent writable models → last-writer-wins silent clobber of the same note open in a real tab, no `.conflict` sidecar (the preview lives outside `openTabs`, so the watcher-adopt never reconciles it — the store's own invariant "path↔id is 1:1 under DEDUP" was violated). **Boss-picked fix (A of A/B): the preview is now a READ-ONLY peek** — `readOnly={true}` (the proven Display-not-Domain primitive the second screen uses) removes the second writer STRUCTURALLY (it can never write → can never clobber); an **"Open to edit"** button promotes the peek to a real single-owner tab via `openNoteTab` path-dedup (activates the existing tab if already open — never a duplicate). `/simplify` deepened it to a **lifecycle-owned `$effect` disposal** (keyed to `indexNoteTab?.id`) that frees the preview's model on any change/clear — the structural fix for a pre-existing model-Map leak a manual close could forget — plus a shared `leaveIndexForNote` helper. **Boss-test follow-up (WA#6):** wikilinks in the peek now behave like the Index note-list — **plain-click → the preview follows the link; Ctrl/middle-click → open a real tab + leave the Index** (no more silent background tab) — via a new optional `onLinkClick?` override on `NoteEditor` (default unchanged → every other mount byte-identical). **Reproduce-First:** `indexPreviewClobber.test.ts` (Recipe Q — RED clobber + RED-2 reconcile-gap + GREEN read-only invariant). Gates: svelte-check 0, vitest 341, per-build safety inspection = **0 in-diff findings**, focused adversarial review of the link increment = SAFE on all 6 vectors, `/simplify` applied. **Boss live-tested end-to-end: Stage 1 (read-only peek + Open-to-edit) PASS · link re-test (follow-in-peek + Ctrl-open) PASS · Stage 2 (no silent overwrite + no duplicate + close edges) PASS.** i18n `indexPanel.openToEdit` ×15. Commit `<this>`. Orientation v3.47. **Per-cycle whole-app safety inspection** (`wf_ca0d3aa9-3d6`, 34 agents, 14 confirmed): 10 map to existing backlog, **4 NEW filed → PJ-098–101**. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-090** (SS Tasks-panel toggle no-broadcast clobber, HIGH) — the next Group-1 safety item now that PJ-089 is closed. The three most recent APP-KILLER-class silent-loss items — PJ-091 (accept-truncate), PJ-092 (rename-cascade), PJ-089 (Index-preview clobber) — are all closed.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH). *(► Next action)*
> 2. **PJ-098** *(NEW · HIGH)* — OrgChart drag-drop move calls raw `invoke('move_item')` instead of the `moveItem()` store wrapper, skipping `migratePathKeyedAuxStateOnRename` + the `openTabs` repath/repathNoteModel → an open tab of a dragged note keeps a stale path and its aux state diverges (`OrgChart.svelte:254`).
> 3. **PJ-093** — reindex silently skipped when `state.db` is None + reindex-error swallow.
> 4. **PJ-086** — switchTab flush gap (HIGH). 5. **PJ-099** *(NEW · MED)* — `loadTabHistoryEntry` (Alt+←/→ history nav) flushes the outgoing model then AWAITS `read_note` before force-reseeding, so a keystroke in the post-flush await window re-dirties then is discarded by the reseed (`store.ts:1295`). 6. **PJ-085 + PJ-073** — frontmatter/YAML round-trip (HIGH; the sweep re-confirmed the `yamlDoc.ts:150` nested-object-list ikhtilāf collapse). 7. **PJ-074** — durable rename + folder-rename descendant cascade (sweep re-confirmed the folder-rename watcher-suppress defeat + link archive/unarchive TARGET aggregate). 8. **PJ-083** — cascade sync-clear hazard. 9. **PJ-087 + PJ-075/076** — persisted-JSON non-atomic (`review.rs:762` save_pulse re-confirmed). 10. **PJ-077** — sync-walk → async (`collect_library_notes` re-confirmed). 11. **PJ-094/095/096/097 + PJ-100** (below). 12. **PJ-072/002**.
>
> **Group 4/5 — smaller items:** PJ-094/095/096/097 (carried) + **PJ-100** *(NEW · MED)* — SenseMakingCanvas auto-save (`write_canvas`) swallows ALL errors on a 1000ms debounce with no save-health surface, no retry, no crash-net, no flush-on-destroy — the canvas's ONLY persistence path (`SenseMakingCanvas.svelte:147`) — and **PJ-101** *(NEW · LOW)* — SS peek preview shows a STALE prior version when a note is re-peeked after it changed on disk while peek was closed (`SecondScreenPage.svelte:427`).
>
> **Group 2/3** — unchanged (PJ-084/080/078/079/069-remainder · PJ-067/068 · MIG-096/088 · Backup & Recovery).
>
> ### CLOSED since v1.25
> - **PJ-089 — Index-panel preview two-writable-model silent clobber — DONE (read-only peek) 2026-07-13.** The bug: the Index split-pane preview (`handleIndexNoteClick` → `<NoteEditor tab={indexNoteTab}>`, `+layout.svelte`) mounted a WRITABLE editor on a standalone tab with a unique `index_preview_${Date.now()}` id. The single-ownership `models` Map keys by id (`noteModel.ts`), so an already-open note got a SECOND independent writable model → both wrote to one path → last-writer-wins silent clobber, invisible to `adoptExternalChangeIntoTabs` (it filters to `openTabs`; the preview is a standalone `$state`). **Fix — read-only peek (Boss ruling A):** `readOnly={true}` on the preview mount (NotePane: `EditorState.readOnly` + non-editable title/props; NoteEditor: every write callback early-returns, onDocChange never mutates the model) — a look-only peek that can never write; **"Open to edit"** promotes it to a real single-owner tab via `openNoteTab` path-dedup; a lifecycle-owned `$effect` frees the preview model on any change/clear (fixes a pre-existing leak); a shared `leaveIndexForNote` helper. **Link follow-up:** an optional `onLinkClick?` override on NoteEditor → the peek follows plain-clicked links / Ctrl-opens a real tab (no background-tab surprise). **Reproduce-First:** `indexPreviewClobber.test.ts` Recipe Q (RED clobber + GREEN read-only invariant). svelte-check 0, vitest 341; per-build safety inspection 0 in-diff; focused adversarial review of the link increment SAFE ×6; `/simplify` applied (lifecycle-owned disposal + shared helper + test cleanups). **Boss live-test: Stage 1 + link re-test + Stage 2 all PASS.** Commit `<this>`. Orientation v3.47. **Lesson: for the content-integrity class, removing the second WRITER (read-only) is the clean single-owner boundary — cheaper and safer than sharing one model across two writable views (the rejected Option B).**
>
> ### NEWLY FILED — PJ-098 → PJ-101 (from the per-cycle whole-app safety inspection `wf_ca0d3aa9-3d6`, 34 agents, 14 confirmed)
> - **PJ-098** *(HIGH)* — OrgChart drag-drop bypasses the `moveItem` store wrapper (raw `invoke('move_item')`) → stale open-tab path + aux-state divergence (`OrgChart.svelte:254`). **Open · Charter · Group 1.**
> - **PJ-099** *(MED)* — `loadTabHistoryEntry` post-flush await-window re-dirty → the keystroke is discarded by the force-reseed (`store.ts:1295`). **Open · Charter · Group 1.**
> - **PJ-100** *(MED)* — SenseMakingCanvas `write_canvas` auto-save swallows all errors, no net/retry/flush-on-destroy (`SenseMakingCanvas.svelte:147`). **Open · Charter · Group 4.**
> - **PJ-101** *(LOW)* — SS peek preview stale-on-re-peek (`SecondScreenPage.svelte:427`). **Open · Charter · Group 5.**
> - *(The other 10 confirmed map to existing PJs: folder-rename + link archive/unarchive = PJ-074 ×3, yamlDoc nested-object-list = PJ-073/085, reindex-no-db-guard = PJ-093, save_pulse non-atomic = PJ-075, collect_library_notes sync-walk = PJ-077, FocusPane-title-discarded + BacklinksPanel-linkMention-swallow + SS-companion-model-leak = existing LOW batch. The PJ-089 diff's OWN change: ZERO findings.)*
>
> ---
>
> *(Prior preambles v1.0–v1.25 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.25.md`.)*

**Version 1.25 | 2026-07-13**

> **What changed in v1.25** (**PJ-092 — the rename-cascade edit-loss/freeze APP-KILLER — FIXED, properly, via the full `/migration`.** After the focused-fix band-aid froze the app and was reverted (v1.24), PJ-092 was redone through all four `/migration` phases with a NEW standing step — a **design-stage safety inspection** before any code. Approach: **flush-gate-exclude** (Boss-picked) — a note whose unsaved edits can't be flushed at rename time is EXCLUDED from the on-disk link rewrite (matched by **file identity**, canonicalize+NFC, so the Arabic-root NFC/NFD hazard can't defeat it), so its file is never touched → no model↔disk divergence → no data-loss, no freeze; every other note cascades normally. Boss live-tested A1/A2/B1/B2 + a clean-binary sanity — all PASS. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-089** (Index-panel preview two-writable-model silent clobber, HIGH) — the next Group-1 safety item. Both this-session APP-KILLERs (PJ-091 accept-truncate, PJ-092 rename-cascade) and the earlier PJ-070/071/088 are now closed.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-089** — Index-panel preview two-writable-model silent clobber (HIGH). *(► Next action)*
> 2. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH).
> 3. **PJ-093** — reindex silently skipped when `state.db` is None + reindex-error swallow.
> 4. **PJ-086** — switchTab flush gap (HIGH). 5. **PJ-085 + PJ-073** — frontmatter/YAML round-trip (HIGH) + PropertyEditor forced-type + block-scalar-projected-empty. 6. **PJ-074** — durable rename + folder-rename descendant cascade + move_item review-reset. 7. **PJ-083** — cascade sync-clear hazard. 8. **PJ-087 + PJ-075/076** — persisted-JSON non-atomic cluster (+ `save_libraries` fsync). 9. **PJ-077** — sync-walk commands → async (`collect_library_notes` too). 10. **PJ-094/095/096/097** (below). 11. **PJ-072/002**.
>
> **Group 4/5 — smaller items:** PJ-094 (`moveItem` no flush-before-repath), PJ-095 (`NoteEditor` `saving` single-flight drops a debounced save), PJ-096 (`.conflict` sidecar write-failure swallowed), and **PJ-097** *(NEW · from the PJ-092 audit)*.
>
> **Group 2/3** — unchanged (PJ-084/080/078/079/069-remainder · PJ-067/068 · MIG-096/088 · Backup & Recovery).
>
> ### CLOSED since v1.24
> - **PJ-092 — rename-cascade edit-loss/freeze APP-KILLER — DONE via `/migration` 2026-07-13.** The bug: the wikilink rename cascade could silently lose an OPEN, dirty backlink note's unsaved edits if its `.md` was locked at the instant of the rename (`flushAllTabsInLibrary` swallowed the flush outcome; the walker rewrote the stale disk; `reloadTabsFromDisk` force-reseeded the model clean). **Fix — flush-gate-exclude:** `flushAllTabsInLibrary` now flushes each dirty tab through the BOUNDED re-flush loop and RETURNS the not-durably-flushed paths; `handleRenameComplete` passes them to the Rust walker `update_links_on_rename`, which SKIPS them by FILE IDENTITY (`path_identity_key` = `canonicalize` + NFC + slash + Windows-lowercase) — so an unflushable note is never rewritten on disk, never in `result.rewritten`, never force-reloaded (INV-1/4/5). Plus: a fail-CLOSED belt (NFC-folded) at both reload sites (the drift-audit fix), a focus-aware reseed (H3), the alias-refresh fix (H5), and the 4 sibling flush-then-reload callers gated via the shared `flushOpenTabOrAbort` (bounded loop, H2). **Process:** the full four phases + a design-stage safety inspection that caught 5 hazards BEFORE code (incl. the Arabic-NFC path-match, the H2 await-window race, the H3 focus-blind reseed, the 4 siblings). `/simplify` caught 2 more contract gaps (belt-not-NFC, siblings-not-bounded) → fixed. Audit: 11/11 invariants HOLD, migration-path PASS, 1 drift (cascade:rewrote listener bypassed the belt) → fixed. Verify: `renameCascadeExclude.test.ts` (3) + Rust `cascade_walker_tests` (incl. NFC/NFD identity, separator-mismatch exclude, empty-exclude rollback); svelte-check 0, vitest 338, cargo walker 16. **Boss live-test:** A1 (normal rename), A2 (locked-note-protected + others-update, on the real Arabic-root universe), B1 (Focus mode), B2 (restart recovery), + clean-binary sanity — all PASS. Commit `<this>`. Orientation v3.46. **Lesson banked: `/migration` (with the new design-stage inspection) is what a rename-cascade/reactive-lifecycle change requires — the reverted band-aid is the counter-example.**
>
> ### NEWLY FILED — PJ-097 (from the PJ-092 Phase-4 audit + safety-inspection)
> - **PJ-097** *(LOW/MED)* — during a rename cascade, **FocusPane is not covered by the `CascadeFreezeOverlay`** the way NotePane panes are, so a keystroke typed into a *rewritten* backlink's Focus view during the cascade window can be discarded by the subsequent `focusReseed` remount (a re-type-during-cascade race; the NotePane equivalent is blocked by its overlay). PRE-EXISTING (FocusPane never had the overlay); PJ-092's H3 reseed is strictly an improvement over the prior silent stale-revert; contrived trigger. **Open · Charter · Group 4/5 · add a FocusPane freeze-overlay + an Editor-Surface-Gate #4 harness assertion.**
>
> ### PROCESS CHANGE (Boss-endorsed 2026-07-13)
> - **The Safety Inspection now reviews the DESIGN, not just the code.** A `/migration` runs a **design-stage safety inspection on the Plan** (adversarial, refute-first, the app-killer taxonomy) BEFORE any code — the highest-leverage place to catch a design flaw. On PJ-092 it caught 5 hazards for free, including the exact Arabic-NFC path-match class that would have silently reintroduced the data-loss. This pairs with the per-build (diff) + per-cycle (whole-app) inspections.
>
> ---
>
> *(Prior preambles v1.0–v1.24 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.24.md`.)*

**Version 1.24 | 2026-07-13**

> **What changed in v1.24** (**PJ-092 — REVERTED and RE-OPENED.** The v1.23 "dirty-guard" fix (`reloadTabsFromDisk` skips reseeding a still-dirty note) stopped the data-loss but INTRODUCED A FREEZE REGRESSION: a note left dirty + disk-mismatched after the cascade hangs the Svelte reactive layer (the editor never remounts to converge). A follow-up flush-outcome-gate rework (skip the cascade when a flush isn't durable) tested as still-not-acceptable to the Boss. **Boss ruling 2026-07-13: "FIX IT, don't patch it, or revert PJ-092." → REVERTED entirely** to the pre-PJ-092 stable state (commit `fd6008bc`'s `reloadTabsFromDisk`/`flushAllTabsInLibrary`); the original bug is re-opened for a PROPER treatment. **Lesson: PJ-092 touches the rename cascade + reactive lifecycle across Rust↔Svelte — it should have gone through the four-phase `/migration`, not a focused fix. The band-aid → freeze → re-patch cycle is exactly what the Migration Rule prevents.** SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-089** (Index-panel preview two-writable-model silent clobber, HIGH) — the next Group-1 safety item. **PJ-092 is re-opened but must NOT be re-attempted as a focused fix** — it needs the full `/migration` (Architect → Boss picks the approach → Plan → Build → Audit), Reproduce-First on the RUNNING app (the freeze was invisible to the store-level test — the exact "vitest is not runtime verification for editor-lifecycle bugs" gap), before any code.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-089** — Index-panel preview two-writable-model silent clobber (HIGH). *(► Next action)*
> 2. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH).
> 3. **PJ-092** *(RE-OPENED · was APP-KILLER)* — the rename wikilink cascade can silently lose an OPEN, dirty backlink-source note's unsaved edits IF its `.md` is locked at the instant of the rename (Syncthing/OneDrive/Defender): `flushAllTabsInLibrary` swallows the flush `SaveOutcome`, the walker rewrites the stale disk, `reloadTabsFromDisk` force-reseeds the model clean. **RARE** (needs a transient lock exactly during a rename). **Must be redone via `/migration`** — the flush-outcome-gate (mirror renameItem's `renameFlushOk`: don't rewrite+reload a note whose flush wasn't durable) is the likely-correct direction, but the reactive-layer freeze it can trip demands a running-app Reproduce-First + a whole-architecture design pass, NOT another focused patch.
> 4. **PJ-093** — reindex silently skipped when `state.db` is None + reindex-error swallow (`search.rs:9179`/`:9285`, `NoteEditor.svelte:264`) + `flushAllTabsInLibrary` no-onSaved-reindex.
> 5. **PJ-086** — switchTab flush gap (HIGH). 6. **PJ-085 + PJ-073** — frontmatter/YAML round-trip (HIGH) + PropertyEditor forced-type-on-every-key. 7. **PJ-074** — durable rename + folder-rename descendant cascade (+ watcher-suppress-defeats-heal; + link archive/unarchive TARGET aggregate). 8. **PJ-083** — cascade sync-clear hazard. 9. **PJ-087 + PJ-075/076** — persisted-JSON non-atomic / fire-and-forget cluster. 10. **PJ-077** — sync-walk commands → async. 11. **PJ-094/095/096** (below). 12. **PJ-072/002**.
>
> **Group 4/5 — smaller items (from the PJ-092 sweeps, still valid — independent of the revert):** **PJ-094** *(MED)* — `moveItem` (`store.ts:3307`) no flush-before-repath. **PJ-095** *(MED)* — `NoteEditor` `saving` single-flight drops a debounced save + stale net (`NoteEditor.svelte:241`). **PJ-096** *(LOW)* — dirty external-edit `.conflict` sidecar write-failure swallowed (`store.ts:364`). Plus FocusPane title discarded, `addLinkToNote` failed-write reload (verify) → LOW batch.
>
> **Group 2 — Architecture & performance debt** — PJ-084/080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
>
> ### REVERTED since v1.23
> - **PJ-092 — the rename-cascade dirty-guard (v1.23) — REVERTED 2026-07-13.** Shipped as commit `0a605f02` (the `reloadTabsFromDisk` dirty-guard). Boss live-test found a **deterministic FREEZE** in the exact scenario the fix protects (a note whose flush fails during a rename, left dirty + disk-mismatched, hangs the reactive layer — the editor doesn't remount to converge, unlike the clobber path). A follow-up flush-outcome-gate rework (uncommitted) still didn't satisfy the Boss. **Boss: "FIX IT, don't patch it, or revert PJ-092."** → the code was restored to `fd6008bc` (pre-PJ-092): `reloadTabsFromDisk` + `flushAllTabsInLibrary` back to their original form; the sibling-caller gates, the LOCKTEST/SHOWBUG live-test hooks, and the PJ-092 tests all removed. svelte-check 0, vitest 335 (pre-PJ-092 count). Commit `<this>`. The v1.23 preamble + Orientation v3.44 that recorded PJ-092 as "Done" are corrected here (they remain as the durable historical trail). **PJ-092 the bug is re-opened (Group 1, above) — to be redone via `/migration`, not a focused fix.**
>
> ---
>
> *(Prior preambles v1.0–v1.23 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.23.md`.)*

**Version 1.23 | 2026-07-12**

> **What changed in v1.23** (**PJ-092 — the rename-cascade silent edit-loss APP-KILLER — FIXED.** `reloadTabsFromDisk` force-reseeded an open tab's model from stale disk even when that tab's pre-cascade flush had FAILED (a locked `.md`), silently discarding the user's unsaved edits + wiping the recovery net while the save-health banner self-healed to green. Fixed with a **dirty-guard at the shared reload primitive**: it never reseeds or clears the net for a still-dirty tab — a Solve-the-Class fix covering ALL flush-then-reload callers (cascade, updateNoteProperty, resolveStructuralConflict, toggleTaskReconciled, addLinkToNote). Reproduce-First: `reloadDirtyGuard.test.ts` (RED→GREEN); vitest 337; svelte-check 0; `/simplify` applied. The whole-app sweep at this build confirms the APP-KILLER is GONE and found NO new in-diff issue. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-089** (Index-panel preview two-writable-model silent clobber, HIGH): the Index-panel preview mounts a WRITABLE `index_preview_*` NoteEditor NOT deduped against the open store tabs → two independent writable models for one path → last-writer-wins silent clobber, no `.conflict` sidecar. `+layout.svelte:6442`. The next Group-1 safety item now that both APP-KILLERs (PJ-091 accept-truncate, PJ-092 rename-cascade) are closed.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-089** — Index-panel preview two-writable-model silent clobber (HIGH). *(► Next action)*
> 2. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH; re-confirmed by this sweep at `SecondScreenPage.svelte:1681`).
> 3. **PJ-093** — reindex silently skipped when `state.db` is None + reindex-error swallow (`search.rs:9179`/`:9285`, `NoteEditor.svelte:264`); + `flushAllTabsInLibrary` no-onSaved-reindex (`store.ts:1055`).
> 4. **PJ-086** — switchTab flush gap (HIGH). 5. **PJ-085 + PJ-073** — frontmatter/YAML round-trip (HIGH; this sweep re-confirmed `yamlDoc.ts:150` nested-object-list flatten) + PropertyEditor forced-type-on-every-key (`PropertyEditor.svelte:364`). 6. **PJ-074** — durable rename + folder-rename descendant cascade (+ watcher-suppress-defeats-heal nuance; + link archive/unarchive TARGET aggregate not recomputed, `search.rs:8243`/`:8275`). 7. **PJ-083** — cascade sync-clear hazard. 8. **PJ-087 + PJ-075/076** — persisted-JSON non-atomic / fire-and-forget cluster (`review.rs:762`, `saveCollections`, `persistWorkspaces`). 9. **PJ-077** — sync-walk commands → async. 10. **PJ-094/095/096** *(NEW, below)*. 11. **PJ-072/002**.
>
> **Group 4/5 — smaller items (this sweep):** **PJ-094** *(NEW · MED)* — `moveItem` (`store.ts:3307`) repaths the model without the flush-before-op guard its siblings have (verify actual loss vs. repath-preserves). **PJ-095** *(NEW · MED)* — a debounced save dropped by `NoteEditor`'s `saving` single-flight guard (`NoteEditor.svelte:241`) is never rescheduled + leaves a stale net → latest edit lost on a same-window crash. **PJ-096** *(NEW · LOW)* — a dirty-note external-edit whose `.conflict` sidecar write FAILS is swallowed (`store.ts:364`) → a silent hole in PJ-070's zero-loss guarantee. FocusPane title discarded (`+layout.svelte:7879`) + `resolveStructuralConflict` no-reindex → LOW batch.
>
> **Group 2 — Architecture & performance debt** — PJ-084/080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
>
> ### CLOSED since v1.22
> - **PJ-092 — rename-cascade silent edit-loss APP-KILLER — DONE 2026-07-12.** `reloadTabsFromDisk` (`store.ts:686`) force-reseeded a tab's model from stale disk (`openNoteModel`) + wiped its write-ahead net (`clearWriteAhead`) unconditionally. When a backlink-source tab's pre-cascade flush FAILED (a locked `.md` — Syncthing/OneDrive/Defender), its model stayed dirty but the cascade walker rewrote its stale disk link, so disk differed from `tab.content` and the reseed fired — rebuilding the model CLEAN from stale disk, discarding the unsaved edits, wiping the sole recovery copy, and self-healing the save-health banner to green. The sibling `renameItem` path was hardened (`renameFlushOk`); the shared reload primitive was not. **Fix:** a dirty-guard in `reloadTabsFromDisk` — never reseed or clear the net for a still-dirty tab (dirty ⟺ the pre-reload flush didn't land); the dirty model + net stay the sole copy, the ~10 s save-health auto-retry persists it once the lock clears. Solve-the-Class: guarding the ONE shared primitive covers every flush-then-reload caller (cascade / updateNoteProperty / resolveStructuralConflict / toggleTaskReconciled / addLinkToNote), and catches a re-type-during-cascade a mere flush-outcome bool would miss; `resolveConflictMerge` gates on `outcome.ok` first, so it's unaffected. **Reproduce-First:** `tests/mig-076/reloadDirtyGuard.test.ts` drives the real primitive with a dirty model + stale disk — RED (edit clobbered) → GREEN (edit preserved, net kept), plus a clean-tab reseed contrast. vitest 337; svelte-check 0; `/simplify` (dirty-guard at the primitive is the right depth — adversarially confirmed no signal hole, resolveConflictMerge verified clean). Whole-app sweep: 0 in-diff findings, APP-KILLER absent. Commit `<this>`. Orientation v3.44.
>
> ### NEWLY FILED — PJ-094, PJ-095, PJ-096 (from the PJ-092 whole-app sweep `wf_b57f6cd6-be3`, 34 agents, 15 confirmed)
> - **PJ-094** *(MED)* — `moveItem` no flush-before-repath. **Open · Charter · Group 1/4.**
> - **PJ-095** *(MED)* — `NoteEditor` `saving` single-flight drops a debounced save + stale net. **Open · Charter · Group 1/4.**
> - **PJ-096** *(LOW)* — dirty external-edit `.conflict` sidecar write-failure swallowed (PJ-070 zero-loss hole). **Open · Charter · Group 5.**
> - *(The other 12 confirmed map to PJ-074/075/087/073/085/090/093 + LOWs; register appended to the Charter. PJ-092 diff: ZERO findings.)*
>
> ---
>
> *(Prior preambles v1.0–v1.22 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.22.md`.)*

**Version 1.22 | 2026-07-12**

> **What changed in v1.22** (**PJ-091 — accepting a classifier suggestion silently truncated a note's MANUAL multi-value frontmatter — FIXED (Boss ruling: merge, never lose a manual value).** Root: "accept" was implemented as `set_manual(suggestion)` — a machine proposal treated as the user's exact manual assertion, so it OVERWROTE. Fixed at every accept seam: the suggestion is now UNIONED with the note's current on-disk values **under the write lock** (race-free), the merged set mirrored to the search index; the exact-set primitive is preserved (default-off `merge` flag) for any future direct-set. Reproduce-First: `pj091_repro_…` (RED→GREEN) + 3 fix tests; full regression green (sources 32 / classifier 15 / write_gate 22); svelte-check 0; `/simplify` applied. **The whole-app safety-inspection at this build found a NEW APP-KILLER → PJ-092** (rename-cascade silent edit-loss). SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-092** *(NEW · APP-KILLER)*: `flushAllTabsInLibrary` discards each pre-cascade flush's `SaveOutcome`, so a FAILED durable write of an open, dirty backlink-source tab is treated as success — the rename cascade then force-reseeds the model CLEAN from stale disk and deletes the recovery net, **silently and permanently losing the user's unsaved edits while the save-health banner self-heals to green.** The sibling `renameItem` path is already hardened against this exact class (`renameFlushOk`); the whole-library cascade path was not. Highest severity on the board — an app-killer jumps to Group-1 top (SO#9). *(Boss to confirm sequencing; or continue the standing Group-1 order below.)*
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-092** *(NEW · APP-KILLER)* — `flushAllTabsInLibrary` (`store.ts:1030`) rename-cascade silent edit-loss (discards the flush `SaveOutcome`; cascade proceeds on a failed flush → `reloadTabsFromDisk` force-reseeds stale + `clearWriteAhead` wipes the net; banner self-heals). **Fix = mirror the `renameItem` `renameFlushOk` gate (store.ts:3194-3240).** Focused, proven-sibling-pattern fix.
> 2. **PJ-089** — Index-panel preview two-writable-model silent clobber (HIGH).
> 3. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH).
> 4. **PJ-093** *(NEW · MED→HIGH)* — reindex silently skipped when `state.db` is None: `constellation_search_reindex`/`reindex_single_note` return `Ok(())` with the index un-updated (`search.rs:9285`/`:9172`, no `ensure_search_db_ready`), and every save/flush/stage-promote fires the reindex as `.catch(()=>{})` (`NoteEditor.svelte:264`) — a note durably written to disk is silently never indexed (search divergence, no error, no retry).
> 5. **PJ-086** — switchTab flush gap (HIGH). 6. **PJ-085 + PJ-073** — frontmatter/YAML round-trip (HIGH); **now also** PropertyEditor injecting a forced/registered property TYPE onto every projected key → breaks the G4 `composeFrontmatter` unification invariant (`PropertyEditor.svelte:364`). 7. **PJ-074** — durable rename + folder-rename descendant cascade *(+ the new nuance: `gate_rename` watcher-suppresses both folder paths, defeating the freshness heal; + the cascade ignores `CascadeResult.failed[]` at `+layout.svelte:6304`)*. 8. **PJ-083** — cascade sync-clear hazard. 9. **PJ-087 + PJ-075/076** — persisted-JSON non-atomic / fire-and-forget cluster (this sweep re-confirmed `review.rs:762`, `saveCollections`/`saveSettings`/`persistWorkspaces`). 10. **PJ-077** — sync-walk commands → async (*this sweep added a third: `collect_library_notes`, `libraries.rs:5280*`). 11. **PJ-072/002**.
>
> **LOW batch (Group 4/5):** `rename_item_db_tail` `.find()` picks the parent library not the most-specific (`libraries.rs:1115`); FocusPane title edit silently discarded — host never wires `ontitlechange` (`+layout.svelte:7879`); SS companion note-view model never `close()`d → unbounded per-window Map growth (`SecondScreenPage.svelte:995`). *(All filed to the Charter register; low blast radius.)*
>
> **Group 2 — Architecture & performance debt** — PJ-084/080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
> **Group 4 — Polish / i18n / small bugs** — unchanged (+ the 3 LOWs above). **Group 5 — Documentation & hygiene** — unchanged.
>
> ### CLOSED since v1.21
> - **PJ-091 — accept silently truncates manual multi-value frontmatter — DONE 2026-07-12** (Boss ruling: *merge, never lose a manual value*). **Root:** "accept a classifier suggestion" reused the exact-set primitive `set_manual(suggestion)`, treating a machine proposal as the user's exact manual assertion → it REPLACED. When a suggestion is stale relative to what the user has since typed on disk, accepting it drops the manual values. **Fix — union at every accept seam, under the write lock:** new `union_preserve_order(existing, additions)` (existing-first, append new); a default-off `merge` flag threaded through `rewrite_note_sources_on_disk`/`rewrite_note_content_type_on_disk`/`sources_set_manual`/`content_type_set_manual` (returns the effective merged set → mirrored to the DB so note_meta never diverges from disk); bulk `accept_one` unions both axes inline in its one dual-axis `gate_rmw`; per-card Accept (plain + edit-override) and disambiguation pass `merge:true`; the exact-set primitive is untouched for clear + any future PropertyEditor direct-set. **Part A:** `build_axis_suggestions` (refactor that DRYs the horizontal/vertical loop) now carries the classifier's `.secondary` sources the old builder dropped. **Verify:** `pj091_repro_accept_replace_truncates_manual_multivalue` (RED→GREEN) + `pj091_accept_merge_preserves_manual_multivalue` + `pj091_union_preserve_order_dedupes_and_appends` + `pj091_build_axis_suggestions_carries_secondary`; sources 32 / classifier 15 / write_gate 22 all pass; svelte-check 0; `/simplify` applied (removed RefCell machinery + 2 redundant clones); whole-app sweep — **0 in-diff findings.** Commit `<this>`. Orientation v3.43.
>
> ### NEWLY FILED — PJ-092, PJ-093 (from the PJ-091 whole-app safety-inspection `wf_f2a07366-fc5`, 37 agents, **17 confirmed**)
> - **PJ-092** *(APP-KILLER)* — `flushAllTabsInLibrary` rename-cascade silent edit-loss. **Open · Charter · Group-1 TOP · ► Next action.** Fix = mirror `renameFlushOk`.
> - **PJ-093** *(MED→HIGH)* — reindex silently skipped when `state.db` is None + reindex-error swallow. **Open · Charter · Group 1.**
> - *(The other 15 confirmed are pre-existing backlog: folder-rename+watcher-suppress=PJ-074, save_pulse=PJ-075, PropertyEditor-type=PJ-073/085, persisted-JSON cluster=PJ-075/087, sync-walk=PJ-077, + 3 LOWs to the Charter. The PJ-091 diff's OWN change: ZERO findings; `cece-sources-derived` scope 0 confirmed.)*
>
> ---
>
> *(Prior preambles v1.0–v1.21 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.21.md`.)*

**Version 1.21 | 2026-07-12**

> **What changed in v1.21** (**PJ-071 — the bulk Accept-All unlocked read-modify-write race — FIXED.** `accept_one` (`sources/bulk_ops.rs`) read the note UNLOCKED then `gate_write` — a concurrent editor save in the window was silently overwritten. Routed through the proven **`gate_rmw`** (read+mutate+write under one per-path lock, off-thread), mirroring the already-migrated per-card path. Behaviour-preserving (31 sources tests + 22 write_gate tests pass); the primitive's serialization is proven by `concurrent_writers_serialize_never_tear`. The per-build sweep surfaced ONE new HIGH in the same function (accept truncates manual multi-value frontmatter) → filed PJ-091. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-091** (accept silently truncates a note's MANUAL multi-value frontmatter): the highest-value new safety item — a real silent content-loss on a routine "Approve All", distinct from PJ-071's race. *(Or continue the standing Group-1 order below; Boss's call.)*
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-091** *(NEW · HIGH)* — accepting a classifier suggestion REPLACES a note's `sources:`/`content_type:` with the suggestion's ids, silently truncating the user's MANUAL multi-value frontmatter (e.g. `sources: [testimony, perception]` → `[testimony]`). Root: the suggestion builder (`classifier/mod.rs:128-148`) reads only `primary + see_also`, drops `.secondary`; `accept_one` then REPLACES (not merges) the axis. Needs a look at the classifier synthesis + a ruling on accept semantics (merge-vs-replace / preserve-manual). *(Surfaced by the PJ-071 sweep; the accept path — same function as PJ-071 but a distinct bug.)*
> 2. **PJ-089** — Index-panel preview two-writable-model silent clobber (HIGH). *(re-confirmed by this sweep at `+layout.svelte:7300`.)*
> 3. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH).
> 4. **PJ-086** — switchTab flush gap (HIGH). 5. **PJ-085 + PJ-073** — frontmatter/YAML (HIGH). 6. **PJ-074** — durable rename + cascades. 7. **PJ-083** — cascade sync-clear hazard. 8. **PJ-087 + PJ-075/076** — atomic-write / lost-update residuals (this sweep re-confirmed `link_types.rs:535`, `review.rs:762`, `style_presets.rs:51`, `saveCollections` — the persisted-JSON non-atomic cluster). 9. **PJ-072/002**.
>
> **Group 2 — Architecture & performance debt** — PJ-084/080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
> **Group 4 — Polish / i18n / small bugs** — unchanged. **Group 5 — Documentation & hygiene** — unchanged.
>
> ### CLOSED since v1.20
> - **PJ-071 — bulk Accept-All RMW race — DONE 2026-07-12.** `accept_one` (`sources/bulk_ops.rs:269`) read the note unlocked (line 305) then `gate_write` (line 310) — the read→modify→write was not atomic, so a concurrent editor save in the window was silently overwritten by the stale-based rewrite. Fixed: one `gate_rmw(path, "bulk_accept", |content| …rewrite both axes…)` — read+mutate+write under the same per-path lock the editor's `write_note` uses, so a save lands before or after but never inside. Runs on the existing `thread::spawn` worker (no dispatch-thread freeze; gate_rmw rule #2 satisfied — the closure is pure string work, DB mirror update is after). A proven-pattern migration (mirrors the per-card `rewrite_note_sources_on_disk` + 6 other `gate_rmw` sites); behaviour-preserving (+ an idempotent-skip: identical rewrite → `Ok(None)`, no write). cargo check + 31 sources tests + 22 write_gate tests pass. Commit `<this>`. Orientation v3.42.
>
> ### NEWLY FILED — PJ-091 (from the PJ-071 per-build whole-app sweep `wf_4dd12a39-694`, 46 agents, 24 confirmed)
> - **PJ-091** — accept truncates manual multi-value frontmatter (HIGH). **Open · Charter · Group 1 · needs a classifier-synthesis look + a Boss ruling on accept semantics.**
> - *(The other 22 confirmed are pre-existing backlog — Index-preview=PJ-089, the persisted-JSON non-atomic cluster=PJ-087/075/076, folder-rename=PJ-074, yamlDoc nested=PJ-085, incoming-aggregate recompute=PJ-074, etc. Register appended to the Charter. The PJ-071 diff's OWN gate_rmw change: ZERO findings.)*
>
> ---
>
> *(Prior preambles v1.0–v1.20 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.20.md`.)*

**Version 1.20 | 2026-07-12**

> **What changed in v1.20** (**PJ-088 — the conflict-resolution SIDE-BY-SIDE MERGE view — SHIPPED + Boss-validated end-to-end.** The follow-up conflict layer the PJ-070 Architect deferred: when a note has a `.conflict` side-copy, the banner's **Merge…** button opens a two-column view (Your version | Outside copy) with a **◀ Copy to mine** button per difference + free edit; **Save merged** writes safely through the model + durability gate, **Cancel** is a pure no-op — zero loss until an explicit save. Art-Director-designed + adversarially-judged; the per-build safety-inspection's one in-diff finding fixed pre-commit. SO#9 reconciliation. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-071** (bulk Accept-All unlocked read-modify-write race) — unchanged; the ruled Group-1 safety item. (PJ-088 was a Boss-directed feature interleave, not a Group-1 item; Group-1 order below is otherwise intact, now with two new HIGH silent-loss items from the PJ-088 sweep.)
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-071** — bulk Accept-All unlocked RMW race. *(► Next action)*
> 2. **PJ-089** *(NEW · HIGH)* — the Index-panel preview mounts a WRITABLE NoteEditor on a standalone `index_preview_*` tab NOT deduped against the open store tabs → two independent writable models for one path → last-writer-wins silent clobber, no `.conflict` sidecar (the dedup + adopt paths both bypass it). `+layout.svelte:6442`.
> 3. **PJ-090** *(NEW · HIGH)* — the second screen's Tasks-panel `onToggle` writes the shared `.md` (Display-not-Domain breach) with NO `broadcastNoteSaved` and the write is watcher-suppressed → the main window never learns of it and clobbers the toggle on its next save. `SecondScreenPage.svelte:1681`/`:1537`.
> 4. **PJ-086** — switchTab flush gap (HIGH). 5. **PJ-085 + PJ-073** — frontmatter/YAML (HIGH). 6. **PJ-074** — durable rename + cascades. 7. **PJ-083** — cascade sync-clear hazard. 8. **PJ-087** — universe.rs tmp race. 9. **PJ-075/076/072/002**.
>
> **Group 2 — Architecture & performance debt** — PJ-084 (SS/main share the adopt primitive) · PJ-080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
> **Group 4 — Polish / i18n / small bugs** — BUG-013 · PJ-082/014/049 · PJ-044.. · PJ-008.. · PJ-004 · PJ-017/018/019. *(unchanged.)*
> **Group 5 — Documentation & hygiene** — PJ-081 (§12 drift + orientation BODY refresh) · PJ-051/057(b) · PJ-011. *(unchanged.)*
>
> ### CLOSED since v1.19
> - **PJ-088 — Conflict-resolution side-by-side MERGE view — DONE / Boss-validated 2026-07-12.** Boss-requested follow-up to PJ-070 (shape = side-by-side, full live preview). Art Director design workflow `wf_d7453254-50e` (census + WA#5 prior art + 3 competing designs + 3 adversarial judges + synthesis) → `docs/PJ-088-Conflict-Merge-Design.md`. Banner **Merge…** → a full-center overlay: the official **`@codemirror/merge`** MergeView (2-way — no common ancestor is stored; lazy-imported into a 29KB chunk, Rule 6), live-preview panes, a **◀ Copy to mine** button per chunk (custom `renderRevertControl` after the default chevron tested too subtle). **The safety wire** `resolveConflictMerge` (store.ts): push the merge into the model via the NEW `replaceContent` (re-bases so compose emits the merged frontmatter verbatim — fixes the sweep's one in-diff finding: `editNoteProps` left `m.base` stale → non-projectable frontmatter silently dropped) → durability gate → remount + `markReseeding`/`await tick` (hazard #6) + `focusReseed` → sidecar `moveToTrash` (reversible) + `dismissConflict`, ONLY after a durable save; **Cancel = pure no-op**. `conflict.*` (13 keys) in all 15 locales. Reproduce-First: runtimeHarness Recipe P (re-base). svelte-check 0, vitest 335, cargo clean. Boss-validated: the merge round-trips (note reconciled + `cid` intact, sidecar in `.trash`, banner cleared). Commits `bc6a1e43` + `cd…59295333`.
>
> ### NEWLY FILED — PJ-089, PJ-090 (from the PJ-088 per-build whole-app safety-inspection `wf_c0dac305-85e`, 40 agents, 19 confirmed)
> - **PJ-089** — Index-panel preview two-writable-models silent clobber (HIGH). **Open · Charter · Group 1.**
> - **PJ-090** — second-screen Tasks-panel toggle no-broadcast + watcher-suppressed clobber (HIGH; incl. the split-view companion `:1537`). **Open · Charter · Group 1.**
> - *(The sweep's other 17 findings are pre-existing backlog — mapped to PJ-073/074/075/076/077/085/086/087 or LOW hygiene items; register appended to the Charter. The PJ-088 diff's OWN only finding — the stale-base compose — was fixed pre-commit via `replaceContent`.)*
>
> ---
>
> *(Prior preambles v1.0–v1.19 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.19.md`.)*

**Version 1.19 | 2026-07-12**

> **What changed in v1.19** (**PJ-070 — the watcher external-change APP-KILLER — SHIPPED + Boss-validated end-to-end + `/migration` CLOSED.** An external `.md` edit to an OPEN note was silently clobbered by the next keystroke; the fix adopts the change into the single-ownership model (clean → adopt + remount; dirty → `.conflict` sidecar, zero loss). The per-cycle whole-app safety-inspection ran at this migration boundary — 1 in-diff finding fixed pre-commit, the rest filed as PJs / to the Charter. PJ-072's registry mystery got a concrete lead. SO#9 reconciliation. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-071** (bulk Accept-All unlocked read-modify-write race): the proven `gate_rmw` pattern already exists next door — per-card accept was migrated, bulk wasn't — so a concurrent editor save in the window is silently overwritten. The next Group-1 safety item now that PJ-070 is done.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. ~~**PJ-070**~~ — **DONE 2026-07-12** (`/migration`, commits `b1a3e388`+`cd5e53fd`, Boss-validated Stages 1+2). See CLOSED below.
> 2. **PJ-071** — bulk Accept-All unlocked RMW race. *(now the ► Next action)*
> 3. **PJ-086** *(NEW · HIGH)* — `switchTab` (`store.ts:2412`) is a note-editing departure that never flushes the outgoing dirty model (unlike the 3 guarded departures) → the last ≤1.5 s of edits on a non-active tab are lost on quit/crash with no banner/net (APP-KILLER #2 class, on the one unwired departure path).
> 4. **PJ-085 (with G4/PJ-073)** *(NEW · HIGH)* — `composeFrontmatter` H1 error-passthrough (`yamlDoc.ts:213`) silently discards **ALL** property edits whenever frontmatter is lenient-parseable by the app but strict-invalid to eemeli (a colon-in-a-title value, a duplicate key, an `@handle`, a stray tab) — the save reports success, the edit is gone. Folds into the frontmatter real-YAML round-trip work.
> 5. **PJ-073 (G4)** — frontmatter real-YAML round-trip (block-scalars/nested-maps/quote corruption) — now paired with PJ-085.
> 6. **PJ-074 (G5)** — durable rename Step-2 + folder rename/delete descendant cascade + link archive/unarchive incoming recompute + `move_item` review reset. *(the sweep re-confirmed the folder-rename/delete + archive/unarchive halves.)*
> 7. **PJ-083** *(NEW)* — the rename-cascade + `drainCidEnsure` **synchronous** `clearCascading` latent hazard #6 (a mis-timed `{#key}` teardown could re-stale). PJ-070 fixed its OWN path (deferred `await tick()` clear + a dedicated refcounted reseed gate); the cascade/cid paths still clear synchronously. Reproduce-First.
> 8. **PJ-087** *(NEW · G6)* — `universe.rs::atomic_write` uses a FIXED shared tmp filename → concurrent async saves of the same persisted-JSON collide/interleave.
> 9. **PJ-075 (G6)** + **PJ-076 (G7)** + **PJ-072** (registry investigation — LEAD below) + **PJ-002** (opt-in dup-`cid_cn` scan).
>
> **Group 2 — Architecture & performance debt**
> 1. **PJ-084** *(NEW)* — share the per-tab freshness-adopt primitive between the main-window `adoptExternalChangeIntoTabs` and `SecondScreenPage.adoptFreshDiskIntoSS` (one home per capability; they are documented twins today, and PJ-070's hazard-#6/reseed-gate fix does NOT apply to the SS copy — which is safe only because the SS is read-only). Verified NON-urgent: SS read-only = no teardown write.
> 2. **PJ-080** (`init_db` cold-init profiling first) · **PJ-078** (`map.rs` last Rule-8 read-walk) · **PJ-079** (`get_360_view` index-read rewrite) · **PJ-077 (G8)** (2 sync-walk commands → async) · **PJ-069 remainder** (whole-entity dedup). *(unchanged from v1.18.)*
>
> **Group 3 — Feature completion** — PJ-067 (Living-Link v2) · PJ-068 (Cockpit 9 facet tabs + contextual-SS) · MIG-096 §3–§6 (right-click rollout) · MIG-088 Ph6–10 (Style Setter) · Backup & Recovery. *(unchanged.)*
>
> **Group 4 — Polish / i18n / small bugs** — BUG-013 + sidebar highlight lag · PJ-082 (search-grammar 7 built-in link types) / PJ-014 (CECE 13-locale) / PJ-049 (Help viewer) · PJ-044/046/047/048/050 · PJ-008/009/010 · PJ-004 · PJ-017/018/019 · Arabic callout caret. *(unchanged.)*
>
> **Group 5 — Documentation & hygiene** — **PJ-081** (§12 doc-drift batch + orientation BODY refresh; NOW also add the *"Eisa Cognitive Knowledge" universe root = `E:\Cognitive Knowledge\`* fact) · PJ-051/057(b) Sight SVGs · PJ-011 (Map dormant).
>
> ### CLOSED since v1.18
> - **PJ-070 — Watcher external-change adopt — DONE / CLOSED 2026-07-12.** The silent-clobber APP-KILLER (the main-window mirror of the closed G3 class): an external `.md` edit to an OPEN note updated only `tab.content`, never the single-ownership note model / `reloadVersion`, so the mounted editor kept the stale body and the next keystroke's debounced save durably overwrote the external edit. **Fix (Boss-ratified Option B + `.conflict` sidecar):** one shared `adoptExternalChangeIntoTabs` helper both ingress paths (watcher flush + second-screen `onNoteSaved`) call — clean model → `adoptDisk` + `reloadVersion` remount (dedicated **refcounted** reseed gate spanning the async `{#key}` teardown, hazard #6); dirty + genuine change → `write_conflict_sidecar` (`<stem>.conflict-<UTCz>.md.txt`, inert to every `.md`-gated surface) + a banner (zero loss). Plus the **`setBody` string-no-op CLASS FIX** (a merely-viewed note never spuriously dirties — which otherwise defeated `adoptDisk` on background/focus tabs + raised phantom sidecars), the **Focus-mode suppressed reseed** (hazard #7), and the **`onNoteSaved` sibling-gap fold** (it adopted the model but forgot the remount). `/migration` Phases 1–4 (Architect `docs/PJ-070-…-Architect.md`, Plan `…-Plan.md`; 3-agent audit — all 11 invariants + the class-fix invariant HOLD). Reproduce-First: `tests/mig-076` Recipe O + `watcherAdoptStore.test.ts`. Behind `WATCHER_ADOPT_ENABLED` (rollback lever). **Boss-validated:** Stage 1 (adopt live, no clobber — disk-verified) + Stage 2 (dirty-conflict → sidecar + banner, both versions preserved). Commits `b1a3e388` + `cd5e53fd`. Orientation v3.40.
>
> ### NEWLY FILED — PJ-083 → PJ-087 (from the PJ-070 close + the per-cycle whole-app safety-inspection `wf_1b7addb3-822`, 38 agents, 15 confirmed)
> - **PJ-083** — rename-cascade + `drainCidEnsure` synchronous `clearCascading` latent hazard #6. **Open · Charter · Group 1 · Reproduce-First.**
> - **PJ-084** — share the per-tab freshness-adopt primitive (main helper ↔ `adoptFreshDiskIntoSS`). **Open · Group 2.**
> - **PJ-085** — `composeFrontmatter` H1 passthrough silently drops ALL property edits (HIGH). **Open · Charter · G4/Group 1.**
> - **PJ-086** — `switchTab` doesn't flush the outgoing dirty model → edit-loss on quit (HIGH). **Open · Charter · Group 1.**
> - **PJ-087** — `universe.rs::atomic_write` fixed shared tmp filename race. **Open · Charter · G6.**
> - *(The other 10 confirmed sweep findings map to existing PJs — `review.rs` save_pulse = PJ-075; folder rename/delete descendant cascade + link archive/unarchive incoming recompute = PJ-074 — or are LOW batch items: `parseFrontmatter` comma-split, `sources` prefix-strip, `saveCollections` un-awaited, `BacklinksPanel.linkMention` swallow, `perf_trace` unbounded Vec. Full register appended to the Charter.)*
>
> ### PJ-072 — INVESTIGATION UPDATE (concrete lead, 2026-07-12 — during the PJ-070 Boss test)
> The running instance's **"Eisa Cognitive Knowledge"** universe resolves to the on-disk root **`E:\Cognitive Knowledge\`** — the write-journal recorded the Boss's test note's `create_note` + `editor_save` + `conflict_sidecar` all under `E:\Cognitive Knowledge\Eisa Test\`. This is a DIFFERENT path than the `E:\Constellation Universes\Eisa Cognitive Knowledge\` folder that ALSO exists on disk (and is what the earlier registry search scanned). So WHERE the active universe's data lives is now known; the diagnostic build is still wanted to explain WHERE the display-name→`E:\Cognitive Knowledge` mapping is persisted (it is not in the findable `universes.json`). **Status: Open · lead captured (Charter updated).**
>
> ---
>
> *(Prior preambles v1.0–v1.18 + the full CLOSED / NEWLY-FILED history follow below, unchanged; the trail is also durable in `docs/Constellation Pending Jobs v1.18.md`.)*

**Version 1.18 | 2026-07-12**

> **What changed in v1.18** (Boss-directed 2026-07-12: fold the whole outstanding-work sweep into the ledger, then RE-PRIORITIZE every open job into five decision groups. Method — a full orientation-audit v3.00→v3.38 (39-version reconciled outstanding sweep), a per-PJ status re-verification of PJ-001→069 against the current code + orientation §8/§9 + session logs, and a merge with the Safety Charter's G2–G8 register. Every status flip cites a MIG/commit/§/date; three highest-value closures spot-verified in code.):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-070** (the watcher external-change app-killer): a note edited *outside* Constellation while open (git-pull / Syncthing / Obsidian) is silently clobbered by the next keystroke. It's the most-common external path and the main-window mirror of the just-closed G3 cross-window class — the fix reuses the existing `adoptFreshDiskIntoSS` pattern. Cheapest high-value safety win.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-070** — watcher external-change never adopts into the note model → next-keystroke clobber. *(silent, most-common external path)*
> 2. **PJ-071** — bulk Accept-All unlocked read-modify-write race. *(proven `gate_rmw` pattern already exists next door — per-card was migrated, bulk wasn't)*
> 3. **PJ-073 (G4)** — frontmatter real-YAML round-trip: block-scalars / nested maps dropped + quote corruption on **every** save. *(highest blast radius — corrupts rich-frontmatter notes silently)*
> 4. **PJ-074 (G5)** — durable rename Step-2 (MIG-098 remediation #1's durable half still owed) + folder rename/delete descendant cascade + link archive/unarchive incoming recompute + `move_item` review-history reset.
> 5. **PJ-072** — INVESTIGATION: universes booting from an unfound registry ("Eisa Cognitive Knowledge" 7,751 notes / "Scratch" absent from every on-disk registry). *(cheap diagnostic build first; unknown registry persistence = latent registration-loss)*
> 6. **PJ-075 (G6)** + **PJ-076 (G7)** — residual atomic-write / lost-update-guard gaps (`review-pulse.json`, swallowed review-priority writes; the write-gate staleness check that never fires). *(small, direct)*
> 7. **PJ-002** — opt-in duplicate-`cid_cn` re-canonicalize scan. *(low residual — MIG-003 §85 + the v3.25 partial index already auto-mitigate; only matters for restored pre-§140 backups)*
>
> **Group 2 — Architecture & performance debt**
> 1. **PJ-080** — profile `init_db` cold-init first (20–40 s, UNMEASURED — "do not guess" before any fix).
> 2. **PJ-078** — `map.rs::constellation_map_universe`, the **last remaining Rule-8 read-time filesystem walk** (OrgChart/Map re-walks disk on every open) → write-time/index-read.
> 3. **PJ-079** — `get_360_view` still re-walks disk (async wrapper shipped v3.22; index-read rewrite owed, own /migration).
> 4. **PJ-077 (G8)** — two sync-walk commands → `(async)` (`provenance.rs:100`, `libraries.rs:5148`). *(one-word freeze fixes)*
> 5. **PJ-069 remainder** — whole-entity dedup: hubs → `note_meta.incoming_count`, then tags / folders / note-lists (MIG-096 §3–§6 + NoteRow across ~26 surfaces). *(Step-0 cull + orphan/fragile already shipped via MIG-094)*
>
> **Group 3 — Feature completion** *(concept-papers ready or parked)*
> 1. **PJ-067** — Living Link Relationship Model v2 (Tension-first families/dimensions; concept ratified, build owed; /migration).
> 2. **PJ-068** — the Cockpit's 9 stubbed facet tabs (only "Links" wired) + the per-mode contextual-SS rulings. *(the read-only Knowledge Cockpit chassis shipped v3.35/3.38)*
> 3. **MIG-096 §3–§6** — the right-click-menu rollout remainder (Groups B/C/D + trees), parked behind the safety audit.
> 4. **MIG-088** — Style-Setter Phases 6–10 (search/index badges · Sky/OrgChart/Map D3 · calendar · dialogs/global · audit).
> 5. **Backup & Recovery system** — Boss-wanted; concept paper done; /migration + WA#5.
>
> **Group 4 — Polish / i18n / small bugs**
> 1. **BUG-013** open-editor cascade race; **sidebar active-item ~10 s highlight lag** (reproduce-and-instrument when it next fires).
> 2. **PJ-082** — search-query grammar hardcodes 7 built-in link types (`store.ts:2071`) so `supersedes [[X]]` / custom-type filters are dead; **PJ-014** — CECE/User-Manual 13-locale backfill (standing translation debt); **PJ-049** — in-app Help viewer (P2 — files exist in 15 locales, no UI surface).
> 3. **PJ-044/046/047/048/050** (MIG-022 UI polish); **PJ-028/029/033** stage-badge (PJ-030/031/032 deferred P3).
> 4. **PJ-008/009/010** backlink/outgoing render-dedup + alias-bleed (re-verify vs the MIG-079 panel rewrite first); **PJ-004** NSIS `os error 32`; **PJ-017/018/019** dead schema/flag/i18n cleanup; **Arabic callout End/Home caret** (documented — Boss ruled stop-patching).
>
> **Group 5 — Documentation & hygiene**
> 1. **PJ-081** — the §12 doc-drift batch (`IPC-CONTRACT.md` ~80 vs ~120 commands · `CANONICAL-FILENAME` `cid`→`cid_cn` · versioning all 0.3.4 · **the now-STALE "no frontend test harness" row** — a full vitest suite exists, 318/318) + the orientation BODY refresh (§3/§4.x/§13/§17 still ~v1.58; current truth lives only in stacked preambles — the SO#8 body-vs-preamble lag).
> 2. **PJ-051 / PJ-057(b)** Sight SVG follow-ups (dormant); **PJ-011** Constellation Map issues (DORMANT — Map disabled by MIG-038).
>
> ---
>
> ### CLOSED since v1.17 — moved to Done (v1.17 carried these wrongly as OPEN, or they shipped after)
>
> - **PJ-003** — rename-collision popup — DONE, MIG-076 §E-1 (`CollisionDialog.svelte`), orient. v2.76.
> - **PJ-005** — MIG-007 Links Settings tab — DONE (verified: `SettingsModal.svelte:180`/`:1381`), orient. v2.62.
> - **PJ-012** — `LinkLifecycle.fresh` tier — DONE (`store.ts:3301`/`:3472`).
> - **PJ-013** — `lenses.rs::apply_lens` decision — DONE (deleted via PJ-069 §0f, orient. v3.28).
> - **PJ-021** — Sky View Rule-8 — DONE (write-time triggers + boot-read killed, MIG-079 §C.2c).
> - **PJ-022 / PJ-023** — Backlinks / Outgoing Rule-8 — DONE (MIG-079 §C.2a `incoming_count` + §C.2c per-note index seeks).
> - **PJ-024** — Tag browser Rule-8 — DONE (MIG-079 §C.1 `tag_counts` + MIG-080 §B).
> - **PJ-041 / PJ-042 / PJ-043** — CECE i18n — DONE (MIG-022 §E; bodies were stale-Open).
> - **PJ-060** — `index_note` cache short-circuit — DONE (verified: `search.rs:5660` `force` gate cites PJ-060).
> - **PJ-063** — `note_links.link_type` globally 'relates' — DONE (verified: real varied `link_type` column, `search.rs:189+`).
> - **PJ-064** — Style-Setter more font types — DONE (full installed-font catalogue, 2026-06-11).
> - **PJ-065** — structural/parent-TOC link — DONE/CLOSED (2026-06-29, orient. v3.16, re-verified v3.35).
> - **PJ-066** — sky-reindex storm (~2 min → 3 s) + connect-freeze — DONE (orient. v3.15, memory `project_pj066`).
>
> *(Total: 16 PJs flipped to Done. The v1.17 §-bodies retain their historical text per the amendment convention — this preamble is the authoritative status delta.)*
>
> ### NEWLY FILED — PJ-070 → PJ-082 (deduped against PJ-001..069; sourced from the orientation-audit sweep + the Safety Charter)
>
> - **PJ-070** — APP-KILLER: watcher external-change never adopts into the note model (`+layout.svelte:3172`); an external `.md` edit updates only `tab.content`, never the single-ownership model / `reloadVersion`, so the next keystroke's debounced save durably overwrites the external edit (then reindexes, so search agrees with the stomp). The main-window mirror of the closed G3 class — fix reuses `adoptFreshDiskIntoSS`. **Open · APP-KILLER · Charter (2026-07-11 sweep).**
> - **PJ-071** — APP-KILLER: bulk Accept-All unlocked read→modify→write (`sources/bulk_ops.rs:305`) — `accept_one` reads unlocked then `gate_write`, the exact race `gate_rmw` prevents (per-card accept was migrated; bulk wasn't); a concurrent editor save in the window is silently overwritten. **Open · APP-KILLER · Charter.**
> - **PJ-072** — INVESTIGATION: universes booting from an unfound registry. The active instance boots "Eisa Cognitive Knowledge" (7,751 notes) + "Scratch", neither in any on-disk `universes.json` findable on the machine, while the shared write-journal records both. Ship a diagnostic build logging the resolved `app_data_dir` + registry path at boot. **Open · Charter OPEN INVESTIGATION (2026-07-12).**
> - **PJ-073** — G4: frontmatter real-YAML round-trip. Block-scalars + nested maps are dropped and quoted values corrupted on save; incl. the `yamlDoc.ts:150` nested-object-list flatten. The audit's #1 remaining silent-corruption item. **Open · Charter G4 (W2-7/8) · /migration.**
> - **PJ-074** — G5: durable rename + index cascades. The MIG-098 rename-index durable Step-2 (boot self-heal shipped, the durable write path didn't) + folder rename/delete descendant cascade + link archive/unarchive incoming recompute + `move_item` review-history reset. **Open · Charter G5 · /migration.**
> - **PJ-075** — G6 residual: `review-pulse.json` non-atomic (`review.rs:762`) + swallowed review-priority writes (`ReviewerView.svelte:286`, `ReviewStatusPanel.svelte:97`). *(Bulk of G6 landed — libraries.json + 4× universe.json in v3.38.)* **Open · Charter G6.**
> - **PJ-076** — G7: the write-gate lost-update / staleness guard never fires on the real save path (`write_gate.rs`, W2-13). **Open · Charter G7.**
> - **PJ-077** — G8 residual: two sync-walk commands → `(async)` (`provenance.rs:100`, `libraries.rs:5148`). *(Core G8 shipped.)* **Open · Charter G8.**
> - **PJ-078** — `map.rs::constellation_map_universe` — the LAST remaining Rule-8 read-time filesystem walk; the OrgChart/Map tree walks disk on every open → move to write-time/index-read. *(Supersedes the mistaken PJ-027 "Map already write-time" closure — MIG-078 found it walks.)* **Open · orient. §12 · /migration.**
> - **PJ-079** — `get_360_view` index-read rewrite (still disk-re-walks; async wrapper shipped v3.22). **Open · /migration.**
> - **PJ-080** — `init_db` cold-init profiling (20–40 s, UNMEASURED — measure before optimizing). **Open · Rule-8 measurement first.**
> - **PJ-081** — the §12 doc-drift batch + orientation BODY refresh (see Group 5 above for the full list, incl. the now-false "no frontend test harness" row). **Open · hygiene.**
> - **PJ-082** — search-query grammar hardcodes 7 built-in link types (`store.ts:2071`); `supersedes [[X]]` / custom-type filters don't work; + MIG-067 §F/§I. *(Distinct from PJ-063, which is DONE.)* **Open · single-surface fix.**
>
> ### Honest caveats (UNVERIFIED)
> - **Workbench → Collections** (v3.27 says shipped as MIG-092; some v3.33–37 preamble QUEUED lists still carry it) and **Quick Switcher** (MIG-093): git log confirms MIG-092 + MIG-093 both shipped, so the later "queued" mentions are stale preamble text — treated as DONE, not re-filed.
> - **PJ-008/009/010** render-dedup / alias-bleed were NOT re-verified against the MIG-079 panel rewrite — Group 4 flags "re-verify first".
>
> ---

> **What changed in v1.17** (PJ-069 opened + its filed counts corrected after the SO#8 cross-check + adversarial re-audit; concept + shape Boss-ratified):
>
> ### PJ-069 — concept-paper delivered, entry recomposed
> - **PJ-069** — **Whole-entity deduplication pass** — is now **IN PROGRESS (concept-paper-first)**. Concept paper: **`docs/concept-papers/PJ-069-Whole-Entity-Deduplication-Concept-Paper.md`**. The v1.16 filing (below) is **STALE in four ways** — the map it quoted (`wf_1d470cb8-9e8`) was drawn 2026-07-05, the *same day* the Navigator was deleted, and predates MIG-092/093. Re-audit `wf_2ae0f8c0-d59` (18 agents, adversarial verify) recomposed it:
>   - **Horse (ratified 2026-07-06):** *"one home per capability — every capability has one owner; every other surface mounts it instead of re-implementing. The whole entity is one system of single-owned capabilities, not a federation of copies."* Priority = **answer-duplication first**; scope = **the 7 filed clusters + 9 newly-found cross-cluster families**; first step = **the dead-code cull**.
>   - **Recomposed counts (map → today, verified live):** tags **6→6** (a *different* six — Navigator's 2 deleted, SS clone + Rust fs-walk pair surfaced; OrgChart/SearchHub are sanctioned mounts) · folders **4→5** (Navigator deleted; MoveDialog picker, DigestPane tree, the LIVE `map.rs` builder surfaced) · recents **3→2 hand-rolls** · orphan/fragile **5→9 surfaces / 5 divergent definitions** · hubs **3→6 in-degree substrates** (the CNS "one home" register disagrees with the Backlinks badge) · note-lists **9→26 live** (NoteRow/NoteList shipped, **1** adopter) · **confidence 2→0** — already resolved by the shared `ConfidencePicker` (MIG-077 §F, `fa98bf6b`, 2026-06-29 — *six days before the map*).
>   - **Four stale parts corrected:** (1) the counts named DELETED surfaces (Navigator pane, NavBrowserPane) — gone with MIG-091 `dfed6333`; (2) the "coordinate with MIG-090" clause is obsolete — MIG-090 resolved into MIG-091 (File Explorer) + MIG-092 (Collections); the `NoteRow`/`NoteList` primitive already **shipped** on `main`; (3) confidence-menu ×2 is **0** (already unified); (4) the entry predates MIG-092 (which executed the method once — Bookmarks+Workbench → Collections, one owner + two mounts) and MIG-093 (`searchFold`/`switcherRank` seeds; a new recents mount).
>   - **Method (proven twice):** one owner + sanctioned mounts — the MIG-092 unify + `RelatedCandidates` "6 mounts / 5 hosts" model. **9 new families in scope:** Arabic folds (×6 Rust), relative-time copies (+ an i18n violation), library-colour dots ×7, cUniverse federation grouping ×7, name-autocompletes, context-menu duo, search-history/MRU, link-type label lookups, note-open dispatch. **8 confirmed-dead items** queued for a Step 0 cull (TagsPanel, NoteGrid, `/libraries`+NavBar, `store.ts::timeAgo`, `lenses.rs` tag-lens, `bases.rs::scan_by_tag`, `map.rs::constellation_map_data`, the boot-bundle bookmarks fetch).
>   - **Correction to the "Sight is disabled" framing:** CNS v2 (`ConstellationSight2` + `SightPanel`) is **LIVE core**; only Sight v3/v4/v6/v7 + the Constellation *Map* frontend are disabled; `map.rs`'s hierarchy builder is **LIVE** via the default-on OrgChart. Status: **Open · IN PROGRESS · concept+shape ratified · per-cluster rulings → /migration.**
>
> ---

> **What changed in v1.16** (one new PJ filed — the entity-wide duplication debt, Boss-ordered):
> *(Superseded by the v1.17 recomposition above; the original filing is retained below for the diff trail.)*
>
> ### New PJ filed
> - **PJ-069** — **Whole-entity deduplication pass** (Boss-ruled 2026-07-05, from the whole-entity mapping `wf_1d470cb8-9e8` done for the Navigator concept). The Boss's law: *"we have to avoid duplication among all Constellation's core plugins/functions. They all should complement each other as a whole entity."* The map found pre-existing duplication debt ACROSS the entity, independent of the Navigator: **tag browsing in ~6 places** (Navigator pane · Dashboard All-tags · note Tags tab · OrgChart source · SearchHub category · NavBrowserPane's duplicate tag-tree builder) · **folder-hierarchy browsing ×4** (Explorer · Navigator pane · OrgChart · the disabled Map) · **recents ×3** · **orphan/fragile diagnostics ×5** (Reviewer lenses · 360° gaps · Tension panel · CNS blind spots · Sky) · **"hubs" rendered by 3 surfaces while CNS's paper claims exclusive ownership** ("this is their one home") · **note-list rendering hand-rolled in ~9 surfaces** · two hand-rolled copies of the confidence menu (Backlinks/Outgoing). Method when taken up: per-cluster rulings (one owner + sanctioned mounts, the Suggested-Connections "one list, five mounts" pattern is the model), then /migration-scale consolidation. Coordinate with MIG-090 (whose shared note-list primitive addresses the ×9 cluster) — do not double-fix. Status: **Open · Boss-ordered · Effort: per-cluster rulings → /migration.**
>
> **Queue note:** MIG-090's Navigator concept is RATIFIED (2026-07-05): the horse — "the Navigator translates what is in the user's mind into the notes it refers to — and holds that working set while the user works it" — with Form C (Workbench + Intent Bar). Architect delta in flight. The Dataview vestige was removed entirely same day (Boss: "We don't have Dataview!"; commit `40f59f44`).


> **What changed in v1.15** (one new PJ filed — the Second-Screen contextual-companion rework, Boss-parked):
>
> ### New PJ filed
> - **PJ-068** — **Second Screen: contextual companion rework** (Boss-conceived + parked 2026-07-04). During the F2′ Test-4 arc the Boss issued two governing rulings: **the SS must be CONTEXTUAL to the main screen**, and **it must NOT REPLICATE what the main screen already displays** — then ordered the full SS history + concept read and a paper written, and parked the rework: “We will set it aside for now… and tackle it in due time.” The paper is **`docs/concept-papers/PJ-068-Second-Screen-Contextual-Companion-Concept-Paper.md`** (history dig workflow `wf_42ec73f0-794`, 5 readers). Its replication audit classifies every SS mode: **REPLICATES** — Navigator companion (the same `NotebookNavigator` the main sidebar shows at the same moment), OrgChart mode (also UNREACHABLE dead code); **MIXED** — Map companion, Index term leg, Universe Dashboard (ambient), plus a NON-CONTEXTUAL fallback tab-strip editor; **COMPLEMENTS (keep)** — Sky graph companion, Split comparison, Editor-panels migration, Index compare leg. Structural riders: the editor-panels mode-shadowing drift (if-chain → needs a real mode state machine; static read, not runtime-reproduced), the dead `screen:open-note` wire, Rule-8 re-walk reads, alias-blind `buildSkyData`, hardcoded-English ×15 + missing right-click, and 5 doc-drift items (incl. CP-26 still listing the FIXED `setActiveUniverse` breach, and the manual’s mode table documenting the unreachable mode). Per-mode retire/redesign decisions are **Boss rulings at reopen**; then /migration. What it must NOT do: add operations to the SS, remove the two-monitor gate, or make the SS self-initiate. Status: **Open · PARKED (Boss 2026-07-04) · Concept paper DONE · Effort at reopen: Boss rulings → /migration.**
>
> **Queue note:** the active work at v1.15 filing is the app-freeze program’s Batch W (shipped `d9f8bd80`, Boss test pending) with deferred batches L′/F1/F3 next in rank; PJ-068 joins the deferred Concept-Paper-done tier alongside PJ-065/PJ-067’s pattern.


> **What changed in v1.14** (MIG-086 in flight — folding in the "typed links in frontmatter" discovery; one new PJ filed):
>
> ### MIG-086 (Suggested-Relatedness → One-Click Typed Link) — in flight
> §A (`suggest_related_notes` BM25 "More Like This") + §B (read-only `<RelatedCandidates>` list) SHIPPED + Boss-validated. §C (the one-click typed-link action: `<LinkTypePicker>` + headless `addLinkToNote`) built + Boss-tested PASS, then **reopened by a Boss design discovery (2026-06-24)**: a typed link appended as dangling `[[type::target]]` body text "without context is illogical." Cross-system research (Obsidian Properties, Dataview, **Breadcrumbs**, Logseq/Roam/Tana, RDF/property-graph KR theory — workflow `wf_ce7372d6-d02`, fact-checked) confirmed: a *declared* relationship (the one-click kind, not woven into prose) belongs as **frontmatter metadata**, not body text. **Boss ruling 2026-06-24: continue MIG-086 §C and FOLD the frontmatter-typed-links approach into this migration** — the connect writes a frontmatter typed-link property; `index_note` learns to read frontmatter typed-links (dual-source with body); inline body `[[type::target]]` stays for *contextual* links; one unified `note_links` index. Earned properties (weight/confidence/traversal) remain index-maintained. (Architect doc + updated §C plan pending; §C currently uncommitted in the working tree pending the fold.)
> **Doc-drift surfaced (must reflect in orientation):** the CLAUDE.md "Living Link = first-class `LINK` file on disk (source of truth) + `note_links` index" claim **does not exist in code** — links are 100% body-`[[type::target]]`-derived into `note_links`; the rich properties live only in that table. The fold is the first real decision on where a link's *birth* is authored.
>
> ### New PJ filed
> - **PJ-065** — **Brand-new frontmatter-structured link type** (Boss-conceived 2026-06-24). A NEW kind of typed link designed specifically for frontmatter "Properties": e.g. linking many child notes to a single parent note (a Table-of-Contents / hierarchy relation). Distinct from the 8 cognitive typed links — this is a *structural/containment* relation with high-value use-cases (authors & screenwriters organizing chapters/scenes/arcs under a work; outline/MOC hierarchies; "the sky is the limit"). **Research-first / Concept-Paper-first; /migration-scale.** Study the precedent (Breadcrumbs' up/down/parent/child direction model + auto-implied reverse; Tana's "Part of" semantic field; Obsidian Properties), decide the schema (property shape, direction, reverse-implication, how it renders/edits, how Sky/Reviewer/Org-chart consume it), THEN build. Depends on the MIG-086 frontmatter-typed-links foundation landing first. Status: **Open · Deferred (future)** · **Effort: /migration (research → Concept Paper → Architect → Plan → Build → Audit).**
>
> - **PJ-066** — **Sky-trigger reindex storm on link-dense notes** (P1 perf; surfaced 2026-06-24 by the MIG-086 §F2 connect-latency diagnosis, workflow `wf_9d58a4b6-d73`). `index_note` DELETEs+re-INSERTs ALL of a note's `note_links` edges on every reindex; each edge fires `note_links_sky_stratum_ai/ad` + `note_links_sky_maturity_ai/ad`, whose `STRATUM_SQL_EXPR`/`MATURITY_SQL_EXPR` (search.rs:183-317) run repeated `COUNT(DISTINCT source_path) FROM note_links WHERE status… AND (target_name = id OR target_name IN (alias subquery))` over the 233,995-row table. For a link-dense note (e.g. a Wikipedia import) that's O(edges × big-subquery) ≈ **~2 minutes**. PRE-EXISTING (every link-dense note's *save* pays it too — hidden because saves reindex fire-and-forget). MIG-086 §F2 exposed it by `await`ing the reindex; fixed THERE by making the connect non-blocking (fire-and-forget + optimistic Reviewer). **This PJ is the ROOT fix.** Options (need Rule-8 measurement on the 7,660-note / 234k-sky-link universe): (a) composite index `note_links(target_name, status, source_path)` to fast-path the COUNT(DISTINCT) subqueries; (b) defer stratum+maturity recompute OFF the per-edge write path into a batched/dirty-flag drain (MIG-079 §C.2a `incoming_count` backfill pattern; `sky_nodes.enrichment_dirty` + a stamp-gated worker already exist); (c) make `index_note` diff edges (only fire triggers for changed edges, not DELETE-all+INSERT-all). Belongs to the SKY/MIG-079 perf domain — Concept/Architect-first, /migration-scale. Status: **Open · P1 · /migration (needs measurement).** Also noted: get_due_notes Lens-2 per-note `stale_probe_sql` (review.rs:315-334) is a secondary contributor — index `note_links(source_path, weight)` if it shows in measurement.
>
 - **PJ-067** — **Living Link Relationship Model v2 — Concept Paper** (Boss-conceived 2026-06-24, from the typology dig `wf_f97e9d18-518`; research doc: `docs/Living-Link-Relationship-Typology-Research-2026-06-24.md`). The directional-vs-symmetric discovery opened a much larger map: a typed link varies on **DIMENSIONS** (symmetry · **transitivity** · **inverse/converse** · **arity (binary vs n-ary)** · cardinality · **taxonomic-vs-thematic**) and belongs to **FAMILIES** (class-inclusion ✓, contrast ✓, causal ✓, evidential ✓, part-whole ✓-coarse, derivation ✓, supersession ✓; **uncharted:** thematic/functional [used-for, prerequisite-of, precedes, near], **analogy/structure-mapping** [analogous-to/maps-to — the synthesis engine], **n-ary synthesis**, argument-attack [**undercuts/undermines**], question relations [**problematizes/answers**], qualifies/limits). Maps to the **Five Acts**: the thinnest acts — **Tension** (no inference/premise attacks, no questions) and **Synthesis** (no analogy, no n-ary) — are exactly the gap for a *formulation* tool. **Two load-bearing flags:** (1) rename the proposed `complements` → **`co-completes`/`jointly-constitutes`** — lexical "complementarity" means the OPPOSITE (mutually-exclusive opposition, dead/alive); (2) "together they form an idea" is genuinely **n-ary** (a synthesis node), not a pairwise symmetric link. **Concept-Paper-FIRST (Boss defines the cognitive vocabulary), then /migration.** The symmetric tier + `co-completes` + all new families/dimensions live HERE, not in MIG-086. Status: **Open · Deferred (Concept-Paper-first).** Depends on MIG-086's frontmatter foundation.
>
> **Top of queue (v1.14):** MIG-086's frontmatter fold is the *active* work (finish §D/§E on the CURRENT 8 + directional/symmetric model); **PJ-066 (P1 perf)** is the highest-leverage new open item; PJ-065 (frontmatter parent/TOC type) + PJ-067 (Relationship Model v2) are the deferred Concept-Paper-first vocabulary frontiers.

> **What changed in v1.13** (RECONCILIATION — the ledger was ~3 weeks + 37 migration numbers stale; MIG-036→072 folded into orientation §8 (v2.61); PJ deltas applied; one ledger error corrected; new top-of-queue):
>
> ### Why this version exists
> v1.12 (2026-05-19) was the last backlog refresh. Since then **37 migration numbers (MIG-036 → MIG-072)** were opened — yet **none** appeared in either this ledger or the orientation §8 Migrations table (which had stopped at MIG-035 since the v2.16 refresh on 2026-05-18). The 2026-06-09 handover compounded this by naming **v1.9** as the latest Pending Jobs file when v1.10 / v1.11 / v1.12 already existed on disk. This version reconciles both ledgers against the orientation preambles (v2.17→v2.60) + session logs (2026-05-19→06-09). **The authoritative migration ledger now lives in orientation §8 (v2.61)**; this file tracks PJs and references §8 for MIG status.
>
> ### Migrations since v1.12 — summary (full table in orientation §8 v2.61)
> - **Shipped / Closed (23):** MIG-038 (disable Sight+Map → external "Wings"), 039 (The Cataloger), 040 (NSC), 041 (term_vocab bigram shrink), 042 (drop `bridge_concept_id` — **closes PJ-016**), 043/044/045 (NSC Core Plug-in P1–P3 + Universe Digest), 055 (Constellation Base rebuild), 056 (Cross-Universe Federation), 057 (Lexicon expansion), 058/059 (federated search latency → sub-second), 060 (Base threading gestures), 061/062 (boot-snapshot + filesystem-walk federation + Tag Browser), 065 (Unified Base), 066 (Living-Link columns), 067 (**User-Definable Link Types** — the Link-Type Registry), 069 (Style Presets), 070 (**Style Setter**), 071 (theme subsystem removed), 072 (Sky View vocabulary under the Style Setter — milestone `localization-complete`).
> - **Reverted (4):** MIG-046/047/048 — the **Constellation Mind** local-LLM stack (Fanar 1.9B) shipped end-to-end then was fully reverted (v2.34, `a9cf4d62`): CPU at ~5 tok/s didn't justify the value. MIG-054 (first Base attempt) reverted same-day → superseded by MIG-055.
> - **Reserved / never-opened (8):** MIG-049→053 (Constellation-Mind roadmap numbers, abandoned with the revert), **063/064 — the remaining ~6 cross-universe federation surfaces (still pending)**, 068 (rank-aware column sort, deferred).
> - **Dormant / Frozen (2):** MIG-036 (Sight v7 redesign, dormant — over-engineered), 037 (Sight v6.3 Time Dome, frozen) — both under MIG-038's Sight-disabled umbrella.
>
> ### PJ deltas applied this version
> - **PJ-016** (drop `term_vocab.bridge_concept_id`) → **DONE via MIG-042** (orientation v2.25). Moved to §9.
> - **PJ-011** (Constellation Map) → **DORMANT** — Map was disabled in core by MIG-038 (re-categorized as a future "Wings" plug-in). Stays filed; not actionable while Map is off.
> - **Ledger-error correction (Eisa-confirmed):** the 2026-05-29 session log labelled two federation-scale fixes **"PJ-10 / PJ-11"** — colliding with the real **PJ-010** (Unlinked Mentions) and **PJ-011** (Map). They were actually filed *unnumbered* on 2026-05-28 ("PJ-NNN-A/B"). They are allocated proper, never-before-used numbers here: **PJ-061** (Sky View federated node sizing, `f05fe6f9`) and **PJ-062** (CNS gravity-well canvas, `9a2d9890`), both **DONE 2026-05-29**. The canonical PJ-010 / PJ-011 are untouched (renumbering is forbidden — these were never formally numbered).
> - **PJ-001** confirmed **DONE via MIG-015** (already correct in the §1 body; the stale memory `project_mig013_v2_migration_blocking_boot` is corrected separately, in this same turn).
>
> ### Newly filed (PJ-059 → PJ-064 — see the "Newly filed" section before §9)
> - **PJ-059** — Sight per-note search/finder (open; **dormant** while Sight is disabled).
> - **PJ-060** — `index_note` cache-hit short-circuit fix (open; **P1 — flagged 2026-05-19 as "the single most-leveraged open fix"**; a write-time-derivation blocker).
> - **PJ-061 / PJ-062** — the two federation fixes above (both **DONE**).
> - **PJ-063** — `note_links.link_type` is globally `'relates'` (open; foundational /migration; **re-verify under MIG-067**, which shipped the Link-Type Registry after this was first observed).
> - **PJ-064** — Style Setter: more font types in the final version (open; minor — named/saved colour swatches already shipped in MIG-070).
>
> ### Already-closed in the v1.12 preamble, now reflected in §9 Done
> PJ-035 (DONE, MIG-019 §2B) · PJ-036 (Abandoned 2026-05-18) · PJ-038 (Superseded by Sight v6) · PJ-040 (DONE, MIG-022 §D `c072700`) · PJ-015 (Abandoned 2026-05-18) · PJ-052 / 053 / 054 / 055 / 056 / 057 / 058 (DONE 2026-05-18/19 in the MIG-026 + Sight-delivery cascade). *(These were announced in v1.12's preamble but never moved into its §9 Done body table; this version completes that bookkeeping.)*
>
> ### New top-of-queue (replaces v1.12's, which still listed the superseded PJ-038 as In-Progress)
> 1. **PJ-060** — `index_note` cache short-circuit (P1 blocker, highest leverage).
> 2. **PJ-005 / MIG-007** — Links Settings tab (P1 user-facing; no Architect yet).
> 3. **PJ-063** — `note_links.link_type` 'relates' bug (foundational /migration; re-verify under MIG-067 first).
> 4. **PJ-002 / PJ-003** — `cid_cn` collision scrub + rename-collision popup.
> 5. **PJ-017 / PJ-018 / PJ-019** — MIG-013 cleanup remainder (PJ-016 now done).
> 6. Backlog: remaining federation surfaces (MIG-063/064) · CECE i18n (PJ-040 done; PJ-041–043 open) · MIG-022 polish (PJ-044–050) · PJ-008/009 typed-link dedupe · MIG-023 Warrant Research (reserved, Concept-Paper-first).
>
> ### Reconciliation depth (honest note)
> This version refreshes the **preamble, top-of-queue, the §9 Done index, and the PJ entries with hard evidence of a status change**. The ~60 carried-forward PJ bodies keep their v1.12 status unless evidenced above; a deeper per-PJ code-audit (and a refresh of the stale "Cross-references" appendix) is a low-priority follow-up, not done here. Every status above cites a commit, an orientation version, or a session-log date.
>
> **What changed in v1.12** (MIG-026 SHIPPED — Sight v6.3 24-tradition expansion + full 15-locale localization · Phase μ ship-gate audit closed clean · 5 new PJs filed for deferred polish):
>
> ### MIG-026 SHIPPED
>
> 24-tradition expansion + 9 shape renderers + user-definable layer (declarative JSON + TS plugin loader) + full localization across 15 locales — all merged on `main` between Phase γ (2026-05-17 evening) and Phase μ (2026-05-18). Boss-validated across 3 stages (Arabic-localization Stage 1 + RTL-chevron polish Stage 2 + cross-locale spot-check Stage 3 in zh / de / ru: **all PASS**). Phase μ Migration Rule audit (3 parallel agents on invariants / drift / migration-path) returned **zero blockers**: all 10 invariants PASS, 2 advisories, 1 high-severity doc-drift (this PJ file's v1.11 header), 2 low-severity drift items.
>
> ### MIG-026 collision finally resolved
>
> v1.11 reserved MIG-026 for "Sight v5 Layer 3 recommendation (V3-§7.b llama.cpp wiring)" — that allocation was contradicted when MIG-026 was actually opened in 2026-05-17 as the Sight v6.3 24-tradition expansion. The Layer-3 recommendation work folds into a future MIG-029 or later (TBD). MIG-027 is also reallocated: it shipped 2026-05-17 evening as Sight-follows-the-interface-theme (chrome/semantic color split), not the Sight v5 Layer 4 coaching workstream.
>
> ### 5 new PJs filed
>
> - **PJ-052 — Concept Paper v4.1** — **DONE 2026-05-18** (`docs/Constellation-Sight-Concept-Paper-v4.1.md`, 919 lines / ~14,242 words; 24 tradition subsections at ~400 words each + new §3.5 nine-shape-renderer architecture + §3.6 user-definable plugin layer + invariants 12+13 on $t labelize + plugin label passthrough + §4.2 trimmed to Mohist-only v1-preview survivor + §4.1.2 pramāṇa / §4.1.3 masādir doc-drift corrected E/S/W/N). Three additional doc-drift items surfaced during the write — folded into **PJ-057** (below).
> - **PJ-053 — λ-fix-6 native-quality translation re-audit** — **DONE 2026-05-18** (4 parallel polish agents). 192 keys polished across 7 locales: ru (35 — wrong-script Cyrillic + transliteration glosses), hi (23 — wrong-script Devanagari + glosses), de + fr + es (42 each — bare transliteration → `transliteration · native-gloss` pattern), pt (6 — PT-PT → PT-BR dialect unification beyond the audit's named 3), zh (2 — extension aria/tooltip transliteration → native Chinese). Now every Sunni-Islamic, Sanskrit pramāṇa, Hebrew PaRDeS/middot, Greek/Latin Husserl, and Arabic Ibn Rushd / Shāṭibī technical term in the 7 polished locales carries a target-language gloss matching the ar/zh/ko quality bar.
> - **PJ-054 — Sight v6 vitest test runner** — **DONE 2026-05-19** in MIG-030. Installed `vitest@4.1.6`; wrote `tradition-isolation.test.ts` (Plan §14.1, channel-isolation invariant for all 24 traditions) + `tradition-perf.test.ts` (Plan §14.2, ≤16ms switch on 7,636-note universe) + `vitest.config.ts` (scope: exclude worktree duplicates + the still-deferred playwright layout-fidelity test). **58/58 tests pass.**
> - **PJ-055 — User-plugin label schema warning** — **DONE 2026-05-18** in commit `e63ee0c7`. `docs/traditions/schema/tradition.v1.schema.json` top-level description now warns that dotted-path-shaped literal labels would collide with Constellation's own i18n key namespace.
> - **PJ-056 — MIG-026 drift cleanup** — **DONE 2026-05-18** (3 stale doc comments fixed inline in commit `e63ee0c7`: dome.ts STRATUM_LABELS, types.ts:444 TraditionModule.name, traditions/index.ts:222 FAMILIES.label). The literal-deletion phase (24 `name:` literals + 10 `FAMILIES[*].label` literals) is **closed by Eisa decision 2026-05-18**: accept the duplication as documentation. The literals serve as the canonical English source-of-truth that en.json mirrors and as a defensive renderer fallback for the unsupported-locale + missing-en-entry edge case. No further work planned.
> - **PJ-057 — Post-MIG-026 doc-drift surfaced during Concept Paper v4.1 write** (3 items): (a) v4.0 §4.2.3 Mohist citation says *Mòzǐ* ch. 35; the shipped manifest cites Book IX — pick one canonical citation. (b) `docs/Sight-vNext-MockB1-Toggle.svg` and `docs/sight-redesign-v0.2-mockE-tradition-registers.svg` show pre-expansion 7-tradition state; needs a fresh 24-tradition variant. (c) Concept Paper §9.1 (and the orientation §17 list of unread files) should footnote that `_manifests.generated.ts` is prebuild-generated and must regenerate when manifests change. **(a)** no-op 2026-05-19 (v4.1 already cites manifest-canonical form); **(c)** DONE 2026-05-19 in `f327d758` (Concept Paper §9.1/§9.3 updated); **(b)** still open (visual design work).
> - **PJ-058 — Constellation Sight Subsystem Concept Paper v1.0** — **DONE 2026-05-19**. New ~6,700-word scholarly document at `docs/Constellation-Sight-Subsystem-Concept-Paper-v1.0.md` defining Sight as a subsystem of Constellation, enumerating its 8 functions (F1-F8), placing it in the 4-stratum subsystem map (structural / authoring / diagnostic-visualization / infrastructure), documenting its 8 architectural invariants (I1-I8), and tracing v2 → v6.3 subsystem history. Complement to the existing `Constellation-Sight-Concept-Paper-v4.1.md` (internal-design contract): subsystem paper = "what Sight is in Constellation"; v4.1 = "how Sight is built internally". Filed + delivered same-turn per Eisa direction.
>
> ### What did NOT close in v1.12
>
> - PJ-005 (MIG-007 Links Settings) — still open.
> - PJ-002 (cid_cn collision scrub) — still open.
> - PJ-008 / PJ-009 (typed-link duplication) — still open.
> - PJ-044/046/047/048/049/050 — MIG-022 polish backlog — still open.
>
> ### What did close (recap)
>
> No prior-open PJ closed by this cascade. The 5 new PJs (052–056) are all newly filed.
>
> ### Post-MIG-026 §μ state-of-standing closures (2026-05-18 turn-late, Eisa decisions)
>
> Triggered by an Eisa-requested triage of all remaining work. The 3-agent audit (MIG entries / PJ-001-040 / PJ-041-057) surfaced 5 NEEDS-DECISION items. Eisa rulings:
>
> - **MIG-005** Alias-aware in-memory inbound — **ABANDONED**. Steps 1-3 (`§121/§122/§123`) stay shipped; Steps 4-8 abandoned after the fabrication-catch pause. Reason: low leverage given current priorities.
> - **PJ-015** 360.3D Stratification Matrix guidance doc — **ABANDONED**. Reason: matrix-UX dependency hasn't moved; doc was low-leverage. Refile fresh if user-facing need surfaces.
> - **PJ-036** Sight layer peeling — **ABANDONED**. Reason: Sight v6's facet sidebar substitutes for the mechanic; the v2 Concept Paper §2.2 mechanism is no longer relevant under v6 architecture.
> - **PJ-056** literal-deletion sub-question — **CLOSED as documentation**. The 24 `name:` + 10 `FAMILIES.label` literals stay; they're canonical EN source-of-truth + defensive renderer fallback.
> - **MIG-022 §N** — **CLOSED 2026-05-18** (this turn). The §N audit landed 2026-05-12 (4 docs at `lab/reports/MIG-022-§N-*.md`) but Eisa never explicitly locked D-N1/D-N2; the P1 trigger-coverage fix shipped in commit `1240984d` (MIG-024 §0 UPSERT) implicitly chose option (α) + timing (a). Retroactive §8 close-out section appended to `MIG-022-§N-FINAL-INTEGRATION-AUDIT.md` recording the close. **MIG-022 status → DONE**. P2/P3 polish items (F2-F7) remain in cleanup backlog as future polish MIG; F8 (i18n gap) partially resolved by MIG-026 §λ across all 15 locales.
>
> ### Drift fixed during the audit (ledger reflected reality after the closures)
>
> - **PJ-035** body status "Open" → **DONE in MIG-019 §2B** (`16063735`). Milky Way density wash shipped the TF-IDF mechanic.
> - **PJ-040** body status "Open" → **DONE in MIG-022 §D** (`c072700`). Already noted in v1.11 preamble; body never flipped.
> - **PJ-038** body status "In-Progress" → **SUPERSEDED by Sight v6 / MIG-024 → MIG-027**. The 3-MIG Sight v3 trajectory was abandoned at commit `29ce0101`.
>
> ### Top of queue rotates (post-MIG-022 §N close)
>
> 1. **PJ-005 / MIG-007** — Links Settings tab (P1 user-facing; no Architect yet).
> 2. **PJ-002** — `cid_cn` collision scrub utility (P1 mini-MIG).
> 3. **PJ-003** — Rename-collision popup (P1 UX).
> 4. **PJ-008 + PJ-009** — Typed-link duplication pair (P2 single-file fixes).
> 5. **PJ-016/017/018/019 bundle** — MIG-013 cleanup MIG (4 PJs → 1 MIG).
> 6. **MIG-023** — Constellation Warrant Research workstream (Concept Paper first).
>
> **Done count after v1.12 post-audit**: 12 (+5 — PJ-052 Concept Paper v4.1 + PJ-053 λ-fix-6 + PJ-055 schema warning + PJ-056 drift cleanup + PJ-035 status-correction + PJ-040 status-correction). **Cancelled / Abandoned**: 4 (PJ-015 + PJ-036 abandoned 2026-05-18; PJ-034 retained from earlier; MIG-005 abandoned). **Rejected**: 1 (PJ-037). **Superseded**: 1 (PJ-038). **Open PJs**: 48.
>
> ### Post-2026-05-18 same-day rollup (2026-05-19 Sight delivery cascade)
>
> Eisa cascade direction: "Proceed with Tiers 1-3 and drop Tier 4. Whatever is related to Sight v5 shall be abandoned." Then: "Add to it developing Constellation Sight Concept paper."
>
> Closed same-day this turn:
> - **PJ-054** — Sight v6 vitest test runner — DONE in MIG-030 (`f327d758`). 58/58 tests pass.
> - **PJ-057.a** + **PJ-057.c** — Mohist citation no-op + prebuild footnote in Concept Paper §9 — DONE in MIG-032 (`f327d758`).
> - **PJ-058** — Constellation Sight Subsystem Concept Paper v1.0 — DONE this turn (~6,700 words; new doc at `docs/Constellation-Sight-Subsystem-Concept-Paper-v1.0.md`).
>
> Plus: **MIG-028** (Sight v5 retirement) + **MIG-030** (vitest) + **MIG-031** (λ-fix-6.b fa/he/ja/tr canvas polish — 57 keys across 3 locales) + **MIG-032** (Tier 3 housekeeping) all shipped in `f327d758` (the Sight delivery cascade). Plus **BUG-fix** (Sight anchor Shift+click) shipped in `4b20795b`.
>
> **Done count after 2026-05-19 rollup**: 12 → **15** (+3 — PJ-054 + PJ-057.a/c + PJ-058). **Open PJs**: 48 → **45**.

> **What changed in v1.11** (Sight v5 Concept Paper canonical · MIG-022 collision resolved · MIG-024 / 025 / 026 / 027 reserved · 1 new PJ filed):
>
> ### Sight v5 Concept Paper v3.1 ratified — MIG number-collision resolved
>
> `docs/Constellation-Sight-Concept-Paper-v3.1.md` is now the canonical Sight v5 design contract (Eisa-approved 2026-05-12 on all 6 validation points). Resolution of the MIG-022 number-collision committed in writing:
>
> - **MIG-022** stays with the gap-analysis-response cascade (§0 + §D + §E + §A shipped; §B Rust foundation shipped; §B UI overlay contradicted-and-deferred; §N can fire now).
> - **MIG-023** stays reserved for the Constellation Warrant Research workstream (per Eisa's D-C1 commitment 2026-05-11).
> - **MIG-024** = Sight v5 Layer 1 visual foundation. Next Architect doc.
> - **MIG-025** = Sight v5 Layer 2 diagnostic.
> - **MIG-026** = Sight v5 Layer 3 recommendation (V3-§7.b llama.cpp wiring lands here as a sub-phase).
> - **MIG-027** = Sight v5 Layer 4 coaching.
> - **Cleanup MIG (TBD)** retires Sight v2 / v3 / v4 components and `lenses.rs::apply_lens` once v5 stable across multiple sessions.
>
> ### MIG-022 §B.5 contradicted-and-deferred
>
> The original MIG-022 Plan §B.5 was *"UI surface per D-B4.β: Sight v3 overlay"*. With Sight v3 explicitly retired by Concept Paper v2.0/v3.1 and Eisa's directive *"we have to focus on this version and avoid wasting our time and effort on patching earlier versions,"* §B.5 is contradicted by the canonical Sight target and **deferred indefinitely**. The temporal-axis history Rust foundation (§B.1–§B.4) shipped and is reusable; the natural consumer is now Sight v5 Layer 2's "growth trajectory" diagnostic within MIG-025.
>
> §B.6 (i18n + help + UM for the §B.5 UI) is also deferred since §B.5 is what would have driven its content.
>
> No PJ-NNN allocated for this deferral — the Rust foundation is in `note_state_history` waiting to be consumed; when Layer 2 designs against it, that consumption happens naturally inside MIG-025.
>
> ### New PJ filed
>
> - **PJ-051 — Mock B1 SVG follow-up edits as Sight v5 evolves.** The Mock B1 was updated 2026-05-12 to add the 7th button (P) per Concept Paper v3.1 §6. Future v3.x revisions of the Concept Paper that touch the visual contract (e.g., if a different mode order is decided, if the legend wording changes, if a Layer 2 "Findings" pill is added to the dome) will need parallel edits to `docs/Sight-vNext-MockB1-Toggle.svg`. Original 6-button version preserved at `Sight-vNext-MockB1-Toggle-v1.svg`. P3 housekeeping; fires when triggered, not on a schedule.
>
> ### What did NOT close in v1.11
>
> - PJ-044 / PJ-046 / PJ-047 / PJ-048 / PJ-049 / PJ-050 — all still open (the 6 polish items from MIG-022 §A Gate 3). Candidates for §N close-out cleanup or future polish MIG.
> - PJ-005 — MIG-007: Links Settings tab — still open.
> - PJ-002 — cid_cn collision scrub — still open.
> - PJ-008 — Outgoing Links typed-link dedupe — still open.
>
> ### What did close (recap from prior versions)
>
> 5 PJs closed during MIG-022 cascade (per v1.10): PJ-040 (in §D), PJ-041 (§E.2.b), PJ-042 (§E.1), PJ-043 (§E.3.d), PJ-045 (inline in §E.2.a).
>
> ### Top of queue rotates
>
> 1. **MIG-024** — Sight v5 visual foundation Architect doc (next-up).
> 2. MIG-022 §N — final integration audit + close-out (parallel-ready).
> 3. MIG-023 — Warrant Research Concept Paper (after MIG-022 §N closes).
> 4. PJ-005 — MIG-007: Links Settings tab.
> 5. PJ-044 / 046 / 047 / 048 / 049 / 050 — MIG-022 polish backlog.
>
> **Done count after v1.11**: 7 (unchanged — PJ-038 stays In-Progress umbrella; the gap-analysis-response work shipped under MIG-022 isn't on the PJ ledger). **Cancelled**: 1. **Rejected**: 1.

> **What changed in v1.10** (MIG-022 §A closes Gate 3 PASS; 5 PJs closed during MIG-022 §0+D+E+A; 7 new PJs filed during the cascade):
>
> ### MIG-022 Phase 3 Build first-half COMPLETE
>
> §0 (cleanup) + §D (PJ-040 fix) + §E (full i18n) + §A (full YAML metadata + supersedes typed-link + ikhtilāf widget per D-A4.α + Epistemic Metadata help topic + UM chapter in 15 locales) all shipped. Boss-Test Gates 1 + 2 + 3 all PASS. ~30 commits across the cascade including 4 Boss-test catches landed inline.
>
> ### Closed during the §0+D+E+A cascade
>
> - **PJ-040 — Done** (closed in §D `c072700`): UA partial-frontmatter short-circuit refactored to per-axis dispatch; both SOURCES and CONTENT TYPE sections now appear on cards with partial frontmatter.
> - **PJ-041 — Done** (closed in §E.2.b `81fba1a`): cataloger reasoning prose i18n via `(template_key, params)` tuples + 15-locale backfill; non-en/non-ar users see localized reasoning prose on Source Review cards.
> - **PJ-042 — Done** (closed in §E.1 `6c1c3ae`): Confidence enum i18n via `cece.confidence.*` keys + `confidenceLabel()` helper; `[high]` chips now translate.
> - **PJ-043 — Done** (closed in §E.3.d `b9f1ab2`): taxonomy node labels backfilled to 15 locales (277 nodes × 13 = ~3,600 translations); Source Review SOURCES/CONTENT TYPE values render in active locale.
> - **PJ-045 — Done** (closed inline in §E.2.a `894b114`): composite_reasoning paren-dedup; "Set by user in frontmatter (Set in note frontmatter (manual). Sources: ...)." → clean "Set by user in frontmatter." or axis-specific partial message via the new compose template scheme.
>
> ### New PJs filed during the cascade
>
> Six new PJs surfaced from Boss-tests across Gates 1+2+3. None blocked the cascade; all candidates for §N close-out cleanup or future polish MIGs:
>
> - **PJ-044** — Right-click "Classify Sources" menu entry missing in NotePane (Gate 1 catch; Eisa workaround: "Classify open note" button on Source Review header invokes the same IPC). P3 polish.
> - **PJ-046** — Properties panel reorder drag broken (Gate 3 Stage 1 side note). P3 polish.
> - **PJ-047** — Typed-link editor colors visually indistinguishable at body font size (Gate 3 Stage 3). The 9 typed-link colors render correctly per CSS but blend visually at small font. Pre-existing across all 9 types. P3 polish — candidates: bump saturation, add per-type icon, render as actual badge pills instead of colored text.
> - **PJ-048** — Type picker dropdown clips longer non-English translations (Gate 3 Stage 4). Pre-existing fixed-width design clips Arabic/etc. type names to first 2-3 characters; only the longest option (the new "Multi-row list") wraps to 3 lines and shows fully. P3 polish — needs `min-width: max-content` or per-locale width adjustment.
> - **PJ-049** — In-app Help viewer not implemented (Gate 3 Stage 5). Help topic + User Manual files exist on disk in 15 locales (created via §A.4.a + §A.4.b) but no UI access surface — no F1 binding, no command palette entry, no ? menu. Future MIG could ship a help-modal browser or wire the topic catalog into the command palette. P2.
> - **PJ-050** — Backlinks panel section header hardcoded English in non-en locales (Gate 3 Stage 4.1 spotted in Spanish UI). P3 polish.
>
> ### Next-up: §B + §N + MIG-023
>
> - **§B** — Temporal axis (note_state_history table + Sight v3 overlay per D-B4.β). Multi-week. Boss-Test Gate 4 fires after §B.6.
> - **§N** — Final integration audit (3-agent like V3-§11) + MIG-022 ships commit + orientation v-bump.
> - **MIG-023** — Constellation Warrant Research workstream (per Eisa's D-C1 commitment; separate Concept Paper + multi-month research project).
>
> **What changed in v1.9** (V3-§10 cascade closed Gate 3 PASS; three deeper i18n gaps filed for the MIG-022 cascade):
>
> - **MIG-021v3 V3-§10 closed Gate 3 PASS today** — full Option C cascade (Settings UI + en/ar i18n + EN docs + 13-locale i18n backfill + 14-locale help topic + 14-locale User Manual chapter) shipped across 9 commits (`d44b115`, `0054981`, `34a96a9`, `259c333`, `7d6e1a0`, `50a67b0`, `a4438ac`, `4ede8ef`, `54276c3`). Two Boss-test catches landed inline: **V3-§10.A.1** (`4ede8ef` — on-save IPC needed to dispatch the `constellation:classify-and-show` event for the Source Review panel to refresh) and **V3-§10.D.2** (`54276c3` — `settings.classifier.*` block was missing from 13 non-en/non-ar locales).
> - **Three new PJs filed from Gate 3 Stage 5 review**: **PJ-041** (cataloger reasoning prose hardcoded English), **PJ-042** (self_reported_confidence enum bypasses i18n), **PJ-043** (taxonomy node labels en+ar only). All three are structural i18n gaps in the engine output, not just missing translation entries — they belong in the MIG-022 cascade.
> - **Next-up: V3-§11** (final integration audit + MIG-021v3 entire close-out), then **MIG-022** Architect doc (response to the gap analysis from `docs/epistemic-content-gap-analysis.md` + the three new PJs from this version).
>
> **What changed in v1.8** (V3-§9 vertical-axis activation cascade closed; one new PJ filed from Gate 2 observation):
>
> - **MIG-021v3 V3-§9 closed Gate 2 PASS today** — vertical-axis activation cascade A→E + V3-§9.C.2 dual-axis reliability fix shipped (commits `4e0981a`, `d9dfa60`, `ec5527e`, `b18a3ee`, `bf07ae1`, `75807a3`). Cumulative test count went from 67 (start of V3-§9) to 92 (now). Two Boss-test catches (V3-§9.A's ال-prefix gap + V3-§9.C.2's dual-axis reliability gap) made it through pre-commit validation; both fixed inline. See `lab/reports/MIG-021v3-V3-§9-VERTICAL-ACTIVATION-ARCHITECT.md` and `MIG-021v3-V3-§9-VERTICAL-ACTIVATION-PLAN.md`.
> - **New PJ-040 filed**: UA-short-circuit on partial frontmatter (only one axis populated) discards the OTHER catalogers' votes on the unfilled axis. Observed during Gate 2 Stage 6 on `الخط العربي` (frontmatter has `sources:` but not `content_type:` → CONTENT TYPE section vanishes from card). Architectural decision worth surfacing for future improvement; not a regression — same behavior since V3-§1.
> - **Next-up: V3-§10** — Settings + i18n + Help docs + User Manual for CECE chrome.
>
> **What changed in v1.7** (same day as v1.6; MIG-018 ships v3 projection foundation):
>
> **MIG-018 (PJ-038 phase 1 of 3) closes Done** — Sight v3 projection foundation live in production. Six-phase cascade (§1A → §1F) shipped today. Boss test passed all 11 steps. Three-agent audit CLEAN (0 P0/P1/P2/P3). `SIGHT_V3_ENABLED = true` committed. See `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-AUDIT.md`.
>
> **PJ-038 status update**: Confirmed → **In-Progress** (1 of 3 MIGs complete).
>
> **MIG-019 (next-up)** — phase 2 of 3 in v3 trajectory:
> - PJ-035 (content-similarity TF-IDF edges → Milky Way band)
> - Calendar rim (Gregorian default + user-add via Settings)
> - Universe-health card in side panel
> - Full search integration (flares + halos)
> - Always-on labels Settings toggle
>
> **Top of queue rotates**:
> 1. **MIG-019** — Sight v3 phase 2 (next-up)
> 2. PJ-005 — MIG-007: Links Settings tab
> 3. PJ-002 — cid_cn collision scrub
> 4. PJ-008 — Outgoing Links typed-link dedupe
>
> **Done count after v1.7**: 7 (unchanged — PJ-038 stays In-Progress until all three MIGs close). **Cancelled**: 1 (PJ-034). **Rejected**: 1 (PJ-037).
>
> **No new PJ-NNN allocated** in v1.7.

**Version 1.6 | 2026-05-07**

> **What changed in v1.6** (same day as v1.5; PJ-038 Sight v3 Concept Paper ratified, PJ-037 rejected):
>
> **PJ-038 Concept Paper v1.1** (`docs/Constellation-Sight-v3-Concept-Paper-v1.1.md`) ratified by Eisa 2026-05-07 with all ten v1.0 §11 questions resolved + two structural revisions (faint-lines-at-rest replaces v2's strict reveal-on-demand; Map↔Sight integration rejected).
>
> **Closed in v1.6:**
> - **PJ-037** (Map ↔ Sight integration) — **Rejected** by Eisa 2026-05-07: *"There won't be Map-Sight integration."* Number retired per stable-reference-numbers rule. Sight v3 stays single-view; Map and Sight remain independent surfaces.
>
> **PJ-038 reframed**: PJ-037 absorption removed from §8 trajectory; MIG-020 phase reduced to PJ-036 (layer peeling) + v2 retire only. Three MIGs (MIG-018 / MIG-019 / MIG-020) instead of three-with-bonus-fourth-feature.
>
> **Top of queue stays**:
> 1. **PJ-038 — Sight v3 build with own dedicated Concept Paper** (now ratified at v1.1; ready for MIG-018 Architect).
> 2. PJ-005 — MIG-007: Links Settings tab.
> 3. PJ-002 — cid_cn collision scrub.
> 4. PJ-008 — Outgoing Links typed-link dedupe.
>
> **Done count after v1.6**: 7 (unchanged). **Cancelled count**: 1 (PJ-034). **Rejected count**: 1 (PJ-037 — new).
>
> **No new PJ-NNN allocated** in v1.6.

**Version 1.5 | 2026-05-07**

> **What changed in v1.5** (same day as v1.4; MIG-017 closes — v2 Sight disabled cleanly):
>
> **Closed in v1.5:**
> - **PJ-039** (MIG-017: Disable v2 Sight) — **Done.** Single phase, single commit. `SIGHT_V2_ENABLED = false` const in new `src/lib/sight/engine.ts` module gates four UI surfaces (dock button, modal mount, Return-to-Lens button, Settings plugin entry) + a banner on the Sight help doc. v2 component + IPCs preserved on disk. Three-agent audit clean (0 P0, 0 P1). Audit at `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`.
>
> **Top of queue rotates:**
> 1. **PJ-038 — Sight v3 build with own dedicated Concept Paper** (next-up; multi-MIG, star-chart aesthetic).
> 2. PJ-005 — MIG-007: Links Settings tab.
> 3. PJ-002 — cid_cn collision scrub.
> 4. PJ-008 — Outgoing Links typed-link dedupe.
>
> **Done count after v1.5**: 7 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027, PJ-039). **Cancelled count**: 1 (PJ-034 — partial-shipped).
>
> **No new PJ-NNN allocated** in v1.5. The MIG-017 cycle was Architect → Plan → Build → Audit → Done in one session — the closure is the value, no spin-off backlog.

**Version 1.4 | 2026-05-07**

> **What changed in v1.4** (Boss-directed 2026-05-07; closes the MIG-016 cycle and frames the Sight v3 trajectory):
>
> **Closed in v1.4:**
> - **PJ-034** (MIG-016: Sight instant-toggle perf) — **Cancelled (partial-shipped)**. §1A instrumentation + §1B edges-on-hover gate shipped (commits `a0babbb` → `7e76b17` → `62718f7`). §1C (Web Worker offload), §1D (post-paint prewarm), §1E (SQLite `sight_cache`) **abandoned mid-flight** because v2 Sight is being disabled under MIG-017 (PJ-039) as a known-good fallback while v3 is built fresh. Original goal — "instant first-toggle on a 30k-edge universe" — not met for v2; designed-in for v3 from the start. Audit close-out at `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`.
>
> **New PJs allocated:**
> - **PJ-035** — Sight content-similarity TF-IDF edges (the InfraNodus-defining mechanic; not in v2; **inheritable into v3**).
> - **PJ-036** — Sight layer peeling (hide top-N centrality nodes and recompute; not in v2; **inheritable into v3**).
> - **PJ-037** — Map ↔ Sight integration (cross-surface filtering and selection sync; **inheritable into v3**).
> - **PJ-038** — **Sight v3 build with own dedicated Concept Paper.** Multi-MIG. Star-chart aesthetic per Boss's design north star (Suwaidi northern-hemisphere chart reference). Inherits the Rust analytics from v2; rebuilds the visualization layer entirely.
> - **PJ-039** — **MIG-017: Disable v2 Sight.** Mini-MIG, single session. Hides v2 Sight's user-visible surface (dock button, modal, Settings entry) while preserving the v2 Svelte component and IPCs as a known-good fallback. Precondition for PJ-038.
>
> **Top of queue rotates** (PJ-039 + PJ-038 are the current Sight track; PJ-005 / PJ-002 / PJ-008 carry over from v1.3 as the non-Sight queue):
> 1. PJ-039 (MIG-017) — disable v2 Sight (next-up, mini-MIG)
> 2. PJ-038 — Sight v3 build with own Concept Paper (after MIG-017)
> 3. PJ-005 — MIG-007: Links Settings tab
> 4. PJ-002 — cid_cn collision scrub
> 5. PJ-008 — Outgoing Links typed-link dedupe
>
> **Done count after v1.4**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027). **Cancelled count**: 1 (PJ-034 — partial-shipped).
>
> **New papers landed alongside this version**:
> - `docs/Constellation-Sight-Concept-Paper-v1.1.md` — markdown port of Eisa's April 2026 v1.0 PDF, refreshed with truth-status, Principle 6, and v3 forward-look.
> - `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` — scope-narrowed audit for the partial-shipped MIG-016.

**Version 1.3 | 2026-05-06**

> **What changed in v1.3** (same day as v1.2; deeper cross-check after PJ-006 catch): the v1.2 cross-check agent missed **PJ-006 — Living Link Architecture P2–P5** because my own instructions told it to read only the "What changed in vX.Y" preambles, not orientation BODIES. Orientation v1.40 §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since 2026-05-05. PJ-006 was already done.
>
> **Eisa's response**: codified as **Standing Order #8** in CLAUDE.md (also memory feedback note `feedback_pj_crosscheck_before_tackle.md`): cross-check any PJ before tackling — read orientation BODIES (§4.x subsystem sections) and session logs, not just preambles. Then re-ran the cross-check correctly.
>
> **Outcome of the deeper cross-check** (orientation v1.49 bodies + session logs 2026-05-01 → 2026-05-06):
> - **1 entry flipped to SHIPPED**: PJ-006 (Living Link P2–P5).
> - **All 27 other entries confirmed unchanged** from v1.2 status. No further stale entries.
> - **No new PJ-NNN allocations needed**.
> - **Scope rewrites already correct** in v1.2 (PJ-010, PJ-014, PJ-021 stay as written).
>
> **Top of queue rotates**: PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub) → PJ-008 (Outgoing Links dedupe).
>
> **Done count after v1.3**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027).

**Version 1.2 | 2026-05-06**

> **What changed in v1.2** (same day as v1.1; cross-check audit + v2 batch close): Eisa-directed cross-check of v1.1 against orientation v1.0 → v1.47 to verify which jobs are still applicable.
>
> **Closed in v1.2 cycle:**
> - **PJ-001** (chunk v2 sentinel migration) — SHIPPED via MIG-015 (commits `0ca7e64` → `877e46e`).
> - **PJ-007** (Note-stage taxonomy) — SHIPPED via MIG-014 (commits `c3b9454` → `339d65b`).
>
> **Marked OBSOLETE in v1.2** (already addressed; numbers retired per stable-reference-numbers rule):
> - **PJ-025** (Sight dashboard) — Sight is on-demand via `toggleLens()` in `+layout.svelte:3354`, not boot-rebuilt. Cached after first compute. The "rebuilds on every boot" framing was incorrect.
> - **PJ-026** (sidebar star counts) — `loadAllStats` is cache-fast (Rust per-library parallelism, fire-and-forget per `+layout.svelte:1939-1941`). Not a boot-blocking gap.
> - **PJ-027** (Map) — `src-tauri/src/map.rs:300` documents the Map data is "maintained by triggers on note-save". Already write-time derived.
>
> **Scope rewritten in v1.2:**
> - **PJ-010** (Unlinked Mentions) — narrowed to the frontmatter alias-bleed half (the "double-count typed-link" half was already fixed in v1.5 §90's `scan_unlinked_mentions` rewrite that skips ALL wikilink forms before plain-text scanning).
> - **PJ-014** (13-locale User Manual backfill) — reflected that MIG-014 + MIG-015 shipped all 15 locales upfront for their new strings; remaining queue is the User Manual / help-doc body content (Stages section §18.6, Cognitive Engine help, etc.) that needs translation in the 13 non-en/ar locales.
> - **PJ-021** (Sky View persistence) — narrowed: `sky_backfill.rs` and `cache_boot_snapshot_sky` already provide partial persistence; the gap is whether the full Rule 8 audit (write-time triggers on every note_meta / note_links change) is complete. Verify-then-narrow.
>
> **New PJ-028 → PJ-033** allocated to the MIG-014 §2F audit P2/P3 follow-ups (six edge-case items from `project_mig014_audit_p2_p3_followups.md`).

**Version 1.1 | 2026-05-06**

> **What changed in v1.1** (Boss-directed 2026-05-06): elevates the **Stable Reference Numbers** rule from the Appendix to the front of the doc — `PJ-NNN` IDs are reference numbers used across session logs, commit messages, and cross-doc references; they are never reused, never recycled, and never renumbered, even when a job is rejected, cancelled, merged into another, or split into siblings. Adds **Rejected** and **Cancelled** as explicit terminal statuses (alongside **Done**) that retire a number permanently with the entry preserved. Updates **PJ-007** from `Open · Boss design call` to `Confirmed · In-Progress` with the chosen baseline (Living Link 6-stage) and the proposed-defaults all approved. PJ-007's closure plan is now a focused MIG; PJ-006 P3 is unblocked by this confirmation.

**Version 1.0 | 2026-05-05**

> **What this is.** A durable, versioned project backlog. Every open job that isn't actively shipping right now lives here. The list is reviewed at the start of every work session and updated whenever a job opens, closes, or changes priority. Like the orientation and Laws docs, this file is versioned: a new version (`v1.1`, `v1.2`, …) is written as a NEW file alongside the previous one whenever the structure changes (new job added / existing job moves status / batch of jobs closes). Older versions stay as historical record so the trail of decisions is durable.
>
> **What this is NOT.** It is not the session log. The session log records what *happened today*. This doc records what's *open across the project*. Jobs flow from this doc into a session log entry when work begins; back into this doc as Done when work closes.
>
> **Audience.** Primary: every future Claude session. Secondary: the Boss reviewing what's outstanding. Tertiary: any future contributor.
>
> ### Stable Reference Numbers (foundational rule, v1.1)
>
> Each job has a stable `PJ-NNN` identifier that **acts as its permanent reference number** — like a ticket ID. The rules:
>
> - **Numbers are unique and never repeated.** PJ-007 means PJ-007 forever. If PJ-007 is rejected, cancelled, or merged into another job, **the number PJ-007 is retired with its entry**; no future job ever reuses it.
> - **Renumbering is forbidden.** When jobs close, are rejected, or split, their numbers stay where they are. New jobs always take the **next unused number** (PJ-028, PJ-029, …), regardless of what's happened to earlier ones.
> - **Splitting preserves the parent number with sibling suffixes.** If PJ-006 splits into PJ-006a and PJ-006b, the parent PJ-006 stays in the doc as a header pointing at its children. PJ-006 itself never disappears.
> - **Merging redirects.** If PJ-008 + PJ-009 + PJ-010 merge into one MIG, all three numbers stay in the doc; two of them point at the surviving entry with `merged into PJ-NNN`.
> - **Why this matters.** Session logs cite jobs by number. Commit messages cite jobs by number. The Pending Jobs doc itself cross-references by number (e.g. PJ-006 depends on PJ-007). If numbers got reused, every historical reference would silently break.
>
> ### Status vocabulary
>
> | Status | Meaning | Number behavior |
> |---|---|---|
> | **Open** | Not yet started; queued for future work | active |
> | **In-Progress** | Work has started; tracked in current session log | active |
> | **Confirmed** | Boss has decided the design / scope; ready to start | active |
> | **Blocked** | Cannot proceed until a dependency lands | active |
> | **Deferred** | Decided not to ship now; revisit later | active |
> | **On-hold** | Conditional on a future trigger (e.g. user feedback) | active |
> | **Done** | Shipped; commit hash recorded | retired (number reserved) |
> | **Rejected** | Boss decided not to do this | retired (number reserved) |
> | **Cancelled** | Started but abandoned; entry kept for the record | retired (number reserved) |
> | **Cancelled (partial-shipped)** | Started, some phases shipped, remainder abandoned (added in v1.4) | retired (number reserved) |
> | **Merged** | Folded into another job; the surviving job's number is referenced | retired (number reserved, points to survivor) |
>
> All terminal statuses (Done, Rejected, Cancelled, Merged) move the entry to **§7 Done** but the number stays referenced from its original spot via a one-line stub if useful for navigation.

---

## Quick reference — top of the queue

The first five rows are what's queued to start *next*; the rest are sequenced by priority within their category.

| ID | Job | Status | Severity | Effort |
|---|---|---|---|---|
| PJ-060 | `index_note` cache-hit short-circuit fix (write-time-derivation blocker) | Open | **P1 (highest leverage)** | Mini-MIG |
| PJ-005 | MIG-007: Links Settings tab | Open | P1 | Single MIG |
| PJ-063 | `note_links.link_type` globally `'relates'` (re-verify under MIG-067) | Open | P1 | /migration |
| PJ-002 | Pre-§140 `cid_cn` collision scrub utility | Open | P1 | Mini-MIG |
| PJ-003 | Rename-collision popup (Override / Rename / Cancel) | Open | P1 | Mini-MIG |

---

## §1 · Mini-MIG candidates (focused, 1–3 days each)

### PJ-001 — MIG-013 P1-M1: chunk the v2 sentinel migration with progress UI

**Status.** **SHIPPED 2026-05-06 via MIG-015** · **Severity.** P1 · **Effort.** Mini-MIG (4 phases)

The MIG-013 §1E audit (`lab/reports/MIG-013-CTSE-AUDIT.md` §3) found that the v2 bigram-sentinel migration's bulk UPDATE blocks boot for 30–90 sec on pre-MIG-013 DBs (~5.7M bigram rows) with zero user feedback. Boss's library has already migrated, but new pre-MIG-013 backups would hit it once.

**What shipped (MIG-015 §1A → §1D)**: the migration moved off the boot critical path. `init_db` only detects pending; a worker thread spawned from `ensure_search_db_ready` runs the chunked migration (100,000 rows per chunk) with the DB mutex acquired+dropped per chunk + a 10ms yield between chunks so other IPC callers can interleave. Tauri event channel `migration:term_vocab_v2` emits start/progress/done phases. Frontend `MigrationProgressStrip.svelte` listens and renders a status-bar strip in a new `.sb-center` group: `Migrating term index — N / M`, then `Term index migration complete`, hidden 4 seconds later. i18n covers all 15 locales upfront.

**Acceptance**: ALL met.
- ✅ Boot proceeds to first paint without waiting on the migration.
- ✅ Status-bar strip shows running counts; hides 4 sec after `done`.
- ✅ Crash-recoverable by construction (WHERE clause is the resume marker).
- ✅ 15 locales translated.
- ✅ Three-agent audit clean after one P0 fix (DB mutex held across loop → split per chunk).

**Visual Boss test skipped** per Eisa's directive: Boss's library is already at v2 from earlier MIG-013 testing, and rolling back to manufacture migration work would touch closed-feature production data (Index closed 2026-05-04 per session log; Working Agreement #4 forbids "let's see what happens" on closed-feature data). Static audit verifies behaviour; future users with pre-MIG-013 backups will exercise the visible path naturally.

**Closed-out commit chain**: `0ca7e64` (§1A) → `df0bf87` (§1B) → `62d3b4a` (§1C) → close-out commit (§1D + P0 fix + audit + orientation v1.47).

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §3 (original deferred); `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`; `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md`; `lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md`.

---

### PJ-002 — Pre-§140 `cid_cn` collision scrub utility

**Status.** Open · **Severity.** P1 · **Effort.** Mini-MIG

One-time fix for libraries with corrupted `cid_cn` values from before MIG-003 §140's hardening. Boss self-healed his own affected note (Hub v6) by delete + recreate. Users restoring old backups would still hit the collision silently.

**Source.** Orientation v1.30, session log 2026-05-03.

**Acceptance.** A boot-time scan walks all `note_meta` rows; any duplicate `cid_cn` triggers a status-bar prompt offering "Re-canonicalize duplicates" with a preview list. Run is opt-in, idempotent, and logs every change to a session-scoped report.

---

### PJ-003 — Rename-collision popup (Override / Rename / Cancel)

**Status.** Open · **Severity.** P1 · **Effort.** Mini-MIG

Today `create_note` / `rename_item` silently refuse a rename when the target filename already exists. Boss expects a system-style dialog with three actions: Override the existing file, Rename to something else, or Cancel.

**Source.** `project_rename_collision_popup_wanted.md` (logged 2026-04-28 from MIG-003 Stage 5 test).

**Acceptance.** When `rename_item` hits a collision, frontend shows a `ConfirmDialog`-style popup with three buttons. Override copies properties from the renamed file before deleting the existing one (preserves `cid_cn`). Rename re-prompts for a new name. Cancel dismisses. Localized in en + ar; placeholders in 13 others.

---

### PJ-004 — NSIS bundling lock workaround

**Status.** Open · **Severity.** P2 · **Effort.** Investigation + small fix

Recurring `os error 32` when Constellation is running during `npm run tauri build`. The NSIS bundle stage tries to write `Constellation_X.Y.Z_x64-setup.exe` into a directory that the running binary holds open. MSI succeeds; NSIS doesn't. We work around by using the MSI; a real fix would let CI bundle reliably.

**Source.** Orientation v1.30, hit again on every build during MIG-013.

**Acceptance.** `npm run tauri build` produces both MSI and NSIS bundles cleanly even when an old binary is running. Likely fix: change the NSIS output path or add a kill-running-instance pre-build hook.

---

### PJ-039 — MIG-017: Disable v2 Sight

**Status.** **Done — 2026-05-07** · **Severity.** P1 · **Effort.** Mini-MIG (single session, single commit)

Disabled v2 Sight (`ConstellationSight2.svelte` + the v2 dock button + the v2 modal + the Settings plugin entry) as a **known-good fallback** while v3 is built fresh under PJ-038. The Rust analytics IPCs (`constellation_sight_centrality`, `constellation_sight_communities`, etc.) and the v2 Svelte component are **kept on disk** — they are the proven baseline if v3 fails.

**What shipped:**
- New module `src/lib/sight/engine.ts` exporting `SIGHT_V2_ENABLED = false`.
- Four UI gates added (`SIGHT_V2_ENABLED && ...`):
  - Dock button at `+layout.svelte:4361`.
  - Modal mount at `+layout.svelte:4993-4994`.
  - "Return to Lens" button at `+layout.svelte:4741`.
  - Settings plugin entry at `SettingsModal.svelte:270`.
- Banner block prepended to `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` — original v2 documentation preserved beneath.

**Why a code constant, not a Settings flag**: the fallback is a *codebase* fallback (developer flips one const + rebuilds), not a user-facing toggle. A Settings flag would have required a one-time migration to flip existing users' saved `enabledFeatures.constellationSight: true` to `false`. The const-based gate wins regardless of saved state — zero churn.

**Acceptance**: ALL met.
- ✅ v2 Sight unreachable from the running app's UI in default config.
- ✅ v2 component code + IPCs + Rust analytics modules preserved on disk.
- ✅ One-edit re-enable (`SIGHT_V2_ENABLED = true` + rebuild) restores v2 behaviour identically.

---

### PJ-040 — UA-short-circuit on partial frontmatter discards other catalogers' votes on the unfilled axis

**Status.** **DONE 2026-05-11** in MIG-022 §D commit `c072700` (refactored to per-axis dispatch; both SOURCES and CONTENT TYPE sections now appear on cards with partial frontmatter). Body entry retained as historical record. · **Severity.** P2 · **Effort.** Single PR (~1 commit) · **Filed.** 2026-05-11 (Gate 2 Stage 6 observation)

When the User-Authority cataloger short-circuits the synthesis (because frontmatter has at least one axis populated), `user_authority_short_circuit` in `src-tauri/src/cece/synthesis.rs:120-167` produces an `AxisDecision` for BOTH axes hardcoded to `regime: ConfidenceRegime::Unanimous`, with `primary` taken from UA's per-axis vote. If UA only voiced on horizontal (frontmatter has `sources:` but not `content_type:`), the vertical axis gets `primary: None` and `regime: Unanimous` — vacuously settled. The result: the OTHER catalogers' vertical-axis votes (Linguistic + Structural + Semantic + Graph) are discarded entirely; the suggestion record has no vertical entry; the Source Review card renders with NO CONTENT TYPE section.

**Reproducer.** Boss-test 2026-05-11 Gate 2 Stage 6: re-classify `الخط العربي` (which has `sources: testimony/authoritative` from a previous Stage 2.1 disambig pick but no `content_type:`). The card lands in the queue showing only SOURCES + Authoritative testimony 100% + Accept/Edit/Reject — no vertical suggestion at all, even though Linguistic + Structural + Semantic all voiced on vertical with high-confidence votes (the body has 30K characters of Quranic vocabulary that fires multiple V3-§9.A vertical lexicon entries + V3-§9.B structural detectors).

**Why this is wrong.** UA's authority is per-axis. When the user has set ONLY `sources:` in frontmatter, UA should short-circuit ONLY the horizontal axis; the vertical axis should still be synthesized normally from the other catalogers. Current behavior loses information the catalogers already produced.

**Why this isn't a regression.** The behavior has been the same since V3-§1's `user_authority_short_circuit` shipped. It only became visible after V3-§9.A populated vertical-axis lexicon entries that would now fire meaningfully on Arabic content.

**Proposed fix.** Refactor `user_authority_short_circuit` to short-circuit only the axes UA voiced on. For the unfilled axis, fall through to `vote_on_axis` (the normal weighted-vote path) using the OTHER catalogers' trails. Synthesis method label can become `"user_authority_partial_short_circuit"` when only one axis was UA-voiced, distinct from full-circuit when both axes had frontmatter values.

**Acceptance.** When a card has frontmatter on horizontal only:
- Horizontal axis still short-circuits to UA's pick (existing behavior preserved)
- Vertical axis is synthesized from Linguistic + Structural + Semantic + Graph votes per the normal weighted-vote path
- The card renders with BOTH SOURCES (the UA-pick) AND CONTENT TYPE (the normally-synthesized vertical primary) sections
- Symmetric for vertical-only frontmatter: vertical short-circuits, horizontal synthesizes normally
- Existing tests (`user_authority_short_circuits` etc.) updated to cover both axes
- New regression test: `partial_frontmatter_synthesizes_unfilled_axis_normally`

**Source.** Boss-test 2026-05-11 Gate 2 Stage 6 observation (orientation v1.91 + session log).

---

### PJ-041 — Cataloger reasoning prose hardcoded English in Rust

**Status.** Open · **Severity.** P2 · **Effort.** Single MIG (~3-5 hrs structural + ~90 translations) · **Filed.** 2026-05-11 (Gate 3 Stage 5 observation)

The per-cataloger `reasoning: String` field — rendered in the Source Review trail as one prose sentence per voicing cataloger ("Structural patterns matched: vertical → higher-order-constructs/worldview (weight 0.75)", "Linguistic match: horizontal → revelation/recited (weight 0.85); vertical → ... CAE roots seen: روي, كون, …", "Semantic neighbor consensus: horizontal → testimony/authoritative (weight 0.50); vertical → ...") — is generated by Rust cataloger code at classification time via `format!()` with hardcoded English templates, stored verbatim in `composite_json`, and rendered as-is by the frontend via `{t.reasoning}`. **The string never goes through `$t()`** — V3-§10.D's i18n backfill couldn't translate it because there's no i18n key to translate.

Visible in any Source Review card after switching the interface language to a non-English locale. Boss-test 2026-05-11 Gate 3 Stage 5 surfaced this on an Arabic-UI screenshot showing reasoning prose rendered in English while the chrome around it (toggle text, pill labels, rule chips) was correctly Arabic.

**Files affected:**
- `src-tauri/src/cece/catalogers/structural.rs::build_reasoning()` (line ~298)
- `src-tauri/src/cece/catalogers/linguistic.rs` (the `reasoning` field assignment in `classify()`)
- `src-tauri/src/cece/catalogers/semantic.rs` (similar)
- `src-tauri/src/cece/catalogers/graph.rs` (similar)
- `src-tauri/src/cece/catalogers/user_authority.rs` (similar)
- `src-tauri/src/cece/catalogers/reasoning.rs` (when LLM eventually wired)
- `src-tauri/src/cece/synthesis.rs::user_authority_short_circuit` (`composite_reasoning: format!("Set by user in frontmatter ({}).", ua.reasoning)`)

**Proposed fix.** Refactor each cataloger's `reasoning: String` to emit a structured `(template_key: String, params: HashMap<String, String>)` tuple instead. Frontend renders via `$t(template_key).replace('{...}', value)` at display time. Templates live in `cece.reasoning.*` i18n block (~6 templates per cataloger × 5 voicing catalogers = ~30 templates × 15 locales = ~450 translations). Backward-compat layer: legacy v3-era cards with prose-string `reasoning` fields render the raw string as a fallback.

**Acceptance.** When the user switches to any non-English locale, the per-cataloger reasoning prose in the Source Review trail renders in that locale (or with the locale's translations of the templates the catalogers emit). No regression on en + ar (the reference translations).

**Source.** Boss-test 2026-05-11 Gate 3 Stage 5 (orientation v1.93 + session log). Composes with PJ-042 (related — confidence enum bypasses i18n) and PJ-043 (related — taxonomy node labels en+ar only).

---

### PJ-042 — `self_reported_confidence` enum bypasses i18n

**Status.** Open · **Severity.** P3 (smallest of the three) · **Effort.** Mini-MIG (~30 min + 60 translations) · **Filed.** 2026-05-11 (Gate 3 Stage 5 observation)

The `Confidence` enum in `src-tauri/src/cece/cataloger.rs` serializes as lowercase string (`"high"` / `"medium"` / `"low"` / `"abstain"`) and the Source Review trail renders it raw via `[{t.self_reported_confidence}]` next to each cataloger's name. Visible in every per-cataloger row of every card. No `$t()` lookup.

**Proposed fix.** Add `cece.confidence.{high,medium,low,abstain}` i18n keys (en + ar populated, then 13-locale backfill via Python batch as in V3-§10.D.2). Frontend wraps:

```ts
function confidenceLabel(c: string): string {
  const k = `cece.confidence.${c}`;
  const t = $t(k);
  return t && t !== k ? t : c; // fallback to raw enum on missing key
}
```

Then trail renders `[{confidenceLabel(t.self_reported_confidence)}]`.

**Acceptance.** When the user switches to any non-English locale, the `[high]` / `[medium]` / `[low]` labels render in that locale. Pre-V3-§10 behavior preserved on en + ar.

**Source.** Same as PJ-041 (Gate 3 Stage 5 review). The smallest of the three i18n gaps; could land standalone in ~30 min.

---

### PJ-043 — Taxonomy node labels en+ar only; missing 13 other locales

**Status.** Open · **Severity.** P2 · **Effort.** Single MIG (~3300 translations across 14 locales × ~240 nodes; significant breadth, modest per-translation effort) · **Filed.** 2026-05-11 (Gate 3 Stage 5 observation)

The vertical taxonomy (`vertical_taxonomy.rs::VERTICAL_NODES`, ~225 nodes) and horizontal taxonomy (`horizontal_taxonomy.rs::HORIZONTAL_NODES`, ~30 nodes) static structs have `en: &'static str` and `ar: &'static str` fields only. Frontend's `labelForId()` reads these via the `currentLocale === 'ar'` ternary. For any other locale, the Arabic field is shown (non-Arabic users see Arabic labels) or — depending on the wrapper logic — the English field is shown.

Visible in any Source Review card for non-en/non-ar locales: the SOURCES + CONTENT TYPE list values (Perception/Sensation, Authoritative testimony, Worldview (ruʾyah kawniyyah), etc.) stay in English/Arabic regardless of the active locale. Sibling Disambiguation chips have the same issue (the candidate ID labels render English/Arabic).

**Proposed fix.** Three sub-options (decide in MIG-022 phase):
- **(a) Extend `VerticalNode`/`HorizontalNode` structs** with 13 more `&'static str` fields. Big edit (255 nodes × 13 new fields = 3315 hardcoded strings) but type-checked and zero-runtime-cost.
- **(b) Move taxonomy labels to per-locale JSON files** (`src-tauri/data/vertical_taxonomy.{locale}.json`) loaded lazily. More maintainable but adds runtime indirection.
- **(c) Move to `src/lib/i18n/{locale}.json::cece.taxonomy.*`** (frontend-only). Simplest but loses Rust-side validation.

**Acceptance.** When the user switches to any non-en/non-ar locale, the vertical + horizontal taxonomy node labels render in that locale (or fall back to English with explicit fallback signal).

**Source.** Same as PJ-041 (Gate 3 Stage 5 review). Largest of the three by translation volume; benefits from being scoped as its own MIG.

- ✅ Help doc banner shipped; existing v2 documentation untouched beneath.
- ✅ `npm run check`: 1 pre-existing PJ-012 error, 0 new errors.
- ✅ Three-agent audit clean. Audit report: `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`.

**Architect / Plan / Audit docs**:
- `lab/reports/MIG-017-DISABLE-V2-SIGHT-ARCHITECT.md`
- `lab/reports/MIG-017-DISABLE-V2-SIGHT-PLAN.md`
- `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`

**Source.** Boss decision 2026-05-07 (recorded in `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`).

**Composes with.** PJ-038 — unblocked the moment this commits.

---

## §2 · Larger MIG candidates

### PJ-005 — MIG-007: Links Settings tab

**Status.** Open · **Severity.** P1 · **Effort.** Single MIG

Consolidate every link-related Settings control into one "Links" tab. Currently the controls are scattered: Auto-update Links toggle is misplaced under Sky View & Links (per `project_autoupdatelinks_toggle_placement.md`); the link-confidence backfill button lives in a generic Maintenance section; Living Link lifecycle preferences (when introduced) need a home.

**Source.** `project_links_settings_tab.md`, `project_autoupdatelinks_toggle_placement.md`.

**Acceptance.** A new `Links` tab in Settings that aggregates: Auto-update Links toggle, link-confidence backfill button, Living Link lifecycle decay rate (if exposed), typed-link visibility preferences, link-archival display toggle. Localized in en + ar. Old toggle locations removed without breaking deep-link bookmarks (settings-section anchors stable).

---

### PJ-006 — Living Link Architecture P2–P5 implementation

**Status.** **SHIPPED (closed in v1.3 cross-check, 2026-05-06)** · **Severity.** P1 · **Effort.** Multi-phase (delivered incrementally)

The Living Link Architecture's five implementation phases (P0 through P5) are all live and user-validated. The v1.2 entry's "Open · Multi-MIG" framing was stale — the work was done in slices over the prior weeks (CE Phase commits in the §90-§142 range), and orientation v1.40 §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since 2026-05-05.

**What's actually live (verified 2026-05-06 deeper cross-check):**

| Phase | Verified shipping |
|---|---|
| P0 | `note_links` SQLite table, `extract_typed_links`, 19,062 links indexed |
| P1 | 7 cognitive search operators in 15 languages, chips in SearchHub + Sky View |
| **P2 — Traversal tracking** | `constellation_link_traverse` IPC at `src-tauri/src/search.rs:3516`; frontend caller at `src/lib/libraries/store.ts:1094` (called on wikilink click) |
| **P3 — Weight + lifecycle + decay** | `LinkLifecycle` type in `src/lib/libraries/store.ts:1521`; `_link_decay`, `_link_dormant`, `_link_set_confidence`, `_link_backfill_confidence`, `_link_archive`/`_unarchive`/`_archived` IPCs (orientation v1.49 §4.4 line 1190 enumerates them) |
| **P4 — Formulation analysis** | `formulationAnalysis` wrapper at `src/lib/libraries/store.ts:1505` calling `constellation_formulation_analysis` IPC |
| **P5 — Knowledge health dashboard** | `src/lib/components/KnowledgeHealthDashboard.svelte`, mounted at `src/routes/+layout.svelte:5975`. Reads from P2-P4's data via `formulationAnalysis` |

**Decay formula** (display-only): `effectiveWeight = rawWeight × exp(−ln(2) × daysSinceTraversal / halfLifeDays)`. Default half-life: 60 days. (Orientation v1.49 §4.4.)

**Auto-promote on traversal**: confidence escalates `hypothesis → evidence` at traversal_count ≥3, `evidence → established` at ≥10. Manual override via right-click in Link Dashboard. (Orientation v1.49 §4.4.)

**Why this entry was missed in v1.1 / v1.2**: the v1.2 cross-check agent read only orientation "What changed in vX.Y" preambles per my instructions. Orientation §4.4 BODY says "all shipped"; the preamble of any individual version doesn't necessarily restate that. SO #8 codifies the lesson.

**Source.** `project_ce_philosophy.md`, `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`, orientation v1.40 → v1.49 §4.4 body, `lab/reports/SESSION-LOG-2026-05-*.md`.

**Acceptance.** Already met. Living Link Architecture is the canonical link-side model; PJ-007 (note-side dependency) shipped same-day via MIG-014; the two now share the lifecycle vocabulary cleanly.

---

### PJ-007 — Note-stage taxonomy: Living Link 6-stage baseline + extensible custom stages

**Status.** **SHIPPED 2026-05-06 via MIG-014 (per-note dash-encoded model)** · **Severity.** P1 · **Effort.** Single focused MIG (delivered as a 2-iteration migration: §1A–§1D flat-list iteration record, §2A–§2F shipped model)

**What actually shipped**: the **per-note dash-encoded model** from `Stages-Concept-Paper-v1.2.md` and Plan v4. Iteration 1 (§1A → §1D, flat custom-stage list with per-Universe `custom_stages: Vec<CustomStage>` + 5 IPC commands + emoji picker) was built then proven wrong in Boss test — it didn't scale (long promote chain), the matrix was wrong (Eisa: "It is allowed only one custom term"), and it broke the Single-Source-of-Truth principle (three local mirrors of the stage value drifted across surfaces).

Iteration 2 (§2A → §2F, the model that ships):

- **6 fixed lifecycle stages** form the canonical chain: spark → birth → growth → maturity → dormancy → archival.
- **Per-note custom term** as a dash suffix in the on-disk frontmatter `stage:` value (e.g. `stage: spark-concept`). No Universe-wide setting.
- **PropertyEditor combobox** is a 6-entry mode-flip dropdown: Mode A (input empty / matches a fixed name) → 6 baselines; Mode B (custom word in input or dash suffix) → 6 paired stages (`Spark-Concept`, `Birth-Concept`, …).
- **Breadcrumb promote/demote** walks the 6-baseline chain; suffix carried verbatim across the chain. Single-source-of-truth (Law 2.7) — `currentStage` is `$derived` from the prop, never a local `$state` mirror.
- **No emoji per custom term**: emoji follows the lifecycle phase.
- **Old Zettelkasten values** (`fleeting / literature / permanent / synthesis`) preserved verbatim on disk; render via `LEGACY_ZETTELKASTEN_EMOJI` for back-compat. They aren't promoteable in the new chain.

**Acceptance**: ALL met.
- ✅ Single combobox in Properties (Mode A / Mode B).
- ✅ On-disk frontmatter is the single canonical source — no Universe-level state.
- ✅ Promote/demote chain length always 6.
- ✅ User Manual + Cognitive Engine help updated (en + ar; PJ-014 queues 13 others).
- ✅ Boss tests passed: combobox + per-note scope + cross-track navigation + boundary cases (Spark/Archival).
- ✅ Three-agent audit clean (invariants / drift / migration-path), audit report at `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`.

**Generalisation produced**: Law 2.7 (Constellation Development Laws v1.4) — every first-class data property has one canonical owner; UI surfaces are subfunctions that derive, never hold their own copy. Triggered by the §2C+§2D stage-sync patch cycle (Eisa: "Enough patching").

**Closed-out commit chain**: `c3b9454` (§1A) → `8a9ab3d` (§1B) → `17bf474` (§1C) → `9973e65` (§1C.5) → `f4eef3e` (§1C.5 fix + §1D) [iteration record] → `2f58b8a` (§2A) → `59ed95c` (§2B) → `432076c` (§2C) → `2c58bda` (§2D) → `bb7a6ef` (§2C+D fix) → `e3a97a1` (Law 2.7 architectural fix) → `a50463c` (§2E) → `339d65b` (§2F closes).

**Source.** Stages Concept Paper v1.0 → v1.2; MIG-014 Plan v1 → v4; MIG-014 §2F audit report.

---

### PJ-034 — MIG-016: Sight instant-toggle perf

**Status.** **Cancelled (partial-shipped) — closed 2026-05-07** · **Severity.** P1 · **Effort.** Was scoped as a 6-phase MIG; closed early after §1B.

**What shipped (§1A + §1B):**
- **§1A — `performance.mark` instrumentation** around `toggleLens()` in `+layout.svelte` and `ConstellationSight2.svelte` mount path. Marks: `sight:rust-centrality`, `sight:louvain`, `sight:structural-gaps`, `sight:universe-health`, `sight:stratum-weighted`, `sight:top-bridges`, `sight:community-profiles`, `sight:bridge-suggestions`, `sight:toggle:total`. Initial alerts/clipboard fallback added then removed in §1B; `console.log` + `performance.mark` retained as no-op-in-production. Commits: `a0babbb` (§1A) + `7e76b17` (§1A clipboard fix).
- **§1B — Edges-on-hover gate** (Principle 6 of the Sight Concept Paper v1.1). `needsEdgeDraw = hoveredNode || selectedNode || searchActive || hoveredLink`; `focusOnly` short-circuit at the top of `drawLinks()` skips non-incident edges in O(1) per link. Drops per-frame edge iteration to zero in the resting case (no hover, no selection, no search). Pre-built `neighborMap: Map<string, Set<string>>` populated once per `buildSimData()` call. Commit: `62718f7`. Boss test: PASSED.

**What was abandoned (§1C / §1D / §1E):**
- **§1C — `sightWorker.ts` extraction** (Louvain + gaps + profiles + bridges off main thread): **Cancelled.** Wasted work on a disabled view.
- **§1D — Post-paint prewarm** (`requestIdleCallback` after first paint to cache results before user toggles): **Cancelled.** Same reason.
- **§1E — SQLite `sight_cache`** (cross-session persistence, mirroring the `sky_backfill` pattern): **Deferred to PJ-038.** v3 will compute identical analytical outputs (centrality, communities, gaps, health) and benefit from the same cross-session persistence pattern. The design knowledge from MIG-016 Plan v1 carries forward.

**Why it closes early**: Eisa's directive 2026-05-07 — "secure what's achieved, never muddle." v2 Sight is being disabled under PJ-039 (MIG-017) as a known-good fallback while v3 is built fresh under PJ-038. Continuing perf work on a view that's about to be shelved is wasted effort — except for §1E's design knowledge, which transfers to v3.

**Original goal**: "instant first-toggle on a 30k-edge universe." **Met for v2?** No (the §1A data showed mount is fast at 175-367 ms, but the toggle pipeline's compute is what would have been targeted by §1C/§1D — not addressed). **Designed-in for v3?** Yes — the star-chart aesthetic (Sight Concept Paper v1.1 §13) makes Principle 6 (reveal-on-demand) the visual default, not an add-on.

**Audit close-out**: `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` (scope-narrowed). 0 P0, 0 P1, 1 P3 logged (mousemove handler iterating `simLinks` for link-annotation hover detection; moot once v2 disabled under PJ-039).

**Source.** `lab/reports/MIG-016-ARCHITECT.md`, `lab/reports/MIG-016-PLAN-v1.md`, `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`.

**Inheritance into v3 (PJ-038)**:
- §1B's reveal-on-demand pattern → v3's constellation-line idiom (lines render only inside the focused constellation territory).
- §1E's SQLite cache design → v3's projection-position cache (the projection math is deterministic per-universe-snapshot — caching is a clean win).

---

## §3 · Bug fixes / quality

### PJ-008 — Outgoing Links panel typed-link duplication

**Status.** Open · **Severity.** P2 · **Effort.** Single-file fix

Outgoing Links panel renders typed-link aliases twice — once as a typed-link badge (e.g. `supports`) and once as a plain text row. Same root pattern as PJ-009.

**Source.** `project_outgoing_typedlink_duplication.md`.

**Acceptance.** Each typed-link target appears exactly once in the panel, with its type as a badge.

---

### PJ-009 — Backlinks panel typed-link duplication

**Status.** Open · **Severity.** P2 · **Effort.** Single-file fix

Backlinks panel duplicates source notes when the same source uses both a regular wikilink and a typed wikilink to the same target. Lunch Plan shows twice for Apple Tree Fruit (regular + supports).

**Source.** `project_backlinks_typed_link_duplication.md`.

**Acceptance.** Each source note appears once per target, with all link types accumulated as badges on the row.

---

### PJ-010 — Unlinked Mentions panel: frontmatter alias bleed (scope rewritten in v1.2)

**Status.** Open · **Severity.** P2 · **Effort.** Small refactor

**Scope rewrite — v1.2 (2026-05-06)**: the original v1.1 description bundled two issues — (a) double-counts typed-link references, and (b) canonical filenames instead of human titles. Cross-check against orientation v1.5 §90 confirmed both were fixed by the `scan_unlinked_mentions` rewrite in commit `5cf779a` ("§90 — BUG-005 fix: autosave writeNote bypassed constellation_search_reindex" cycle). That rewrite skips ALL wikilink forms (regular + embed + typed + aliased) before plain-text scanning, AND uses the canonical-filename helper for human-title display. So both v1.1 bullets are stale.

**Remaining genuine gap**: frontmatter aliases (e.g. `aliases: [Foo, Bar]`) still surface as "unlinked mentions" because the alias-bleed fix never landed. A note with `aliases: [Bar]` in its frontmatter and a body that says "Bar" appears in the Unlinked Mentions panel as a separate row, even though the alias on the SAME note's frontmatter is what's matching. Memory note `project_unlinked_mentions_alias_bleed.md` (2026-04-29) describes the case.

**Source.** `project_unlinked_mentions_alias_bleed.md`.

**Acceptance.** A note's frontmatter aliases do NOT surface as "unlinked mentions" against the same note's body (the body word should be parsed as a self-alias-match and excluded). All other panel behaviour stays as v1.5 §90 made it.

---

### PJ-011 — Constellation Map open issues

**Status.** **DORMANT (2026-06-09)** — Constellation Map was disabled in core by MIG-038 (re-categorized as a future external "Wings" plug-in). Not actionable while Map is off; re-activate this PJ if/when Map returns. · **Severity.** P2 · **Effort.** Single MIG

Three issues bundled (logged 2026-04-27):

- Performance / memory leak in the D3 sunburst rendering on large libraries.
- Tooltip shows canonical filename instead of human title (same root as PJ-010).
- Search doesn't highlight matched arcs.

**Source.** `project_constellation_map_backlog.md`.

**Acceptance.** Map renders cleanly on 7,600-note libraries (no leak across navigation). Tooltips show human titles. Search highlights matching arcs with a visible style.

---

### PJ-012 — `LinkLifecycle.fresh` TS error

**Status.** Deferred · **Severity.** P2 · **Effort.** 2-line fix

Pre-existing svelte-check error at `store.ts:2212`: `Property 'fresh' is missing in type '{emerging, established, load-bearing, stale}'`. Option B approved 2026-05-01 but deferred until post-CE: add `fresh: 1`, shift `stale: 0`, `fresh: 1`, `emerging: 2`, `established: 3`, `load-bearing: 4`. Runtime impact silent; this is a type-completeness fix.

**Source.** `project_link_lifecycle_dedupe_fix.md`.

**Acceptance.** `npm run check` produces zero errors (currently shows this one + warnings).

**Composes with.** PJ-006 P3 — LinkLifecycle taxonomy is the same vocab the Living Link lifecycle uses; fix it as part of P3 if not done sooner.

---

### PJ-013 — `lenses::apply_lens` dead-code decision

**Status.** Open · **Severity.** P2 · **Effort.** Decision + small fix

`lenses.rs::apply_lens` is dead code (zero frontend callers, verified 2026-04-27). Settings can still create + save lens definitions but they're never applied. Two paths: delete the function + the orphaned Settings UI, or re-wire it for CE Phase 9 (whatever that turns out to be).

**Source.** `project_lenses_apply_lens_dead_code.md`.

**Acceptance.** Either deleted (along with `list_lenses` / `save_lenses` if those are also unused) and the Settings lens-builder UI removed, or re-wired into a frontend consumer that exercises it.

**Composes with.** PJ-039 (MIG-017): may resolve as part of disabling v2 Sight if the orphaned Settings UI is removed at the same time.

---

## §4 · Doc backlog

### PJ-014 — 13-locale User Manual backfill (scope updated in v1.2)

**Status.** Open · **Severity.** P2 · **Effort.** Translation work

**Scope update — v1.2 (2026-05-06)**: MIG-014 + MIG-015 broke the "en+ar first, others queued" pattern by shipping all 15 locales upfront for their *new* string keys (Eisa-directed). So the i18n .json files don't lag for those two MIGs.

**Remaining queue is the User Manual / help-doc body content** that DOES lag in 13 locales:

- **MIG-014 §2E**: Stages model rewrite in `docs/User Manual.md` §18.6 (en) + `docs/help.ar/User Manual.md` §18.6 (ar) + `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` Feature 6 (en). 13 other locale User Manuals still carry the old Externalization-Engine 4-stage description.
- **Older deferrals** still queued: MIGs 008, 010, 011, 012, 013 deferred sections.

**Source.** `project_user_manual_13_locales_backfill.md`.

**Acceptance.** All 13 locale User Manuals receive the deferred sections (Stages model + earlier-MIG deferrals). Done as one batch translation pass; can be split per locale if Boss has translator capacity for a few at a time.

---

### PJ-015 — 360.3D Stratification Matrix guidance doc

**Status.** **ABANDONED 2026-05-18** (Eisa decision during post-MIG-026 state-of-standing triage) · **Severity.** P2 · **Effort.** Single doc (~2000 words)

Originally a Boss-requested teaching doc on how to read / interpret the 360.3D Stratification Matrix (Three reads — Position / Profile / Absence, mental shapes catalogue, matrix → action examples). Blocked indefinitely on 360.3D Stage 3 closure + matrix UX stabilization.

**Why abandoned**: Eisa's call 2026-05-18 — the matrix-UX dependency hasn't moved, the doc is low-leverage given other priorities, and 360.3D itself may not need a separate guidance doc. If a user-facing need surfaces later it can be filed fresh.

**Source.** `project_360_3d_matrix_guidance_doc.md` (memory note retained as historical record).

---

## §5 · Cleanup / hygiene

### PJ-016 — Drop `term_vocab.bridge_concept_id` column

**Status.** **DONE via MIG-042** (orientation v2.25 — "drop the dead `term_vocab.bridge_concept_id` column"; +2 lurking bugs found & fixed during testing). Moved to §9. · **Severity.** P2 · **Effort.** Schema migration

Dead schema after MIG-013 §1D Option B. Nothing reads or writes the column on the live CTSE path. Forward-compat preserved deliberately, but a future cleanup migration can drop it (along with the v1 / v2 schema gates and the `sentinel_bigram_rows` helper).

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2.

**Acceptance.** Schema v3 migration drops the column, the index, and all the dead helper code. M11 zero-diff invariant still holds.

**Defer.** Wait at least 2–3 sessions after MIG-013 close to confirm nothing reactivates the column.

---

### PJ-017 — Drop orphaned `term_embeddings` table on existing DBs

**Status.** Open · **Severity.** P2 · **Effort.** Schema migration

Leftover from MIG-012 on pre-MIG-013 universes. Tens of MB of dead disk per Universe. No correctness issue.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §3 Notes.

**Acceptance.** A schema migration runs `DROP TABLE IF EXISTS term_embeddings`. Bundle with PJ-016 into a single cleanup migration.

---

### PJ-018 — Drop `index.semanticSearchEnabled` settings flag

**Status.** Open · **Severity.** P2 · **Effort.** 2-line fix

Kept for forward-compat after §1D-B; zero readers in `src/`. Bundle with the rest of the MIG-013 cleanup.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2 Notes.

**Acceptance.** Removed from `DEFAULT_SETTINGS` and the `IndexSettings` interface in `store.ts`.

---

### PJ-019 — Drop `searchHub.concept` / `searchBadges.concept` i18n keys

**Status.** Open · **Severity.** P2 · **Effort.** 2-key delete × 15 locales

Kept after the SearchHub `concept` category was reverted in §1D-D; zero callers.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2 Notes.

**Acceptance.** Both keys removed from all 15 locale JSONs. Bundle with PJ-016 / PJ-017 / PJ-018 into one MIG-013 cleanup commit.

---

### PJ-020 — Optional `≈ similar` kill-switch

**Status.** On-hold · **Severity.** P3 · **Effort.** Settings toggle + gating

The CTSE `≈ similar` feature is currently always-on with no Settings toggle. Add only if Boss reports noise (irrelevant terms surfacing in the Index dropdown).

**Source.** MIG-013 close-out note.

**Acceptance.** Add a toggle to Settings → Index ("Cross-language `≈ similar` matches"). Default ON. When OFF, the `ctseSearchTermsByConcept` effect short-circuits.

**On-hold.** No action until Boss reports the feature is producing noise.

---

## §6 · Standing-Order audit — Write-Time Derivation (CLAUDE.md Rule 8)

CLAUDE.md Rule 8 ("Every computed view in Constellation is maintained at write time, not read time") explicitly names these surfaces as needing audit. Each currently rebuilds at boot or on tab focus instead of being maintained at write-time via triggers/hooks. Each is its own focused MIG.

### PJ-021 — Sky View (`skyNodes` / `skyLinks`) — scope updated in v1.2

**Status.** Open (verify-then-narrow) · **Severity.** P2 · **Effort.** Verify + targeted MIG

**Scope update — v1.2 (2026-05-06)**: cross-check found that `src-tauri/src/sky_backfill.rs` and `cache.rs::cache_boot_snapshot_sky` already provide partial persistence of `sky_nodes` / `sky_links` (since MIG-001 §136 / §142 timeframe). The original v1.1 description ("rebuilt on every boot") is partly outdated.

**Verify-then-narrow plan**:
1. Open `src-tauri/src/sky_backfill.rs` + `cache.rs::cache_boot_snapshot_sky` and confirm what's already persisted vs. what still rebuilds.
2. If full Rule 8 (write-time triggers on every `note_meta` / `note_links` change) is in place: close as Done.
3. If partial: narrow this PJ to "the gap that remains" (likely a specific trigger that's missing or an edge-case where the cache invalidates wholesale instead of incrementally).

**Acceptance.** Either confirmed Rule 8-clean (close) or narrowed to a specific bounded gap with new acceptance criteria.

---

### PJ-022 — Backlinks panel

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Currently recomputed on tab focus by walking `note_links` per-target. Should be a cached table or a materialized view.

**Acceptance.** Tab focus shows backlinks instantly even on 100-link nodes.

---

### PJ-023 — Outgoing Links panel

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Currently recomputed on tab focus. Same shape as PJ-022; pair into one MIG.

**Acceptance.** Same as PJ-022.

---

### PJ-024 — Tag browser

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Scanned on open. Should read from a maintained tag→notes index.

**Acceptance.** Tag browser opens instantly on libraries of any size.

---

### PJ-025 — Sight dashboard (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: Sight is invoked from the frontend's `toggleLens()` handler (`+layout.svelte:3354` calls `constellation_sight_centrality`). It runs on-demand when the user toggles the Sight overlay; results are cached and reused while fresh. Boot path does NOT invoke Sight. The "rebuilds on every boot" framing in the original PJ description was incorrect — there was no boot-time Sight rebuild to migrate away from.

The PJ-025 number is retired per the stable-reference-numbers rule. No further action.

---

### PJ-026 — Sidebar star counts (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: `loadAllStats` walks the per-library star count at boot, but per `+layout.svelte:1939-1941` comment: *"loadAllStats remains because its Rust side is already cache-fast (metadata-only walk + per-library thread parallelism). It's fire-and-forget so the sidebar star counts populate without blocking anything."* This is a cached-fast path, not a Rule-8 violation. Not boot-blocking, not full filesystem walk. Original PJ-026 framing was incorrect.

PJ-026 number retired.

---

### PJ-027 — Map (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: `src-tauri/src/map.rs:300` documents that the Map data is *"maintained by triggers on note-save, so even an explicit open is..."* — already write-time derived per Rule 8. No Rebuild on open / boot.

(PJ-011's separate Map open issues — perf/leak, tooltip showing canonical filename, search-highlight — remain open as P2; those are panel UX bugs unrelated to the persistence question.)

PJ-027 number retired.

---

## §7 · MIG-014 §2F audit follow-ups (PJ-028 → PJ-033 — carried from v1.2)

Six edge-case items found by the MIG-014 §2F three-agent audit (2026-05-06) but logged for later, not blocking close. Memory note: `project_mig014_audit_p2_p3_followups.md`. All non-blocking — graceful degradation in every case. P2/P3 severity.

### PJ-028 — `splitStage` and a leading dash

**Status.** Open · **Severity.** P2 · **Effort.** 2-line fix

`stage: -concept` (no lifecycle prefix) splits to `lifecycle=''`, `suffix='concept'`. Renders as `-Concept` with empty emoji and no promote/demote arrows.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 1; MIG-014 §2F audit.

**Acceptance.** `splitStage` treats empty lifecycle as "no stage" — returns both empty when no valid lifecycle prefix is present. Edge-case display normalizes cleanly.

---

### PJ-029 — Concept Paper §6.1 vs `commitStage` multi-dash drift

**Status.** Open · **Severity.** P2 · **Effort.** Decision + 2-line code fix OR doc fix

Stages Concept Paper v1.2 §6.1 says suffix may not contain `-`. `commitStage` at `PropertyEditor.svelte:199` doesn't enforce. Multi-dash values like `stage: spark-foo-bar` are accepted. Either tighten code (reject) or update doc (allow). Doc-vs-code drift; not a runtime bug.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 2.

**Acceptance.** Doc and code agree on whether multi-dash suffixes are allowed. Whichever direction, the surviving rule is enforced by tests.

---

### PJ-030 — Stale `custom_stages: [...]` from §1A-era testing in `universe.json`

**Status.** Deferred · **Severity.** P3 · **Effort.** None (acceptable)

Serde silently ignores the unknown field; gone on next read-modify-write cycle of `universe.json`. Affects only Boss-equivalent dev-build users (none reported). Acceptable graceful self-healing.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 3.

**Acceptance.** Self-heals on next universe-meta write. No active fix needed unless surfaces as a bug.

---

### PJ-031 — Trailing-dash on disk (`stage: spark-`)

**Status.** Deferred · **Severity.** P3 · **Effort.** None (acceptable)

`splitStage` returns `suffix=''`; nextStage carries no suffix. Display correct. The trailing dash stays on disk verbatim until the user re-commits via promote. Acceptable graceful self-healing.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 4.

**Acceptance.** Self-heals on next promote/demote. No active fix needed.

---

### PJ-032 — Uppercase on disk (`stage: SPARK-CONCEPT`)

**Status.** Deferred · **Severity.** P3 · **Effort.** Optional 2-line fix

`LIVING_LINK_BASELINE.findIndex` is case-sensitive → returns -1. No emoji, no arrows. Display falls back to verbatim render. User must re-pick to recover. Could be normalized in `splitStage` (lowercase the lifecycle component) but the current behavior is acceptable graceful degradation.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 5.

**Acceptance.** Either lowercase normalization in `splitStage`, or status quo if Boss prefers strict canonical form.

---

### PJ-033 — NotePane stage badge `<span>` has no `dir="auto"`

**Status.** Open · **Severity.** P3 · **Effort.** 1-line fix

`src/lib/components/NotePane.svelte:951`. A long Arabic suffix in an LTR UI may render slightly off due to Chromium's bidi defaults. Easy polish — add `dir="auto"` to the badge span.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 6.

**Acceptance.** `dir="auto"` on the stage-badge `<span>`. Mixed-script suffixes render with proper directionality in any UI direction.

---

## §8 · Sight v3 trajectory (NEW in v1.4 — PJ-035 → PJ-038)

The Sight Concept Paper v1.1 (`docs/Constellation-Sight-Concept-Paper-v1.1.md`) §12 truth-status matrix surfaced three implementation gaps in v2 Sight relative to the paper's analytical promise. Each is now an inheritable PJ.

### PJ-035 — Sight content-similarity TF-IDF edges

**Status.** **DONE (within MIG-019 §2B)** in commit `16063735` "MIG-019 §2B — Milky Way density wash (PJ-035) + Settings toggle". The PJ-035 mechanic shipped under MIG-019's Milky Way band per the v3 inheritance promise. v1.11/v1.12 preamble never updated to reflect this; corrected in v1.12 §μ state-of-standing audit (2026-05-18). · **Severity.** P2 · **Effort.** Multi-step (vector compute + cache + integration)

**The InfraNodus-defining mechanic.** v2 Sight wires explicit-wikilink edges (weight 1.0) and shared-tag edges (weight 0.6) into its graph build. The third edge type from the Concept Paper §3.3 — **content similarity (weight 0.3, TF-IDF cosine)** — is not implemented. This is the mechanic that lets Sight surface *latent* connections — notes that talk about the same topic without being explicitly linked. Without it, Sight cannot detect structural gaps that span un-linked-but-related clusters.

**Source.** `docs/Constellation-Sight-Concept-Paper-v1.1.md` §3.3 + §12 truth-status row.

**Acceptance.** TF-IDF vectors computed per note (incremental on changed notes only, cached). Cosine similarity computed lazily between notes above a configurable threshold. Edges merged into the Sight graph build with weight 0.3 (toggleable in Settings). For non-English content, configurable per-language stemmer pipeline (Arabic via ISRIStemmer or equivalent). Graceful degradation if NLP for a given language isn't loaded — tag/link edges still work.

**Inheritable into v3 (PJ-038)**: in v3's star-chart aesthetic, content-similarity edges become the **Milky Way band** (a diffuse density wash, not extra edge lines competing with constellation connectors). The compute layer is identical; only the visualization changes.

---

### PJ-036 — Sight layer peeling

**Status.** **ABANDONED 2026-05-18** (Eisa decision during post-MIG-026 state-of-standing triage — Sight v6's facet sidebar substitutes for the layer-peeling need; the v2 Concept Paper §2.2 mechanic 5 is no longer relevant under the v6 architecture). · **Severity.** P3 · **Effort.** Single feature (compute + UI toggle)

Originally proposed in `docs/Constellation-Sight-Concept-Paper-v1.1.md` §2.2 as the "remove the obvious to reveal the subtle" mechanic — hide top-N centrality nodes (typically MOC / index notes) and recompute analytics on the residual graph. Intended to be inheritable into v3 as a "hide brightest stars" toggle.

**Why abandoned**: Sight v6 ships a facet sidebar (`facetSidebar.svelte`) with 6 facet groups (Folder / Library / Stratum / Confidence / Stage / Provenance), each filtering to a subset of stars. The user can already isolate secondary structure by negative-selecting the dominant nodes' facets — the same diagnostic Outcome layer-peeling promised, delivered through a different UI metaphor that fits v6's tradition-aware grammar.

---

### PJ-037 — Map ↔ Sight integration

**Status.** **Rejected — 2026-05-07** · **Severity.** P2 (was) · **Effort.** Single MIG (was)

**Rejected by Eisa 2026-05-07** during the v3 Concept Paper review: *"There won't be Map-Sight integration."*

The two surfaces remain independent. Map is the "shape" view (radial sunburst, organizational hierarchy). Sight is the "patterns" view (star chart, conceptual relationships). Each lives in its own dock and is opened separately. The "Map diagnoses, Sight prescribes" loop — framed in the Sight Concept Paper v1.1 §7 — happens in the user's head, not in a shared cursor.

**Number retired** per the stable-reference-numbers rule. Entry preserved here as historical record.

**If revisited later**: a future PJ would need a fresh number (not PJ-037). Cross-references in commits or session logs to PJ-037 should be read as "the rejected Map↔Sight integration concept," not as a live job.

**Source (historical).** `docs/Constellation-Sight-Concept-Paper-v1.1.md` §7; v3 Concept Paper v1.0 §5.3 (the absorption proposal Eisa rejected).

---

### PJ-038 — Sight v3 build with own dedicated Concept Paper

**Status.** **SUPERSEDED (by Sight v6 / MIG-024 → MIG-027)** — marked at v1.12 §μ state-of-standing audit (2026-05-18). The Sight v3 trajectory was abandoned mid-flight (commit `29ce0101` "MIG-019 §0 — Sight v3 → v4 clean-slate pivot") in favor of the v4 redesign, which itself superseded into v5 then v6 (the radial-anchor + mini-domes + tradition-chip architecture that just shipped under MIG-026). `SIGHT_V3_ENABLED = false` in `engine.ts`; `SIGHT_V6_ENABLED = true`. PJ-052 + PJ-053 (Sight v6 follow-ups) closed under MIG-026. The 3-MIG decomposition described in this entry's body (MIG-018 / MIG-019 / MIG-020) is no longer the live trajectory. · **Severity.** P1 (was) · **Effort.** Multi-MIG (was)

**Phase 1 of 3 — MIG-018 closed Done (2026-05-07)**: projection foundation. Star-chart visualization with graph-distance Landmark-MDS embedding (Rust `sight_layout::compute_layout_embedding`), Lambert + stereographic projections (user-toggle in Settings → Sight), constellation territories drawn as Suwaidi warm-cream + gold soft-fill polygons (cycled by Louvain community id), faint connector lines at rest (Eisa's design call: visible structure without dominating attention), hover/click/double-click interactivity with side panel, RTL-aware tooltip + side panel. Six-phase cascade (§1A schema → §1B Landmark-MDS compute → §1C frontend skeleton → §1D star rendering + projection toggle → §1E territories + lines + interactivity [Boss-test gate] → §1F audit + close-out). 5 unit tests passing. Three-agent audit CLEAN (0 P0/P1/P2/P3). `SIGHT_V3_ENABLED = true` committed. Full record: `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-{ARCHITECT,PLAN,AUDIT}.md`.

**Phase 2 of 3 — MIG-019 (next-up)**: density + time + search + universe-health.

**Decision** (Eisa, 2026-05-07): rebuild Sight from scratch on the **star-chart aesthetic** — a 2D polar projection where star magnitude maps to centrality, constellation territories map to Louvain communities, the Milky Way band maps to content-similarity density, and a calendar rim maps to time. Reference image: 19th-century-style northern-hemisphere chart (Suwaidi reference; sample owned by the Boss). This is the design north star articulated in `docs/Constellation-Sight-Concept-Paper-v1.1.md` §13.

**What v3 inherits from v2** (the analytical pipeline is preserved as-is):
- Brandes' betweenness centrality (`constellation_sight_centrality` IPC).
- Louvain community detection (`constellation_sight_communities` IPC).
- Structural gap detection (`constellation_sight_structural_gaps` IPC).
- Universe-health metric (`constellation_sight_universe_health` IPC: M + D + E + C).
- Reveal-on-demand (Principle 6) — already shipped in v2's MIG-016 §1B.

**What v3 absorbs (the deferred PJs)**:
- **PJ-035** content-similarity edges → Milky Way band (diffuse density wash).
- **PJ-036** layer peeling → "hide brightest stars" toggle (drag right per astronomy convention).
- ~~PJ-037 Map ↔ Sight integration~~ — **Rejected** 2026-05-07; not part of v3.

**What v3 rebuilds entirely** (the visualization layer):
- Force-directed Pixi.js simulation → 2D polar projection (likely Lambert azimuthal equal-area or similar; specified in v3's own Concept Paper).
- D3-style force layout → stable astronomy-style projection math.
- Edge-render hot path → constellation-line idiom (lines render only inside the focused constellation territory).

**Mandatory deliverable: own dedicated Concept Paper** (Boss directive 2026-05-07). The v3 paper is the canonical reference for *what v3 looks like and how it is built*. The v1.1 paper continues as the canonical reference for *what Sight is for* (the analytical foundations both versions share). They are read side-by-side.

**Source.** Boss decision 2026-05-07; Sight Concept Paper v1.1 §13 + §14; MIG-016 audit §6 ("inheritance into v3").

**Acceptance.** v3 ships behind feature flag (`sight.engine: 'v3'`), default ON in production once Boss-test passes. Star-chart projection renders the user's full universe at one glance. Constellation territories visually distinct. Reveal-on-demand for connector lines (Principle 6 baked into the visual grammar). Universe-health metric readable in one glance from dome-balance. Three deferred PJs (PJ-035 / PJ-036 / PJ-037) integrated as the design intends. Three-agent audit clean. Own Concept Paper delivered alongside the build.

**Composes with**:
- PJ-039 (precondition: v2 disabled before v3 starts).
- PJ-035 / PJ-036 / PJ-037 (absorbed as v3 features rather than v2 add-ons).
- PJ-013 (`apply_lens` dead code may be cleaned up as part of the v3 cutover).

---

## Newly filed since v1.12 (PJ-059 → PJ-064)

Six numbers allocated 2026-05-19 → 2026-06-09. PJ-059 / PJ-060 were filed in the 2026-05-19 session log; **PJ-061 / PJ-062 give proper, never-before-used identities to two fixes the 2026-05-29 log informally (and incorrectly) called "PJ-10 / PJ-11"** — colliding with the canonical PJ-010 / PJ-011, which are untouched; PJ-063 / PJ-064 are promoted here from memory notes.

### PJ-059 — Sight per-note search/finder

**Status.** Open — **dormant** (Sight disabled by MIG-038) · **Severity.** P3 · **Effort.** Single-file feature

A per-note search/finder inside the Sight dome, scoped to the Sight v7 close-out. Filed 2026-05-19. Dormant while Sight is a disabled / "Wings" surface; re-activate alongside Sight.

**Source.** SESSION-LOG-2026-05-19 (Sight delivery cascade, carried PJs).

---

### PJ-060 — `index_note` cache-hit short-circuit fix

**Status.** Open · **Severity.** **P1 — highest-leverage open fix** · **Effort.** Mini-MIG

`index_note`'s cache-hit short-circuit (`search.rs:3004`) returns early when the file mtime matches, so the write-time refresh of `note_meta` never fires on an unchanged-mtime re-save. Traced 2026-05-19 as the root cause blocking the MIG-029 write path; flagged as "the single most-leveraged open fix." A focused mini-MIG: correct the short-circuit so write-time derivation always runs.

**Source.** SESSION-LOG-2026-05-19 (MIG-029 → MIG-036 pivot, root-cause section).

**Acceptance.** A re-save with unchanged mtime still refreshes `note_meta` + every write-time-derived surface; no boot / typing regression on the 7,600-note universe.

---

### PJ-061 — Sky View federated node sizing  *(DONE)*

**Status.** **DONE 2026-05-29** · **Severity.** P2 · **Effort.** Tuning

Once MIG-061 made Sky View show the full 8,751-node federation, nodes rendered too large. Fixed over 3 rounds: count-aware fill damping + halving the fixed-pixel decorative frames (stratum / provenance / maturity / MOC rings) in dense (>1500-node) mode → ~0.12× at 8,751. Boss-verified. *(Filed unnumbered 2026-05-28 as "PJ-NNN-A"; the 2026-05-29 log called it "PJ-10" — a collision with the real PJ-010; numbered properly here.)*

**Commit chain.** `9a2d9890` → `62a9a198` → `f05fe6f9`. See §9.

---

### PJ-062 — CNS gravity-well canvas fill  *(DONE)*

**Status.** **DONE 2026-05-29** · **Severity.** P2 · **Effort.** Tuning

The CNS (Constellation Nervous System) gravity well left big margins on wide monitors (`maxR = min(w,h)×0.45`). Bumped to `×0.58` + fitToScreen zoom `0.85→0.93`; stayed circular (Form-Aligns-To-Purpose: radius = centrality, angle = library). Boss-verified first try. *(Filed unnumbered 2026-05-28 as "PJ-NNN-B"; the 2026-05-29 log called it "PJ-11" — a collision with the real PJ-011; numbered properly here.)*

**Commit.** `9a2d9890`. See §9.

---

### PJ-063 — `note_links.link_type` globally `'relates'`

**Status.** Open · **Severity.** P1 · **Effort.** /migration (foundational)

`note_links.link_type` is globally `'relates'`: the backend `extract_typed_links` (`search.rs:3614`) parsed `[[type::target]]` while notes + editor stored the type as the last segment of `[[target|display|type]]`. Exposed by the MIG-066 Link-types column. Affects every `link_type` consumer (Base columns, Sky View edge colours, typed-link panels). **Re-verify against the post-MIG-067 state** — MIG-067 shipped the Link-Type Registry + predicate-first `[[type::target]]` authoring after this was first observed, which may have shifted or partly resolved it.

**Source.** Memory `project_note_links_link_type_relates_bug`; MIG-066 / MIG-067 (orientation v2.46 / v2.47).

**Acceptance.** Every note's `note_links.link_type` reflects the authored type across all consumers; both legacy `[[target|…|type]]` and predicate-first `[[type::target]]` resolve correctly.

---

### PJ-064 — Style Setter: more font types in the final version

**Status.** Open · **Severity.** P3 · **Effort.** Single-component feature

The Style Setter's font pickers currently expose System / Serif / Mono as placeholders; the final version should offer the full font catalogue. (The other half of this request — named / saved colour swatches — already shipped in MIG-070.)

**Source.** Memory `project_style_setter_feature_requests` (2026-06-02).

---

## §9 · Done

Items move here from the categories above when they close. Format preserved per stable-reference-numbers rule: the original entry stays in its source section with its closure status; this section provides a quick chronological index.

| PJ-NNN | Title | Status | Closed | Commit chain |
|---|---|---|---|---|
| PJ-001 | Chunk the v2 sentinel migration with progress UI | Done | 2026-05-06 (via MIG-015) | `0ca7e64` → `df0bf87` → `62d3b4a` → `877e46e` |
| PJ-006 | Living Link Architecture P2–P5 (all phases verified shipped & user-validated) | Done | 2026-05-06 (closed in v1.3 cross-check; shipped earlier in CE Phase §90-§142 commit range) | (multi-commit; orientation §4.4 line 1167+ for canonical state) |
| PJ-007 | Note-stage taxonomy: per-note dash-encoded model | Done | 2026-05-06 (via MIG-014) | `c3b9454` → `8a9ab3d` → `17bf474` → `9973e65` → `f4eef3e` → `2f58b8a` → `59ed95c` → `432076c` → `2c58bda` → `bb7a6ef` → `e3a97a1` → `a50463c` → `339d65b` |
| PJ-025 | Sight dashboard (Obsolete — verified write-time-derived) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-026 | Sidebar star counts (Obsolete — verified cache-fast) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-027 | Map (Obsolete — verified trigger-maintained) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-034 | MIG-016: Sight instant-toggle perf | **Cancelled (partial-shipped)** | 2026-05-07 (§1A + §1B shipped; §1C/§1D cancelled; §1E deferred to PJ-038) | `a0babbb` → `7e76b17` → `62718f7` |
| PJ-039 | MIG-017: Disable v2 Sight (precondition for v3) | Done | 2026-05-07 (single mini-MIG) | `8c4019c` |
| PJ-037 | Map ↔ Sight integration | **Rejected** | 2026-05-07 (v3 Concept Paper review — Eisa rejected the integration concept) | n/a |
| PJ-015 | 360.3D Stratification Matrix guidance doc | **Abandoned** | 2026-05-18 (state-of-standing triage; low-leverage) | n/a |
| PJ-035 | Sight content-similarity TF-IDF edges → Milky Way band | Done | 2026-05 (via MIG-019 §2B) | `16063735` |
| PJ-036 | Sight layer peeling | **Abandoned** | 2026-05-18 (v6 facet sidebar substitutes for the mechanic) | n/a |
| PJ-038 | Sight v3 build (3-MIG trajectory) | **Superseded** | by Sight v6 (MIG-024→027); v3 abandoned `29ce0101` | n/a |
| PJ-040 | UA short-circuit on partial frontmatter | Done | 2026-05 (via MIG-022 §D) | `c072700` |
| PJ-052 | Sight Concept Paper v4.1 | Done | 2026-05-18 | (docs) |
| PJ-053 | λ-fix-6 native-quality translation polish (192 keys, 7 locales) | Done | 2026-05-18 | (i18n) |
| PJ-054 | Sight v6 vitest test runner (58/58 pass) | Done | 2026-05-19 (via MIG-030) | `f327d758` |
| PJ-055 | User-plugin label schema warning | Done | 2026-05-18 | `e63ee0c7` |
| PJ-056 | MIG-026 drift cleanup | Done | 2026-05-18 | `e63ee0c7` |
| PJ-057 | Post-MIG-026 doc-drift (a/c done; b deferred) | Done (partial) | 2026-05-19 (via MIG-032) | `f327d758` |
| PJ-058 | Constellation Sight Subsystem Concept Paper v1.0 | Done | 2026-05-19 | (docs) |
| PJ-016 | Drop `term_vocab.bridge_concept_id` column | Done | via MIG-042 | (orientation v2.25) |
| PJ-061 | Sky View federated node sizing | Done | 2026-05-29 | `9a2d9890` → `62a9a198` → `f05fe6f9` |
| PJ-062 | CNS gravity-well canvas fill | Done | 2026-05-29 | `9a2d9890` |

---

## Appendix — How to amend this document

1. **Adding a job.** Append to the appropriate `§N` section with the **next unused** `PJ-NNN` ID — never reuse a number from a Done / Rejected / Cancelled / Merged entry. Bump the version. Commit + push as a new file (`v1.5.md` etc.).
2. **Updating a job.** Edit the existing entry (status, severity, source, acceptance). Bump the version if the change is structural (status transition, scope change); same version if just refining wording.
3. **Closing a job.** Move the entry to `§9 Done` keeping its `PJ-NNN`, strikethrough the title, append closing date and commit hash. Bump the version. The number is retired with the entry — never recycled.
4. **Rejecting / cancelling a job.** Same shape as closing — move to `§9 Done` with status `Rejected` or `Cancelled` (or `Cancelled (partial-shipped)` if some phases shipped before abandonment), keep the number, record the date and reason. Number retired.
5. **Splitting a job.** Keep the parent ID. Add `PJ-NNNa`, `PJ-NNNb`, etc. as siblings. The parent entry stays as a header pointing at the children. Cross-reference in both directions.
6. **Merging jobs.** All merged numbers stay in the doc. Survivor keeps its number; merged entries point at the survivor with `merged into PJ-NNN`. All merged-in numbers are retired.
7. **Renumbering. Strictly forbidden.** PJ-NNN is a permanent reference identity. Session logs, commits, and other docs cite jobs by number. Renumbering would silently break every historical reference.
8. **Filename convention.** Same as orientation + Laws docs + NotePane Specs. New version = new file alongside the previous. Older versions stay as historical record.

---

## Appendix — Cross-references

This doc is read alongside:

- `CLAUDE.md` — operational rules. Several jobs cite specific top-principals or rules.
- `docs/Constellation Orientation & Onboarding v1.55.md` (current at v1.4) — project's operating state, with predecessor versions back to v1.0. Jobs cite orientation versions where they were first surfaced.
- `docs/Constellation Development Laws v1.4.md` (current) — higher-order law statements. Jobs are the *surfaces* the Laws operate on.
- `docs/Constellation-Sight-Concept-Paper-v1.1.md` (NEW in v1.4) — Sight's analytical foundation. PJ-035 / PJ-036 / PJ-037 / PJ-038 all derive from its §3 / §12 / §13 / §14.
- `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — daily engineering record. Jobs are surfaced into a session log entry when work begins; entry ID + commit hash flow back into Done.
- `lab/reports/MIG-NNN-*.md` — Architect / Plan / Audit docs for each MIG. Several jobs cite specific audit findings.
- `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` (NEW in v1.4) — partial-shipped audit closing PJ-034.
- Auto-memory at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\` — every `project_*.md` file is a candidate source for a Pending Job entry.

---

**End of v1.13.** Reconciliation version — folds 37 migration numbers (MIG-036→072) into the record (authoritative table now in orientation §8 v2.61), allocates PJ-059→064, corrects the 2026-05-29 "PJ-10/11" collision, and refreshes the top-of-queue. New #1: **PJ-060** (`index_note` cache short-circuit). Previous milestone: `localization-complete` (MIG-072 closed, 2026-06-09).
