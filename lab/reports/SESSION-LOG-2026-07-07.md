# Session Log — 2026-07-07

**Function in hand:** the **note-lists right-click cluster (MIG-096)** — adopting one full-set right-click menu + a refresh-after-mutate broadcast across ~26 note-list surfaces, Reviewer first. PJ-069's biggest form-duplication cluster + the Boss's right-click ask, as ONE migration.

Continues from `SESSION-LOG-2026-07-06.md` (PJ-069 concept → MIG-094 orphan/fragile vocabulary shipped+tested → MIG-095 Health-tab enrichment shipped+tested → MIG-096 Architect + 4 Boss rulings + Plan approved).

Plan: `docs/MIG-096-NoteLists-RightClick-Plan.md`. Architect: `docs/MIG-096-NoteLists-RightClick-Architect.md`. Concept paper: `docs/concept-papers/PJ-069-Note-Lists-RightClick-Concept-Paper.md`.

Rulings locked (Boss, 2026-07-07): (1) exempt the 8 non-note-lists; (2) Five Acts + 360 matrix navigate-only; (3) confidence→hover button; (4) broadcast + uniform Move.

---

## §0 — Predecessor Lookup + exemption ledger (no code)

Per the Predecessor Lookup Rule (top principal) — written BEFORE the §1 edits. Every entry's **replacement lives in the same place** unless noted.

| # | Feature (predecessor) | Where it lives now | Where the replacement lives | Cut / kept |
|---|---|---|---|---|
| P1 | Row right-click affordance | `NoteRow.svelte` (fixed 52px shared row, MIG-090 §4) had NO `oncontextmenu` | **Same file** — optional `onContext` prop wired to the root `.nr` div's `oncontextmenu` | Kept: layout/selection/dir/a11y/height. Added: one optional prop. Sole current consumer (CollectionsPanel) passes nothing → browser default menu, unchanged. |
| P2 | The 3 near-duplicate inline note-menu bags | `+layout.svelte`: `handleBookmarkContextMenu` (safe subset, MIG-092), `handleSearchResultContextMenu` (safe subset, MIG-077 B2), `handleBaseRowContextMenu` (→ delegates to search) | **Same file** — new `buildNoteActions(path,name,ctx)` + `showNoteContextMenu(...)` closures consolidate them. §1 lands them with ZERO callers (dormant); the 3 copies are migrated to them in §2–§5. | Kept (untouched in §1): all 3 copies keep their exact current menus. Cut (later, per group): the inline bags, replaced by the shared builder. The file-tree `getContextMenuItems` is NOT consolidated — inline rename + folder/library kinds are tree-specific (standing exemption). |
| P3 | Refresh-after-mutate transport | Only `note-created` was emitted (store `createNote`, F2′). Rename/move/delete emitted NOTHING global — mutated open tabs in place, relied on the caller to imperatively refresh the file tree. | **New** `src/lib/noteMutations.ts` — `note-renamed`/`note-moved`/`note-deleted` emitted from the gated handlers; `onNoteMutation` subscribe helper. Emit sites: rename tail of `handleRenameComplete` (post-cascade), `handleMoveConfirm` (single+batch), `handleDeleteConfirm` (single+batch). | Kept: `note-created` + all existing imperative tree refreshes (belt-and-suspenders). Added: the 3 events + the subscribe helper. No existing wiring removed. |
| P4 | (ruling 3 — deferred to §4) Backlinks/Outgoing right-click ConfidencePicker | `BacklinksPanel`/`OutgoingPanel` `oncontextmenu` → ConfidencePicker (MIG-077 A4) | **Same panels** — relocate to a hover button so `oncontextmenu` becomes the note menu | Not touched in §1/§0. Logged here as the §4 Predecessor→Replacement per ruling 3. |

**Standing exemptions confirmed OUT (no note menu):** QuickSwitcher, BaseTab, FileTree (keeps its own richer tree menu). **8 ruling-1 exemptions OUT:** KnowledgeHealth + CCS (link pairs), Tasks + GlobalTasks + Calendar task-rows (task subjects), Cataloger + Forge pickers, Suggested Connections (concept-invariant). **Navigate-only (ruling 2):** Five Acts host-notes (`allowMutate:false`), Inspector360 matrix.

---

## §1 — dormant primitives (commit pending §1 audit)

**Landed (dormant — nothing adopts yet):**
- **NEW `src/lib/noteMutations.ts`** — `NoteRenamedEvent{oldPath,newPath,newName}` / `NoteMovedEvent{oldPath,newPath}` / `NoteDeletedEvent{path}`; `emitNoteRenamed/Moved/Deleted` (fire-and-forget `emit().catch()`); `onNoteMutation({onRenamed,onMoved,onDeleted,onAnyChange})` — granular callbacks fire immediately (cheap splice/re-title), `onAnyChange` coalesced 300 ms (re-run surfaces), returns an unlisten that clears the timer + all 3 listeners (Rule 4).
- **`NoteRow.svelte`** — optional `onContext` prop → root `.nr` div `oncontextmenu`. Behaviour-identical when unset.
- **`+layout.svelte`** — `buildNoteActions(path,name,ctx)` + `showNoteContextMenu(...)` (ZERO callers — dormant); THREE emit sites: rename tail (post-cascade, invariant 2 / BUG-023 — never from inside `renameItem`, carries `newName` so canonical title-only renames are detectable), move single+batch, delete single+batch (batch emits granular events once at the tail, only for successfully-mutated paths).

