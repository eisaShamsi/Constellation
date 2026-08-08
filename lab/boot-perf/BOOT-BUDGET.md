# Boot Performance Budget — Constellation

Status: **SHIP GATE** — a build that misses any single criterion below is considered non-shippable, regardless of which other work it contains.

These are the acceptance criteria defined by the expert panel (Obsidian internals, Tauri/Rust systems, PKM architecture generalist) convened on 2026-04-15 after the "endless patching" session. They are non-negotiable until the architecture work around them is ratified as complete.

## Reference target: Obsidian on the same hardware

Measured by the user on identical Windows hardware:
- UI visible: **< 5 seconds**
- Fully responsive: **< 10 seconds**
- No "Not Responding" banner, no audible disk thrashing.

## Test corpus

- Universe: the Trial Universe v1 (`lab/trial-universe/output/Constellation Discovery`)
- Scale: **7,600 notes across 16 libraries**, 656k typed links, 4k images.
- OS: Windows 11, NTFS, Defender real-time scan active (standard user environment).
- Boot type measured: **warm cache boot** — after at least one prior successful launch (the first-ever launch builds the cache and is measured separately).

## The five criteria

### 1. UI visible within **2.5 seconds**

"Visible" = the sidebar (library list) is painted and the user sees the last-open note's skeleton, not a blank window.
- Measurement: `performance.mark('boot:paint')` at first paint in `+layout.svelte` onMount.
- Recorded value: `boot-perf.latest.json#paint_ms`.

### 2. Fully responsive within **6 seconds**

"Responsive" = typing in the editor responds instantly, clicking any toolbar icon opens its pane, search returns results, no "Not Responding" banner.
- Measurement: Tauri event `boot:hydrated` emitted when the main cache snapshot has been applied to all reactive stores.
- Recorded value: `boot-perf.latest.json#hydrated_ms`.

### 3. Idle RSS memory ≤ **350 MB**

Measured 30 seconds after Criterion 2 passes, with no user input and no notes open.
- Measurement: `tauri::process_info::memory_rss()` sampled and written to `boot-perf.latest.json#rss_mb`.

### 4. Post-boot stat-sweep detects **50 externally-modified files in ≤ 3 seconds**, non-blocking

Procedure:
1. Close Constellation cleanly.
2. Run `touch-50.ps1` (provided) which updates mtime on 50 random `.md` files in the trial Universe.
3. Relaunch Constellation.
4. Measurement: time from `boot:hydrated` (Criterion 2) to `boot:reconciled` event with `changed: 50`.
5. During this window, the user MUST be able to type in the editor without any lag.
- Recorded value: `boot-perf.latest.json#reconcile_ms` AND `boot-perf.latest.json#reconcile_blocked_ui: false`.

### 5. Kill mid-index recovery

Procedure:
1. Delete `search.db`, `search.db-wal`, `search.db-shm` from `<universe>/.constellation/`.
2. Launch Constellation. The first-boot "Building index…" modal appears.
3. When progress reaches ~50%, force-kill the Tauri process.
4. Relaunch Constellation.
5. Expected: modal re-appears, resumes cleanly (or restarts), completes. **No duplicated note rows**, **no orphaned LINK rows**, **no WAL corruption error**.
- Measurement: after successful recovery, run `SELECT COUNT(*), COUNT(DISTINCT path) FROM note_meta`. They must be equal.
- Recorded value: `boot-perf.latest.json#recovery_pass: true`.

## How to run the test

Once the implementation work lands, the user:

1. Launches Constellation with the trial Universe active.
2. Waits ~10 seconds.
3. Opens **Settings → Debug → Boot Performance Report**.
4. Sees a 5-row scorecard: each criterion with PASS / FAIL and the measured value.
5. Criteria 4 and 5 have "Run now" buttons that execute the procedure automatically.

## Production baseline (post-fix, commit `ce47e13`)

Reported by user, 2026-04-15. Measured against the trial Universe
(7,600 notes, 16 libraries, 656k typed links).

| Criterion | Target | Measured (production .exe) | Status |
|---|---|---|---|
| 1. UI visible | ≤ 2.5s | **~1s** | **PASS** |
| 2. Fully responsive | ≤ 6s | **~8s** | NEAR (1.3× target) |
| 3. RSS ≤ 350 MB | ≤ 350 MB | not yet measured | TBD |
| 4. Stat sweep 50 files | ≤ 3s non-blocking | not implemented | future |
| 5. Kill-mid-index recovery | pass | not implemented | future |

For comparison, Obsidian on the same hardware reports < 5s UI / < 10s
fully responsive. Constellation production now beats UI and is within
1.3× of fully-responsive on the largest test corpus we ship today.

## Pre-fix baseline (commit `69cfa6a`, for reference)

| Criterion | Target | Measured | Status |
|---|---|---|---|
| 1. UI visible | ≤ 2.5s | 25s | FAIL |
| 2. Fully responsive | ≤ 6s | > 360s (user terminated) | FAIL |

The fix series (commits `a76a717` through `ce47e13`) reduced UI-visible
time by 25× and fully-responsive time by 45×.

## Open work (Criteria 4 and 5)

- **Criterion 4 — detection half SHIPPED (PJ-207 §9, 2026-08-08).** The boot
  reconcile (already walking every own library each launch) now compares each
  `.md`'s mtime against `note_meta.modified` and surfaces the result as a
  user-facing notice ("N notes changed on disk while Constellation was
  closed…"). Measured in the shipped code: the whole walk is **17–19 ms warm on
  7,964 notes** (the §9 commit also replaced `path.is_dir()` with
  `entry.file_type()`, taking the walk from ~255 ms — so Criterion 4's ≤ 3 s
  budget is met by ~150×). Boss-validated live: an external Notepad edit
  raised the reported count 19 → 20 on the next launch. **Still open from the
  original spec:** the formal `touch-50.ps1` / `boot:reconciled {changed: 50}`
  harness and the `reconcile_ms` / `reconcile_blocked_ui` fields in
  `boot-perf.latest.json` are not wired; the Settings scorecard row still
  reads "Not measured".
- **Criterion 5** — kill-mid-index recovery: schema-version check on
  search.db with auto-rebuild modal if WAL is corrupt. Not implemented.

These do not gate the current "boot-perf-fix-2026-04-15" milestone but
must land before we can claim full ship-readiness on this dimension.

## Changelog

- **2026-08-08 (Criterion 4 detection)** — PJ-207 §9 ships the stat-only
  drift check inside the existing boot reconcile, with a user-facing notice.
  Formal measurement harness still open (see above). Also recorded for
  honesty: the corpus drive `E:` is a **USB mechanical HDD** (LaCie d2), not
  an SSD — cold-cache walks measured 3.5–8.7 s vs 17–19 ms warm, so every
  boot-perf figure needs a cold/warm label (§9.5 of the orientation).
- **2026-04-15 (initial)** — Budget created after 3-agent expert panel
  review. Pre-fix baseline recorded.
- **2026-04-15 (production verified)** — Fix series complete. Production
  build measured at 1s UI / 8s fully responsive on the trial Universe.
  Dev-mode IPC remains slow due to Vite + WebView2 + DevTools attachment
  overhead — not a shipping concern.
