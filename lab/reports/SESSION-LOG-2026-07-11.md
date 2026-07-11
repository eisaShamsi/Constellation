# Session Log — 2026-07-11 (post-cockpit session — MIG-100 opens)

> Session start state: HEAD `31f64db2`, working tree clean, binary 2026-07-11 14:03 (cockpit lenses complete + Boss-validated — see `HANDOVER-2026-07-11-cockpit-lenses-complete.md`).

## MIG-100 — Auto-Restore Tabs on Relaunch — Phase 1 (Architect) COMPLETE

- **Function in hand:** Auto-restore-tabs-on-relaunch (Boss-wanted 2026-07-09; a Settings toggle, default ON — open tabs are not persisted across restart today, only manual named workspaces).
- **Boss picked this thread** at session start (over the safety-sweep backlog and Orrery polish).
- **Architect workflow `wf_54e79170-db3`** — 15 agents, 0 errors, ~1.48 M tokens: 4 census mappers (boot/close path, workspace machinery, settings plumbing, second-screen/gate surface) · 1 WA#5 prior-art researcher (VS Code / Obsidian / Sublime / Firefox / Chrome, sources cited) · 3 competing designers (D1 max-reuse, D2 clean session file, D3 minimal) · 6 adversarial refuters (2 per design) · 1 completeness critic.
- **Key census facts:** nothing restores tabs at boot today; `restoreWorkspace` has latent defects (unflushed `openTabs.set([])`, newTab-undefined collapse) and must not be reused; +layout has NO beforeunload; Rust `CloseRequested` handler exists but persists nothing; `setActiveUniverse` flips the Rust active pointer BEFORE `handleUniverseSwitch` runs and has 6 call sites incl. the boot loop (the cross-universe landmine).
- **Prior-art verdict:** continuous debounced persist (1–15 s) + best-effort close write; close-only = canonical anti-pattern; atomic write + previous-generation fallback; missing files skip non-fatally; default-ON in every editor-class app; crash-loop breaker (Firefox).
- **Adversarial verdicts:** D1 REJECTED (failure ceiling = named workspaces wiped via the shared whole-array atomic write — 3 independent kill paths); D3 rejected as stated (its `setActiveUniverse` flush hook fires at 5 non-switch sites incl. boot → boot-breaking; accepted silent persist failure = forbidden class); D2 RECOMMENDED — every fatal finding absorbed by a cheap correction (explicit-root IPC signature, ~20-LOC crash-loop breaker).
- **Critic's structural catch (gap all 3 designs shared):** the close path should use the EXISTING Rust `CloseRequested` handler (lib.rs:633-648) for a guaranteed final flush on graceful close — not a new DOM beforeunload; `beforeunload`+IPC survival in Tauri v2/WebView2 is UNKNOWN until tested.
- **Deliverable:** `docs/MIG-100-Auto-Restore-Tabs-Architect.md` (options table, 11 mandatory corrections, invariants, rollback).
- **Boss picked D2** (dedicated session file) — committed `9a6ab328`.

## MIG-100 — Phase 2 (Plan) COMPLETE

- **Plan workflow `wf_2cfdbdc3-6d6`** — planner + coverage checker + feasibility checker (3 agents, 0 errors). All checker findings folded into the final plan:
  - **Feasibility ERROR #1:** the toggle gate in §3 would not compile before the setting exists → `AppSettings` field + `DEFAULT_SETTINGS` moved into §2.
  - **Feasibility ERROR #2:** `vitest.config.ts` uses an explicit include whitelist (:24-90) → new `session.test.ts` must be added there or its tests silently never run.
  - **Stale anchor:** `get_perf_trace_log` is invoked at +layout.svelte:3578 (registered lib.rs:618), not :3473.
  - **Coverage WEAK ×3 closed:** safeBootMode-skip verification clause added (§3); deferred-cid-ensure drain test added (R6); mid-restore-throw arm-in-finally test added (R7).
  - **Bundling split:** draft §4 split into §4a (frontend switch/create hooks) + §4b (Rust CloseRequested close path) — two subsystems, two commits.
  - **Inspection-cadence GAP closed:** diff-scoped `safety-inspection` before EVERY code commit touching persisted-JSON/lifecycle (§1, §3, §4a, §4b, §5, §6); whole-app sweep at §8 (migration close = cycle boundary).
  - Implementation notes: CloseRequested closure is sync → `prevent_close` + `async_runtime::spawn` + `window.destroy()` (destroy bypasses CloseRequested = re-entry guard); universe root sourced from `get_active_universe_path` (universe/store.ts:34-36), captured at arm time.
