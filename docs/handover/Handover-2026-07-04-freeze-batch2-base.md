# Handover — 2026-07-04 — the freeze marathon (Batch-1/2 + Base + wide-app audit)

**Branch:** `main` (pushed; tip after this session — verify with `git log --oneline -3`). **Orientation:** **v3.24** is the file to read. **Session log:** `lab/reports/SESSION-LOG-2026-07-03.md` (this whole multi-day arc appended there; read tail-up). **MoCh:** `docs/MoCh/MoCh-2026-07-04-0900.md`.

## ⚠️ ONE PENDING ACTION AT PICKUP
The **release binary was NOT relinked** for the final commit (`constellation.exe` from Jul 4 16:58 predates the close-out commit's source: the R2-instrumentation removal + the Base-search O(rows) fix). The Boss's app was open, blocking the linker. **Both changes are behavior-safe** (instrumentation removal is pure; the search fix is validated-equivalent + faster, svelte-check 0) — but before the next Boss test of anything, run: `npm run build && (cd src-tauri && cargo build --release)` with the app CLOSED, then verify mtime. There is nothing to re-test from this session's tail; the last Boss-validated states (Base features PASS, switch instant) still hold.

## What shipped this session (all Boss-validated, pushed)
- **`get_360_view` freeze** (safe tier: async + loading-state) — v3.22.
- **Note-open freeze CLASS (Batch-1)** — 24 commands `(async)` + 9 stale-result seq guards + PULSE_LOCK — v3.23.
- **Batch-2 (§B2-1..B2-5)** — the note-file-writing family → async behind new **write-gate locked primitives** (`gate_rmw`/`gate_rmw_rename`/`gate_delete`/`with_path_lock`, 8 concurrency tests); `delete_item` RETIRED; the **3-round rename bug** fixed (double-fire guard + tail-park detach + permanent journal forensics).
- **Base table** — safe RC menu + in-memory search (now O(rows)) + multi-script letter rail.
- **Wide-app freeze audit** (5 agents) + **Batch S/D** — switch/boot path off the dispatch thread; `set_active_universe` measured ~5 ms.

## Canonical facts (mental model)
- **Freeze cure = `#[tauri::command(async)]`.** Near-universal now. Any NEW command touching `state.db`/`init_lock`/a long fs walk MUST be `(async)`; any frontend caller of an async command that writes shared state on resolve needs a stale-result seq/gen guard.
- **write-gate now has locked-RMW/delete primitives.** Two hard rules: no nested gate_* on the same path (non-reentrant Mutex); NO `state.db` wait inside a path lock (re-freezes the dispatch thread through the back door). DB work goes AFTER the gate returns.
- **The rename DB tail + cascade reindex are DETACHED** (`spawn_blocking`) → note_meta/note_links path rows trail a rename briefly (eventual-consistency; heals on tail/next reindex). Permanent forensic markers: `journal_marker` (Rust) + `journal_frontend_marker` (frontend, captures swallowed-exception text) → `write-journal.jsonl`.
- **`delete_item` is GONE** — use `delete_path` / `deleteWithSetting`.
- **The 20–40 s cold `init_db` is unmeasured** — it now runs in the background (panels fill when done); shrinking it needs a profiling MIG first (do NOT guess its phases).

## NEXT (ranked — the freeze audit's deferred batches; verifier-ordered)
1. **F2** — stop refreshing the file tree on our own 1.5 s autosaves (`+layout.svelte:~3053` unconditional `pendingTreeRefresh.add`). **Needs a create-path verification** (that note-creation flows call `refreshLibraryTree` explicitly, not via the watcher). Small, high-felt (kills the typing hitch with a big library expanded).
2. **W** — panel fs-walk sweep → async (Navigator `collect_library_notes_with_metadata`, `read_library_tree`, calendar/tasks scans, second-screen `scan_library_links`, `notes_by_tag`, `search_by_property`/`search_stars`, dataview, embeds). Attribute sweep; **2 WA#4 writer walks** (`repair_external_libraries_on_startup`, `cece_record_correction_for_card`).
3. **L** — importer family (canonicalize preview/execute/…) minutes-scale → async + progress events; `cache_full_links` per-schema federated_conn hold. **MEDIUM — real write commands, WA#4 caller walk + one test-library import.**
4. **F1** — **Focus mode saves per KEYSTROKE via sync `write_note`** (`FocusPane.svelte:165` + `+layout:7228` — `saveTimer` declared but never armed). **Content-integrity class → ships ALONE through the Editor-Surface Gate harness (`npx vitest run tests/mig-076/` + the reproduction recipes). Do NOT flip casually.**
5. **F3** — FileTree virtualization; CNS Louvain → Web Worker; sky 234k-parse off main thread; second-screen per-note reads (`get_backlink_rows` instead of `scan_library_links`).
- Also queued: the `get_360_view` **index-read `/migration`** (the deeper Rule-8 fix — parity findings in v3.21 log); the **switch-back-fast residual** (reproduce by switching back BEFORE settle); **init_db profiling MIG**; MIG-088 Phases 6–10; the Arabic callout End/Home caret.

## Process lessons banked
- Lifecycle/content-integrity/freeze bugs are **journal-forensic, not static-analysis** (the rename bug: 3 rounds, cracked by the write journal + added markers). Reproduce → read the trace → fix the pinned mechanism.
- **Measure before fixing a "latency"** — the switch "lag" was measured at 5 ms and the hypothesis refuted; no speculative fix shipped.
- Build + freshness-verify the binary BEFORE any Boss test tutorial (memory `feedback_build_binary_before_test_instructions`).

---

## Ready-to-paste next-session prompt

Resume Constellation. Last session (a marathon, 2026-07-03→04) shipped: the `get_360_view` freeze fix (async+loading); the note-open freeze CLASS (Batch-1: 24 cmds async + stale guards); **Batch-2** — the note-file-writing family migrated to async behind new write-gate locked primitives (`gate_rmw`/`gate_rmw_rename`/`gate_delete`), with a 3-round journal-forensic rename-bug fix and `delete_item` retired; the **Base** table's right-click + search + multi-script letter rail; and a **wide-app freeze audit** (5 agents) + **Batch S/D** fixing the universe-switch/boot freeze. All Boss-validated, pushed to `main`. Orientation is **v3.24**.

Before anything:
1. `git pull origin main`.
2. **Relink the binary FIRST** (it lags the last close-out commit — see the handover's "ONE PENDING ACTION"): with the app closed, `npm run build && (cd src-tauri && cargo build --release)`, verify mtime.
3. Read `docs/Constellation Orientation & Onboarding v3.24.md` — the "What changed in v3.24" preamble carries the whole picture (the freeze program, the write-gate primitives, the rename forensics, the deferred F2/W/L/F1/F3 batches).
4. Skim `docs/handover/Handover-2026-07-04-freeze-batch2-base.md` + the tail of `lab/reports/SESSION-LOG-2026-07-03.md`.

Then pick up (SO #8 cross-check first) the freeze audit's **deferred batches in ranked order**: **F2** (stop the file-tree refresh on our own 1.5 s autosaves — needs the create-path verification) is the next quick, high-felt win; then **W** (panel fs-walk async sweep, 2 WA#4 writer walks). **F1 (Focus per-keystroke save) is content-integrity class — it ships ALONE through the Editor-Surface Gate harness, never a casual flip.** Ask the Boss which batch. Also open: the `get_360_view` index-read `/migration`, an `init_db` phase-profiling MIG (to shrink the 20–40 s cold init — currently unmeasured, don't guess), the switch-back-fast residual, MIG-088 Phases 6–10, the Arabic callout End/Home caret. Ultracode. **Standing rule: build + freshness-verify the binary BEFORE sending any test tutorial.**
