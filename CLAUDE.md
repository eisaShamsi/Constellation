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
- The 8 link types are the **cognitive vocabulary**: supports, contradicts, causes, exemplifies, generalizes, derives-from, part-of, supersedes. (Plus `associative` as the default/null type — that's 9 typed-link names total but only 8 carry semantic meaning. `supersedes` was added by MIG-022 §A.2 per gap-analysis §6.1: "this note replaces an earlier stance" is a first-class typed relationship between notes, not a flat YAML scalar.)
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

### Form-Aligns-To-Purpose *(top-principal — 2026-05-19)*

Every visual element, interaction, or computational layer in Constellation must serve its core cognitive purpose. Each function's design and behavior must be aligned with what it answers. Do NOT add visual filler, decorative axes, padding-only positions, or non-meaningful spread to occupy space that the chosen geometric primitive *affords* but the cognitive content does not *require*. If the chosen primitive has degrees of freedom that the answer doesn't fill, change the primitive — don't fill the freedom with noise to look complete.

**Special application — traditions.** A tradition in Constellation Sight is a *cognitive lens* — each one frames knowledge through a specific epistemic grammar (Aristotelian maturity gradient; masādir kinds-of-proof; pramāṇa valid means of knowing; PaRDeS interpretive levels; Mencian sprouts; Mohist sān biǎo; etc.). The visual rendering of a tradition must express *that specific grammar and nothing else*. Don't import dimensions from other traditions just because the dome geometry has them available; if a tradition's grammar is purely radial, the angular axis vanishes from its rendering; if a tradition's grammar is categorical, within-category positions are not "free space" to fill with hash-jitter — they are *cognitive non-existence*. The tradition's purpose is its visual contract.

**Canonical violation prevented**: 2026-05-19 — proposed hash-jittering notes inside Aristotelian's stratum rings to "use" the angular axis the dome geometry exposes, when Aristotelian's actual epistemic grammar is purely radial (a maturity gradient) and the angular axis has no meaning in that frame. The right move is to change the rendering primitive so the unused dimension goes away — not to fill it with synthetic noise so the geometry "looks complete." Same principle applies whenever a categorical tradition currently jitters within-cell; the fix is the primitive, not the filler.

This rule sits alongside Constraint as Design (which forbids adding features without justification). Form-Aligns-To-Purpose forbids adding *visual / structural noise* within features without cognitive justification. Together they mean: every feature must justify its existence AND every part of every feature must justify its presence within it.

### Style Setter Preview Rule *(Boss-dictated 2026-06-11)*

**Take advantage of the entire center zone. Never squeeze an element mimicry into a tiny box.**

When a Style Setter category renders a centre preview (the three-zone layout), the preview card stretches to fill the stage (the `--sky` / `--cns` modifier pattern: `width/height: 100%`, capped ~1100px) and the mimicry inside scales to the card — rings, chips, bars, and samples sized relative to the available space, never fixed thumbnail pixels. A preview the user must squint at fails its one job: showing the edit at a glance.

Canonical violation: 2026-06-11 — the first CNS preview shipped as a fixed 180×140 well floating inside a fixed 560×360 card in a ~1100×600 stage. Eisa: *"Taking advantage of the entire center zone. Not to squeeze any element mimicry in a tiny box."*

### Language-First by Design
- Constellation supports all languages simultaneously, from the ground up by design.
- Per-line bidirectional text (bidiPlugin) is a core architectural feature, not an add-on.
- Every editor view — NotePane, FocusPane, and any future view — must support multilingual, mixed-script content natively.
- Never build a single-language assumption into layout, fonts, cursor behavior, or input handling.

### Constellation Knowledge Hierarchy
Constellation organizes knowledge in a four-level structural hierarchy with an **optional federation layer** at the top — no other PKM system has this depth:

```
Universe (root) — directory; auto-registered as the default "universe_notes"
                  Library (is_universe_notes: true, path == Universe root,
                  Obsidian-style flat). Notes and folders dropped at the
                  Universe root are content of this default library.
│
├── Folder, Note (directly at the Universe root — content of "universe_notes")
│
├── Library (zero or more — additional registered libraries with their own paths)
│    └── Folder
│         └── Note
│
└── cUniverse (zero or more — optional federation links)
     └── Library (libraries from the linked Universe — recursive)
          └── Folder
               └── Note
```

The structural levels of stored knowledge are **Universe → Library → Folder → Note** (four levels). **The Universe root is itself a Library** — when a Universe is created, `ensure_universe_notes_folder` (in `universe.rs`) auto-registers a `universe_notes` library entry whose `path` equals the Universe root, marked `is_universe_notes: true`. This is the Obsidian-style flat default; notes/folders dropped directly at the Universe root are content of this library, not "loose files." A Universe can also have **additional registered libraries** with their own paths (subfolders or external) and **optional cUniverse children** for federation. Verified against `src-tauri/src/universe.rs::resolve_libraries_recursive` (loads `libraries.json` directly, then recurses into `universe.json` children) and `ensure_universe_notes_folder` (auto-creates the root-as-library entry on universe init).

- **Universe**: The top-level container directory. Named by the user. Auto-registers a default `universe_notes` library pointing at itself. Contains its own libraries (own + the auto-`universe_notes` one), settings, bases, bookmarks, and an optional list of cUniverse children. One Universe is "active" per Constellation instance. Stored as a directory with `universe.json` (the federation + meta manifest) and `.constellation/libraries.json` (the libraries manifest).
- **Library**: A complete, self-contained knowledge base (equivalent to an Obsidian vault) — **a direct child of a Universe**. Has its own color, appearance, tags, links, and index. Registered in `libraries.json`. Multiple libraries coexist in one Universe. The default `universe_notes` library has `path == Universe root` (the flat layout). Additional libraries can have any path. Libraries are never copied — Constellation reads them in place.
- **cUniverse (Child Universe)** — *optional layer*: A linked Universe whose libraries get federated into the parent at runtime. Each cUniverse is itself a full Universe (with its own libraries and its own optional cUniverse children); `resolve_libraries_recursive` flattens the federation tree into one library list. Enables viewing notes from multiple independent Universes in one window. **A Universe with zero cUniverses is a complete, valid setup** — federation is opt-in.
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
6. **Maintain `docs/Constellation Orientation & Onboarding vX.Y.md` as the canonical onboarding document.** The filename always carries its version suffix (e.g. `Constellation Orientation & Onboarding v1.0.md`, `... v1.1.md`, `... v1.2.md`). When bumping the version, **write the new version as a NEW file alongside the existing ones — do NOT delete or overwrite previous versions.** Older versions stay in `docs/` as a historical record so the project owner (and future sessions) can diff what changed and when. Each session reads only the highest-version file, but the trail behind it is durable. Never leave a versionless filename in the docs folder. This is the FIRST file every new Claude session reads — it conveys architectural fluency in one read so a fresh AI doesn't rediscover the project from `git log` + screenshots. When any of these change, update the orientation doc in the same commit that lands the change: a migration starts/ships/closes; a top-principal rule is added or reworded; a BUG-NNN opens or closes; a doc-drift item from §12 of the orientation is fixed; a Lessons-Learned entry is added; a boot-perf criterion changes; a version bumps; a subsystem ships a major feature; a help topic ships or restructures. Bump the version (1.0 → 1.1) on structural changes, date-stamp section updates. Keep it readable in one pass — if it grows past ~1500 lines, split into `docs/orientation/` sub-documents. The doc itself enumerates §17 "what Claude has NOT read in detail" — keep that list honest; remove items when verified, add items when new files appear unread.
7. **Minutes of Chating (MoCh) — every ~3 hours of direct chat, write a fresh file at `docs/MoCh/MoCh-YYYY-MM-DD-HHMM.md`.** HHMM is the block's start time in 24-h local time. The MoCh records direct Boss ↔ Claude interaction only — the questions Boss asked, the steers Boss gave, the decisions reached, the outputs delivered. It is *not* the session log (which captures *what shipped* in `lab/reports/SESSION-LOG-YYYY-MM-DD.md`); it is the conversational trace — *what was said*. Skip internal tool plumbing, agent traces, and commit-message recitation. Each file is self-contained and uniquely identified by its filename's date+time. Convention established 2026-05-06.
8. **Cross-check any Pending Job before tackling it.** Before starting work on any `PJ-NNN` entry — drafting an Architect doc, Concept Paper, Plan, or even an exploratory query — verify the entry isn't stale. Cross-check against (a) the **body** of the latest orientation version (especially §3 Architecture and §4.x subsystem sections — NOT just the "What changed in vX.Y" preamble; preambles capture *what changed in that version*, bodies capture *the canonical current state*), AND (b) the relevant session logs (`lab/reports/SESSION-LOG-YYYY-MM-DD.md` for the timeframe the PJ entry mentions). If the cross-check reveals the entry is shipped, obsolete, or scope-drifted, **stop and mark it accordingly in the next Pending Jobs version before proceeding to any other work.** Source: 2026-05-06 — PJ-006 (Living Link P2-P5) was nearly cascaded into despite orientation v1.40 §4.4 being titled "P0–P5 all shipped + user-validated" since 2026-05-05. The first cross-check agent read only preambles (per my own instructions) and missed it; this rule exists so the next session doesn't repeat that miss.

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

## State the Function in Hand (top principal)

At the start of every task — and again at the start of every fresh session, and again any time the work pivots to a different surface — state the **function in hand** in a single line:

> Working on: **the Index panel filter** (the term-browser dropdown in the sidebar Index pane, where MIG-010/MIG-011/MIG-012 added the `via {lemma}` and `≈ similar` badges).

The function-in-hand statement is a one-line anchor naming the feature exactly the way the orientation doc names it. It precedes every other rule. The Predecessor Lookup Rule reads against it ("predecessor of *this surface*"); the Stop-On-Correction Rule fires when the conversation drifts off it; the Testing Instructions Rule frames test tutorials around it.

The point is to make wrong-target drift **impossible at the first line** of any work. If I don't write the function-in-hand statement, I haven't started the task — I'm in undirected coding, which is how features land in the wrong panel.

Re-fire the statement when:

- A fresh session starts.
- The work pivots to a different feature (a "now we're working on Y" moment).
- The Boss corrects the framing (the Stop-On-Correction Rule's exit ramp loops back here).
- A multi-day task resumes after a gap.

Canonical violation: the entire §1D wrong-target incident. I never wrote down "Working on: the Index panel filter" before §1D-A. With that line in place, the moment I drafted "wire to SearchHub" the contradiction with the function-in-hand would have been visible immediately. Without it, drift took 4 commits + 3 builds to catch.

## Predecessor Lookup Rule (top principal)

Before removing, moving, or replacing any user-facing feature, IPC surface, settings entry, or UI wiring, write a **Predecessor → Replacement** entry into the current day's session log, capturing:

- **Where it lives now.** Specific file path, function name, settings path (e.g. `$appSettings.index.semanticSearchEnabled`), and the predecessor MIG number that introduced it.
- **Where its replacement will live.** **Default: the same place.** A different place ONLY with explicit Boss approval logged in the same session.
- **What gets cut and what gets kept.** The in-place call sites that will go away; any consumers that need re-pointing.

This entry comes **BEFORE any code edit**. It's verified against the current orientation doc — not memory — because orientation is the durable record of where features live across MIG history.

The rule fires for: removing a Tauri command, deleting a frontend store wrapper, dropping a Settings UI element, retiring a writable store, replacing an existing search / filter / panel / UI surface, relocating any wiring across components.

It does NOT fire for: bug fixes that don't relocate features, comment edits, dependency bumps, single-file refactors that don't change call shape.

Canonical violation (2026-05-05): §1D-B retired IndexPanel's per-keystroke `searchTermsSemantic` and added a new `concept` category to SearchHub instead of restoring the equivalent in IndexPanel. Four explicit pointers — Settings flag named `index.semanticSearchEnabled`, IndexPanel was the actual call site, MIGs 010 / 011 / 012 all operated on IndexPanel, the Settings progress strip lived under Settings → Index — all said "Index panel"; I read past every one and shipped the wrong-target replacement. The rule above would have surfaced "Predecessor: IndexPanel filter, third layer; Replacement: same place" as the first written line, and the SearchHub option would have required Boss approval that was never sought.

## Stop-On-Correction Rule (top principal)

When the Boss says "wrong target", "you're confused", "no", "unacceptable", "we're working on X" (when X corrects my framing), or any equivalent course-correction phrasing, I **stop all in-flight code edits**, summarize what's changed since the last explicit Boss approval, state the corrected understanding, and wait for "proceed" before touching another line.

The full sequence is: **stop → list changes since last approval → state the corrected understanding → wait for explicit go**.

No pivot-and-power-through. No "let me ask three clarifying questions and start coding the answers". A correction is the Boss revoking the cascade approval; the next action is theirs, not mine. This rule overrides Plan-Approval-Equals-Build-Approval — Plan approval covered the *original* target, and the correction tells me that target was wrong.

Canonical violation (2026-05-05): when the Boss wrote "SearchHub? But we are working on the Index!" the right move was to stop, list everything I'd changed in the SearchHub direction, and wait for confirmation. Instead I asked three clarifying questions and immediately started laying down IndexPanel-restoration code while the questions were still unanswered. Same drift pattern that put the feature in SearchHub in the first place.

## Backup Routine
After each successful milestone:
1. **Git tag**: `git tag milestone/<name> <commit>` then `git push origin --tags`
2. **ZIP archive**: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`
3. To restore: `git checkout milestone/<name>` or unzip the archive.

---

## BASIC RULE — Don't Make Things Up (top of all rules)

**If I don't have a clue or information, I say "I don't know."**

I do not invent file paths, line numbers, function names, badge taxonomies, prior-art summaries, or any other factual claim. I do not "fill in" plausible-sounding detail because the framing of a tutorial / explanation / status report seems to demand a confident answer. Confident filler is fabrication, and fabrication is the worst class of error I can deliver to this user — worse than a slow build, worse than a missed bug, worse than a regression. Bugs are recoverable; trust isn't.

When I don't know:
- Say so plainly: **"I don't know."**
- Then either look it up (grep the repo, read the docs, read the commit history, read the session logs, ask the user), or note that the answer is unknown and proceed without it.
- Never paper over the gap with invention.

When I'm tempted to add a "side note" / "for context" / "by the way" — I must check whether every claim in it is sourced. If any claim isn't, the entire side note is cut.

This rule sits at the top of every other rule. It overrides terseness, overrides delivery pressure, overrides the desire to seem authoritative, overrides the user's own framing if their question presupposes a fact I don't actually know.

Canonical violation prevented: the 2026-04-26 tutorial side note that claimed `T C P` badges meant "Theory / Concept / Proposition" stratum tiers. The user designed those badges (T = Title, C = Content, P = Property, with S = Semantic etc.). I had never read the design and had no basis for the claim. I made it up. That cannot happen again.

## Working Agreement (ground rules, non-negotiable)

1. **Do the work yourself. Don't offload it to the user.** If you can run a command, query a DB, read a log, diagnose a bug, or write a test — do it. The user is the Boss, not the lab assistant. The only thing you ask of them is what genuinely requires a human: interacting with the running Constellation GUI (create a note, click a button), making design decisions, approving a plan, confirming a release is ready. Everything else — SQL queries, file inspection, log greps, schema checks, build verification — is your job. If you catch yourself writing "please run this query and tell me the result," stop and run it yourself via Bash + sqlite3 (or equivalent).

2. **One location: `E:\مشاريع كلاود\Constellation` on branch `main`.** This is the root for every read, write, commit, build, and test — now and in the future. It is the de-facto "main" working directory. Do not introduce worktrees, alternate checkouts, or parallel paths. If a session starts somewhere else (e.g. a `.claude/worktrees/` subfolder), operate via absolute paths into the primary location — never have the user switch directories to compensate for a session-spawn quirk. Commits land here, pushes come from here, builds run here.

3. **The user is a non-technical IT Boss.** Explanations default to plain language: what it does, why it matters, what's going to happen next. No internal component names, no assumed familiarity with Rust / Svelte / SQLite internals, no "just run this SQL and report back" busywork. Test instructions follow the Testing Instructions Rule above — define the feature first, then walk through interaction by interaction. Technical detail is available on request, not pushed by default.

4. **Don't ship changes you haven't proven safe against the whole architecture.** You are the consultant/engineer/SME, not a feature-pleaser. Before every code change — every function, subfunction, routine, every line you add or alter — answer in writing: *what wires does this touch, and what will it cut?* If the change crosses a subsystem boundary or interacts with a write path / lifecycle / reactivity / IPC contract, you MUST run a full architectural-impact review before submitting it: spawn as many parallel agents as needed (Explore for read paths, Plan for design verification, code-reviewer for cross-cutting risk) to map the call graph, list every consumer that will see the new behavior, and identify every invariant that could break. The user's job is to direct intent and approve plans; bug-creation by oversight is on you, not on them. If the review reveals risk you can't characterize, stop and surface it — never ship "let's see what happens" code. This rule overrides delivery pressure: a slower, proven change is always preferred to a fast change that introduces a regression. (The MIG-006 §3-expanded → BUG-015 incident, where a value-prop → doc sync `$effect` raced with `{#key}` onDestroy and corrupted target body content, is the canonical violation this rule prevents.)

5. **Cross-check every non-trivial fix or design against proven methods before applying it.** Before locking in a fix or implementing any feature that touches subsystem boundaries, scope architecture, or chooses a write-path/read-path strategy, run parallel `WebSearch` queries against how mature systems and communities solve the same problem: Lucene/Elasticsearch, SQLite/Postgres, vector DB practice (Pinecone/Faiss/pgvector), library science (LCSH/MeSH thesaurus systems), academic IR/CLIR literature, PKM tools (Obsidian/Logseq/Roam). Compare the dominant industry pattern against your proposal honestly. Surface both options to the Boss with the tradeoffs that matter — name the standard pattern the field uses, name your proposal, and explain which wins for Constellation's constraints. **Don't ship an inventive solution when a battle-tested pattern exists.** When the Boss asks "cross-check this against other proven methods" before you've done it yet, that's not a request — it's a correction. Do the research first; ask second. (Canonical violation: 2026-05-05 §1D backfill — three SMEs and the architect doc proposed pre-computing `term_vocab.bridge_concept_id` for every user term; I implemented it without checking that Lucene's SynonymGraphFilter, SQLite FTS5 Method 2, CLIR query-translation, and Primo's controlled-vocabulary expansion all favor *query-time* concept expansion. Cost: two mid-test fixes plus a full §1D redesign before Boss could even reach Stage 2.) When NOT applicable: trivial single-file bug fixes, local refactors, single-component UI tweaks where there's no broader pattern in play.

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
