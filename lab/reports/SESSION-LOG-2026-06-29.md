# Session Log — 2026-06-29

## Context
Session start after PJ-065 (structural / parent-TOC link) shipped + closed (commits `f10e8bd0` → `8791a675`). Build phase continues. Boss chose the **right-click context-menu** work as the next build (over PJ-067 and the MIG-084 Style-Setter sub-item). Ultracode on.

---

## SO #8 SCOPE CORRECTION — the right-click work is **MIG-077 (resume)**, NOT "MIG-080"

The session-start handover (`Handover-2026-06-29-PJ-065-shipped.md`) and the ready-to-paste prompt labelled the right-click work **"MIG-080 (right-click) — banked, NOT built."** The SO #8 cross-check (orientation v3.16 BODY + Plan-on-disk + Debt Register B) shows this is a **scope conflation**. Corrected facts:

- **MIG-080** = the *right-sidebar → note-context cascade* (§A Calendar→left dock · §B Tags note-scoped · §C Tasks · §D Source Review/Cataloger · §E Knowledge Health · §F Review Pulse split · §G audit). **SHIPPED** (orientation v2.90 → v3.02). It has **nothing to do with right-click menus** beyond having *banked the right-click reference* during it (v2.96).

- The **right-click context-menu system is `MIG-077` — "App-Wide Right-Click Context Menus"** — opened v2.79 as a **full /migration, Boss-approved Option A**, with an Architect + Plan on disk:
  - `lab/reports/MIG-077-RIGHTCLICK-CONTEXT-MENUS-ARCHITECT.md`
  - `lab/reports/MIG-077-RIGHTCLICK-CONTEXT-MENUS-PLAN.md`
  - **Shipped + Boss-validated:** Phase A0–A3 (thin) → **A3-R rich/contextual rebuild** (the Boss "I want the full list — Delete, Rename, Move…" steer 2026-06-14). The shared builder **`src/lib/components/contextMenuBuilder.ts`** (`buildContextMenu(target, actions)`) emits the full note/folder/library menus. The note menu = Open · Open-in-new-tab · Rename · Move · Add tag · Copy path · Copy name · Reveal in tree · Suggest sources (md) · Delete — **DONE**.
  - **Paused ~2026-06-15** to finish MIG-080.

