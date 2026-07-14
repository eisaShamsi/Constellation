# PJ-106 — Arabic/RTL typing: Boss-reported symptoms (verbatim, 2026-07-14)

These are the CONCRETE, reproducible symptoms the Boss gave. Each becomes a named
Reproduce-First recipe (Recipe T-series) in the Plan phase; the Architect option
table must state which option fixes which. The Boss writes Arabic daily on Windows.

## Round 1 (initial report)
- Home and End are not acting as they should.
- Cannot navigate through a line or a paragraph.
- Cannot select a word, a sentence, a line, a whole paragraph, or a whole page.
- Struggling to write; **worse when bilingual** (an Arabic note containing Latin characters).

## Round 2 (precise symptoms — the load-bearing detail)
1. **Empty-line caret sits on the LEFT after switching to the Arabic keyboard.**
   "When I switch to the Arabic keyboard, the cursor remains at the beginning of the
   line on the left side." → An empty / start-of-line caret does not adopt RTL base
   direction; it should sit at the **right** edge (the reading start for RTL). Likely:
   line base-direction is decided by content (first-strong), and an empty line (or the
   pre-type state) defaults to LTR regardless of the active keyboard/IME. **The keyboard
   language is not (and per Unicode should not be) the signal — but the empty line must
   still present an RTL caret when the note/paragraph context is RTL.** Policy question
   for the design: per-note default direction vs per-line first-strong vs an explicit
   RTL toggle.
2. **End key jumps to the WRONG location on an Arabic line** (not the visual/logical end).
   "if I want to move the cursor to the end of the line, by using the End key, the cursor
   jumps to the wrong location, not to the end of the line." → the classic CM6 RTL
   Home/End logical-vs-visual defect: `cursorLineBoundary{Forward,Backward}` resolve to the
   wrong visual edge on an RTL line. THE headline fix.
3. **A Latin character at the END of an Arabic line breaks caret direction.**
   "the cursor lost their direction, as if it doesn't know if it is in Arabic or Latin
   context." → the bidi boundary at the line's trailing LTR run: caret affinity /
   direction at the run boundary is ambiguous. Needs bidi-isolate handling + correct
   caret affinity at direction boundaries.
4. **Word / sentence / paragraph selection is difficult** on RTL / mixed text.
   → selection-by-unit commands (word/sentence/paragraph) misbehave across bidi runs.

## Testability split (for Reproduce-First)
- CM6 STATE-level (headless vitest, EditorState + dir=rtl): line-boundary command
  resolution (End/Home target offset), word/paragraph selection ranges, first-strong
  base-direction of a line.
- RENDER/MEASURE-level (running app + Boss test): the visual caret POSITION (left/right
  edge), caret behaviour at a trailing LTR run boundary — coordsAtPos/bidi spans need
  layout, so these are the staged Boss-test items, not pure unit tests.

## Notes
- The prior "Arabic callout End/Home caret" item was ruled stop-patching — this /migration
  supersedes it: fix the CLASS (bidi motion/selection) at the editor core, not another patch.
- Windows convention the Boss's muscle memory expects: Home/End on an RTL line go to the
  reading start/end (visual right/left respectively for RTL). Confirm against Word/browsers
  in the WA#5 research.

## Round 3 — Boss directive: the direction-toggle shortcut (2026-07-14)

**"Whenever a user presses Right Ctrl + Shift, the paragraph switches to 100% RTL, and vice versa (Left Ctrl + Shift = LTR)."**

Exact spec (this is the Microsoft Word / Windows convention):
- **Right Ctrl + Shift → the current paragraph becomes 100% RTL** (a HARD, explicit
  direction override — not the first-strong auto heuristic).
- **Left Ctrl + Shift → the current paragraph becomes 100% LTR** (hard override).
- Scope = the **paragraph** the caret is in (and every paragraph a selection spans).
- "100%" = an EXPLICIT per-paragraph direction that WINS over automatic detection.

Design points the Plan (Part B) MUST resolve — flag to the Boss if a decision is needed:
1. **Left vs Right Ctrl detection.** A normal CM6 keymap keys on the logical key and
   CANNOT tell Left Ctrl from Right Ctrl. This needs a custom keydown handler reading
   `KeyboardEvent.code` (`ControlRight` vs `ControlLeft`) / `.location`. Windows may ALSO
   use Left/Right Ctrl+Shift as the OS keyboard-layout switch — spot-check whether WebView2
   delivers the keydown to us before the OS consumes it; if the OS eats it, offer a fallback
   binding.
2. **Persistence.** Does the 100% override PERSIST in the note (survives close/reopen, syncs)
   or is it a live-session visual only? Markdown has no native per-paragraph `dir`. Options
   to weigh (WA#5): a leading RLM/LRM control mark; an HTML `<div dir=…>`/span; a frontmatter
   paragraph-direction map; a zero-width sentinel. Must round-trip byte-safely (INV-6 — but
   THIS is an explicit user WRITE, distinct from navigation) and stay interoperable (.md files
   remain plain, per File-Over-App). Present the options to the Boss with the tradeoffs.
3. Interaction with Part-A's automatic per-line direction: the hard override takes precedence;
   a paragraph with no override keeps first-strong auto behavior.
