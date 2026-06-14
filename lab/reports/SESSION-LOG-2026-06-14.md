# Session Log — 2026-06-14

## §MIG-077 — App-Wide Right-Click Context Menus — Phase A continuation (A3 →)

**Function in hand: MIG-077 Phase A** — consolidating the 6 hand-rolled context menus onto
the shared `ContextMenu` (+ a separate `<ConfidencePicker>` for the one radio-group menu),
per the Boss-approved Option A. Resumed at **A3 (OrgChart)** per `HANDOVER-2026-06-14.md`.

**Session ritual:** `git pull origin main` (already up to date at `ed15d3ad`); orientation
**v2.79 read** (v2.79 preamble + §3 architecture); `HANDOVER-2026-06-14.md` read in full;
`MIG-077-RIGHTCLICK-CONTEXT-MENUS-PLAN.md` read in full. A0/A1/A2 are shipped + Boss-validated
(commits `eb3a246f`, `381a471e`, `a999c165`, `8aab9db2`). Plan approval = build approval —
cascading A3 → A4 → A5 → Phase B → Phase 4, stopping only at the [GATE] Boss tests.

### A3 — OrgChart node menu → shared `ContextMenu` — SHIPPED (awaiting Boss gate)

**Commit:** `<pending>` · svelte-check **0 errors** · `npm run build` + `cargo build --release`
(1m59s) green · bundle-confirmed.

- Replaced the inline `.oc-fs-ctx` menu (`OrgChart.svelte`) with `getOrgNodeMenuItems(node): MenuItem[]`
  rendered via the shared `<ContextMenu>` — the proven A1/A2 idiom. Open for a note; an
  Expand/Collapse toggle for a container.
- **Latent bug fixed.** The old render was `$t('contextMenu.open') || 'Open'`, but the bare
  `contextMenu.open` key **never existed** — and Constellation's custom `t()` returns the literal
  key string on a miss (active-locale → en → key), which is truthy, so the `|| 'Open'` fallback
  was dead and the menu button rendered the literal text **"contextMenu.open"**. A3 adds the
  proper key, so the label now reads correctly in every locale.
- **i18n ×15:** added `contextMenu.{open,expand,collapse}` natively to all 15 locales (derived from
  existing app vocabulary — `open` from `contextMenu.openLink`'s verb; `expand`/`collapse` from
  `sidebar.expandAll`/`collapseAll` minus the "all" quantifier — not invented). The canonical home
  for these (reused later by B1/B2/B5/B6). Replaces the previous reuse of `sidebar.expandAll`
  ("Expand all") which was wrong wording for a single-node toggle.
- **Dead code removed:** the now-unused `handleCtxAction` function + the `.oc-fs-ctx` CSS block.
- **Bundle proof:** `build/` contains `open:"Open",expand:"Expand",collapse:"Collapse"` in BOTH
  the main app chunk (`_app/immutable/chunks/D7b8oqZV.js`) and the second-screen bundle
  (`assets/screen-CMUc73KQ.js`) — fresh frontend embedded. Binary mtime 14:31 == build time.

### BOSS STEER (after the A3 thin gate): menus must be RICH + CONTEXTUAL

At the A3 gate Eisa rejected the faithful 1-item consolidation: *"What is the use of a right-click
with only one command? I want the full list, like: Delete, Rename, Move, etc."* (notes) and *"Same
thing!"* (containers) — MIG-077's origin observation #3. AskUserQuestion settled three decisions:
**(1)** note menu = FULL, build everything now; **(2)** container menu = RICH; **(3)** NOT
identical-everywhere but **contextual** — *"it should be contextual and adapt to each type of
function."* Plan updated with the ADDENDUM; the thin A3 (`2e95b04a`) stays in history, its wiring +
keys carry forward. Re-scoped: A3-R1 shared builder → A3-R2 OrgChart ready set+rename → A3-R3 Move
→ A3-R4 Add tag (each its own gate, staged).

### A3-R1/R2 — contextual rich menu + OrgChart — SHIPPED (awaiting Boss gate)

