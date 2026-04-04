# Constellation — Claude Instructions

## Project
Tauri v2 desktop app (Rust + SvelteKit/Svelte 5) for managing Markdown note libraries.

## Before Starting Work
1. Always `git pull origin main` first to sync changes from other devices/sessions.
2. Check `git log --oneline -5` to understand recent work.
3. Consult `docs/LESSONS-LEARNED.md` — hard-won rules from iterative testing. These override assumptions.

## Conventions
- **Terminology**: Use "Library" everywhere, never "vault" (except for Obsidian import compatibility).
- **Svelte 5 runes**: Use `$state`, `$derived`, `$derived.by`, `$effect`, `$props` — no legacy Svelte 4 patterns.
- **i18n**: All user-facing strings go through `$t()`. Update all 15 locale files (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh).
- **RTL support**: Use `dir` attributes, `detectDir()` from `$lib/utils`. Flip chevrons/arrows in RTL.
- **Cross-window sync**: Second screen is a separate Tauri window. Use `emit`/`listen` from `@tauri-apps/api/event` for communication. Settings changes must call `notifySettingsChanged()` to propagate.
- **CSS**: NotePane uses `.pane` (not `.note-pane`). Override child styles with `:global()` + `!important` when needed.
- **Fonts**: Global fonts from `appSettings` (interfaceFont, textFont, monoFont, fontSize, scriptFonts). Per-library fonts from `libraryAppearances`. Both must be applied in main window AND second screen.
- **Units**: Use `px` for layout discussions. Code may use `rem`/`em` for font accessibility.

## Editor Parity Rule
- **All note views must have identical markdown rendering.** Standard NotePane and any future note types must share the same CM6 extensions (livePreview, callouts, syntax highlighting, etc.).
- **Exception**: FocusPane is plain text only — no markdown parser, no syntax highlighting, no decorations. Focus = capture ideas fast.
- New markdown features added to the editor MUST work in ALL note modes (except FocusPane) — never add a feature to only one view.
- The shared extension set lives in `$lib/editor/` and is imported by every editor instance.

---

## ⚡ Performance Rules — "Fast Software, the Best Software"

> "Speed and reliability are often intuited hand-in-hand. Speed is a proxy for general engineering quality." — Craig Mod
> "If you want to create digital artifacts that last, they must be files you can control." — Steph Ango (kepano)

### Rule 1: Every Keystroke Must Be Instant
- **Zero perceptible lag** between typing and screen update. If the user notices delay, it's a bug.
- FocusPane: NO markdown parser, NO syntax highlighting, NO decorations. Plain CM6 + history + line wrapping only.
- NotePane: Decorations rebuild only on `docChanged`, `selectionSet`, or `viewportChanged` — never on every frame.
- ViewPlugin `update()` must use a **line-change guard** for selectionSet — never rebuild on every cursor move:
  ```typescript
  update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) { this.decorations = rebuild(update.view); return; }
      if (update.selectionSet) {
          const newLine = currentLine(update.view);
          if (newLine !== this.lastLine) { this.lastLine = newLine; this.decorations = rebuild(update.view); }
      }
  }
  ```
- **Pre-cache module-level Decoration objects** — never create `Decoration.mark()` / `Decoration.replace()` inside `buildDecorations()`. Allocate once at module load, reuse on every rebuild.
- **Fast-path on docChanged**: call `this.decorations.map(update.changes)` first, then debounce the full rebuild (≥300ms). This keeps typing instant even before the rebuild fires.

### Rule 2: No $effect Loops
- **NEVER** write a `$effect` that reads and writes the same reactive variable.
- **NEVER** write a `$effect` that watches a prop it also modifies via callback (the FocusPane value sync bug).
- Use `$derived` for computed values, `$effect` only for side effects (DOM manipulation, Tauri IPC, timers).
- When syncing editor content with a prop, use a `lastInternalValue` guard to prevent echo loops.
- If you must write to `$state` inside `$effect`, wrap the write in `untrack()`.

### Rule 3: No Heavy Work on the Main Thread
- Vault indexing, search, file I/O → **Rust side** via Tauri commands. Never parse 1000+ notes in JS.
- CM6 syntax tree iteration: only process `view.visibleRanges`, never the full document.
- Large decoration sets: use `RangeSetBuilder` (sorted insert), never `Decoration.set()` with unsorted arrays.
- Debounce saves: 1500ms minimum. Never save on every keystroke.
- **IPC boundary rules** (see `docs/IPC-CONTRACT.md`):
  - Zero `invoke()` calls on the keystroke hot path — ever.
  - Debounce search queries ≥300ms; cancel the previous call if a new one arrives.
  - Batch index update events — never one IPC call per character typed.
  - Prefer Tauri events (Rust → frontend push) over polling `invoke()`.
- **Virtualize every list** that can exceed 50 items: file tree, search results, backlinks, tag browser, command palette. Render only visible rows regardless of vault size.

### Rule 4: No Memory Leaks
- Every `setTimeout`/`setInterval` → clear in `onDestroy`.
- Every `addEventListener` → remove in `onDestroy`.
- Every `EditorView` → `.destroy()` in `onDestroy`.
- Every Tauri `listen()` → call the unlisten function in `onDestroy`.
- `requestAnimationFrame` → cancel with `cancelAnimationFrame` in `onDestroy`.
- Never create circular references with `Rc`/`Arc` in Rust without `Weak`.

