# Experiment: Phase 1 — The Bare Editor

## Hypothesis
A CM6 editor with only history, drawSelection, markdown (no codeLanguages), keymap, lineWrapping, and dir — with one-way onchange — achieves < 5ms typing latency.

## Spec Reference
- Section 2.1: The Editor Owns Its Content
- Section 2.6: No $effect for Editor State
- Section 3.3: Phase 1 extensions
- Section 4.1: Typing Must Be Instant

## Implementation
- CM6 EditorView with 7 bare extensions
- One-way: editor → onchange(text) → parent (onchange is no-op in Phase 1)
- No $effect for value sync
- EditorView.destroy() in onDestroy
- Enter in title → view.focus()
- Clean theme: no borders, no gutters, no active line highlight

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (added CM6, 181 lines)
- `src/routes/+layout.svelte` — MODIFIED (pass value + onchange to eNotePane)

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | Zero ViewPlugins, updateListener guarded by docChanged, no regex, onchange is no-op |
| Architecture (AA) | PASS | One-way flow, no $effect echo loops, only $effect is dir change (guarded by prevDir) |
| Memory (MA) | PASS | EditorView.destroy() in onDestroy, view nulled, zero timers/listeners/rAF |
| Spec Compliance (SCA) | PASS | All 7 Phase 1 extensions present, no codeLanguages, desk #e8e8ec, paper 1200px/48px |
| RTL/Bidi (RA) | PASS | editorAttributes dir via compartment, contentAttributes dir="auto", unicode-bidi: plaintext on .cm-line |
| UX (UXA) | PASS | Title focused on mount, Enter→editor focus, content visible on open |
| Code Quality (CQA) | PASS | 181 lines, clean sections, no dead code, flexbox layout |
| Environment (EA) | PASS | BLOCKING-001 fixed, onchange is no-op — zero store updates during typing |

## Testing Protocol (user-tested 2026-03-27)

| Test | Result |
|---|---|
| Note content visible on open | PASS |
| Type text — appears instantly | PASS |
| Rapid Arabic typing (20 chars) — zero lag | PASS |
| Rapid English typing (20 chars) — zero lag | PASS |
| Enter in title → editor focus | PASS |
| Undo (Ctrl+Z) / Redo (Ctrl+Y) | PASS |
| Line wrapping (no horizontal scroll) | PASS |
| RTL in editor (Arabic flows right-to-left) | PASS |
| Mixed RTL/LTR (Arabic + English on separate lines) | PASS |
| Phase 0 tests still pass | PASS |

## Decision
- [x] APPROVED — all 8 auditors pass, all 10 user tests pass
- [ ] REJECTED
- [ ] NEEDS WORK

## Note
Once a phase passes, its tests are not repeated in subsequent phases.

## Date
2026-03-27
