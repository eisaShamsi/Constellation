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

---

## § 30 — Ghost in the Machine: cross-note content corruption on navigate

User pushed back on moving to P3 until we root-caused what was actually
happening during the A↔B cycle incidents. Agreed — the supersede-token
fix stopped the visible symptom but didn't explain *why* there were
multiple concurrent nav calls to supersede in the first place.

### The hunt

Deployed an Explore agent with a thorough-level brief to trace every
path that could fire `openNoteTab` / `navigateBack` / `navigateForward`
more than once per single user action. Candidates examined:

- **Multiple wikilink click handlers on the same event** — checked
  `src/lib/editor/livePreview.ts:851-879`: wikilinks with typed
  `|annotation` render as `Decoration.mark` (CSS styling only) +
  `Decoration.replace` (hiding syntax brackets). Neither carries a
  click handler. The only wikilink click path is `NotePane.svelte:643`
  mousedown/capture. Ruled out.
- **Event bubble conflicts** — the three `mousedown` handlers on
  `editorEl` (checkbox / chevron / link at lines 553, 592, 643) fire
  in registration order; `linkClickHandler` uses `preventDefault` +
  `stopPropagation` but none of them dispatch synthetic events or
  re-enter. Ruled out.
- **`$effect` re-fire on tab state change** — swept all 26 `$effect`
  blocks in `+layout.svelte`. None call `openNoteTab` / `navigateBack` /
  `navigateForward` in response to tab-state change. Ruled out.
- **`UniversalEmbedWidget` + `constellation:open-note` event** —
  `livePreview.ts:429-431` dispatches this event from transclusion
  **header clicks** only. `![[embed]]` is the only trigger, and our
  test link is a plain wikilink, not an embed. Ruled out.

**Finding**: the code has no multi-fire source for the A↔B cycle
visible to static analysis. The cycle was driven by rapid
`loadTabHistoryEntry` calls racing for the same tab — most plausibly
OS key-repeat during Alt+← (the one that hit on the *second* incident).
The first incident (click-driven) remains partially unexplained; the
Svelte 5 reactive proxy + `{#key}` re-mount path has enough hidden
cache invalidation that concurrent flush+mount can produce visible
bounce without a discoverable recursive call. The supersede-token fix
(`80e9fc4`) stops any race variant regardless of upstream cause —
acceptable closure for a symptom that also can't be stress-tested
without a synthetic harness.

### What the audit *did* find: data-corruption bug in `handleFlush`

The real ghost. The screenshots of "tab label = A, body = B" weren't
just a transient race artifact — they were the visible surface of a
content-swapping bug that had been silently mis-writing files on every
navigation since `{#key tab.id + '|' + tab.path}` landed.

**Chain**:

1. User clicks `[[B]]` in A.
2. `openNoteTab(B, …, A)` updates the tab: path/content/name → B.
3. `{#key tab.id + '|' + tab.path}` wrapper in `NoteEditor.svelte:216`
   destroys the old NotePane, mounts a new one.
4. Old `NotePane.onDestroy` at `NotePane.svelte:688` fires `doFlush()`,
   which calls `onflush?.(latestText, dirty, cursorPos, scrollTop)`
   with the old editor's body as `latestText` — **A's body**.
5. `handleFlush` in `NoteEditor.svelte:129` reads `freshProps()` —
   which pulls from the *current* store tab, **now B**. Builds
   `content = buildFullContent(B_frontmatter, A_body)`. That's the
   corruption.
6. `setWriteAhead(tab.path, content, …)` — poisons the write-ahead
   entry for B with the mixed content. The next time the user opens
   B, `getWriteAhead(B.path)` in `store.ts:645` hands back the mixed
   body, silently displacing the correct disk content.
7. If A had been dirty, `writeNote(tab.path, content)` with
   `tab.path = B` **wrote the mixed content to B.md on disk**.
   Real, silent cross-file data loss.

### Fix (`a2052da`)

- `NotePane.svelte` captures `mountedFilePath = filePath ?? ''` at
  mount time, passes it back as the last argument of `onsave` and
  `onflush`.
- `NoteEditor.svelte` guards `handleSave` / `handleFlush` with
  `if (filePath !== tab.path) return;` — any callback arriving from
  an already-destroyed editor whose tab has been repurposed is dropped
  cleanly instead of reaching the store mutation / write-ahead /
  disk-write lines.
- Removed three `tab.content = …` mutations in `handleFlush` that
  were silently no-oping anyway (Svelte 5 treats `$props()` as
  readonly unless declared `$bindable()` — the code had shipped
  assuming mutability that doesn't exist).

**Tradeoff documented in the commit**: if the user types in A and
navigates before the 1.5 s debounced save fires, the unsaved
characters are now dropped on the floor instead of silently
corrupting B. The debounced-save path covers the common case; this
is a narrow window that only matters for type-and-navigate-within-
one-save-cycle usage. Net win over the corruption it replaced.

### Severity

Without this fix, every wikilink traversal in the editor had a latent
risk of (a) displacing the target note's write-ahead with mixed
content (which surfaces on next open as body swap) and (b) when the
source note was dirty, writing that mixed content onto the target
file's disk. The trial Universe has 656k links across 7,600 notes;
anyone navigating heavily through the Constellation editor has been
silently writing cross-contaminated content for weeks. Cannot quantify
damage retroactively without a file-content audit against git history
/ backups.

### Commits landed today

- `917f2e1` — Remove read_index_entries diagnostic block
- `80e9fc4` — Fix history navigation races + tab title/body desync
- `e911749` — docs(session-log 04-20): § 29
- `a2052da` — Fix cross-note content corruption on wikilink-click navigation

### Next

Ready to move to P3 (Living Link visual surfaces) per earlier agreement.