- **Remaining MIG-077** (per the Plan + Debt Register B in `00-MASTER-Bring-Up-Charter-and-Checklist.md` §7.B):
  - **A4** — extract a shared `<ConfidencePicker>` for Backlinks + Outgoing (fold the 2 duplicated inline confidence/archive popovers).
  - **A5** — fold GraphMind's `.gm-context-menu` (optional).
  - **Phase B** — B1 List-mode rows (Boss #1) · B2 Search results · B3 Tags · B4 Calendar · B5 Sky View bubbles · B6 in-editor wikilinks · B7 lower-value.
  - **Phase 4** — 3-agent audit + `/simplify` + close.
  - Plus Debt Register B "missing-entirely" surfaces → add a menu **or formally rule out** (Constraint-as-Design) per surface.

**Naming ruling for this session:** the work is tracked as **MIG-077 (resume)**. No new MIG number is allocated. The handover's "MIG-080 right-click" label is recorded here as corrected drift (does NOT change the decision Boss made — the right-click menus — only the accurate identity + the fact that the shared infra already ships, so this is *complete-coverage*, not greenfield).

**Predecessor → Replacement (Predecessor Lookup Rule):** the right-click system's predecessor is **MIG-077's `contextMenuBuilder.ts` + the shared menu render** (already in place). Remaining surfaces route through that SAME builder — same place, no relocation.

---

## MIG-077 Plan refresh (SO #8 cross-check workflow) — DONE
Ran `mig077-resume-crosscheck` (run `wf_d17fb72f-749`, 11 parallel Explore agents) verifying every remaining step against the **current** tree. Key refreshed findings:
- **Foundation** `ContextMenu.svelte` is live + complete (click-outside, Escape, viewport-clamp), consumed by file-tree/tabs/OrgChart/IndexPanel via `contextMenuBuilder.ts`. **One gap:** no RTL awareness (fixed `left/top`, no `dir`).
- **A4** — BacklinksPanel + OutgoingLinksPanel still carry the **byte-identical** inline `.conf-overlay` popover; no ConfidencePicker.svelte existed.
- **A5** — GraphMindView `.gm-context-menu` hand-rolled (Open·Focus·Pin·Suggest·Hide), already i18n+RTL; not flag-gated.
- **B1** — List-mode rows had ZERO menu (native WebView2 only); not virtualized; checkbox multi-select via `selectedPaths`.
- **B3** — post-MIG-080 there are TWO tag surfaces (right-rail `.rs-tag-chip` + Dashboard `.dashboard-tag`); `TagsPanel.svelte` is DEAD code.
- **B4** — `CalendarPanel.svelte` `.cal-cell`; the agent's "every day opens today" claim is **STALE** — `get_daily_note_path` (libraries.rs:5048) was fixed by MIG-079 §D (honors the clicked date). No bug.
- **Map/Sight** — both off-by-default Wings (`$appSettings.enabledFeatures?.constellationMap === true`; Sight flag-off). Deferred.
- **i18n** — all Phase-A keys complete ×15, zero gaps; net-new keys only at B3/B4/B5/B6 gates.

## Boss decisions (AskUserQuestion 2026-06-29)
1. **Order:** A4 → RTL hardening → B1 → B6 → B2 → B5 → B3 → B4 → A5 → audit. First Boss test at B1.
2. **Coverage:** Core 6 **+ diagnostic panels** (Inspector360, Knowledge Health, Review Pulse, Tasks, Forge/Canvas, Federation, Universe/Library Mgmt); **rule out** read-only navigators (Properties, Outline, Quick Switcher, Style Setter); Sight/Map deferred Wings.
3. **GraphMind:** fold into the shared engine.

## Build progress (in-tree, type-clean — svelte-check 0 errors throughout)
- **§A4 — shared `ConfidencePicker.svelte`** created (self-contained: owns popover UI + CSS + the confidence/archive IPC + dismissal). BacklinksPanel + OutgoingLinksPanel folded onto it; ~55 lines of duplication removed from each; dangling-ref grep clean. *Verification at the B1 gate (right-click a backlink → set confidence still works).*
- **§RTL hardening** — `ContextMenu.svelte` + `ConfidencePicker.svelte` now open toward the start side in RTL (`dir={$dir}` + `$isRTL` right-anchor); LTR byte-identical. ConfidencePicker dismissal switched to the ContextMenu document-listener pattern (click-outside + right-click + **Escape**), dropping the backdrop div → removed 1 a11y warning (321→320).
- **§B1 — List-mode right-click menu.** `NavFileItem` gains `data-path`; `NavFileList` gets ONE delegated `oncontextmenu` on `.nav-list-scroll` (reads `closest('[data-path]')` — no per-row listener); `NotebookNavigator` forwards `onNoteContextMenu` (main mode only). `+layout.handleListNoteContextMenu` builds the note menu via the shared builder, reusing every existing handler (open, open-in-new-tab, rename dialog, move dialog, safe add-tag dialog, copy path/name, **reveal-in-tree**, suggest sources [md], delete) — NOT via `handleOrgNodeMenuAction` (its open/reveal cases carry OrgChart-only side-effects). Own `listCtxMenu` state + render block.
- **NEXT (superseded — see scope expansion below).**

---

## SCOPE EXPANSION — Boss steer 2026-06-29: cover the WHOLE app right-click (match Obsidian, adapted)
Eisa re-sent the Obsidian RC reference images (Note / Folder / Link-selection / Editor-empty) confirming the target is the **full** Obsidian right-click across all menu families, **including fly-out submenus** (`Copy path ▸`, `Format ▸`, `Paragraph ▸`, `Insert ▸`). Ran coverage-audit workflow (`wf_70806068-d9c`, 4 agents) mapping every Obsidian item → Constellation capability. **Headline: ~45 of ~63 items already have the underlying op; this is mostly wiring + a submenu primitive + a few small ops.**

### Coverage verdicts (audit-backed)
- **Note (14):** reuse-now (11): Open · Open-new-tab · Open-to-right (split) · Open-new-window (second screen) · Move · Bookmark · Copy-path · Open-in-default-app (`open_path` lib.rs:136) · Show-in-explorer (`constellation_show_in_folder` lib.rs:108) · Rename · Delete. Small-new: Make-a-copy (note duplicate), Copy-path *relative*. Build (heavy): Copy-as-deep-link. DROP/DEFER: Merge, Presentation, Version-history(→Backup).
- **Folder (12):** reuse-now (6): New note/folder/base · Move · Rename · Delete (+Expand/Collapse). Small-new: New canvas (`create_canvas` infra exists), Copy-path, Show-in-explorer, Bookmark-folder. Build (heavy): Search-in-folder, Duplicate-folder. 
- **Library (NEW — Boss-added):** modeled on Folder RC, adapted: New note/folder/canvas/base · Search-in-library · Copy-path · Show-in-explorer · Bookmark; **Rename = rename library**; **"Delete" = remove/unlink the library (NEVER trash files — File-Over-App)**; NO Move / Make-a-copy. (Currently `getContextMenuItems` kind==='library' is create-only; MIG-008 put rename/remove in Library Manager — this ADDS them to the RC per Boss "similar to Folder RC", wiring to the Library-Manager ops. Confirm at gate.)
- **Link / selection (in-editor):** currently only Edit/Open/Remove. Build from existing ops: Open/Open-new-tab/Open-to-right · Add link · Add external link · Edit link · Cut/Copy/Paste/Paste-plain · Select all · Copy-path(resolve) · Bookmark · Open-in-default-app · Show-in-explorer · Reveal-in-tree. DROP: Rename/Move a link-target (editor = content surface). DEFER: new-window, presentation, new-drawing.
- **Editor formatting (~88% exists):** Bold/Italic/Strike/Highlight/Code/Comment/Clear · lists · H1-6/Body/Quote · Table/Callout/HR/Code-block/Math-block · clipboard ALL exist — gap is *organizing into `Format ▸ / Paragraph ▸ / Insert ▸`* + 4 small commands (external-link, inline-math, footnote, select-all). DROP: New-base-in-editor.
- **Foundation:** shared `<ContextMenu>` is FLAT → add **fly-out submenu support** (`MenuItem.submenu`). `EditorContextMenu` already has its own submenu (Headings) → reorganize.

### Boss decisions (AskUserQuestion 2026-06-29)
- Q1 **Defer/drop all as listed** (no silent drops).
- Q2 **Build all three** heavy backend ops (deep-link, folder-search, folder-duplicate) **+ add a Library RC** (similar to Folder RC).

### Revised cascade order
1. **§F-sub** — submenu (fly-out) support in `ContextMenu.svelte` + `MenuItem.submenu` *(foundation; internal)*.
2. **§F-ops** — new ops: note-duplicate, folder-duplicate (Rust recursive), folder-scoped search (Rust folderScope), deep-link `constellation://` (Tauri protocol), + frontend wiring helpers (relative-path, show-in-explorer/open-in-default for folders).
3. **§F-Note/Folder/Library** — enrich the shared builder (rich items + Copy-path ▸ submenu); enriches **file-tree + List-mode + library roots** at once. **[GATE]**
4. **§F-Editor** — reorganize EditorContextMenu into Format▸/Paragraph▸/Insert▸ + add external-link/inline-math/footnote/select-all + link-target ops. **[GATE]**
5. **B2 Search · B5 Sky · B3 Tags · B4 Calendar · diagnostic panels** (Inspector360, KH, Review Pulse, Tasks, Forge/Canvas, Federation, Universe/Library Mgmt). **[GATES]**
6. **A5 GraphMind fold** → **Phase-4 audit** → close.

(The earlier basic B1 wiring stays valid — it shares the builder, so it auto-enriches when §F-Note lands.)

---

## Build progress (cont.) — §F-sub + §F-Note/Folder/Library (Stage 1)
- **§F-sub — submenu fly-out support SHIPPED (in-tree).** `ContextMenu.svelte` rewritten: `MenuItem.submenu` (imported from `contextMenuBuilder.ts` as the one source of truth — fixed an svelte2tsx self-ref error), fly-out via `inset-inline-start: 100%` (RTL auto-flips), hover + click open, chevron `›`/`‹`. `contextMenuBuilder.MenuItem` gained `submenu?`.
- **§F-Note/Folder/Library — rich menus SHIPPED (in-tree), Stage 1 scope (existing-op reuse only):**
  - Builder: new `ContextActions` (`bookmark`, `copyPathRelative`, `openInDefaultApp`, `showInExplorer`) + `ContextTarget.bookmarked`; `copyPathItem()` emits **`Copy path ▸`** (from Library folder / from system root) when both rel+abs are wired, else flat; `bookmarkItem()` toggles label by `bookmarked`. **Group order re-laid to match Obsidian** (open · organize · copy/reveal · system · diagnostic · rename+delete). No tests assert order (verified) → safe.
  - `+layout.getContextMenuItems` (file-tree) + `handleListNoteContextMenu` (List-mode) wired: **note** gains Bookmark · Copy path ▸ · Open-in-default-app · Show-in-explorer; **folder** gains Bookmark · Copy path ▸ · Show-in-explorer; **library** gains Bookmark · Copy path ▸ · Show-in-explorer (on top of create). Helpers `toggleBookmarkPath` + `copyRelativePath` (relative = strip library-root prefix). All reuse existing ops (`open_path`, `constellation_show_in_folder`, `addBookmark`/`removeBookmark`/`isBookmarked`). svelte-check **0 errors**.
  - i18n: existing keys reused (`showInExplorer`/`openDefaultApp`/`copyPath`/`openNewWindow`/`revealInTree` already localized ×15). **4 net-new keys** (`bookmark`, `removeBookmark`, `fromLibraryFolder`, `fromSystemRoot`) added to en.json; localizer Workflow `wf_3839964a-036` translating ×14 (in-flight).
- **DEFERRED to §F Stage 2** (need more wiring / new backend): Open-to-right (split), Open-in-new-window (second-screen send), Make-a-copy (note dup), New canvas; the 3 heavy ops (deep-link, folder-search, folder-duplicate); Library rename/remove. **§F-Editor** (Format/Paragraph/Insert reorg + 4 commands + link-target ops) after.
- i18n ×14 applied (localizer `wf_3839964a-036`; tr/ur corrected vault→Library per terminology rule); all 15 parse + carry the 4 keys; bundle-grep verified (`fromLibraryFolder` + ar value in build/); release exe rebuilt fresh (touch lib.rs → re-embed; exe 11:29 > build/ 11:23; 0 errors).

## Stage 1 Boss test — results (2026-06-29)
- **Check 1 (rich note menu, file tree): PASS.**
- **Check 2 (Copy path ▸ fly-out submenu): PASS.**
- **Check 3 (Notes Navigator note menu): FAIL — menu renders perfectly but actions don't reflect.** Root cause: **`NotebookNavigator` is a SEPARATE data domain** (loads its own `allNotesWithMeta` on mount; only refreshes via its own `refreshData()` after its own batch ops). The shared `+layout` handlers (delete via `confirmDelete`→`deleteWithSetting`, move, rename) mutate disk + refresh the **tree**, but the Navigator never hears → stale view. The Delete DID fire (`+layout:5491` `deleteWithSetting`) → note went to system trash (recoverable); Navigator just didn't update = a silent-loss hazard. Violates **Additional screens are displays, not domains**.

### Boss ruling (2026-06-29): PARK the Notes Navigator + align it with the philosophy
- **B1 PARKED + DISABLED (interim, this session):** removed the `onNoteContextMenu` wiring from the `NotebookNavigator` mount + added `if (!onNoteContextMenu) return;` guard in `NavFileList.handleRowContextMenu` (so it falls through to the native menu, never a dead/​hazardous right-click). `handleListNoteContextMenu` kept ready to re-wire. Takes effect next build. The rich note RC still works in the **file tree** (Checks 1/2 PASS).
- **FOLLOW-UP (Boss-directed, substantial — own task):** rework the **Notes Navigator** into a true *display over shared data* (react to the core note-mutation events / read +layout's note list, not its own `collectLibraryNotesWithMeta` domain) so every RC action reflects live — AND align it with the Constellation philosophy (formulation, not a file-manager list). Then re-enable B1. Candidate for `/migration` + WA#5. NOT now.

## NEXT
- **Stage 2 Boss test (current build):** folder RC · library RC · confidence picker (A4) · Arabic/RTL — all on file-tree/backlinks, unaffected by the Navigator issue.
- **Then build §F-Editor (B6):** EditorContextMenu → Format▸/Paragraph▸/Insert▸ reorg + external-link/inline-math/footnote/select-all + link-target ops → rebuild → test.
- Then B2 Search · B5 Sky · B3 Tags · B4 Calendar · diagnostic panels · A5 GraphMind fold · Phase-4 audit.

## SCOPE ADDITION — Boss 2026-06-29: a "Style…" item in EVERY RC menu (style the function-in-hand)
**Concept (horse):** every right-click menu gains a **"Style…"** item that opens the Constellation **Style Setter** (CSS, MIG-070) **focused on the category that styles the right-clicked element** — a proximity bridge so the user jumps from any surface straight into its own styling controls (no opening the Setter + hunting). Contextual: note/folder/library/editor/tab/backlink/Sky each → its matching Setter category.
**Guards:** (a) **LL-032** — the Style Setter has a documented FREEZE history (rendering BUILTIN_THEMES as a gallery / even a `<select>`); the RC item must ONLY *navigate/open-at-category*, never touch that path. (b) Needs an **open-at-category** capability (likely a new `styleSetterFocusCategory` set before opening) + a surface→category map (gaps → omit the item or open the general/Frame category). Scoping via `wf_488671f8-120` (2 agents: Setter internals/capability/freeze-check + surface→category mapping). Folds into §F (added to the shared builder + EditorContextMenu so it appears on every surface).

### Scope findings (wf_488671f8-120) + build
- **Capability EXISTS:** `openStyleSetterToCategory(cat)` (`src/lib/stores/styleSetter.ts:26-27`) sets `styleSetterCategoryRequest` + `styleSetterOpen=true`; a `$effect` in `StyleSetter.svelte:872-877` navigates via `pickCategory`. Used today by Settings → Links. **No new capability to build.**
- **LL-032 SAFE:** open-at-category is pure store-flag navigation — never renders/lists `BUILTIN_THEMES` (the cursed freeze path). Confirmed.
- **12 categories:** interface · components · editor · global · links · sky · cns · calendar · globalTasks · org · index · cataloger.
- **§F-Style SHIPPED (in-tree, svelte-check 0):** builder `ContextActions.style?` + a 🎨 "Style…" group (before rename/delete). Wired: `+layout.getContextMenuItems` (note/folder/library → `interface`), `getTabContextMenuItems` (→ `components`), OrgChart `getOrgNodeMenuItems` (→ `org` via `handleOrgNodeMenuAction` new `case 'style'`), `IndexPanel.getIndexTermMenuItems` (→ `index`). i18n `contextMenu.style="Style…"` added to en (×14 batched with the editor-menu keys before next build). EditorContextMenu "Style…" → with §F-Editor (B6).
- **Defaults (confirm at test):** label "Style…"; "style the function in-hand" = the SURFACE right-clicked (file-tree note → `interface`, not `editor`).
- **Not yet rebuilt** — §F-Style + the navigator-disable land in the NEXT binary (batched with §F-Editor + the ×14 i18n).

## §F-Editor (B6) — SHIPPED in-tree (svelte-check 0), Boss "Go" 2026-06-29
Milestone `fa98bf6b` committed (Stage 1+2). Then §F-Editor:
- **CM6 handlers (CodeMirrorEditor.svelte) — safe additive cases:** `handleContextFormat` +`math` ($…$); `handleContextInsert` +`externalLink` (`[sel](url)`) +`footnote` (ref at cursor + `[^1]: ` def at doc end); `handleContextClipboard` +`selectAll`; `handleContextLinkAction` +`copyTarget` (copy the wikilink target). Import `openStyleSetterToCategory`; render passes `onStyle={() => openStyleSetterToCategory('editor')}`.
- **EditorContextMenu.svelte — reorganized to the Obsidian shape** (handlers UNCHANGED — only grouping moved, LL-023-safe): link-target actions (Open · Copy target · Edit · Remove) when on a `[[wikilink]]`; **Add link · Add external link**; **Format ▸** (Bold/Italic/Underline/Strike/Highlight · Code/Math/Comment · Super/Sub/Clear); **Paragraph ▸** (Bullet/Numbered/Task · H1-6/Body · Quote); **Insert ▸** (Footnote/Table/Callout/HR · Code-block/Math-block/Image); clipboard +**Select all**; **Style…** → `editor`. One-at-a-time `openSub` fly-outs; RTL-aware (`◂/▸`, `.ecm-sub.rtl`). Table/checkbox context sections preserved. New `onStyle?` prop.
- **DROPPED (per Boss Q1):** New-base-in-editor; Rename/Move on a link's target (editor = content surface). DEFERRED: link-target open-in-new-tab/reveal/open-in-default (need wikilink cross-library resolution — follow-up).
- **i18n:** 7 new en keys (format/insert/body/math/footnote/externalLink/copyTarget) + `style`; localizer `wf_8d59ecb7-ca2` translating 8 keys ×14 (in-flight). `selectAll` already existed ×15.
- **i18n DONE (8 keys ×14):** first localizer run (`wf_8d59ecb7-ca2`, Haiku) returned MALFORMED (metadata/JSON dumped into fields) — REJECTED (BASIC RULE, don't apply garbage). Re-ran strict + Sonnet (`wf_4ff4a628-926`) → clean native labels for all 14; applied (+8 each, all 15 parse + carry the keys). Lesson: for tight structured i18n, use Sonnet + a "labels-ONLY, no JSON/notes" prompt, not Haiku.
- **Binary rebuilt:** frontend 40.57s (no EPERM; `copyTarget` + zh `复制目标` bundle-verified), cargo re-embed (touch lib.rs). Carries §F-Editor + §F-Style + navigator-disable together.
## Stage 3 test — Check 8 FAIL → root-caused + fixed
**Check 8 (editor right-click): FAIL — native WebView2 menu showed, not Constellation's.** Root cause (NOT a regression of mine; pre-existing): the editor is at a **minimal perf baseline** (`CodeMirrorEditor.svelte:1402` "CORE ONLY… plugins stripped for performance baseline"; :1440 "Event listeners disabled"). **`editorEventHandlers()`** (line 268 — holds the **contextmenu** handler + Ctrl/Cmd+Click link-follow) was DEFINED but **never added to the `extensions` array** → the contextmenu handler never registered → no `preventDefault` → native menu. My §F-Editor reorg had restyled a menu that was never being triggered. (This is the bring-up "minimal editor, re-enable functions one at a time" state — memory `project_bringup_concept_papers`.)
**Fix:** added `editorEventHandlers()` to the editor extensions (`CodeMirrorEditor.svelte` ~:1415). Safe: `click` is Ctrl/Cmd-gated; `domEventHandlers` fire only on mouse events (zero typing-path cost, Perf Rule 1). Render chain confirmed: handler → `contextMenuVisible=true` → `{#if contextMenuVisible && view}<EditorContextMenu>` (:1724). svelte-check 0. Rebuilding to re-test.

## Check 8 — SECOND root cause (the real one): WRONG EDITOR FILE (LL-029 ×2)
After the first fix (re-add editorEventHandlers to CodeMirrorEditor) STILL showed the native menu, Boss corrected: **"Our NotePane is in Live Preview by default."** Truth:
- **`CodeMirrorEditor.svelte` is DEAD** — `grep "import.*CodeMirrorEditor|<CodeMirrorEditor" src/` = EMPTY. Both my §F-Editor edits there were on dead code. **Reverted** (`git checkout`).
- **`NotePane.svelte` IS the live note editor** (Live Preview default-on; own CM6 + own command helpers `wrapSelection`/`insertLinePrefix`/`insertAtCursor`/`clearFormatting`; toolbar + heading/list/insert menus). It **never rendered EditorContextMenu** → native RC menu. THAT is the gap.
- Architecture saved to memory `project_editor_architecture_notepane` (+ MEMORY.md) per Boss "update your knowledge."
**Fix (body RC):** wired `EditorContextMenu` INTO NotePane — `oncontextmenu` on `editorEl` → `handleEditorContextMenu` (caret lands at click; detects link/heading context) → renders `<EditorContextMenu>` with callbacks (`ecmFormat`/`ecmInsert`/`ecmHeading`/`ecmList`/`ecmClipboard`/`ecmLinkAction`/onStyle→`editor`) mapped to NotePane's OWN commands (reuse, no reinvention). svelte-check 0. Rebuilding.
**Frontmatter RC (Boss also wants):** the frontmatter shows as the **PropertyEditor** "Properties" panel (separate from editorEl) in Live Preview; in Source mode the raw `---` is in the editor (now covered by the body RC). The Properties-panel RC shape is a Boss design call (formatting menu doesn't fit a property form) — ASK before building.

## Check 8 — body RC works, but TWO issues (Boss) → fixed
Boss: Check 8 now shows Constellation's editor menu, BUT (1) Format/Paragraph/Insert submenus don't open, (2) the menu style differs from the file-tree RC.
- **Root cause (both):** the note RC used the separate `EditorContextMenu.svelte` (`.ecm`, its own styling, no icons) whose `overflow-y:auto` forces `overflow-x:auto` → the `left:100%` fly-outs were **clipped** by the scroll container (stray horizontal scrollbar visible).
- **Fix:** **dropped EditorContextMenu for the note RC; NotePane now renders the SHARED `<ContextMenu>`** (same chrome + icons as the file tree; fly-out submenus proven in Check 2; no overflow clip). New `getEditorMenuItems(): MenuItem[]` builds the menu (link-target section when on a `[[wikilink]]`; Add link/external; Format▸/Paragraph▸/Insert▸ submenus; clipboard+Select all; Style…) reusing the ecm* command helpers. svelte-check 0. `EditorContextMenu.svelte` now unused (cleanup later).

## Frontmatter RC — Boss ruling: "all the above" (NOT only property-specific)
Boss wants the Properties-panel RC to include the property actions (Copy value/name, Remove, Add) **+** the editor menu **+** Style… — comprehensive. Build NEXT (needs PropertyEditor wiring), after the body-RC fix is Boss-confirmed.

## §F-Editor — Boss re-test: Checks 8–11 PASS (committed 698da978); submenu truncation fixed
- Check 8 (menu + consistent style) · 9 (commands) · 10 (wikilink) · 11 (Style…) **PASS**. Check 12 (Notes Navigator → native) = parked-as-designed.
- **Submenu truncation (Boss finding):** the fly-out ran off the right viewport edge. Fixed in shared `ContextMenu` — `openSubmenu()` measures + flips the fly-out LEFT (`.ctx-submenu.flip`) when no room on the right. Space-based (LTR+RTL).

## §F-Editor-Frontmatter — Boss ruling "all the above" → BUILT (in-tree, svelte-check 0)
Frontmatter (Properties panel) RC = property actions + the full editor menu + Style. NotePane owns it (has the editor commands + view); PropertyEditor emits the row + exposes add/remove.
- **PropertyEditor:** new `onPropContextMenu(prop, idx, x, y)` prop; `oncontextmenu` on `.pe-row`; `addProperty`/`removeProperty` now `export`ed (callable via `bind:this`).
- **NotePane:** `bind:this={propEditorRef}`; `handlePropContextMenu` → `fmCtxMenu` state; `getFrontmatterMenuItems()` = Copy value · Copy property name (reuse `copyName`) · Remove property · Add property (reuse `addProperty`) · — · `getEditorMenuItems(false)` (editor menu + Style). `getEditorMenuItems` refactored to take `onLink`. Renders the shared `<ContextMenu>`.
- i18n: 2 new en keys (`copyValue`, `removeProperty`); localizer `wf_bf4e9973-8a6` ×14 (in-flight).
- Concept note: editor formatting commands from the frontmatter act on the note BODY (the property fields are inputs) — per Boss "all the above"; flag at test if undesired.

## BLOCKER — binary locked by the running app (LL-028)
The flip-fix cargo re-embed FAILED ("Access is denied"); `constellation.exe` (PID 50980) is running → can't overwrite. **Boss must close Constellation**, then ONE rebuild lands flip-fix + frontmatter RC + the ×14 i18n.

## §F-Editor-Frontmatter — Boss-validated → COMMITTED 42b0a585
Truncation fix + frontmatter RC (Copy value/name, Remove, Add, + editor menu + Style) all PASS. Editor surface COMPLETE.

---

## PIVOT (Boss 2026-06-29): MIG-077 PAUSED → MIG-088 "Style Setter Completeness"
Boss, via the new "Style…" RC, opened the Style Setter and found gaps: the **frontmatter property tags** (`.pe-tag`) aren't styleable (they use `--pill-*` + a fixed `--background-modifier-border-focus`/white, NOT the Setter's `--tag-bg`/`--tag-color`/`--tag-radius` "Tags & callouts" control). Boss ruled: (1) give the property tags **their own Style Setter control**, and (2) **sweep the whole app** for every element NOT exposed in the Style Setter + add controls — a separate larger pass that pauses MIG-077's remaining surfaces.

### SO #5 STATE-OF-STANDING — MIG-077 (App-Wide Right-Click Menus), paused 2026-06-29
- **Verified-shipped + committed (this session):** §A4 ConfidencePicker fold · §RTL hardening · §F-sub submenu engine + **left-flip on overflow** · §F rich Note/Folder/Library menus (Bookmark, Copy path▸, Open-in-default-app, Show-in-explorer) · §F-Style "Style…" on file-tree/tabs/OrgChart/Index/**editor** · §F-Editor note body RC (NotePane via shared ContextMenu; Format▸/Paragraph▸/Insert▸; new commands math/footnote/external-link/select-all/copy-target) · **§F-Editor-Frontmatter** comprehensive Properties-panel RC. i18n: 14 new contextMenu keys ×15. Commits `fa98bf6b` · `698da978` · `42b0a585` (+ the Stage-1+2 milestone). All Boss-validated.
- **Parked (in-tree, disabled):** Notes Navigator RC (B1) — Navigator is a separate data domain (silent-delete hazard); disabled; rework = task `task_fcc8396c`.
- **REMAINING MIG-077 (not started):** B2 Search · B5 Sky View · B3 Tags · B4 Calendar · the diagnostic panels (Inspector360/KH/Review/Tasks/Forge/Federation/Universe-Lib-Mgmt — Boss chose "core + diagnostic") · A5 GraphMind fold · Phase-4 audit · orientation v-bump (§F completion). RESUME after MIG-088.
- **Dead/cleanup:** `EditorContextMenu.svelte` now unused (NotePane uses the shared ContextMenu) — delete in cleanup. `CodeMirrorEditor.svelte` is dead (memory `project_editor_architecture_notepane`).
- **Doc drift:** orientation v3.16 is 934 KB (over the 1500-line split threshold) — split pending.

### MIG-088 plan
- **Tag fix (decided):** new "Property tags" Style Setter element (background/text/radius/height) wired to `.pe-tag`.
- **Completeness audit (Architect/discovery):** fan out across surfaces → list every user-visible element NOT exposed in the Style Setter + propose its control. Then Plan (prioritize) → Build (add controls + wire) → Audit. Treat as /migration (touches the Setter schema + many components).
- Audit launched: `mig088-stylesetter-completeness-audit`.

### MIG-088 audit DONE (wf_703dfdca-2e4) + Boss scope = "EVERYTHING (149)"
- **149 worth-restyling elements (50 high)** across all surfaces; Setter covers 262 vars/13 cats. Full list: `lab/reports/MIG-088-STYLESETTER-AUDIT.md`.
- **Key pattern:** ~half are repeated *cognitive-vocabulary* colour sets hardcoded per-surface — **Maturity / Confidence / Origin / Stage / Search-match-category** — to be CONSOLIDATED into shared controls (style once → everywhere) + the frontmatter Property tags + Taxonomy pills (Boss's trigger).
- **Boss ruling:** build EVERYTHING (all 149), semantic consolidation first → per-surface tail. Multi-session.
- **Plan:** `lab/reports/MIG-088-STYLESETTER-COMPLETENESS-PLAN.md` — 10 phases (1 Frontmatter+mechanism · 2 Shared semantic colours · 3 Editor · 4 Chrome · 5 Panels · 6 Search/Index · 7 Sky/Org/Map · 8 Calendar · 9 Dialogs/Global · 10 Audit). Each: new Setter element/control + preview case + component wired to the var with a fallback to today's value (byte-identical until edited; LL-032-safe — no `BUILTIN_THEMES`). i18n ×15 per label. svelte-check 0 + Boss test per phase.
- Then B2 Search · B5 Sky · B3 Tags · B4 Calendar · diagnostic panels · A5 GraphMind · Phase-4 audit · orientation v-bump at §F completion.

### MIG-088 §Phase 1 BUILT (Properties category) — awaiting Boss test
The add-an-element recipe, learned + applied:
- **ELEMENTS** (StyleSetter.svelte): `pTags` (Property tags: `--pe-tag-bg`/`--pe-tag-text-color`/`--pe-tag-radius`/`--pe-tag-height`) + `pTaxo` (Taxonomy pills: `--pe-taxo-bg`/`--pe-taxo-text-color`/`--pe-taxo-radius`). Same control format as `cTags`.
- **CATEGORIES**: new `{ key:'frontmatter', name:'Properties', surface:'editor', elements:['pTags','pTaxo'] }`.
- **3-zone**: added `frontmatter` to the twoZone exception — KEY finding: `.ss-stage` lives inside `.ss-center`, which `.ss--twozone` hides (relies on live-behind), but the right-anchored panel would occlude the right-sidebar Properties panel → Properties needs a dedicated centre preview. (cTags/chrome `.ss-focus` previews are effectively dead in their twoZone categories — live-behind instead.)
- **Preview**: `pk==='pTags'/'pTaxo'` → a mini Properties-panel card (`.ss-feprops` on `.ss-fcard`), `.ss-fep-hot` rings the selected row; pills read the same draft vars w/ same fallbacks → re-colour live. Fills the centre (Full-Center-Zone rule).
- **Wire** (PropertyEditor): `.pe-tag` + `.pe-taxo-pill` now read the new vars, fallback-chained to today's exact values (`--background-modifier-border-focus`/#fff/`--pill-radius`/`--pill-height`) → byte-identical until edited.
- **RC routing** (NotePane): `getEditorMenuItems(onLink, styleCat='editor')`; frontmatter menu passes `'frontmatter'` → "Style…" in the Properties RC opens the new category ("style the function in-hand"). Note-body "Style…" still → 'editor'.
- **i18n**: category reuses existing `properties` slug (zero cost); 2 new labels `property_tags`/`taxonomy_pills` ×15 (Sonnet, consistent w/ each locale's tag/properties terms; "pills"=badge/chip, never pharmaceutical). control labels reuse existing slugs. LL-032-safe (no BUILTIN_THEMES). svelte-check 0.
- **DEFERRED to Phase 1b:** taxonomy-pill TIER colours (`tierColorForId` is JS-computed per row) — wrap to user vars later.
- Phase 1 Boss-validated (A/B/C Pass) → COMMITTED `dbf5507e`. Recipe banked to memory `project_stylesetter_add_element_recipe`.

### MIG-088 §Phase 2 — colour map + design decision + §2a BUILT (awaiting test)
- **Colour map** (wf_1c77386c-019): `lab/reports/MIG-088-PHASE2-COLOR-MAP.md`. KEY FINDING: every semantic state uses DIFFERENT colours per surface today (wilting = green@40% file-tree / lime `#a3e635` Map / `#16a34a` Sky, etc.).
- **Boss ruling: "unify on demand"** (per-surface fallback) — `var(--<set>-<state>, <this surface's current value>)` → byte-identical until the user sets the shared var, then all surfaces snap. Alternative (canonical-now) rejected (would change current look).
- **Scope split:** CSS surfaces wire directly; **Sky** (PIXI, own `--skyview-maturity-*` control) KEPT separate; **Map/OrgChart** (D3) → Phase 7 via getComputedStyle.
- **§2a Maturity BUILT:** wired file-tree (`.note.mat-*` FileTree.svelte:216), tabs (`.tab-maturity.mat-*` +layout:8737), Inspector360 (`MATURITY_COLORS` map) to `--maturity-{seed,sapling,evergreen,canonical,wilting}` (fallbacks = exact current). New Setter element `cogMaturity` (Maturity) in new category **"Cognitive colours"** (key `cognitive`, 3-zone, legend preview). i18n: 2 new slugs `seed`+`cognitive_colours` ×15 (reused each locale's existing maturity-seed + standard "cognitive"); Maturity/Sapling/Evergreen/Canonical/Wilting reuse existing slugs. svelte-check 0.
- **Sub-steps queued:** §2b Confidence · §2c Origin · §2d Stage · §2e Match-category.
- §2a Maturity Boss-validated (Pass) → COMMITTED `ea68a565`.

### MIG-088 §2b+§2c+§2d BUILT (one batch) — awaiting test
Concept refinements (Concept-Before-Function): EXCLUDED BacklinksPanel traversal-chip "tiers" from Confidence (link-weight tiers ≠ note confidence — belongs with link tiers later); EXCLUDED the note stage badge from Stage (it's `--text-muted` for all stages today — wiring would ADD colour, not consolidate).
- **§2b Confidence** → `--confidence-{hypothesis,evidence,established,contested}`: ConfidencePicker `.conf-dot-*` (CSS, color-mix/accent fallbacks), KH `confidenceColors` map. Note: ConfidencePicker + KH use DIFFERENT palettes → per-surface fallback preserves both until unified.
- **§2c Origin** → `--origin-{received,discovered,mixed,none}`: Inspector360 `ORIGIN_COLORS` map, ProvenancePanel `originColor()` fn. (No conflicts — identical hex across both.)
- **§2d Stage** → `--stage-{spark,birth,growth,maturity,dormancy,archival}`: KH `stageColors` map (lifecycle cards).
- **Setter:** 3 new elements (cogConfidence/cogOrigin/cogStage) added to "Cognitive colours" category + legend previews.
- **i18n:** 13 new slugs ×15, reusing each locale's in-app vocab (`knowledgeHealth.confidence.*`, `notePane.stage.*`, `provenancePanel.mixed`) for state names + hand-translated group names (confidence/origin/stage). svelte-check 0.
- **§2e Match-category** = next (CM6 editor-search + SearchHub; fiddlier — own sub-step).

### MIG-088 §2b-2d Boss-validated (Pass) + 2 Arabic fixes + §2e BUILT — Phase 2 complete (CSS surfaces)
- §2b/§2c/§2d → COMMITTED `2d7f1ac3`. Boss Arabic review found 2 issues, both fixed (`372e1b29`):
  - `styleSetter.labels.maturity` was NEVER defined → English fallback in ALL locales (rail element name + Stage maturity row). Added ×15 (reused `notePane.stage.maturity`; ar = "نضوج" per Boss).
  - ar `mixed` "مختلطة"→"مختلط" in styleSetter.labels AND source provenancePanel.mixed.
- **§2e Match-category** → COMMITTED `605642d8`: editor search-highlight (NotePane CM6 `.cm-hl-*` ×6), SearchHub `categoryColors`; `cogMatch` element + legend. `--match-category-{title,content,tag,wikilink,property,semantic,structured}`. i18n: structured + match_category ×15 (rest existed + translated). Sky badges kept; OrgChart/Map/Sight D3 → Phase 7. svelte-check 0.
- **PHASE 2 COMPLETE on CSS surfaces** (all 5 cognitive sets: Maturity/Confidence/Origin/Stage/Match). Canvas/D3 surfaces deferred to Phase 7.
- Combined Arabic+§2e binary rebuilding (bxpj76vm2) for one Boss test.

### MIG-088 §2e Boss-validated (All Pass) + 2 findings
Phase 2 fully Boss-validated. Two findings on the §2e test:
- **Finding 1 (FIXED → §2f, `c1bed90e`):** SearchHub result match-highlight was uniform (accent) except tags. Now per-category, reusing the Match-category colours (badge + editor + result highlight all unify on one control). Built; awaiting test.
- **Finding 2 (QUEUED — Boss wants):** *search results need their own right-click menu.* This is MIG-077 **B2 (Search RC)** — a context menu on SearchHub result rows (open / open in new tab / copy link / reveal in tree / Style… etc.). Build when MIG-077 right-click resumes (after MIG-088), or as a focused item if Boss prioritises. Not started.
