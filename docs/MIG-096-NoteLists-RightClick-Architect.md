# MIG-096 — Note-Lists Right-Click Cluster — Architect

**Date:** 2026-07-07 · **Status:** **ARCHITECT — awaiting the 6 Boss rulings (§8) before the Plan.** PJ-069's note-lists cluster (the biggest *form*-duplication cluster) + the Boss's right-click ask, ratified as ONE migration (2026-07-07). Built from workflow `wf_66f0f6b1-172` (4 agents: 2 surface inventories, 1 primitives/handlers/events map, 1 synthesizer), verified against source.

**Concept paper:** `docs/concept-papers/PJ-069-Note-Lists-RightClick-Concept-Paper.md` (horse ratified; Decisions 1–4 ruled: full menu, all 26 surfaces, one migration).

---

## 1. The horse

> **Adopt the shared `NoteRow` (carrying one full-set right-click menu) across the note-list surfaces so a note can be operated on — opened, revealed, starred, collected, copied, renamed, moved, deleted — from wherever it appears, while a single refresh-after-mutate contract guarantees no computed list ever shows a stale or dangling row — retiring the 26-way row-markup drift in one migration.**

## 2. The pleasant surprise — the full menu needs NO new menu code

Verified from source, not assumed:
- **`buildContextMenu(target, actions)` already emits the FULL Boss set** (Open · Open-in-new-tab · Move · Star · Add-to-collection · Add-tag · Copy · Reveal · Rename · Delete) when the matching callbacks are supplied. The MIG-077 "SAFE subset" is not a separate builder — it is just an actions bag that *omits* rename/move/delete. **So the full menu is entirely a WIRING job; nothing new in the menu builder or `ContextMenu.svelte`.**
- **`renameItem` / `moveItem` / `deleteWithSetting`** (store.ts) are the BUG-023 / Editor-Surface-Gate / MIG-076 §C content-integrity-safe wrappers — the **only** mutation route. The migration adds **no new write path** (WA#4 + Boss ruling).
- **`NoteRow`** today is fixed 52px, props `{name, meta, chips[], selected, missing, onActivate, trailing}`, per-title RTL correct, no `oncontextmenu`. Sole consumer: CollectionsPanel.

## 3. Three new primitives (land dormant, one commit)

1. **`NoteRow` gains an optional `onContext` prop** wired to `oncontextmenu` on its root — the ONE additive change to the shipped component. Rich surfaces (SearchHub highlights, Backlinks pills, Reviewer why-line, depth-indented trees) do **not** swap to the NoteRow *component*; they adopt only `onContext` + the menu + the refresh contract on their existing markup.
2. **`buildNoteActions(path, name, ctx)`** — a shared helper extracted from the 3 near-duplicate inline copies in `+layout`. Returns the full actions bag wiring open/reveal/star/collect/copy + the mutating trio (rename→`handleRenameComplete`, move→`openMoveDialog`, delete→`confirmDelete`, all over the gated wrappers). `ctx = {allowMutate, host: 'main'|'secondscreen', onRefresh}`: `host==='secondscreen'` forwards mutations to the main window (Display-not-Domain); `allowMutate===false` omits the trio → the menu **degrades to the safe subset automatically** (no separate builder).
3. **`onNoteMutation({onRenamed, onMoved, onDeleted})`** — a shared listen helper (onMount/onDestroy, 300 ms coalesced), the subscribe side of the refresh broadcast.

## 4. The central design — refresh-after-mutate

**The gap (verified):** only `note-created` is emitted today; rename/move/delete emit **nothing global** — they mutate open tabs in place and rely on the *caller* to imperatively refresh the file tree. The 26 lists are results of a computation that never re-run, so a mutation leaves them stale — the exact hazard the full menu creates.

**Mechanism (recommended):**
- Emit **three new events from inside the gated wrappers** — `note-renamed{old,new}`, `note-moved{old,new}`, `note-deleted{path}`. Tauri `emit()` reaches all windows, so second-screen companions hear them for free.
- **Cascade-safe ordering (the highest risk — BUG-023):** a rename triggers a universe-wide wikilink cascade. `note-renamed` fires **only after `handleRenameComplete` awaits both the rename AND the cascade settle** — never from inside `renameItem` — so no list re-runs its IPC against a half-rewritten universe. Move/delete have no cascade (fire on resolve). Batch loops (MIG-091 loops the handlers N×) emit **once at the tail** (mirror `refreshAllLoadedTrees`) — no event storm.
- **Per-surface pattern, chosen by cost + correctness:** **SPLICE** the row on delete/move-out (cheap, no IPC — the default); **RE-TITLE** in place on rename (patch name/path, no IPC); **RE-RUN** the surface's own IPC only where a rename can change *which* rows belong (tag/tension/reviewer/search membership), coalesced + guarded on panel-visible + never on the keystroke path. The common rename/delete case stays IPC-free even on 7,600 notes.

## 5. Adoption inventory (grouped by difficulty)

- **Group 1 — clean `NoteRow` component drop-in:** Sidebar Starred, Dashboard recents + tag-notes, Sidebar Five Acts *(ruling 2)*.
- **Group 2 — menu + refresh ONLY (rich rows keep their markup):** SearchHub results (*the* canonical non-refreshing surface — gains refresh first), Backlinks + Outgoing (ConfidencePicker relocation, *ruling 5*; Outgoing gates mutate on wikilink resolution), IndexPanel mention rows (the weakest refresh — `mentionsCache` drop+re-expand), Reviewer master queue (*first build target*; keeps `reason|path` selection identity + why-line + per-lens virtualization), Tension clickable rows, Inspector360 matrix (*ruling 3*).
- **Group 3 — depth-indented trees (menu-only, keep markup):** DigestPane, StructuralOutlinePanel, ProvenancePanel.
- **Group 4 — reference surfaces (already menu+refresh; unify to full set):** OrgChart (the refresh *template*; add Star + Add-to-collection), SecondScreenPage companions (forward mutations to main window).
- **Group 5 — RECOMMENDED EXEMPTIONS (Form-Aligns-To-Purpose — these aren't note-lists):** KnowledgeHealth + CCS (typed-**link** pair rows, no note identity; CCS carries the I2b traversal landmine), Tasks + GlobalTasks + Calendar-task-rows (task-line subjects, not notes), Cataloger + ExpressionForge pickers (rows *classify/add*, not open), RelatedCandidates (concept-invariant; name intentionally not open-clickable; embedded in 4 surfaces so its treatment cascades). SourceReviewPanel = menu-only on the card title.

## 6. Invariants

1. **The gated write path is the ONLY mutation route** (rename/move/delete via `renameItem`/`moveItem`/`deleteWithSetting`); no new write path; second-screen forwards to main (Display-not-Domain).
2. **Rename cascade stays safe** — `note-renamed` fires only after the cascade settles.
3. **Virtualize >50 rows; any height change updates `getItemHeight`/`getRowHeight` in LOCKSTEP** (Backlinks/Outgoing/Reviewer/IndexPanel/Digest use variable/synthetic heights — the exact trap NoteRow defused only for fixed-height); unvirtualized big lists (GlobalTasks, Inspector360 800+) get `VirtualList` in the same pass.
4. **Per-title RTL preserved** (NoteRow `detectDir`, ContextMenu `$isRTL`) — inherited, not re-implemented.
5. **Zero new per-keystroke IPC** — listeners 300 ms-coalesced, re-run only while mounted+visible; common case is splice/re-title (no IPC).
6. **Data + actions stay host-owned** — leaves forward `(path,name,e)` up; the menu is built once via `buildNoteActions` in the host; gated handlers live in one place.
7. **CCS I2b** — opening a row must NEVER fire `constellation_link_traverse`.
8. **No content-integrity regression** — every mutation splices the exact row or re-titles from `{old→new}`; never a dangling/stale row (Editor-Surface-Gate).

## 7. Migration path

- **Phase 0 — Predecessor Lookup + exemption ledger (no code):** Predecessor→Replacement entries (NoteRow `onContext`; the 3 safe-subset menu sites overridden to full; the ConfidencePicker relocation; IndexPanel term-menu coexistence). Record the §8 rulings. *No edit until this lands.*
- **Phase 1 — Primitive lands DORMANT:** `NoteRow.onContext` + the 3 gated-wrapper emits (cascade-safe) + `buildNoteActions` (extracted, behavior-identical) + `onNoteMutation`. Nothing adopts yet. **Verify:** existing menus unchanged; typing/boot unchanged; svelte-check + Rust suite green **AND** a manual rename/move/delete round-trip through the Editor-Surface-Gate checklist.
- **Phase 2 — Group A (Reviewer first, per your headline) + OrgChart reference + Second-Screen.** Staged Boss test.
- **Phase 3 — Group 1 clean drop-ins.** Staged Boss test.
- **Phase 4 — Group 2 rich surfaces (menu+refresh).** Staged Boss test.
- **Phase 5 — Group 3 trees + SourceReviewPanel.** Staged Boss test.
- **Phase 6 — /simplify + the /migration audit trio + full PCS** (orientation v-bump, MoCh, help/manual ×15 — the right-click is user-facing).

## 8. The Boss rulings needed (before the Plan)

1. **Exemptions (§5 Group 5):** confirm OUT — KnowledgeHealth + CCS (link pairs), Tasks + GlobalTasks + Calendar-task-rows (task subjects), Cataloger + ExpressionForge pickers, RelatedCandidates (concept-invariant) — or does a restricted/link menu belong on any?
2. **System notes (Five Acts host-notes):** FULL menu (rename/move/delete could break the `{universe}/Five Acts/` convention + the embedded base-lens block), or restricted (open/reveal/copy)? *(Recommend restricted.)*
3. **Cognitive diagrams (Inspector360 matrix):** do the matrix dots/cells carry mutating actions (Rename/Move/Delete), or navigate-only? *(Form-Aligns-To-Purpose — recommend navigate-only.)*
4. **Move UX:** uniform `openMoveDialog` on every eligible surface (the one gated move path), or omit Move where a destination picker is awkward (popovers)? *(Recommend uniform.)*
5. **Predecessor relocation:** approve moving Backlinks/Outgoing's right-click ConfidencePicker (MIG-077 A4) to a hover button so the note menu owns `oncontextmenu`; and the IndexPanel note-menu coexisting with the term menu (Hide/Show)?
6. **Refresh transport:** ratify the broadcast (`note-renamed/moved/deleted` from the gated wrappers) over per-surface callbacks?

## 9. Open risks

Cascade-ordering race (highest — emit from the handler tail, not the wrapper body) · virtualizer keep-in-sync (variable/synthetic heights) · IPC re-run cost on universe-scale lists (splice/re-title the common cases) · second-screen double-write (forward, never call locally) · IndexPanel lazy-cache (new invalidation logic, easiest to under-build) · Form-Aligns-To-Purpose overreach (diagram surfaces) · batch-op event storm (emit once at the tail) · one-pass scope (Phase 1 must be proven dormant-safe before any adoption).

---

**This session's deliverable stops at this Architect + the §8 rulings.** Plan follows ratification.
