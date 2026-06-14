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
