# NotePane Regression Test Plan — 2026-03-31

Run these tests in the **live app** (`npm run tauri dev`).
Each section must fully pass before moving on. Mark each test ✅ PASS or ❌ FAIL.
If a test fails, note the symptom — do NOT move forward until it's fixed.

---

## R1 — Basic Editing & Performance

> Critical: zero perceptible lag on every keystroke.

1. [ ] Open any note → NotePane renders (paper + breadcrumb + editor area)
2. [ ] Click in editor → cursor appears, ready to type
3. [ ] Type 20 English characters rapidly → **zero lag**, every character appears instantly
4. [ ] Type 20 Arabic characters rapidly → **zero lag**, characters appear right-to-left correctly
5. [ ] Press Enter → new line, no delay
6. [ ] Press Space → no delay, no cursor jump
7. [ ] Pause 3 seconds → resume typing → no delay, no cursor jump
8. [ ] Hold Backspace → characters delete smoothly, no stutter
9. [ ] Undo (Ctrl+Z) several times → each undo is instant
10. [ ] Redo (Ctrl+Y / Ctrl+Shift+Z) → each redo is instant

**PASS criteria**: ALL 10. Tests 3–8 are CRITICAL — any lag = fail.

---

## R2 — Tab Switching & Save/Restore

1. [ ] Type "REGRESSION_TEST_123" in note A → wait 2 seconds (autosave) → switch to note B → switch back to A → text is there
2. [ ] Open 3 tabs → switch between them rapidly → each tab shows correct content
3. [ ] Open a **new tab** (+ button) → blank command screen appears (NOT a NotePane editor)
4. [ ] Close a tab with unsaved content → note is not corrupted when reopened
5. [ ] Type rapidly during the 1.5s autosave window → **no cursor jump, no lag**
6. [ ] Close the app entirely → reopen → last edited note content is preserved

**PASS criteria**: ALL 6. Test 5 is CRITICAL.

---

## R3 — Live Preview

1. [ ] Type `**bold text**` → when cursor moves away, markers hide and text appears **bold**
2. [ ] Click on bold text → markers reappear for editing
3. [ ] Type `*italic*` → shows *italic* when cursor away, markers reappear on click
4. [ ] Type `# Heading 1` → shows large heading, `#` hides when cursor away
5. [ ] Type `## Heading 2` / `### Heading 3` → each renders at correct size
6. [ ] Type `[[SomeNote]]` → brackets hide, text styled as wikilink when cursor away
7. [ ] Type `[label](https://example.com)` → renders as link when cursor away
8. [ ] Type `` `inline code` `` → renders with monospace background when cursor away
9. [ ] Toggle Live Preview OFF (breadcrumb ⋯ menu) → raw markdown shows everywhere
10. [ ] Toggle Live Preview ON again → decorations restore
11. [ ] Run R1 tests 3–7 again with live preview ON → **still zero lag**

**PASS criteria**: ALL 11. Test 11 is CRITICAL.

---

## R4 — Callouts

1. [ ] Type `> [!info]` on its own line → block renders with colored border + ℹ️ icon
2. [ ] Type `> [!info] My Title` → title text "My Title" appears styled inside callout
3. [ ] Click on the title text → cursor appears, can edit the title
4. [ ] Type inside callout body → no cursor jump to title line
5. [ ] Type `> [!warning]`, `> [!tip]`, `> [!danger]` → each renders in its correct color
6. [ ] Click the chevron (▶) → callout collapses (body hides)
7. [ ] Click chevron again → callout expands
8. [ ] Run R1 tests 3–7 near a callout → **zero lag**

**PASS criteria**: ALL 8.

---

## R5 — Line Decorations

