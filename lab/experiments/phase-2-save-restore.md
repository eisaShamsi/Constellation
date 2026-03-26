# Experiment: Phase 2 — Save & Restore

## Hypothesis
Notes persist across tab switches and app restarts with zero data loss. Cursor and scroll positions are preserved. Save is background-only with no typing interference.

## Spec Reference
- Section 2.2: No Store Updates During Typing
- Section 4.2: Save Is Background-Only
- Section 4.3: Tab Switch = Component Recreation

## Implementation
- `onsave` callback: debounced at 1500ms, parent writes to disk
- `onflush` callback: immediate on destroy, parent updates store + writes to disk
- `latestText`: non-reactive variable tracking latest content
- `oncursorchange` / `onscrollchange`: track position per tab
- `initialCursorPos` / `initialScrollTop`: restore on mount
- `{#key tab.id}` in parent for tab switch (component recreation)

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (added save/restore)

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PENDING | |
| Architecture (AA) | PENDING | |
| Memory (MA) | PENDING | |
| Spec Compliance (SCA) | PENDING | |
| RTL/Bidi (RA) | PENDING | |
| UX (UXA) | PENDING | |
| Code Quality (CQA) | PENDING | |

## Decision
- [ ] APPROVED
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-26
