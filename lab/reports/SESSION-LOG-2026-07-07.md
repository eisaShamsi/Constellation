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

**Verify:** svelte-check **0 errors** across all §2 files.

**§2 adversarial audit** (workflow `wf_d23bf978-ecd`, 4 high-effort skeptics): reviewer-refresh **SAFE**, write-path/invariants **SAFE**, OrgChart **RISK ×2**, Second-Screen **RISK ×1**. All three fixed same pass (WA#6):
- **[MED] OrgChart fullscreen right-click → Open opened the note HIDDEN behind the chart** — the shared menu's `open` used `handleNoteClick`, which (unlike the old `handleOrgNodeMenuAction('open')`) never set `showOrgChart=false`. The Reviewer had the identical latent bug (`{#if showReviewer}`). **Fix:** added `NoteActionCtx.onOpen` — a full-page-overlay surface supplies its own open that dismisses the overlay; wired `openNoteFromReviewer` / `openNoteFromOrgChart` on the two `onContext` props.
- **[MED] Second-screen "Open" forwarded to main set `orgChartReturnPending=true`** unconditionally → spurious "Return to OrgChart" button for a chart never open. **Fix:** `orgChartReturnPending = showOrgChart` (only if it was actually open).
- **[LOW] Embedded (sidebar) OrgChart's `onNoteContext` was inert** — the sidebar tree-node span had no `oncontextmenu`. **Fix:** added `oncontextmenu` firing `onNoteContext` for note nodes.

svelte-check re-run **0 errors**. Release-binary build pending before the staged Boss test.

---

## MIG-097 — index rename-drift self-heal (interrupt during §2 Boss test)

**Trigger:** §2 Stage-1 Part B — Boss renamed a note in the Reviewer; on reopen the row reverted to the OLD name and opening it hit an empty Dashboard, while the file tree showed the new name.

**Diagnosis (Reproduce-First — confirmed against the live DB + disk):** `get_due_notes` reads `note_meta.name`/`.path`. For this one note, `note_meta` pointed at `…التجربة الثانية_إعادة تسمية.md` (**gone from disk**) with the old name, while disk had `التجربة الثانية ن2.md` (correct title). A rename writes the file immediately (gated) but updates the index in a **detached best-effort tail** (`rename_item_db_tail`, §B2-4 — detached to avoid a freeze on large libraries); on the Boss's 2 GB / 7,713-note library that tail was lost, and gated renames suppress the watcher, so nothing healed it. **Severity: 1 of 7,713 rows drifted** — not systemic; the §2 feature wrote disk correctly. **Boss ruling: fix now (safe self-heal), then resume §2.**

**Fix (`src-tauri/src/reconcile.rs`):** enhanced the existing MIG-078 boot reconcile (which only *removed* dead rows) to first **RELOCATE** a dead-path row to the note's current file, matched by the stable **`cid_cn`** (the orphan half of a lost-tail rename), preserving the row's aux data (review history, links, aliases, embeddings) via `relocate_row` (transactional, never overwrites an existing row); only rows whose note is genuinely gone fall back to removal. `collect_orphans` walks the library only when drift exists and reads frontmatter only for orphan files. Same WA#4 safety (accessible-roots-only, 10 %/200-row abort cap, lock-free stat/walk). Runs on every universe-open (`ensure_search_db_ready` → `reconcile::maybe_schedule`), so the Boss's next launch heals the drifted note.

**Verify:** 3 Rust tests green — `relocate_row_migrates_note_and_aux_by_path` (cid preserved, all aux migrated), `relocate_row_refuses_occupied_target` (no row lost), `collect_orphans_maps_unknown_md_by_cid` (skips already-indexed). Full lib compiles clean. Only `reconcile.rs` touched (private `run()` signature; `maybe_schedule` unchanged) → no other-module risk.

**Known limitation (logged, not a blocker):** drift heals on next universe-open, so a lost-tail rename shows stale in note_meta readers until the next launch (rare event; Boss chose the boot self-heal over hardening the async rename path).

---

## MIG-098 — rename → index sync reliability (MIG-097 proved INSUFFICIENT)

**Correction (Reproduce-First, from the Boss's live `diagnostics.log`):** my "rare, 1/7713" framing of the drift was WRONG. The log shows the MIG-078 boot reconcile removing 1–2 stale rows on ~every launch for ~9 days — renames drift **routinely** on the 2 GB / 7,713-note universe, and the reconcile has been **silently deleting** the drifted notes from the index each boot. Correlates with the **2026-07-03 §B2-4** change that detached `rename_item_db_tail` to a best-effort `spawn_blocking` (relying on "the watcher heals misses" — but gated renames suppress the watcher). Re-test failures explained: `التجربة الثانية ن2` — dead row already removed by an earlier boot → orphan file, no index row (missing); `§D test 1 v2` — renamed mid-session, note_meta still at dead `§D test 1.md`. **MIG-097 is insufficient:** boot-only (misses mid-session) + can't re-adopt already-deleted rows. **Boss ruled: proceed with a proper `/migration`.**

**Step 1 — instrumentation SHIPPED (Reproduce-First; symptom reproduced, mechanism not yet pinned).** `libraries.rs`: `diag_log` traces (release-safe → `diagnostics.log`) at the spawn point (`[rename-tail] scheduling …`) and through `rename_item_db_tail` (`START`, `note_meta path UPDATE affected N row(s)`/ERROR, canonical branch, reindex OK/ERROR/`NO LIBRARY matched … SKIPPED`, `END`). Decision tree in `docs/MIG-098-Rename-Index-Sync-Architect.md` §Step-1: `scheduling` w/o `START` → tail starved/dropped; `START` + `0 row(s)` → path mismatch; `START` + `1` but still stale → later revert; `NO LIBRARY` → prefix mismatch. `cargo check` clean.

**Step 1 TRACE (Boss renamed one note, 2026-07-07):** the tail **worked** — `scheduling` → `START` → `note_meta path UPDATE affected 1 row(s)` → `reindex OK (lib Eisa Test)` → `END`, all same second. So the tail is **NOT universally broken; the drift is intermittent** (the §B2-4 note: the tail *parks* on the writer lock under contention; lost if the app closes before a parked tail completes — fits the Boss's frequent-restart testing). Live-DB check: `§2 Renamed trace` correctly indexed; **`§D test 1 v2` (cid 6C47) + `التجربة الثانية ن2` (cid 8878) are now ORPHANS** — files on disk, **zero note_meta rows** (their dead rows were removed by earlier reconciles); **0 dead paths**. This exposed MIG-097's gap: it only acts on a *present dead row* — these have none, so it skipped them → they need **re-adopt**, not relocate.

**Step 3 (Part A) — complete the self-heal SHIPPED (pending audit).** `reconcile.rs` rewritten to reconcile note_meta↔disk in BOTH directions every universe-open: dead rows (per-path stat, safe) → relocate-by-cid (preserves aux) or remove; **orphan files** (`.md` on disk not in note_meta, via `collect_md`) → relocate a surviving dead row, OR **re-adopt** (index fresh — recovers §D test 1 v2 + التجربة الثانية ن2 + any note a prior reconcile deleted). WA#4 safety: accessible-roots-only, cap on removal AND re-adopt (a large orphan set = mid-index race → skip), lock-free stat/walk, per-path stat (a walk error can't mass-mark-dead). 4 Rust tests green (relocate migrates+preserves cid, refuses occupied, collect_md finds orphans+complete, lib_for nested). `cargo check` clean.

**WA#4 audit** (workflow `wf_37378780-144`, 4 skeptics): re-adopt SAFE, heal-correctness SAFE, perf SAFE, data-loss **RISK ×3 (1 HIGH, 2 MED)** + 2 LOW perf. All 6 fixed same pass (WA#6) — the common root was *removal destroying review-history/link aux (not on disk) on a FALSE "gone" signal*:
- **[HIGH]** a renamed note whose orphan the walk MISSED (read_dir error / depth>20) fell to REMOVE not relocate → permanent aux loss + permanent index-invisibility. **Fix:** `collect_md` now reports `walk_complete` (false on any read_dir error / depth cutoff); removals are SKIPPED entirely when the walk was incomplete (phantoms left for a clean pass — aux never destroyed on incomplete evidence).
- **[MED]** `Path::exists()` swallows transient stat errors as "gone" → live note removed. **Fix:** a fresh re-stat right before each removal (`if Path::new(p).exists() continue`).
- **[MED/LOW]** a `relocate_row` failure (occupied target / contention) fell to REMOVE, destroying the aux relocate exists to preserve. **Fix:** a failed relocate is now LEFT for next boot (logged), never removed.
- **[LOW perf]** nested/overlapping roots walked twice + orphans double-counted. **Fix:** walk only top-level roots (skip a root nested under another; `lib_for` still attributes via ALL roots) + a `seen` dedup set + `lib_for` longest-root match (correct library attribution).

**Step 2 (root cause) — DEFERRED to a failing trace.** The tail works uncontended; a *failing* trace (rename under contention / rename-then-quick-quit) is needed before designing the root fix (Reproduce-First — don't fix an unreproduced mechanism). Instrumentation stays in to catch it. Part A makes drift self-heal on boot regardless; Step 2 will reduce/eliminate the drift + the mid-session window.

**Step 2 FAILING TRACE captured (2026-07-07, Boss provoke-the-drift: rename within 5 s of launch + quick close).** `Apple Tree Fruit`→`Apple Tree Fruit drift`: `scheduling` → `START` → **_[no "note_meta path UPDATE" line]_** → `reindex OK` → `END`. Live-DB confirm: note_meta has ONLY the dead old-path row; NO new-path row; disk has the renamed file. **Root cause (empirically confirmed): the detached best-effort tail ran while the search DB connection was `None` (still initializing — the Boss renamed during boot), so BOTH the path UPDATE (guarded by `if let Some(conn)`) AND the reindex silently no-op'd. `reindex_single_note` returns `Ok(())` on a `None` connection — a false-success that logged "reindex OK" while doing nothing.** General mechanism: the rename's note_meta update is a fire-and-forget task with NO durability + NO retry that silently no-ops on conn-`None` / contention / app-close. This is the trigger for the full safety audit.

---

## PIVOT — Constellation Safety & Integrity Audit (Eisa directive, 2026-07-07)

**Directive:** *"Stop everything and put the app under inspection to find and fix those app-killing bugs… declare the app safe and secure."* Feature work HALTS; a systematic hunt for the whole **silent-app-killer** class begins. Charter: `docs/Constellation-Safety-Audit-CHARTER.md`.

### State-of-standing (SO #5) — before the pivot

- **(a) Verified-shipped + protected (committed on `main`):** MIG-096 §0/§1 (dormant right-click primitives, audited) · §2 Reviewer + OrgChart + Second-Screen (audited, Boss-tested PASS for Reviewer) — commits `6278d6e4`, `c02b3fbd`, `7deb0d82`, `a8c413fa`. MIG-097 relocate self-heal (`eab166d2`). MIG-098 §1 instrumentation (`22f79bf1`) + §3-A reconcile re-adopt, WA#4-audited (`501537fe`). Tree clean.
- **(b) In-flight / paused:** MIG-096 §2 Boss tests for **OrgChart + Second-Screen NOT done** (paused mid-test). MIG-096 §3–§6 (Groups B/C/D + audit + PCS) **not started** (Plan approved, queued). MIG-098 §2 rename-durability **root fix not built** (root cause now confirmed; investigation workflow `wf_870c2b89` synthesizing prior-art fix). Rename-tail **instrumentation still in the binary** (remove when the fix lands).
- **(c) Known-broken:** the rename→index durability bug (confirmed above) — MITIGATED by the boot reconcile self-heal (recovers drift every launch; 12 notes recovered 2026-07-07) but the source-of-truth write is still non-durable mid-session. This is remediation item #1 of the audit.
- **(d) Pending / not-started:** the audit itself (P1→P5); MIG-096 remaining groups; MIG-098 §2 fix.
- **(e) Doc drift:** Orientation NOT yet bumped for MIG-096/097/098 (mid-flight; bump when the audit closes or they land). Pending-Jobs not updated for the pivot.

Resume path after the audit: MIG-098 §2 fix (item #1) → remaining audit remediation → MIG-096 §2 tests (OrgChart/Second-Screen) → §3–§6.

### Audit FIND phase COMPLETE (P1–P2, Waves 1–3) — 30 confirmed silent-failure defects

Three adversarial-verify workflows (`wf_c4054ac3` durability/false-success · `wf_4c7d9c3a` content-integrity/index-divergence · `wf_858afde9` concurrency/freeze/leaks) — 75 agents total, every candidate refuted-or-confirmed. **Reactivity-loops + concurrency-TOCTOU came back CLEAN.** Full register in `docs/Constellation-Safety-Audit-CHARTER.md`. 30 defects consolidate to **9 root-cause fix-groups**: G1 FocusPane save (HIGH) · G2 save/model-ownership (HIGH — nav discards unsaved edits, 2-tabs-1-note clobber) · G3 cross-window integrity (HIGH) · **G4 frontmatter parser lossy (HIGH — drops block-scalars/nested-maps + corrupts quotes on EVERY save)** · G5 index cascade + MIG-098 durability (MED) · G6 persisted-state writes (MED — non-atomic libraries.json) · G7 write-gate staleness inert (MED) · G8 sync-command freeze (HIGH/MED) · G9 listener leak (MED). Remediation order (worst-silent-loss first): G4 → G2 → G1 → G3 → G5 → G6 → G7 → G8 → G9; deep groups each rate a `/migration`. **P4 remediation begins next.**

### Remediation landed (P4) + Create/Rename latency DIAGNOSED

**Quick-wins batch SHIPPED** (`90ba3829` + `d153dd95`): G8 (async freezes) · G9 (leak guard) · G6 (atomic-write class: `libraries.json` + `universe::atomic_write` on registry/settings/workspaces/collections/property-types; swallowed writes surfaced) · G1 (FocusPane debounced save + write-ahead net + `onflush`). **The standing per-build inspection's first live run (`wf_012f1593`, 49 agents) caught a regression I introduced (G1 reindexing per keystroke) BEFORE commit** — fixed to a proper debounce — and surfaced 3 new findings (registered). Deep `/migration` fixes (G4→G2→G3→G5→G7) remain.

**Create/Rename ~10 s latency — Boss-reported, DIAGNOSED** (`wf_8fa06cde`, 3 converging traces). **Main reason: full-universe WHOLE-FILE content reads on the awaited path** — to show a note's title the code `fs::read_to_string`s the ENTIRE `.md` across all ~7,700 notes / 2 GB, cold. ONE class, 3 sites: (1) `read_library_tree`→`read_dir_recursive` reads every note's whole file for its title (`libraries.rs:2161`; comment claims "first 1 KB" but reads the whole file) — BOTH paths (create runs it TWICE); (2) `resolve_wikilink_cross_library`→`find_note_by_title_or_alias` (`libraries.rs:1893`) reads every `.md` (canonical filenames defeat the stat-only pass) — BOTH; (3) `update_links_on_rename`→`update_links_recursive` (`libraries.rs:5083`) reads every `.md` to regex `[[oldName]]` — RENAME only. Ruled OUT (earlier fixes held): the detached DB tail, `cache_boot_snapshot_graph` (sub-second since MIG-079), `ensureFullLinks` (no-op), stats (index read). **Fix (Rule 8): serve titles + collision/alias check from `note_meta`; bound any read to ~1 KB; scope the cascade to actual linkers via `note_links`; coalesce the create double-refresh** → sub-second. **Boss chose confirm-first:** `diag_log` timing probes on `read_library_tree` + `update_links_on_rename` (release-safe) → one Boss create+rename → read `diagnostics.log` → then the fix `/migration`.

### Create-latency root cause CONFIRMED by measurement — the collision check (→ MIG-099)

**Two timing probe rounds settled it.** Round 1 refuted the prime suspect `read_library_tree` (measured **0–2 ms**, not seconds). Boss clarified: **create is slow, rename is fast** — which pointed away from the tree read (both paths run it) toward a create-specific cost. Round 2 instrumented the duplicate-name collision check `resolve_wikilink_cross_library` (commit `9bcd590a`). The live `diagnostics.log` (universe **Eisa Cognitive Knowledge**, 2 GB / ~7,700 notes) gave the smoking gun:

```
21:22:37  resolve_wikilink_cross_library took 13575 ms (matched=false)   ← the CREATE
21:22:56  resolve_wikilink_cross_library took   801 ms (matched=false)   ← the RENAME (OS file-cache now warm)
21:22:xx  read_library_tree           0–32 ms      update_links_on_rename 2 ms
```

**Mechanism (grounded in code):** `resolve_wikilink_cross_library_impl` (`libraries.rs:1779`) → for a name that matches no filename stem, `find_note_by_title_or_alias` (`libraries.rs:1905`) `fs::read_to_string`s the **whole** `.md` of **every** note in the current library **and** every other library to check frontmatter `title:`/`aliases:`. A **brand-new note name matches nothing** → full 2 GB **cold** read → **13.6 s**. Rename is fast because (a) the second scan hit the warm OS cache (801 ms) and (b) `update_links_on_rename` found 0 linkers (2 ms). Create runs this from `createNoteWithTemplate` (`+layout.svelte:~4042`, `resolveWikilinkCrossLibrary(lib.path, name)`).

**This is a textbook Rule 8 (Write-Time Derivation) violation:** the existence/collision check reads the source-of-truth (all files) at read-time instead of the always-current `note_meta` index (`path, name, name_lower` + `note_aliases(path, alias_lower)`; `name_lower` folded via `fold_match_key`). **Note:** `read_library_tree`'s whole-file read (site 1 of the earlier 3-site diagnosis) and `update_links_on_rename` (site 3) were NOT the dominant cost on this universe — the earlier 3-site framing over-attributed; measurement shows the collision check dominates. Sites 1/3 remain latent inefficiencies to bound, but MIG-099's target is the collision check.

**MIG-099 opened.** `/migration` Architect analysis running (`wf_cdcc99bc-1a7`, multi-agent): call-site census · index schema/currency + federation gap · resolution-invariant enumeration · WA#5 proven-pattern cross-check (Obsidian metadataCache / Logseq DataScript / SQLite multilingual folded-key lookup) · design synthesis · adversarial verification of the riskiest invariant claims. Plan → Boss approval before build.

### MIG-099 §1–§3 BUILT (create-latency Rule-8 fix) — awaiting Boss speed test

Plan approved (all 4 defaults). Architect: `docs/MIG-099-Create-Latency-Index-Resolver-Architect.md` (workflow `wf_cdcc99bc-1a7`, 11 agents; 4 of 5 naïve-design claims adversarially refuted → 5 corrections baked into the build).

- **§1 (`5db3e162`)** — additive indexed-resolver helpers in `libraries.rs`: `query_index_candidates` (two index-seeking arms over `note_meta.name_lower` + `note_aliases.alias_lower`), `path_under_library`/`norm_lib_path` (scope to one library → current-first/Vec-order/`library:note` preserved), `has_dot_segment` (drop `.trash`/`.constellation` the walk skips). 7 unit tests (name/alias/canonical-title/NULL-name_lower/dot-exclusion/prefix/Arabic-byte-tie-break). No call-site change.
- **§2 (uncommitted)** — `resolve_wikilink_cross_library` wiring: OWN libraries (`load_libraries`, NON-recursive — the C1 fix; NOT path-vs-root, verified against the live universe where own libs span 3 roots incl. external `E:\Cognitive Knowledge`) resolve title/alias via the indexed seek through `with_read_conn`; FEDERATED libraries keep the bounded walk; reader-unavailable → full walk fallback. Stem stage 1 (`find_note_by_name`) unchanged. Folds target with `fold_match_key`/`normalize_alias_for_match` (C4). Byte-shortest tie-break stays in the caller's Rust sort (C3). The dead single-library `resolve_wikilink` command (0 live callers) left as-is.
- **§3 (uncommitted)** — currency gaps: (a) `exists()` stat-guard on index hits (orphan-alias/unmounted/stale-trash rows); (b) `create_note` → `(async)` + **synchronous** `reindex_single_note` (surfaced-on-error, non-fatal) so the collision check is authoritative the instant a note exists — closes the divergent-title (`"Ratio A/B"`→stem `"Ratio A B"`) rapid-double-create false-negative (C2); (c) **discovered + fixed a latent index-drift app-killer:** `move_to_trash` moved the file to `.trash` but never dropped the `note_meta` row → the trashed note lingered in search at a dead path AND could phantom-collide a later create. Now `(async)` + `reindex_delete_note` (delete_path already did this; the standalone trash move did not).

**Self-validation on the live 2 GB / 7,725-note DB (WA#1, before the Boss test):** the exact helper SQL resolves the Arabic note `الحضارة الإسلامية` to the CORRECT path via `COVERING INDEX idx_note_name_lower` (no scan); the brand-new-name create case returns 0 hits in <1 ms (name arm). **13,575 ms → single-digit ms.** `cargo check` + the 7 §1 tests green. Per-build safety-inspection running diff-scoped (`wf_a7d5e452-16d`) — confirmed findings fixed before commit. Next: build release binary → Boss create-speed test.

### MIG-099 §6 — "truly instant" create (Boss chose it) — Predecessor→Replacement

**Boss ruling (2026-07-07):** create timing 13,575 ms → 324 ms (§2, 42×) verified, but Boss wants "truly instant" (sub-10 ms). The residual 324 ms is stage-1 `find_note_by_name` (filename-stem `read_dir` recursion across all 19 own libraries) — deliberately kept in §2. For the CREATE/RENAME title-collision use case the stem scan is unnecessary: the check answers "does a note with this TITLE exist?", which `note_meta.name_lower` + `note_aliases.alias_lower` answers directly (MIG-076 §E1b is title-ambiguity, not filename).

**Predecessor → Replacement (Predecessor Lookup Rule):**
- **Where it lives now:** `createNoteWithTemplate` (`+layout.svelte:4042`) and `handleRenameComplete` (`+layout.svelte:6040`) call `resolveWikilinkCrossLibrary(lib.path, name)` (`store.ts:2692` → `resolve_wikilink_cross_library`, `libraries.rs:1760`) — the FULL resolver (stem stage-1 + title/alias stage-2). Introduced by MIG-076 §E1b.
- **Where the replacement lives:** the SAME two call sites (same place, no relocation). A new purpose-built command `resolve_title_collision` (index-only for own libs: `name_lower`+`alias_lower`, NO stem `read_dir`; bounded title/alias walk for federated) + JS wrapper `resolveTitleCollision`. Implemented as the shared resolution impl with a `skip_stem` flag (ResolveCtx) — the 6 wikilink-resolution callers keep `resolveWikilinkCrossLibrary` (stem stage intact) unchanged.
- **Cut / kept:** the create/rename collision check stops paying the stem `read_dir`; the 6 read-path wikilink callers are untouched. **Behavior nuance (flagged to Boss):** the collision warning now keys on TITLE/alias match (its actual purpose), not on a bare filename-stem match when titles differ (that case auto-suffixes the filename as before). §3's synchronous create-reindex is load-bearing for §6 (index-only check requires the just-created note be indexed immediately — it is).
