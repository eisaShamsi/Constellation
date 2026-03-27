# Experiment: Phase 2 — Save & Restore

## Hypothesis
Notes persist across tab switches and app restarts with zero data loss. Cursor and scroll positions are preserved. Save is background-only with no typing interference.

## Spec Reference
- Section 2.2: No Store Updates During Typing
- Section 4.2: Save Is Background-Only
- Section 4.3: Tab Switch = Component Recreation

## Implementation
- `onsave` callback: debounced at 1500ms, parent writes to disk only
- `onflush` callback: immediate on destroy, parent updates store + writes to disk
- `latestText`: non-reactive variable tracking latest content (not $state)
- `dirty` flag: prevents unnecessary flush on destroy if nothing changed
- `oncursorchange` / `onscrollchange`: saved on destroy only (not during typing)
- `initialCursorPos` / `initialScrollTop`: restored on mount
- `{#key tab.id + '|' + path}` in parent for tab switch (component recreation)
- `{@const _mountedTab = $activeTab}` captures tab at mount to prevent race conditions

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (added save/restore)
- `src/routes/+layout.svelte` — MODIFIED (wired onsave/onflush/oncursorchange/onscrollchange)

## Benchmark Results

| Metric | Target | Actual | Pass? |
|---|---|---|---|
| Avg latency (ms) | < 5 | PENDING user test | |
| P95 latency (ms) | < 10 | PENDING user test | |
| Max latency (ms) | < 50 | PENDING user test | |

Note: Typing latency must be re-confirmed by user in running app. Save path adds zero overhead to keystroke path (debounced, fires after 1500ms pause).

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | updateListener guarded by docChanged, save debounced 1500ms, onchange is no-op in parent, zero overhead on keystroke path |
| Architecture (AA) | PASS | One-way editor->parent flow, no $effect echo loops, latestText non-reactive, store updated only on flush (destroy) |
| Memory (MA) | PASS | setTimeout cleared in onDestroy, rAF cancelled in onDestroy, EditorView.destroy() called, view nulled |
| Spec Compliance (SCA) | PASS | 1500ms debounce, no store update during autosave, cursor/scroll preserved, {#key} tab switch, desk #e8e8ec, paper 1200px/48px |
| RTL/Bidi (RA) | PASS | No changes from Phase 1 — CM6 editorAttributes + contentAttributes dir, unicode-bidi: plaintext |
| UX (UXA) | PASS | Save invisible to user, no flash on tab switch (value passed at creation), dirty flush prevents data loss |
| Code Quality (CQA) | PASS | 270 lines, clean sections (props/state/mount/destroy/effect/title/exports), no dead code, flexbox layout |

## Testing Protocol (TEST-PLAN.md) — PENDING USER

| Test | Result |
|---|---|
| Type "test123" -> wait 2s -> close tab -> reopen -> content there | PENDING |
| Type in A -> switch to B -> switch back to A -> content preserved | PENDING |
| Place cursor at line 5 -> close -> reopen -> cursor near line 5 | PENDING |
| Scroll down -> close -> reopen -> scroll position restored | PENDING |
| Type rapidly during autosave (1.5s) -> NO cursor jump, NO lag | PENDING |
| Close app entirely -> reopen -> last note content preserved | PENDING |

## Decision
- [ ] APPROVED
- [ ] REJECTED
- [ ] NEEDS WORK

## Commit
`8d3244e` — eNotePane Phase 2: Save & Restore — ALL 7 AUDITORS PASS

## Date
2026-03-26
