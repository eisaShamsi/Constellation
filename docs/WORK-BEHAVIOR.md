# Constellation — Work Behavior & Session Protocols

This document defines how we work on Constellation. Every session must follow these protocols.

---

## 1. Session Start Protocol

1. **Always `git pull origin main`** first to sync changes from other devices/sessions.
2. **Check `git log --oneline -5`** to understand recent work.
3. **Read `lab/reports/SESSION-LOG-{latest-date}.md`** to pick up where the last session left off.
4. **Read `docs/LESSONS-LEARNED.md`** — hard-won rules from iterative testing. These override assumptions.
5. **Read `CLAUDE.md`** — project conventions and principle rules.
6. **Read memory files** — check MEMORY.md for cross-session context.

## 1.1 Environment & Push/Pull Protocol

**CRITICAL: This project has TWO environments:**

1. **Local machine** — `E:\مشاريع كلاود\Constellation` on the user's Windows PC. This is where the Tauri app is built and tested. This directory is a git repo synced with GitHub.

2. **Cloud session** — Where Claude Code runs. This is a SEPARATE environment. It can push to GitHub but CANNOT directly access the user's local filesystem.

**The workflow is:**
- Claude codes, commits, and pushes to `origin/main`
- The user's local machine must `git pull origin main` to receive the changes
- The user builds and tests locally (`cargo tauri dev`)

**If Claude is running ON the user's local machine** (indicated by being able to read/write `E:\مشاريع كلاود\Constellation` directly), then push AND pull happen in the same environment — no manual step needed.

