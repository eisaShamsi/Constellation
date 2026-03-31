# Session Log — 2026-03-31

## Phase: NotePane Regression Testing + Critical Bug Fixes

---

## Commits This Session

| Hash | Description |
|------|-------------|
| `57c9b8b` | Docs: add NotePane regression test plan (86 tests across 13 sections) |
| `83fe37f` | Fix: typing lag + callout collapse/expand |
| `3f1ed8f` | Fix: post-pause typing stutter + O(N) lineDecoPlugin scan + callout standard |
| `2b6f5bb` | Fix: rebuild calloutPlugin from scratch + remove async rebuild from all 3 plugins |
| `8f1900d` | Fix: map+debounce perf, all callouts foldable, RTL Enter cursor via bidiPlugin |
| `fe2707a` | Fix: click→selection, callout sync rebuild, RTL Enter cursor, Obsidian foldable standard |
| `5fa9589` | Fix: replace Decoration.replace body collapse with per-line CSS hiding |
| `ae4b7b5` | Fix: chevron collapse moves cursor out of body in same transaction |
| `1026905` | Fix: calloutExitOnEnter needs Prec.highest to beat lang-markdown keymap |
| `c24ee30` | Fix: callout Enter exit + 1500ms debounced save + collapse freeze guard |
| `64b6bd2` | Fix: rebuild calloutPlugin with Obsidian full-line widget pattern |
| `42fe06c` | Redesign calloutPlugin: explicit Rule A + Rule B freeze-proof architecture |
| `f30805a` | LL-014: Three strikes — fix from the root, don't patch |
| `a7cfdff` | Perf: cache decorations + codify performance architecture rules |

---

## Work Completed

### WiW (Window in Window) — from previous session
- Renamed all PiP/pip identifiers → WiW/wiw throughout codebase
- Added hint text to mini-window topbar subtitle
- Added WiW on/off toggle button to main Sky View topbar
- Sky View freeze fixed (Pixi ticker stop order + destroy order)
- Empty page on node click fixed (await openNoteTab before showStarView = false)

### NotePane Regression Plan
- Created `lab/NOTEPANE-REGRESSION-2026-03-31.md` — 86 tests across R1–R13
- R1 PASS, R2 PASS, R3 in progress

### Critical Bugs Fixed

#### Typing Lag (R1)
- **Root cause**: `doc.toString()` called on every `docChanged` event — O(N) per keystroke
- **Fix**: moved `toString()` to save functions only (`doSave`, `doFlush`)
- **Root cause 2**: `requestAnimationFrame` throttled in idle Tauri webview caused deferred rebuild to fire on first keystroke after a pause
- **Fix**: removed all async rebuild (setTimeout + rAF + view.dispatch) from all 3 plugins
- **Root cause 3**: livePreview / lineDecoPlugin use `syntaxTree` internally — sync rebuild on every keystroke was 10-30ms per character
- **Fix**: map(changes) fast path on docChanged + debounced full rebuild (no view.dispatch)

#### Callout Freeze (R4)
- **Root cause**: `view.dispatch({})` inside rAF re-entered CM6's update cycle while replace-decorations were active → DOM reconciliation stall / freeze
- **Fix**: removed view.dispatch from all plugins entirely
- **Root cause 2**: calloutPlugin used map+debounce without view.dispatch → decorations updated in memory but never shown until next user interaction
- **Fix**: calloutPlugin is regex-only (no syntaxTree), so restored synchronous rebuild on all updates. Fast (<1ms per rebuild)
- **Foldable**: Obsidian standard — only `> [!type]-` (collapsed) and `> [!type]+` (expanded) get a chevron. Plain `> [!info]` blocks are not foldable

#### Click → Selection Bug (R1, introduced by bidiPlugin)
- **Root cause 1**: `editorDir === 'auto'` (literal string) caused `'ltr' !== 'auto'` to always be true → `dir='ltr'` added to EVERY line in EVERY LTR document → full DOM mutation on every visible line
- **Root cause 2**: `$effect` for scriptFonts fired on every `appSettings` change (any property), causing spurious bidiPlugin rebuilds + DOM mutations during click events
- **Fix 1**: `resolveEditorDir()` resolves 'auto' by scanning first 10 content lines
- **Fix 2**: scriptFonts `$effect` guards with JSON key comparison — only dispatches when scriptFonts actually changed

