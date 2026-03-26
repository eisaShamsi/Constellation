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
- One-way: editor → onchange(text) → parent (no debounce in editor)
- No $effect for value sync
- EditorView.destroy() in onDestroy
- Enter in title → view.focus()
- Clean theme: no borders, no gutters, no active line highlight

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (added CM6)

## Benchmark Results

| Metric | Target | Actual | Pass? |
|---|---|---|---|
| Avg latency (ms) | < 5 | PENDING | |
| P95 latency (ms) | < 10 | PENDING | |
| Max latency (ms) | < 50 | PENDING | |

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