**If Claude is running in a cloud environment** (indicated by NOT being able to access `E:\` paths), then after every push, Claude MUST instruct the user to pull. Better yet: ask the user if a previous session is still running locally that can do the pull.

**How to detect which environment you're in:**
```bash
# Try to access the local directory
ls "E:\مشاريع كلاود\Constellation\CLAUDE.md"
# If this succeeds → you're on the local machine
# If this fails → you're in the cloud, user must pull manually
```

**After EVERY push from a cloud session, remind the user:**
```
Pushed to origin/main. On your machine, run:
cd "E:\مشاريع كلاود\Constellation" && git pull origin main
```

---

## 2. Testing Protocol — The Tutorial Rule

**Every time we test a feature (new, updated, or fixed), we MUST prepare a tutorial first.**

The tutorial has two parts:

### Part 1: Define the Feature
Before any testing, explain:
- **What it is** — what the feature does, in plain language
- **Why it exists** — what problem it solves, why it matters to the user
- **Why it matters** — how it fits into Constellation's mission as an extension of the mind

This explanation should be written as it would appear in the **help files and User Manual**. This serves double duty: it tests our understanding AND builds the documentation.

### Part 2: Step-by-Step Walkthrough
Walk through the test scenario in detail:
- **Every click** — which button, which icon, where on the screen
- **Every field** — what to type, what to select
- **Every expected result** — what should appear, what should change
- **Number every step** — so the user can report "Step 5.12 doesn't work"

**Never assume the user knows:**
- Internal component names (say "the book icon in the left dock", not "IndexPanel")
- Technical syntax (say "type a word", not "enter a filter query")
- How to set up test scenarios from brief descriptions

**Example format:**
```
## Feature Name

### What It Is
[Plain language explanation...]

### Why It Matters
[User value proposition...]

### Testing — Step by Step

#### Step 1: Open the Feature
1. Click the **Feature icon** (description of icon) in the left dock
2. The main window should switch to show...
3. You should see...

#### Step 2: Verify Basic Functionality
4. **Do this action** — expected result
5. **Do that action** — expected result
...
```

---

## 3. PCS Protocol (Push + Commit + Standing Order)

After every milestone or phase completion:

### Push + Commit
1. `git add` relevant files (never use `git add -A`)
2. `git commit` with descriptive message + `Co-Authored-By` tag
3. `git push origin main`

### Standing Order (SO)
1. **Milestone tag**: `git tag milestone/<name> <commit>` then `git push origin --tags`
2. **ZIP backup**: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`
3. **Session log**: Update `lab/reports/SESSION-LOG-YYYY-MM-DD.md` with:
   - Phase name and commit range
   - What was built/changed
   - Files created/modified
   - Open items for next session
4. **Help files**: Update `docs/help.uConstellation.World/` and `docs/User Manual.md` with any user-facing changes
5. **All 14 translations**: Update all translated User Manuals (`docs/help.{lang}/User Manual.md`)

**PCS = the safety net.** If the session is cleared or restarted, the next session can pick up exactly where this one left off.

---

## 4. Principle Rules (Non-Negotiable)

### Secure the Winning
If a feature works in one place, extract it into a shared component and reuse it everywhere. Never copy-paste and adapt. One source of truth, tested once, used many times.

### Screens Are Displays, Not Domains
The Second Screen (and any future screens) mount core components and display them — they NEVER re-implement save/load/edit operations. The core editor handles all operations regardless of which window it's in.

### Don't Patch More Than Three Times
If three attempts fail to fix a bug, stop and find the root cause. Don't keep patching symptoms.

### Don't Reinvent the Wheel
Before building anything, check if we already built it. If the working version exists, use it. If it needs adaptation, extract and extend — don't rewrite.

---

## 5. Code Quality Standards

### Before Every Commit
- `npx vite build` must pass clean (frontend)
- `cargo check` must pass clean (Rust backend)
- Type 10 characters rapidly in both NotePane and FocusPane — if there's lag, fix it
- After adding a `$effect`: verify it doesn't fire in a loop
- After adding CSS: resize the window from max to min — if layout breaks, fix it

### Svelte 5 Rules
- Use `$state`, `$derived`, `$derived.by`, `$effect`, `$props` — no legacy Svelte 4 patterns
- Never write a `$effect` that reads and writes the same reactive variable
- Use `$derived` for computed values, `$effect` only for side effects

### Performance Rules
- Zero perceptible lag between typing and screen update
- No `invoke()` calls on the keystroke hot path
- Debounce saves: 1500ms minimum
- Virtualize every list that can exceed 50 items
- Pre-cache module-level Decoration objects — never create inside builders

### i18n
- All user-facing strings go through `$t()`
- Update all 15 locale files (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh)

### RTL Support
- Use `dir` attributes and `detectDir()` from `$lib/utils`
- Use CSS logical properties (`inset-inline-start`, `padding-inline`, etc.)
- Test with Arabic content

---

## 6. Architecture Principles

### File Over App
`.md` files on disk are the source of truth. The app is just a window.

### Local-First
All data stays on the user's device. No telemetry, no tracking, no cloud dependency.

### Constraint as Design
Don't add features just because you can. Every feature must justify its existence.

### Language-First by Design
Constellation supports all languages simultaneously, from the ground up.

---

## 7. Constellation's Essence

Constellation is an **extension of the mind**. It serves the user's thinking, not the other way around.

**Natural flow:**
1. User opens Constellation → **focus on writing**. No clutter, no distractions.
2. With a second monitor → **distractions move to the Second Screen**. The main window becomes pure writing space.
3. When the user **deliberately** activates a capability (Map, Index, Star View) → the Second Screen provides the **extended experience**.

**Simple by default, powerful on demand.**

---

## 8. CE (Cognitive Engine) Architecture

- **Layer 1**: Structural Cognition (11 phases) — COMPLETE
- **Layer 2**: Constellation Map — radial knowledge visualization
- **Layer 3**: Constellation Lens — network analysis engine (future)
- **Layer 4**: AI Discovery — embeddings, semantic links, AI-powered insight (future)

Each layer builds on the previous. The Map gives shape, the Lens finds patterns, AI generates connections you couldn't see yourself.
