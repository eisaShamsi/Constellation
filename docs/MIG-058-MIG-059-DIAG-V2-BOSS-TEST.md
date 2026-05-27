# MIG-058 + MIG-059 Diagnostic v2 Boss-test

**Purpose: capture hard evidence so the NEXT fix is sourced, not guessed.** No behavior changes beyond logging. After we see the diagnostics.log lines, the actual fix is small and targeted.

---

## Stage 0 — Install

```
E:\مشاريع كلاود\Constellation\src-tauri\target\release\bundle\nsis\Constellation_0.1.0_x64-setup.MIG058-059-DIAG-v2.exe
```

1. Close Constellation.
2. Open File Explorer → `E:\Constellation Universes\Eisa Universe\.constellation\`
3. **Delete `diagnostics.log`** (fresh slate so we see only this test's lines).
4. Install the new build.
5. Launch.

---

## Stage 1 — Backend diagnostics fire during federation attach

Wait **15 seconds** after launch (federation attaches, per-cUniverse diagnostics run). No user action needed.

What's being captured (silently, into diagnostics.log):
- `sqlite_stat1` contents for cu1 — what stats does the planner actually have?
- Forced `ANALYZE main` timing
- `sqlite_stat1` contents AFTER analyze — did anything change?
- `EXPLAIN QUERY PLAN` for the canonical 9-term OR query

Tag in the log: `[mig-059-diag]`.

---

## Stage 2 — Trigger a search and capture per-branch timing

1. Press **Ctrl+O** to open the QuickSwitcher.
2. **Paste `الرباط`** (don't type — we want a single, well-defined search event for timing).
3. Wait until results appear.
4. Press **Escape** to close.

What's being captured (during the search):
- For each branch (main + cu1): time in ms + row count returned

Tag in the log: `[mig-059-diag-search]`.

---

## Stage 3 — Trigger the Arabic input truncation and capture keystroke events

1. Press **Ctrl+O** to open the QuickSwitcher (it should be empty — close + reopen if not).
2. **Slowly type `الرباط`** — 300-400ms between each character. Do NOT paste. Type each character one at a time.
3. After typing all 6 characters (or however many landed), look at the input box.
4. Note in your reply: **how many characters landed in the input** (e.g. "5 characters: الربا", or "6 characters: الرباط", or "less").
5. Press **Escape** to close.

What's being captured (during typing):
- Every keystroke: keyCode, isComposing, current `input.value`, current `query` state
- Every input event: inputType (insertText vs insertCompositionText), data, isComposing, input.value, query, composing_flag
- Every composition event (start / update / end): data, input.value, query

Tag in the log: `[mig-058-diag]`.

---

## Stage 4 — Paste the diagnostics

1. Open `E:\Constellation Universes\Eisa Universe\.constellation\diagnostics.log` in Notepad.
2. **Select all** (Ctrl+A), **copy** (Ctrl+C).
3. Paste back in our chat.

Also tell me:
- **How many characters were in the input box after Stage 3?**
- **How long Stage 2's search took (rough seconds)?**

---

## What I'll learn from the diagnostic data

### For MIG-059 (the speed issue)

Looking at the `[mig-059-diag]` lines:

| If sqlite_stat1 shows... | Then... |
|---|---|
| 0 rows for notes_fts shadow tables | ANALYZE doesn't reach FTS5 shadow tables by default — need `INSERT INTO notes_fts(notes_fts) VALUES('optimize')` |
| Many rows including `notes_fts_*` entries | Stats are present; bottleneck is elsewhere — likely FTS5 vtable cold-start at query time |
| Plan output shows "SCAN notes_fts" or similar | Linear scan — needs index hint or different query shape |
| Plan output shows "SEARCH notes_fts USING (some idx)" | Plan is good; bottleneck is purely I/O |

Looking at the `[mig-059-diag-search]` lines:

- If `branch=main time_ms` is fast (~50ms) and `branch=cu1 time_ms` is slow (15000+ms): cu1's Connection has a problem that main's doesn't. Different fix needed.
- If both branches are slow: the lexical_search function itself is slow regardless of which Connection runs it — that's an FTS5/lexicon expansion issue.

### For MIG-058 (the truncation)

Looking at the `[mig-058-diag]` lines for the Stage 3 slow-typed `الرباط`:

| Pattern | Diagnosis |
|---|---|
| Every keystroke fires, `input.value` accumulates the full word, but `query` lags behind | Svelte bind:value writes are being lost — needs manual oninput |
| Some keystrokes don't fire `keydown` events at all | WebView2 dropping events at the OS layer (Tauri #3136 territory) |
| `compositionstart` fires for Arabic | Arabic IS going through IME — the React #34485 / Vue v-model composition-gate pattern applies |
| `compositionstart` never fires | Arabic 101 is direct keystrokes as Agent 4's research said; truncation is at a different layer |
| `inputType=insertCompositionText` appears | IME composition is happening; gate the search effect on it |

Whichever pattern shows up tells us exactly which fix is right. Each fix is a few lines of code.

---

## Don't worry about

- The volume of log lines (we expect ~30-80 lines for this test, easily fits in a chat paste).
- The search being slow during Stage 2 — that's expected and we're measuring it.
- The truncation in Stage 3 — that's expected and we're capturing why.

The point of this stage is to STOP GUESSING. Once you paste the data, the fix becomes one-line targeted instead of speculative.
