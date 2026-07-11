# MIG-100 — Auto-Restore Tabs on Relaunch — Plan (Phase 2)

**Date:** 2026-07-11 · **Base:** `31f64db2` · **Design:** D2 (dedicated `.constellation/session.json`) + all 11 Architect corrections · **Architect:** `docs/MIG-100-Auto-Restore-Tabs-Architect.md`
**Plan workflow:** `wf_2cfdbdc3-6d6` (planner + coverage checker + feasibility checker; all checker findings folded in below — 2 build-order errors, 1 stale anchor, 3 weak verification clauses, 1 bundling split, 1 inspection-cadence gap).

**Concept (the horse):** a thinking session doesn't end because the app closed — reopening Constellation puts the desk back the way you left it. Settings toggle, default ON.

**Scope honesty (correction 10):** v1 restores tab list + active tab + split on/off + direction + pinned. No per-pane split assignment (not persisted today by named workspaces either), no `layout` field, no second screen (SS always starts closed — standing decision). **Known limitation (documented, not built):** two app instances on one universe are last-writer-wins on session.json — identical to `workspaces.json` today.

Each step is one commit (`MIG-100 §N — …`) and leaves the app build-green (`svelte-check` 0/0, `cargo test` green). **Safety-inspection cadence (standing order, per-build):** every step touching persisted-JSON / lifecycle / close paths — §1, §3, §4a, §4b, §5, §6 — runs the `safety-inspection` workflow **diff-scoped over that step's changed files before its commit**; every confirmed finding fixed before the commit. §8 runs the **whole-app** sweep (per-cycle, migration close).

---

## §1 — Rust IPC pair + rotation + boot-bundle field + baseline measurement

**Files:** `src-tauri/src/universe.rs` (new commands beside `read_universe_workspaces` :1397 / `save_universe_workspaces` :1411, reusing `atomic_write` :116-129), `src-tauri/src/boot_bundle.rs` (new `session` field, cloned from the workspaces step :92-96), `src-tauri/src/lib.rs` (register both, beside :574-575). **~90 LOC + ~70 test LOC.**