1. [ ] Type `> simple blockquote` (not a callout) → left border decoration appears
2. [ ] Type a fenced code block (``` ``` ```) → background tint appears on code lines
3. [ ] Scroll through a note with many blockquotes/code blocks → no stutter
4. [ ] Run R1 tests 3–7 → **still zero lag**

**PASS criteria**: ALL 4.

---

## R6 — Syntax Highlighting

1. [ ] `# Heading` → heading text is a distinct color from body text
2. [ ] `**bold**` markers → markers themselves are styled/dimmed
3. [ ] `> blockquote` → text has distinct color
4. [ ] `` `code` `` → inline code has monospace + distinct color
5. [ ] Links `[text](url)` → URL portion colored distinctly
6. [ ] Open a 2000+ word note → scroll from top to bottom → **no stutter**

**PASS criteria**: ALL 6.

---

## R7 — Autocomplete

1. [ ] Type `[[` → wikilink completion popup appears with note suggestions
2. [ ] Arrow-key through suggestions → highlighted item updates
3. [ ] Press Enter/Tab → selected note path is inserted and brackets closed
4. [ ] Type `#` at start of autocomplete context → tag suggestions appear
5. [ ] Type `/` on a blank line → slash command menu appears
6. [ ] Type `[[SomeNote|` → typed link completion popup shows (supports, contradicts, etc.)
7. [ ] Press Escape → any open completion popup dismisses
8. [ ] Completion popup does NOT appear mid-word in normal prose

**PASS criteria**: ALL 8.

---

## R8 — Table Toolbar

1. [ ] Place cursor inside a markdown table → TableToolbar appears above the table
2. [ ] Click "Add Row" → new row appended, table re-formatted
3. [ ] Click "Add Column" → new column appended
4. [ ] Click "Delete Row" → current row removed
5. [ ] Click column alignment toggle → column alignment changes (left/center/right)
6. [ ] Move cursor outside table → toolbar disappears
7. [ ] Type inside a table cell → no lag, toolbar stays visible

**PASS criteria**: ALL 7.

---

## R9 — Properties (Frontmatter)

1. [ ] Open a note with YAML frontmatter → Properties panel shows above the editor
2. [ ] Properties panel starts **collapsed** by default
3. [ ] Click to expand → existing properties listed with key/value
4. [ ] Edit a property value → saved correctly on blur/autosave
5. [ ] Add a new property → appears in frontmatter when note is reopened
6. [ ] Run R1 tests 3–7 with properties panel open → **zero lag**

**PASS criteria**: ALL 6.

---

## R10 — Breadcrumb & Navigation

1. [ ] Breadcrumb shows `LibraryName / NoteName` at top of note
2. [ ] Back arrow (←) navigates to previously opened note
3. [ ] Forward arrow (→) navigates forward after going back
4. [ ] ⋯ (more) menu opens with options (Live Preview toggle, etc.)
5. [ ] Breadcrumb does not interfere with typing speed (run R1 tests 3–7)

**PASS criteria**: ALL 5.

---

## R11 — RTL & Multilingual

1. [ ] Open an Arabic note → editor direction is RTL, text aligns right
2. [ ] Open an English note → editor direction is LTR
3. [ ] Mix Arabic and English in same note → each paragraph detects direction correctly
4. [ ] Live preview decorations render correctly in RTL note (bold, headings, callouts)
5. [ ] Autocomplete popup positions correctly in RTL layout

**PASS criteria**: ALL 5.

---

## R12 — FocusPane Isolation

> FocusPane must be plain text only — no decorations, no syntax, no lag.

1. [ ] Switch a note to Focus mode → no markdown rendering visible
2. [ ] Type 20 characters rapidly in FocusPane → **zero lag** (must be faster than NotePane)
3. [ ] Switch back from Focus → NotePane loads the same content with decorations
4. [ ] No data loss on mode switch (content identical in both modes)

**PASS criteria**: ALL 4. Test 2 is CRITICAL.

---

## R13 — Sky View Integration (post-fix)

1. [ ] Open Sky View → graph loads, no freeze
2. [ ] Click a node → Sky View closes, correct note opens in NotePane
3. [ ] Click × (close) → Sky View closes cleanly, app remains responsive
4. [ ] Click a note in sidebar while Sky View open → WiW mini-window appears
5. [ ] Toggle WiW off (toggle button in header) → mini-window disappears, sidebar clicks no longer trigger it
6. [ ] Toggle WiW back on → mini-window reappears on next sidebar click

**PASS criteria**: ALL 6.

---

## Summary Scorecard

| Section | Tests | Pass | Fail |
|---------|-------|------|------|
| R1 Basic Editing & Performance | 10 | | |
| R2 Tab Switching & Save/Restore | 6 | | |
| R3 Live Preview | 11 | | |
| R4 Callouts | 8 | | |
| R5 Line Decorations | 4 | | |
| R6 Syntax Highlighting | 6 | | |
| R7 Autocomplete | 8 | | |
| R8 Table Toolbar | 7 | | |
| R9 Properties | 6 | | |
| R10 Breadcrumb & Navigation | 5 | | |
| R11 RTL & Multilingual | 5 | | |
| R12 FocusPane Isolation | 4 | | |
| R13 Sky View Integration | 6 | | |
| **Total** | **86** | | |

**GO criteria**: 80+ tests pass, zero CRITICAL failures.
