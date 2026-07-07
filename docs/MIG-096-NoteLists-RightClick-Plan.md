# MIG-096 — Note-Lists Right-Click Cluster — Plan

**Date:** 2026-07-07 · **Status:** **PLAN — awaiting Boss approval (Plan approval = build approval).** Follows the ratified Architect (`docs/MIG-096-NoteLists-RightClick-Architect.md`). PJ-069's note-lists cluster + the right-click, as ONE migration.

## Rulings locked (Boss, 2026-07-07)

1. **Exempt the 8 non-note-lists** — Knowledge Health + CCS (link pairs), Tasks + Global Tasks + Calendar task-rows (task subjects), Cataloger + Forge pickers, Suggested Connections (concept-invariant). No note menu on those. *(Plus the standing exemptions: QuickSwitcher, BaseTab, FileTree.)*
2. **Restricted / navigate-only** on the Five Acts host-notes (Open/Reveal/Copy — no rename/move/delete, to protect the `{universe}/Five Acts/` convention + the embedded base lens) and the 360° matrix (dots navigate only — no mutation from a cognitive diagram).
3. **Confidence → hover button** — Backlinks/Outgoing move the ConfidencePicker off `oncontextmenu` (which becomes the note menu) to a hover button; the IndexPanel note menu coexists with the term Hide/Show menu. Logged as Predecessor→Replacement.
4. **Broadcast + uniform Move** — one `note-renamed/moved/deleted` event from the gated wrappers, every list subscribes; Move opens the one shared `openMoveDialog` on every eligible surface.

The **full menu** (Open · Open-in-new-tab · Reveal · Star · Add-to-collection · Copy · **Rename · Move · Delete**) is the default on genuine note-lists; it **degrades to the safe subset automatically** where `allowMutate=false` (Five Acts, unresolved wikilink targets). The full menu needs **no new menu code** — `buildContextMenu` already emits it.

---

## Steps (each a landable commit-group + verification clause)

**§0 — Predecessor Lookup + exemption ledger (no code).** Write the Predecessor→Replacement entries into the session log: `NoteRow` gains `onContext`; the 3 safe-subset menu sites overridden to full (retire the "non-refreshing surface" comments); the Backlinks/Outgoing ConfidencePicker relocation; the IndexPanel term-menu coexistence. Record the 4 rulings. *Verify:* the ledger lands before any edit (Predecessor Lookup Rule).

**§1 — The primitive lands DORMANT (one commit).** (a) `NoteRow` gains optional `onContext` (root `oncontextmenu`); (b) the 3 gated-wrapper emits — `note-renamed{old,new}` fired **after** `handleRenameComplete` awaits the rename AND the wikilink cascade settle, `note-moved`/`note-deleted` on handler resolve, batch loops emitting **once at the tail**; (c) `buildNoteActions(path,name,{allowMutate,host,onRefresh})` extracted from the 3 inline copies (existing sites call it — behavior-identical); (d) `onNoteMutation({onRenamed,onMoved,onDeleted})` listen helper (onMount/onDestroy, 300 ms coalesced). Nothing adopts yet. *Verify:* existing menus (FileTree, Base rows, Bookmarks) unchanged; typing/boot unchanged on 7,600 notes; svelte-check + Rust suite green **AND a manual rename/move/delete round-trip through the Editor-Surface-Gate checklist** (the events fire, no editor-surface corruption). **This is the load-bearing gate — no adoption ships until §1 is proven dormant-safe.**

**§2 — Build Group A: Reviewer (your headline) + OrgChart (reference) + Second-Screen.** Reviewer adopts `onContext` + full menu (keeping `reason|path` selection identity, the why-line, per-lens virtualization) + refresh (splice-delete / re-title-rename / re-run-move, visible-guarded). OrgChart unifies its `getOrgNodeMenuItems` with `buildNoteActions` (adds Star + Add-to-collection). Second-Screen companions forward mutations to the main window. **Boss test (tutorial):** in the Reviewer, right-click a queued note → Rename → the row re-titles (not dangling) and a linked probe updates (BUG-023 pair); Delete → the note vanishes from *all* lenses it appeared in; the detail-pane Reviewed/Snooze still work. Rename on OrgChart (expand/pan/zoom preserved). Second-screen rename → main window writes, companion re-renders.

**§3 — Build Group B: clean `NoteRow` drop-ins.** Sidebar Starred, Dashboard recents + tag-notes, Five Acts (restricted, per ruling 2). Handle Starred path/name patch on rename + prune on delete; Dashboard localStorage prune; dual-host routing. **Boss test:** rename a starred note → the ⭐ shows the new name; delete → the ⭐ disappears; a Dashboard recent doesn't reappear after the 5 s poll; Five Acts shows only Open/Reveal/Copy.

**§4 — Build Group C: rich surfaces (menu + refresh only).** SearchHub (retire the safe-subset comment; splice/re-title/re-run), Backlinks + Outgoing (ConfidencePicker → hover button; Outgoing gates mutation on wikilink resolution), IndexPanel mention rows (add `mentionsCache` drop + re-expand — the weakest-refresh fix), Tension clickable rows, Inspector360 (navigate-only, ruling 2). **Boss test:** in SearchHub, rename a result → re-titles, no dangling; delete → splices out; keyboard-nav + category collapse still work. In Backlinks, confidence still reachable (hover button) AND the note menu works. In IndexPanel, rename a mentioned note → the cached mention updates (the old stale-cache bug).

**§5 — Build Group D: trees + SourceReviewPanel (menu-only, keep markup).** DigestPane / StructuralOutlinePanel / ProvenancePanel adopt `onContext` + menu on their name elements + refresh (refreshTick / parent re-pass); StructuralOutline's mutating actions route through the gated handlers (never its `resolveStructuralConflict` path). SourceReviewPanel card-title menu (two-mode host). **Boss test:** rename a note that appears in the Structure outline and the Provenance chain → both re-render, indent/lineage intact; delete → splices without breaking the tree.

**§6 — /simplify + the /migration Audit trio + full PCS.** `/simplify` the whole diff. Audit: invariants (gated-path-only, cascade ordering, CCS I2b, virtualizer lockstep), drift (the new listeners — LL-023), migration path (first-boot, mid-cascade-interrupt rename, batch N-loop, second-screen cross-window, rollback). Full PCS incl. **help files + User Manual ×15** (the right-click is user-facing), Orientation v-bump (same commit), MoCh, handover.

## Boss-test checkpoints (staged, one at a time)

§2, §3, §4, §5 each pause for a tutorial-framed test. §0, §1, §6 are internal/verified (§1 also gets a manual Editor-Surface-Gate round-trip). Per the staged-tests standing order, one test at a time.

## Scope fences

Does NOT: add any new write path (mutations route through `renameItem`/`moveItem`/`deleteWithSetting` only) · put a note menu on the 8 exempt surfaces or the standing exemptions · put mutating actions on Five Acts / the 360 matrix · re-run IPC on the keystroke path or while a panel is hidden · change any surface's data source or the confidence/link semantics.

---

**On approval I cascade §0→§6**, pausing at the four staged Boss tests (§2–§5) and holding §2+ until §1 is proven dormant-safe. The old markup keeps running until each group's validated swap.
