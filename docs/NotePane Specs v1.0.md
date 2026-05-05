# NotePane Specs

**Version 1.0 | 2026-05-05**

> **What this is.** A durable, versioned **specification** of the current NotePane — its purpose, architecture, hard invariants, proven decisions, and forbidden anti-patterns. Distilled from 121 commits of development history (`NotePane.svelte`'s full git log), the existing `docs/NotePane-spec.md` (the original eNotePane-era spec, retained as historical), `docs/eNotePane-development-record.md`, and the orientation versions that landed each phase.
>
> **What this is NOT.** It is not a development history (chronological). It is not a tutorial. It is not the implementation; the implementation lives in `src/lib/components/NotePane.svelte` (~1,420 lines). This doc captures what is **proven and final** so future work doesn't undo or duplicate what's settled.
>
> **Audience.** Primary: every future Claude session before any NotePane edit. Secondary: the Boss reviewing the editor's commitments. Tertiary: any future contributor.
>
> **Update cadence.** Same versioning convention as the orientation, the Constellation Development Laws, and the Pending Jobs doc — a new version (`v1.1`, `v1.2`, …) is written as a NEW file alongside the previous whenever a *proven* design decision changes. Older versions stay as historical record. Wording-only refinements within a version are fine.
>
> **How to use this doc.** Before any NotePane edit: read §3 (Architecture), §4 (Hard invariants), §7 (Forbidden anti-patterns). If your change touches a hard invariant or proposes one of the anti-patterns, surface it to the Boss before any code edit.

---

## §1 · Lineage

The NotePane the user sees today is the result of a deliberate three-stage evolution:

| Stage | Name | Commit / artifact | Status |
|---|---|---|---|
| Prototype | **cNotePane** (the "c" stands for Constellation) | Pre-tracked in early Constellation experiments. The code that became `eNotePane` was Boss's third or later iteration on this surface. | Origin of the design; not in current git. |
| Experimental | **eNotePane** (the "e" stands for experimental) | Git-tracked from `36ba7b7` ("eNotePane Phase 0: The Skeleton — APPROVED by all 5 auditors") through `fb1e954` ("Promote eNotePane → NotePane"). | Promoted to production. |
| Production | **NotePane** (current) | Promoted at `fb1e954` (2026-03-29). The promotion overwrote the previous NotePane (which is preserved as `archive/NotePane-legacy.svelte`). 121 commits to date have evolved this surface. | Live and final. |

The fb1e954 commit message says verbatim: "The experiment succeeded. The 'e' prefix is removed." The cNotePane → eNotePane → NotePane lineage is the proof that each phase delivered the foundation the next phase needed.

**Implication.** The current NotePane is the *third* major rewrite of this surface. Reverting any portion to a TipTap-era, AtmospherePane-era, or eNotePane-era pattern is forbidden unless Boss explicitly directs.

---

## §2 · Purpose & Scope

### 2.1 What NotePane is

NotePane is **Constellation's core note editor** — the surface where a markdown note is read, written, structured, formatted, linked, navigated, and saved. It is the desk where the captured idea becomes a complete, polished, connected document.

The product framing (carried forward from `NotePane-spec.md` §0):

> A note is the most democratic form of knowledge. It doesn't require education to start. The note is the product, not the app. The user should feel like they're writing on paper, not operating software.

### 2.2 What NotePane is NOT

| Not | Why |
|---|---|
| **Not FocusPane** | FocusPane is plain-text-only capture (no markdown parser, no syntax highlighting, no decorations). It is a deliberate sister surface, not a sub-mode of NotePane. The two share the editor parity contract (§4.4) for keystroke fundamentals but diverge on every rendered surface. |
| **Not a document editor** | NotePane edits the source `.md` file. There is no separate "rendered view" — live preview is a CodeMirror decoration layer over the source, not a parallel render. |
| **Not a TipTap WYSIWYG** | TipTap was tried (`8f0574a` 2026-03-17, "Add Document editor mode (TipTap WYSIWYG) as alternative") and **fully removed** in `d8b7767` (2026-03-22, "Migrate from TipTap to CM6"). Reverting to a WYSIWYG editor is forbidden. |
| **Not the Second Screen note view** | The Second Screen mounts the same NotePane component in a separate window per the "displays not domains" rule (Law 4.3). It is NOT a re-implementation. |
| **Not a Notion / OneNote / Bear clone** | Constellation's notes are `.md` files on disk. The editor is bound to that constraint (Law 1.3 — File Over App). |

### 2.3 Boundaries

NotePane delegates outward when:

- **The Properties panel** is needed → `PropertyEditor.svelte` (with `onstagechange` callback, NOT `$effect`).
- **The Editor extension stack** is needed → `$lib/editor/*` modules (livePreview, callout, lineDeco, bidi, markdownHighlight, completions, shortcodeAutocomplete).
- **A wikilink completion** is needed → `createWikilinkCompletion` from `$lib/editor/completions.ts`.
- **An image embed** is resolved → Rust-backed `embeds.rs` (`Universal Embed Resolver`).
- **The active-editor registry** is updated → `$lib/editor/activeEditor.ts` `registerActiveEditor` / `unregisterActiveEditor`.
- **A table operation** is performed → `$lib/editor/tableUtils.ts` + `$lib/editor/tableFormulas.ts`.

NotePane **does not** own any of these surfaces; it consumes them.

---

## §3 · Architecture

### 3.1 Layout — PaperOnDesk

The visual structure is fixed and proven (`fe6cb9a`, `e55c25c`, `2603cf7`, `ea33b45`):

```
┌─ Gray Desk ───────────────────────────────────────────┐
│                                                       │
│   ┌─ Breadcrumb (16px → 48px aligned with paper) ──┐  │
│   │  ‹ Back  Note Title  Stage▾  ⋮      Trail ⤺⤻ │  │
│   └────────────────────────────────────────────────┘  │
│                                                       │
│   ┌─ White Paper (1200px max-width, 48px padding) ─┐  │
│   │                                                │  │
│   │  Title (editable inline)                       │  │
│   │  Properties (collapsed by default)             │  │
│   │  Toolbar (floating, RTL-aware, toggleable)     │  │
│   │  ─────────────────────────────────────────     │  │
│   │  CM6 Editor (markdown source +                 │  │
│   │             livePreview decorations)           │  │
│   │                                                │  │
│   └────────────────────────────────────────────────┘  │
│                                                       │
└───────────────────────────────────────────────────────┘
```

**Frozen dimensions.** 1200px paper width. 48px paper padding. Breadcrumb padding aligned to paper padding (commit `9bdb65c`). Margin between breadcrumb and paper preserved. Cursor is a thin vertical line without serifs (`2603cf7`).

### 3.2 Editor — CodeMirror 6

The editor is **CodeMirror 6**. This is final.

History: TipTap was added briefly (March 2026), full migration to CM6 landed at `d8b7767` (2026-03-22). Obsidian-feature-parity rebuild on CM6 at `91aef5d` (2026-03-09). All editor work since flows through the CM6 extension model.

Imports (the proven set, do not deviate):

| Import | From | Purpose |
|---|---|---|
| `EditorView, keymap, drawSelection, Decoration, DecorationSet` | `@codemirror/view` | Core view layer |
| `EditorState, Compartment, Prec, StateField, StateEffect, RangeSetBuilder` | `@codemirror/state` | State machinery — `Compartment` for hot-swappable config, `Prec.highest` for keymap precedence (e.g. callout Enter exit, `1026905`), `RangeSetBuilder` for sorted decorations |
| `markdown, markdownLanguage` | `@codemirror/lang-markdown` | Language support |
| `syntaxHighlighting, HighlightStyle` | `@codemirror/language` | Markdown syntax colors (`markdownHighlightStyle`) |
| `tags` | `@lezer/highlight` | Tag references for HighlightStyle |
| `defaultKeymap, history, historyKeymap, undo, redo, indentWithTab` | `@codemirror/commands` | Edit + history commands |
| `autocompletion, closeBrackets, closeBracketsKeymap` | `@codemirror/autocomplete` | Completions + Obsidian-style smart bracket pairing (`b00bf5f`) |
| `search, openSearchPanel, searchKeymap, SearchQuery, setSearchQuery, findNext` | `@codemirror/search` | Search panel |

### 3.3 Shared editor extensions — `$lib/editor/`

The CM6 extension stack lives in `$lib/editor/` and is **shared with FocusPane** (where applicable) per the Editor Parity Rule. NotePane imports:

| Module | Purpose |
|---|---|
| `livePreview.ts` (`livePreviewPlugin`, `livePreviewTheme`, `libraryPathField`/`setLibraryPath`, `notePathField`/`setNotePath`, `attachmentFolderField`/`setAttachmentFolder`, `linkTraversalMapField`/`setLinkTraversalMap`) | The live preview layer — wikilink rendering, image embeds, link traversal chips, attachment-folder resolution. |
| `calloutPlugin.ts` (`calloutPlugin`, `calloutTheme`, `calloutCollapseField`, `toggleCallout`) | Obsidian-style foldable callouts with Enter exit (Prec.highest). |
| `lineDecoPlugin.ts` (`lineDecoPlugin`, `lineDecoTheme`) | Per-line decorations. |
| `bidiPlugin.ts` (`bidiPlugin`, `bidiTheme`, `scriptFontsField`/`setScriptFonts`) | **Per-line bidirectional text** + per-script font selection. Foundational to Law 2.5 (Language-First by Design). |
| `activeEditor.ts` (`registerActiveEditor`, `unregisterActiveEditor`) | Process-wide registry of the focused CM6 view. Used by template insertion, emoji picker, slash command handlers. |
| `markdownHighlight.ts` (`Highlight as HighlightExt`) | Inline highlight syntax. |
| `completions.ts` (`createWikilinkCompletion`, `createTagCompletion`, `createSlashCompletion`, `createTypedLinkCompletion`) | Per-keystroke completion sources. |
| `shortcodeAutocomplete.ts` (`shortcodeCompletion`) | Emoji + Icon `:shortcode:` autocomplete (`6d36c14`). |
| `tableUtils.ts` (`parseTable`, `formatTable`, `addRow`, `addColumn`, `deleteRow`, `deleteColumn`, `setAlignment`, `moveRow`, `moveColumn`, `sortByColumn`, `ParsedTable`) | Table operations driven by the floating TableToolbar. |
| `tableFormulas.ts` (`evaluateTableFormulas`, `indexToCol`) | Table formula support. |

### 3.4 Save model

- **Save trigger**: debounced 1500ms after the last keystroke. Debounce constant: `1500` (line ~399). Increased from earlier values at `e1f3c02` to prevent save-during-typing interference.
- **Idle save**: every 30,000ms (`IDLE_SAVE_INTERVAL`) when the editor is idle.
- **No save on every keystroke.** Ever.
- **No reactive recalculation during typing.** Parsed frontmatter is cached in `$state`, recomputed via `reparse()` only on tab switch or edit/reading mode transition (`ea970bc` — "Eliminate ALL reactive recalculation during typing"). Properties and noteBody derived from the cached parsed object, NOT from `tab.content`.
- **Saving indicator**: removed at `74b863e` ("save silently in background"). No visible "Saving..." text.

### 3.5 Stage + breadcrumb

- Stage breadcrumb is a dropdown menu (`6cbe87c`, "Replace promote button with stage dropdown menu"). Stage values lowercased (`da0d03d`, `5e45315`).
- **Stage sync**: `Properties → breadcrumb` is via the **`onstagechange` callback** from PropertyEditor, NOT a `$effect` watching `properties`. The earlier `$effect` pattern (`b46b51a`, `63ca542`) was REMOVED at `3441358` because it disrupted CM6 cursor display.
- Stage redesign at `90c1ea8` (§136, 2026-05-02) consolidated the design.
- Breadcrumb shows: navigate back/forward, title, stage dropdown, three-dot menu, optional trail navigation chevrons.

### 3.6 Toolbar

- Floating toolbar visible above the editor surface (`d8023ab`).
- Toggle button on the toolbar itself (`3ff7f3f`), state in `$appSettings.showFloatingToolbar`.
- **RTL-aware**: alignment buttons are direction-aware (`225e99f`); undo/redo + alignment icons mirrored via `scaleX(-1)` (`5a6504c`); `dir={toolbarDir}` on the toolbar element (`31d34c0`).
- Buttons (proven set): Bold, Italic, Underline, Sub/Superscript, Alignment (Left/Center/Right), Clear formatting, Find, RTL toggle (`bf0e13e`).

### 3.7 Properties panel

- `PropertyEditor.svelte` component, embedded in the paper.
- **Collapsed by default** (`83fa39f`, R9.2 fix).
- Stage editing: dropdown with the Zettelkasten 4 stages (currently — pending PJ-007 decision).
- Communicates to NotePane via `onstagechange` callback (no `$effect` echo).

### 3.8 Wikilink navigation

- **Single click**: pointer cursor + single-click navigation (`4e24f68`).
- **Ctrl+Click**: opens a new tab. If the target doesn't exist, creates the note (`d627d7e`).
- **Mousedown timing**: handled before CM6 strips decorations (`08b2f5b`).
- **Traversal chip**: `[[link]]` rendered with a small traversal-count chip in rendered prose (`e148446`, reapplied after a brief revert at `5bca489`).
- **Cross-note corruption fix** (`a2052da`): wikilink-click navigation no longer corrupts content across notes.

### 3.9 Search highlighting

- When a note is opened from Index, the matched term is highlighted (`2bbe7aa`).
- Whole-word search instead of regex (`ba214bf`).
- First match scrolled to and centered (`0364f16`).
- Arabic term highlighting respects reverse-normalization for search (`fc08661`, `fd53199`).
- Multi-color highlight decorations via `colorHighlightField` + `setColorHighlights` effect (current code lines 54-71). Six categories: title, content, tag, property, wikilink, semantic. Driven by SearchHub matches (`1dc859d`).

### 3.10 Per-script fonts

- `scriptFontsField` + `setScriptFonts` from `bidiPlugin.ts`.
- Per-script font selection via `getEffectiveScriptFonts` from `$lib/libraries/store`.
- Typewriter font theme bundled with per-script authentic fonts (`068b217`).
- Editor inherits configured font settings (`600d9bf`).

### 3.11 Universal Embed

- Image embeds resolved via Rust-backed `embeds.rs` (`21b64d8`).
- Per-path try/catch + attachment-folder fallback + multiple candidate paths (`e781dfb`).
- Obsidian-parity embed rendering (`e050cb5`).
- 404 flood from relative-path embeds on first render: fixed (`455bdd7`).
- Diagnostic info on missing embeds + auto-refresh index (`45f4a4e`).

### 3.12 Emoji + Icon shortcodes

- Insert as `:shortcodes:`, render as widgets (`8c63ea0`).
- Inline `:shortcode:` autocomplete (`6d36c14`).
- Picker fixes: empty Icons tab + insert-at-cursor (`e7bf160`).

---

## §4 · Hard Invariants — These Must NEVER Be Undone

These are the non-negotiable design commitments earned through real bugs, real fixes, and real performance wins. Each is sourced to a commit. Reintroducing the bad pattern is a regression.

### 4.1 No `$effect` echo loops

CLAUDE.md Rule 2. NotePane and CodeMirrorEditor must NOT contain a `$effect` that reads and writes the same reactive variable, or watches a prop it modifies via callback. Cursor-jump bugs traced to this pattern were fixed at `77852d9`, `6595825`. Any new `$effect` in NotePane must declare its dependencies explicitly and wrap writes in `untrack()`.

### 4.2 No value-prop → CM6 doc sync `$effect` (BUG-015)

The `$effect` that synced `value` (prop) into the CM6 doc raced with `{#key tab.id+'|'+tab.path}` onDestroy during cascade. Result: target file body overwritten with source body, real corruption observed via `bug015.log`. Fully removed at `5afe0c2` (§116). The "rename target while source is visible" UX gap is the documented cost; user data integrity wins. **Reintroducing any value→doc sync $effect in NotePane is forbidden** without a Migration Rule audit and Boss approval.

### 4.3 Save debounce ≥1500ms

`e1f3c02`. Earlier values caused save-during-typing interference. Lower values are forbidden.

### 4.4 Editor parity with FocusPane via `$lib/editor/`

Law 2.5 + CLAUDE.md Editor Parity Rule. Every CM6 extension that lives in `$lib/editor/` (livePreview, callout, lineDeco, bidi, markdownHighlight, completions, shortcodeAutocomplete) is shared between NotePane and any future note view. **FocusPane is the deliberate exception** — plain text only, no markdown parser, no decorations. Forking or duplicating the extension set across panes is forbidden.

### 4.5 No reactive recalculation during typing

`ea970bc` (2026-03-25, "Eliminate ALL reactive recalculation during typing"). The keystroke path is: keystroke → CM6 internal → `onchange` → `latestEditorText` (non-reactive) → debounced save (1500ms) → store update → **NO cascading recalculations**. Any new reactive read on `tab.content`, `properties`, or any derived chain that touches the keystroke path must be eliminated.

### 4.6 Static `noteDir` on tab load

`23c35b1`. `noteDir` is `$state` set on tab load, NOT `$derived` from `tab.content`. Direction recalculation on every save cascades into editor reconfiguration during typing.

### 4.7 Cached parsed frontmatter

`ea970bc`. Parsed frontmatter is cached in `$state` and only recomputed via `reparse()` on tab switch or edit/reading mode transition. Properties and noteBody are derived from the cached parsed object, not from `tab.content`. Re-parsing on save is forbidden.

### 4.8 Stage sync via callback, not `$effect`

`3441358`. The `$effect` that watched `properties` for stage changes triggered micro re-renders on every editor update, disrupting CM6 cursor display. Removed. Stage sync now flows via the `onstagechange` callback from PropertyEditor.

### 4.9 No provenance computation in sidebar cycle

`27ce210`. Provenance computation in the sidebar cycle caused typing lag. Removed and replaced with a deferred path. Don't reintroduce.

### 4.10 Pre-cache module-level `Decoration` objects

CLAUDE.md Performance Rule 1. Never create `Decoration.mark()`, `Decoration.replace()`, or `Decoration.widget()` inside a decoration builder function. Allocate once at module load. NotePane's `colorHighlightField` follows this pattern (lines 54-71).

### 4.11 `RangeSetBuilder` for sorted decorations

CLAUDE.md Performance Rule 3. Large decoration sets must use `RangeSetBuilder` (sorted insert), never `Decoration.set()` with unsorted arrays. NotePane's `colorHighlightField` uses `RangeSetBuilder` and pre-sorts ranges (line 61-64).

### 4.12 Process-wide active-editor registry

`registerActiveEditor` on focusin, `unregisterActiveEditor` on destroy. Template insertion, emoji picker insert-at-cursor, and slash commands depend on this registry. NotePane registers on mount and unregisters on destroy (lines ~697-720).

### 4.13 `onDestroy` cleanup contract

CLAUDE.md Performance Rule 4. NotePane's `onDestroy` must:
- `clearTimeout(debouncedSaveTimer)` and `clearTimeout(_idleSaveTimer)`.
- `unregisterActiveEditor(view)`.
- `view?.destroy()`.

Any new `setTimeout` / `setInterval` / `addEventListener` / Tauri `listen()` added inside the component must be paired with cleanup in this `onDestroy`.

### 4.14 BIDI plugin is core, not optional

Law 2.5 + the bidiPlugin's foundational role in per-line direction handling. Disabling, replacing, or short-circuiting the bidiPlugin is forbidden.

---

## §5 · CM6 Extension Stack (Proven, In-Order)

The order matters for layering precedence. The proven stack (paraphrased from `NotePane.svelte` `EditorView` construction):

1. **History** + **historyKeymap** — undo/redo first so subsequent extensions can layer over.
2. **Search panel** + **searchKeymap**.
3. **Autocompletion** with sources: `createWikilinkCompletion`, `createTagCompletion`, `createSlashCompletion`, `createTypedLinkCompletion`, `shortcodeCompletion`.
4. **closeBrackets** + **closeBracketsKeymap** (Obsidian-style smart bracket pairing).
5. **markdown(markdownLanguage)** language support.
6. **syntaxHighlighting(markdownHighlightStyle)** — markdown syntax colors.
7. **livePreviewPlugin** + **livePreviewTheme** + the four state fields (`libraryPathField`, `notePathField`, `attachmentFolderField`, `linkTraversalMapField`).
8. **calloutPlugin** + **calloutTheme** + **calloutCollapseField**.
9. **lineDecoPlugin** + **lineDecoTheme**.
10. **bidiPlugin** + **bidiTheme** + **scriptFontsField**.
11. **HighlightExt** (inline highlight syntax).
12. **colorHighlightField** (multi-color search highlights).
13. **drawSelection**.
14. **Compartments** for hot-swappable config (e.g. font, dir).
15. **Prec.highest** for keymap precedence overrides (e.g. callout Enter exit, `1026905`).
16. **defaultKeymap** + **indentWithTab** last (lowest precedence).

Replacing or reordering any of these is a Migration Rule change.

---

## §6 · Cross-Cutting Integrations

### 6.1 With FocusPane (sister surface)

- Shared paper dimensions: 1200px width, 48px padding (`ea33b45`, "NotePane + FocusPane: unified paper dimensions").
- Shared editor extension imports for the keystroke fundamentals.
- Distinct: FocusPane has NO markdown parser, NO syntax highlighting, NO decorations. Plain CM6 + history + line wrapping only.

### 6.2 With CodeMirrorEditor

The earlier separate `CodeMirrorEditor.svelte` and NotePane's CM6 view share the cursor-jump fix lineage (`6595825`) — both removed `$effect` echo loops in the same commit.

### 6.3 With Properties (PropertyEditor)

- Two-way sync via callback only (`onpropschange`, `onstagechange`).
- Properties panel is collapsed by default.
- Stage normalized to lowercase.

### 6.4 With Sky View / Index / Search Hub

- Search Hub opens NotePane with a highlight term (`2bbe7aa`); NotePane scrolls to first match and centers.
- Index click → NotePane opens with the matched stem highlighted (`0364f16`, `fc08661`).
- Sky View edge click navigates to a NotePane tab; multi-color edge-source highlights (`1dc859d`).

### 6.5 With Universal Embed

- Image embeds resolved via Rust IPC, not frontend filesystem walks.
- Diagnostic info on missing embeds visible in the editor (`45f4a4e`).

### 6.6 With templates + slash commands

- Slash command registry exposes templates; insertion happens via `getActiveEditor()` from the active-editor registry.
- Template engine is async (`5bb410c`, "Implement Templater-like template system with async engine").

### 6.7 With the file watcher

- File-system changes from external editors trigger a reload via the watcher pipeline. NotePane reloads via the tab-key invalidation pattern (`{#key}` bump), NOT via a `$effect` on `value` / `editBody`. (BUG-015 lesson, §4.2.)

### 6.8 With Second Screen

- The Second Screen mounts NotePane verbatim per Law 4.3 ("Additional screens are displays, not domains"). Save / load / edit operations are owned by the NotePane core, not duplicated in the second-screen wrapper.

---

## §7 · Forbidden Anti-Patterns

These patterns appeared in the history and were **proven harmful**. Every entry is sourced to a commit that either removed the pattern or fixed the bug it caused. Reintroducing them is forbidden without an explicit Boss-approved Migration Rule deviation.

| Anti-pattern | First observed | Fix commit | Why forbidden |
|---|---|---|---|
| Value-prop → CM6 doc sync `$effect` | `3c4732d` (§115) | Reverted at `5afe0c2` (§116) | BUG-015 — raced with `{#key}` onDestroy during cascade, corrupted target file with source body. |
| `$effect` echo loop syncing `value`↔editor | Pre-2026-03-25 | `77852d9`, `6595825` | Cursor jump on every save. |
| `$effect` watching `properties` for stage changes | `b46b51a`, `63ca542` | `3441358` | Triggered micro re-renders on every editor update; CM6 cursor invisible. |
| `noteDir` as `$derived` from `tab.content` | Pre-`23c35b1` | `23c35b1` | Direction recalculation on every save cascaded into editor reconfiguration. |
| Reactive recalculation during typing (parsed FM, properties chain) | Pre-`ea970bc` | `ea970bc` | Typing lag. Parsed FM must be cached in `$state` and recomputed only on tab switch / mode transition. |
| Provenance computation in sidebar cycle | Pre-`27ce210` | `27ce210` | Typing lag. |
| TipTap as the editor | `8f0574a` (added) | `d8b7767` (removed) | WYSIWYG diverges from `.md`-on-disk source-of-truth. Heavyweight. CM6 wins on every axis. |
| Visible "Saving..." indicator | Pre-`74b863e` | `74b863e` | Distraction. Save silently. |
| `confirm()` for destructive actions | Pre-MIG-012 §Build.8-fix | `8d98a3a` (general pattern) | OS-locale labels bypass i18n. Use `ConfirmDialog.svelte` with localized strings. |
| Save on every keystroke | n/a | `e1f3c02` | Filesystem thrashing, conflict with watcher events. Debounce ≥1500ms. |
| Dispatch on `dir` change without prevDir guard | Pre-`ea970bc` | `ea970bc` (FocusPane fix; pattern applies) | Spurious dispatches on every tab switch. |
| `Decoration.mark/replace/widget` inside builder fn | n/a | CLAUDE.md Rule 1 | Allocations on every rebuild. Pre-cache at module load. |
| `Decoration.set([...])` with unsorted ranges | n/a | CLAUDE.md Rule 3 | Sort cost on every rebuild. Use `RangeSetBuilder`. |
| ImageEmbed first-render 404 flood | Pre-`455bdd7` | `455bdd7` | Console noise + WebView2 crash risk. |

---

## §8 · Lifecycle

### 8.1 Mount

```
onMount → build EditorState (stack §5 + initial doc) →
EditorView created → applied to mount point →
registerActiveEditor(view) →
view.dispatch initial selection (initialCursorPos) →
view scrollTo (initialScrollTop) →
focus if appropriate
```

### 8.2 Tab switch

Tab switches re-render the component via `{#key tab.id + '|' + tab.path}` in the parent layout. **The component is destroyed and re-created**, NOT updated in place.

This is the critical safeguard against value-prop → doc sync races (BUG-015). Tab navigation cannot reuse a stale CM6 instance.

### 8.3 Save

```
keystroke → CM6 internal update → onchange callback →
latestEditorText (non-reactive) → debounced save timer →
1500ms idle → doSave() → onsave callback to parent →
parent persists to disk via Tauri IPC
```

### 8.4 Destroy

```
onDestroy →
  clearTimeout(debouncedSaveTimer) →
  clearTimeout(_idleSaveTimer) →
  unregisterActiveEditor(view) →
  view?.destroy()
```

Any future feature that holds a resource (timer, listener, IPC subscription) inside NotePane MUST add cleanup to `onDestroy`.

---

## §9 · Performance Commitments

| Metric | Commitment | Source |
|---|---|---|
| Keystroke → screen update | Zero perceptible lag | CLAUDE.md Rule 1 |
| Save debounce | ≥1500ms | `e1f3c02` |
| Idle save | 30,000ms | `IDLE_SAVE_INTERVAL` |
| Decoration rebuild cadence | Only on `docChanged`, `viewportChanged`, or `selectionSet` with line-change guard | CLAUDE.md Rule 1 |
| Reactive recalculation during typing | Zero | `ea970bc` |
| Parsed frontmatter recompute frequency | Tab switch + edit/reading mode transition only | `ea970bc` |
| `tokenize_to_vec` body cap | 1 MiB | `BODY_CAP_BYTES` (separate constant; relevant when NotePane edits trigger re-tokenization) |

A 5,000-word note must scroll without stutter; typing 10 characters rapidly must produce zero lag (CLAUDE.md Performance Rule 7).

---

## §10 · Internationalization & Direction

- All user-facing strings go through `$t()`.
- Directions: `dir={dir}` propagates to the desk; `dir={toolbarDir}` to the toolbar.
- Per-line bidirectional text via `bidiPlugin` (Law 2.5).
- RTL-aware alignment (`225e99f`), mirrored undo/redo + alignment icons (`5a6504c`).
- Per-language date format (`17da523`), per-script numeral style (`ba46aea`).
- Editable note title supports any script (`5981ddd`).
- 15 launch locales — every visible string must have a key.

---

## §11 · Versioning Rules for This Spec

1. **Adding a section.** Append to the appropriate `§N` with a sub-heading. Bump the version. New file (`v1.1.md`) alongside the previous.
2. **Adding an invariant or anti-pattern.** Append to §4 / §7 with the source commit. Bump the version.
3. **Refining wording.** Same version is fine.
4. **Retiring an invariant.** Forbidden unless the underlying decision was reversed by an explicit Migration Rule MIG with Boss approval. The retired invariant moves to a "Retired" subsection at the bottom of §4 with the retiring commit cited.
5. **Filename convention.** New version = new file alongside the previous. Same as orientation, Laws, Pending Jobs.

---

## §12 · Cross-References

- `src/lib/components/NotePane.svelte` — the implementation.
- `src/lib/components/FocusPane.svelte` — the sister surface, plain-text capture.
- `src/lib/components/PropertyEditor.svelte` — properties panel + stage dropdown.
- `src/lib/editor/*` — the shared CM6 extension stack.
- `archive/NotePane-legacy.svelte` — the pre-`fb1e954` NotePane, archived.
- `docs/NotePane-spec.md` — the original eNotePane-era spec, retained as historical (still uses "eNotePane" branding throughout).
- `docs/eNotePane-development-record.md` — the full eNotePane experiment record (Phase 0 through promotion).
- `CLAUDE.md` → Performance Rules 1–8, Editor Parity Rule, Architecture Principles → Language-First by Design.
- `docs/Constellation Development Laws v1.3.md` (current) — Laws 2.1, 2.5, 2.6, and the canonical-violation timeline.
- `docs/Constellation Pending Jobs v1.0.md` — open jobs (PJ-007 note-stage taxonomy is the most NotePane-adjacent open decision).

---

**End of v1.0.** This spec captures the current proven state of NotePane — twelve cross-cutting integrations, fourteen hard invariants, twelve forbidden anti-patterns, all sourced to commits in the 121-commit history. Any future NotePane work reads this doc first; if a proposed change touches §4 or §7, it stops and surfaces to Boss before any code edit.
