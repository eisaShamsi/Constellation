# Constellation Pending Jobs

**Version 1.22 | 2026-07-12**

> **What changed in v1.22** (**PJ-091 — accepting a classifier suggestion silently truncated a note's MANUAL multi-value frontmatter — FIXED (Boss ruling: merge, never lose a manual value).** Root: "accept" was implemented as `set_manual(suggestion)` — a machine proposal treated as the user's exact manual assertion, so it OVERWROTE. Fixed at every accept seam: the suggestion is now UNIONED with the note's current on-disk values **under the write lock** (race-free), the merged set mirrored to the search index; the exact-set primitive is preserved (default-off `merge` flag) for any future direct-set. Reproduce-First: `pj091_repro_…` (RED→GREEN) + 3 fix tests; full regression green (sources 32 / classifier 15 / write_gate 22); svelte-check 0; `/simplify` applied. **The whole-app safety-inspection at this build found a NEW APP-KILLER → PJ-092** (rename-cascade silent edit-loss). SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-092** *(NEW · APP-KILLER)*: `flushAllTabsInLibrary` discards each pre-cascade flush's `SaveOutcome`, so a FAILED durable write of an open, dirty backlink-source tab is treated as success — the rename cascade then force-reseeds the model CLEAN from stale disk and deletes the recovery net, **silently and permanently losing the user's unsaved edits while the save-health banner self-heals to green.** The sibling `renameItem` path is already hardened against this exact class (`renameFlushOk`); the whole-library cascade path was not. Highest severity on the board — an app-killer jumps to Group-1 top (SO#9). *(Boss to confirm sequencing; or continue the standing Group-1 order below.)*
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-092** *(NEW · APP-KILLER)* — `flushAllTabsInLibrary` (`store.ts:1030`) rename-cascade silent edit-loss (discards the flush `SaveOutcome`; cascade proceeds on a failed flush → `reloadTabsFromDisk` force-reseeds stale + `clearWriteAhead` wipes the net; banner self-heals). **Fix = mirror the `renameItem` `renameFlushOk` gate (store.ts:3194-3240).** Focused, proven-sibling-pattern fix.
> 2. **PJ-089** — Index-panel preview two-writable-model silent clobber (HIGH).
> 3. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH).
> 4. **PJ-093** *(NEW · MED→HIGH)* — reindex silently skipped when `state.db` is None: `constellation_search_reindex`/`reindex_single_note` return `Ok(())` with the index un-updated (`search.rs:9285`/`:9172`, no `ensure_search_db_ready`), and every save/flush/stage-promote fires the reindex as `.catch(()=>{})` (`NoteEditor.svelte:264`) — a note durably written to disk is silently never indexed (search divergence, no error, no retry).
> 5. **PJ-086** — switchTab flush gap (HIGH). 6. **PJ-085 + PJ-073** — frontmatter/YAML round-trip (HIGH); **now also** PropertyEditor injecting a forced/registered property TYPE onto every projected key → breaks the G4 `composeFrontmatter` unification invariant (`PropertyEditor.svelte:364`). 7. **PJ-074** — durable rename + folder-rename descendant cascade *(+ the new nuance: `gate_rename` watcher-suppresses both folder paths, defeating the freshness heal; + the cascade ignores `CascadeResult.failed[]` at `+layout.svelte:6304`)*. 8. **PJ-083** — cascade sync-clear hazard. 9. **PJ-087 + PJ-075/076** — persisted-JSON non-atomic / fire-and-forget cluster (this sweep re-confirmed `review.rs:762`, `saveCollections`/`saveSettings`/`persistWorkspaces`). 10. **PJ-077** — sync-walk commands → async (*this sweep added a third: `collect_library_notes`, `libraries.rs:5280*`). 11. **PJ-072/002**.
>
> **LOW batch (Group 4/5):** `rename_item_db_tail` `.find()` picks the parent library not the most-specific (`libraries.rs:1115`); FocusPane title edit silently discarded — host never wires `ontitlechange` (`+layout.svelte:7879`); SS companion note-view model never `close()`d → unbounded per-window Map growth (`SecondScreenPage.svelte:995`). *(All filed to the Charter register; low blast radius.)*
>
> **Group 2 — Architecture & performance debt** — PJ-084/080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
> **Group 4 — Polish / i18n / small bugs** — unchanged (+ the 3 LOWs above). **Group 5 — Documentation & hygiene** — unchanged.
>
> ### CLOSED since v1.21
> - **PJ-091 — accept silently truncates manual multi-value frontmatter — DONE 2026-07-12** (Boss ruling: *merge, never lose a manual value*). **Root:** "accept a classifier suggestion" reused the exact-set primitive `set_manual(suggestion)`, treating a machine proposal as the user's exact manual assertion → it REPLACED. When a suggestion is stale relative to what the user has since typed on disk, accepting it drops the manual values. **Fix — union at every accept seam, under the write lock:** new `union_preserve_order(existing, additions)` (existing-first, append new); a default-off `merge` flag threaded through `rewrite_note_sources_on_disk`/`rewrite_note_content_type_on_disk`/`sources_set_manual`/`content_type_set_manual` (returns the effective merged set → mirrored to the DB so note_meta never diverges from disk); bulk `accept_one` unions both axes inline in its one dual-axis `gate_rmw`; per-card Accept (plain + edit-override) and disambiguation pass `merge:true`; the exact-set primitive is untouched for clear + any future PropertyEditor direct-set. **Part A:** `build_axis_suggestions` (refactor that DRYs the horizontal/vertical loop) now carries the classifier's `.secondary` sources the old builder dropped. **Verify:** `pj091_repro_accept_replace_truncates_manual_multivalue` (RED→GREEN) + `pj091_accept_merge_preserves_manual_multivalue` + `pj091_union_preserve_order_dedupes_and_appends` + `pj091_build_axis_suggestions_carries_secondary`; sources 32 / classifier 15 / write_gate 22 all pass; svelte-check 0; `/simplify` applied (removed RefCell machinery + 2 redundant clones); whole-app sweep — **0 in-diff findings.** Commit `<this>`. Orientation v3.43.
>
> ### NEWLY FILED — PJ-092, PJ-093 (from the PJ-091 whole-app safety-inspection `wf_f2a07366-fc5`, 37 agents, **17 confirmed**)
> - **PJ-092** *(APP-KILLER)* — `flushAllTabsInLibrary` rename-cascade silent edit-loss. **Open · Charter · Group-1 TOP · ► Next action.** Fix = mirror `renameFlushOk`.
> - **PJ-093** *(MED→HIGH)* — reindex silently skipped when `state.db` is None + reindex-error swallow. **Open · Charter · Group 1.**
> - *(The other 15 confirmed are pre-existing backlog: folder-rename+watcher-suppress=PJ-074, save_pulse=PJ-075, PropertyEditor-type=PJ-073/085, persisted-JSON cluster=PJ-075/087, sync-walk=PJ-077, + 3 LOWs to the Charter. The PJ-091 diff's OWN change: ZERO findings; `cece-sources-derived` scope 0 confirmed.)*
>
> ---
>
> *(Prior preambles v1.0–v1.21 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.21.md`.)*

**Version 1.21 | 2026-07-12**

> **What changed in v1.21** (**PJ-071 — the bulk Accept-All unlocked read-modify-write race — FIXED.** `accept_one` (`sources/bulk_ops.rs`) read the note UNLOCKED then `gate_write` — a concurrent editor save in the window was silently overwritten. Routed through the proven **`gate_rmw`** (read+mutate+write under one per-path lock, off-thread), mirroring the already-migrated per-card path. Behaviour-preserving (31 sources tests + 22 write_gate tests pass); the primitive's serialization is proven by `concurrent_writers_serialize_never_tear`. The per-build sweep surfaced ONE new HIGH in the same function (accept truncates manual multi-value frontmatter) → filed PJ-091. SO#9. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-091** (accept silently truncates a note's MANUAL multi-value frontmatter): the highest-value new safety item — a real silent content-loss on a routine "Approve All", distinct from PJ-071's race. *(Or continue the standing Group-1 order below; Boss's call.)*
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-091** *(NEW · HIGH)* — accepting a classifier suggestion REPLACES a note's `sources:`/`content_type:` with the suggestion's ids, silently truncating the user's MANUAL multi-value frontmatter (e.g. `sources: [testimony, perception]` → `[testimony]`). Root: the suggestion builder (`classifier/mod.rs:128-148`) reads only `primary + see_also`, drops `.secondary`; `accept_one` then REPLACES (not merges) the axis. Needs a look at the classifier synthesis + a ruling on accept semantics (merge-vs-replace / preserve-manual). *(Surfaced by the PJ-071 sweep; the accept path — same function as PJ-071 but a distinct bug.)*
> 2. **PJ-089** — Index-panel preview two-writable-model silent clobber (HIGH). *(re-confirmed by this sweep at `+layout.svelte:7300`.)*
> 3. **PJ-090** — SS Tasks-panel toggle no-broadcast clobber (HIGH).
> 4. **PJ-086** — switchTab flush gap (HIGH). 5. **PJ-085 + PJ-073** — frontmatter/YAML (HIGH). 6. **PJ-074** — durable rename + cascades. 7. **PJ-083** — cascade sync-clear hazard. 8. **PJ-087 + PJ-075/076** — atomic-write / lost-update residuals (this sweep re-confirmed `link_types.rs:535`, `review.rs:762`, `style_presets.rs:51`, `saveCollections` — the persisted-JSON non-atomic cluster). 9. **PJ-072/002**.
>
> **Group 2 — Architecture & performance debt** — PJ-084/080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
> **Group 4 — Polish / i18n / small bugs** — unchanged. **Group 5 — Documentation & hygiene** — unchanged.
>
> ### CLOSED since v1.20
> - **PJ-071 — bulk Accept-All RMW race — DONE 2026-07-12.** `accept_one` (`sources/bulk_ops.rs:269`) read the note unlocked (line 305) then `gate_write` (line 310) — the read→modify→write was not atomic, so a concurrent editor save in the window was silently overwritten by the stale-based rewrite. Fixed: one `gate_rmw(path, "bulk_accept", |content| …rewrite both axes…)` — read+mutate+write under the same per-path lock the editor's `write_note` uses, so a save lands before or after but never inside. Runs on the existing `thread::spawn` worker (no dispatch-thread freeze; gate_rmw rule #2 satisfied — the closure is pure string work, DB mirror update is after). A proven-pattern migration (mirrors the per-card `rewrite_note_sources_on_disk` + 6 other `gate_rmw` sites); behaviour-preserving (+ an idempotent-skip: identical rewrite → `Ok(None)`, no write). cargo check + 31 sources tests + 22 write_gate tests pass. Commit `<this>`. Orientation v3.42.
>
> ### NEWLY FILED — PJ-091 (from the PJ-071 per-build whole-app sweep `wf_4dd12a39-694`, 46 agents, 24 confirmed)
> - **PJ-091** — accept truncates manual multi-value frontmatter (HIGH). **Open · Charter · Group 1 · needs a classifier-synthesis look + a Boss ruling on accept semantics.**
> - *(The other 22 confirmed are pre-existing backlog — Index-preview=PJ-089, the persisted-JSON non-atomic cluster=PJ-087/075/076, folder-rename=PJ-074, yamlDoc nested=PJ-085, incoming-aggregate recompute=PJ-074, etc. Register appended to the Charter. The PJ-071 diff's OWN gate_rmw change: ZERO findings.)*
>
> ---
>
> *(Prior preambles v1.0–v1.20 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.20.md`.)*

**Version 1.20 | 2026-07-12**

> **What changed in v1.20** (**PJ-088 — the conflict-resolution SIDE-BY-SIDE MERGE view — SHIPPED + Boss-validated end-to-end.** The follow-up conflict layer the PJ-070 Architect deferred: when a note has a `.conflict` side-copy, the banner's **Merge…** button opens a two-column view (Your version | Outside copy) with a **◀ Copy to mine** button per difference + free edit; **Save merged** writes safely through the model + durability gate, **Cancel** is a pure no-op — zero loss until an explicit save. Art-Director-designed + adversarially-judged; the per-build safety-inspection's one in-diff finding fixed pre-commit. SO#9 reconciliation. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-071** (bulk Accept-All unlocked read-modify-write race) — unchanged; the ruled Group-1 safety item. (PJ-088 was a Boss-directed feature interleave, not a Group-1 item; Group-1 order below is otherwise intact, now with two new HIGH silent-loss items from the PJ-088 sweep.)
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-071** — bulk Accept-All unlocked RMW race. *(► Next action)*
> 2. **PJ-089** *(NEW · HIGH)* — the Index-panel preview mounts a WRITABLE NoteEditor on a standalone `index_preview_*` tab NOT deduped against the open store tabs → two independent writable models for one path → last-writer-wins silent clobber, no `.conflict` sidecar (the dedup + adopt paths both bypass it). `+layout.svelte:6442`.
> 3. **PJ-090** *(NEW · HIGH)* — the second screen's Tasks-panel `onToggle` writes the shared `.md` (Display-not-Domain breach) with NO `broadcastNoteSaved` and the write is watcher-suppressed → the main window never learns of it and clobbers the toggle on its next save. `SecondScreenPage.svelte:1681`/`:1537`.
> 4. **PJ-086** — switchTab flush gap (HIGH). 5. **PJ-085 + PJ-073** — frontmatter/YAML (HIGH). 6. **PJ-074** — durable rename + cascades. 7. **PJ-083** — cascade sync-clear hazard. 8. **PJ-087** — universe.rs tmp race. 9. **PJ-075/076/072/002**.
>
> **Group 2 — Architecture & performance debt** — PJ-084 (SS/main share the adopt primitive) · PJ-080/078/079/077/069-remainder. *(unchanged.)*
> **Group 3 — Feature completion** — PJ-067/068 · MIG-096 §3–§6 · MIG-088 Ph6–10 · Backup & Recovery. *(unchanged.)*
> **Group 4 — Polish / i18n / small bugs** — BUG-013 · PJ-082/014/049 · PJ-044.. · PJ-008.. · PJ-004 · PJ-017/018/019. *(unchanged.)*
> **Group 5 — Documentation & hygiene** — PJ-081 (§12 drift + orientation BODY refresh) · PJ-051/057(b) · PJ-011. *(unchanged.)*
>
> ### CLOSED since v1.19
> - **PJ-088 — Conflict-resolution side-by-side MERGE view — DONE / Boss-validated 2026-07-12.** Boss-requested follow-up to PJ-070 (shape = side-by-side, full live preview). Art Director design workflow `wf_d7453254-50e` (census + WA#5 prior art + 3 competing designs + 3 adversarial judges + synthesis) → `docs/PJ-088-Conflict-Merge-Design.md`. Banner **Merge…** → a full-center overlay: the official **`@codemirror/merge`** MergeView (2-way — no common ancestor is stored; lazy-imported into a 29KB chunk, Rule 6), live-preview panes, a **◀ Copy to mine** button per chunk (custom `renderRevertControl` after the default chevron tested too subtle). **The safety wire** `resolveConflictMerge` (store.ts): push the merge into the model via the NEW `replaceContent` (re-bases so compose emits the merged frontmatter verbatim — fixes the sweep's one in-diff finding: `editNoteProps` left `m.base` stale → non-projectable frontmatter silently dropped) → durability gate → remount + `markReseeding`/`await tick` (hazard #6) + `focusReseed` → sidecar `moveToTrash` (reversible) + `dismissConflict`, ONLY after a durable save; **Cancel = pure no-op**. `conflict.*` (13 keys) in all 15 locales. Reproduce-First: runtimeHarness Recipe P (re-base). svelte-check 0, vitest 335, cargo clean. Boss-validated: the merge round-trips (note reconciled + `cid` intact, sidecar in `.trash`, banner cleared). Commits `bc6a1e43` + `cd…59295333`.
>
> ### NEWLY FILED — PJ-089, PJ-090 (from the PJ-088 per-build whole-app safety-inspection `wf_c0dac305-85e`, 40 agents, 19 confirmed)
> - **PJ-089** — Index-panel preview two-writable-models silent clobber (HIGH). **Open · Charter · Group 1.**
> - **PJ-090** — second-screen Tasks-panel toggle no-broadcast + watcher-suppressed clobber (HIGH; incl. the split-view companion `:1537`). **Open · Charter · Group 1.**
> - *(The sweep's other 17 findings are pre-existing backlog — mapped to PJ-073/074/075/076/077/085/086/087 or LOW hygiene items; register appended to the Charter. The PJ-088 diff's OWN only finding — the stale-base compose — was fixed pre-commit via `replaceContent`.)*
>
> ---
>
> *(Prior preambles v1.0–v1.19 + full history follow below; also durable in `docs/Constellation Pending Jobs v1.19.md`.)*

**Version 1.19 | 2026-07-12**

> **What changed in v1.19** (**PJ-070 — the watcher external-change APP-KILLER — SHIPPED + Boss-validated end-to-end + `/migration` CLOSED.** An external `.md` edit to an OPEN note was silently clobbered by the next keystroke; the fix adopts the change into the single-ownership model (clean → adopt + remount; dirty → `.conflict` sidecar, zero loss). The per-cycle whole-app safety-inspection ran at this migration boundary — 1 in-diff finding fixed pre-commit, the rest filed as PJs / to the Charter. PJ-072's registry mystery got a concrete lead. SO#9 reconciliation. Ultracode):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-071** (bulk Accept-All unlocked read-modify-write race): the proven `gate_rmw` pattern already exists next door — per-card accept was migrated, bulk wasn't — so a concurrent editor save in the window is silently overwritten. The next Group-1 safety item now that PJ-070 is done.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. ~~**PJ-070**~~ — **DONE 2026-07-12** (`/migration`, commits `b1a3e388`+`cd5e53fd`, Boss-validated Stages 1+2). See CLOSED below.
> 2. **PJ-071** — bulk Accept-All unlocked RMW race. *(now the ► Next action)*
> 3. **PJ-086** *(NEW · HIGH)* — `switchTab` (`store.ts:2412`) is a note-editing departure that never flushes the outgoing dirty model (unlike the 3 guarded departures) → the last ≤1.5 s of edits on a non-active tab are lost on quit/crash with no banner/net (APP-KILLER #2 class, on the one unwired departure path).
> 4. **PJ-085 (with G4/PJ-073)** *(NEW · HIGH)* — `composeFrontmatter` H1 error-passthrough (`yamlDoc.ts:213`) silently discards **ALL** property edits whenever frontmatter is lenient-parseable by the app but strict-invalid to eemeli (a colon-in-a-title value, a duplicate key, an `@handle`, a stray tab) — the save reports success, the edit is gone. Folds into the frontmatter real-YAML round-trip work.
> 5. **PJ-073 (G4)** — frontmatter real-YAML round-trip (block-scalars/nested-maps/quote corruption) — now paired with PJ-085.
> 6. **PJ-074 (G5)** — durable rename Step-2 + folder rename/delete descendant cascade + link archive/unarchive incoming recompute + `move_item` review reset. *(the sweep re-confirmed the folder-rename/delete + archive/unarchive halves.)*
> 7. **PJ-083** *(NEW)* — the rename-cascade + `drainCidEnsure` **synchronous** `clearCascading` latent hazard #6 (a mis-timed `{#key}` teardown could re-stale). PJ-070 fixed its OWN path (deferred `await tick()` clear + a dedicated refcounted reseed gate); the cascade/cid paths still clear synchronously. Reproduce-First.
> 8. **PJ-087** *(NEW · G6)* — `universe.rs::atomic_write` uses a FIXED shared tmp filename → concurrent async saves of the same persisted-JSON collide/interleave.
> 9. **PJ-075 (G6)** + **PJ-076 (G7)** + **PJ-072** (registry investigation — LEAD below) + **PJ-002** (opt-in dup-`cid_cn` scan).
>
> **Group 2 — Architecture & performance debt**
> 1. **PJ-084** *(NEW)* — share the per-tab freshness-adopt primitive between the main-window `adoptExternalChangeIntoTabs` and `SecondScreenPage.adoptFreshDiskIntoSS` (one home per capability; they are documented twins today, and PJ-070's hazard-#6/reseed-gate fix does NOT apply to the SS copy — which is safe only because the SS is read-only). Verified NON-urgent: SS read-only = no teardown write.
> 2. **PJ-080** (`init_db` cold-init profiling first) · **PJ-078** (`map.rs` last Rule-8 read-walk) · **PJ-079** (`get_360_view` index-read rewrite) · **PJ-077 (G8)** (2 sync-walk commands → async) · **PJ-069 remainder** (whole-entity dedup). *(unchanged from v1.18.)*
>
> **Group 3 — Feature completion** — PJ-067 (Living-Link v2) · PJ-068 (Cockpit 9 facet tabs + contextual-SS) · MIG-096 §3–§6 (right-click rollout) · MIG-088 Ph6–10 (Style Setter) · Backup & Recovery. *(unchanged.)*
>
> **Group 4 — Polish / i18n / small bugs** — BUG-013 + sidebar highlight lag · PJ-082 (search-grammar 7 built-in link types) / PJ-014 (CECE 13-locale) / PJ-049 (Help viewer) · PJ-044/046/047/048/050 · PJ-008/009/010 · PJ-004 · PJ-017/018/019 · Arabic callout caret. *(unchanged.)*
>
> **Group 5 — Documentation & hygiene** — **PJ-081** (§12 doc-drift batch + orientation BODY refresh; NOW also add the *"Eisa Cognitive Knowledge" universe root = `E:\Cognitive Knowledge\`* fact) · PJ-051/057(b) Sight SVGs · PJ-011 (Map dormant).
>
> ### CLOSED since v1.18
> - **PJ-070 — Watcher external-change adopt — DONE / CLOSED 2026-07-12.** The silent-clobber APP-KILLER (the main-window mirror of the closed G3 class): an external `.md` edit to an OPEN note updated only `tab.content`, never the single-ownership note model / `reloadVersion`, so the mounted editor kept the stale body and the next keystroke's debounced save durably overwrote the external edit. **Fix (Boss-ratified Option B + `.conflict` sidecar):** one shared `adoptExternalChangeIntoTabs` helper both ingress paths (watcher flush + second-screen `onNoteSaved`) call — clean model → `adoptDisk` + `reloadVersion` remount (dedicated **refcounted** reseed gate spanning the async `{#key}` teardown, hazard #6); dirty + genuine change → `write_conflict_sidecar` (`<stem>.conflict-<UTCz>.md.txt`, inert to every `.md`-gated surface) + a banner (zero loss). Plus the **`setBody` string-no-op CLASS FIX** (a merely-viewed note never spuriously dirties — which otherwise defeated `adoptDisk` on background/focus tabs + raised phantom sidecars), the **Focus-mode suppressed reseed** (hazard #7), and the **`onNoteSaved` sibling-gap fold** (it adopted the model but forgot the remount). `/migration` Phases 1–4 (Architect `docs/PJ-070-…-Architect.md`, Plan `…-Plan.md`; 3-agent audit — all 11 invariants + the class-fix invariant HOLD). Reproduce-First: `tests/mig-076` Recipe O + `watcherAdoptStore.test.ts`. Behind `WATCHER_ADOPT_ENABLED` (rollback lever). **Boss-validated:** Stage 1 (adopt live, no clobber — disk-verified) + Stage 2 (dirty-conflict → sidecar + banner, both versions preserved). Commits `b1a3e388` + `cd5e53fd`. Orientation v3.40.
>
> ### NEWLY FILED — PJ-083 → PJ-087 (from the PJ-070 close + the per-cycle whole-app safety-inspection `wf_1b7addb3-822`, 38 agents, 15 confirmed)
> - **PJ-083** — rename-cascade + `drainCidEnsure` synchronous `clearCascading` latent hazard #6. **Open · Charter · Group 1 · Reproduce-First.**
> - **PJ-084** — share the per-tab freshness-adopt primitive (main helper ↔ `adoptFreshDiskIntoSS`). **Open · Group 2.**
> - **PJ-085** — `composeFrontmatter` H1 passthrough silently drops ALL property edits (HIGH). **Open · Charter · G4/Group 1.**
> - **PJ-086** — `switchTab` doesn't flush the outgoing dirty model → edit-loss on quit (HIGH). **Open · Charter · Group 1.**
> - **PJ-087** — `universe.rs::atomic_write` fixed shared tmp filename race. **Open · Charter · G6.**
> - *(The other 10 confirmed sweep findings map to existing PJs — `review.rs` save_pulse = PJ-075; folder rename/delete descendant cascade + link archive/unarchive incoming recompute = PJ-074 — or are LOW batch items: `parseFrontmatter` comma-split, `sources` prefix-strip, `saveCollections` un-awaited, `BacklinksPanel.linkMention` swallow, `perf_trace` unbounded Vec. Full register appended to the Charter.)*
>
> ### PJ-072 — INVESTIGATION UPDATE (concrete lead, 2026-07-12 — during the PJ-070 Boss test)
> The running instance's **"Eisa Cognitive Knowledge"** universe resolves to the on-disk root **`E:\Cognitive Knowledge\`** — the write-journal recorded the Boss's test note's `create_note` + `editor_save` + `conflict_sidecar` all under `E:\Cognitive Knowledge\Eisa Test\`. This is a DIFFERENT path than the `E:\Constellation Universes\Eisa Cognitive Knowledge\` folder that ALSO exists on disk (and is what the earlier registry search scanned). So WHERE the active universe's data lives is now known; the diagnostic build is still wanted to explain WHERE the display-name→`E:\Cognitive Knowledge` mapping is persisted (it is not in the findable `universes.json`). **Status: Open · lead captured (Charter updated).**
>
> ---
>
> *(Prior preambles v1.0–v1.18 + the full CLOSED / NEWLY-FILED history follow below, unchanged; the trail is also durable in `docs/Constellation Pending Jobs v1.18.md`.)*

**Version 1.18 | 2026-07-12**

> **What changed in v1.18** (Boss-directed 2026-07-12: fold the whole outstanding-work sweep into the ledger, then RE-PRIORITIZE every open job into five decision groups. Method — a full orientation-audit v3.00→v3.38 (39-version reconciled outstanding sweep), a per-PJ status re-verification of PJ-001→069 against the current code + orientation §8/§9 + session logs, and a merge with the Safety Charter's G2–G8 register. Every status flip cites a MIG/commit/§/date; three highest-value closures spot-verified in code.):
>
> ### ★ THE RE-PRIORITIZED BACKLOG — five groups, work top-down
>
> **► NEXT ACTION — PJ-070** (the watcher external-change app-killer): a note edited *outside* Constellation while open (git-pull / Syncthing / Obsidian) is silently clobbered by the next keystroke. It's the most-common external path and the main-window mirror of the just-closed G3 cross-window class — the fix reuses the existing `adoptFreshDiskIntoSS` pattern. Cheapest high-value safety win.
>
> **Group 1 — Safety & correctness** *(fix before any feature work)*
> 1. **PJ-070** — watcher external-change never adopts into the note model → next-keystroke clobber. *(silent, most-common external path)*
> 2. **PJ-071** — bulk Accept-All unlocked read-modify-write race. *(proven `gate_rmw` pattern already exists next door — per-card was migrated, bulk wasn't)*
> 3. **PJ-073 (G4)** — frontmatter real-YAML round-trip: block-scalars / nested maps dropped + quote corruption on **every** save. *(highest blast radius — corrupts rich-frontmatter notes silently)*
> 4. **PJ-074 (G5)** — durable rename Step-2 (MIG-098 remediation #1's durable half still owed) + folder rename/delete descendant cascade + link archive/unarchive incoming recompute + `move_item` review-history reset.
> 5. **PJ-072** — INVESTIGATION: universes booting from an unfound registry ("Eisa Cognitive Knowledge" 7,751 notes / "Scratch" absent from every on-disk registry). *(cheap diagnostic build first; unknown registry persistence = latent registration-loss)*
> 6. **PJ-075 (G6)** + **PJ-076 (G7)** — residual atomic-write / lost-update-guard gaps (`review-pulse.json`, swallowed review-priority writes; the write-gate staleness check that never fires). *(small, direct)*
> 7. **PJ-002** — opt-in duplicate-`cid_cn` re-canonicalize scan. *(low residual — MIG-003 §85 + the v3.25 partial index already auto-mitigate; only matters for restored pre-§140 backups)*
>
> **Group 2 — Architecture & performance debt**
> 1. **PJ-080** — profile `init_db` cold-init first (20–40 s, UNMEASURED — "do not guess" before any fix).
> 2. **PJ-078** — `map.rs::constellation_map_universe`, the **last remaining Rule-8 read-time filesystem walk** (OrgChart/Map re-walks disk on every open) → write-time/index-read.
> 3. **PJ-079** — `get_360_view` still re-walks disk (async wrapper shipped v3.22; index-read rewrite owed, own /migration).
> 4. **PJ-077 (G8)** — two sync-walk commands → `(async)` (`provenance.rs:100`, `libraries.rs:5148`). *(one-word freeze fixes)*
> 5. **PJ-069 remainder** — whole-entity dedup: hubs → `note_meta.incoming_count`, then tags / folders / note-lists (MIG-096 §3–§6 + NoteRow across ~26 surfaces). *(Step-0 cull + orphan/fragile already shipped via MIG-094)*
>
> **Group 3 — Feature completion** *(concept-papers ready or parked)*
> 1. **PJ-067** — Living Link Relationship Model v2 (Tension-first families/dimensions; concept ratified, build owed; /migration).
> 2. **PJ-068** — the Cockpit's 9 stubbed facet tabs (only "Links" wired) + the per-mode contextual-SS rulings. *(the read-only Knowledge Cockpit chassis shipped v3.35/3.38)*
> 3. **MIG-096 §3–§6** — the right-click-menu rollout remainder (Groups B/C/D + trees), parked behind the safety audit.
> 4. **MIG-088** — Style-Setter Phases 6–10 (search/index badges · Sky/OrgChart/Map D3 · calendar · dialogs/global · audit).
> 5. **Backup & Recovery system** — Boss-wanted; concept paper done; /migration + WA#5.
>
> **Group 4 — Polish / i18n / small bugs**
> 1. **BUG-013** open-editor cascade race; **sidebar active-item ~10 s highlight lag** (reproduce-and-instrument when it next fires).
> 2. **PJ-082** — search-query grammar hardcodes 7 built-in link types (`store.ts:2071`) so `supersedes [[X]]` / custom-type filters are dead; **PJ-014** — CECE/User-Manual 13-locale backfill (standing translation debt); **PJ-049** — in-app Help viewer (P2 — files exist in 15 locales, no UI surface).
> 3. **PJ-044/046/047/048/050** (MIG-022 UI polish); **PJ-028/029/033** stage-badge (PJ-030/031/032 deferred P3).
> 4. **PJ-008/009/010** backlink/outgoing render-dedup + alias-bleed (re-verify vs the MIG-079 panel rewrite first); **PJ-004** NSIS `os error 32`; **PJ-017/018/019** dead schema/flag/i18n cleanup; **Arabic callout End/Home caret** (documented — Boss ruled stop-patching).
>
> **Group 5 — Documentation & hygiene**
> 1. **PJ-081** — the §12 doc-drift batch (`IPC-CONTRACT.md` ~80 vs ~120 commands · `CANONICAL-FILENAME` `cid`→`cid_cn` · versioning all 0.3.4 · **the now-STALE "no frontend test harness" row** — a full vitest suite exists, 318/318) + the orientation BODY refresh (§3/§4.x/§13/§17 still ~v1.58; current truth lives only in stacked preambles — the SO#8 body-vs-preamble lag).
> 2. **PJ-051 / PJ-057(b)** Sight SVG follow-ups (dormant); **PJ-011** Constellation Map issues (DORMANT — Map disabled by MIG-038).
>
> ---
>
> ### CLOSED since v1.17 — moved to Done (v1.17 carried these wrongly as OPEN, or they shipped after)
>
> - **PJ-003** — rename-collision popup — DONE, MIG-076 §E-1 (`CollisionDialog.svelte`), orient. v2.76.
> - **PJ-005** — MIG-007 Links Settings tab — DONE (verified: `SettingsModal.svelte:180`/`:1381`), orient. v2.62.
> - **PJ-012** — `LinkLifecycle.fresh` tier — DONE (`store.ts:3301`/`:3472`).
> - **PJ-013** — `lenses.rs::apply_lens` decision — DONE (deleted via PJ-069 §0f, orient. v3.28).
> - **PJ-021** — Sky View Rule-8 — DONE (write-time triggers + boot-read killed, MIG-079 §C.2c).
> - **PJ-022 / PJ-023** — Backlinks / Outgoing Rule-8 — DONE (MIG-079 §C.2a `incoming_count` + §C.2c per-note index seeks).
> - **PJ-024** — Tag browser Rule-8 — DONE (MIG-079 §C.1 `tag_counts` + MIG-080 §B).
> - **PJ-041 / PJ-042 / PJ-043** — CECE i18n — DONE (MIG-022 §E; bodies were stale-Open).
> - **PJ-060** — `index_note` cache short-circuit — DONE (verified: `search.rs:5660` `force` gate cites PJ-060).
> - **PJ-063** — `note_links.link_type` globally 'relates' — DONE (verified: real varied `link_type` column, `search.rs:189+`).
> - **PJ-064** — Style-Setter more font types — DONE (full installed-font catalogue, 2026-06-11).
> - **PJ-065** — structural/parent-TOC link — DONE/CLOSED (2026-06-29, orient. v3.16, re-verified v3.35).
> - **PJ-066** — sky-reindex storm (~2 min → 3 s) + connect-freeze — DONE (orient. v3.15, memory `project_pj066`).
>
> *(Total: 16 PJs flipped to Done. The v1.17 §-bodies retain their historical text per the amendment convention — this preamble is the authoritative status delta.)*
>
> ### NEWLY FILED — PJ-070 → PJ-082 (deduped against PJ-001..069; sourced from the orientation-audit sweep + the Safety Charter)
>
> - **PJ-070** — APP-KILLER: watcher external-change never adopts into the note model (`+layout.svelte:3172`); an external `.md` edit updates only `tab.content`, never the single-ownership model / `reloadVersion`, so the next keystroke's debounced save durably overwrites the external edit (then reindexes, so search agrees with the stomp). The main-window mirror of the closed G3 class — fix reuses `adoptFreshDiskIntoSS`. **Open · APP-KILLER · Charter (2026-07-11 sweep).**
> - **PJ-071** — APP-KILLER: bulk Accept-All unlocked read→modify→write (`sources/bulk_ops.rs:305`) — `accept_one` reads unlocked then `gate_write`, the exact race `gate_rmw` prevents (per-card accept was migrated; bulk wasn't); a concurrent editor save in the window is silently overwritten. **Open · APP-KILLER · Charter.**
> - **PJ-072** — INVESTIGATION: universes booting from an unfound registry. The active instance boots "Eisa Cognitive Knowledge" (7,751 notes) + "Scratch", neither in any on-disk `universes.json` findable on the machine, while the shared write-journal records both. Ship a diagnostic build logging the resolved `app_data_dir` + registry path at boot. **Open · Charter OPEN INVESTIGATION (2026-07-12).**
> - **PJ-073** — G4: frontmatter real-YAML round-trip. Block-scalars + nested maps are dropped and quoted values corrupted on save; incl. the `yamlDoc.ts:150` nested-object-list flatten. The audit's #1 remaining silent-corruption item. **Open · Charter G4 (W2-7/8) · /migration.**
> - **PJ-074** — G5: durable rename + index cascades. The MIG-098 rename-index durable Step-2 (boot self-heal shipped, the durable write path didn't) + folder rename/delete descendant cascade + link archive/unarchive incoming recompute + `move_item` review-history reset. **Open · Charter G5 · /migration.**
> - **PJ-075** — G6 residual: `review-pulse.json` non-atomic (`review.rs:762`) + swallowed review-priority writes (`ReviewerView.svelte:286`, `ReviewStatusPanel.svelte:97`). *(Bulk of G6 landed — libraries.json + 4× universe.json in v3.38.)* **Open · Charter G6.**
> - **PJ-076** — G7: the write-gate lost-update / staleness guard never fires on the real save path (`write_gate.rs`, W2-13). **Open · Charter G7.**
> - **PJ-077** — G8 residual: two sync-walk commands → `(async)` (`provenance.rs:100`, `libraries.rs:5148`). *(Core G8 shipped.)* **Open · Charter G8.**
> - **PJ-078** — `map.rs::constellation_map_universe` — the LAST remaining Rule-8 read-time filesystem walk; the OrgChart/Map tree walks disk on every open → move to write-time/index-read. *(Supersedes the mistaken PJ-027 "Map already write-time" closure — MIG-078 found it walks.)* **Open · orient. §12 · /migration.**
> - **PJ-079** — `get_360_view` index-read rewrite (still disk-re-walks; async wrapper shipped v3.22). **Open · /migration.**
> - **PJ-080** — `init_db` cold-init profiling (20–40 s, UNMEASURED — measure before optimizing). **Open · Rule-8 measurement first.**
> - **PJ-081** — the §12 doc-drift batch + orientation BODY refresh (see Group 5 above for the full list, incl. the now-false "no frontend test harness" row). **Open · hygiene.**
> - **PJ-082** — search-query grammar hardcodes 7 built-in link types (`store.ts:2071`); `supersedes [[X]]` / custom-type filters don't work; + MIG-067 §F/§I. *(Distinct from PJ-063, which is DONE.)* **Open · single-surface fix.**
>
> ### Honest caveats (UNVERIFIED)
> - **Workbench → Collections** (v3.27 says shipped as MIG-092; some v3.33–37 preamble QUEUED lists still carry it) and **Quick Switcher** (MIG-093): git log confirms MIG-092 + MIG-093 both shipped, so the later "queued" mentions are stale preamble text — treated as DONE, not re-filed.
> - **PJ-008/009/010** render-dedup / alias-bleed were NOT re-verified against the MIG-079 panel rewrite — Group 4 flags "re-verify first".
>
> ---

> **What changed in v1.17** (PJ-069 opened + its filed counts corrected after the SO#8 cross-check + adversarial re-audit; concept + shape Boss-ratified):
>
> ### PJ-069 — concept-paper delivered, entry recomposed
> - **PJ-069** — **Whole-entity deduplication pass** — is now **IN PROGRESS (concept-paper-first)**. Concept paper: **`docs/concept-papers/PJ-069-Whole-Entity-Deduplication-Concept-Paper.md`**. The v1.16 filing (below) is **STALE in four ways** — the map it quoted (`wf_1d470cb8-9e8`) was drawn 2026-07-05, the *same day* the Navigator was deleted, and predates MIG-092/093. Re-audit `wf_2ae0f8c0-d59` (18 agents, adversarial verify) recomposed it:
>   - **Horse (ratified 2026-07-06):** *"one home per capability — every capability has one owner; every other surface mounts it instead of re-implementing. The whole entity is one system of single-owned capabilities, not a federation of copies."* Priority = **answer-duplication first**; scope = **the 7 filed clusters + 9 newly-found cross-cluster families**; first step = **the dead-code cull**.
>   - **Recomposed counts (map → today, verified live):** tags **6→6** (a *different* six — Navigator's 2 deleted, SS clone + Rust fs-walk pair surfaced; OrgChart/SearchHub are sanctioned mounts) · folders **4→5** (Navigator deleted; MoveDialog picker, DigestPane tree, the LIVE `map.rs` builder surfaced) · recents **3→2 hand-rolls** · orphan/fragile **5→9 surfaces / 5 divergent definitions** · hubs **3→6 in-degree substrates** (the CNS "one home" register disagrees with the Backlinks badge) · note-lists **9→26 live** (NoteRow/NoteList shipped, **1** adopter) · **confidence 2→0** — already resolved by the shared `ConfidencePicker` (MIG-077 §F, `fa98bf6b`, 2026-06-29 — *six days before the map*).
>   - **Four stale parts corrected:** (1) the counts named DELETED surfaces (Navigator pane, NavBrowserPane) — gone with MIG-091 `dfed6333`; (2) the "coordinate with MIG-090" clause is obsolete — MIG-090 resolved into MIG-091 (File Explorer) + MIG-092 (Collections); the `NoteRow`/`NoteList` primitive already **shipped** on `main`; (3) confidence-menu ×2 is **0** (already unified); (4) the entry predates MIG-092 (which executed the method once — Bookmarks+Workbench → Collections, one owner + two mounts) and MIG-093 (`searchFold`/`switcherRank` seeds; a new recents mount).
>   - **Method (proven twice):** one owner + sanctioned mounts — the MIG-092 unify + `RelatedCandidates` "6 mounts / 5 hosts" model. **9 new families in scope:** Arabic folds (×6 Rust), relative-time copies (+ an i18n violation), library-colour dots ×7, cUniverse federation grouping ×7, name-autocompletes, context-menu duo, search-history/MRU, link-type label lookups, note-open dispatch. **8 confirmed-dead items** queued for a Step 0 cull (TagsPanel, NoteGrid, `/libraries`+NavBar, `store.ts::timeAgo`, `lenses.rs` tag-lens, `bases.rs::scan_by_tag`, `map.rs::constellation_map_data`, the boot-bundle bookmarks fetch).
>   - **Correction to the "Sight is disabled" framing:** CNS v2 (`ConstellationSight2` + `SightPanel`) is **LIVE core**; only Sight v3/v4/v6/v7 + the Constellation *Map* frontend are disabled; `map.rs`'s hierarchy builder is **LIVE** via the default-on OrgChart. Status: **Open · IN PROGRESS · concept+shape ratified · per-cluster rulings → /migration.**
>
> ---

> **What changed in v1.16** (one new PJ filed — the entity-wide duplication debt, Boss-ordered):
> *(Superseded by the v1.17 recomposition above; the original filing is retained below for the diff trail.)*
>
> ### New PJ filed
> - **PJ-069** — **Whole-entity deduplication pass** (Boss-ruled 2026-07-05, from the whole-entity mapping `wf_1d470cb8-9e8` done for the Navigator concept). The Boss's law: *"we have to avoid duplication among all Constellation's core plugins/functions. They all should complement each other as a whole entity."* The map found pre-existing duplication debt ACROSS the entity, independent of the Navigator: **tag browsing in ~6 places** (Navigator pane · Dashboard All-tags · note Tags tab · OrgChart source · SearchHub category · NavBrowserPane's duplicate tag-tree builder) · **folder-hierarchy browsing ×4** (Explorer · Navigator pane · OrgChart · the disabled Map) · **recents ×3** · **orphan/fragile diagnostics ×5** (Reviewer lenses · 360° gaps · Tension panel · CNS blind spots · Sky) · **"hubs" rendered by 3 surfaces while CNS's paper claims exclusive ownership** ("this is their one home") · **note-list rendering hand-rolled in ~9 surfaces** · two hand-rolled copies of the confidence menu (Backlinks/Outgoing). Method when taken up: per-cluster rulings (one owner + sanctioned mounts, the Suggested-Connections "one list, five mounts" pattern is the model), then /migration-scale consolidation. Coordinate with MIG-090 (whose shared note-list primitive addresses the ×9 cluster) — do not double-fix. Status: **Open · Boss-ordered · Effort: per-cluster rulings → /migration.**
>
> **Queue note:** MIG-090's Navigator concept is RATIFIED (2026-07-05): the horse — "the Navigator translates what is in the user's mind into the notes it refers to — and holds that working set while the user works it" — with Form C (Workbench + Intent Bar). Architect delta in flight. The Dataview vestige was removed entirely same day (Boss: "We don't have Dataview!"; commit `40f59f44`).


> **What changed in v1.15** (one new PJ filed — the Second-Screen contextual-companion rework, Boss-parked):
>
> ### New PJ filed
> - **PJ-068** — **Second Screen: contextual companion rework** (Boss-conceived + parked 2026-07-04). During the F2′ Test-4 arc the Boss issued two governing rulings: **the SS must be CONTEXTUAL to the main screen**, and **it must NOT REPLICATE what the main screen already displays** — then ordered the full SS history + concept read and a paper written, and parked the rework: “We will set it aside for now… and tackle it in due time.” The paper is **`docs/concept-papers/PJ-068-Second-Screen-Contextual-Companion-Concept-Paper.md`** (history dig workflow `wf_42ec73f0-794`, 5 readers). Its replication audit classifies every SS mode: **REPLICATES** — Navigator companion (the same `NotebookNavigator` the main sidebar shows at the same moment), OrgChart mode (also UNREACHABLE dead code); **MIXED** — Map companion, Index term leg, Universe Dashboard (ambient), plus a NON-CONTEXTUAL fallback tab-strip editor; **COMPLEMENTS (keep)** — Sky graph companion, Split comparison, Editor-panels migration, Index compare leg. Structural riders: the editor-panels mode-shadowing drift (if-chain → needs a real mode state machine; static read, not runtime-reproduced), the dead `screen:open-note` wire, Rule-8 re-walk reads, alias-blind `buildSkyData`, hardcoded-English ×15 + missing right-click, and 5 doc-drift items (incl. CP-26 still listing the FIXED `setActiveUniverse` breach, and the manual’s mode table documenting the unreachable mode). Per-mode retire/redesign decisions are **Boss rulings at reopen**; then /migration. What it must NOT do: add operations to the SS, remove the two-monitor gate, or make the SS self-initiate. Status: **Open · PARKED (Boss 2026-07-04) · Concept paper DONE · Effort at reopen: Boss rulings → /migration.**
>
> **Queue note:** the active work at v1.15 filing is the app-freeze program’s Batch W (shipped `d9f8bd80`, Boss test pending) with deferred batches L′/F1/F3 next in rank; PJ-068 joins the deferred Concept-Paper-done tier alongside PJ-065/PJ-067’s pattern.


> **What changed in v1.14** (MIG-086 in flight — folding in the "typed links in frontmatter" discovery; one new PJ filed):
>
> ### MIG-086 (Suggested-Relatedness → One-Click Typed Link) — in flight
> §A (`suggest_related_notes` BM25 "More Like This") + §B (read-only `<RelatedCandidates>` list) SHIPPED + Boss-validated. §C (the one-click typed-link action: `<LinkTypePicker>` + headless `addLinkToNote`) built + Boss-tested PASS, then **reopened by a Boss design discovery (2026-06-24)**: a typed link appended as dangling `[[type::target]]` body text "without context is illogical." Cross-system research (Obsidian Properties, Dataview, **Breadcrumbs**, Logseq/Roam/Tana, RDF/property-graph KR theory — workflow `wf_ce7372d6-d02`, fact-checked) confirmed: a *declared* relationship (the one-click kind, not woven into prose) belongs as **frontmatter metadata**, not body text. **Boss ruling 2026-06-24: continue MIG-086 §C and FOLD the frontmatter-typed-links approach into this migration** — the connect writes a frontmatter typed-link property; `index_note` learns to read frontmatter typed-links (dual-source with body); inline body `[[type::target]]` stays for *contextual* links; one unified `note_links` index. Earned properties (weight/confidence/traversal) remain index-maintained. (Architect doc + updated §C plan pending; §C currently uncommitted in the working tree pending the fold.)
> **Doc-drift surfaced (must reflect in orientation):** the CLAUDE.md "Living Link = first-class `LINK` file on disk (source of truth) + `note_links` index" claim **does not exist in code** — links are 100% body-`[[type::target]]`-derived into `note_links`; the rich properties live only in that table. The fold is the first real decision on where a link's *birth* is authored.
>
> ### New PJ filed
> - **PJ-065** — **Brand-new frontmatter-structured link type** (Boss-conceived 2026-06-24). A NEW kind of typed link designed specifically for frontmatter "Properties": e.g. linking many child notes to a single parent note (a Table-of-Contents / hierarchy relation). Distinct from the 8 cognitive typed links — this is a *structural/containment* relation with high-value use-cases (authors & screenwriters organizing chapters/scenes/arcs under a work; outline/MOC hierarchies; "the sky is the limit"). **Research-first / Concept-Paper-first; /migration-scale.** Study the precedent (Breadcrumbs' up/down/parent/child direction model + auto-implied reverse; Tana's "Part of" semantic field; Obsidian Properties), decide the schema (property shape, direction, reverse-implication, how it renders/edits, how Sky/Reviewer/Org-chart consume it), THEN build. Depends on the MIG-086 frontmatter-typed-links foundation landing first. Status: **Open · Deferred (future)** · **Effort: /migration (research → Concept Paper → Architect → Plan → Build → Audit).**
>
> - **PJ-066** — **Sky-trigger reindex storm on link-dense notes** (P1 perf; surfaced 2026-06-24 by the MIG-086 §F2 connect-latency diagnosis, workflow `wf_9d58a4b6-d73`). `index_note` DELETEs+re-INSERTs ALL of a note's `note_links` edges on every reindex; each edge fires `note_links_sky_stratum_ai/ad` + `note_links_sky_maturity_ai/ad`, whose `STRATUM_SQL_EXPR`/`MATURITY_SQL_EXPR` (search.rs:183-317) run repeated `COUNT(DISTINCT source_path) FROM note_links WHERE status… AND (target_name = id OR target_name IN (alias subquery))` over the 233,995-row table. For a link-dense note (e.g. a Wikipedia import) that's O(edges × big-subquery) ≈ **~2 minutes**. PRE-EXISTING (every link-dense note's *save* pays it too — hidden because saves reindex fire-and-forget). MIG-086 §F2 exposed it by `await`ing the reindex; fixed THERE by making the connect non-blocking (fire-and-forget + optimistic Reviewer). **This PJ is the ROOT fix.** Options (need Rule-8 measurement on the 7,660-note / 234k-sky-link universe): (a) composite index `note_links(target_name, status, source_path)` to fast-path the COUNT(DISTINCT) subqueries; (b) defer stratum+maturity recompute OFF the per-edge write path into a batched/dirty-flag drain (MIG-079 §C.2a `incoming_count` backfill pattern; `sky_nodes.enrichment_dirty` + a stamp-gated worker already exist); (c) make `index_note` diff edges (only fire triggers for changed edges, not DELETE-all+INSERT-all). Belongs to the SKY/MIG-079 perf domain — Concept/Architect-first, /migration-scale. Status: **Open · P1 · /migration (needs measurement).** Also noted: get_due_notes Lens-2 per-note `stale_probe_sql` (review.rs:315-334) is a secondary contributor — index `note_links(source_path, weight)` if it shows in measurement.
>
 - **PJ-067** — **Living Link Relationship Model v2 — Concept Paper** (Boss-conceived 2026-06-24, from the typology dig `wf_f97e9d18-518`; research doc: `docs/Living-Link-Relationship-Typology-Research-2026-06-24.md`). The directional-vs-symmetric discovery opened a much larger map: a typed link varies on **DIMENSIONS** (symmetry · **transitivity** · **inverse/converse** · **arity (binary vs n-ary)** · cardinality · **taxonomic-vs-thematic**) and belongs to **FAMILIES** (class-inclusion ✓, contrast ✓, causal ✓, evidential ✓, part-whole ✓-coarse, derivation ✓, supersession ✓; **uncharted:** thematic/functional [used-for, prerequisite-of, precedes, near], **analogy/structure-mapping** [analogous-to/maps-to — the synthesis engine], **n-ary synthesis**, argument-attack [**undercuts/undermines**], question relations [**problematizes/answers**], qualifies/limits). Maps to the **Five Acts**: the thinnest acts — **Tension** (no inference/premise attacks, no questions) and **Synthesis** (no analogy, no n-ary) — are exactly the gap for a *formulation* tool. **Two load-bearing flags:** (1) rename the proposed `complements` → **`co-completes`/`jointly-constitutes`** — lexical "complementarity" means the OPPOSITE (mutually-exclusive opposition, dead/alive); (2) "together they form an idea" is genuinely **n-ary** (a synthesis node), not a pairwise symmetric link. **Concept-Paper-FIRST (Boss defines the cognitive vocabulary), then /migration.** The symmetric tier + `co-completes` + all new families/dimensions live HERE, not in MIG-086. Status: **Open · Deferred (Concept-Paper-first).** Depends on MIG-086's frontmatter foundation.
>
> **Top of queue (v1.14):** MIG-086's frontmatter fold is the *active* work (finish §D/§E on the CURRENT 8 + directional/symmetric model); **PJ-066 (P1 perf)** is the highest-leverage new open item; PJ-065 (frontmatter parent/TOC type) + PJ-067 (Relationship Model v2) are the deferred Concept-Paper-first vocabulary frontiers.

> **What changed in v1.13** (RECONCILIATION — the ledger was ~3 weeks + 37 migration numbers stale; MIG-036→072 folded into orientation §8 (v2.61); PJ deltas applied; one ledger error corrected; new top-of-queue):
>
> ### Why this version exists
> v1.12 (2026-05-19) was the last backlog refresh. Since then **37 migration numbers (MIG-036 → MIG-072)** were opened — yet **none** appeared in either this ledger or the orientation §8 Migrations table (which had stopped at MIG-035 since the v2.16 refresh on 2026-05-18). The 2026-06-09 handover compounded this by naming **v1.9** as the latest Pending Jobs file when v1.10 / v1.11 / v1.12 already existed on disk. This version reconciles both ledgers against the orientation preambles (v2.17→v2.60) + session logs (2026-05-19→06-09). **The authoritative migration ledger now lives in orientation §8 (v2.61)**; this file tracks PJs and references §8 for MIG status.
>
> ### Migrations since v1.12 — summary (full table in orientation §8 v2.61)
> - **Shipped / Closed (23):** MIG-038 (disable Sight+Map → external "Wings"), 039 (The Cataloger), 040 (NSC), 041 (term_vocab bigram shrink), 042 (drop `bridge_concept_id` — **closes PJ-016**), 043/044/045 (NSC Core Plug-in P1–P3 + Universe Digest), 055 (Constellation Base rebuild), 056 (Cross-Universe Federation), 057 (Lexicon expansion), 058/059 (federated search latency → sub-second), 060 (Base threading gestures), 061/062 (boot-snapshot + filesystem-walk federation + Tag Browser), 065 (Unified Base), 066 (Living-Link columns), 067 (**User-Definable Link Types** — the Link-Type Registry), 069 (Style Presets), 070 (**Style Setter**), 071 (theme subsystem removed), 072 (Sky View vocabulary under the Style Setter — milestone `localization-complete`).
> - **Reverted (4):** MIG-046/047/048 — the **Constellation Mind** local-LLM stack (Fanar 1.9B) shipped end-to-end then was fully reverted (v2.34, `a9cf4d62`): CPU at ~5 tok/s didn't justify the value. MIG-054 (first Base attempt) reverted same-day → superseded by MIG-055.
> - **Reserved / never-opened (8):** MIG-049→053 (Constellation-Mind roadmap numbers, abandoned with the revert), **063/064 — the remaining ~6 cross-universe federation surfaces (still pending)**, 068 (rank-aware column sort, deferred).
> - **Dormant / Frozen (2):** MIG-036 (Sight v7 redesign, dormant — over-engineered), 037 (Sight v6.3 Time Dome, frozen) — both under MIG-038's Sight-disabled umbrella.
>
> ### PJ deltas applied this version
> - **PJ-016** (drop `term_vocab.bridge_concept_id`) → **DONE via MIG-042** (orientation v2.25). Moved to §9.
> - **PJ-011** (Constellation Map) → **DORMANT** — Map was disabled in core by MIG-038 (re-categorized as a future "Wings" plug-in). Stays filed; not actionable while Map is off.
> - **Ledger-error correction (Eisa-confirmed):** the 2026-05-29 session log labelled two federation-scale fixes **"PJ-10 / PJ-11"** — colliding with the real **PJ-010** (Unlinked Mentions) and **PJ-011** (Map). They were actually filed *unnumbered* on 2026-05-28 ("PJ-NNN-A/B"). They are allocated proper, never-before-used numbers here: **PJ-061** (Sky View federated node sizing, `f05fe6f9`) and **PJ-062** (CNS gravity-well canvas, `9a2d9890`), both **DONE 2026-05-29**. The canonical PJ-010 / PJ-011 are untouched (renumbering is forbidden — these were never formally numbered).
> - **PJ-001** confirmed **DONE via MIG-015** (already correct in the §1 body; the stale memory `project_mig013_v2_migration_blocking_boot` is corrected separately, in this same turn).
>
> ### Newly filed (PJ-059 → PJ-064 — see the "Newly filed" section before §9)
> - **PJ-059** — Sight per-note search/finder (open; **dormant** while Sight is disabled).
> - **PJ-060** — `index_note` cache-hit short-circuit fix (open; **P1 — flagged 2026-05-19 as "the single most-leveraged open fix"**; a write-time-derivation blocker).
> - **PJ-061 / PJ-062** — the two federation fixes above (both **DONE**).
> - **PJ-063** — `note_links.link_type` is globally `'relates'` (open; foundational /migration; **re-verify under MIG-067**, which shipped the Link-Type Registry after this was first observed).
> - **PJ-064** — Style Setter: more font types in the final version (open; minor — named/saved colour swatches already shipped in MIG-070).
>
> ### Already-closed in the v1.12 preamble, now reflected in §9 Done
> PJ-035 (DONE, MIG-019 §2B) · PJ-036 (Abandoned 2026-05-18) · PJ-038 (Superseded by Sight v6) · PJ-040 (DONE, MIG-022 §D `c072700`) · PJ-015 (Abandoned 2026-05-18) · PJ-052 / 053 / 054 / 055 / 056 / 057 / 058 (DONE 2026-05-18/19 in the MIG-026 + Sight-delivery cascade). *(These were announced in v1.12's preamble but never moved into its §9 Done body table; this version completes that bookkeeping.)*
>
> ### New top-of-queue (replaces v1.12's, which still listed the superseded PJ-038 as In-Progress)
> 1. **PJ-060** — `index_note` cache short-circuit (P1 blocker, highest leverage).
> 2. **PJ-005 / MIG-007** — Links Settings tab (P1 user-facing; no Architect yet).
> 3. **PJ-063** — `note_links.link_type` 'relates' bug (foundational /migration; re-verify under MIG-067 first).
> 4. **PJ-002 / PJ-003** — `cid_cn` collision scrub + rename-collision popup.
> 5. **PJ-017 / PJ-018 / PJ-019** — MIG-013 cleanup remainder (PJ-016 now done).
> 6. Backlog: remaining federation surfaces (MIG-063/064) · CECE i18n (PJ-040 done; PJ-041–043 open) · MIG-022 polish (PJ-044–050) · PJ-008/009 typed-link dedupe · MIG-023 Warrant Research (reserved, Concept-Paper-first).
>
> ### Reconciliation depth (honest note)
> This version refreshes the **preamble, top-of-queue, the §9 Done index, and the PJ entries with hard evidence of a status change**. The ~60 carried-forward PJ bodies keep their v1.12 status unless evidenced above; a deeper per-PJ code-audit (and a refresh of the stale "Cross-references" appendix) is a low-priority follow-up, not done here. Every status above cites a commit, an orientation version, or a session-log date.
>
> **What changed in v1.12** (MIG-026 SHIPPED — Sight v6.3 24-tradition expansion + full 15-locale localization · Phase μ ship-gate audit closed clean · 5 new PJs filed for deferred polish):
>
> ### MIG-026 SHIPPED
>
> 24-tradition expansion + 9 shape renderers + user-definable layer (declarative JSON + TS plugin loader) + full localization across 15 locales — all merged on `main` between Phase γ (2026-05-17 evening) and Phase μ (2026-05-18). Boss-validated across 3 stages (Arabic-localization Stage 1 + RTL-chevron polish Stage 2 + cross-locale spot-check Stage 3 in zh / de / ru: **all PASS**). Phase μ Migration Rule audit (3 parallel agents on invariants / drift / migration-path) returned **zero blockers**: all 10 invariants PASS, 2 advisories, 1 high-severity doc-drift (this PJ file's v1.11 header), 2 low-severity drift items.
>
> ### MIG-026 collision finally resolved
>
> v1.11 reserved MIG-026 for "Sight v5 Layer 3 recommendation (V3-§7.b llama.cpp wiring)" — that allocation was contradicted when MIG-026 was actually opened in 2026-05-17 as the Sight v6.3 24-tradition expansion. The Layer-3 recommendation work folds into a future MIG-029 or later (TBD). MIG-027 is also reallocated: it shipped 2026-05-17 evening as Sight-follows-the-interface-theme (chrome/semantic color split), not the Sight v5 Layer 4 coaching workstream.
>
> ### 5 new PJs filed
>
> - **PJ-052 — Concept Paper v4.1** — **DONE 2026-05-18** (`docs/Constellation-Sight-Concept-Paper-v4.1.md`, 919 lines / ~14,242 words; 24 tradition subsections at ~400 words each + new §3.5 nine-shape-renderer architecture + §3.6 user-definable plugin layer + invariants 12+13 on $t labelize + plugin label passthrough + §4.2 trimmed to Mohist-only v1-preview survivor + §4.1.2 pramāṇa / §4.1.3 masādir doc-drift corrected E/S/W/N). Three additional doc-drift items surfaced during the write — folded into **PJ-057** (below).
> - **PJ-053 — λ-fix-6 native-quality translation re-audit** — **DONE 2026-05-18** (4 parallel polish agents). 192 keys polished across 7 locales: ru (35 — wrong-script Cyrillic + transliteration glosses), hi (23 — wrong-script Devanagari + glosses), de + fr + es (42 each — bare transliteration → `transliteration · native-gloss` pattern), pt (6 — PT-PT → PT-BR dialect unification beyond the audit's named 3), zh (2 — extension aria/tooltip transliteration → native Chinese). Now every Sunni-Islamic, Sanskrit pramāṇa, Hebrew PaRDeS/middot, Greek/Latin Husserl, and Arabic Ibn Rushd / Shāṭibī technical term in the 7 polished locales carries a target-language gloss matching the ar/zh/ko quality bar.
> - **PJ-054 — Sight v6 vitest test runner** — **DONE 2026-05-19** in MIG-030. Installed `vitest@4.1.6`; wrote `tradition-isolation.test.ts` (Plan §14.1, channel-isolation invariant for all 24 traditions) + `tradition-perf.test.ts` (Plan §14.2, ≤16ms switch on 7,636-note universe) + `vitest.config.ts` (scope: exclude worktree duplicates + the still-deferred playwright layout-fidelity test). **58/58 tests pass.**
> - **PJ-055 — User-plugin label schema warning** — **DONE 2026-05-18** in commit `e63ee0c7`. `docs/traditions/schema/tradition.v1.schema.json` top-level description now warns that dotted-path-shaped literal labels would collide with Constellation's own i18n key namespace.
> - **PJ-056 — MIG-026 drift cleanup** — **DONE 2026-05-18** (3 stale doc comments fixed inline in commit `e63ee0c7`: dome.ts STRATUM_LABELS, types.ts:444 TraditionModule.name, traditions/index.ts:222 FAMILIES.label). The literal-deletion phase (24 `name:` literals + 10 `FAMILIES[*].label` literals) is **closed by Eisa decision 2026-05-18**: accept the duplication as documentation. The literals serve as the canonical English source-of-truth that en.json mirrors and as a defensive renderer fallback for the unsupported-locale + missing-en-entry edge case. No further work planned.
> - **PJ-057 — Post-MIG-026 doc-drift surfaced during Concept Paper v4.1 write** (3 items): (a) v4.0 §4.2.3 Mohist citation says *Mòzǐ* ch. 35; the shipped manifest cites Book IX — pick one canonical citation. (b) `docs/Sight-vNext-MockB1-Toggle.svg` and `docs/sight-redesign-v0.2-mockE-tradition-registers.svg` show pre-expansion 7-tradition state; needs a fresh 24-tradition variant. (c) Concept Paper §9.1 (and the orientation §17 list of unread files) should footnote that `_manifests.generated.ts` is prebuild-generated and must regenerate when manifests change. **(a)** no-op 2026-05-19 (v4.1 already cites manifest-canonical form); **(c)** DONE 2026-05-19 in `f327d758` (Concept Paper §9.1/§9.3 updated); **(b)** still open (visual design work).
> - **PJ-058 — Constellation Sight Subsystem Concept Paper v1.0** — **DONE 2026-05-19**. New ~6,700-word scholarly document at `docs/Constellation-Sight-Subsystem-Concept-Paper-v1.0.md` defining Sight as a subsystem of Constellation, enumerating its 8 functions (F1-F8), placing it in the 4-stratum subsystem map (structural / authoring / diagnostic-visualization / infrastructure), documenting its 8 architectural invariants (I1-I8), and tracing v2 → v6.3 subsystem history. Complement to the existing `Constellation-Sight-Concept-Paper-v4.1.md` (internal-design contract): subsystem paper = "what Sight is in Constellation"; v4.1 = "how Sight is built internally". Filed + delivered same-turn per Eisa direction.
>
> ### What did NOT close in v1.12
>
> - PJ-005 (MIG-007 Links Settings) — still open.
> - PJ-002 (cid_cn collision scrub) — still open.
> - PJ-008 / PJ-009 (typed-link duplication) — still open.
> - PJ-044/046/047/048/049/050 — MIG-022 polish backlog — still open.
>
> ### What did close (recap)
>
> No prior-open PJ closed by this cascade. The 5 new PJs (052–056) are all newly filed.
>
> ### Post-MIG-026 §μ state-of-standing closures (2026-05-18 turn-late, Eisa decisions)
>
> Triggered by an Eisa-requested triage of all remaining work. The 3-agent audit (MIG entries / PJ-001-040 / PJ-041-057) surfaced 5 NEEDS-DECISION items. Eisa rulings:
>
> - **MIG-005** Alias-aware in-memory inbound — **ABANDONED**. Steps 1-3 (`§121/§122/§123`) stay shipped; Steps 4-8 abandoned after the fabrication-catch pause. Reason: low leverage given current priorities.
> - **PJ-015** 360.3D Stratification Matrix guidance doc — **ABANDONED**. Reason: matrix-UX dependency hasn't moved; doc was low-leverage. Refile fresh if user-facing need surfaces.
> - **PJ-036** Sight layer peeling — **ABANDONED**. Reason: Sight v6's facet sidebar substitutes for the mechanic; the v2 Concept Paper §2.2 mechanism is no longer relevant under v6 architecture.
> - **PJ-056** literal-deletion sub-question — **CLOSED as documentation**. The 24 `name:` + 10 `FAMILIES.label` literals stay; they're canonical EN source-of-truth + defensive renderer fallback.
> - **MIG-022 §N** — **CLOSED 2026-05-18** (this turn). The §N audit landed 2026-05-12 (4 docs at `lab/reports/MIG-022-§N-*.md`) but Eisa never explicitly locked D-N1/D-N2; the P1 trigger-coverage fix shipped in commit `1240984d` (MIG-024 §0 UPSERT) implicitly chose option (α) + timing (a). Retroactive §8 close-out section appended to `MIG-022-§N-FINAL-INTEGRATION-AUDIT.md` recording the close. **MIG-022 status → DONE**. P2/P3 polish items (F2-F7) remain in cleanup backlog as future polish MIG; F8 (i18n gap) partially resolved by MIG-026 §λ across all 15 locales.
>
> ### Drift fixed during the audit (ledger reflected reality after the closures)
>
> - **PJ-035** body status "Open" → **DONE in MIG-019 §2B** (`16063735`). Milky Way density wash shipped the TF-IDF mechanic.
> - **PJ-040** body status "Open" → **DONE in MIG-022 §D** (`c072700`). Already noted in v1.11 preamble; body never flipped.
> - **PJ-038** body status "In-Progress" → **SUPERSEDED by Sight v6 / MIG-024 → MIG-027**. The 3-MIG Sight v3 trajectory was abandoned at commit `29ce0101`.
>
> ### Top of queue rotates (post-MIG-022 §N close)
>
> 1. **PJ-005 / MIG-007** — Links Settings tab (P1 user-facing; no Architect yet).
> 2. **PJ-002** — `cid_cn` collision scrub utility (P1 mini-MIG).
> 3. **PJ-003** — Rename-collision popup (P1 UX).
> 4. **PJ-008 + PJ-009** — Typed-link duplication pair (P2 single-file fixes).
> 5. **PJ-016/017/018/019 bundle** — MIG-013 cleanup MIG (4 PJs → 1 MIG).
> 6. **MIG-023** — Constellation Warrant Research workstream (Concept Paper first).
>
> **Done count after v1.12 post-audit**: 12 (+5 — PJ-052 Concept Paper v4.1 + PJ-053 λ-fix-6 + PJ-055 schema warning + PJ-056 drift cleanup + PJ-035 status-correction + PJ-040 status-correction). **Cancelled / Abandoned**: 4 (PJ-015 + PJ-036 abandoned 2026-05-18; PJ-034 retained from earlier; MIG-005 abandoned). **Rejected**: 1 (PJ-037). **Superseded**: 1 (PJ-038). **Open PJs**: 48.
>
> ### Post-2026-05-18 same-day rollup (2026-05-19 Sight delivery cascade)
>
> Eisa cascade direction: "Proceed with Tiers 1-3 and drop Tier 4. Whatever is related to Sight v5 shall be abandoned." Then: "Add to it developing Constellation Sight Concept paper."
>
> Closed same-day this turn:
> - **PJ-054** — Sight v6 vitest test runner — DONE in MIG-030 (`f327d758`). 58/58 tests pass.
> - **PJ-057.a** + **PJ-057.c** — Mohist citation no-op + prebuild footnote in Concept Paper §9 — DONE in MIG-032 (`f327d758`).
> - **PJ-058** — Constellation Sight Subsystem Concept Paper v1.0 — DONE this turn (~6,700 words; new doc at `docs/Constellation-Sight-Subsystem-Concept-Paper-v1.0.md`).
>
> Plus: **MIG-028** (Sight v5 retirement) + **MIG-030** (vitest) + **MIG-031** (λ-fix-6.b fa/he/ja/tr canvas polish — 57 keys across 3 locales) + **MIG-032** (Tier 3 housekeeping) all shipped in `f327d758` (the Sight delivery cascade). Plus **BUG-fix** (Sight anchor Shift+click) shipped in `4b20795b`.
>
> **Done count after 2026-05-19 rollup**: 12 → **15** (+3 — PJ-054 + PJ-057.a/c + PJ-058). **Open PJs**: 48 → **45**.

> **What changed in v1.11** (Sight v5 Concept Paper canonical · MIG-022 collision resolved · MIG-024 / 025 / 026 / 027 reserved · 1 new PJ filed):
>
> ### Sight v5 Concept Paper v3.1 ratified — MIG number-collision resolved
>
> `docs/Constellation-Sight-Concept-Paper-v3.1.md` is now the canonical Sight v5 design contract (Eisa-approved 2026-05-12 on all 6 validation points). Resolution of the MIG-022 number-collision committed in writing:
>
> - **MIG-022** stays with the gap-analysis-response cascade (§0 + §D + §E + §A shipped; §B Rust foundation shipped; §B UI overlay contradicted-and-deferred; §N can fire now).
> - **MIG-023** stays reserved for the Constellation Warrant Research workstream (per Eisa's D-C1 commitment 2026-05-11).
> - **MIG-024** = Sight v5 Layer 1 visual foundation. Next Architect doc.
> - **MIG-025** = Sight v5 Layer 2 diagnostic.
> - **MIG-026** = Sight v5 Layer 3 recommendation (V3-§7.b llama.cpp wiring lands here as a sub-phase).
> - **MIG-027** = Sight v5 Layer 4 coaching.
> - **Cleanup MIG (TBD)** retires Sight v2 / v3 / v4 components and `lenses.rs::apply_lens` once v5 stable across multiple sessions.
>
> ### MIG-022 §B.5 contradicted-and-deferred
>
> The original MIG-022 Plan §B.5 was *"UI surface per D-B4.β: Sight v3 overlay"*. With Sight v3 explicitly retired by Concept Paper v2.0/v3.1 and Eisa's directive *"we have to focus on this version and avoid wasting our time and effort on patching earlier versions,"* §B.5 is contradicted by the canonical Sight target and **deferred indefinitely**. The temporal-axis history Rust foundation (§B.1–§B.4) shipped and is reusable; the natural consumer is now Sight v5 Layer 2's "growth trajectory" diagnostic within MIG-025.
>
> §B.6 (i18n + help + UM for the §B.5 UI) is also deferred since §B.5 is what would have driven its content.
>
> No PJ-NNN allocated for this deferral — the Rust foundation is in `note_state_history` waiting to be consumed; when Layer 2 designs against it, that consumption happens naturally inside MIG-025.
>
> ### New PJ filed
>
> - **PJ-051 — Mock B1 SVG follow-up edits as Sight v5 evolves.** The Mock B1 was updated 2026-05-12 to add the 7th button (P) per Concept Paper v3.1 §6. Future v3.x revisions of the Concept Paper that touch the visual contract (e.g., if a different mode order is decided, if the legend wording changes, if a Layer 2 "Findings" pill is added to the dome) will need parallel edits to `docs/Sight-vNext-MockB1-Toggle.svg`. Original 6-button version preserved at `Sight-vNext-MockB1-Toggle-v1.svg`. P3 housekeeping; fires when triggered, not on a schedule.
>
> ### What did NOT close in v1.11
>
> - PJ-044 / PJ-046 / PJ-047 / PJ-048 / PJ-049 / PJ-050 — all still open (the 6 polish items from MIG-022 §A Gate 3). Candidates for §N close-out cleanup or future polish MIG.
> - PJ-005 — MIG-007: Links Settings tab — still open.
> - PJ-002 — cid_cn collision scrub — still open.
> - PJ-008 — Outgoing Links typed-link dedupe — still open.
>
> ### What did close (recap from prior versions)
>
> 5 PJs closed during MIG-022 cascade (per v1.10): PJ-040 (in §D), PJ-041 (§E.2.b), PJ-042 (§E.1), PJ-043 (§E.3.d), PJ-045 (inline in §E.2.a).
>
> ### Top of queue rotates
>
> 1. **MIG-024** — Sight v5 visual foundation Architect doc (next-up).
> 2. MIG-022 §N — final integration audit + close-out (parallel-ready).
> 3. MIG-023 — Warrant Research Concept Paper (after MIG-022 §N closes).
> 4. PJ-005 — MIG-007: Links Settings tab.
> 5. PJ-044 / 046 / 047 / 048 / 049 / 050 — MIG-022 polish backlog.
>
> **Done count after v1.11**: 7 (unchanged — PJ-038 stays In-Progress umbrella; the gap-analysis-response work shipped under MIG-022 isn't on the PJ ledger). **Cancelled**: 1. **Rejected**: 1.

> **What changed in v1.10** (MIG-022 §A closes Gate 3 PASS; 5 PJs closed during MIG-022 §0+D+E+A; 7 new PJs filed during the cascade):
>
> ### MIG-022 Phase 3 Build first-half COMPLETE
>
> §0 (cleanup) + §D (PJ-040 fix) + §E (full i18n) + §A (full YAML metadata + supersedes typed-link + ikhtilāf widget per D-A4.α + Epistemic Metadata help topic + UM chapter in 15 locales) all shipped. Boss-Test Gates 1 + 2 + 3 all PASS. ~30 commits across the cascade including 4 Boss-test catches landed inline.
>
> ### Closed during the §0+D+E+A cascade
>
> - **PJ-040 — Done** (closed in §D `c072700`): UA partial-frontmatter short-circuit refactored to per-axis dispatch; both SOURCES and CONTENT TYPE sections now appear on cards with partial frontmatter.
> - **PJ-041 — Done** (closed in §E.2.b `81fba1a`): cataloger reasoning prose i18n via `(template_key, params)` tuples + 15-locale backfill; non-en/non-ar users see localized reasoning prose on Source Review cards.
> - **PJ-042 — Done** (closed in §E.1 `6c1c3ae`): Confidence enum i18n via `cece.confidence.*` keys + `confidenceLabel()` helper; `[high]` chips now translate.
> - **PJ-043 — Done** (closed in §E.3.d `b9f1ab2`): taxonomy node labels backfilled to 15 locales (277 nodes × 13 = ~3,600 translations); Source Review SOURCES/CONTENT TYPE values render in active locale.
> - **PJ-045 — Done** (closed inline in §E.2.a `894b114`): composite_reasoning paren-dedup; "Set by user in frontmatter (Set in note frontmatter (manual). Sources: ...)." → clean "Set by user in frontmatter." or axis-specific partial message via the new compose template scheme.
>
> ### New PJs filed during the cascade
>
> Six new PJs surfaced from Boss-tests across Gates 1+2+3. None blocked the cascade; all candidates for §N close-out cleanup or future polish MIGs:
>
> - **PJ-044** — Right-click "Classify Sources" menu entry missing in NotePane (Gate 1 catch; Eisa workaround: "Classify open note" button on Source Review header invokes the same IPC). P3 polish.
> - **PJ-046** — Properties panel reorder drag broken (Gate 3 Stage 1 side note). P3 polish.
> - **PJ-047** — Typed-link editor colors visually indistinguishable at body font size (Gate 3 Stage 3). The 9 typed-link colors render correctly per CSS but blend visually at small font. Pre-existing across all 9 types. P3 polish — candidates: bump saturation, add per-type icon, render as actual badge pills instead of colored text.
> - **PJ-048** — Type picker dropdown clips longer non-English translations (Gate 3 Stage 4). Pre-existing fixed-width design clips Arabic/etc. type names to first 2-3 characters; only the longest option (the new "Multi-row list") wraps to 3 lines and shows fully. P3 polish — needs `min-width: max-content` or per-locale width adjustment.
> - **PJ-049** — In-app Help viewer not implemented (Gate 3 Stage 5). Help topic + User Manual files exist on disk in 15 locales (created via §A.4.a + §A.4.b) but no UI access surface — no F1 binding, no command palette entry, no ? menu. Future MIG could ship a help-modal browser or wire the topic catalog into the command palette. P2.
> - **PJ-050** — Backlinks panel section header hardcoded English in non-en locales (Gate 3 Stage 4.1 spotted in Spanish UI). P3 polish.
>
> ### Next-up: §B + §N + MIG-023
>
> - **§B** — Temporal axis (note_state_history table + Sight v3 overlay per D-B4.β). Multi-week. Boss-Test Gate 4 fires after §B.6.
> - **§N** — Final integration audit (3-agent like V3-§11) + MIG-022 ships commit + orientation v-bump.
> - **MIG-023** — Constellation Warrant Research workstream (per Eisa's D-C1 commitment; separate Concept Paper + multi-month research project).
>
> **What changed in v1.9** (V3-§10 cascade closed Gate 3 PASS; three deeper i18n gaps filed for the MIG-022 cascade):
>
> - **MIG-021v3 V3-§10 closed Gate 3 PASS today** — full Option C cascade (Settings UI + en/ar i18n + EN docs + 13-locale i18n backfill + 14-locale help topic + 14-locale User Manual chapter) shipped across 9 commits (`d44b115`, `0054981`, `34a96a9`, `259c333`, `7d6e1a0`, `50a67b0`, `a4438ac`, `4ede8ef`, `54276c3`). Two Boss-test catches landed inline: **V3-§10.A.1** (`4ede8ef` — on-save IPC needed to dispatch the `constellation:classify-and-show` event for the Source Review panel to refresh) and **V3-§10.D.2** (`54276c3` — `settings.classifier.*` block was missing from 13 non-en/non-ar locales).
> - **Three new PJs filed from Gate 3 Stage 5 review**: **PJ-041** (cataloger reasoning prose hardcoded English), **PJ-042** (self_reported_confidence enum bypasses i18n), **PJ-043** (taxonomy node labels en+ar only). All three are structural i18n gaps in the engine output, not just missing translation entries — they belong in the MIG-022 cascade.
> - **Next-up: V3-§11** (final integration audit + MIG-021v3 entire close-out), then **MIG-022** Architect doc (response to the gap analysis from `docs/epistemic-content-gap-analysis.md` + the three new PJs from this version).
>
> **What changed in v1.8** (V3-§9 vertical-axis activation cascade closed; one new PJ filed from Gate 2 observation):
>
> - **MIG-021v3 V3-§9 closed Gate 2 PASS today** — vertical-axis activation cascade A→E + V3-§9.C.2 dual-axis reliability fix shipped (commits `4e0981a`, `d9dfa60`, `ec5527e`, `b18a3ee`, `bf07ae1`, `75807a3`). Cumulative test count went from 67 (start of V3-§9) to 92 (now). Two Boss-test catches (V3-§9.A's ال-prefix gap + V3-§9.C.2's dual-axis reliability gap) made it through pre-commit validation; both fixed inline. See `lab/reports/MIG-021v3-V3-§9-VERTICAL-ACTIVATION-ARCHITECT.md` and `MIG-021v3-V3-§9-VERTICAL-ACTIVATION-PLAN.md`.
> - **New PJ-040 filed**: UA-short-circuit on partial frontmatter (only one axis populated) discards the OTHER catalogers' votes on the unfilled axis. Observed during Gate 2 Stage 6 on `الخط العربي` (frontmatter has `sources:` but not `content_type:` → CONTENT TYPE section vanishes from card). Architectural decision worth surfacing for future improvement; not a regression — same behavior since V3-§1.
> - **Next-up: V3-§10** — Settings + i18n + Help docs + User Manual for CECE chrome.
>
> **What changed in v1.7** (same day as v1.6; MIG-018 ships v3 projection foundation):
>
> **MIG-018 (PJ-038 phase 1 of 3) closes Done** — Sight v3 projection foundation live in production. Six-phase cascade (§1A → §1F) shipped today. Boss test passed all 11 steps. Three-agent audit CLEAN (0 P0/P1/P2/P3). `SIGHT_V3_ENABLED = true` committed. See `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-AUDIT.md`.
>
> **PJ-038 status update**: Confirmed → **In-Progress** (1 of 3 MIGs complete).
>
> **MIG-019 (next-up)** — phase 2 of 3 in v3 trajectory:
> - PJ-035 (content-similarity TF-IDF edges → Milky Way band)
> - Calendar rim (Gregorian default + user-add via Settings)
> - Universe-health card in side panel
> - Full search integration (flares + halos)
> - Always-on labels Settings toggle
>
> **Top of queue rotates**:
> 1. **MIG-019** — Sight v3 phase 2 (next-up)
> 2. PJ-005 — MIG-007: Links Settings tab
> 3. PJ-002 — cid_cn collision scrub
> 4. PJ-008 — Outgoing Links typed-link dedupe
>
> **Done count after v1.7**: 7 (unchanged — PJ-038 stays In-Progress until all three MIGs close). **Cancelled**: 1 (PJ-034). **Rejected**: 1 (PJ-037).
>
> **No new PJ-NNN allocated** in v1.7.

**Version 1.6 | 2026-05-07**

> **What changed in v1.6** (same day as v1.5; PJ-038 Sight v3 Concept Paper ratified, PJ-037 rejected):
>
> **PJ-038 Concept Paper v1.1** (`docs/Constellation-Sight-v3-Concept-Paper-v1.1.md`) ratified by Eisa 2026-05-07 with all ten v1.0 §11 questions resolved + two structural revisions (faint-lines-at-rest replaces v2's strict reveal-on-demand; Map↔Sight integration rejected).
>
> **Closed in v1.6:**
> - **PJ-037** (Map ↔ Sight integration) — **Rejected** by Eisa 2026-05-07: *"There won't be Map-Sight integration."* Number retired per stable-reference-numbers rule. Sight v3 stays single-view; Map and Sight remain independent surfaces.
>
> **PJ-038 reframed**: PJ-037 absorption removed from §8 trajectory; MIG-020 phase reduced to PJ-036 (layer peeling) + v2 retire only. Three MIGs (MIG-018 / MIG-019 / MIG-020) instead of three-with-bonus-fourth-feature.
>
> **Top of queue stays**:
> 1. **PJ-038 — Sight v3 build with own dedicated Concept Paper** (now ratified at v1.1; ready for MIG-018 Architect).
> 2. PJ-005 — MIG-007: Links Settings tab.
> 3. PJ-002 — cid_cn collision scrub.
> 4. PJ-008 — Outgoing Links typed-link dedupe.
>
> **Done count after v1.6**: 7 (unchanged). **Cancelled count**: 1 (PJ-034). **Rejected count**: 1 (PJ-037 — new).
>
> **No new PJ-NNN allocated** in v1.6.

**Version 1.5 | 2026-05-07**

> **What changed in v1.5** (same day as v1.4; MIG-017 closes — v2 Sight disabled cleanly):
>
> **Closed in v1.5:**
> - **PJ-039** (MIG-017: Disable v2 Sight) — **Done.** Single phase, single commit. `SIGHT_V2_ENABLED = false` const in new `src/lib/sight/engine.ts` module gates four UI surfaces (dock button, modal mount, Return-to-Lens button, Settings plugin entry) + a banner on the Sight help doc. v2 component + IPCs preserved on disk. Three-agent audit clean (0 P0, 0 P1). Audit at `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`.
>
> **Top of queue rotates:**
> 1. **PJ-038 — Sight v3 build with own dedicated Concept Paper** (next-up; multi-MIG, star-chart aesthetic).
> 2. PJ-005 — MIG-007: Links Settings tab.
> 3. PJ-002 — cid_cn collision scrub.
> 4. PJ-008 — Outgoing Links typed-link dedupe.
>
> **Done count after v1.5**: 7 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027, PJ-039). **Cancelled count**: 1 (PJ-034 — partial-shipped).
>
> **No new PJ-NNN allocated** in v1.5. The MIG-017 cycle was Architect → Plan → Build → Audit → Done in one session — the closure is the value, no spin-off backlog.

**Version 1.4 | 2026-05-07**

> **What changed in v1.4** (Boss-directed 2026-05-07; closes the MIG-016 cycle and frames the Sight v3 trajectory):
>
> **Closed in v1.4:**
> - **PJ-034** (MIG-016: Sight instant-toggle perf) — **Cancelled (partial-shipped)**. §1A instrumentation + §1B edges-on-hover gate shipped (commits `a0babbb` → `7e76b17` → `62718f7`). §1C (Web Worker offload), §1D (post-paint prewarm), §1E (SQLite `sight_cache`) **abandoned mid-flight** because v2 Sight is being disabled under MIG-017 (PJ-039) as a known-good fallback while v3 is built fresh. Original goal — "instant first-toggle on a 30k-edge universe" — not met for v2; designed-in for v3 from the start. Audit close-out at `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`.
>
> **New PJs allocated:**
> - **PJ-035** — Sight content-similarity TF-IDF edges (the InfraNodus-defining mechanic; not in v2; **inheritable into v3**).
> - **PJ-036** — Sight layer peeling (hide top-N centrality nodes and recompute; not in v2; **inheritable into v3**).
> - **PJ-037** — Map ↔ Sight integration (cross-surface filtering and selection sync; **inheritable into v3**).
> - **PJ-038** — **Sight v3 build with own dedicated Concept Paper.** Multi-MIG. Star-chart aesthetic per Boss's design north star (Suwaidi northern-hemisphere chart reference). Inherits the Rust analytics from v2; rebuilds the visualization layer entirely.
> - **PJ-039** — **MIG-017: Disable v2 Sight.** Mini-MIG, single session. Hides v2 Sight's user-visible surface (dock button, modal, Settings entry) while preserving the v2 Svelte component and IPCs as a known-good fallback. Precondition for PJ-038.
>
> **Top of queue rotates** (PJ-039 + PJ-038 are the current Sight track; PJ-005 / PJ-002 / PJ-008 carry over from v1.3 as the non-Sight queue):
> 1. PJ-039 (MIG-017) — disable v2 Sight (next-up, mini-MIG)
> 2. PJ-038 — Sight v3 build with own Concept Paper (after MIG-017)
> 3. PJ-005 — MIG-007: Links Settings tab
> 4. PJ-002 — cid_cn collision scrub
> 5. PJ-008 — Outgoing Links typed-link dedupe
>
> **Done count after v1.4**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027). **Cancelled count**: 1 (PJ-034 — partial-shipped).
>
> **New papers landed alongside this version**:
> - `docs/Constellation-Sight-Concept-Paper-v1.1.md` — markdown port of Eisa's April 2026 v1.0 PDF, refreshed with truth-status, Principle 6, and v3 forward-look.
> - `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` — scope-narrowed audit for the partial-shipped MIG-016.

**Version 1.3 | 2026-05-06**

> **What changed in v1.3** (same day as v1.2; deeper cross-check after PJ-006 catch): the v1.2 cross-check agent missed **PJ-006 — Living Link Architecture P2–P5** because my own instructions told it to read only the "What changed in vX.Y" preambles, not orientation BODIES. Orientation v1.40 §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since 2026-05-05. PJ-006 was already done.
>
> **Eisa's response**: codified as **Standing Order #8** in CLAUDE.md (also memory feedback note `feedback_pj_crosscheck_before_tackle.md`): cross-check any PJ before tackling — read orientation BODIES (§4.x subsystem sections) and session logs, not just preambles. Then re-ran the cross-check correctly.
>
> **Outcome of the deeper cross-check** (orientation v1.49 bodies + session logs 2026-05-01 → 2026-05-06):
> - **1 entry flipped to SHIPPED**: PJ-006 (Living Link P2–P5).
> - **All 27 other entries confirmed unchanged** from v1.2 status. No further stale entries.
> - **No new PJ-NNN allocations needed**.
> - **Scope rewrites already correct** in v1.2 (PJ-010, PJ-014, PJ-021 stay as written).
>
> **Top of queue rotates**: PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub) → PJ-008 (Outgoing Links dedupe).
>
> **Done count after v1.3**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027).

**Version 1.2 | 2026-05-06**

> **What changed in v1.2** (same day as v1.1; cross-check audit + v2 batch close): Eisa-directed cross-check of v1.1 against orientation v1.0 → v1.47 to verify which jobs are still applicable.
>
> **Closed in v1.2 cycle:**
> - **PJ-001** (chunk v2 sentinel migration) — SHIPPED via MIG-015 (commits `0ca7e64` → `877e46e`).
> - **PJ-007** (Note-stage taxonomy) — SHIPPED via MIG-014 (commits `c3b9454` → `339d65b`).
>
> **Marked OBSOLETE in v1.2** (already addressed; numbers retired per stable-reference-numbers rule):
> - **PJ-025** (Sight dashboard) — Sight is on-demand via `toggleLens()` in `+layout.svelte:3354`, not boot-rebuilt. Cached after first compute. The "rebuilds on every boot" framing was incorrect.
> - **PJ-026** (sidebar star counts) — `loadAllStats` is cache-fast (Rust per-library parallelism, fire-and-forget per `+layout.svelte:1939-1941`). Not a boot-blocking gap.
> - **PJ-027** (Map) — `src-tauri/src/map.rs:300` documents the Map data is "maintained by triggers on note-save". Already write-time derived.
>
> **Scope rewritten in v1.2:**
> - **PJ-010** (Unlinked Mentions) — narrowed to the frontmatter alias-bleed half (the "double-count typed-link" half was already fixed in v1.5 §90's `scan_unlinked_mentions` rewrite that skips ALL wikilink forms before plain-text scanning).
> - **PJ-014** (13-locale User Manual backfill) — reflected that MIG-014 + MIG-015 shipped all 15 locales upfront for their new strings; remaining queue is the User Manual / help-doc body content (Stages section §18.6, Cognitive Engine help, etc.) that needs translation in the 13 non-en/ar locales.
> - **PJ-021** (Sky View persistence) — narrowed: `sky_backfill.rs` and `cache_boot_snapshot_sky` already provide partial persistence; the gap is whether the full Rule 8 audit (write-time triggers on every note_meta / note_links change) is complete. Verify-then-narrow.
>
> **New PJ-028 → PJ-033** allocated to the MIG-014 §2F audit P2/P3 follow-ups (six edge-case items from `project_mig014_audit_p2_p3_followups.md`).

**Version 1.1 | 2026-05-06**

> **What changed in v1.1** (Boss-directed 2026-05-06): elevates the **Stable Reference Numbers** rule from the Appendix to the front of the doc — `PJ-NNN` IDs are reference numbers used across session logs, commit messages, and cross-doc references; they are never reused, never recycled, and never renumbered, even when a job is rejected, cancelled, merged into another, or split into siblings. Adds **Rejected** and **Cancelled** as explicit terminal statuses (alongside **Done**) that retire a number permanently with the entry preserved. Updates **PJ-007** from `Open · Boss design call` to `Confirmed · In-Progress` with the chosen baseline (Living Link 6-stage) and the proposed-defaults all approved. PJ-007's closure plan is now a focused MIG; PJ-006 P3 is unblocked by this confirmation.

**Version 1.0 | 2026-05-05**

> **What this is.** A durable, versioned project backlog. Every open job that isn't actively shipping right now lives here. The list is reviewed at the start of every work session and updated whenever a job opens, closes, or changes priority. Like the orientation and Laws docs, this file is versioned: a new version (`v1.1`, `v1.2`, …) is written as a NEW file alongside the previous one whenever the structure changes (new job added / existing job moves status / batch of jobs closes). Older versions stay as historical record so the trail of decisions is durable.
>
> **What this is NOT.** It is not the session log. The session log records what *happened today*. This doc records what's *open across the project*. Jobs flow from this doc into a session log entry when work begins; back into this doc as Done when work closes.
>
> **Audience.** Primary: every future Claude session. Secondary: the Boss reviewing what's outstanding. Tertiary: any future contributor.
>
> ### Stable Reference Numbers (foundational rule, v1.1)
>
> Each job has a stable `PJ-NNN` identifier that **acts as its permanent reference number** — like a ticket ID. The rules:
>
> - **Numbers are unique and never repeated.** PJ-007 means PJ-007 forever. If PJ-007 is rejected, cancelled, or merged into another job, **the number PJ-007 is retired with its entry**; no future job ever reuses it.
> - **Renumbering is forbidden.** When jobs close, are rejected, or split, their numbers stay where they are. New jobs always take the **next unused number** (PJ-028, PJ-029, …), regardless of what's happened to earlier ones.
> - **Splitting preserves the parent number with sibling suffixes.** If PJ-006 splits into PJ-006a and PJ-006b, the parent PJ-006 stays in the doc as a header pointing at its children. PJ-006 itself never disappears.
> - **Merging redirects.** If PJ-008 + PJ-009 + PJ-010 merge into one MIG, all three numbers stay in the doc; two of them point at the surviving entry with `merged into PJ-NNN`.
> - **Why this matters.** Session logs cite jobs by number. Commit messages cite jobs by number. The Pending Jobs doc itself cross-references by number (e.g. PJ-006 depends on PJ-007). If numbers got reused, every historical reference would silently break.
>
> ### Status vocabulary
>
> | Status | Meaning | Number behavior |
> |---|---|---|
> | **Open** | Not yet started; queued for future work | active |
> | **In-Progress** | Work has started; tracked in current session log | active |
> | **Confirmed** | Boss has decided the design / scope; ready to start | active |
> | **Blocked** | Cannot proceed until a dependency lands | active |
> | **Deferred** | Decided not to ship now; revisit later | active |
> | **On-hold** | Conditional on a future trigger (e.g. user feedback) | active |
> | **Done** | Shipped; commit hash recorded | retired (number reserved) |
> | **Rejected** | Boss decided not to do this | retired (number reserved) |
> | **Cancelled** | Started but abandoned; entry kept for the record | retired (number reserved) |
> | **Cancelled (partial-shipped)** | Started, some phases shipped, remainder abandoned (added in v1.4) | retired (number reserved) |
> | **Merged** | Folded into another job; the surviving job's number is referenced | retired (number reserved, points to survivor) |
>
> All terminal statuses (Done, Rejected, Cancelled, Merged) move the entry to **§7 Done** but the number stays referenced from its original spot via a one-line stub if useful for navigation.

---

## Quick reference — top of the queue

The first five rows are what's queued to start *next*; the rest are sequenced by priority within their category.

| ID | Job | Status | Severity | Effort |
|---|---|---|---|---|
| PJ-060 | `index_note` cache-hit short-circuit fix (write-time-derivation blocker) | Open | **P1 (highest leverage)** | Mini-MIG |
| PJ-005 | MIG-007: Links Settings tab | Open | P1 | Single MIG |
| PJ-063 | `note_links.link_type` globally `'relates'` (re-verify under MIG-067) | Open | P1 | /migration |
| PJ-002 | Pre-§140 `cid_cn` collision scrub utility | Open | P1 | Mini-MIG |
| PJ-003 | Rename-collision popup (Override / Rename / Cancel) | Open | P1 | Mini-MIG |

---

## §1 · Mini-MIG candidates (focused, 1–3 days each)

### PJ-001 — MIG-013 P1-M1: chunk the v2 sentinel migration with progress UI

**Status.** **SHIPPED 2026-05-06 via MIG-015** · **Severity.** P1 · **Effort.** Mini-MIG (4 phases)

The MIG-013 §1E audit (`lab/reports/MIG-013-CTSE-AUDIT.md` §3) found that the v2 bigram-sentinel migration's bulk UPDATE blocks boot for 30–90 sec on pre-MIG-013 DBs (~5.7M bigram rows) with zero user feedback. Boss's library has already migrated, but new pre-MIG-013 backups would hit it once.

**What shipped (MIG-015 §1A → §1D)**: the migration moved off the boot critical path. `init_db` only detects pending; a worker thread spawned from `ensure_search_db_ready` runs the chunked migration (100,000 rows per chunk) with the DB mutex acquired+dropped per chunk + a 10ms yield between chunks so other IPC callers can interleave. Tauri event channel `migration:term_vocab_v2` emits start/progress/done phases. Frontend `MigrationProgressStrip.svelte` listens and renders a status-bar strip in a new `.sb-center` group: `Migrating term index — N / M`, then `Term index migration complete`, hidden 4 seconds later. i18n covers all 15 locales upfront.

**Acceptance**: ALL met.
- ✅ Boot proceeds to first paint without waiting on the migration.
- ✅ Status-bar strip shows running counts; hides 4 sec after `done`.
- ✅ Crash-recoverable by construction (WHERE clause is the resume marker).
- ✅ 15 locales translated.
- ✅ Three-agent audit clean after one P0 fix (DB mutex held across loop → split per chunk).

**Visual Boss test skipped** per Eisa's directive: Boss's library is already at v2 from earlier MIG-013 testing, and rolling back to manufacture migration work would touch closed-feature production data (Index closed 2026-05-04 per session log; Working Agreement #4 forbids "let's see what happens" on closed-feature data). Static audit verifies behaviour; future users with pre-MIG-013 backups will exercise the visible path naturally.

**Closed-out commit chain**: `0ca7e64` (§1A) → `df0bf87` (§1B) → `62d3b4a` (§1C) → close-out commit (§1D + P0 fix + audit + orientation v1.47).

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §3 (original deferred); `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`; `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md`; `lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md`.

---

### PJ-002 — Pre-§140 `cid_cn` collision scrub utility

**Status.** Open · **Severity.** P1 · **Effort.** Mini-MIG

One-time fix for libraries with corrupted `cid_cn` values from before MIG-003 §140's hardening. Boss self-healed his own affected note (Hub v6) by delete + recreate. Users restoring old backups would still hit the collision silently.

**Source.** Orientation v1.30, session log 2026-05-03.

**Acceptance.** A boot-time scan walks all `note_meta` rows; any duplicate `cid_cn` triggers a status-bar prompt offering "Re-canonicalize duplicates" with a preview list. Run is opt-in, idempotent, and logs every change to a session-scoped report.

---

### PJ-003 — Rename-collision popup (Override / Rename / Cancel)

**Status.** Open · **Severity.** P1 · **Effort.** Mini-MIG

Today `create_note` / `rename_item` silently refuse a rename when the target filename already exists. Boss expects a system-style dialog with three actions: Override the existing file, Rename to something else, or Cancel.

**Source.** `project_rename_collision_popup_wanted.md` (logged 2026-04-28 from MIG-003 Stage 5 test).

**Acceptance.** When `rename_item` hits a collision, frontend shows a `ConfirmDialog`-style popup with three buttons. Override copies properties from the renamed file before deleting the existing one (preserves `cid_cn`). Rename re-prompts for a new name. Cancel dismisses. Localized in en + ar; placeholders in 13 others.

---

### PJ-004 — NSIS bundling lock workaround

**Status.** Open · **Severity.** P2 · **Effort.** Investigation + small fix

Recurring `os error 32` when Constellation is running during `npm run tauri build`. The NSIS bundle stage tries to write `Constellation_X.Y.Z_x64-setup.exe` into a directory that the running binary holds open. MSI succeeds; NSIS doesn't. We work around by using the MSI; a real fix would let CI bundle reliably.

**Source.** Orientation v1.30, hit again on every build during MIG-013.

**Acceptance.** `npm run tauri build` produces both MSI and NSIS bundles cleanly even when an old binary is running. Likely fix: change the NSIS output path or add a kill-running-instance pre-build hook.

---

### PJ-039 — MIG-017: Disable v2 Sight

**Status.** **Done — 2026-05-07** · **Severity.** P1 · **Effort.** Mini-MIG (single session, single commit)

Disabled v2 Sight (`ConstellationSight2.svelte` + the v2 dock button + the v2 modal + the Settings plugin entry) as a **known-good fallback** while v3 is built fresh under PJ-038. The Rust analytics IPCs (`constellation_sight_centrality`, `constellation_sight_communities`, etc.) and the v2 Svelte component are **kept on disk** — they are the proven baseline if v3 fails.

**What shipped:**
- New module `src/lib/sight/engine.ts` exporting `SIGHT_V2_ENABLED = false`.
- Four UI gates added (`SIGHT_V2_ENABLED && ...`):
  - Dock button at `+layout.svelte:4361`.
  - Modal mount at `+layout.svelte:4993-4994`.
  - "Return to Lens" button at `+layout.svelte:4741`.
  - Settings plugin entry at `SettingsModal.svelte:270`.
- Banner block prepended to `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` — original v2 documentation preserved beneath.

**Why a code constant, not a Settings flag**: the fallback is a *codebase* fallback (developer flips one const + rebuilds), not a user-facing toggle. A Settings flag would have required a one-time migration to flip existing users' saved `enabledFeatures.constellationSight: true` to `false`. The const-based gate wins regardless of saved state — zero churn.

**Acceptance**: ALL met.
- ✅ v2 Sight unreachable from the running app's UI in default config.
- ✅ v2 component code + IPCs + Rust analytics modules preserved on disk.
- ✅ One-edit re-enable (`SIGHT_V2_ENABLED = true` + rebuild) restores v2 behaviour identically.

---

### PJ-040 — UA-short-circuit on partial frontmatter discards other catalogers' votes on the unfilled axis

**Status.** **DONE 2026-05-11** in MIG-022 §D commit `c072700` (refactored to per-axis dispatch; both SOURCES and CONTENT TYPE sections now appear on cards with partial frontmatter). Body entry retained as historical record. · **Severity.** P2 · **Effort.** Single PR (~1 commit) · **Filed.** 2026-05-11 (Gate 2 Stage 6 observation)

When the User-Authority cataloger short-circuits the synthesis (because frontmatter has at least one axis populated), `user_authority_short_circuit` in `src-tauri/src/cece/synthesis.rs:120-167` produces an `AxisDecision` for BOTH axes hardcoded to `regime: ConfidenceRegime::Unanimous`, with `primary` taken from UA's per-axis vote. If UA only voiced on horizontal (frontmatter has `sources:` but not `content_type:`), the vertical axis gets `primary: None` and `regime: Unanimous` — vacuously settled. The result: the OTHER catalogers' vertical-axis votes (Linguistic + Structural + Semantic + Graph) are discarded entirely; the suggestion record has no vertical entry; the Source Review card renders with NO CONTENT TYPE section.

**Reproducer.** Boss-test 2026-05-11 Gate 2 Stage 6: re-classify `الخط العربي` (which has `sources: testimony/authoritative` from a previous Stage 2.1 disambig pick but no `content_type:`). The card lands in the queue showing only SOURCES + Authoritative testimony 100% + Accept/Edit/Reject — no vertical suggestion at all, even though Linguistic + Structural + Semantic all voiced on vertical with high-confidence votes (the body has 30K characters of Quranic vocabulary that fires multiple V3-§9.A vertical lexicon entries + V3-§9.B structural detectors).

**Why this is wrong.** UA's authority is per-axis. When the user has set ONLY `sources:` in frontmatter, UA should short-circuit ONLY the horizontal axis; the vertical axis should still be synthesized normally from the other catalogers. Current behavior loses information the catalogers already produced.

**Why this isn't a regression.** The behavior has been the same since V3-§1's `user_authority_short_circuit` shipped. It only became visible after V3-§9.A populated vertical-axis lexicon entries that would now fire meaningfully on Arabic content.

**Proposed fix.** Refactor `user_authority_short_circuit` to short-circuit only the axes UA voiced on. For the unfilled axis, fall through to `vote_on_axis` (the normal weighted-vote path) using the OTHER catalogers' trails. Synthesis method label can become `"user_authority_partial_short_circuit"` when only one axis was UA-voiced, distinct from full-circuit when both axes had frontmatter values.

**Acceptance.** When a card has frontmatter on horizontal only:
- Horizontal axis still short-circuits to UA's pick (existing behavior preserved)
- Vertical axis is synthesized from Linguistic + Structural + Semantic + Graph votes per the normal weighted-vote path
- The card renders with BOTH SOURCES (the UA-pick) AND CONTENT TYPE (the normally-synthesized vertical primary) sections
- Symmetric for vertical-only frontmatter: vertical short-circuits, horizontal synthesizes normally
- Existing tests (`user_authority_short_circuits` etc.) updated to cover both axes
- New regression test: `partial_frontmatter_synthesizes_unfilled_axis_normally`

**Source.** Boss-test 2026-05-11 Gate 2 Stage 6 observation (orientation v1.91 + session log).

---

### PJ-041 — Cataloger reasoning prose hardcoded English in Rust

**Status.** Open · **Severity.** P2 · **Effort.** Single MIG (~3-5 hrs structural + ~90 translations) · **Filed.** 2026-05-11 (Gate 3 Stage 5 observation)

The per-cataloger `reasoning: String` field — rendered in the Source Review trail as one prose sentence per voicing cataloger ("Structural patterns matched: vertical → higher-order-constructs/worldview (weight 0.75)", "Linguistic match: horizontal → revelation/recited (weight 0.85); vertical → ... CAE roots seen: روي, كون, …", "Semantic neighbor consensus: horizontal → testimony/authoritative (weight 0.50); vertical → ...") — is generated by Rust cataloger code at classification time via `format!()` with hardcoded English templates, stored verbatim in `composite_json`, and rendered as-is by the frontend via `{t.reasoning}`. **The string never goes through `$t()`** — V3-§10.D's i18n backfill couldn't translate it because there's no i18n key to translate.

Visible in any Source Review card after switching the interface language to a non-English locale. Boss-test 2026-05-11 Gate 3 Stage 5 surfaced this on an Arabic-UI screenshot showing reasoning prose rendered in English while the chrome around it (toggle text, pill labels, rule chips) was correctly Arabic.

**Files affected:**
- `src-tauri/src/cece/catalogers/structural.rs::build_reasoning()` (line ~298)
- `src-tauri/src/cece/catalogers/linguistic.rs` (the `reasoning` field assignment in `classify()`)
- `src-tauri/src/cece/catalogers/semantic.rs` (similar)
- `src-tauri/src/cece/catalogers/graph.rs` (similar)
- `src-tauri/src/cece/catalogers/user_authority.rs` (similar)
- `src-tauri/src/cece/catalogers/reasoning.rs` (when LLM eventually wired)
- `src-tauri/src/cece/synthesis.rs::user_authority_short_circuit` (`composite_reasoning: format!("Set by user in frontmatter ({}).", ua.reasoning)`)

**Proposed fix.** Refactor each cataloger's `reasoning: String` to emit a structured `(template_key: String, params: HashMap<String, String>)` tuple instead. Frontend renders via `$t(template_key).replace('{...}', value)` at display time. Templates live in `cece.reasoning.*` i18n block (~6 templates per cataloger × 5 voicing catalogers = ~30 templates × 15 locales = ~450 translations). Backward-compat layer: legacy v3-era cards with prose-string `reasoning` fields render the raw string as a fallback.

**Acceptance.** When the user switches to any non-English locale, the per-cataloger reasoning prose in the Source Review trail renders in that locale (or with the locale's translations of the templates the catalogers emit). No regression on en + ar (the reference translations).

**Source.** Boss-test 2026-05-11 Gate 3 Stage 5 (orientation v1.93 + session log). Composes with PJ-042 (related — confidence enum bypasses i18n) and PJ-043 (related — taxonomy node labels en+ar only).

---

### PJ-042 — `self_reported_confidence` enum bypasses i18n

**Status.** Open · **Severity.** P3 (smallest of the three) · **Effort.** Mini-MIG (~30 min + 60 translations) · **Filed.** 2026-05-11 (Gate 3 Stage 5 observation)

The `Confidence` enum in `src-tauri/src/cece/cataloger.rs` serializes as lowercase string (`"high"` / `"medium"` / `"low"` / `"abstain"`) and the Source Review trail renders it raw via `[{t.self_reported_confidence}]` next to each cataloger's name. Visible in every per-cataloger row of every card. No `$t()` lookup.

**Proposed fix.** Add `cece.confidence.{high,medium,low,abstain}` i18n keys (en + ar populated, then 13-locale backfill via Python batch as in V3-§10.D.2). Frontend wraps:

```ts
function confidenceLabel(c: string): string {
  const k = `cece.confidence.${c}`;
  const t = $t(k);
  return t && t !== k ? t : c; // fallback to raw enum on missing key
}
```

Then trail renders `[{confidenceLabel(t.self_reported_confidence)}]`.

**Acceptance.** When the user switches to any non-English locale, the `[high]` / `[medium]` / `[low]` labels render in that locale. Pre-V3-§10 behavior preserved on en + ar.

**Source.** Same as PJ-041 (Gate 3 Stage 5 review). The smallest of the three i18n gaps; could land standalone in ~30 min.

---

### PJ-043 — Taxonomy node labels en+ar only; missing 13 other locales

**Status.** Open · **Severity.** P2 · **Effort.** Single MIG (~3300 translations across 14 locales × ~240 nodes; significant breadth, modest per-translation effort) · **Filed.** 2026-05-11 (Gate 3 Stage 5 observation)

The vertical taxonomy (`vertical_taxonomy.rs::VERTICAL_NODES`, ~225 nodes) and horizontal taxonomy (`horizontal_taxonomy.rs::HORIZONTAL_NODES`, ~30 nodes) static structs have `en: &'static str` and `ar: &'static str` fields only. Frontend's `labelForId()` reads these via the `currentLocale === 'ar'` ternary. For any other locale, the Arabic field is shown (non-Arabic users see Arabic labels) or — depending on the wrapper logic — the English field is shown.

Visible in any Source Review card for non-en/non-ar locales: the SOURCES + CONTENT TYPE list values (Perception/Sensation, Authoritative testimony, Worldview (ruʾyah kawniyyah), etc.) stay in English/Arabic regardless of the active locale. Sibling Disambiguation chips have the same issue (the candidate ID labels render English/Arabic).

**Proposed fix.** Three sub-options (decide in MIG-022 phase):
- **(a) Extend `VerticalNode`/`HorizontalNode` structs** with 13 more `&'static str` fields. Big edit (255 nodes × 13 new fields = 3315 hardcoded strings) but type-checked and zero-runtime-cost.
- **(b) Move taxonomy labels to per-locale JSON files** (`src-tauri/data/vertical_taxonomy.{locale}.json`) loaded lazily. More maintainable but adds runtime indirection.
- **(c) Move to `src/lib/i18n/{locale}.json::cece.taxonomy.*`** (frontend-only). Simplest but loses Rust-side validation.

**Acceptance.** When the user switches to any non-en/non-ar locale, the vertical + horizontal taxonomy node labels render in that locale (or fall back to English with explicit fallback signal).

**Source.** Same as PJ-041 (Gate 3 Stage 5 review). Largest of the three by translation volume; benefits from being scoped as its own MIG.

- ✅ Help doc banner shipped; existing v2 documentation untouched beneath.
- ✅ `npm run check`: 1 pre-existing PJ-012 error, 0 new errors.
- ✅ Three-agent audit clean. Audit report: `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`.

**Architect / Plan / Audit docs**:
- `lab/reports/MIG-017-DISABLE-V2-SIGHT-ARCHITECT.md`
- `lab/reports/MIG-017-DISABLE-V2-SIGHT-PLAN.md`
- `lab/reports/MIG-017-DISABLE-V2-SIGHT-AUDIT.md`

**Source.** Boss decision 2026-05-07 (recorded in `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`).

**Composes with.** PJ-038 — unblocked the moment this commits.

---

## §2 · Larger MIG candidates

### PJ-005 — MIG-007: Links Settings tab

**Status.** Open · **Severity.** P1 · **Effort.** Single MIG

Consolidate every link-related Settings control into one "Links" tab. Currently the controls are scattered: Auto-update Links toggle is misplaced under Sky View & Links (per `project_autoupdatelinks_toggle_placement.md`); the link-confidence backfill button lives in a generic Maintenance section; Living Link lifecycle preferences (when introduced) need a home.

**Source.** `project_links_settings_tab.md`, `project_autoupdatelinks_toggle_placement.md`.

**Acceptance.** A new `Links` tab in Settings that aggregates: Auto-update Links toggle, link-confidence backfill button, Living Link lifecycle decay rate (if exposed), typed-link visibility preferences, link-archival display toggle. Localized in en + ar. Old toggle locations removed without breaking deep-link bookmarks (settings-section anchors stable).

---

### PJ-006 — Living Link Architecture P2–P5 implementation

**Status.** **SHIPPED (closed in v1.3 cross-check, 2026-05-06)** · **Severity.** P1 · **Effort.** Multi-phase (delivered incrementally)

The Living Link Architecture's five implementation phases (P0 through P5) are all live and user-validated. The v1.2 entry's "Open · Multi-MIG" framing was stale — the work was done in slices over the prior weeks (CE Phase commits in the §90-§142 range), and orientation v1.40 §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since 2026-05-05.

**What's actually live (verified 2026-05-06 deeper cross-check):**

| Phase | Verified shipping |
|---|---|
| P0 | `note_links` SQLite table, `extract_typed_links`, 19,062 links indexed |
| P1 | 7 cognitive search operators in 15 languages, chips in SearchHub + Sky View |
| **P2 — Traversal tracking** | `constellation_link_traverse` IPC at `src-tauri/src/search.rs:3516`; frontend caller at `src/lib/libraries/store.ts:1094` (called on wikilink click) |
| **P3 — Weight + lifecycle + decay** | `LinkLifecycle` type in `src/lib/libraries/store.ts:1521`; `_link_decay`, `_link_dormant`, `_link_set_confidence`, `_link_backfill_confidence`, `_link_archive`/`_unarchive`/`_archived` IPCs (orientation v1.49 §4.4 line 1190 enumerates them) |
| **P4 — Formulation analysis** | `formulationAnalysis` wrapper at `src/lib/libraries/store.ts:1505` calling `constellation_formulation_analysis` IPC |
| **P5 — Knowledge health dashboard** | `src/lib/components/KnowledgeHealthDashboard.svelte`, mounted at `src/routes/+layout.svelte:5975`. Reads from P2-P4's data via `formulationAnalysis` |

**Decay formula** (display-only): `effectiveWeight = rawWeight × exp(−ln(2) × daysSinceTraversal / halfLifeDays)`. Default half-life: 60 days. (Orientation v1.49 §4.4.)

**Auto-promote on traversal**: confidence escalates `hypothesis → evidence` at traversal_count ≥3, `evidence → established` at ≥10. Manual override via right-click in Link Dashboard. (Orientation v1.49 §4.4.)

**Why this entry was missed in v1.1 / v1.2**: the v1.2 cross-check agent read only orientation "What changed in vX.Y" preambles per my instructions. Orientation §4.4 BODY says "all shipped"; the preamble of any individual version doesn't necessarily restate that. SO #8 codifies the lesson.

**Source.** `project_ce_philosophy.md`, `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`, orientation v1.40 → v1.49 §4.4 body, `lab/reports/SESSION-LOG-2026-05-*.md`.

**Acceptance.** Already met. Living Link Architecture is the canonical link-side model; PJ-007 (note-side dependency) shipped same-day via MIG-014; the two now share the lifecycle vocabulary cleanly.

---

### PJ-007 — Note-stage taxonomy: Living Link 6-stage baseline + extensible custom stages

**Status.** **SHIPPED 2026-05-06 via MIG-014 (per-note dash-encoded model)** · **Severity.** P1 · **Effort.** Single focused MIG (delivered as a 2-iteration migration: §1A–§1D flat-list iteration record, §2A–§2F shipped model)

**What actually shipped**: the **per-note dash-encoded model** from `Stages-Concept-Paper-v1.2.md` and Plan v4. Iteration 1 (§1A → §1D, flat custom-stage list with per-Universe `custom_stages: Vec<CustomStage>` + 5 IPC commands + emoji picker) was built then proven wrong in Boss test — it didn't scale (long promote chain), the matrix was wrong (Eisa: "It is allowed only one custom term"), and it broke the Single-Source-of-Truth principle (three local mirrors of the stage value drifted across surfaces).

Iteration 2 (§2A → §2F, the model that ships):

- **6 fixed lifecycle stages** form the canonical chain: spark → birth → growth → maturity → dormancy → archival.
- **Per-note custom term** as a dash suffix in the on-disk frontmatter `stage:` value (e.g. `stage: spark-concept`). No Universe-wide setting.
- **PropertyEditor combobox** is a 6-entry mode-flip dropdown: Mode A (input empty / matches a fixed name) → 6 baselines; Mode B (custom word in input or dash suffix) → 6 paired stages (`Spark-Concept`, `Birth-Concept`, …).
- **Breadcrumb promote/demote** walks the 6-baseline chain; suffix carried verbatim across the chain. Single-source-of-truth (Law 2.7) — `currentStage` is `$derived` from the prop, never a local `$state` mirror.
- **No emoji per custom term**: emoji follows the lifecycle phase.
- **Old Zettelkasten values** (`fleeting / literature / permanent / synthesis`) preserved verbatim on disk; render via `LEGACY_ZETTELKASTEN_EMOJI` for back-compat. They aren't promoteable in the new chain.

**Acceptance**: ALL met.
- ✅ Single combobox in Properties (Mode A / Mode B).
- ✅ On-disk frontmatter is the single canonical source — no Universe-level state.
- ✅ Promote/demote chain length always 6.
- ✅ User Manual + Cognitive Engine help updated (en + ar; PJ-014 queues 13 others).
- ✅ Boss tests passed: combobox + per-note scope + cross-track navigation + boundary cases (Spark/Archival).
- ✅ Three-agent audit clean (invariants / drift / migration-path), audit report at `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`.

**Generalisation produced**: Law 2.7 (Constellation Development Laws v1.4) — every first-class data property has one canonical owner; UI surfaces are subfunctions that derive, never hold their own copy. Triggered by the §2C+§2D stage-sync patch cycle (Eisa: "Enough patching").

**Closed-out commit chain**: `c3b9454` (§1A) → `8a9ab3d` (§1B) → `17bf474` (§1C) → `9973e65` (§1C.5) → `f4eef3e` (§1C.5 fix + §1D) [iteration record] → `2f58b8a` (§2A) → `59ed95c` (§2B) → `432076c` (§2C) → `2c58bda` (§2D) → `bb7a6ef` (§2C+D fix) → `e3a97a1` (Law 2.7 architectural fix) → `a50463c` (§2E) → `339d65b` (§2F closes).

**Source.** Stages Concept Paper v1.0 → v1.2; MIG-014 Plan v1 → v4; MIG-014 §2F audit report.

---

### PJ-034 — MIG-016: Sight instant-toggle perf

**Status.** **Cancelled (partial-shipped) — closed 2026-05-07** · **Severity.** P1 · **Effort.** Was scoped as a 6-phase MIG; closed early after §1B.

**What shipped (§1A + §1B):**
- **§1A — `performance.mark` instrumentation** around `toggleLens()` in `+layout.svelte` and `ConstellationSight2.svelte` mount path. Marks: `sight:rust-centrality`, `sight:louvain`, `sight:structural-gaps`, `sight:universe-health`, `sight:stratum-weighted`, `sight:top-bridges`, `sight:community-profiles`, `sight:bridge-suggestions`, `sight:toggle:total`. Initial alerts/clipboard fallback added then removed in §1B; `console.log` + `performance.mark` retained as no-op-in-production. Commits: `a0babbb` (§1A) + `7e76b17` (§1A clipboard fix).
- **§1B — Edges-on-hover gate** (Principle 6 of the Sight Concept Paper v1.1). `needsEdgeDraw = hoveredNode || selectedNode || searchActive || hoveredLink`; `focusOnly` short-circuit at the top of `drawLinks()` skips non-incident edges in O(1) per link. Drops per-frame edge iteration to zero in the resting case (no hover, no selection, no search). Pre-built `neighborMap: Map<string, Set<string>>` populated once per `buildSimData()` call. Commit: `62718f7`. Boss test: PASSED.

**What was abandoned (§1C / §1D / §1E):**
- **§1C — `sightWorker.ts` extraction** (Louvain + gaps + profiles + bridges off main thread): **Cancelled.** Wasted work on a disabled view.
- **§1D — Post-paint prewarm** (`requestIdleCallback` after first paint to cache results before user toggles): **Cancelled.** Same reason.
- **§1E — SQLite `sight_cache`** (cross-session persistence, mirroring the `sky_backfill` pattern): **Deferred to PJ-038.** v3 will compute identical analytical outputs (centrality, communities, gaps, health) and benefit from the same cross-session persistence pattern. The design knowledge from MIG-016 Plan v1 carries forward.

**Why it closes early**: Eisa's directive 2026-05-07 — "secure what's achieved, never muddle." v2 Sight is being disabled under PJ-039 (MIG-017) as a known-good fallback while v3 is built fresh under PJ-038. Continuing perf work on a view that's about to be shelved is wasted effort — except for §1E's design knowledge, which transfers to v3.

**Original goal**: "instant first-toggle on a 30k-edge universe." **Met for v2?** No (the §1A data showed mount is fast at 175-367 ms, but the toggle pipeline's compute is what would have been targeted by §1C/§1D — not addressed). **Designed-in for v3?** Yes — the star-chart aesthetic (Sight Concept Paper v1.1 §13) makes Principle 6 (reveal-on-demand) the visual default, not an add-on.

**Audit close-out**: `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` (scope-narrowed). 0 P0, 0 P1, 1 P3 logged (mousemove handler iterating `simLinks` for link-annotation hover detection; moot once v2 disabled under PJ-039).

**Source.** `lab/reports/MIG-016-ARCHITECT.md`, `lab/reports/MIG-016-PLAN-v1.md`, `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`.

**Inheritance into v3 (PJ-038)**:
- §1B's reveal-on-demand pattern → v3's constellation-line idiom (lines render only inside the focused constellation territory).
- §1E's SQLite cache design → v3's projection-position cache (the projection math is deterministic per-universe-snapshot — caching is a clean win).

---

## §3 · Bug fixes / quality

### PJ-008 — Outgoing Links panel typed-link duplication

**Status.** Open · **Severity.** P2 · **Effort.** Single-file fix

Outgoing Links panel renders typed-link aliases twice — once as a typed-link badge (e.g. `supports`) and once as a plain text row. Same root pattern as PJ-009.

**Source.** `project_outgoing_typedlink_duplication.md`.

**Acceptance.** Each typed-link target appears exactly once in the panel, with its type as a badge.

---

### PJ-009 — Backlinks panel typed-link duplication

**Status.** Open · **Severity.** P2 · **Effort.** Single-file fix

Backlinks panel duplicates source notes when the same source uses both a regular wikilink and a typed wikilink to the same target. Lunch Plan shows twice for Apple Tree Fruit (regular + supports).

**Source.** `project_backlinks_typed_link_duplication.md`.

**Acceptance.** Each source note appears once per target, with all link types accumulated as badges on the row.

---

### PJ-010 — Unlinked Mentions panel: frontmatter alias bleed (scope rewritten in v1.2)

**Status.** Open · **Severity.** P2 · **Effort.** Small refactor

**Scope rewrite — v1.2 (2026-05-06)**: the original v1.1 description bundled two issues — (a) double-counts typed-link references, and (b) canonical filenames instead of human titles. Cross-check against orientation v1.5 §90 confirmed both were fixed by the `scan_unlinked_mentions` rewrite in commit `5cf779a` ("§90 — BUG-005 fix: autosave writeNote bypassed constellation_search_reindex" cycle). That rewrite skips ALL wikilink forms (regular + embed + typed + aliased) before plain-text scanning, AND uses the canonical-filename helper for human-title display. So both v1.1 bullets are stale.

**Remaining genuine gap**: frontmatter aliases (e.g. `aliases: [Foo, Bar]`) still surface as "unlinked mentions" because the alias-bleed fix never landed. A note with `aliases: [Bar]` in its frontmatter and a body that says "Bar" appears in the Unlinked Mentions panel as a separate row, even though the alias on the SAME note's frontmatter is what's matching. Memory note `project_unlinked_mentions_alias_bleed.md` (2026-04-29) describes the case.

**Source.** `project_unlinked_mentions_alias_bleed.md`.

**Acceptance.** A note's frontmatter aliases do NOT surface as "unlinked mentions" against the same note's body (the body word should be parsed as a self-alias-match and excluded). All other panel behaviour stays as v1.5 §90 made it.

---

### PJ-011 — Constellation Map open issues

**Status.** **DORMANT (2026-06-09)** — Constellation Map was disabled in core by MIG-038 (re-categorized as a future external "Wings" plug-in). Not actionable while Map is off; re-activate this PJ if/when Map returns. · **Severity.** P2 · **Effort.** Single MIG

Three issues bundled (logged 2026-04-27):

- Performance / memory leak in the D3 sunburst rendering on large libraries.
- Tooltip shows canonical filename instead of human title (same root as PJ-010).
- Search doesn't highlight matched arcs.

**Source.** `project_constellation_map_backlog.md`.

**Acceptance.** Map renders cleanly on 7,600-note libraries (no leak across navigation). Tooltips show human titles. Search highlights matching arcs with a visible style.

---

### PJ-012 — `LinkLifecycle.fresh` TS error

**Status.** Deferred · **Severity.** P2 · **Effort.** 2-line fix

Pre-existing svelte-check error at `store.ts:2212`: `Property 'fresh' is missing in type '{emerging, established, load-bearing, stale}'`. Option B approved 2026-05-01 but deferred until post-CE: add `fresh: 1`, shift `stale: 0`, `fresh: 1`, `emerging: 2`, `established: 3`, `load-bearing: 4`. Runtime impact silent; this is a type-completeness fix.

**Source.** `project_link_lifecycle_dedupe_fix.md`.

**Acceptance.** `npm run check` produces zero errors (currently shows this one + warnings).

**Composes with.** PJ-006 P3 — LinkLifecycle taxonomy is the same vocab the Living Link lifecycle uses; fix it as part of P3 if not done sooner.

---

### PJ-013 — `lenses::apply_lens` dead-code decision

**Status.** Open · **Severity.** P2 · **Effort.** Decision + small fix

`lenses.rs::apply_lens` is dead code (zero frontend callers, verified 2026-04-27). Settings can still create + save lens definitions but they're never applied. Two paths: delete the function + the orphaned Settings UI, or re-wire it for CE Phase 9 (whatever that turns out to be).

**Source.** `project_lenses_apply_lens_dead_code.md`.

**Acceptance.** Either deleted (along with `list_lenses` / `save_lenses` if those are also unused) and the Settings lens-builder UI removed, or re-wired into a frontend consumer that exercises it.

**Composes with.** PJ-039 (MIG-017): may resolve as part of disabling v2 Sight if the orphaned Settings UI is removed at the same time.

---

## §4 · Doc backlog

### PJ-014 — 13-locale User Manual backfill (scope updated in v1.2)

**Status.** Open · **Severity.** P2 · **Effort.** Translation work

**Scope update — v1.2 (2026-05-06)**: MIG-014 + MIG-015 broke the "en+ar first, others queued" pattern by shipping all 15 locales upfront for their *new* string keys (Eisa-directed). So the i18n .json files don't lag for those two MIGs.

**Remaining queue is the User Manual / help-doc body content** that DOES lag in 13 locales:

- **MIG-014 §2E**: Stages model rewrite in `docs/User Manual.md` §18.6 (en) + `docs/help.ar/User Manual.md` §18.6 (ar) + `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` Feature 6 (en). 13 other locale User Manuals still carry the old Externalization-Engine 4-stage description.
- **Older deferrals** still queued: MIGs 008, 010, 011, 012, 013 deferred sections.

**Source.** `project_user_manual_13_locales_backfill.md`.

**Acceptance.** All 13 locale User Manuals receive the deferred sections (Stages model + earlier-MIG deferrals). Done as one batch translation pass; can be split per locale if Boss has translator capacity for a few at a time.

---

### PJ-015 — 360.3D Stratification Matrix guidance doc

**Status.** **ABANDONED 2026-05-18** (Eisa decision during post-MIG-026 state-of-standing triage) · **Severity.** P2 · **Effort.** Single doc (~2000 words)

Originally a Boss-requested teaching doc on how to read / interpret the 360.3D Stratification Matrix (Three reads — Position / Profile / Absence, mental shapes catalogue, matrix → action examples). Blocked indefinitely on 360.3D Stage 3 closure + matrix UX stabilization.

**Why abandoned**: Eisa's call 2026-05-18 — the matrix-UX dependency hasn't moved, the doc is low-leverage given other priorities, and 360.3D itself may not need a separate guidance doc. If a user-facing need surfaces later it can be filed fresh.

**Source.** `project_360_3d_matrix_guidance_doc.md` (memory note retained as historical record).

---

## §5 · Cleanup / hygiene

### PJ-016 — Drop `term_vocab.bridge_concept_id` column

**Status.** **DONE via MIG-042** (orientation v2.25 — "drop the dead `term_vocab.bridge_concept_id` column"; +2 lurking bugs found & fixed during testing). Moved to §9. · **Severity.** P2 · **Effort.** Schema migration

Dead schema after MIG-013 §1D Option B. Nothing reads or writes the column on the live CTSE path. Forward-compat preserved deliberately, but a future cleanup migration can drop it (along with the v1 / v2 schema gates and the `sentinel_bigram_rows` helper).

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2.

**Acceptance.** Schema v3 migration drops the column, the index, and all the dead helper code. M11 zero-diff invariant still holds.

**Defer.** Wait at least 2–3 sessions after MIG-013 close to confirm nothing reactivates the column.

---

### PJ-017 — Drop orphaned `term_embeddings` table on existing DBs

**Status.** Open · **Severity.** P2 · **Effort.** Schema migration

Leftover from MIG-012 on pre-MIG-013 universes. Tens of MB of dead disk per Universe. No correctness issue.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §3 Notes.

**Acceptance.** A schema migration runs `DROP TABLE IF EXISTS term_embeddings`. Bundle with PJ-016 into a single cleanup migration.

---

### PJ-018 — Drop `index.semanticSearchEnabled` settings flag

**Status.** Open · **Severity.** P2 · **Effort.** 2-line fix

Kept for forward-compat after §1D-B; zero readers in `src/`. Bundle with the rest of the MIG-013 cleanup.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2 Notes.

**Acceptance.** Removed from `DEFAULT_SETTINGS` and the `IndexSettings` interface in `store.ts`.

---

### PJ-019 — Drop `searchHub.concept` / `searchBadges.concept` i18n keys

**Status.** Open · **Severity.** P2 · **Effort.** 2-key delete × 15 locales

Kept after the SearchHub `concept` category was reverted in §1D-D; zero callers.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2 Notes.

**Acceptance.** Both keys removed from all 15 locale JSONs. Bundle with PJ-016 / PJ-017 / PJ-018 into one MIG-013 cleanup commit.

---

### PJ-020 — Optional `≈ similar` kill-switch

**Status.** On-hold · **Severity.** P3 · **Effort.** Settings toggle + gating

The CTSE `≈ similar` feature is currently always-on with no Settings toggle. Add only if Boss reports noise (irrelevant terms surfacing in the Index dropdown).

**Source.** MIG-013 close-out note.

**Acceptance.** Add a toggle to Settings → Index ("Cross-language `≈ similar` matches"). Default ON. When OFF, the `ctseSearchTermsByConcept` effect short-circuits.

**On-hold.** No action until Boss reports the feature is producing noise.

---

## §6 · Standing-Order audit — Write-Time Derivation (CLAUDE.md Rule 8)

CLAUDE.md Rule 8 ("Every computed view in Constellation is maintained at write time, not read time") explicitly names these surfaces as needing audit. Each currently rebuilds at boot or on tab focus instead of being maintained at write-time via triggers/hooks. Each is its own focused MIG.

### PJ-021 — Sky View (`skyNodes` / `skyLinks`) — scope updated in v1.2

**Status.** Open (verify-then-narrow) · **Severity.** P2 · **Effort.** Verify + targeted MIG

**Scope update — v1.2 (2026-05-06)**: cross-check found that `src-tauri/src/sky_backfill.rs` and `cache.rs::cache_boot_snapshot_sky` already provide partial persistence of `sky_nodes` / `sky_links` (since MIG-001 §136 / §142 timeframe). The original v1.1 description ("rebuilt on every boot") is partly outdated.

**Verify-then-narrow plan**:
1. Open `src-tauri/src/sky_backfill.rs` + `cache.rs::cache_boot_snapshot_sky` and confirm what's already persisted vs. what still rebuilds.
2. If full Rule 8 (write-time triggers on every `note_meta` / `note_links` change) is in place: close as Done.
3. If partial: narrow this PJ to "the gap that remains" (likely a specific trigger that's missing or an edge-case where the cache invalidates wholesale instead of incrementally).

**Acceptance.** Either confirmed Rule 8-clean (close) or narrowed to a specific bounded gap with new acceptance criteria.

---

### PJ-022 — Backlinks panel

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Currently recomputed on tab focus by walking `note_links` per-target. Should be a cached table or a materialized view.

**Acceptance.** Tab focus shows backlinks instantly even on 100-link nodes.

---

### PJ-023 — Outgoing Links panel

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Currently recomputed on tab focus. Same shape as PJ-022; pair into one MIG.

**Acceptance.** Same as PJ-022.

---

### PJ-024 — Tag browser

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Scanned on open. Should read from a maintained tag→notes index.

**Acceptance.** Tag browser opens instantly on libraries of any size.

---

### PJ-025 — Sight dashboard (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: Sight is invoked from the frontend's `toggleLens()` handler (`+layout.svelte:3354` calls `constellation_sight_centrality`). It runs on-demand when the user toggles the Sight overlay; results are cached and reused while fresh. Boot path does NOT invoke Sight. The "rebuilds on every boot" framing in the original PJ description was incorrect — there was no boot-time Sight rebuild to migrate away from.

The PJ-025 number is retired per the stable-reference-numbers rule. No further action.

---

### PJ-026 — Sidebar star counts (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: `loadAllStats` walks the per-library star count at boot, but per `+layout.svelte:1939-1941` comment: *"loadAllStats remains because its Rust side is already cache-fast (metadata-only walk + per-library thread parallelism). It's fire-and-forget so the sidebar star counts populate without blocking anything."* This is a cached-fast path, not a Rule-8 violation. Not boot-blocking, not full filesystem walk. Original PJ-026 framing was incorrect.

PJ-026 number retired.

---

### PJ-027 — Map (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: `src-tauri/src/map.rs:300` documents that the Map data is *"maintained by triggers on note-save, so even an explicit open is..."* — already write-time derived per Rule 8. No Rebuild on open / boot.

(PJ-011's separate Map open issues — perf/leak, tooltip showing canonical filename, search-highlight — remain open as P2; those are panel UX bugs unrelated to the persistence question.)

PJ-027 number retired.

---

## §7 · MIG-014 §2F audit follow-ups (PJ-028 → PJ-033 — carried from v1.2)

Six edge-case items found by the MIG-014 §2F three-agent audit (2026-05-06) but logged for later, not blocking close. Memory note: `project_mig014_audit_p2_p3_followups.md`. All non-blocking — graceful degradation in every case. P2/P3 severity.

### PJ-028 — `splitStage` and a leading dash

**Status.** Open · **Severity.** P2 · **Effort.** 2-line fix

`stage: -concept` (no lifecycle prefix) splits to `lifecycle=''`, `suffix='concept'`. Renders as `-Concept` with empty emoji and no promote/demote arrows.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 1; MIG-014 §2F audit.

**Acceptance.** `splitStage` treats empty lifecycle as "no stage" — returns both empty when no valid lifecycle prefix is present. Edge-case display normalizes cleanly.

---

### PJ-029 — Concept Paper §6.1 vs `commitStage` multi-dash drift

**Status.** Open · **Severity.** P2 · **Effort.** Decision + 2-line code fix OR doc fix

Stages Concept Paper v1.2 §6.1 says suffix may not contain `-`. `commitStage` at `PropertyEditor.svelte:199` doesn't enforce. Multi-dash values like `stage: spark-foo-bar` are accepted. Either tighten code (reject) or update doc (allow). Doc-vs-code drift; not a runtime bug.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 2.

**Acceptance.** Doc and code agree on whether multi-dash suffixes are allowed. Whichever direction, the surviving rule is enforced by tests.

---

### PJ-030 — Stale `custom_stages: [...]` from §1A-era testing in `universe.json`

**Status.** Deferred · **Severity.** P3 · **Effort.** None (acceptable)

Serde silently ignores the unknown field; gone on next read-modify-write cycle of `universe.json`. Affects only Boss-equivalent dev-build users (none reported). Acceptable graceful self-healing.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 3.

**Acceptance.** Self-heals on next universe-meta write. No active fix needed unless surfaces as a bug.

---

### PJ-031 — Trailing-dash on disk (`stage: spark-`)

**Status.** Deferred · **Severity.** P3 · **Effort.** None (acceptable)

`splitStage` returns `suffix=''`; nextStage carries no suffix. Display correct. The trailing dash stays on disk verbatim until the user re-commits via promote. Acceptable graceful self-healing.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 4.

**Acceptance.** Self-heals on next promote/demote. No active fix needed.

---

### PJ-032 — Uppercase on disk (`stage: SPARK-CONCEPT`)

**Status.** Deferred · **Severity.** P3 · **Effort.** Optional 2-line fix

`LIVING_LINK_BASELINE.findIndex` is case-sensitive → returns -1. No emoji, no arrows. Display falls back to verbatim render. User must re-pick to recover. Could be normalized in `splitStage` (lowercase the lifecycle component) but the current behavior is acceptable graceful degradation.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 5.

**Acceptance.** Either lowercase normalization in `splitStage`, or status quo if Boss prefers strict canonical form.

---

### PJ-033 — NotePane stage badge `<span>` has no `dir="auto"`

**Status.** Open · **Severity.** P3 · **Effort.** 1-line fix

`src/lib/components/NotePane.svelte:951`. A long Arabic suffix in an LTR UI may render slightly off due to Chromium's bidi defaults. Easy polish — add `dir="auto"` to the badge span.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 6.

**Acceptance.** `dir="auto"` on the stage-badge `<span>`. Mixed-script suffixes render with proper directionality in any UI direction.

---

## §8 · Sight v3 trajectory (NEW in v1.4 — PJ-035 → PJ-038)

The Sight Concept Paper v1.1 (`docs/Constellation-Sight-Concept-Paper-v1.1.md`) §12 truth-status matrix surfaced three implementation gaps in v2 Sight relative to the paper's analytical promise. Each is now an inheritable PJ.

### PJ-035 — Sight content-similarity TF-IDF edges

**Status.** **DONE (within MIG-019 §2B)** in commit `16063735` "MIG-019 §2B — Milky Way density wash (PJ-035) + Settings toggle". The PJ-035 mechanic shipped under MIG-019's Milky Way band per the v3 inheritance promise. v1.11/v1.12 preamble never updated to reflect this; corrected in v1.12 §μ state-of-standing audit (2026-05-18). · **Severity.** P2 · **Effort.** Multi-step (vector compute + cache + integration)

**The InfraNodus-defining mechanic.** v2 Sight wires explicit-wikilink edges (weight 1.0) and shared-tag edges (weight 0.6) into its graph build. The third edge type from the Concept Paper §3.3 — **content similarity (weight 0.3, TF-IDF cosine)** — is not implemented. This is the mechanic that lets Sight surface *latent* connections — notes that talk about the same topic without being explicitly linked. Without it, Sight cannot detect structural gaps that span un-linked-but-related clusters.

**Source.** `docs/Constellation-Sight-Concept-Paper-v1.1.md` §3.3 + §12 truth-status row.

**Acceptance.** TF-IDF vectors computed per note (incremental on changed notes only, cached). Cosine similarity computed lazily between notes above a configurable threshold. Edges merged into the Sight graph build with weight 0.3 (toggleable in Settings). For non-English content, configurable per-language stemmer pipeline (Arabic via ISRIStemmer or equivalent). Graceful degradation if NLP for a given language isn't loaded — tag/link edges still work.

**Inheritable into v3 (PJ-038)**: in v3's star-chart aesthetic, content-similarity edges become the **Milky Way band** (a diffuse density wash, not extra edge lines competing with constellation connectors). The compute layer is identical; only the visualization changes.

---

### PJ-036 — Sight layer peeling

**Status.** **ABANDONED 2026-05-18** (Eisa decision during post-MIG-026 state-of-standing triage — Sight v6's facet sidebar substitutes for the layer-peeling need; the v2 Concept Paper §2.2 mechanic 5 is no longer relevant under the v6 architecture). · **Severity.** P3 · **Effort.** Single feature (compute + UI toggle)

Originally proposed in `docs/Constellation-Sight-Concept-Paper-v1.1.md` §2.2 as the "remove the obvious to reveal the subtle" mechanic — hide top-N centrality nodes (typically MOC / index notes) and recompute analytics on the residual graph. Intended to be inheritable into v3 as a "hide brightest stars" toggle.

**Why abandoned**: Sight v6 ships a facet sidebar (`facetSidebar.svelte`) with 6 facet groups (Folder / Library / Stratum / Confidence / Stage / Provenance), each filtering to a subset of stars. The user can already isolate secondary structure by negative-selecting the dominant nodes' facets — the same diagnostic Outcome layer-peeling promised, delivered through a different UI metaphor that fits v6's tradition-aware grammar.

---

### PJ-037 — Map ↔ Sight integration

**Status.** **Rejected — 2026-05-07** · **Severity.** P2 (was) · **Effort.** Single MIG (was)

**Rejected by Eisa 2026-05-07** during the v3 Concept Paper review: *"There won't be Map-Sight integration."*

The two surfaces remain independent. Map is the "shape" view (radial sunburst, organizational hierarchy). Sight is the "patterns" view (star chart, conceptual relationships). Each lives in its own dock and is opened separately. The "Map diagnoses, Sight prescribes" loop — framed in the Sight Concept Paper v1.1 §7 — happens in the user's head, not in a shared cursor.

**Number retired** per the stable-reference-numbers rule. Entry preserved here as historical record.

**If revisited later**: a future PJ would need a fresh number (not PJ-037). Cross-references in commits or session logs to PJ-037 should be read as "the rejected Map↔Sight integration concept," not as a live job.

**Source (historical).** `docs/Constellation-Sight-Concept-Paper-v1.1.md` §7; v3 Concept Paper v1.0 §5.3 (the absorption proposal Eisa rejected).

---

### PJ-038 — Sight v3 build with own dedicated Concept Paper

**Status.** **SUPERSEDED (by Sight v6 / MIG-024 → MIG-027)** — marked at v1.12 §μ state-of-standing audit (2026-05-18). The Sight v3 trajectory was abandoned mid-flight (commit `29ce0101` "MIG-019 §0 — Sight v3 → v4 clean-slate pivot") in favor of the v4 redesign, which itself superseded into v5 then v6 (the radial-anchor + mini-domes + tradition-chip architecture that just shipped under MIG-026). `SIGHT_V3_ENABLED = false` in `engine.ts`; `SIGHT_V6_ENABLED = true`. PJ-052 + PJ-053 (Sight v6 follow-ups) closed under MIG-026. The 3-MIG decomposition described in this entry's body (MIG-018 / MIG-019 / MIG-020) is no longer the live trajectory. · **Severity.** P1 (was) · **Effort.** Multi-MIG (was)

**Phase 1 of 3 — MIG-018 closed Done (2026-05-07)**: projection foundation. Star-chart visualization with graph-distance Landmark-MDS embedding (Rust `sight_layout::compute_layout_embedding`), Lambert + stereographic projections (user-toggle in Settings → Sight), constellation territories drawn as Suwaidi warm-cream + gold soft-fill polygons (cycled by Louvain community id), faint connector lines at rest (Eisa's design call: visible structure without dominating attention), hover/click/double-click interactivity with side panel, RTL-aware tooltip + side panel. Six-phase cascade (§1A schema → §1B Landmark-MDS compute → §1C frontend skeleton → §1D star rendering + projection toggle → §1E territories + lines + interactivity [Boss-test gate] → §1F audit + close-out). 5 unit tests passing. Three-agent audit CLEAN (0 P0/P1/P2/P3). `SIGHT_V3_ENABLED = true` committed. Full record: `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-{ARCHITECT,PLAN,AUDIT}.md`.

**Phase 2 of 3 — MIG-019 (next-up)**: density + time + search + universe-health.

**Decision** (Eisa, 2026-05-07): rebuild Sight from scratch on the **star-chart aesthetic** — a 2D polar projection where star magnitude maps to centrality, constellation territories map to Louvain communities, the Milky Way band maps to content-similarity density, and a calendar rim maps to time. Reference image: 19th-century-style northern-hemisphere chart (Suwaidi reference; sample owned by the Boss). This is the design north star articulated in `docs/Constellation-Sight-Concept-Paper-v1.1.md` §13.

**What v3 inherits from v2** (the analytical pipeline is preserved as-is):
- Brandes' betweenness centrality (`constellation_sight_centrality` IPC).
- Louvain community detection (`constellation_sight_communities` IPC).
- Structural gap detection (`constellation_sight_structural_gaps` IPC).
- Universe-health metric (`constellation_sight_universe_health` IPC: M + D + E + C).
- Reveal-on-demand (Principle 6) — already shipped in v2's MIG-016 §1B.

**What v3 absorbs (the deferred PJs)**:
- **PJ-035** content-similarity edges → Milky Way band (diffuse density wash).
- **PJ-036** layer peeling → "hide brightest stars" toggle (drag right per astronomy convention).
- ~~PJ-037 Map ↔ Sight integration~~ — **Rejected** 2026-05-07; not part of v3.

**What v3 rebuilds entirely** (the visualization layer):
- Force-directed Pixi.js simulation → 2D polar projection (likely Lambert azimuthal equal-area or similar; specified in v3's own Concept Paper).
- D3-style force layout → stable astronomy-style projection math.
- Edge-render hot path → constellation-line idiom (lines render only inside the focused constellation territory).

**Mandatory deliverable: own dedicated Concept Paper** (Boss directive 2026-05-07). The v3 paper is the canonical reference for *what v3 looks like and how it is built*. The v1.1 paper continues as the canonical reference for *what Sight is for* (the analytical foundations both versions share). They are read side-by-side.

**Source.** Boss decision 2026-05-07; Sight Concept Paper v1.1 §13 + §14; MIG-016 audit §6 ("inheritance into v3").

**Acceptance.** v3 ships behind feature flag (`sight.engine: 'v3'`), default ON in production once Boss-test passes. Star-chart projection renders the user's full universe at one glance. Constellation territories visually distinct. Reveal-on-demand for connector lines (Principle 6 baked into the visual grammar). Universe-health metric readable in one glance from dome-balance. Three deferred PJs (PJ-035 / PJ-036 / PJ-037) integrated as the design intends. Three-agent audit clean. Own Concept Paper delivered alongside the build.

**Composes with**:
- PJ-039 (precondition: v2 disabled before v3 starts).
- PJ-035 / PJ-036 / PJ-037 (absorbed as v3 features rather than v2 add-ons).
- PJ-013 (`apply_lens` dead code may be cleaned up as part of the v3 cutover).

---

## Newly filed since v1.12 (PJ-059 → PJ-064)

Six numbers allocated 2026-05-19 → 2026-06-09. PJ-059 / PJ-060 were filed in the 2026-05-19 session log; **PJ-061 / PJ-062 give proper, never-before-used identities to two fixes the 2026-05-29 log informally (and incorrectly) called "PJ-10 / PJ-11"** — colliding with the canonical PJ-010 / PJ-011, which are untouched; PJ-063 / PJ-064 are promoted here from memory notes.

### PJ-059 — Sight per-note search/finder

**Status.** Open — **dormant** (Sight disabled by MIG-038) · **Severity.** P3 · **Effort.** Single-file feature

A per-note search/finder inside the Sight dome, scoped to the Sight v7 close-out. Filed 2026-05-19. Dormant while Sight is a disabled / "Wings" surface; re-activate alongside Sight.

**Source.** SESSION-LOG-2026-05-19 (Sight delivery cascade, carried PJs).

---

### PJ-060 — `index_note` cache-hit short-circuit fix

**Status.** Open · **Severity.** **P1 — highest-leverage open fix** · **Effort.** Mini-MIG

`index_note`'s cache-hit short-circuit (`search.rs:3004`) returns early when the file mtime matches, so the write-time refresh of `note_meta` never fires on an unchanged-mtime re-save. Traced 2026-05-19 as the root cause blocking the MIG-029 write path; flagged as "the single most-leveraged open fix." A focused mini-MIG: correct the short-circuit so write-time derivation always runs.

**Source.** SESSION-LOG-2026-05-19 (MIG-029 → MIG-036 pivot, root-cause section).

**Acceptance.** A re-save with unchanged mtime still refreshes `note_meta` + every write-time-derived surface; no boot / typing regression on the 7,600-note universe.

---

### PJ-061 — Sky View federated node sizing  *(DONE)*

**Status.** **DONE 2026-05-29** · **Severity.** P2 · **Effort.** Tuning

Once MIG-061 made Sky View show the full 8,751-node federation, nodes rendered too large. Fixed over 3 rounds: count-aware fill damping + halving the fixed-pixel decorative frames (stratum / provenance / maturity / MOC rings) in dense (>1500-node) mode → ~0.12× at 8,751. Boss-verified. *(Filed unnumbered 2026-05-28 as "PJ-NNN-A"; the 2026-05-29 log called it "PJ-10" — a collision with the real PJ-010; numbered properly here.)*

**Commit chain.** `9a2d9890` → `62a9a198` → `f05fe6f9`. See §9.

---

### PJ-062 — CNS gravity-well canvas fill  *(DONE)*

**Status.** **DONE 2026-05-29** · **Severity.** P2 · **Effort.** Tuning

The CNS (Constellation Nervous System) gravity well left big margins on wide monitors (`maxR = min(w,h)×0.45`). Bumped to `×0.58` + fitToScreen zoom `0.85→0.93`; stayed circular (Form-Aligns-To-Purpose: radius = centrality, angle = library). Boss-verified first try. *(Filed unnumbered 2026-05-28 as "PJ-NNN-B"; the 2026-05-29 log called it "PJ-11" — a collision with the real PJ-011; numbered properly here.)*

**Commit.** `9a2d9890`. See §9.

---

### PJ-063 — `note_links.link_type` globally `'relates'`

**Status.** Open · **Severity.** P1 · **Effort.** /migration (foundational)

`note_links.link_type` is globally `'relates'`: the backend `extract_typed_links` (`search.rs:3614`) parsed `[[type::target]]` while notes + editor stored the type as the last segment of `[[target|display|type]]`. Exposed by the MIG-066 Link-types column. Affects every `link_type` consumer (Base columns, Sky View edge colours, typed-link panels). **Re-verify against the post-MIG-067 state** — MIG-067 shipped the Link-Type Registry + predicate-first `[[type::target]]` authoring after this was first observed, which may have shifted or partly resolved it.

**Source.** Memory `project_note_links_link_type_relates_bug`; MIG-066 / MIG-067 (orientation v2.46 / v2.47).

**Acceptance.** Every note's `note_links.link_type` reflects the authored type across all consumers; both legacy `[[target|…|type]]` and predicate-first `[[type::target]]` resolve correctly.

---

### PJ-064 — Style Setter: more font types in the final version

**Status.** Open · **Severity.** P3 · **Effort.** Single-component feature

The Style Setter's font pickers currently expose System / Serif / Mono as placeholders; the final version should offer the full font catalogue. (The other half of this request — named / saved colour swatches — already shipped in MIG-070.)

**Source.** Memory `project_style_setter_feature_requests` (2026-06-02).

---

## §9 · Done

Items move here from the categories above when they close. Format preserved per stable-reference-numbers rule: the original entry stays in its source section with its closure status; this section provides a quick chronological index.

| PJ-NNN | Title | Status | Closed | Commit chain |
|---|---|---|---|---|
| PJ-001 | Chunk the v2 sentinel migration with progress UI | Done | 2026-05-06 (via MIG-015) | `0ca7e64` → `df0bf87` → `62d3b4a` → `877e46e` |
| PJ-006 | Living Link Architecture P2–P5 (all phases verified shipped & user-validated) | Done | 2026-05-06 (closed in v1.3 cross-check; shipped earlier in CE Phase §90-§142 commit range) | (multi-commit; orientation §4.4 line 1167+ for canonical state) |
| PJ-007 | Note-stage taxonomy: per-note dash-encoded model | Done | 2026-05-06 (via MIG-014) | `c3b9454` → `8a9ab3d` → `17bf474` → `9973e65` → `f4eef3e` → `2f58b8a` → `59ed95c` → `432076c` → `2c58bda` → `bb7a6ef` → `e3a97a1` → `a50463c` → `339d65b` |
| PJ-025 | Sight dashboard (Obsolete — verified write-time-derived) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-026 | Sidebar star counts (Obsolete — verified cache-fast) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-027 | Map (Obsolete — verified trigger-maintained) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-034 | MIG-016: Sight instant-toggle perf | **Cancelled (partial-shipped)** | 2026-05-07 (§1A + §1B shipped; §1C/§1D cancelled; §1E deferred to PJ-038) | `a0babbb` → `7e76b17` → `62718f7` |
| PJ-039 | MIG-017: Disable v2 Sight (precondition for v3) | Done | 2026-05-07 (single mini-MIG) | `8c4019c` |
| PJ-037 | Map ↔ Sight integration | **Rejected** | 2026-05-07 (v3 Concept Paper review — Eisa rejected the integration concept) | n/a |
| PJ-015 | 360.3D Stratification Matrix guidance doc | **Abandoned** | 2026-05-18 (state-of-standing triage; low-leverage) | n/a |
| PJ-035 | Sight content-similarity TF-IDF edges → Milky Way band | Done | 2026-05 (via MIG-019 §2B) | `16063735` |
| PJ-036 | Sight layer peeling | **Abandoned** | 2026-05-18 (v6 facet sidebar substitutes for the mechanic) | n/a |
| PJ-038 | Sight v3 build (3-MIG trajectory) | **Superseded** | by Sight v6 (MIG-024→027); v3 abandoned `29ce0101` | n/a |
| PJ-040 | UA short-circuit on partial frontmatter | Done | 2026-05 (via MIG-022 §D) | `c072700` |
| PJ-052 | Sight Concept Paper v4.1 | Done | 2026-05-18 | (docs) |
| PJ-053 | λ-fix-6 native-quality translation polish (192 keys, 7 locales) | Done | 2026-05-18 | (i18n) |
| PJ-054 | Sight v6 vitest test runner (58/58 pass) | Done | 2026-05-19 (via MIG-030) | `f327d758` |
| PJ-055 | User-plugin label schema warning | Done | 2026-05-18 | `e63ee0c7` |
| PJ-056 | MIG-026 drift cleanup | Done | 2026-05-18 | `e63ee0c7` |
| PJ-057 | Post-MIG-026 doc-drift (a/c done; b deferred) | Done (partial) | 2026-05-19 (via MIG-032) | `f327d758` |
| PJ-058 | Constellation Sight Subsystem Concept Paper v1.0 | Done | 2026-05-19 | (docs) |
| PJ-016 | Drop `term_vocab.bridge_concept_id` column | Done | via MIG-042 | (orientation v2.25) |
| PJ-061 | Sky View federated node sizing | Done | 2026-05-29 | `9a2d9890` → `62a9a198` → `f05fe6f9` |
| PJ-062 | CNS gravity-well canvas fill | Done | 2026-05-29 | `9a2d9890` |

---

## Appendix — How to amend this document

1. **Adding a job.** Append to the appropriate `§N` section with the **next unused** `PJ-NNN` ID — never reuse a number from a Done / Rejected / Cancelled / Merged entry. Bump the version. Commit + push as a new file (`v1.5.md` etc.).
2. **Updating a job.** Edit the existing entry (status, severity, source, acceptance). Bump the version if the change is structural (status transition, scope change); same version if just refining wording.
3. **Closing a job.** Move the entry to `§9 Done` keeping its `PJ-NNN`, strikethrough the title, append closing date and commit hash. Bump the version. The number is retired with the entry — never recycled.
4. **Rejecting / cancelling a job.** Same shape as closing — move to `§9 Done` with status `Rejected` or `Cancelled` (or `Cancelled (partial-shipped)` if some phases shipped before abandonment), keep the number, record the date and reason. Number retired.
5. **Splitting a job.** Keep the parent ID. Add `PJ-NNNa`, `PJ-NNNb`, etc. as siblings. The parent entry stays as a header pointing at the children. Cross-reference in both directions.
6. **Merging jobs.** All merged numbers stay in the doc. Survivor keeps its number; merged entries point at the survivor with `merged into PJ-NNN`. All merged-in numbers are retired.
7. **Renumbering. Strictly forbidden.** PJ-NNN is a permanent reference identity. Session logs, commits, and other docs cite jobs by number. Renumbering would silently break every historical reference.
8. **Filename convention.** Same as orientation + Laws docs + NotePane Specs. New version = new file alongside the previous. Older versions stay as historical record.

---

## Appendix — Cross-references

This doc is read alongside:

- `CLAUDE.md` — operational rules. Several jobs cite specific top-principals or rules.
- `docs/Constellation Orientation & Onboarding v1.55.md` (current at v1.4) — project's operating state, with predecessor versions back to v1.0. Jobs cite orientation versions where they were first surfaced.
- `docs/Constellation Development Laws v1.4.md` (current) — higher-order law statements. Jobs are the *surfaces* the Laws operate on.
- `docs/Constellation-Sight-Concept-Paper-v1.1.md` (NEW in v1.4) — Sight's analytical foundation. PJ-035 / PJ-036 / PJ-037 / PJ-038 all derive from its §3 / §12 / §13 / §14.
- `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — daily engineering record. Jobs are surfaced into a session log entry when work begins; entry ID + commit hash flow back into Done.
- `lab/reports/MIG-NNN-*.md` — Architect / Plan / Audit docs for each MIG. Several jobs cite specific audit findings.
- `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` (NEW in v1.4) — partial-shipped audit closing PJ-034.
- Auto-memory at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\` — every `project_*.md` file is a candidate source for a Pending Job entry.

---

**End of v1.13.** Reconciliation version — folds 37 migration numbers (MIG-036→072) into the record (authoritative table now in orientation §8 v2.61), allocates PJ-059→064, corrects the 2026-05-29 "PJ-10/11" collision, and refreshes the top-of-queue. New #1: **PJ-060** (`index_note` cache short-circuit). Previous milestone: `localization-complete` (MIG-072 closed, 2026-06-09).
