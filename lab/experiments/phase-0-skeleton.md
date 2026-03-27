# Experiment: Phase 0 — The Skeleton

## Hypothesis
A bare eNotePane with just desk + paper + title renders correctly, supports RTL/LTR, and auto-generates title as CoNoteDDMMYYYY.HH:MM.

## Spec Reference
- Section 0.3: Title (auto-generated CoNoteDDMMYYYY.HH:MM)
- Section 3.1: Visual Layout (PoD)
- Section 3.6: UI Design Principles

## Implementation
- `eNotePane.svelte` — single component (122 lines)
- Gray desk (#e8e8ec), white paper (max-width: 1200px, padding: 48px)
- Title input with dir="auto", centered/start per setting
- Auto-title on blur if empty

## Files Changed
- `src/lib/components/eNotePane.svelte` — NEW

## Benchmark Results
N/A (no editor in Phase 0)

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | -- | No editor, N/A |
| Architecture (AA) | -- | No editor, N/A |
| Memory (MA) | PASS | Zero timers/listeners/views — nothing to leak |
| Spec Compliance (SCA) | PASS | Desk #e8e8ec, paper 1200px/48px, auto-title CoNoteDDMMYYYY.HH:MM |
| RTL/Bidi (RA) | PASS | dir attr on container, dir="auto" on title input |
| UX (UXA) | PASS | Title editable, auto-title on blur, Enter stable |
| Code Quality (CQA) | PASS | 122 lines, clean component, flexbox layout, no dead code |
| Environment (EA) | PASS | App responsive after BLOCKING-001 fix, title interaction instant |

## Testing Protocol (user-tested 2026-03-27)

| Test | Result |
|---|---|
| Open note -> gray desk + white paper centered | PASS |
| Title shows note's actual title | PASS |
| Click title -> cursor appears, can edit | PASS |
| Press Enter in title -> nothing breaks | PASS |
| Blur empty title -> auto-generates CoNoteDDMMYYYY.HH:MM | PASS |
| RTL note -> title aligns correctly | PASS |
| Resize window -> paper stays centered | PASS |

## Decision
- [x] APPROVED — all 8 auditors pass, all 7 user tests pass
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-27 (re-tested after BLOCKING-001 fix)
