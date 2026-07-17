# FM+ (Focus Mode Plus) — Design Decision Brief

*PJ-114 · Focus mode · for Boss approval before any code is written · design pass 2026-07-17 (Art Director & Team, workflow wf_12bd4169-78c)*

> **Status: awaiting Boss approval of a direction.** No code until a direction is chosen. The in-Focus navigation half is a `/migration`-class change (touches the save/open lifecycle) — this brief is the concept + design; the migration Plan is a second, separate approval gate before that half is built.

---

## 1. Concept recap (the horse)

Focus mode is Constellation's fast, plain-text capture surface — a white paper on a gray table, no toolbar, no formatting, no distractions. **FM+ is an opt-in layer that lets you do two things without ever leaving that quiet surface: (1) right-click a `[[link]]` and rename the note it points to — which heals every link to it across your whole Universe — and (2) follow a link to *wander* from note to note, staying inside Focus the entire time.** It builds no new machinery; it simply reaches the rename engine, the link resolver, and the note-opening path the rest of the app already uses. When FM+ is off, Focus is byte-for-byte what it is today. The one rule that governs everything below: **if FM+ ever started to look or feel like the full editor (NotePane) with a new name, it has failed.**

The guardian's line, verbatim: *FM+ may only add chrome that floats ABOVE the editor and gestures that DELEGATE to code that already exists. The instant it touches the CM6 buffer's rendering, it is NotePane.*

---

## 2. The FM+ toggle — recommended design

A single quiet text token — **`FM+`** — living in the existing bottom-center word-count strip (the `.focus-footer`), sitting beside the word count. **Default OFF, and it is a persisted app setting** (remembered between sessions, and it carries to the second screen). Off: the token sits faint and recessive (like the ghost title), just legible on a deliberate look. On: it lights up with a small filled dot and full opacity — an always-present "FM+ is live" signal so you're never surprised a link is now followable or renamable. It fades away while you actively type and returns on a pause, so nothing intrudes on the writing trance. Click to toggle — no keyboard shortcut (a set-once switch doesn't merit one). The `FM+` mark stays as-is in every language (it's a mode name, like *Sight* or *Map*), but its tooltip is translated into all 15 locales; in Arabic/Hebrew the footer mirrors automatically, and the `+` is isolated so it never reorders. No accelerator means nothing to reconcile between macOS and Windows.

---

## 3. The right-click menu — two options

Both options use the app's **shared menu renderer and builder** (so every label is translated ×15 for free, and the menu clamps to the screen and anchors correctly in right-to-left languages with zero new code). Which `[[link]]` you clicked is found with the **same parser-free line-scan trick the main editor already uses** — never the markdown parser. The choice for the Boss is *how many items appear*.

### Option A — "Bare Essentials" (the lean floor)

**Right-click ON a `[[link]]`:**
1. **Follow** — opens the target note *inside Focus* (reuses the note-open path + Focus remount).
2. **Rename…** — renames the link's target note and rewrites every `[[link]]` to it universe-wide (reuses the one rename engine via the shared rename dialog).
3. **Copy link text** — copies `[[Target]]` to the clipboard (trivial, no engine).

**Right-click on plain text:** nothing FM+-specific (the system already provides cut/copy/paste).

### Option B — "Essentials+" (recommended)

**Right-click ON a `[[link]]`:**
1. **Open in Focus** — follow the link, staying in Focus *(the headline navigation)*.
2. **Rename…** — rename the target + universe-wide cascade *(the headline rename)*.
3. **Copy link text** — copy the `[[Target]]` token.
4. **Copy path** — copy the target note's location on disk (reuses the shared builder's action).

