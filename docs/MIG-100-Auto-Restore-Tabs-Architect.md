# MIG-100 — Auto-Restore Tabs on Relaunch — Architect

**Date:** 2026-07-11 · **Phase:** 1 of 4 (Architect) · **Workflow:** `wf_54e79170-db3` (15 agents: 4 census mappers · 1 WA#5 prior-art researcher · 3 competing designers · 6 adversarial refuters · 1 completeness critic; zero errors)

**Concept (the horse):** a thinking session doesn't end because the app closed — the desk should look the way you left it, so reopening Constellation resumes the train of thought instead of restarting it from a blank slate. A Settings toggle, **default ON**.

---

## 1. Territory (census, verified against `31f64db2`)

- **Nothing restores tabs at boot today.** `openTabs` starts `[]` (store.ts:965); no `openNoteTab` is reachable from `initializeApp`; `restoreWorkspace` has exactly one caller — a user click in WorkspaceManager.svelte:29. The boot bundle already carries the workspaces list (boot_bundle.rs:92-94) but nothing applies one.
- **The named-workspace machinery is the proven precedent:** `Workspace` shape + `saveWorkspace`/`restoreWorkspace` (store.ts:5010-5105), per-universe `.constellation/workspaces.json`, `atomic_write` (universe.rs:1395-1414). But `restoreWorkspace` itself carries two latent defects (an unflushed `openTabs.set([])` at :5073; a `newTab`-undefined loop that collapses multi-tab restores through the in-place-replace branch, :1838-1889) — it must NOT be reused for boot.
- **Window close persists nothing.** `+layout.svelte` has no `beforeunload`; the Rust `CloseRequested` handler (lib.rs:633-648) only destroys the second-screen window. Note **content** is already safe at close (pane-level `beforeunload` flushes + the synchronous write-ahead net, store.ts:236-271) — this migration records only tab *arrangement* (paths + metadata; File-Over-App).
- **The universe-switch landmine:** `setActiveUniverse` flips the Rust active-universe pointer BEFORE `handleUniverseSwitch` runs (UniverseManager.svelte:54-56), and it has 6 call sites including the boot pick-loop (+layout.svelte:2984). Any session write keyed off the ambient `active_constellation_dir` can cross-contaminate universes. Switch also discards tabs raw (`$openTabs=[]`, :2672-2674) with no flush.

## 2. Prior art (WA#5 — VS Code, Obsidian, Sublime, Firefox, Chrome)

The dominant pattern is unanimous: **persist continuously during runtime, debounced 1–15 s** (VS Code 1 s; Obsidian ~1 s; Chrome 2.5 s; Firefox 15 s), with the close-time write as best-effort only — close-only persistence is the canonical anti-pattern (Sublime crash-loss threads; Obsidian's 0 KB `workspace.json` wipe). Atomic write + a **previous-generation fallback file** (Firefox `previous.jsonlz4`). Missing files skip non-fatally, never aborting the rest. Restore is **default-ON in every editor-class app** (Obsidian doesn't even offer an off switch). Restored tabs are lazy-loaded (only the active tab loads content) so restore never taxes startup. Firefox adds a **crash-loop breaker** (restore-attempt sentinel).

## 3. Options

| Option | Shape | Effort | Boot-speed impact | Risk | Review verdict |
|---|---|---|---|---|---|
| **D1 — max reuse** | Hidden reserved entry inside `workspaces.json`; zero Rust changes | ~220 LOC | none (post-hydration, fire-and-forget) | **HIGH** | **REJECTED.** Shares the file and the whole-array write with the user's named workspaces — three independent failure paths (cross-universe flush, stale debounce, module-subscriber realm) all terminate in *named workspaces wiped by atomic_write*. The failure ceiling is precious user data; the corrections narrow paths but cannot remove the coupling. Its sole advantage (zero Rust) is voided by the one structural fix every refuter demanded. |
| **D2 — dedicated session file** | New per-universe `.constellation/session.json` (machine-written, versioned, disposable) + `read_universe_session`/`save_universe_session` IPC pair with **explicit universe root**, boot-bundle field, tracker module `session.ts`, 1 s debounce | ~330 LOC + corrections | none (fire-and-forget after `boot:hydrated`) | **MEDIUM** | **RECOMMENDED.** Every fatal finding is absorbed by a correction that costs a decision not yet spent (a signature on a brand-new IPC; a ~20-LOC breaker). Worst-case loss is capped at "the disposable session, one generation deep, with a `.prev` fallback." Only design whose persist path retries instead of silently dropping. |
| **D3 — minimal** | Same session.json idea, fewer layers: no `.prev` rotation, silent persist-failure accepted, flush hooked into `setActiveUniverse` | ~165 LOC nominal | none | **MED-HIGH** | **Rejected as stated.** Its flush hook fires at 5 non-switch call sites *including the boot loop* (boot-breaking as written), and its accepted silent-persist-failure is the exact class the safety audit forbids. Corrected, it converges onto D2 minus D2's safety layers — the LOC advantage evaporates. |

## 4. Mandatory corrections (adversarially derived; become Plan items)

1. **Explicit-universe-root IPC** — `save_universe_session(universe_root, session)`, root captured at tracking-start, never the ambient pointer; all persists serialized through one in-flight promise. *(kills the cross-universe contamination class)*
2. **Crash-loop breaker** — restore-attempt sentinel (write marker before the loop, clear on success; marker present at next boot → skip once) + gate under `safeBootMode`.
3. **Batch-insert restore** — build the restored tab array and set `openTabs` once; activate exactly one tab, once; no per-tab focus churn (kills N×CM6 mount/teardown + wab pollution); if the user already opened a tab before restore fires, append without stealing focus.
4. **Zero disk writes at restore (Gate #8)** — defer `ensure_cid_cn` on restored tabs to first user focus; bracket the restore with `journal_frontend_marker('session_restore_begin/end')` for journal-provable zero-write restores.
5. **Arm tracking in `finally`** + journal marker on restore failure (no silent dead tracker).
6. **0-of-N restore = failure** — preserve the file, don't arm tracking until the first user tab mutation (kills the unmounted-drive session wipe).
7. **Toggle lifecycle** — OFF deletes session.json and stops tracking; ON mid-session starts tracking live.
8. **Per-launch (not per-write) `.prev` rotation.**
9. **Close the close-path gap** — wire a guaranteed final flush into the EXISTING Rust `CloseRequested` handler (lib.rs:641-646): `prevent_close` → flush → close (graceful close = zero loss; the runtime debounce remains the net for force-kill). Verify `beforeunload`+IPC behavior in Tauri v2/WebView2 empirically — no in-repo precedent proves a fire-and-forget invoke survives teardown.
10. **Honest scope** — drop the dead `layout?` field; v1 restores tabs + active tab + split on/off + direction (per-pane tab assignment is not persisted today by named workspaces either — narrow the claim, don't fake it). Second screen OUT (Display-not-Domain; SS always starts closed — standing decision).
11. **Measured verification** — before/after boot timing on the 7,600+-note universe as a build-phase verification clause; stray-debounce cancel-and-flush on `stopSessionTracking`, signature-guarded `persistSessionNow`.

## 5. Invariants that must not break

- Boot time, typing latency, IPC responsiveness — unregressed (measured, not asserted).
- A relaunch alone writes **zero** bytes to any `.md` file (Editor-Surface Gate #8).
- The named-workspaces file is never touched by the auto path.
- No session write on the keystroke path (Rule 8); no write while `openTabs` is still boot-empty (tracking armed only post-restore).
- Universe A's session can never land in universe B's file.
- Content durability paths (wab net, `noteSession.save`) — untouched.
- SS remains read-only/closed at boot.

## 6. Backfill / migration / rollback

**No schema, no backfill.** The file appears on first run; missing/corrupt → `.prev` → null → boots tabless exactly as today. `{v:1}` version field; unknown version = no session, never an error. **Rollback:** remove the boot call + tracker; a stale session.json on disk is inert. Toggle-off is the user-level rollback and deletes the file.

## 7. Recommendation

**D2 with the 11 corrections baked into the Plan.** D1's failure ceiling is the user's named workspaces; D3 corrected is D2 with fewer nets. D2 is the only option where every adversarial finding is a graft, not a rebuild.
