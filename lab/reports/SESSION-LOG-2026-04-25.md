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
