# MIG-079 §C.2d — Plan: Defer the Sky read off the boot critical path

**Companion to:** `docs/MIG-079-C2d-Architect-Defer-Sky-Read.md`.
**Approach (Boss-approved 2026-06-17 — Option B+):** defer the sky read to first Sky-surface open (LL-022 lazy-mount) + async-ify `cache_boot_snapshot_sky` (the §9.1 lever, required companion) + a `_skyEpoch` stale-guard + an **after-idle background warm-up** so the first Sky open is warm. Boss chose B+ over pure-lazy: the boot-to-graph-ready win is preserved (the warm-up is scheduled *after* `boot:graph-ready`, async, off the IPC thread), and the first Sky open is instant when the warm-up has finished.
**Gate:** Boss approved this Plan. Plan-Approval = Build-Approval — cascade §C.2d-1 → §C.2d-2, stopping only at the Boss-test verification clause (§C.2d-2) and on genuine architectural surprise.

---

## Step §C.2d-1 — Rust: async-ify `cache_boot_snapshot_sky` (foundation, low-risk)
**Change:** `cache.rs:788` `#[tauri::command]` → `#[tauri::command(async)]`. Body unchanged (rusqlite is synchronous internally; Tauri runs an async command on its worker thread pool so it no longer monopolises the single IPC dispatch thread — the exact §9.1 fix that closed Boot Criterion 2).
**Why first:** the on-open read in §C.2d-2 *must* be async, or opening Sky View would freeze the whole app for ~11 s (cold) while it reads. Landing it alone is isolated and independently sound.
**Files:** `cache.rs` (1 attribute). No registration change (`lib.rs:446` unaffected).
**Verify (Claude-side):** `cargo build --release` clean; `cargo test --lib` green (the existing `read_sky_*_in_schema` / `is_federated_sky_ready` tests still pass — they exercise the helpers, not the async attribute). No behaviour change yet (sky still fires at boot until §C.2d-2); this step only removes the IPC-thread monopoly.

---