**Right-click on plain text / a selection:**
1. **Cut** · 2. **Copy** · 3. **Paste** — standard capture editing (paste-a-quote is a first-class capture act; reuses the editor's own clipboard).
4. **Rename this note…** — the *only* way to rename the note you're writing from inside Focus, since the file tree is hidden here (reuses the same rename engine).

**Recommendation: Option B.** It keeps the two headline verbs front-and-center, adds only clipboard actions that a capture surface genuinely uses, and — critically — gives you a way to rename *the note you're in* (the tree is hidden in Focus, so without this there is no other way). It is still tiny. If you'd rather start as minimal as possible and add later, Option A is the safe floor and nothing about B blocks a later trim.

### What we CUT from the full Obsidian menu — and why (the "not another NotePane" discipline)

Of the ~22 items in Obsidian's link menu, we drop about 18:

- **Open to the right / Open in new window** — Focus is one single surface with no splits or panes; honoring these would break out of Focus.
- **Add link / Add external link / Edit link / Paragraph▸ / Insert▸** — these *insert or reformat markdown*. That is the toolbar, which Focus deliberately doesn't have. In capture you just type.
- **Paste as plain text** — Focus *is* plain text, so this is identical to Paste. It cannot exist here.
- **Move file to… / Reveal file in navigation** — file-management, not knowledge-formulation; and "reveal in navigation" points at a tree that Focus hides by design.
- **Bookmark / Start presentation / New drawing / Open in default app** — separate subsystems or Obsidian-only features with no capture-surface purpose.

Each cut item failed one test: it either dragged in the editor chrome Focus refuses, or it had no honest job in a fast plain-text capture surface.

---

## 4. In-Focus navigation — recommended interaction

**The gesture (macOS-aware, resolves the real conflict).** Following a link uses **`Mod`-click** — that's **Ctrl-click on Windows, ⌘Cmd-click on macOS**. Focus *already* binds a modifier-click to "select the sentence," so there's a genuine collision. We resolve it **by what's under the pointer, not by inventing a second key**: when you modifier-click, the app checks first whether you clicked *on* a `[[link]]`. **On a link → follow it. Anywhere else → select the sentence, exactly as today.** One gesture, disambiguated by target, nothing new to learn. A keyboard-only alternative — **`Mod`-Enter** with the cursor inside a link — is also offered for hands-on-keys flow, and it has no conflict at all. Note: on macOS, Ctrl-click *is* the system right-click, so both follow and sentence-select must key off ⌘ there — this is the "explicit-Ctrl needs a Mac pass" item flagged in the standing cross-platform rule, and it gets settled here jointly with the existing sentence-select binding so the two never diverge.

**What it feels like.** The white paper and gray table never move. You `Mod`-click a link, the outgoing note's words are replaced by the target's (a brief, calm cross-fade — instant if you've asked the system to reduce motion), the title fades in for the new note, and the word count updates. You've walked through a connection without leaving the trance.

**Back / forward.** Yes — and we **reuse the app's existing history navigation** (`Alt`+←/→ on Windows; the Finder-style `⌘[` / `⌘]` on macOS, since `⌘`+arrows mean line-start/end in text) rather than inventing a new one. Because every follow goes through the normal note-open path, the app's history already records each hop. **No new back-button chrome on the paper** — an optional faint chevron can appear only once you've actually wandered somewhere, and only if you want it.

**The safety promise (plain language).** *The instant before Focus swaps to the next note, your current note is written to disk — every keystroke, even ones typed a fraction of a second ago.* Leaving a note you're editing is exactly the situation that caused a past data-loss bug ("nav-loss"), and it was fixed with a strict "flush before you leave" discipline. FM+ navigation obeys that same discipline on every hop — follow, back, and forward alike. And if a rename cascade is mid-flight ("Updating…"), Focus goes safely read-only until it finishes.

---

## 5. Reuse & safety map — FM+ builds no new engine

FM+ is a thin layer that borrows five things that already exist and are already proven:

- **The rename + universe-wide cascade** — the single existing orchestrator (`handleRenameComplete`). FM+ calls it; it never copies or re-implements it.
- **The link resolver** — turns `[[link]]` text into a real note, spanning every library including federated ones ("it is one Universe").
- **The shared menu renderer + builder** — draws the menu, clamps it to the screen, handles right-to-left, and translates labels ×15.
- **The note-open path + Focus remount** — the exact mechanism the tabs already use; Focus simply re-displays the newly active note.
- **The "flush before you leave" save discipline** — the fix from the earlier nav-loss bug, reused verbatim.

The *only* genuinely new code is a ~15-line helper that finds the `[[link]]` under your click — and even that is **extracted from the main editor, which already does this exact scan**, so both surfaces share one copy rather than duplicating it.

**Free fix folded in (PJ-116).** Today, if you type a title in Focus, it is *silently discarded* — Focus fires the "title changed" signal but the app currently ignores it. Wiring that signal to the rename engine fixes a live data-loss bug with zero new logic, and it ships as part of this work.

---

## 6. How we'll build it safely

The **navigation half touches the save-and-open lifecycle** — the highest-risk, content-integrity area of the app (the same seam as the past nav-loss bug). So it goes through the full **four-phase `/migration` treatment** (Architect → Plan → Build → Audit), is **reproduced as a failing case on the *running* app before it's fixed** (static tests do not count as proof for this class of bug), and must pass the **Editor-Surface Gate** — in particular the "type a keystroke, follow a link, prove the keystroke landed in the note you left and nothing bled into the note you arrived at" check. The **menu half is lighter** and rides the normal review + safety-inspection path — except its two lifecycle-touching actions (Rename and Follow), which ride inside the migration.

**Recommended ship sequence (everything behind the FM+ master toggle, default OFF):**
1. **PJ-116 title-discard fix** — ships first, standalone, unflagged (pure bug fix).
2. **Shared link-finder helper** — extracted and proven against the main editor first (no behavior change).
3. **FM+ toggle + menu shell + no-risk items** (copy path, copy link text) — behind the flag; proves the menu, translation, and right-to-left.
4. **In-Focus navigation (Follow)** — the migration core, with the reproduce-first harness.
5. **Right-click Rename cascade from Focus** — rides the existing rename engine; audited with a linked-note probe pair.

---

## 7. Open decisions for the Boss

- **Menu size — Option A ("Bare Essentials," 3 link items) or Option B ("Essentials+," adds copy-path + clipboard + rename-this-note)?**  → *Recommend **B*** — still tiny, and it's the only way to rename the note you're currently in (the tree is hidden in Focus).
- **Follow gesture — `Mod`-click on a link (Ctrl/Win, ⌘/Mac), disambiguated from sentence-select by what's under the pointer, plus optional `Mod`-Enter?**  → *Recommend **yes to both*** — one gesture to learn, no new modifier, and a keyboard path for flow typists.
- **Back / forward in v1 — reuse the app's existing history keys (`Alt`+←/→ on Windows, `⌘[`/`⌘]` on Mac), with no new on-screen button?**  → *Recommend **yes*** — wandering needs a way home, and this adds zero new chrome.
- **Link appearance — leave `[[links]]` as plain bracketed text (no coloring/underlining)?**  → *Recommend **yes, leave them plain*** — the brackets already show where links are; coloring them would require the exact markdown-rendering machinery Focus forbids, and is the first step toward becoming a second NotePane.
