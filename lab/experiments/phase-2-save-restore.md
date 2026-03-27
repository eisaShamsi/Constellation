# Experiment: Phase 2 — Save & Restore

## Hypothesis
Notes persist across tab switches and app restarts with zero data loss. Cursor and scroll positions are preserved. Save is background-only with no typing interference.

## Spec Reference
- Section 2.2: No Store Updates During Typing
- Section 4.2: Save Is Background-Only
- Section 4.3: Tab Switch = Component Recreation

## Implementation

### Architecture: No IPC During Typing
After 13 rounds of testing, we discovered that ANY Rust IPC call (`invoke`) during or near typing causes perceptible lag. The final architecture eliminates all disk writes from the typing path:

- **During typing:** Content stays in JS memory only (`latestText` non-reactive variable)
- **Tab switch/close:** `onflush` fires in `onDestroy` — writes to Write-Ahead Buffer (WAB) + async disk write
- **App losing focus:** `visibilitychange` listener triggers `doSave()`
- **Periodic idle save:** Every 30s via `setInterval` + `requestIdleCallback`
- **App close safety net:** `beforeunload` handler triggers `doFlush()`

### Write-Ahead Buffer (WAB)
Synchronous in-memory Map + localStorage backup. Ensures content, cursor, and scroll survive:
- Tab close + reopen (in-memory buffer)
- App crash/restart (localStorage backup)
- Async disk write not completing in time (buffer checked before disk read)

### Key Functions
- `doSave()` — calls `onsave` (lightweight disk write via `writeNote`)
- `doFlush()` — captures content + cursor + scroll from EditorView, calls `onflush`
- `setWriteAhead()` / `getWriteAhead()` / `clearWriteAhead()` — WAB with localStorage persistence
- `markRecentWrite()` — prevents file watcher from re-reading our own writes

### Save Flow
```
Typing → latestText updated (non-reactive) → NO IPC
Pause 30s → requestIdleCallback → doSave → writeNote (lightweight, fire-and-forget)
Tab switch → onDestroy → doFlush → WAB + direct tab mutation + writeNote
Tab close → onDestroy → doFlush → WAB (localStorage) + writeNote
App close → beforeunload → doFlush → WAB (localStorage)
Reopen note → openNoteTab checks WAB first → instant restore
```

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (save/restore, WAB integration, beforeunload)
- `src/routes/+layout.svelte` — MODIFIED (onflush/onsave wiring, WAB integration, save guard)
- `src/lib/libraries/store.ts` — MODIFIED (WAB functions, markRecentWrite, openNoteTab WAB check)

## Blocking Issues Resolved During Phase 2
- **BLOCKING-002:** Re-entrant store update in onDestroy → fixed with direct mutation
- **BLOCKING-003:** closeTab 3 separate store updates → fixed with batched updates
- **IPC Lag:** Rust invoke causes progressive lag during typing → fixed by removing all IPC from typing path

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | Zero IPC during typing. updateListener only sets non-reactive vars. Save via requestIdleCallback. |
| Architecture (AA) | PASS | One-way editor→parent flow. No $effect echo loops. WAB for close/reopen. Direct mutation for tab switch. |
| Memory (MA) | PASS | setInterval cleared, visibilitychange removed, beforeunload removed, rAF cancelled, view.destroy() + null |
| Spec Compliance (SCA) | PASS | Content persists across tab switch, tab close+reopen, and app restart. Cursor/scroll preserved. |
| RTL/Bidi (RA) | PASS | No changes from Phase 1 |
| UX (UXA) | PASS | Zero typing lag. Save invisible. Content/cursor/scroll restored on reopen. |
| Code Quality (CQA) | PASS | Clean separation: doSave (disk), doFlush (WAB+disk), WAB functions in store |
| Environment (EA) | PASS | BLOCKING-002 + BLOCKING-003 resolved. File watcher respects markRecentWrite. |

## Testing Protocol (user-tested 2026-03-27, 13 rounds)

| Test | Result |
|---|---|
| Type "test123" → wait 2s → close tab → reopen → content there | PASS |
| Type in A → switch to B → switch back → A content preserved | PASS |
| Place cursor at line 2 → close → reopen → cursor at line 2 | PASS |
| Scroll down → close → reopen → scroll position restored | PASS |
| Rapid typing, pause 1-2s, resume (5+ times) → zero lag | PASS |
| Close app entirely → reopen → content preserved | PASS |

## Decision
- [x] APPROVED — all 8 auditors pass, all 6 user tests pass (after 13 rounds of iterative fixes)
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-27