**Verify:** svelte-check **0 errors** (317 warnings = baseline). No Rust delta. Emits are fire-and-forget with no await/no reactive write → cannot affect the mutation path (dormant-safe by construction).

**Adversarial §1 audit** (workflow `wf_a692c3e6-f93`, 4 high-effort skeptics): cascade-ordering **SAFE**, dormancy **SAFE**, NoteRow-integration **SAFE**, module-correctness **RISK ×1 (LOW)**. Finding: `onNoteMutation` registered its 3 listeners in an array literal — if the 2nd/3rd `await listen()` rejected, the already-registered listener leaked (array literal aborts atomically) and the caller got no unlisten handle (Rule 4). **Fixed same pass (WA#6):** push each listener into the set as it resolves, `try/catch` → `cleanup()` unwinds the registered ones + clears the timer, then re-throw. svelte-check re-run **0 errors**. §1 committed after fix.

**Runtime note:** §1 is dormant (no menu appears anywhere yet) — the meaningful Editor-Surface-Gate round-trip test arrives at §2 (Reviewer), the first surface where the full menu goes live.

**Committed:** `6278d6e4` MIG-096 §0+§1.

---

## §2 — Group A (Reviewer + OrgChart done; Second-Screen forked)

**Reviewer (`ReviewerView.svelte`) — the headline surface, DONE:**
- `onContext?` prop; both master-row variants (virtualized >80 + plain ≤80) gain `oncontextmenu` → selects the row + forwards `(path,name,e)` to the host's `showNoteContextMenu` (full menu). Host wires it with `e.preventDefault()`.
- Refresh via `onNoteMutation` (leak-safe: destroy-before-resolve guarded): rename/move **re-title/re-path in place** (review membership is rename/move-invariant — cheap, no IPC, no loading flash), delete **splices** from every lens; `selectedKey` (which embeds `note_path` as `reason|path`) migrates alongside via `migrateSelectedKey`, mirroring the existing `act()`/`refreshAfterConnect()` re-point pattern.

**OrgChart (`OrgChart.svelte`) — the refresh template, DONE:**
- New `onNoteContext?` prop. `handleContextMenu`: a **note** node routes to the host's shared menu (gaining Star + Add-to-collection — the dedup win); **folder/library** nodes keep the internal create/expand menu. Wired on BOTH mounts (fullscreen overlay + embedded sidebar). Refresh already handled by the existing `markOrgChartDirty()` calls in every host mutation handler.
- **Flagged for §6 /simplify:** the internal `getOrgNodeMenuItems` note branch is now a graceful fallback (both live mounts route notes away from it) — dead for the live mounts, kept for degradation; the /simplify pass decides removal.

**Verify:** svelte-check **0 errors**. Committed pending the Second-Screen ruling.

**Second-Screen — fork RULED: "Full menu (mutations forward to main)" (Boss, 2026-07-07). DONE:**
- **Forward channel** — `secondScreen.ts::requestNoteActionOnMain(action,path,name)` emits `screen:request-note-action`; `+layout` listens (registered in `cleanupFns`) and dispatches via the existing `handleOrgNodeMenuAction` — so rename/move/delete open their dialogs on the MAIN window. Added a `bookmark` case to `handleOrgNodeMenuAction` (star, forwarded).
- **`SecondScreenPage.svelte`** — `showSSNoteMenu(path,name,e)` builds the menu via the SAME shared `buildContextMenu` (no bespoke copy): open/openInNewTab/reveal/star/addTag/rename/move/delete/suggest all `fwd()` to main; copy-path/copy-name act LOCALLY (pure clipboard read). Wired on all 4 `sc-link-item` sites (split + editor-panel backlinks/forward-links) + the embedded OrgChart (`onNoteContext`). `<ContextMenu>` rendered.
- **Refresh** — `onNoteMutation({onAnyChange})` re-runs the last panel scan (`loadSplitCompanionPanelData`/`loadEditorPanelsData`), leak-safe (destroy-before-resolve guarded). A stale 2nd-screen row is only ever a dead click (the 2nd screen never writes), so best-effort re-scan suffices.
- **Deferred to §3:** the 2nd-screen `DashboardView` menu (shared component — host-routed with Dashboard in §3).

**Verify:** svelte-check **0 errors** across all §2 files. §2 adversarial audit + release-binary build pending before the staged Boss test.