- `read_universe_session(universe_root)` — **explicit root parameter, never the ambient `active_constellation_dir`** (correction 1). Reads `{root}/.constellation/session.json`; missing → null; parse-fail → try `session.prev.json` → null. Never an `Err` for absence/corruption.
- `save_universe_session(universe_root, session)` — explicit root; `session: null` **deletes** both generations (correction 7's delete primitive). **Per-launch rotation** (correction 8): the first save of the process lifetime (`std::sync::Once`/AtomicBool) renames `session.json` → `session.prev.json`; later saves overwrite only the current generation.
- Boot bundle: populate `session` by resolving the active root once in Rust and passing it explicitly — same fault-tolerant `time_step!` shape as :92-96.
- Rust tests: rotation-once-per-launch; missing→null; corrupt→`.prev` fallback; two temp roots stay isolated; null-deletes-both.
- **Baseline measurement (correction 11, first half):** before any frontend change, record boot timing on the 7,600+-note universe — 3 runs, `boot:paint`→`boot:hydrated` from the perf trace (`invoke('get_perf_trace_log')`, +layout.svelte:3578, registered lib.rs:618), medians into the session log.

**Verification:** `cargo test` green; app boots and behaves identically (no frontend consumer yet); `workspaces.json` untouched by any new code path; baseline numbers logged. Diff-scoped safety-inspection clean.

## §2 — Frontend `session.ts` module (types, signature, tracker, persist) + settings field

**Files:** new `src/lib/libraries/session.ts`; `src/lib/libraries/store.ts` (**`AppSettings` field + `DEFAULT_SETTINGS` entry land HERE**, not §6 — feasibility ERROR #1: §3's toggle gate must compile); `vitest.config.ts` (**add `session.test.ts` to the explicit include list** :24-90 — feasibility ERROR #2: otherwise the tests silently never run). No wiring into boot yet. **~210 LOC + ~80 vitest LOC.**

- `SessionSnapshot` type: `{version: 1, savedAt, tabs: [{path, libraryName, libraryColor, pinned}], activeTabPath, splitActive, splitDir}` — no `layout`, no `secondScreen` (correction 10). Frontend validates version+shape; unknown version = no session, never an error.
- `restoreTabsOnRelaunch: boolean` in the `AppSettings` `// Editor` group beside `alwaysFocusNewTabs` (store.ts:3927); `DEFAULT_SETTINGS` (:4394) = `true`. Existing users get ON automatically (`applyParsedSettings` spreads `DEFAULT_SETTINGS` first, :4667).
- **Signature** over tabs/active/split only — content, cursor, scroll excluded → keystrokes can never schedule a write (Rule 8; content lives in noteModel, not `OpenTab`).
- `startSessionTracking(universeRoot, seedSignature)` — root captured at arm time via `get_active_universe_path` (wrapper src/lib/universe/store.ts:34-36; feasibility note #5) — **all persists, including the later switch-time flush, are thereby gated by the toggle of the universe that armed them** (a universe whose toggle is OFF never arms, so no flush of it can ever exist). One subscription over `openTabs`/`activeTabId`/`splitActive`/`splitDirection`; signature ≠ lastWritten → 1000 ms debounce → persist.
- `persistSessionNow()` — **signature-guarded** (never writes an unchanged or empty-because-unarmed snapshot) and **serialized through one in-flight promise** (correction 1); failure keeps the dirty flag and retries on next mutation — never silent-drop.
- `stopSessionTracking()` — **cancel-and-flush** the pending timer to the captured root (correction 11), then unsubscribe. Restore **generation token**: bumped by `stopSessionTracking`; the boot restore checks it before committing tabs (kills switch-mid-restore).

**Verification:** vitest green AND the new file is proven to actually execute (fail-first canary once, then real assertions: signature stability, debounce coalescing, empty-guard, in-flight serialization, cancel-and-flush); `svelte-check` 0/0; zero behavior change (module unimported by the app).

## §3 — Boot restore (batch-insert, sentinel, gates, journal markers)

**Files:** `src/routes/+layout.svelte` (`initializeApp` :2319; hook after `await refreshLibraryCaches()` :2550, i.e. post-`boot:hydrated` :3677; bundle-failure fallback :2482-2507), `src/lib/libraries/store.ts` (new exported batch-insert helper beside `openNoteTab` :1707), `session.ts` (`restoreSessionThenTrack`). **~130 LOC.**

- Fire-and-forget `restoreSessionThenTrack(bundle.session, universeRoot)` — adds **zero awaited work** to the boot path; `appReady` paint (:2330) and `boot:hydrated` untouched. Fallback path (bundle IPC failed): invoke `read_universe_session` directly (closes refuter minor #9).
- **Gates, in order:** toggle OFF → skip entirely; `safeBootMode` (pattern :2459-2465) → skip (correction 2); **crash-loop sentinel** — localStorage marker written before the loop, cleared on success; marker present at boot → skip once, journal it (correction 2).
- **Batch-insert (correction 3):** build the full `OpenTab[]` + note models from the snapshot (per-path `resolveNoteContent` read; missing file → skip that tab, never abort the rest), then **one** `openTabs` update, **one** activation by path — no per-tab focus churn, no N×CM6 mount/teardown, no wab pollution. If the user already opened tabs before restore fires, **append without stealing focus**. Check the §2 generation token immediately before the commit; stale → abort, journal.
- **Zero disk writes (correction 4 / Gate #8):** the batch path performs **no** `ensure_cid_cn_cmd`; cid-ensure is **deferred to first user focus** of each restored tab (module-level pending set, drained on user tab activation). Bracket the whole restore with `journal_frontend_marker('session_restore_begin'/'end')` (write_gate.rs:784, registered lib.rs:499).
- **0-of-N restored = restore failure (correction 6):** preserve the file, journal, and **do not arm tracking until the first user tab mutation** (kills the unmounted-drive wipe). Partial success arms normally, seeded with the actually-restored signature (self-healing prune on next persist).
- **Arm in `finally` (correction 5):** `startSessionTracking` (or the 0-of-N deferred arm) runs in `finally`; any restore failure journals a marker — no silent dead tracker. Tracking is never armed while `openTabs` is boot-empty (empty-overwrite structurally impossible).

**Verification:** hand-written `session.json` → relaunch restores tabs, correct active tab, split; write-journal shows **zero .md lines between the restore markers**; corrupt file → `.prev` → tabless boot, no error; sentinel test (kill app mid-restore → next boot skips once); **safeBootMode boot → journal skip-marker present, no restore attempted** (coverage WEAK #2 closed); typing latency unchanged. Diff-scoped safety-inspection clean.

## §4a — Universe switch + universe create hooks

**File:** `src/routes/+layout.svelte` (`handleUniverseSwitch` :2660, `handleUniverseCreated` :2614). **~30 LOC.** *(Split from the draft's §4 — coverage WEAK #14: two subsystems, two commits.)*

- `handleUniverseSwitch` step 0: `stopSessionTracking()` (cancel-and-flush to the **captured** old root — safe even though `setActiveUniverse` already flipped the ambient pointer at UniverseManager.svelte:54-56, because the IPC takes the explicit root; corrections 1+11 close both the ordering kill and the stray-debounce clobber). After `initializeApp()` re-runs, the new universe's bundle session restores and tracking re-arms — symmetric with boot.
- `handleUniverseCreated`: same stop-and-flush **before** `setActiveUniverse` (:2615). Boot pick-loop (:2984) needs nothing — tracking is never armed before restore.

**Verification:** A→B→A switch round-trip — each universe's file contains only its own tabs (inspect both files); pending-debounce-at-switch writes no empty session and no cross-root write. Diff-scoped safety-inspection clean.

## §4b — The close path (Rust `CloseRequested` final flush)

**File:** `src-tauri/src/lib.rs` (`CloseRequested` :634-648) + a small `+layout.svelte` listener. **~50 LOC.**

- **Correction 9 (the critic's gap):** in the existing main-window `CloseRequested` arm (lib.rs:641-646): `prevent_close()` → emit `session:final-flush` → frontend listener runs `persistSessionNow()` and acks → Rust awaits ack with a short timeout (~700 ms) → destroy second screen → close. **Implementation note (feasibility #4):** `on_window_event` is a synchronous closure — the await runs via `prevent_close()` + `tauri::async_runtime::spawn` + `window.destroy()`; `destroy` bypasses `CloseRequested`, so that alone is the re-entry guard. Graceful close = zero arrangement loss; the 1 s debounce remains the net for force-kill.
- **Empirical `beforeunload` verification item:** instrument a temporary `beforeunload` → `journal_frontend_marker` in `+layout.svelte`, close the app, check the journal. Record the result in the session log; keep a DOM-side belt-and-braces hook only if proven to fire. No in-repo precedent proves a fire-and-forget invoke survives teardown — this settles it with evidence.

**Verification:** graceful close persists a just-made tab change (file mtime + content proof); force-kill loses ≤1 s of arrangement, never content; second screen still hides/destroys correctly; app close is not perceptibly delayed (timeout honored). Diff-scoped safety-inspection clean.

## §5 — Re-route `restoreWorkspace` through the shared batch-insert

**File:** `src/lib/libraries/store.ts` (`restoreWorkspace` :5071-5100). **~25 LOC.**

The Architect census documents two latent defects in `restoreWorkspace` (unflushed `openTabs.set([])` :5073; `newTab`-undefined collapse loop :5077-5081) — and auto-restore makes manual restores *more* likely to hit them (tabs guaranteed open early). WA#6 (fix what you discover) + the reuse rule: replace its clear+loop with flush-per-departing-tab (the v3.34 `flushOutgoing` discipline) + the §3 batch-insert helper. Named-workspace file handling untouched.

**Verification:** manual workspace restore of a 3-tab workspace yields 3 tabs (not 1); dirty note at restore time survives (harness recipe); tracker observes the mutation → session.json converges ≤1 s later; `workspaces.json` byte-identical through an auto-session lifecycle. Diff-scoped safety-inspection clean.

## §6 — Settings toggle UI + lifecycle + i18n ×15

**Files:** `src/lib/components/SettingsModal.svelte` (editor section, row cloned from :954-964, via `updateSettings` :4878), `session.ts` (lifecycle), all 15 locale files (`settings.editor.restoreTabsOnRelaunch` + `Desc`). *(The `AppSettings` field + default landed in §2.)* **~50 LOC + 30 i18n lines.**

- **Toggle lifecycle (correction 7):** OFF → `stopSessionTracking` (no final flush) + `save_universe_session(root, null)` deletes both generations ("stop remembering" honesty); ON mid-session → `startSessionTracking` live, seeded from current tabs. Wired via a `session.ts` subscription armed during `restoreSessionThenTrack`'s `finally` (exists whether or not restore ran).
- SS receives the value free via `screen:settings-changed` (store.ts:4882) and ignores it.

**Verification:** OFF → file + `.prev` gone from disk, relaunch boots tabless; ON mid-session → file appears ≤1 s after the next tab change; relaunch restores; `svelte-check` 0/0; all 15 locales carry both keys. Diff-scoped safety-inspection clean.

## §7 — Harness recipes + Gate #8 proof + after-measurement

**Files:** `tests/mig-076/runtimeHarness.test.ts` (beside Recipe F :130-147 — existing file, already on the vitest include list), session log. **~160 test LOC.**

- New recipes: (R1) batch-restore N tabs → fake-disk write count = 0 (Gate #8, harness-level); (R2) 0-of-N failure → snapshot preserved, tracking not armed, first user tab-open persists fresh state; (R3) stray-debounce at switch → no empty write, no cross-root write; (R4) restore + already-open user tab → append, focus unchanged; (R5) restore→edit→close round-trip → on-screen === disk (Gate checklist); **(R6) deferred cid-ensure actually drains** — restored tab with stripped cid: zero writes at restore, cid ensured on first user focus (coverage WEAK #4 closed); **(R7) mid-restore throw** — forced failure (e.g. split-apply rejects) → tracking still armed + failure marker journaled (coverage WEAK #5 closed).
- **Runtime Gate #8 proof:** relaunch against the 7,600-note universe, then confirm the write journal has zero .md-write lines between `session_restore_begin/end`.
- **After-measurement (correction 11):** repeat §1's protocol — 3 runs, medians vs baseline; regression beyond noise blocks close until fixed. Typing-latency spot check (Rule 7 burst test) in both panes.

**Verification:** all recipes green; journal proof archived in the session log; boot delta ≈ 0; then **build the Boss binary** (`npm run build` first, then `cargo build --release`, grep `build/` for the new settings string, verify binary mtime) and run the staged Boss tests below.

## §8 — /simplify + whole-app safety-inspection + docs close

Run `/simplify` on the full diff; run the `safety-inspection` workflow **whole-app** (per-cycle rule — a migration close is a cycle boundary); fix every confirmed finding before commit (WA#6). Docs in the same close: help topic in `docs/help.uConstellation.World/` + User Manual + 14 translations; **orientation doc version-bump in the same commit**; session log; MoCh housekeeping. **Verification:** inspection register clean; `svelte-check` 0/0; `cargo test` green; PCS complete.

---

## Traceability — correction → step

| # | Architect correction | Step(s) |
|---|---|---|
| 1 | Explicit-root IPC + serialized in-flight promise | §1, §2, §4a |
| 2 | Crash-loop breaker sentinel + safeBootMode gate | §3 (both verified) |
| 3 | Batch-insert restore, single activation, append-if-user-active | §3, §7 (R4) |
| 4 | Zero writes at restore: deferred cid-ensure + journal markers | §3, §7 (R1 + R6) |
| 5 | Arm-in-`finally` + journal on restore failure | §3, §7 (R7) |
| 6 | 0-of-N = failure; preserve file; delayed arm | §3, §7 (R2) |
| 7 | Toggle lifecycle: OFF deletes, ON arms live | §6 (delete primitive §1) |
| 8 | Per-launch `.prev` rotation | §1 |
| 9 | `CloseRequested` final flush + empirical beforeunload check | §4b |
| 10 | Honest scope: no `layout`, narrowed split claim, SS out | §2, Boss tutorials |
| 11 | Measured boot timing; cancel-and-flush; signature-guarded persist | §1 (baseline), §2, §7 (after) |

**Invariants → steps:** boot/typing/IPC unregressed — §1+§7 (measured); relaunch writes zero .md bytes — §3+§7 (journal-provable); `workspaces.json` never touched by the auto path — §1 (separate file/IPC), §5+§8 (verified byte-identical); no keystroke-path writes / no boot-empty write — §2+§3; universe A never lands in B's file — §1+§4a (round-trip proven); content durability (wab, `noteSession.save`) untouched — §4b adds only, verified §7 (R5); SS closed at boot — §2 scope (and §4b re-verifies hide/destroy).

## Risk mitigation (Phase-1 risks)

- **Cross-universe contamination** — explicit-root IPC, root captured at arm time; serialized persists; stop-and-flush as step 0 of switch/create (§1, §2, §4a; proven by round-trip + R3).
- **Empty-overwrite at boot** — tracking armed only post-restore in `finally`, seeded with the restored signature; signature-guarded persist; no subscription exists while `openTabs` is boot-empty (§2, §3).
- **Crash-loop** — sentinel skips restore once after a failed attempt; safeBootMode skips always; toggle reachable in both cases (§3).
- **Restore vs live user** — batch-insert appends without stealing focus if the user beat the restore; single activation only when the user hasn't navigated; generation token aborts a restore superseded by a switch (§3, R4).
- **Unmounted drive** — 0-of-N = failure: file preserved, arm deferred to first user tab mutation (§3, R2).
- **Silent dead tracker** — arm-in-`finally`; journal marker on failure; persist failures retry instead of dropping (§2, §3, R7).

## Rollback

- **Per-step:** commits land in dependency order; each reverts cleanly in reverse. §1 is inert without §3 (no consumer); §3 is the single boot call site — reverting it alone disables the feature wholesale; §4a/§4b/§5/§6 revert independently.
- **Global:** the toggle is the user-level kill switch (OFF = no restore, no tracking, file deleted). Code-level: revert §3–§6; the Rust pair may stay (unused, harmless) or be unregistered.
- **Stale `session.json` after rollback:** inert — nothing reads it; `{v:1}` versioning means any future reader treats unknown shapes as no-session. Manual cleanup: delete `.constellation/session.json` + `session.prev.json`.

## Boss tests (after §7's binary; staged — Stage 2 only after Stage 1 passes)

**What this feature is:** Until now, closing Constellation forgot which notes you had open — every launch started blank, and you rebuilt your desk by hand. Now the app remembers your open tabs, which one you were reading, and whether the window was split, and puts them back automatically the next time you launch. It's on by default; a switch in Settings turns it off. Your notes' *content* was always safe — this remembers the *arrangement*.

### Stage 1 — basic relaunch restore

1. **Pre-state:** Constellation open in your main universe, no tabs. **Action:** open three notes from the file tree, one after another. **Expect:** three tabs across the top.
2. **Action:** click the middle tab so it's the active one. Wait ~3 seconds (the app saves the arrangement about a second after any change). **Expect:** nothing visible — that's normal.
3. **Action:** close Constellation with the window's X button. Relaunch it. **Expect:** after the app finishes loading, the same three tabs reappear in the same order, and the **middle** tab is the active one. **If instead** the app opens blank: the restore didn't fire — report it, and tell me whether Settings → Editor shows "Restore tabs on relaunch" switched on. **If** only some tabs return: note which ones — a file may have been moved/renamed between runs (missing files are skipped by design, silently).
4. **Action:** split the window (open a note in split view), wait 3 seconds, close, relaunch. **Expect:** the split comes back on the same side. **Known limit (by design):** which tabs sat in which half isn't remembered yet — the tabs all return, the split returns, but the arrangement inside the split may differ.
5. **Action:** type a few sentences in a note, then immediately (within 1 second) close the app. Relaunch. **Expect:** your text is there — content safety is a separate, older mechanism and must be unaffected. **If** text is missing, stop and report immediately: that's the most serious possible failure.

### Stage 2 — edge cases

1. **Toggle off.** Open Settings → Editor → turn **off** "Restore tabs on relaunch." Close and relaunch. **Expect:** the app starts blank, like before this feature. Relaunch once more — still blank. **If** tabs return with the switch off, that's a failure.
2. **Toggle back on.** Turn the switch on, open two notes, wait 3 seconds, relaunch. **Expect:** both tabs return — turning it back on starts remembering immediately, not on the next launch.
3. **Missing file.** With two tabs open (wait 3 seconds), close the app. In Windows Explorer, move one of those two .md files into another folder. Relaunch. **Expect:** the surviving note's tab returns; the moved one is simply absent; no error dialog, no blank screen. Move the file back afterwards.
4. **Universe switch.** Open universe A with two tabs, then switch to universe B (open different tabs there), then switch back to A. **Expect:** A shows A's two tabs again; B remembered its own separately. **If** you ever see B's notes appear inside A (or the reverse), stop and report — that's the cross-universe failure this design specifically guards against.

---

**Estimated total:** ~610 LOC product + ~310 LOC tests across 9 commits. Boss decision points: Stage 1 and Stage 2 test verdicts; everything else cascades under Plan-Approval-Equals-Build-Approval.
