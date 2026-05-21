# Session Log — 2026-05-21

Continuation of the MIG-040 session (the 2026-05-20 log carries MIG-040 through commit `e48a0f04`). Today: the **NSC summary backfill** (Rule 8 first-time population), built on Eisa's directive to prioritize the backfill over the help-docs/manual sync ("2 > 1", 2026-05-20).

## NSC summary backfill — background, resumable, gentle

**Goal**: pre-compute the summary for every note lacking a current one, into `note_summaries`, so cards show summaries instantly instead of computing lazily on scroll.

**Architect** (approved via the trigger-model question — Eisa chose "Auto after app opens"): mirror two proven patterns rather than invent —
- `classifier::scan_job` worker shape (AtomicBool running/cancel, AtomicUsize completed/total, `nsc:backfill` events, resumable by re-enumeration), and
- `sky_backfill` throttle (inter-note sleep so it yields).

### Phase 1 — Rust worker (`src-tauri/src/nsc/backfill.rs`, new)
- `NscBackfillState` + `nsc_backfill_start` / `_status` / `_cancel` commands (registered in `lib.rs`; state `.manage`d).
- `run_backfill`: enumerate pending → emit `start` (only when total > 0) → per note: cancel check, **pause while a classifier scan runs** (`while classifier_scan_running`), `crate::nsc::get_or_compute_cached`, progress event every 25, **30 ms inter-note sleep** → `done`.
- `enumerate_pending`: `SELECT path FROM note_meta WHERE NOT EXISTS (SELECT 1 FROM note_summaries WHERE path=m.path AND content_hash LIKE 'v2:%')`. Resumable + recomputes pre-v2 (old-algorithm) rows.
- Exposed `nsc::NSC_ALGO_VERSION` + `nsc::get_or_compute_cached` as `pub(crate)`.
- `cargo check` clean; 16 NSC unit tests still pass.

### Phase 2 — Frontend trigger + strip + settings
- `NscBackfillProgressStrip.svelte` (new) — mirrors `ClassifierScanProgressStrip`: listens `nsc:backfill`, percent + Cancel, recover-on-mount via `nsc_backfill_status`, hides 4 s after done/cancelled/error. Rendered in the `+layout` footer `.sb-center`.
- `+layout.svelte`: after-paint trigger — `setTimeout(() => invoke('nsc_backfill_start'), 8000)` in `onMount`, gated on `appSettings.nsc?.backfillEnabled !== false` (default ON). Deferred a little past the CECE scan; worker also stands down during a scan.
- `SettingsModal.svelte`: toggle in the CECE section (after Background classification) writing `nsc.backfillEnabled`.
- `store.ts`: `AppSettings.nsc?.backfillEnabled` added to interface + DEFAULT_SETTINGS (`true`) + deep-merge on load.
- i18n ×15: `nscBackfill.{label,done,cancelled,error,cancelling,cancel,cancelTitle,settingName,settingDesc}` via `scripts/add-nsc-backfill-i18n.mjs` (full translations, not English fallbacks).
- `svelte-check`: same 3 pre-existing errors, 0 new.

### Phase 3 — audit (structural, pre-build)
- No boot regression — trigger is after-paint + deferred 8 s, worker is a spawned thread (cannot touch boot). Empirical confirm folds into Boss test.
- Resumable by re-enumeration; pauses during classifier scan; engine lock released per note + 30 ms sleep for interactive fairness.