#### RTL Enter Cursor (R11/R12)
- **Root cause 1**: bidiPlugin was not integrated into NotePane — was untracked and never wired up
- **Root cause 2**: NotePane CSS `.cm-line { unicode-bidi: plaintext }` ignored the `dir` attribute for cursor placement on empty lines
- **Fix 1**: integrated bidiPlugin into NotePane (import + extensions + scriptFonts effect)
- **Fix 2**: added `.e-editor .cm-line[dir] { unicode-bidi: isolate }` — higher specificity (0,2,1 vs 0,2,0) → `dir='rtl'` now governs cursor placement on empty lines
- **Fix 3**: bidiPlugin empty-line inheritance — empty lines after RTL lines inherit `dir='rtl'` so pressing Enter in Arabic text lands the cursor on the right side

---

## Plugin Architecture (final state)

| Plugin | Rebuild strategy | Uses syntaxTree |
|--------|-----------------|-----------------|
| `calloutPlugin` | Sync on all updates (docChanged, viewportChanged, selectionSet line-cross, toggle) | No |
| `livePreview` | map+debounce(300ms) on docChanged; sync on selectionSet/viewportChanged | Yes |
| `lineDecoPlugin` | map+debounce(300ms) on docChanged; sync on viewportChanged | Yes |
| `bidiPlugin` | map+debounce(300ms) on docChanged; sync on viewportChanged/scriptFonts | No |

**Rule**: Plugins using `syntaxTree` must use map+debounce — syntaxTree parsing is 10-30ms and blocks typing. Regex-only plugins can rebuild synchronously.

**Rule**: Never use `view.dispatch({})` in a timeout/rAF — causes re-entrant CM6 update while replace-decorations are active → freeze.

---

## Callout Plugin — Permanent Freeze Fix (this session)

### Root Cause (LL-014 applied)
After 7+ patch attempts, stopped patching and diagnosed root cause:
`Decoration.replace([from, to])` creates a cursor-exclusion range. Cursor inside → CM6 nudges cursor out → `selectionSet` fires → plugin rebuilds → range restored → nudge again → **infinite freeze loop**.

### Permanent Fix — Two Rules
- **Rule A**: `Decoration.replace` only added when cursor is on a **different line** (title widget + `>` prefix removal). Provably safe at line granularity.
- **Rule B**: Collapsed body uses `Decoration.line` at `(line.from, line.from)` — zero-length, no range, cursor can never be "inside" it. CSS `display:none` hides lines. Freeze loop architecturally impossible.

### New Lesson
**LL-014**: If a bug survives three fix attempts, stop patching. Find root cause and redesign from that level.

---

## Performance Architecture (this session)

### livePreview.ts — Allocation fix
- `headingDecos[]` — 6 `Decoration.mark()` objects pre-cached at module load (were recreated on every `buildDecorations()` call)
- `checkboxCheckedDeco` / `checkboxUncheckedDeco` — pre-cached (only 2 states)

### CLAUDE.md — New enforced rules
- ViewPlugin line-change guard pattern (copy-paste template)
- Decoration pre-cache rule
- IPC boundary rules
- Virtual scrolling requirement for lists > 50 items
- Three-strikes patch rule (LL-014)

### docs/IPC-CONTRACT.md (NEW)
- Full registry of all 60+ `invoke()` calls
- Hot-path watch list with mitigation
- Design rules: payload budgets, debounce minimums, push-over-poll

---

## Test Results

| Section | Result |
|---------|--------|
| R1 Basic Editing & Performance | ✅ PASS |
| R2 Tab Switching & Save/Restore | ✅ PASS |
| R3 Live Preview | 🔄 Pending |
| R4 Callout Rendering | 🔄 Pending (redesigned — needs re-test) |
| R5–R13 | Not yet started |

---

## Open Items

- R3–R13 regression testing pending
- Autocomplete popup: Arabic interface — UI chrome vs. note names (needs clarification)
- `+layout.svelte` decomposition into reactive islands (Priority 3 from perf plan) — future session
- Rust-side search with tantivy — future session
- Graph force simulation in Rust — future session
- Performance regression infrastructure (synthetic vault generator, frame budget overlay) — future session

---

## Next Session Pickup

1. Run regression tests R3–R13 in the app
2. After all pass → milestone tag `milestone/notepane-stable` + ZIP backup
3. After milestone → `/simplify` code review pass