**Commit:** `<pending>` · svelte-check **0 errors / 315 warnings** (baseline) · `npm run build` +
`cargo build --release` (2m18s) green · bundle-confirmed · binary mtime 17:12.

- **NEW `src/lib/components/contextMenuBuilder.ts`** — `buildContextMenu(target, actions)`: ONE
  shared source, contextual output by **(object kind × surface capability)**. An item appears only
  when its callback is provided AND fits the kind; group-based separators stay clean regardless of
  which items are present. This IS the "contextual" mechanism (each surface passes the callbacks it
  can fulfil). Reused later by the file tree, List-mode, Search, Sky View.
- **NEW `src/lib/components/RenameDialog.svelte`** — a small reusable rename dialog (RTL-aware via
  `detectDir`; reuses `actions.rename`/`dialogs.cancel` — no new strings). Full-page surfaces have
  no inline tree row, so they rename through this → the host's existing `handleRenameComplete`
  (rename + wikilink cascade + collision dialog all reused).
- **OrgChart** now builds its menu via `buildContextMenu` and emits a single `onNodeMenuAction(action,
  target)` to `+layout`'s new `handleOrgNodeMenuAction` (every op reuses an existing handler —
  openNoteTab / clipboard / handleSuggestSourcesForNote / confirmDelete / handleCreate* /
  handleRenameComplete; Expand/Collapse stays OrgChart-local). `libIdForPath` = longest-prefix lib
  lookup (correct with nested libraries).
- **R2 menu:** notes → Open · Open in new tab · Rename · Copy path · Copy name · Suggest sources ·
  Delete. Folders → New Note/Folder/Base · Expand/Collapse · Rename · Delete. Libraries → New
  Note/Folder/Base · Expand/Collapse. (**Move + Add tag** = A3-R3/R4.)
- **Reveal in tree OMITTED — it is dead app-wide.** Repo-wide grep: `constellation:reveal-in-tree`
  is dispatched (editor breadcrumb, `+layout:6333`) but **has no listener anywhere**. Shipping it in
  the menu would be a dead item — against Eisa's "useful menus." Flagged as a separate fix
  (`spawn_task` task_bd6d4802); will be added to the menus once the listener exists.
- **i18n ×15:** `contextMenu.{openInNewTab,move,addTag}` added natively (move = `tableToolbar.move`
  verbatim; addTag = `addProperty` grammar with the native "tag" noun; openInNewTab = standard
  localized phrasing). Bundle-confirmed (`openInNewTab:"Open in new tab"`, `addTag:"إضافة وسم"`).
- **Known minor (logged):** deleting/creating from the full-page chart doesn't auto-refresh the
  chart (the node lingers until reopened) — cosmetic; the file op itself is correct. Follow-up.

### task_bd6d4802 — "Reveal in file tree" listener (was dead app-wide) — SHIPPED

Interleaved on Boss's action of the chip I spawned during A3-R2 (the menu omitted Reveal-in-tree
because it was dead). Root cause: `constellation:reveal-in-tree` was **dispatched** (editor breadcrumb,
NotePane → `+layout:6333`) but **had no listener anywhere in src**. Fix:

- **`+layout` `revealInTree(path)` + listener** (`onMount`, cleaned up on destroy): switch to tree
  mode → longest-prefix library match over `$libraryStats` → expand the child universe (if nested)
  and the library (`toggleLibrary` lazy-loads the tree) → after `tick()` + 2 frames, find the row by
  `[data-tree-path]`, DOM-open every ancestor `<details>` (persists — Svelte only applies `open` at
  creation), `scrollIntoView({block:'center'})`, brief outline flash (1.6 s).
- **`FileTree`**: `data-tree-path={entry.path}` added to the note button + folder summary (rows were
  previously un-locatable by path).
- **Now functional everywhere:** the editor breadcrumb (⋯) "Reveal in file tree" works, AND
  "Reveal in tree" is added back to the OrgChart contextual menu (notes + folders), no longer omitted.
- svelte-check 0 errors / 315 warnings; build green; binary 17:28; `data-tree-path` + `reveal-in-tree`
  bundle-confirmed.