## Open
- Boss test the backfill on the 7,600-note Universe (progress strip, cancel, toggle off, typing-while-running smoothness, boot unchanged).
- Commit as a MIG-040 follow-on + orientation v2.20 → v2.21 (§4.6 NSC: add the backfill worker + Settings toggle) in the same commit (SO #6).
- Still deferred: help files + User Manual (Cataloger + NSC, 15 langs).
- `e48a0f04` (MIG-040) remains UNPUSHED — Eisa hasn't called for a push.

## STATE OF STANDING — boot-time pivot (2026-05-21, SO #5)

Eisa's Stage-3 finding redirected priorities to **boot time**. Snapshot before pivoting:

**Verified-shipped & protected (committed `e48a0f04`, UNPUSHED):** MIG-040 — NSC summaries + callout precedence + disambiguation fix + Cataloger cross-instance sync + reload-on-reopen + note-picker + newest-first queue + picker-Escape fix. Boss-tested Stages 1–3 + Esc.

**In-flight / uncommitted (worktree):** NSC summary backfill — `nsc/backfill.rs`, `NscBackfillProgressStrip.svelte`, `+layout` after-paint trigger, `SettingsModal` toggle, `store.ts` `nsc.backfillEnabled`, i18n ×15, `scripts/add-nsc-backfill-i18n.mjs`, `SESSION-LOG-2026-05-21`. Stages 1–2 PASS.

**KNOWN-BROKEN (the pivot trigger):** the backfill, when ON, regresses *perceived* boot ~4 s → ~28 s. Root cause from `boot-perf.latest.json` (7,646 notes): **instrumented boot is 3.3 s** (paint 445 ms, hydrated 1.6 s, graph-ready 3.3 s). The embedding engine is lazy (`embeddings.rs:362`, `with_intra_threads(2)`); nothing loads it at boot normally. The backfill's after-paint trigger (`+layout`, `setTimeout 8000` → `nsc_backfill_start`) FORCES the model load + 7,646-note embedding from 8 s on, pinning ~2 cores → app feels busy for ~24 s. `cece.backgroundScan="on_save"` (NOT a boot factor); `nsc.backfillEnabled=false` now (Eisa disabled it Stage 3 → his app is at 4 s). My earlier "structurally cannot affect boot" claim was WRONG — empirically disproved.

**Pending decision (Eisa):** how the backfill must behave so instant boot is GUARANTEED (a MUST). Leading options: (A) default OFF + manual "Build summaries now"; (B) idle-gated auto (runs only when app idle, stops on interaction). Until decided, the backfill stays uncommitted.

**Pending (not started):** help files + User Manual (Cataloger + NSC, 15 langs). Optional deeper boot tuning below 3.3 s (`ensure_db` ~1.1 s + graph transport ~1.1 s are the largest sub-phases — already fast).

**Doc drift:** orientation v2.20 intentionally does NOT yet mention the backfill (uncommitted + design in flux).

## Note-open lag — root cause + fix (2026-05-21)

Eisa re-scoped: his real daily pain is **every note open lags/stutters ~5s before scrolling smooths**, independent of note length/media, **and lags again on reopen**. Researched Obsidian instant-boot (persistent metadata cache + `onLayoutReady` deferral + lazy graph; SQLite WAL hygiene; render-shell-first; tiny critical bundle) via a research agent — findings logged.

Diagnostic answers ("every note", "lags again on reopen") ruled out a one-time warm-up (so NOT the embedding model load). Traced the note-open path: `+layout.svelte:1246` fires `scanUnlinkedMentions(tab.name, tab.path)` (debounced 500ms) on **every** active-tab change. **Root cause (read in `libraries.rs:2015` `scan_unlinked_mentions` + `scan_unlinked_recursive`): it walked the ENTIRE library tree and `fs::read_to_string` + regex-scanned EVERY `.md` file (all 7,646) on every note open, uncached.** That pegs CPU/disk for ~5s and starves the WebView's scroll rendering. Explains "size/media-independent" (cost is scanning the OTHER notes) and "lags again on reopen" (uncached). Classic Rule 3 + Rule 8 violation.

**Fix (`libraries.rs`, Rust-only, frontend contract unchanged):** candidate selection now uses the always-current `notes_fts` index — Arabic-normalized phrase MATCH for the title → JOIN note_meta → ≤300 candidate (path, library_name), `ORDER BY bm25`. Then the EXACT original gate runs on just those candidates: read raw file, strip wikilinks, word-boundary regex, build context + human title, cap 50. ~50× fewer file reads → sub-100ms vs ~5s. Removed `scan_unlinked_recursive` (dead). Kept the frontend debounced effect as-is (now cheap enough). Minor deliberate narrowing: a title mentioned ONLY in another note's frontmatter (not body) is no longer surfaced (FTS indexes body, not frontmatter) — arguably more correct for "unlinked mentions." `cargo check` clean. Build running for Boss before/after test.

Note: other full-vault scanners exist (e.g. `scan_library_tags`, `libraries.rs:2123` — boot/Dashboard cost) — separate, not the note-open culprit; candidates for the same FTS treatment later. WAL hygiene (372 MB) still pending — would further speed the FTS query + boot.

Committed `b7e17603` (note-open fix) + pushed `e48a0f04`+`b7e17603` to origin/main (Eisa: push now).

## Boot < 2s push — WAL hygiene + backfill manual-only (2026-05-21)

Eisa set a hard goal: **boot in < 2 seconds**, and chose WAL cleanup as the next task + decided the parked backfill becomes **manual-only**.

### WAL hygiene (`search.rs`, committed-pending)
Confirmed `search.db-wal` = 372.8 MB (healthy = a few MB); boot trace shows DB-open (`ensure_db`) ≈ 1.1 s — the WAL traversal. Root: passive auto-checkpoints reset the WAL reuse position but never shrink the FILE; a past heavy write (re-index/backfill) left a 372 MB high-water mark; nothing runs TRUNCATE. No long-lived reader pins it (`cache.rs::open_reader` is short-lived per read), so TRUNCATE is safe. Index is EPHEMERAL (rebuilt from `.md`) → `synchronous=NORMAL` carries no real durability risk.
- `init_db`: added `PRAGMA synchronous=NORMAL; busy_timeout=5000; mmap_size=268435456;` on the main connection.
- `spawn_wal_checkpoint_daemon(path)`: own connection; sleeps 20 s post-boot, `PRAGMA wal_checkpoint(TRUNCATE)`, then every 5 min. Called once from `ensure_search_db_ready` (which early-returns after first init). `cargo check` clean.

### Backfill → manual-only (Boss decision)
The auto-after-paint trigger was the boot regressor; removed it.
- `+layout.svelte`: removed the `setTimeout(nsc_backfill_start, 8000)` auto-trigger (kept the footer `NscBackfillProgressStrip`).
- `CatalogerView.svelte`: added a **"Build all summaries"** button (outline style) → `invoke('nsc_backfill_start')`; `backfillRunning` tracked via `nsc:backfill` listener + `nsc_backfill_status` recovery; listener cleaned up in `onDestroy`.
- `SettingsModal.svelte`: removed the "Pre-build note summaries" toggle.
- `store.ts`: removed `AppSettings.nsc.backfillEnabled` (interface + default + merge) — no longer referenced.
- i18n: `nscBackfill.buildNow` + `buildNowTitle` ×15 (replaced the obsolete `settingName`/`settingDesc` via the rewritten `scripts/add-nsc-backfill-i18n.mjs`).
`svelte-check`: 3 pre-existing errors, 0 new.

Build running for Boss test: (1) boot < 2 s, (2) WAL shrank, (3) the manual "Build all summaries" button works + progress strip + cancel. Commit after pass (likely two: WAL hygiene; backfill manual-only).
