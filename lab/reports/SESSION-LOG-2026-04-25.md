# Session Log — 2026-04-25

## §112 — BUG-012 fix: title-leak via mountedFilePath staleness guard

**Commit**: `952f3c7`
**Files**: `src/lib/components/NotePane.svelte`, `src/lib/components/NoteEditor.svelte` (+20 / −5)

### What broke

Discovered during MIG-006 §1 testing. User attempted to replace
`Link me to [[§2 Round3]]` with `Link me to [[§2 Round3_v3]]` in
`§2 Round5`'s body. The editor jumped to the target (`§2 Round3_v3`),
but the target's frontmatter title was rewritten to the source's
(`§2 Round5`), and the target's prior titles (`§2 Round3`,
`§2 Round3_renamed`, `§2 Round3_v3`) ended up stamped into its
aliases. Two notes (`NOTE_531D` and `NOTE_EE1E`) ended up with the
same `id='§2 round5'`. Same class as BUG-002/003 (display-sync
race), but on the *write* path, not the display path.

### Root cause

When a `[[wikilink]]` click reuses the active tab in-place,
`openNoteTab` swaps `tab.path` / `tab.name` to the target, the
`{#key tab.id + '|' + tab.path}` block in `NoteEditor.svelte`
remounts `NotePane`, and the OLD `NotePane` runs `onDestroy`. A
final blur on the OLD title `<input>` fires `ontitlechange` —
but `ontitlechange(titleValue)` was emitted with no snapshot of
which file this title belonged to. `handleTitleChange` then read
the LIVE `tab.path` (now the TARGET) and called
`renameItem(target_path, source_title.md)`. In canonical mode
that runs `update_frontmatter_title` on the target file: title
overwrite + alias stamp.

`handleSave` / `handleFlush` already had a `mountedFilePath`
snapshot + `filePath !== tab.path` guard for exactly this race
on the body-write path. The title-write path lacked the matching
guard.

### Fix

- `ontitlechange` signature widened to `(newTitle, filePath)`.
  `NotePane.handleTitleBlur` now passes its `mountedFilePath`
  snapshot.
- `NoteEditor.handleTitleChange` bails when
  `filePath !== tab.path`. All file-path uses inside the handler
  read the snapshot, not the live `tab.path`, so even if the
  guard were ever loosened the rename target couldn't be
  misdirected.

### Verification

- `svelte-check`: 0 errors, 284 warnings (unchanged).
- User test: `[[§2 Round3]]` → `[[§2 Round3_v3]]` swap in
  `§2 Round5`. Editor jumps. Target title remains
  `§2 Round3_v3`, target aliases unchanged, source `§2 Round5`
  unchanged. **Pass.**
- Cosmetic follow-up logged: sidebar's "active item" highlight
  lags ~10s behind the navigation. Benign — no disk write, no
  reactive corruption — but worth a separate cleanup pass after
  MIG-006 closes.

### Why this matters for MIG-006

MIG-006 rewrites wikilinks across notes. Until this guard
landed, every `[[wikilink]]` typed during cascade testing risked
silently mutating the target's frontmatter title — turning a
benign rename test into a duplicate-id corruption that took
manual SQL surgery to recover from. With BUG-012 fixed, MIG-006
§1 verification can resume safely.

### Pending after this commit

- Resume MIG-006 §1 GUI verification.
- MIG-006 §2-§11.
- MIG-005 (alias-aware in-memory inbound consumers, deferred
  per user direction).
- MIG-002 §7-§10 (enrichment_worker drain loop).
- MIG-003 (human-name filenames).
- Cosmetic: sidebar active-item sync lag (~10s after wikilink
  navigation).

---

## §113 — MIG-006 §1 GUI verification + plan §3 expansion

**Files**: `lab/reports/MIG-006-WIKILINK-CASCADE.md` (plan update only — no code changes this commit)

### §1 verified working

After BUG-012 unblocked safe wikilink testing, ran §1 (lift `oldName`
from frontmatter) end-to-end through the GUI:

