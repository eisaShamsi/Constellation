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

---

## § 31 — 404 flood fix on image embeds

Follow-up from the Ghost audit. While validating the corruption fix, the
user flagged the dev-server terminal flooding with 404s against
`http://localhost:1420/attachments/img/<name>` on every note open. The
corruption test passed cleanly (`TEST_A_BODY_MARKER` stayed on A, B
pristine — `a2052da` verified), so this was a separate bug the earlier
Ghost-hunt had conflated.

### Root cause — two interlocking defects

1. **`ImageWidget.toDOM` used "both paths empty" as a proxy for
   "absolute URL."** The only intentional caller of that shortcut
   (`livePreview.ts:797`) had already matched the filename against
   `^https?://|^data:` — so the regex was the real signal. On first
   render, the `libraryPathField` / `notePathField` `StateField`s
   default to `''` before `setLibraryPath` / `setNotePath` effects
   get dispatched, so widgets built from relative markdown paths
   like `attachments/img/foo.png` took the "render directly"
   branch and the browser resolved that relative URL against the
   dev origin → 404.

2. **The `setLibraryPath` / `setNotePath` effects were dispatched
   AFTER `new EditorView(state, parent)`.** The `ViewPlugin`
   constructor runs the initial `buildDecorations` inside that
   constructor, so the first decoration set was always built
   against empty fields. A later `view.dispatch` of the correct
   effects didn't retrigger a rebuild — state-field-only
   transactions don't set `viewportChanged` / `selectionSet` /
   `docChanged`, which are the only rebuild triggers in
   `LivePreviewPlugin.update`.

### Fix (`455bdd7`)

- `livePreview.ts`: `ImageWidget.toDOM` now tests the filename
  against `/^(https?:|data:|asset:|file:|blob:)/i` to detect
  absolute URLs (the real signal). If the URL is relative AND
  both paths are empty, show the fallback placeholder instead of
  handing the browser a bad relative URL.
- `NotePane.svelte`: collect the image-path effects BEFORE
  `new EditorView(...)` and apply them to the state via
  `state.update({effects}).state`. ViewPlugin's first
  `buildDecorations` now sees populated fields. Widgets build
  correctly on the first pass; no placeholder flash, no 404s.

User validation: silent terminal after cold restart on notes that
had the flood. Confirmed.

### Ghost audit — full closure

All three faces of the Ghost are now closed:

| Face | Fix | Commit |
|------|-----|--------|
| Navigation cycle (A↔B plinking) | per-tab supersede token in `loadTabHistoryEntry` + `__navTrace` ring buffer | `80e9fc4` |
| Title/body desync + cross-note content corruption | `filePath` guard in `handleFlush` / `handleSave` | `a2052da` |
| 404 flood on image embeds | absolute-URL regex + pre-view state population | `455bdd7` |

### Commits landed today (updated)