## Step §C.2d-2 — Frontend: defer the sky read to first Sky-surface open (the boot win)
**Changes in `+layout.svelte`:**
1. **State:** add `let skyEverOpened = $state(false)`, `let skyReady = $state(false)`, `let _skyEpoch = 0` (mirror `_linksEpoch`).
2. **Open-trap `$effect`:** set `skyEverOpened = true` when any sky surface becomes visible — `showSkyView || lensActive || sightV3Active || sightV4Active || sightV6Active || sightV7Active || showWiW || showExpressionForge || rightSidebarTab === 'star'`.
3. **`ensureSky(force?)`** — memoised async loader, modelled on `ensureFullLinks` (`:815`): invoke `cache_boot_snapshot_sky` (now async); capture `_skyEpoch` at entry; on resolve, if `epoch !== _skyEpoch` discard (stale-universe guard); if `sky.isReady`, assign `skyNodes`/`skyLinks` and bump `skyVersion`, set `skyReady = true`; if `!isReady` (mid-backfill), leave a retry armed (re-run on `federation:ready`) and do **not** use the dead `buildSkyData` fallback (it can't build under `perNoteLinkQueries`).
4. **Trigger `$effect`:** when `skyEverOpened` flips true and sky isn't loaded for the current epoch → `void ensureSky()`.
5. **`loadGraph` (`:3317`):** remove the boot `skyPromise` kick-off (`:3341`) and the `await skyPromise` + sky-assignment block (`:3405–3437`). `graphReady`/`boot:graph-ready` now fire after the graph (tags + aliases) lands only — no sky on the boot critical path. Keep `ensureFullLinks` idle pre-fetch behaviour unchanged.
6. **`handleUniverseSwitch` (`:2402`):** add `skyNodes = []; skyLinks = []; skyVersion++; skyEverOpened = false; skyReady = false; _skyEpoch++; localSkyNodes = []; localSkyLinks = [];` (closes INV-3, a pre-existing latent stale-sky race).
7. **`federation:ready` handlers (`:2752/2816`):** route the sky re-fetch through `ensureSky()` (only when `skyEverOpened`) and respect `_skyEpoch`.
8. **Loading state:** the Sky-surface containers show a "Building the graph…" state while `skyEverOpened && !skyReady` (graceful, not a frozen blank).
9. **After-idle warm-up (Option B+):** after `boot:graph-ready` fires, on the existing `schedule(...)` idle hook (requestIdleCallback, the same primitive that defers `ensureFullLinks`), call `void ensureSky()`. Because `ensureSky` is memoised, an explicit Sky open before the warm-up finishes **coalesces** onto the same in-flight promise (no double read, no need for literal cancellation); if the warm-up finished first, the open is instant (`skyReady` already true). The read runs on the worker thread (§C.2d-1 async), so the ~11 s cold scan happens in the background **after** graph-ready, never on the IPC/main thread — `graph_ready_ms` still drops to sub-second. The warm-up respects `_skyEpoch` (a universe switch mid-warm-up discards the stale result).

**Files:** `+layout.svelte`. Possibly a small loading-state in the Sky View container component if it doesn't already accept a `loading`/empty prop (confirm during build; reuse the existing pattern).

**Verify (the Boss-test gate — measured + tutorial-articulated):**
- **Measured boot (cold, PC restart):** `lab/boot-perf/read-boot-history.py` → `graph_ready_ms` drops from ~11,000 ms toward the sub-second editor floor; `cache_snapshot_graph_queue_ms` collapses (no sky ahead of it); `hydrated_ms` unchanged (~1.1 s). The warm-up's background sky read happens *after* graph-ready, so it does not regress these numbers. Before/after captured per INV-6.
- **First Sky-View open (warm-up path):** if opened a few seconds after boot, the graph is already warm and renders ~instantly; if opened immediately (before the warm-up finishes), it coalesces onto the in-flight warm-up and shows the loading state until it lands. Either way it renders the same bubbles as before. CNS / Lens / Sight / WiW / ExpressionForge / right-sidebar star all populate on open.
- **`skyNodePathSet`:** Lens-row CNS icons show at boot (permissive), then become exact after Sky first loads (INV-2).
- **Universe switch:** open Sky in universe A, switch to B, open Sky — shows B's graph, never A's (INV-3). Second screen Sky companion unaffected (INV-4).
- **Editor-Surface Gate:** read-path only — Focus round-trip + tab switch + body intact (belt-and-suspenders; no content/save/lifecycle code touched).

---

## Step §C.2d-3 — i18n + `/simplify` + docs (close-out)
- i18n the loading string (`sky.building` or reuse `common.loading`) — EN + `|| fallback`; ×15 either added now or rides the existing translation debt (Boss call; default: rides the debt, consistent with §C.2c).
- Run `/simplify` on the full §C.2d diff.
- SO #6: orientation **v-bump (new file)** in the same commit; session log; MoCh if due; User Manual note (Sky View now loads on first open).
- **Verify:** `svelte-check` 0 errors; `npm run build` then `cargo build --release`; grep `build/` for a new §C.2d string to confirm the frontend re-embedded (per the `frontend_build_before_cargo` lesson).

---

## Then — Audit (Phase 4 of the /migration)
Three parallel agents on the shipped §C.2d diff: **invariant-checker** (INV-1…8), **drift-detector** (new guards the system doesn't know about — `_skyEpoch`, `skyEverOpened`, the readiness/loading states), **migration-path-validator** (first-boot, mid-backfill `is_ready=false`, schema mismatch/rollback, universe-switch mid-load, second screen). Fix any P0/P1 before final close.

## Note on the warm-up trade (Boss-accepted)
The warm-up re-introduces the 234k-row sky read at boot — but **async, on the worker thread, scheduled after graph-ready**, so it never blocks the IPC/main thread and never delays graph-ready. Its only cost is ~11 s of background disk I/O after boot, which can lightly compete with disk-bound user actions (opening a note, a search). If Boss finds that early-second contention noticeable, the fallback is trivial: gate the warm-up behind a longer idle delay, or drop it (reverting to pure-lazy) by removing the one `schedule(() => ensureSky())` line.
