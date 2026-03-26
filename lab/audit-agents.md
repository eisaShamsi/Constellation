# Audit Agents

Independent, unbiased reviewers for eNotePane development.
Each agent has ONE job. They don't accept work that fails their criteria.

---

## Agent 1: Performance Auditor (PA)
**Mission:** Ensure every keystroke is processed in < 5ms.

**Checks:**
- [ ] No ViewPlugin rebuilds ALL decorations on docChanged
- [ ] No $effect reads or writes editor content (value, editBody)
- [ ] No store update (updateTabContent) during autosave
- [ ] updateListener has NO debounce (fires immediately)
- [ ] Save debounce is >= 1500ms
- [ ] No @codemirror/language-data import (500KB+ bloat)
- [ ] No Unicode regex running on every keystroke
- [ ] Only viewport ranges processed (view.visibleRanges)
- [ ] MatchDecorator.updateDeco used instead of full rebuild
- [ ] No setTimeout/setInterval without cleanup in onDestroy

**Verdict:** PASS / FAIL with specific line numbers.

---

## Agent 2: Architecture Auditor (AA)
**Mission:** Ensure the editor-parent communication is one-way and clean.

**Checks:**
- [ ] Editor → onchange → Parent (one-way, no echo)
- [ ] No $effect syncing value back to editor
- [ ] {#key tab.id} used for tab switch (component recreation)
- [ ] onDestroy updates store + saves to disk
- [ ] No circular reactive dependencies
- [ ] No $state variable read by both $effect and template that could cause cascade
- [ ] Props to editor are stable references (no object recreation per render)

**Verdict:** PASS / FAIL with data flow diagram.

---

## Agent 3: Memory Auditor (MA)
**Mission:** Ensure zero memory leaks.

**Checks:**
- [ ] Every setTimeout → clearTimeout in onDestroy
- [ ] Every setInterval → clearInterval in onDestroy
- [ ] Every addEventListener → removeEventListener in onDestroy
- [ ] Every EditorView → .destroy() in onDestroy
- [ ] Every Tauri listen() → unlisten in onDestroy
- [ ] Every requestAnimationFrame → cancelAnimationFrame in onDestroy
- [ ] No circular references in closures
- [ ] No growing arrays/maps without bounds

**Verdict:** PASS / FAIL with leak location.

---

## Agent 4: Spec Compliance Auditor (SCA)
**Mission:** Ensure implementation matches eNotePane specification exactly.

**Checks:**
- [ ] Paper max-width: 1200px
- [ ] Paper padding: 48px all sides
- [ ] Desk color: #e8e8ec
- [ ] No visible editor borders
- [ ] No active line highlight
- [ ] No gutter borders
- [ ] Title auto-generates as CoNoteDDMMYYYY.HH:MM if empty
- [ ] Title is core but not required to start typing
- [ ] Body can be empty
- [ ] Toolbar present with formatting buttons
- [ ] Properties collapsible
- [ ] Tab locked to paper edge
- [ ] Horizontal scroll for overflow tabs

**Verdict:** PASS / FAIL with deviation list.

---

## Agent 5: RTL/Bidi Auditor (RA)
**Mission:** Ensure perfect bidirectional text support.

**Checks:**
- [ ] dir attribute set on editor element
- [ ] No custom bidiPlugin (use CM6 built-in)
- [ ] Arabic text right-aligned in RTL notes
- [ ] English text left-aligned in LTR notes
- [ ] Mixed Arabic+English in same line renders correctly
- [ ] Cursor moves in correct direction per script
- [ ] Title input respects dir="auto"
- [ ] Toolbar icons don't flip incorrectly
- [ ] Breadcrumb respects note direction

**Verdict:** PASS / FAIL with screenshot evidence.

---

## Agent 6: UX Auditor (UXA)
**Mission:** Ensure the note feels right — fast, intuitive, no friction.

**Checks:**
- [ ] New note opens with cursor ready to type (no click needed)
- [ ] Rapid typing (20 chars) shows zero visible delay
- [ ] Tab switch shows content instantly (no flash of empty)
- [ ] Close + reopen preserves content
- [ ] Undo/redo works across session
- [ ] Scroll position preserved on tab switch
- [ ] Cursor position preserved on tab switch
- [ ] Title editing → Enter moves to body
- [ ] Escape from title cancels edit
- [ ] Word count updates without lag

**Verdict:** PASS / FAIL with user flow analysis.

---

## Agent 7: Code Quality Auditor (CQA)
**Mission:** Ensure code is clean, minimal, and maintainable.

**Checks:**
- [ ] No dead code (unused imports, unreachable branches)
- [ ] No undeclared variables
- [ ] No TypeScript errors
- [ ] CSS uses logical properties (margin-inline-start, not margin-left)
- [ ] No magic numbers without comments
- [ ] No inline styles that should be classes
- [ ] Component is < 500 lines (split if larger)
- [ ] Every function has a clear purpose
- [ ] No duplicated logic between eNotePane and FocusPane

**Verdict:** PASS / FAIL with issue list.

---

## Audit Workflow

```
Developer submits code
    ↓
All 7 agents run in parallel
    ↓
Each agent returns PASS or FAIL
    ↓
ALL must PASS to merge
    ↓
Any FAIL → fix → re-audit
```

## Running an Audit

To audit a file, spawn all 7 agents simultaneously:

```
Agent 1 (PA):  "Audit [file] against Performance rules in eNotePane spec"
Agent 2 (AA):  "Audit [file] against Architecture rules in eNotePane spec"
Agent 3 (MA):  "Audit [file] for memory leaks per eNotePane spec"
Agent 4 (SCA): "Audit [file] against visual/behavioral spec in eNotePane spec"
Agent 5 (RA):  "Audit [file] for RTL/bidi compliance per eNotePane spec"
Agent 6 (UXA): "Audit [file] for UX quality per eNotePane spec"
Agent 7 (CQA): "Audit [file] for code quality per eNotePane spec"
```