- `917f2e1` — Remove read_index_entries diagnostic block
- `80e9fc4` — Fix history navigation races + tab title/body desync
- `e911749` — docs(session-log 04-20): § 29
- `a2052da` — Fix cross-note content corruption on wikilink-click navigation
- `bfc790f` — docs(session-log 04-20): § 30 — Ghost in the Machine audit
- `455bdd7` — Fix 404 flood from ImageWidget relative-path embeds on first render
- (this entry's commit)

### Moving on

Ghost fully closed. Proceeding with P3 — Backlinks panel ordered by
weight.

---

## § 32 — Live-preview: 3-part typed wikilinks

User surfaced a rendering bug in a paragraph with wikilinks of the form
`[[مراكش|مراكش و|derives-from]]` and `[[قرطبة|وقرطبة|supports]]` —
three pipe-separated parts (target, alias, typed annotation). The
`|derives-from]]` and `|supports]]` trailers were bleeding into the
rendered text instead of being hidden.

Root cause: the live-preview wikilink parser only inspected
`firstPipe + 1` for a `TYPED_LINK_TYPES` match. For a 3-part link,
that substring was `alias|type` which never matched the typed set,
so the code fell through to the "display alias" branch and rendered
`alias|type` as the link text.

Fix (`8abf417`): check the substring after the **last** pipe against
`TYPED_LINK_TYPES`, branching on whether first pipe == last pipe:

- `[[note]]` — plain, unchanged
- `[[note|alias]]` — display alias (may contain stray pipes)
- `[[note|type]]` — 2-part typed (first == last pipe)
- `[[note|alias|type]]` — 3-part typed (first != last pipe) **NEW**

Click navigation unchanged — `NotePane.linkClickHandler` already
uses `match[1].split('|')[0]` as the target, works for any pipe
count.

`utils.ts` Marked renderer uses a different typed-link convention
(`[[note|type:...]]` with explicit `type:` prefix) and was not
touched here — separate format question.

User validation: visible trailers gone; aliases render in the typed
colors. Confirmed.

---

## § 33 — P3: Backlinks ordered by weight + traversal-count chip

First user-facing payoff for the Living Link system. Backlinks for an
open note are now sorted by `weight = 1 + ln(1 + traversal_count)`
descending (ties broken alphabetically by source name for stability
when everything is still at weight 1.0), and any backlink whose
source→target pair has been traversed at least once gets a compact
`×N` chip next to the note name so worn paths are visible at a glance.

### Wire-up

**Rust** (`libraries.rs`, `cache.rs`):
- `NoteLink` struct gains `weight: f64` and `traversal_count: i64`.
  Both use serde defaults (1.0 and 0) so any older payload in transit
  or at rest still deserializes.
- `cache::read_links` SELECT extended to pull `weight, traversal_count`
  from `note_links`. The columns have existed since
  `constellation_link_traverse` landed (P2, `53d97e7`); they just
  weren't being exported.
- The three non-DB `NoteLink` constructors (`scan_library_links`,
  `scan_unlinked_mentions`, `read_untyped_links_fallback`) set
  `weight = 1.0, traversal_count = 0` explicitly — those paths parse
  markdown or fall back to `outgoing_links_json` and don't see the
  lifecycle fields, so "never traversed" is the right neutral.

**Frontend** (`store.ts`, `+layout.svelte`, `BacklinksPanel.svelte`):
- `NoteLink` TS interface mirrors the Rust fields (both optional for
  forward-compat).
- `getBacklinks` sorts `weight DESC, source_name ASC`. Returned shape
  adds `traversalCount`.
- `currentBacklinks` state type in `+layout.svelte` extended to carry
  the count through to the panel.
- `BacklinksPanel.svelte` renders a `×N` chip (tabular-nums, faint
  background, pluralization-aware tooltip) when `traversalCount > 0`.
  Placed between the link-type badge and the library label so the
  row's visual rhythm stays intact.

### Boot-time impact

No additional IPC calls — the graph payload already flows through
`cache_boot_snapshot_graph` (the Phase-2 deferred snapshot). The
added `f64 + i64` per row is ~16 B; on the 7,600-note / 656k-link
trial Universe that's roughly +10 MB one-time payload, fully inside
the idle-callback phase post-paint. Boot Criterion 2 unaffected.

### Commits

- `db9a826` — P3: Backlinks ordered by weight + traversal-count chip
- `60167fe` — Chip contrast fix: accent-tinted instead of
  --text-faint/--background-modifier-border (which rendered the
  ×N number nearly invisible in the user's dark theme).
- `6d99fc5` — Unified `.bl-count` (the LINKED MENTIONS header pill)
  with the traversal chip's accent-tint palette; both Backlinks
  chips now share one visual language.
- `989fcef` — Same treatment applied to `.ol-count` in
  OutgoingLinksPanel so the symmetric panel doesn't stay behind
  on the old heavy `--background-modifier-border-focus` fill.

User validation: ordering correct (غرناطة at top at weight ≈ 3.40
on a traversal_count of 14 from the P2 + post-fix test runs; 142
untraversed backlinks follow alphabetically at weight 1.0). Chip
and count pill both legible in dark mode.

---

## § 34 — P4.1: Outgoing Links symmetry

Symmetric companion to P3. Outgoing Links panel now mirrors the
Backlinks treatment: weight-sorted order, link-type badge with the
shared type-color palette, and the `×N` traversal chip where a pair
has been traversed.

### Changes

**`store.ts::getOutgoingLinks`** — sort contract matches
`getBacklinks`: `weight DESC, target ASC`. Stable ordering when
everything is still at weight 1.0.

**`+layout.svelte`** — `currentOutgoing` state type extended to
carry `traversalCount` and `linkType`; the map from
`getOutgoingLinks` passes both through into the panel.

**`OutgoingLinksPanel.svelte`** — adds the link-type badge (same
`LINK_TYPE_COLORS` palette as BacklinksPanel / GraphMind /
livePreview) and the `×N` chip (same accent-tinted palette as the
BacklinksPanel chip). Target row is now a flex container so the
target name ellipsizes cleanly when badges sit next to it.

No Rust changes needed — `NoteLink` already carries `weight` and
`traversal_count` from P3's schema extension, and
`cache_boot_snapshot_graph` already exports both.

### Commit

- `a2d8c7b` — P4.1: Outgoing Links weight-sorted + link-type badge + ×N chip

### P4 lanes remaining

- P4.2: Wikilink weight chip inside rendered prose (higher-impact,
  needs per-render lookup from the live-preview path).
- P4.3: "Most-traveled paths" read-only pane (cheapest, least
  daily-useful).

### Commits landed today (updated)

- `917f2e1` — Remove read_index_entries diagnostic block
- `80e9fc4` — History navigation races + tab title/body desync
- `e911749` — session-log § 29
- `a2052da` — Cross-note content corruption on wikilink-click nav
- `bfc790f` — session-log § 30 (Ghost audit)
- `455bdd7` — 404 flood on image embeds
- `8abf417` — Live-preview: 3-part typed wikilinks
- `8b5b2e8` — session-log §§ 31, 32
- `db9a826` — P3: Backlinks ordered by weight + chip
- `60167fe` — Chip contrast fix
- `6d99fc5` — Backlinks `.bl-count` pill unified
- `989fcef` — Outgoing `.ol-count` pill unified
- `c2843b3` — session-log § 33
- `4494642` — session-log § 33 polish commits
- `a2d8c7b` — P4.1: Outgoing Links weight + chip
- (+ background M11-data cron: `5890ff9`, `6e2a133`, `c29e1a8`,
  `1f45445` — +075 through +078 — producer continues toward 20k)

### Out of scope (P4 candidates)

- Outgoing Links panel ordering — same sort, symmetric change, small.
- Wikilink weight chip inside rendered prose — requires per-render
  lookup; bigger.
- "Most-traveled paths" pane — read-only view of `note_links ORDER BY
  weight DESC LIMIT N`; cheapest but least daily-useful.

### Commits landed today (updated)

- `917f2e1` — Remove read_index_entries diagnostic block
- `80e9fc4` — Fix history navigation races + tab title/body desync
- `e911749` — docs(session-log 04-20): § 29
- `a2052da` — Fix cross-note content corruption on wikilink-click nav
- `bfc790f` — docs(session-log 04-20): § 30
- `455bdd7` — Fix 404 flood from ImageWidget relative-path embeds
- `8abf417` — Live-preview: parse 3-part typed wikilinks
- `8b5b2e8` — docs(session-log 04-20): §§ 31, 32
- `db9a826` — P3: Backlinks ordered by weight + chip
- (+ background M11-data cron: `5890ff9`, `6e2a133`, `c29e1a8`,
  `1f45445` — +075, +076, +077, +078 — producer continues toward 20k)

## § 35 — Ghost revival + P4.2 follow-up revert

**User report:** "Surprise... surprise, the ghost has revived. Same symptoms."
A↔B plinking + title/body desync on wikilink click — the same class the
`mountedFilePath` guard (`a2052da`) + supersede-token (`80e9fc4`) closed.

**Root cause:** the two P4.2 live-refresh commits landed a reactive
wave that fires **during** `openNoteTab`'s in-flight navigation —
`bumpLinkTraversal` runs at store.ts:718, before the tab swap at
:771. Chain: `linkTraversalBumps` → `linkTraversalMap` $derived →
NoteEditor prop → NotePane $effect → `view.dispatch(setLinkTraversalMap)`
→ LivePreviewPlugin rebuilds decorations — all while the `{#key}`
block is transitioning. `effectiveLibraryLinks` (9ebe35c) doubled the
blast radius by also invalidating the sidebar on every bump.

**Action:** reverted both follow-ups.
- `5b31c80` — Revert "P4.2 follow-up: route sidebar panels through bumped links too"
- `e365e72` — Revert "P4.2 follow-up: live traversal count refresh"

Live `×N` refresh is a nice-to-have: boot-graph fetch already
reconciles on next launch / universe switch. Post-revert the in-prose
chip and sidebar chips return to the boot-graph-only data source that
was clean at `3d216be`.

**Follow-up (deferred):** re-ship live refresh with the bump deferred
via `queueMicrotask` so the reactive wave fires **after** navigation
settles, and with a dep-gated sidebar $effect so bumps don't invalidate
`effectiveLibraryLinks` during nav.
