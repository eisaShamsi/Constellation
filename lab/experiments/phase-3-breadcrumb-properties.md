# Experiment: Phase 3 — Breadcrumb & Properties

## Hypothesis
Breadcrumb navigation and collapsible properties panel work without affecting typing performance.

## Spec Reference
- Section 0.3.1: Note Metadata (YAML Frontmatter)
- Section 3.2: Components (breadcrumb, properties)
- Section 5.2: Zettelkasten note type property

## Implementation
- Breadcrumb: library / note path, back/forward nav, more options
- Properties: collapsible panel, add/remove/edit key-value pairs
- Auto-populated metadata in frontmatter (spec 0.3.1)
- Properties callbacks: onpropertieschange
- Navigation callbacks: onnavigateback, onnavigateforward, onmoreoptions

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED
- `docs/eNotePane-spec.md` — MODIFIED (added 0.3.1)
- `src/lib/i18n/*.json` — MODIFIED (added propertyKey, propertyValue)

## Date
2026-03-26
