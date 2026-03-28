# Experiment: Phase 5 — Syntax Highlighting

## Hypothesis
Adding a custom `HighlightStyle` with visible markdown-specific colors will colorize syntax (headings, bold, italic, code, links) without any measurable typing latency increase.

## Spec Reference
- Section 3.3: Phase 5 extensions
- Section 4.4: No Feature Shall Slow Typing
- Benchmark: confirm < 5ms after adding

## Implementation
- Import `HighlightStyle` from `@codemirror/language`, `tags` from `@lezer/highlight`
- Define `markdownHighlightStyle` with explicit colors per tag:
  - Headings: red `#d73a49`, bold weight
  - Strong: orange `#e36209`
  - Emphasis: purple `#7c3aed`
  - Monospace: green `#16a34a`
  - Link: blue `#2563eb`
  - URL: cyan `#0891b2`
  - Meta: gray `#888`
- Add `syntaxHighlighting(markdownHighlightStyle)` after `markdown({ base: markdownLanguage })`
- Zero ViewPlugins. Operates on syntax tree already built by `markdown()` parser

### Note on `defaultHighlightStyle`
Initial attempt used `defaultHighlightStyle` — it compiled but produced no visible color change because its colors are too subtle / overridden by app text color. Switched to custom `HighlightStyle.define()` with explicit, visible colors.

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (imports line 16-17, style definition lines 20-35, extension line 137)

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | No ViewPlugin. Highlighter reuses syntax tree from `markdown()` — zero extra parsing per keystroke |
| Architecture (AA) | PASS | Declarative `HighlightStyle.define()` + `syntaxHighlighting()`. No $effect, no state, no callbacks |
| Memory (MA) | PASS | No timers, no listeners, no subscriptions added |
| Spec Compliance (SCA) | PASS | Headings, bold, italic, code, links all colored. Matches Phase 5 spec |
| RTL/Bidi (RA) | PASS | Syntax highlighting is text decoration only — no layout impact, works in any direction |
| UX (UXA) | PASS | Distinct, visible colors per syntax type. Consistent color language (red=heading, orange=bold, etc.) |
| Code Quality (CQA) | PASS | ~18 lines added (imports + style definition + extension). Clean, declarative |
| Environment (EA) | PASS | No store updates, no IPC, no reactivity changes |

## Testing Protocol (user-tested 2026-03-28)

| Test | Result |
|---|---|
| Heading (`# Title`) shows colored (red, bold) | PASS |
| Bold (`**text**`) shows colored (orange) | PASS |
| Italic (`*text*`) shows colored (purple) | PASS |
| Inline code (`` `code` ``) shows colored (green) | PASS |
| Link (`[text](url)`) shows colored (blue/cyan) | PASS |
| Rapid typing (10 chars) → zero lag | PASS |
| Long note → scroll smooth | PASS |

## Known Behavior
Initial 3-5s lag when first opening a long note — the markdown parser builds the syntax tree incrementally. After the tree is cached, scrolling is smooth. Same behavior as CodeMirrorEditor/NotePane. Not caused by syntax highlighting (parser runs regardless).

## Decision
- [x] APPROVED — all 8 auditors pass, all 7 user tests pass
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-28
