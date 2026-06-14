# PLAN — MIG-077: App-Wide Right-Click Context Menus

**Date:** 2026-06-14 · **Phase:** 2 (Plan) · **Architecture:** Option A (Boss-approved 2026-06-14) — shared `ContextMenu` for action-menus + a separate shared `<ConfidencePicker>` for the confidence radio-group. Architect: [MIG-077-RIGHTCLICK-CONTEXT-MENUS-ARCHITECT.md](MIG-077-RIGHTCLICK-CONTEXT-MENUS-ARCHITECT.md).

Each step = ONE landable commit with a verification clause. **[GATE]** = Boss-testable, pause per the Testing Instructions Rule. Steps without a gate are internal (svelte-check + 10-char type-burst smoke + bundle-grep on any UI string). Every UI string goes through `$t()` and into all 15 locales; every proper build = `npm run build` THEN `cargo build --release` (frontend-only here, but re-embed required) verified by grepping `build/` for the new string.

Standing invariants for EVERY step (from the Architect §3): the 2 working menus (file tree + library headers) stay byte-identical; no `invoke()` on the right-click build path (IPC on click only); virtualized lists stay virtualized; delegation (not per-row listeners) on non-virtualized lists; RTL via logical CSS + `detectDir()`; Escape + click-outside on every menu; second-screen surfaces emit callbacks, never own ops; `EditorContextMenu` untouched.

---

## Phase A — Consolidate the 6 hand-rolled menus (one surface per commit)

### A0 — `MenuItem` Tier-1 extension *(internal)*
- `ContextMenu.svelte`: add optional `separator?: boolean` and `disabled?: boolean` to `MenuItem`; render a `<hr>`-style divider for separators and a non-interactive greyed row for disabled items. Backward-compatible (existing items omit both → render exactly as today).
- **Verify:** svelte-check 0; file-tree + library-header menus render and behave identically (regression check on the only two current `ContextMenu` consumers); a separator + a disabled item render correctly via a throwaway test item, then removed.

### A1 — Tabs → `ContextMenu` **[GATE]**
- Replace the inline `.tab-ctx-menu` (`+layout.svelte:233`, render `:5648-5662`) with `getTabContextMenuItems(tabId): MenuItem[]` (Pin/Unpin dynamic label · separator · Close [disabled if pinned] · Close Others · Close to the Right · Close to the Left · Close All · separator · Copy Path · Copy Name) rendered via the existing `<ContextMenu>` machinery.
- i18n: add `tabContextMenu.{close,closeOthers,closeToRight,closeToLeft,closeAll}` ×15; reuse existing pin/unpin/copyPath/copyName keys (closes the Tabs hardcoded-English violation).
- **[GATE]** Right-click a tab → Pin/Unpin toggles; Close is greyed on a pinned tab; Close-Others/Right/Left/All behave as before; Copy Path/Name work — verified in EN **and** Arabic (RTL).

### A2 — IndexPanel → `ContextMenu` **[GATE]**
- Replace `.gp-context-menu` (`IndexPanel.svelte:1245`/`:1335`) with `getIndexTermMenuItems(term): MenuItem[]` (one dynamic Hide/Show item). Keep the right-click attach INSIDE the existing `VirtualList` row.
- i18n: add `indexPanel.{hideTerm,showTerm}` ×15 (closes the Index hardcoded-English violation).
- **[GATE]** Right-click a term → Hide (greys/hides); right-click again → Show. The term list still scrolls smoothly (virtualization intact).

### A3 — OrgChart → `ContextMenu` **[GATE]**
- Replace `.oc-fs-ctx` (`OrgChart.svelte:714`/`:978`) with `getOrgNodeMenuItems(node): MenuItem[]` (Open for a note; Expand/Collapse for a container). Reuse `contextMenu.open` / `sidebar.expandAll`-family keys.
- **[GATE]** Right-click a note box → Open; right-click a folder/library box → Expand/Collapse.

### A4 — Extract shared `<ConfidencePicker>` **[GATE]**
- New `src/lib/components/ConfidencePicker.svelte`: header ("Set confidence") + 4 swatch radios with active-state + separator + Archive. Click-outside + Escape (mirror the current `.conf-overlay`). Reuse `linkConfidence.*` keys.
- Point BOTH `BacklinksPanel.svelte` (`:260-277`) and `OutgoingLinksPanel.svelte` (`:208-225`) at it; delete the two duplicated inline popovers.
- **[GATE]** In a note with backlinks: right-click a backlink row → set each confidence level (dot + highlight reflect current); Archive removes the link. Repeat on an outgoing-link row → identical behavior. EN + RTL.

