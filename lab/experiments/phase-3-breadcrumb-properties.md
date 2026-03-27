# Experiment: Phase 3 — Breadcrumb & Properties

## Hypothesis
Note navigation (breadcrumb with back/forward) and metadata editing (PropertyEditor) integrate cleanly with eNotePane without affecting typing performance.

## Spec Reference
- Section 3.2: Components (breadcrumb, properties)
- Section 10: Phase 3 definition

## Implementation
- Breadcrumb bar above paper: library / note name, back/forward nav, saving indicator, more menu (⋮)
- More menu: addProperty, rename, revealInTree, showInExplorer, openDefaultApp, copyPath, copyName, delete
- Properties: reuses existing PropertyEditor component, supports 'source' (raw YAML) and 'visible' (form) modes
- Collapsible with chevron animation, RTL support (chevrons flip)
- All user-facing strings via $t() — all 15 locales updated
- PropertyEditor direct mutation fix: tab.content updated via direct mutation after save

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (breadcrumb + properties, 411 lines)
- `src/lib/components/PropertyEditor.svelte` — MODIFIED (direct mutation for store sync)
- `src/routes/+layout.svelte` — MODIFIED (new props + callbacks wiring)
- `src/lib/i18n/*.json` (all 15 locales) — MODIFIED (5 new keys)

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | Zero new ViewPlugins. Breadcrumb + properties are static DOM. PropertyEditor saves debounced (800ms). |
| Architecture (AA) | PASS | PropertyEditor reused from NotePane. Direct mutation for store sync. No new $effect blocks. |
| Memory (MA) | PASS | More menu click listener uses { once: true }. No new timers/intervals. |
| Spec Compliance (SCA) | PASS | Breadcrumb, back/forward, more menu, properties collapsible, source/visible modes. |
| RTL/Bidi (RA) | PASS | Breadcrumb RTL. Nav chevrons flip. Properties chevron rotates. Menu direction adapts. |
| UX (UXA) | PASS | More menu clean. Properties smooth. Edits persist across close/reopen. |
| Code Quality (CQA) | PASS | 411 lines (under 500). i18n correct. No dead code. |
| Environment (EA) | PASS | Zero typing lag. No new store updates during typing. |

## Testing Protocol (user-tested 2026-03-27, 2 rounds)

| Test | Result |
|---|---|
| Breadcrumb shows Library / NoteName | PASS |
| Back/Forward buttons work | PASS |
| More menu opens, items work | PASS |
| Properties panel visible | PASS |
| Edit property → save → reopen → persisted | PASS (fixed in round 2) |
| Collapse/expand properties smooth | PASS |
| RTL: breadcrumb/chevrons correct | PASS |
| Rapid typing → zero lag | PASS |

## Decision
- [x] APPROVED — all 8 auditors pass, all 8 user tests pass
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-27