1. Source `§2 Round5` body edited to `Link me to [[§2 Round3_v5]]`,
   flushed via tab switch.
2. Target renamed via file tree: `§2 Round3_v5` → `§2 Round3_v6`.
3. Disk re-read confirmed body now `Link me to [[§2 Round3_v6]]`.

The walker received correct args (`old_name="§2 Round3_v5"`,
`new_name="§2 Round3_v6"`), visited every `.md` in the universe,
and rewrote the matching source — proving §1's `oldName` lift
fixed the call site. Verified via temporary Rust + JS
instrumentation (cascade-debug.log written to `%TEMP%`); both
instrumentation patches reverted before commit so the working tree
is back to the §111 state for app code.

### BUG-013 — open-editor cascade race (separate from §1 itself)

User-reported during verification: cascade fails when the source
note is open and the user renames the target without first
switching tabs. Three races identified:

1. **Pre-cascade staleness** — source's in-memory edits not yet
   flushed to disk; walker reads stale text.
2. **Post-cascade stomp** — walker rewrites disk; source tab's
   next autosave (with pre-cascade in-memory text) overwrites
   the cascade's update, silently undoing it.
3. **Watcher loop** — walker's `fs::write` bubbles back through
   the file watcher as an external edit, racing the editor's
   read-back.

The original MIG-006 §3 plan ("Rust-side `recent_writes`
suppression for the file watcher") only addressed (3). All
three must be solved or the cascade is unreliable for the
realistic usage pattern (user is reading/editing a note,
realises the linked target needs a rename, renames it).

### Plan update — §3 expanded

`MIG-006-WIKILINK-CASCADE.md` updated:

- §3 row in Phase 2 plan table widened to:
  *"Open-editor coherence: flush-before-cascade +
  reload-after-cascade + `recent_writes` watcher suppression"*.
- §3 detailed section rewritten to specify three coordinated
  pieces:
  - **(a) Flush-before-cascade (frontend)**: new helper
    `flushAllTabsInLibrary(libraryPath)` in
    `src/lib/libraries/store.ts`, awaited in
    `+layout.svelte::handleRenameComplete` before
    `updateLinksOnRename`.
  - **(b) Reload-after-cascade (Rust→event→frontend)**:
    `update_links_on_rename` returns `CascadeResult` with
    `rewritten: Vec<String>`; emits `cascade:rewrote { paths }`
    Tauri event; frontend listener re-reads each rewritten
    path's content into the open tab's `content` field while
    preserving cursor/scroll/history.
  - **(c) Watcher suppression (Rust)**: original §3 content
    moved here unchanged.
  - **Input block during cascade**: editor `editable.of(false)`
    between flush-start and reload-complete (5 s safety
    timeout) so no keystroke is lost in the flush→write→reload
    window.
- Verification clause now: "open source, rename target without
  clicking away, wikilink updates in-place, cursor/scroll
  preserved, no autosave reverts it."
- All three pieces ship as one cohesive commit — they only make
  sense together.

### Why this matters

The cascade as it stands today (post-§1) works *if the user
remembers to switch away from the source before renaming the
target*. That's an unacceptable UX rule — it's the kind of
hidden constraint that turns a feature into a bug report.
Expanded §3 makes the cascade fire reliably regardless of which
tab the user happens to be looking at.

### Pending after this commit

- MIG-006 §2 (walker correctness pass — regex, transcludes,
  link-type preservation).
- MIG-006 §3 expanded (per the rewritten plan).
- MIG-006 §4-§11 unchanged.
- BUG-014 logged: `NOTE_EE1E` has an orphaned `cid_cn` value
  pointing at `NOTE_531D` from BUG-012's manual recovery —
  cosmetic only (cid_cn is never used as an identity key
  outside legacy migrations) but should be cleaned up.
- Cosmetic: relocate "Auto-update links on rename" toggle from
  Sky View & Links to Knowledge Management.
- Cosmetic: sidebar active-item sync lag.
