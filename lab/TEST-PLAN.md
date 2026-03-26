# eNotePane Test Plan — Each Phase Must Pass Before Next

## The Rule
NO phase is approved until ALL tests pass in the RUNNING APP.
Code auditors check quality. Runtime tests check reality.

---

## Phase 0: The Skeleton
**What**: Gray desk + white paper + editable title. No editor.

**Tests (manual in running app)**:
1. [ ] Open a note → see gray desk with white paper centered
2. [ ] Title shows the note's actual title (not "Untitled" if note has content)
3. [ ] Click title → cursor appears, can edit
4. [ ] Press Enter in title → nothing breaks (editor not yet added)
5. [ ] Blur empty title → auto-generates CoNoteDDMMYYYY.HH:MM
6. [ ] RTL note → title aligns correctly
7. [ ] Resize window → paper stays centered, no overflow

**Pass criteria**: ALL 7 tests pass. Screenshot required.

---

## Phase 1: The Bare Editor
**What**: CM6 with ZERO plugins. Just text input.

**Tests (manual in running app)**:
1. [ ] Click below title → cursor appears in editor
2. [ ] Type 20 Arabic characters rapidly → ZERO lag, every character appears instantly
3. [ ] Type 20 English characters rapidly → ZERO lag
4. [ ] Press Enter → new line, no delay
5. [ ] Press Space → no delay, no cursor jump
6. [ ] Pause 3 seconds, resume typing → no delay, no cursor jump
7. [ ] Type `> [!info]` → raw text appears (no decoration, no syntax)
8. [ ] Type `**bold**` → raw text appears (no decoration)
9. [ ] Editor direction matches note (RTL note = RTL editor)
10. [ ] Scroll down in long note → smooth, no stutter

**Pass criteria**: ALL 10 tests pass. Tests 2-6 are CRITICAL — any lag = fail.

---

## Phase 2: Save & Restore
**What**: Content persists. Cursor/scroll position remembered.

**Tests (manual in running app)**:
1. [ ] Type "test123" → wait 2 seconds → close tab → reopen → "test123" is there
2. [ ] Type in note A → switch to note B → switch back to A → content preserved
3. [ ] Place cursor at line 5 → close tab → reopen → cursor near line 5
4. [ ] Scroll down → close tab → reopen → scroll position restored
5. [ ] Type rapidly during autosave (1.5s) → NO cursor jump, NO lag
6. [ ] Close app entirely → reopen → last note content is preserved

**Pass criteria**: ALL 6 tests pass. Test 5 is CRITICAL.

---

## Phase 3: Breadcrumb
**What**: Library / note path display above the paper.

**Tests (manual in running app)**:
1. [ ] Breadcrumb shows "Library / Note Name"
2. [ ] Back/forward arrows work
3. [ ] More options (⋮) button appears
4. [ ] Breadcrumb doesn't interfere with typing speed (run Phase 1 tests 2-6 again)

**Pass criteria**: ALL 4 tests pass.

---

## Phase 4: Properties
**What**: Collapsible property editor.

**Tests (manual in running app)**:
1. [ ] Properties panel shows, starts collapsed
2. [ ] Click to expand → shows existing properties
3. [ ] Add a property → saves correctly
4. [ ] Properties don't interfere with typing speed (run Phase 1 tests 2-6 again)
5. [ ] Collapse/expand doesn't cause cursor jump

**Pass criteria**: ALL 5 tests pass.

---

## Phase 5: Markdown Syntax Highlighting
**What**: CM6 markdown parser + defaultHighlightStyle. No live preview.

**Tests (manual in running app)**:
1. [ ] Type `# Heading` → syntax colored differently
2. [ ] Type `**bold**` → markers colored
3. [ ] Type `[link](url)` → syntax colored
4. [ ] Run Phase 1 tests 2-6 → ALL must still pass (no lag from highlighting)
5. [ ] Open a 5000-word note → scroll smoothly, no stutter

**Pass criteria**: ALL 5 tests pass. Test 4 is CRITICAL.

---

## Phase 6: Live Preview
**What**: livePreviewPlugin — hides markers, shows styled text.

**Tests (manual in running app)**:
1. [ ] `**bold**` → shows bold text, markers hidden when cursor away
2. [ ] Click on bold text → markers reappear for editing
3. [ ] `# Heading` → large text, `#` hidden when cursor away
4. [ ] `[[wikilink]]` → brackets hidden, text styled as link
5. [ ] Run Phase 1 tests 2-6 → ALL must still pass
6. [ ] Pause 3 seconds, resume typing → no lag, no cursor jump

**Pass criteria**: ALL 6 tests pass. Tests 5-6 are CRITICAL.

---

## Phase 7: Callouts
**What**: calloutPlugin — styled callout blocks.

**Tests (manual in running app)**:
1. [ ] `> [!info] Title` → shows colored border + icon + title text
2. [ ] Title text IS EDITABLE (click on it, cursor appears, can type)
3. [ ] Click chevron → collapses/expands content
4. [ ] Type inside callout → no cursor jump to title
5. [ ] Run Phase 1 tests 2-6 → ALL must still pass
6. [ ] Pause and resume near callout → no lag

**Pass criteria**: ALL 6 tests pass.

---

## Phase 8: Line Decorations
**What**: lineDecoPlugin — blockquote borders, code block backgrounds.

**Tests (manual in running app)**:
1. [ ] `> quote` → left border appears
2. [ ] ``` code block ``` → background tint appears
3. [ ] Run Phase 1 tests 2-6 → ALL must still pass
4. [ ] Open large note with many code blocks → scroll smoothly

**Pass criteria**: ALL 4 tests pass.

---

## Phase 9: Integration
**What**: Replace old NotePane with eNotePane in +layout.svelte.

**Tests (manual in running app)**:
1. [ ] Open any note → renders in eNotePane (not old NotePane)
2. [ ] All previous phase tests still pass
3. [ ] Tab switching works (content loads correctly)
4. [ ] Multiple tabs → each has correct content
5. [ ] Focus mode still works (FocusPane unaffected)
6. [ ] Split view falls back to old NotePane (intentional)

**Pass criteria**: ALL 6 tests pass.
