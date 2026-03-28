# Experiment: Phase 4 — Toolbar

## Hypothesis
A formatting toolbar with Bold, Italic, Strikethrough, Highlight, Code, Headings, Lists, Link, Insert (blockquote/code/hr/table/image), and Undo/Redo can be added without any measurable typing latency increase.

## Spec Reference
- Section 3.2: Components (toolbar between properties and editor)
- Section 3.3: Phase 4 extensions
- Section 4.4: No Feature Shall Slow Typing

## Implementation
- `wrapSelection(before, after)`: toggle markdown marks. Handles no selection (insert marks), selection (wrap), already-wrapped (unwrap/toggle)
- `insertLinePrefix(prefix)`: toggle heading/list prefix at line start
- `insertAtCursor(text)`: insert blockquote, code block, hr, table, image
- `tbUndo()`/`tbRedo()`: dispatch CM6 undo/redo via imported `undo`/`redo` functions
- Dropdown menus: heading (H1-H6), list (bullet/numbered/task), insert (5 items)
- Menu management: `showHeadingMenu`, `showListMenu`, `showInsertMenu` $state with click-outside dismiss
- `onmousedown={preventDefault}` on toolbar container prevents editor blur on button click
- All buttons use `view.dispatch()` — never modify editor state directly

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (411 → 498 lines)

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | Zero ViewPlugins. Toolbar dispatches CM6 commands — zero per-keystroke cost |
| Architecture (AA) | PASS | Toolbar → view.dispatch() one-way. No $effect. No store updates |
| Memory (MA) | PASS | Menu click listeners use { once: true }. No new timers/intervals |
| Spec Compliance (SCA) | PASS | H1-H6, Bold, Italic, Strikethrough, Highlight, Lists, Link, Insert, Undo/Redo |
| RTL/Bidi (RA) | PASS | Dropdown menus flip via :global([dir="rtl"]) |
| UX (UXA) | PASS | Toggle behavior, menus dismiss on click outside, editor keeps focus |
| Code Quality (CQA) | PASS | 498 lines (under 500). wrapSelection matches CodeMirrorEditor pattern |
| Environment (EA) | PASS | No store updates, no IPC, no reactivity from toolbar |

## Testing Protocol (user-tested 2026-03-28)

| Test | Result |
|---|---|
| Bold: select text → click B → wraps with ** | PASS |
| Bold toggle: select **bold** → click B → unwraps | PASS |
| Italic, Strikethrough, Highlight, Code work | PASS |
| Heading dropdown: H1-H6 applies # prefix | PASS |
| List dropdown: bullet, numbered, task | PASS |
| Link button wraps with [[ ]] | PASS |
| Insert: blockquote, code block, hr, table, image | PASS |
| Undo/Redo buttons work | PASS |
| Rapid typing → zero lag (toolbar doesn't affect performance) | PASS |

## Known Issue
Progressive delay on repeated 2-3s pauses — same as Phase 2 environmental issue (IPC accumulation). Not caused by toolbar (zero per-keystroke cost). Confirmed by user.

## Decision
- [x] APPROVED — all 8 auditors pass, all 9 user tests pass
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-28
