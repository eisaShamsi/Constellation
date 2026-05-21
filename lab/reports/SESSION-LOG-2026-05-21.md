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

Shipped + pushed: `a532eaeb` (WAL hygiene), `a338d9e2` (backfill manual-only), `6aa37e0d` (orientation v2.21 + LL-024). Boss-confirmed: WAL 372→0, boot ~4s→~3s, instant typing.

## Boot < 2s investigation — bundle dead-end + the real DB finding (2026-05-21, SO #5 triage)

**Goal**: Boss wants warm boot < 2 s. Pursued the "lazy-load heavy views to shrink the boot bundle" lever (Obsidian/Rule-6 research said this is the path).

**Lazy-load GraphMindView (PIXI) — built, measured, REVERTED.** Converted to a $state holder loaded on first `showSkyView` + `{#if GraphMindView}` guards. Objective win: largest boot chunk 3.28 MB → split (PIXI now a lazy chunk). svelte-check 0 new errors; Sky View works. **BUT Boss-measured warm boot unchanged (3 s before & after).** Conclusion: **the JS bundle parse is NOT the warm-boot bottleneck** — bundle-splitting is a dead end here. Reverted (`git checkout +layout.svelte`) at Boss request; tree clean.

**Cold-boot chaos diagnosed = OS file-cache, not code.** After installing a build, boots were wild (84 s, 53 s, 12 s) then settled to 3 s (tries 5–7). `boot-perf` from a 26 s boot: `ensure_db` = 23.4 s, WAL = 0.4 MB. The Rust boot path is byte-identical to the fast WAL build → the variance is the OS reading the **2.35 GB** DB cold from disk (install + AV evicting/scanning). Note Navigator plug-in exonerated (try 3 = 53 s with it OFF).

**DB audit (read-only) — the real bloat.** 2.35 GB, only ~0.11 GB reclaimable by VACUUM (4.7% free). `term_vocab` = **5,730,175 rows** (~50 K stems + **5.68 M bigrams**, ~1.7 GB) — matches MIG-013's known bigram blow-up. `ctse/search.rs` filters to single stems and **skips bigrams** (`search.rs:193`), and `bridge_concept_id` is dead schema (`search.rs:50`). So the 5.68 M bigrams may be ~1.7 GB of dead weight — NEEDS confirmation that nothing reads them before removal. Removing them → ~70 % DB shrink → fast cold boots + lighter app. Embeddings tiny (366 rows); links normal (232 K).

**NEW finding (Boss observation):** during the warm 3 s, "the app stays unpopulated for the whole 3 s, then the whole universe comes to life at once." This contradicts the instrumented `hydrated`=0.9 s — the populated UI (file tree + note) does NOT paint early; everything paints together at ~3 s. Suggests a render/reactivity gate (the heavy graph cascade or file-tree paint blocking the main thread) delays first meaningful paint until ~graph_ready. This — not the bundle — is the warm-boot lever if it's fixable.

**State**: tree clean on `origin/main` (6aa37e0d). Open directions: (A) the "paints-all-at-once-at-3s" render gate (warm-boot lever, aligned with <2 s goal + the new data); (B) term_vocab bigram shrink (cold-boot lever, ~1.7 GB, careful search-engine migration); (C) accept 3 s. Awaiting Boss direction.

## WARM-BOOT CRACKED — sidebar gated on per-library counts (2026-05-21)

Boss chose (A) the render gate, via his own "switch-off-and-measure" idea — executed as instrumentation, not feature-toggling.

**Definitive measurement** (MutationObserver on `.sidebar-content` + longtask recorder, written via a final report at the populating mutation): `sidebar_node_timeline: [[398,0],[2452,97]]` — sidebar DOM EMPTY (0 nodes) until 2452 ms, then populated. `boot_long_tasks: []` (no blocking), heartbeat max 108 ms (thread responsive). So **Svelte renders the sidebar LATE** (not a paint/present delay). Correlated with `load_all_stats_wall_ms: 1580`.

**Root cause**: the sidebar's library sections derive from `$libraryStats` (`ownLibraries` = `$derived($libraryStats.filter(...))`, `universeNotesStats`), which is populated only when `loadAllStats()` → `get_all_library_stats` (libraries.rs:382) finishes — a per-library NOTE-COUNT query against the cold 2.35 GB DB, ~1.5–3 s. The note list (`allNotes`) is ready at 0.86 s, but the sidebar waits on the COUNTS it doesn't need to draw.

**Fix** (`+layout.svelte`, after `libraries.set(bundle.libraries)`): seed `libraryStats` from `bundle.libraries` immediately (placeholder `star_count/folder_count/recent_stars`; badge hidden when 0); `loadAllStats()` enriches later. **Boss-confirmed: sidebar_populated_ms 2452 → 423 ms** (universe structure now appears at ~0.4 s; images show 18 libraries instantly). svelte-check 0 new errors. Diagnostic instrumentation reverted; only the seed committed.

**Remaining**: the note-counts (badges + "7,652 notes") still trail in at ~3.5 s via `get_all_library_stats` (cosmetic — app fully usable at 0.4 s). Boss chose to speed these up next (likely replace per-library counts with one grouped `COUNT(*) ... GROUP BY library` query). Tracked next.

## Counts from the index (2026-05-21) — boot finish line

`get_all_library_stats` (libraries.rs) stat-walked every library tree (~7,600 cold stat calls) + read preview files = the ~3.5 s counts-trail. Rewrote it to read from `note_meta`: `aggregate_library_counts` (sequential let-bindings to avoid the if-let MutexGuard lifetime trap) → `star_count` exact COUNT, `folder_count` = distinct ancestor dirs of notes under the library root, `recent_stars` dropped (verified unused). Removed 4 dead FS-walk helpers (count_contents/count_recursive/get_recent_notes/collect_recent_meta_recursive). cargo check clean. Commit `f616ce51`. **Boss-confirmed: structure + counts both at ~0.4 s.** Transient post-write boot slowdown (cache/WAL disturbed by the re-index) self-heals in 1–2 boots — Boss said leave it.

**Boot saga outcome (committed + Boss-validated)**: note-open ~5 s→instant (`b7e17603`); WAL 372 MB→0 + instant typing + 4 s→3 s (`a532eaeb`); NSC backfill manual-only (`a338d9e2`); warm-boot structure instant (`f1ddfa9e`); counts instant (`f616ce51`); orientation v2.21 (`6aa37e0d`) + v2.22 (this). Bundle-split tried + reverted (not the lever). term_vocab bigram shrink (PJ #26) parked: proven safe on a real-DB copy but only ~0.6 GB / 26% (overestimate corrected). PCS done; help docs (Cataloger + NSC ×15) next.
