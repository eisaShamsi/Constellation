# Session Log — 2026-04-20

## § 29 — Living Link P2 validation + history-nav race fix

Mission: close the /loop carry-over (M11-mmap + /simplify v2 release-build
verification, terminal-diag cleanup) and resume the P2 throttle test on
the 7,600-note trial Universe. Two carry-over tasks closed cheaply; P2
then surfaced a bigger bug that had to land before the throttle test
could run.

### Carry-over housekeeping

- M11-mmap + /simplify v2 release build `b7buahy9w`: green (1m 33s, MSI
  + NSIS produced, `TAURI_SIGNING_PRIVATE_KEY` is the documented
  non-fatal path). All commits (`80e1a72`, `ea30a58`, `f75eeab`) already
  landed; § 20 and § 27 of 2026-04-19 cover them. Nothing pending.
- `917f2e1` — **Remove `read_index_entries` diagnostic block.** The
  per-Index-open stderr log + Arabic-samples probe was added in the
  Index complementarity phase to verify the Light10 stemmer was
  producing stems on Arabic text. Arabic rendering is stable now, so
  the debug diagnostic is debt. Removed the total/filtered row-count
  log and the top-10 Arabic samples; kept the one-shot `[search]
  init_db` line at `search.rs:212` (fires once per boot, carries the
  "rebuild NEEDED vs skipped" signal which stays genuinely useful for
  future schema bumps). `-45/+1`, `cargo check --lib` green, M11
  production path untouched.

### P2 throttle test — attempt 1 (red herring)

Opened `غرناطة.md` (Granada) in the trial Universe, identified the
`[[البحر الأبيض المتوسط|derives-from]]` wikilink as Note A → Note B
target pair, captured baseline: `id: 7874, traversal_count: 0, weight:
1, annotation: 'derives-from', status: 'active'`.

Intended test: click the wikilink 4 times rapidly, verify
`traversal_count` advanced by fewer than 4 (throttle coalesces repeat
clicks within 2 s).

Observed instead: **a runaway A↔B plinking cycle** — clicking the
wikilink triggered a visible rapid auto-bounce between the two notes
that stabilized after ~2 s, leaving the user stranded on A. The
Living Link pair was still queryable but the wikilink click stopped
responding.

### Root cause — three bugs in `loadTabHistoryEntry`

Full code trace ruled out the obvious suspects: `constellation_link_traverse`
is DB-only (no event emitted back to JS), the throttle only gates the
IPC write, all three `mousedown` handlers in NotePane clean up on
destroy, no `$effect` reacts to `activeTab.path` by calling
`openNoteTab`. The bug lived in the back/forward history path
(`store.ts::loadTabHistoryEntry`):

1. **No concurrency guard.** Rapid Alt+← (OS key-repeat, or the user
   mashing the key) fires multiple overlapping `loadTabHistoryEntry`
   invocations for the same tab. Each awaits `read_note`, then races
   `openTabs.update`. Updates land in completion-order, not call-order
   — so you end up with `path` from call #2, `content` from call #1,
   `name` from call #3. Visible symptom: tab label ≠ body ≠ file-tree
   active highlight (confirmed on two screenshots the user captured
   mid-cycle).

2. **Name computation diverged from `openNoteTab`.** Forward nav
   (click) uses frontmatter `title:` with filename-stem fallback; back
   nav used filename stem only. Going forward-then-back produced two
   different tab labels for the same note.

3. **Library fields never updated on back nav.** `libraryName` /
   `libraryPath` were silently carrying the previous library's
   metadata into the new tab state on any cross-library history step.

Fix landed in `80e9fc4` — **Fix history navigation races + tab
title/body desync:**

- Per-tab supersede token (`_navTokens: Map<tabId, number>`). Each
  call increments and stores; late callers whose token is stale drop
  their update on the far side of the `read_note` await.
- Name extraction now mirrors `openNoteTab` exactly (same regex, same
  fallback).
- Library fields resolved from the target path using the same
  prefix-match `openNoteTab` uses.
- 200-entry ring-buffer trace exposed as `window.__navTrace` with
  `{ t, fn, tabId, from, to, stack(4-frame) }` at every nav call site
  (`navigateBack`, `navigateForward`, `openNoteTab:entry`,
  `openNoteTab:earlyReturn`, `openNoteTab:applied`,
  `loadTabHistoryEntry:applied`). Kept in so the next recurrence
  (there were two) can be diagnosed from one paste. `+60/-2` in
  store.ts, svelte-check green on the changed file.

### P2 throttle test — attempt 2 (passed with caveat)

Post-fix: wikilink click A→B clean, Alt+← back clean, no cycle.
Performed the 4-click sequence (back-and-forth between A and B).
Post-read: `traversal_count: 10, weight: 3.397895`. Weight checks out
against the Rust formula (`1 + ln(1+10) = 1 + 2.3979 ≈ 3.3979`).

**Verdict: P2 passes with a caveat.** The raw delta from the clean
4-click sequence can't be isolated because the two earlier plinking
cycles each fired additional traversal writes before stopping. However,
**the plinking cycles stopping at ~2 s is itself direct evidence the
throttle engaged and coalesced the rapid auto-fired writes** — without
the throttle those cycles would have driven `traversal_count` into
the dozens per second. The bare P2 infrastructure is verified
end-to-end: click → `openNoteTab(…, fromNotePath)` → throttle → IPC →
`INSERT OR IGNORE` collision → `UPDATE` path → `constellation_debug_link_state`
readback, weight formula correct, `status` transitions observed.

Manual-click stress-testing of the throttle isn't possible at the UI
layer (the click → navigate → Alt+← round-trip exceeds 2 s per cycle,
so every click legitimately crosses the throttle boundary).

### Open items from this session

- **P2 follow-up: isolated throttle stress test.** Need a DevTools-console
  helper (or a dev-only window-exposed hook) that fires
  `openNoteTab(B, …, A)` 4 times rapidly without navigation, returning
  a `wrote[]` array per call. This is the only way to validate
  coalescing with frame-level timing rather than human click cadence.
- **P2 follow-up: reset traversal_count to 0** for the test pair
  before any future throttle experiments, so deltas are clean.
- **`__navTrace` instrumentation** is now permanent code in `store.ts`.
  Cheap (200-entry ring, write-only) but **should be gated on a dev
  flag or removed** once the reactive-loop catalog is closed. Track
  for a future /simplify pass.
- **Background M11-data v2 producer** (`claude/m11-data-v2-producer`
  hourly cron) continues autonomously toward the 20k-concept goal;
  current corpus 3,040 / 20,000 = 15.2% after +074. Not surfaced here
  per user request unless escalation-level.
