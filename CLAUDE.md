# Constellation — Claude Instructions

## Project
Tauri v2 desktop app (Rust + SvelteKit/Svelte 5) — a Personal Knowledge Formulation system. Not management — formulation.

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

### Rule 8: Write-Time Derivation
> **Every computed view in Constellation is maintained at write time, not read time.**
>
> When a note changes, every derived surface that depends on it updates in the same transaction. The app does not recompute on boot. It does not recompute on panel open. It reads what's already stored.

- **Canonical example**: SQLite FTS5. The `notes_fts` virtual table is kept in sync with `note_meta` via the `note_meta_ai` / `note_meta_ad` / `note_meta_au` triggers in `init_db`. Search is instant because the index is always current. No `scan_*` command is needed.
- **Canonical use case**: the Index panel (`read_index_entries`) reads directly from the FTS5 vocabulary dictionary (`notes_vocab` virtual table — an `fts5vocab(notes_fts, 'row')` view). Term expansion (`read_term_mentions`) is a single FTS5 `MATCH` query. Nothing is rebuilt on boot.
- **Don't**: write a `scan_*_library` or `rebuild_*` command that re-walks the Universe to produce a derived view. If you find yourself doing that, stop — the shape is wrong. That path is how LL-XXX happened (hand-rolled term index that OOMed and required a 3 GB WAL vacuum to recover).
- **Do**: persist the derived view, wire a trigger or hook on the source-of-truth write path, let reads be cheap lookups.
- **First-time population**: when a new surface is added to an existing Universe, the one-off back-fill should run in the background after paint, with progress in the status bar — and must be resumable.
- **Where this rule must be applied next** (audit pending): Sky View (`skyNodes`/`skyLinks` rebuilt on every boot), Backlinks/Outgoing panels (recomputed on tab focus), Tag browser (scanned on open), Sight dashboard, sidebar star counts, Map. Each of these should persist its derived data and maintain it via triggers or watcher hooks.
- **Hard constraint**: no new feature may regress boot time, typing latency, or IPC responsiveness. Measure before/after on a large Universe (7,600+ notes) before committing.

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

### Knowledge Formulation (Not Management)
- Constellation is a **Personal Knowledge Formulation** system — not a file manager.
- Knowledge is not about storing information. It is about **connecting, challenging, synthesizing, and building** understanding.
- Links are **living vessels** — they carry type, annotation, weight, confidence, and temporal data.
- Links follow a lifecycle: Spark → Birth → Growth → Maturity → Dormancy → Renewal/Archival.
- The 7 link types are the **cognitive vocabulary**: supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of.
- The search engine is a **diagnostic instrument** for intellectual life — not a file finder.
- The Five Acts of Knowledge Creation: Observation → Connection → Tension → Synthesis → Conviction.
- Full specification: `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`

### The Living Link Architecture
- Links are **first-class knowledge objects** with the `LINK` file kind (`YYYYMMDDTHHMMSSZ_LINK_XXXX.md`).
- **Dual-layer storage**: LINK files on disk (source of truth) + `note_links` SQLite table (fast index).
- **Eight properties**: Type, Direction, Annotation, Weight, Confidence, Created, Last Traversed, Traversal Count.
- **Four confidence levels**: hypothesis → evidence → established → contested.
- **Weight is earned through use**: logarithmic growth on traversal, 5% monthly decay without use.
- Links must be **searchable by all properties** in the user's own language.
- Every link operation must be **reversible** — archival, not deletion.

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

### Constellation Knowledge Hierarchy
Constellation organizes knowledge in a five-level hierarchy — no other PKM system has this depth:

```
Universe (root)
  └── cUniverse (child universe)
       └── Library
            └── Folder
                 └── Note
```

- **Universe**: The top-level container. Named by the user. Contains all libraries, settings, bases, bookmarks. One per Constellation instance. Stored as a directory with `universe.json`.
- **cUniverse (Child Universe)**: A linked Universe that contributes its libraries to a parent. Enables federation — viewing notes from multiple independent Universes in one window.
- **Library**: A complete, self-contained knowledge base (equivalent to an Obsidian vault). Has its own color, appearance, tags, links, and index. Registered in `libraries.json`. Multiple libraries coexist in one Universe. Libraries are never copied — Constellation reads them in place.
- **Folder**: A subdirectory within a Library. Organizational structure only. Supports nesting.
- **Note**: A single `.md` file with optional YAML frontmatter. The atomic unit of knowledge.