### A5 — GraphMind → `ContextMenu` *(optional — Boss decision at this gate)* **[GATE if done]**
- Replace `.gm-context-menu` (`GraphMindView.svelte:1154`) with `getGraphNodeMenuItems(node)`. Lowest priority — already i18n'd + RTL-aware, so value is single-source purity vs. leaving the lower risk. **Decide at the A4 gate.**

*End of Phase A: run `/simplify` on the cumulative diff. Six hand-rolled menus → two primitives (`ContextMenu` + `ConfidencePicker`), all i18n-clean. Standing Order: session log + (if user-visible) help/manual + orientation bump.*

---

## Phase B — Extend to missing surfaces (priority order)

### B1 — List-mode rows (BOSS #1) **[GATE]**
- Add `data-path` to `NavFileItem.svelte` rows + a single delegated `oncontextmenu` on `NavFileList`'s scroll container (reads `e.target.closest('[data-path]')` — one listener, virtualization-safe). Build the menu in `NotebookNavigator` via `getListRowMenuItems(note): MenuItem[]` reusing existing ops.
- **Item set — Boss decision at this gate.** Proposed: Open · Open in new tab · Rename · Move · Add tag · Delete (+ optional: Reveal in tree, Copy path).
- **[GATE]** Right-click a List-mode note → each item performs its action; works with the existing multi-select (right-clicking within a selection acts on the selection where sensible).

### B2 — Search results **[GATE]**
- Delegated `oncontextmenu` on `SearchHub` `.sh-item` rows; `getSearchResultMenuItems(result)` (Open · Open in new tab · Copy path/name · Reveal in tree — reuse existing keys). **[GATE]**

### B3 — Tags rows **[GATE]** *(item set Boss decision at gate)*
- `oncontextmenu` on `.tp-tag` (`TagsPanel.svelte`) + `onTagContextMenu` callback. Proposed (read-only nav): Filter by tag · Copy tag. Heavier ops (Rename / Remove-from-all = cascade writes) deferred unless Boss wants them. **[GATE]**

### B4 — Calendar day cells **[GATE]** *(Boss decision at gate — include or skip)*
- `oncontextmenu` on day cells; proposed: New note dated this day · Jump to this day. **[GATE]**

### B5 — Sky View bubbles **[GATE]**
- One `oncontextmenu` on the `LocalSkyView` canvas; hit-test the existing `nodePositions`; emit a callback (display-not-domain). Items: Open · Pin · Show relations. **[GATE]**

### B6 — In-editor wikilinks **[GATE]**
- New wikilink branch in the CM6 `contextmenu` handler (`CodeMirrorEditor.svelte:301`) — when a `[[wikilink]]` token is under the pointer, show Open · Open in new tab · Copy target. `EditorContextMenu` formatting menu unchanged. **[GATE]**

### B7 — Lower-value surfaces *(Boss-prioritized, only if wanted)*
- NoteGrid / Map / bookmarks / index-mention rows — one commit each, same idiom. Some may not warrant a menu per "Constraint as Design." **Decide priority at the end of B6.**

---

## Phase 4 — Audit + close
- 3 parallel agents per the Migration Rule: (1) invariants (the 2 original menus + all consolidated menus behave correctly; no `invoke()` on build paths; virtualization intact); (2) drift (any new hardcoded strings; any locale missing a key); (3) migration-path (first-boot, RTL, second-screen, theme variants). Plus `/simplify` on the full diff. Milestone tag + orientation bump + session log.

---

## Deferred / defaulted open questions (Architect §5)
- **Q5 GraphMind (A5):** decide at the A4 gate (default: consolidate for single-source).
- **Q2/Q3/Q4/Q6 item sets + lower-value priority:** decided at each surface's gate (proposals above).
- **Q7 NavFileList virtualization:** SEPARATE task (MIG-077 uses delegation so it's not blocked) — default: spin off later.
- **Q8 keyboard arrow-nav:** default DEFER to a later a11y MIG (absent today; non-regression). If added, lands in the shared `ContextMenu` so all menus benefit.

## Rollback
Each step is an independent commit; revert any single commit to undo that surface. A0's `MenuItem` extension is additive (no consumer breaks if reverted after later steps are also reverted). No schema, no data-path, no IPC contract change — pure render layer.