- **Deliverable:** `docs/MIG-100-Auto-Restore-Tabs-Plan.md` — 9 commits (§1..§8 with §4a/§4b), traceability table (11 corrections + 7 invariants → steps), risk mitigations, rollback, staged Boss tests.
- **Boss APPROVED the plan** → Phase 3 cascade.

## MIG-100 — Phase 3 (Build) — code complete §1–§7-recipes

- **§1 Rust IPC pair** — `read_universe_session`/`save_universe_session` (explicit `universe_root`, never ambient; per-launch `.prev` rotation keyed per root; null = delete both generations; corrupt current → `.prev` → null, never Err), boot-bundle `session` field (root resolved in Rust, passed explicitly), registration. **6 new Rust tests**, full lib suite 1047 passed / 0 failed.
- **Baseline boot measurement (correction 11)** — 3 launches of the PRE-change binary (2026-07-11 14:03) on the live universe `E:\Constellation Universes\كون عيسى`: paint 492/453/2263 ms, hydrated 820/616/2394 ms → **medians paint 492 ms, hydrated 820 ms** (run 3 = cold-cache outlier). Protocol: `boot-perf.latest.json` mtime-watch per launch.
- **§2 `session.ts` tracker** — snapshot/signature (arrangement only — content/cursor/scroll excluded, Rule 8), 1s debounce, signature-guarded + serialized `persistSessionNow` (sync capture at call time — a store clear one tick later can't change what gets written), failed-write dirty-retry, `stopSessionTracking` = cancel-AND-flush to the arm-time root + generation bump. `AppSettings.restoreTabsOnRelaunch` default true (moved here per feasibility ERROR #1). vitest include updated (ERROR #2). **11 tests.**
- **§3 boot restore** — `restoreSessionThenTrack` (gates: toggle → safeBootMode → crash sentinel → validate; journal-bracketed; arm-in-`finally`; 0-of-N → preserve + defer arm to first user tab mutation; bundle-failure fallback read) + `restoreSessionTabs` batch-insert in store.ts (one openTabs commit, one activation only when the user hasn't navigated, per-path skip on missing files, `stillValid` generation check before commit, models born with tabs) + **deferred cid-ensure** (pending set drained on first USER activation via the toggleTaskReconciled recipe — mark-cascading → flush-if-dirty → ensure → model ADOPTS; zero writes at boot). `deriveLibraryForPath` extracted (shared with openNoteTab). Wired in `initializeApp` post-`boot:hydrated`, fire-and-forget.
- **§4a switch/create hooks** — `handleUniverseSwitch` step 0 = `await stopSessionTracking()` (flush to captured root; kills the stray-debounce empty-clobber + aborts in-flight restore); `handleUniverseCreated` = stop + clear tabs BEFORE `setActiveUniverse` (the created-path contamination fix).
- **§4b close path** — Rust `CloseRequested` (main): `prevent_close` → emit `session:final-flush` → frontend `persistSessionNow()` + `session_flush_ack` → Rust proceeds on ack or 700 ms timeout → destroy second screen → `win.destroy()` (bypasses CloseRequested = re-entry-safe). tokio `Notify` with `notify_one` (permit stored — early ack not lost).
- **§5 restoreWorkspace re-route** — flush-per-departing-dirty-tab (durability gate) + model disposal AFTER flush + the shared batch-insert (fixes the unflushed `openTabs.set([])` and the collapse-to-one-tab loop).
- **§6 Settings toggle** — SettingsModal editor row + `onRestoreTabsToggle` lifecycle (ON mid-session arms live; OFF stops tracking + `deleteSessionOnDisk` both generations) + **i18n ×15** (`settings.editor.restoreTabsOnRelaunch(+Desc)`, all locales JSON-validated).
- **§7 recipes** — `tests/mig-100/restore.test.ts` R1–R7: Gate-#8 zero-write proof, 0-of-N deferred arm, switch-abort, focus safety, model-as-source screen===disk, deferred cid drain on activation, arm-in-finally on mid-restore throw, crash-sentinel skip-once, toggle-OFF/safeBootMode gates. **11 tests.** Full vitest 307 expected (296+11), svelte-check 0 errors.

## Safety inspections (MIG-100 build)

- **Whole-app sweep ran mid-build** (`wf_127a517c-479`, 87 agents, **63 confirmed** incl. 2 pre-existing APP-KILLERs: watcher external-change never adopts into the note model → stale-editor clobbers external edits on next keystroke (+layout:3172 class, G3-adjacent); bulk Accept-All unlocked RMW race (bulk_ops.rs:305)). Intended diff-scoped; args arrived stringified → fell back to whole-app. Register = Charter material at §8; **zero findings in MIG-100's new code**. Two findings adjacent to the diff FIXED in-build (WA#6): `atomic_write` now fsyncs before the rename (the G6 gap — protects every persisted-JSON file incl. session.json); the ambient-keyed sibling saves HIGH (settings/workspaces/collections/property-types share the cross-universe race MIG-100's session IPC avoids) is a class fix needing its own migration → Boss ruling + Charter.
- **Diff-scoped inspection over the full MIG-100 diff** — `wf_f5d6b0f3-6c6`, 11 agents, **7 confirmed — ALL FIXED before commit** (WA#6):
  1. **APP-KILLER (store.ts restore path):** the restore consumed + destroyed the write-ahead crash-recovery net while the restored model was born CLEAN — recovered content could never reach disk (background tabs never mount the editor whose teardown re-stashes). FIX: `resolveNoteContent(path, preserveNet)` — the restore honors the net's content but never clears it; a real durable save replaces it. Regression recipe R8.
  2. **MED (freeze class):** the 5 persisted-JSON save commands were SYNC and now fsync → `(async)` on all five + `read_universe_session`.
  3. **MED (content loss):** `drainCidEnsure`'s `reloadTabsFromDisk` adopt could discard keystrokes typed during the drain window. FIX: inline dirty-guarded adopt (model wins; path re-pends). Regression recipe R9.
  4. + 5. + 6. **LOW (false success):** toggle-off delete failures were swallowed at both layers. FIX: Rust null-delete surfaces failure; SettingsModal runs lifecycle-first and reverts the checkbox on failure (re-arming if the delete failed after the stop).
  7. **LOW (leak):** universe switch/create cleared tabs without disposing models. FIX: `flushDisposeClearTabs(origin)` — the ONE departure primitive (flush dirty → clear → tick → dispose), now used by restoreWorkspace + both universe-departure sites.

## /simplify (4 agents) — 6 applied, 2 judgment-call skips

Applied: the departure primitive (drift between two hand-written copies eliminated); `deriveTabName` (title-regex was triplicated); `deriveLibraryForPath` → `normalizePathKey`; `subscribeSkipInitial` in `$lib/utils` (idiom was duplicated); `setSessionEnabled` lifecycle primitive in session.ts (policy out of the view); `restoreSessionTabs` returns `activatedId` (re-scan dropped). Skipped: boot-bundle session field removal (keeps the zero-extra-IPC contract); SessionSnapshot/SessionRestoreInput type merge (import direction).

## Runtime verification (fresh binary 2026-07-11 18:10, live universe)

- Crafted 3-tab `session.json` → 3 launches: journal-proven **`session_restore_begin: 3 tabs` → `session_restore_end: 3/3 restored`** (~150 ms), **zero `.md` write lines** (Gate #8 at runtime); graceful close (`WM_CLOSE`) → §4b handshake exits cleanly ×3; `session.json` **byte-identical** through all cycles (no empty-clobber; sentinel cleared each run).
- **After-measurement:** paint 518/509/460 → **median 509 ms** (baseline 492); hydrated 807/694/627 → **median 694 ms** (baseline 820). **No regression** — within noise/better.

## Commits

- `8310ec6e` — MIG-100 §1+§4b (Rust: IPC pair, rotation, boot-bundle field, CloseRequested handshake, atomic_write fsync, 5 saves → async).
- `9d2f419e` — MIG-100 §2–§7 (frontend: tracker, restore, wiring, restoreWorkspace repair, toggle + i18n ×15, 24 recipes, inspection fixes, simplify cleanups).
- Docs close (this commit): orientation **v3.37**, User Manual + 14 translations, help topic, MoCh 15:30, this log.

**Boss Stage 1: PASS** (2026-07-11 — all five checks incl. the type-then-immediately-close content-safety check). **NEXT:** Stage 2 (toggle lifecycle, missing file, universe switch) → migration close (Charter append + tag + PCS).