**Library ≠ Folder.** A Library is a first-class citizen with its own identity. A Folder is just file organization inside a Library. The "New Library" button in the sidebar toolbar creates or links libraries — distinct from "New Folder".

### Smooth Transitions
- NotePane and FocusPane edit the same `.md` file. Switching between them must be seamless.
- What the user types in Focus (plain text markdown) renders beautifully in NotePane.
- No data loss on mode switch. Save before transition, load after.

---

## The Migration Rule (major changes)

Any change that touches **schema, core data flow, cross-surface invariants, or multiple subsystems** goes through the four-phase `/migration` workflow before any code is written:

1. **Architect** — map the territory, enumerate design options with speed/effort/risk, list the invariants that must not break.
2. **Plan** — phase-by-phase steps, each landable as one commit, each with a verification clause. User approves the plan.
3. **Build** — implement step by step, verify after each step, commit tied to the plan. Run `/simplify` on the final diff.
4. **Audit** — three agents in parallel check invariants, drift (new guards the system doesn't know about — see LL-023), and migration path (first-boot, schema mismatch, mid-backfill interrupt, rollback).

The four phases are not ceremony. They are the verification protocol that keeps Constellation from shipping a regression that takes three sessions to undo. The cost of running them is ~30 minutes of agent time. The cost of skipping them is the entire iteration that built the feature that broke.

Single-file refactors and local bug fixes do not need `/migration` — `/simplify` is sufficient. The rule of thumb: does the change cross subsystem boundaries (Rust ↔ Svelte, schema ↔ code, write path ↔ read path)? If yes, `/migration`. If no, don't.

Definition and agent briefs: `.claude/skills/migration.md`.

---

## Standing Order (SO)
After every phase, step, or significant commit:
1. Update `lab/reports/SESSION-LOG-YYYY-MM-DD.md` with: phase name, commit hash, test results, bugs fixed, open items.
2. Update **help files** (`docs/help.uConstellation.World/`) and **User Manual** (`docs/User Manual.md` + all 14 translations in `docs/help.{lang}/`) with any user-facing changes.
3. This is the safety net — if the session is cleared or restarted, the next session can pick up exactly where this one left off.
4. The SO also includes running `/simplify` (code review) after each phase.
5. **State-of-standing record before any pivot or major triage.** When the user says "where do we stand?", "let's regroup", asks for a backlog/inventory, or asks to redirect priorities: write a snapshot record into the current day's session log capturing (a) what's verified-shipped and protected, (b) what's at-risk / in-flight / uncommitted in the worktree, (c) what's known-broken, (d) what's pending but not started, and (e) any documentation drift. Never proceed to the new direction until that record is written. The record lets a fresh session — or a fresh you — pick up the exact state without rediscovering it from git log + screenshots.

**PCS = Push + Commit + SO** — always includes help files and user manual updates.

## Testing Instructions Rule (top principal)

**Every test instruction is a tutorial.** When asking the user to test ANY feature — new, updated, or fixed — the message must read like a help-file entry the user could hand to someone unfamiliar with Constellation:

1. **Define the feature first.** What it is, why it exists, why it matters in plain language. This is the same paragraph that would appear in `docs/help.uConstellation.World/` or `docs/User Manual.md`. If the test is for a fix, define what the user-visible behavior was broken, what it now does, and why the fix matters.
2. **Then walk through it click by click.** Every navigation step ("In the sidebar, right-click the file you renamed → choose Rename"), every field ("type the new title — for this test, use `Foo v2`"), every expected result ("the file's body now shows `[[Foo v2]]` where it previously said `[[Foo]]`"), every observable cue ("you'll briefly see the cursor stop blinking — that's the cascade running"). Plain language only. No internal component names (`NotePane`, `+layout.svelte`, `handleRenameComplete`) unless the user asks for technical detail.
3. **Pre-state, action, post-state.** Each step has a known starting point, a single action, and a single expected outcome. The user should never have to guess "what was supposed to happen here?" If a step has multiple observable outcomes, list them.
4. **Failure modes spelled out.** "If you see X instead, that means Y is broken." This lets the user report meaningfully rather than just "it didn't work."

The user is a human, not an AI. They are the Boss, not a developer in your team. They should never have to read source, parse internal jargon, or set up test scenarios from a sentence-long description.

This rule sits at the same tier as Working Agreement #4 (validate before shipping) and Standing Order #5 (state-of-standing record). It overrides terseness and overrides delivery pressure.

## Plan Approval = Build Approval (top principal)

Once the user approves a plan (a `/migration` Phase 2 plan, or any explicitly-laid-out step sequence), Claude cascades through the build steps autonomously. Claude does NOT seek per-step approval; doing so wastes the user's time and signals a lack of confidence in the plan that was already approved.

Stops happen only at:
1. **User-testable verification clauses.** Every step that produces something the user can test must pause for testing, articulated per the Testing Instructions Rule above.
2. **Genuine architectural surprise.** If during build a step reveals an unmapped invariant or a contract change not in the plan, stop and surface it. The user may approve a deviation; they should not be ambushed.
3. **Plan completion.** At the end of the cascade, summarize what shipped + the next decision point.

The Standing Order session-log discipline still applies between steps — log each `§NNN` commit as it lands. But that's record-keeping, not approval-seeking.

## Backup Routine
After each successful milestone:
1. **Git tag**: `git tag milestone/<name> <commit>` then `git push origin --tags`
2. **ZIP archive**: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`
3. To restore: `git checkout milestone/<name>` or unzip the archive.

---

## Working Agreement (ground rules, non-negotiable)

1. **Do the work yourself. Don't offload it to the user.** If you can run a command, query a DB, read a log, diagnose a bug, or write a test — do it. The user is the Boss, not the lab assistant. The only thing you ask of them is what genuinely requires a human: interacting with the running Constellation GUI (create a note, click a button), making design decisions, approving a plan, confirming a release is ready. Everything else — SQL queries, file inspection, log greps, schema checks, build verification — is your job. If you catch yourself writing "please run this query and tell me the result," stop and run it yourself via Bash + sqlite3 (or equivalent).

2. **One location: `E:\مشاريع كلاود\Constellation` on branch `main`.** This is the root for every read, write, commit, build, and test — now and in the future. It is the de-facto "main" working directory. Do not introduce worktrees, alternate checkouts, or parallel paths. If a session starts somewhere else (e.g. a `.claude/worktrees/` subfolder), operate via absolute paths into the primary location — never have the user switch directories to compensate for a session-spawn quirk. Commits land here, pushes come from here, builds run here.

3. **The user is a non-technical IT Boss.** Explanations default to plain language: what it does, why it matters, what's going to happen next. No internal component names, no assumed familiarity with Rust / Svelte / SQLite internals, no "just run this SQL and report back" busywork. Test instructions follow the Testing Instructions Rule above — define the feature first, then walk through interaction by interaction. Technical detail is available on request, not pushed by default.

4. **Don't ship changes you haven't proven safe against the whole architecture.** You are the consultant/engineer/SME, not a feature-pleaser. Before every code change — every function, subfunction, routine, every line you add or alter — answer in writing: *what wires does this touch, and what will it cut?* If the change crosses a subsystem boundary or interacts with a write path / lifecycle / reactivity / IPC contract, you MUST run a full architectural-impact review before submitting it: spawn as many parallel agents as needed (Explore for read paths, Plan for design verification, code-reviewer for cross-cutting risk) to map the call graph, list every consumer that will see the new behavior, and identify every invariant that could break. The user's job is to direct intent and approve plans; bug-creation by oversight is on you, not on them. If the review reveals risk you can't characterize, stop and surface it — never ship "let's see what happens" code. This rule overrides delivery pressure: a slower, proven change is always preferred to a fast change that introduces a regression. (The MIG-006 §3-expanded → BUG-015 incident, where a value-prop → doc sync `$effect` raced with `{#key}` onDestroy and corrupted target body content, is the canonical violation this rule prevents.)

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
- **Additional screens are displays, not domains.** Second screen (and any future screens) mount core components and display them — they NEVER re-implement save/load/edit operations. The core editor handles all operations regardless of which window it's in. No `onNoteSaved` re-reads, no `loading = true` on file changes, no competing tab management.

## Testing Instructions Rule
When asking the user to test ANY feature (new, updated, or fixed):
1. **Define the feature first** — explain what it is, why it exists, and why it matters (as it would appear in the help files / User Manual)
2. **Then walk through step by step** — explain every click, every field, every expected result in plain language
Never assume the user knows internal syntax, component names, or can set up test scenarios from brief descriptions. The user is a human, not an AI.
