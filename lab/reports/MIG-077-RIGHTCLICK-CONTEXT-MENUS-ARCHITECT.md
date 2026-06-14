# ARCHITECT — MIG-077: App-Wide Right-Click Context Menus

**Date:** 2026-06-14 · **Phase:** 1 (Architect) — read + design only, no code.

**Function in hand:** the app-wide right-click pass — consolidate every hand-rolled context menu onto the shared `ContextMenu` component (`src/lib/components/ContextMenu.svelte`) and extend menus to the surfaces that lack them, keeping the specialized `EditorContextMenu` untouched.

Origin: Boss observations during the delete-path test (2026-06-13/14) — (#1) no right-click batch delete in List mode; (#2, shipped) the native confirm box; (#3) "right-click should include every aspect of the app." Boss chose the full four-phase `/migration`.

---

## 0. Inventory corrections (verified against source — the prior quick-inventory was wrong in three places)

**There is no single shared menu today — there are TWO `ContextMenu` consumers and SIX independent hand-rolled menus.**

| # | Surface | Quick-inventory said | Actual (verified) | Evidence |
|---|---------|----------------------|-------------------|----------|
| 1 | File tree notes/folders | shared ContextMenu | ✅ shared `ContextMenu` | `+layout.svelte:7068` render; items `getContextMenuItems` `:4497` |
| 2 | Library/universe headers | shared ContextMenu | ✅ shared (same render) | `+layout.svelte:4476` → same builder |
| 3 | Open-note tabs | INLINE | ✅ INLINE `.tab-ctx-menu`, **hardcoded English**, separators + disabled | `+layout.svelte:233`, render `:5648-5662` |
| 4 | Editor text | EditorContextMenu (keep) | ✅ specialized `EditorContextMenu` | `CodeMirrorEditor.svelte:301` |
| 5 | Backlinks rows | INLINE | ✅ INLINE `.conf-menu` (confidence + archive) | `BacklinksPanel.svelte:86`, render `:260-277` |
| 6 | Outgoing-links rows | INLINE | ✅ INLINE `.conf-menu` — mirror of Backlinks | `OutgoingLinksPanel.svelte:130`, render `:208-225` |
| 7 | Index term rows | INLINE | ✅ INLINE `.gp-context-menu`, **hardcoded English** | `IndexPanel.svelte:1245`, render `:1335-1343` |
| 8 | OrgChart (Sky View) nodes | "uses ContextMenu" | ❌ **WRONG — INLINE `.oc-fs-ctx`** | `OrgChart.svelte:714`, render `:978-986` |
| 9 | GraphMindView nodes | listed MISSING | ❌ **WRONG — already INLINE `.gm-context-menu`** (i18n'd, RTL-aware) | `GraphMindView.svelte:1154-1161` |

**Net:** 2 shared consumers, **6 hand-rolled** (Tabs, Backlinks, Outgoing, IndexPanel, OrgChart, GraphMind), 1 specialized (Editor). Tabs + IndexPanel ship **untranslated hardcoded English** — a live i18n-rule violation MIG-077 closes as a side benefit.

**Genuinely missing (verified):** List-mode rows (`navigator/NavFileItem.svelte` — Boss #1), Search results (`SearchHub.svelte` `.sh-item`), Tags rows (`TagsPanel.svelte` `.tp-tag`), Calendar day cells (`CalendarPanel.svelte`), Sky View bubbles (`LocalSkyView.svelte`, canvas), in-editor wikilinks. Lower-value: NoteGrid, Map, bookmarks, index-mention rows.

---

## 1. Territory map

### 1.1 The shared `ContextMenu` — real API (`ContextMenu.svelte`, 111 lines)
Props: `{ x, y, items: MenuItem[], onClose }`. `MenuItem = { label: string; icon?: string; action: () => void; danger?: boolean }`.
Supports: flat button list, emoji `icon`, `danger` (red), viewport **clamp** (slide left/up — no corner-flip), click-outside + Escape (10ms arm delay, listeners cleaned in onMount return — Perf Rule 4 OK), RTL via `text-align:start` + logical props.
**Does NOT support:** submenu/nested, separator, disabled, section header, active/checked state, custom row widget (swatch), keyboard arrow-nav. (Dynamic labels ARE expressible today — the builder runs per-open.)

### 1.2 The canonical "good" pattern — `getContextMenuItems` (`+layout.svelte:4497-4581`)
A per-surface pure function → `MenuItem[]`, branching on context, every label via `$t()`, `danger` on Delete, single `{#if contextMenu}<ContextMenu/>` render (`:7068`), one `$state` object set from `e.clientX/Y`. **This is the target shape for all six hand-rolled menus.**

### 1.3 The six hand-rolled menus + cost to converge
- **(a) Tabs** (`:233`/`:5648`) — 9 items, 2 separators, 1 disabled, hardcoded EN. Needs `separator`+`disabled` on MenuItem; new keys `closeOthers/closeToRight/closeToLeft/closeAll/close` ×15 (pin/unpin/copyPath/copyName already exist).
- **(b)(c) Backlinks + Outgoing confidence picker** (`:260-277` / `:208-225`) — IDENTICAL: header "Set confidence" + 4 swatch radios with active-state + separator + Archive. Strings i18n'd (`linkConfidence.*`). **The one true capability gap** (radio group, not an action list).
- **(d) IndexPanel** (`:1245`/`:1335`) — 1 dynamic Hide/Show item, hardcoded EN. Trivial; add `indexPanel.hideTerm/showTerm` ×15. Right-click already attaches inside VirtualList.
- **(e) OrgChart** (`:714`/`:978`) — 1 item (Open / Expand-Collapse). Trivial.
- **(f) GraphMind** (`:1154`) — 4 items (Open/Focus/Pin/Hide), already i18n'd + RTL-aware. Lowest-priority consolidation (value = single-source only).

### 1.4 The capability GAP
Only the **confidence picker** forces a component decision (radio group + swatches + header). Everything else is small additive `MenuItem` extension. Split:
- **Tier-1 (cheap, additive, breaks nothing):** `separator?`, `disabled?` on `MenuItem`.
- **Tier-2 (the real fork):** how to express the confidence radio-group → §2.

---

## 2. Design options

### Option A — Thin extension + extract a shared `<ConfidencePicker>` *(RECOMMENDED)*
Add only `separator?`/`disabled?` to `MenuItem`; migrate Tabs/IndexPanel/OrgChart/GraphMind onto `ContextMenu`; extract the duplicated confidence popover into ONE shared `<ConfidencePicker>` (it's a radio group, not a menu). **Speed: fastest · Effort: low · Risk: low.** Two primitives, each used many times, zero copy-paste.

### Option B — Grow `ContextMenu` into a full menu kit
`MenuItem` becomes a tagged union (`action|separator|header|radio` + swatch/checked). All 6 + the picker converge onto one component. **Speed: medium · Effort: medium-high · Risk: medium** (re-renders the 2 *working* menus through an enlarged template; pushes toward a mini-framework — tension with "Constraint as Design").

### Option C — Submenu-capable `ContextMenu` (nested flyouts)
B plus nested flyouts. **Speed: slowest · Effort: high · Risk: high — and NO current caller needs nesting.** YAGNI.

**Recommendation: Option A.** Reasoning: (1) satisfies the goal (one source of truth for action-menus + one for the picker); (2) Form-Aligns-To-Purpose — the confidence picker is a single-select radio group, not a command list, so forcing it into a menu fills degrees of freedom it doesn't need; (3) lowest risk to the two menus that already work (only two additive booleans); (4) smallest change that closes the two i18n violations + removes the Backlinks≡Outgoing duplication; (5) matches mature practice (VS Code/Obsidian/ARIA separate `menu` from single-select controls). If the Boss wants strictly-everything-on-`ContextMenu`, fall back to **B** (never C).

**Q1 — submenu needed?** No (no in-scope caller nests; confidence = flat radio; heading picker = out-of-scope Editor). **Q2 — per-row handlers vs Perf Rule 3 (virtualized lists):** virtualized lists attach per-VISIBLE-row safely (IndexPanel does this). NavFileList (#1) + SearchHub are NOT virtualized → use **event delegation** (one `oncontextmenu` on the scroll container reading `e.target.closest('[data-path]')`); flag NavFileList virtualization as a SEPARATE task (don't block #1). Canvas surfaces (Sky View/OrgChart/GraphMind) → one canvas `oncontextmenu` + hit-test the existing position array. **Q3 — item set:** universal pure `getXContextMenuItems(payload): MenuItem[]`, NO `invoke()` in the builder (IPC on click only).

---

## 3. Invariants that must not break
1. The two working menus (FileTree + library headers) stay byte-identical; `separator?`/`disabled?` are backward-compatible (omitted = render as today).
2. Every new string via `$t()` ×15 locales — INCLUDING retro-fixing Tabs + IndexPanel hardcoded English. Reuse existing keys (`linkConfidence.*`, `actions.*`, pin/unpin/copyPath/copyName) where present.
3. RTL: `dir`/`detectDir()` + logical CSS; preserve GraphMind's existing canvas-menu RTL flip if consolidated.
4. Performance Rules: NO `invoke()` on the right-click path (builders read in-memory only; IPC on click); virtualized lists stay virtualized; no per-row listener bloat (delegation on non-virtualized lists); no per-keystroke work.
5. Accessibility: Escape + click-outside on every menu; keyboard arrow-nav is a non-regression (absent today; if added, land it in the shared component).
6. No boot-time / typing-latency regression (render-layer only); 10-char type-burst smoke since `+layout.svelte` is touched.
7. Second-screen surfaces are displays — `LocalSkyView` right-click emits a callback, never owns a save/delete.
8. `EditorContextMenu` untouched; the in-editor wikilink menu (B6) is a NEW branch in the CM6 handler, not a change to the formatting menu.

---

## 4. Phased plan skeleton
> Each step = one landable commit + verification clause. **[GATE]** = Boss-testable pause.

### Phase A — Consolidate (one surface per commit)
- **A0** — add `separator?`/`disabled?` to `MenuItem` + render branches. *(internal; regression-check the 2 working menus)*
- **A1** — Tabs → `ContextMenu` (`getTabContextMenuItems`); new close-* keys ×15. **[GATE]**
- **A2** — IndexPanel → `ContextMenu`; `indexPanel.hideTerm/showTerm` ×15 (closes i18n violation); keep VirtualList attach. **[GATE]**
- **A3** — OrgChart → `ContextMenu`. **[GATE]**
- **A4** — extract shared `<ConfidencePicker>`; point Backlinks + Outgoing at it. **[GATE]**
- **A5** — *(optional)* GraphMind → `ContextMenu` (lowest priority; already i18n+RTL). **[GATE if done]**
- *End Phase A: `/simplify`; six menus → two primitives, all i18n-clean.*

### Phase B — Extend (priority order)
- **B1** — **List-mode rows (BOSS #1)**: `data-path` + delegated `oncontextmenu` on `NavFileList`; `getListRowMenuItems`. **[GATE]**
- **B2** — Search results (`.sh-item`, delegated). **[GATE]**
- **B3** — Tags rows (`.tp-tag`). **[GATE]**
- **B4** — Calendar day cells. **[GATE]**
- **B5** — Sky View bubbles (canvas hit-test, callback-only). **[GATE]**
- **B6** — In-editor wikilinks (new CM6 handler branch). **[GATE]**
- **B7** — lower-value (NoteGrid/Map/bookmarks/index-mentions), one commit each, only if prioritized.
- *After each phase: Standing Order (session log + help/manual); `/simplify`.*

---

## 5. Open questions for the Boss
1. **Option A (two primitives: `ContextMenu` + `ConfidencePicker`) vs Option B (everything on `ContextMenu`)?** Recommend A.
2. **List-mode menu (B1) item set?** Proposed: Open · Open in new tab · Rename · Move · Add tag · Delete. Add/remove (Reveal in tree, Copy path, Suggest sources)?
3. **Tags menu (B3):** read-only navigation (filter-by-tag) only, or also Rename / Remove-from-all (cascade writes + confirm)?
4. **Calendar (B4):** New-note-dated / Jump-to-day, or skip Calendar this pass?
5. **GraphMind consolidation (A5):** do it now (single-source) or leave it (lower risk; already i18n+RTL)?
6. **Lower-value surfaces (B7) priority?** Which of NoteGrid/Map/bookmarks/index-mentions matter, in what order? (Some may not warrant a menu per "Constraint as Design.")
7. **NavFileList virtualization** (pre-existing Perf-Rule-3 debt): spin off as its own task? (MIG-077 uses delegation so it doesn't block.)
8. **Keyboard arrow-nav** (absent today): add ↑/↓/Enter to the shared `ContextMenu` in this pass, or a later a11y MIG?

---

### Critical files
- `src/lib/components/ContextMenu.svelte` — `MenuItem` extension lands here.
- `src/routes/+layout.svelte` — canonical builder `:4497`, Tabs inline menu `:233`/`:5648`, render site `:7068`.
- `src/lib/components/navigator/{NavFileItem,NavFileList}.svelte` + `NotebookNavigator.svelte` — Boss #1 surface.
- `src/lib/components/BacklinksPanel.svelte` (+ identical `OutgoingLinksPanel.svelte`) — confidence picker to extract.
- `src/lib/i18n/en.json` (+ 14 locales) — all new strings ×15; retro-fix Tabs + IndexPanel.
