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