### Rule 5: Minimal DOM
- CM6 handles its own DOM — don't fight it with extra wrappers.
- Hide elements with `display: none`, not by removing/re-adding DOM nodes.
- Avoid `:global()` CSS that triggers layout recalculation across the tree.
- Use `flex` and `grid` for layout — no JavaScript-based positioning.
- Tab scroll: use native CSS `overflow-x: auto`, not JS scroll emulation.

### Rule 6: No Unnecessary Imports
- Don't import `@codemirror/language-data` (all languages) in FocusPane — it pulls 500KB+ of parsers.
- Don't import full icon libraries — use inline SVGs.
- Tree-shake aggressively: import only what you use from each package.
- Lazy-load heavy features (graph view, PDF export, AI) — don't bundle them in the main chunk.

### Rule 7: Test Before Commit
- After every code change: type 10 characters rapidly in both NotePane and FocusPane. If there's lag, fix it before committing.
- After adding a `$effect`: verify it doesn't fire in a loop by adding a temporary `console.log` and checking it fires ≤2 times on load.
- After adding a CM6 extension: open a 5000-word note and scroll. If scrolling stutters, optimize or remove.
- After adding CSS: resize the window from max to min. If layout breaks, fix before committing.

---

## 🏗️ Architecture Principles

### File Over App
- `.md` files on disk are the source of truth. The app is just a window.
- Never modify file content silently. Every change must come from explicit user action.
- Never lock files in proprietary formats. Everything is standard Markdown + YAML frontmatter.
- The vault index is ephemeral — rebuilt from files at startup, updated incrementally at runtime.

### Local-First
- All data stays on the user's device. No telemetry, no tracking, no cloud dependency.
- Sync is the user's choice (Git, Syncthing, iCloud) — Constellation doesn't own it.
- The app must work fully offline, instantly, always.

### Constraint as Design
- Don't add features just because you can. Every feature must justify its existence.
- FocusPane has no toolbar, no properties, no markdown rendering — that IS the design.
- Prefer CSS-only solutions over JavaScript. Prefer Rust over JavaScript for computation.
- When in doubt, do less.

### Language-First by Design
- Constellation supports all languages simultaneously, from the ground up by design.
- Per-line bidirectional text (bidiPlugin) is a core architectural feature, not an add-on.
- Every editor view — NotePane, FocusPane, and any future view — must support multilingual, mixed-script content natively.
- Never build a single-language assumption into layout, fonts, cursor behavior, or input handling.

### Smooth Transitions
- NotePane and FocusPane edit the same `.md` file. Switching between them must be seamless.
- What the user types in Focus (plain text markdown) renders beautifully in NotePane.
- No data loss on mode switch. Save before transition, load after.

---

## Standing Order (SO)
After every phase, step, or significant commit:
1. Update `lab/reports/SESSION-LOG-YYYY-MM-DD.md` with: phase name, commit hash, test results, bugs fixed, open items.
2. Update **help files** (`docs/help.uConstellation.World/`) and **User Manual** (`docs/User Manual.md` + all 14 translations in `docs/help.{lang}/`) with any user-facing changes.
3. This is the safety net — if the session is cleared or restarted, the next session can pick up exactly where this one left off.
4. The SO also includes running `/simplify` (code review) after each phase.

**PCS = Push + Commit + SO** — always includes help files and user manual updates.

## Testing Instructions Rule
When asking the user to test ANY feature (new, updated, or fixed):
1. **Define the feature first** — explain what it is, why it exists, and why it matters (as it would appear in the help files / User Manual)
2. **Then walk through step by step** — explain every click, every field, every expected result in plain language
Never assume the user knows internal syntax, component names, or can set up test scenarios from brief descriptions. The user is a human, not an AI.

## Backup Routine
After each successful milestone:
1. **Git tag**: `git tag milestone/<name> <commit>` then `git push origin --tags`
2. **ZIP archive**: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`
3. To restore: `git checkout milestone/<name>` or unzip the archive.

---

## Don't
- Don't use preview/screenshot tools unless essential.
- Don't add unnecessary abstractions or over-engineer.
- Don't use "vault" terminology in new code.
- Don't add a feature that makes the app slower.
- Don't commit code with known `$effect` loops.
- Don't import heavy libraries in FocusPane.
- Don't use `position: absolute` for layout — use flexbox/grid.
- Don't write CSS with magic pixel numbers without documenting why (e.g., `margin-inline-start: -9px /* align tab to paper edge */`).
- **Don't patch the same bug more than three times.** If three attempts fail, stop and find the root cause (LL-014).
- Don't create `Decoration.mark/replace/widget` inside a decoration builder function — pre-cache at module level.
- Don't call `invoke()` from a CM6 ViewPlugin, an input event handler, or any synchronous hot path.
- **Don't duplicate working code by copy-pasting and adapting.** If a feature works in one place, extract it into a shared component and reuse it everywhere. Secure the winning — one source of truth, tested once, used many times.
