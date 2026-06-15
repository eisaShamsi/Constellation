# 25 — Federation (cUniverse) (Concept Paper)

> Function #26 in [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md), Phase 6. Federates child universes into the active one so search, library stats, lens, and the status-bar notes count span the whole federation tree. Its boot IPC currently fires **unconditionally** — the central bring-up issue.

## 1. Function in hand
**Federation (cUniverse)** — the cross-universe federation layer (`src-tauri/src/federation/`: `attach.rs`, `query.rs`, `migrate.rs`, `failure.rs`, `mod.rs`) plus its frontend surface: the **federation warning badge + popup** in `src/routes/+layout.svelte` (the amber-triangle status-bar badge, ~`:7491`), backed by `getFederationWarnings()` in `src/lib/federation/store.ts`. Introduced by MIG-056; extended by MIG-061 (CNS/Sky) and MIG-062 (filesystem-level federated sidebar groups).

## 2. Purpose
The ONE job: **make notes from linked child universes (cUniverses) appear in the active universe's read surfaces** — global search, library stats, the lens, the status-bar notes count — by ATTACHing each cUniverse's `search.db` read-only and `UNION ALL`-ing their rows. It serves **Connection** (the second Act): it lets the user reason across independent knowledge bases as one. It exists because a Universe can declare cUniverse children (the federation manifest), and federation is what turns that declaration into queryable reach. The *warning surface* specifically answers: *"which linked universes are unreachable right now, and why?"* — honest failure reporting under the skip_unavailable model.

## 3. What it is NOT
- **NOT** a copy/import — cUniverses are read **in place**, read-only (`?mode=ro`). Federation never writes to a child universe.
- **NOT** a write path — there is no "edit a federated note through the parent." Editing requires switching to that universe as active.
- **NOT** the second screen, and **NOT** the sidebar's cUniverse tree rendering itself (that's the library-panel concern); this paper is the *federation engine + its warning surface*.
- **NOT** mandatory — a Universe with zero cUniverses is a complete, valid setup; federation is opt-in (CLAUDE.md hierarchy).

## 4. Wiring
- **Inputs:** boot spawn from `ensure_search_db_ready` (search.rs ~`:6948`) on a background thread; `resolve_universe_libraries` + `active_universe_dir` to enumerate cUniverse roots; each cUniverse's `.constellation/search.db`. Frontend reads via the `federation_get_warnings` command (`store.ts::getFederationWarnings`).
- **Outputs (IPC/events):** `federation_get_warnings` (returns `Vec<FederationWarning>`); the `federation:ready` event (search.rs ~`:7108`) emitted once attach completes. Writes the `FederationContext` + `federated_conn` into `SearchState`. Auto-migrates schema-drifted cUniverses (`migrate.rs`, §5.3) with lock-check + backup + atomic txn.
- **Consumers:** federated query builders (lens / status-bar libraryStats / global search — Architect §5.1); the `+layout.svelte` `federation:ready` listener (~`:2552`) which re-invokes `cache_boot_snapshot_sky`/`_graph` and `loadAllStats()`; the warning badge/popup.
- **Connection to the Editor (the gate):** **indirect / downstream only.** Federation is a read-side aggregator; it never attaches to the Editor and never participates in the keystroke or save path. The Editor edits one note in the active universe; federation simply widens what the *read* surfaces see. No direct wire to the gate exists — verify in bring-up that none is needed.

## 5. Right-click / context menu
- **None.** The warning badge (`.sb-federation-warning`, ~`:7491`) and popup are plain `onclick` toggles; the cUniverse sidebar header button (~`:5654`) has **no `oncontextmenu`** (confirmed — unlike sibling library headers, which call `handleLibraryHeaderContextMenu`). No action in the federation surface is reachable only by right-click.
- **Gap flagged:** a cUniverse row arguably *should* have a right-click menu (e.g. "Switch to this universe", "Detach cUniverse", "Reveal in file manager", "Copy path", "Retry attach"). Today those are absent. If added during bring-up it **must** use the shared `<ContextMenu>` / `buildContextMenu` (MIG-077) — not a hand-rolled menu. The exact item set is **unknown — verify in bring-up** (do not assume the list above is canonical).

## 6. Multilingual
- The warning surface uses `$t()` for all four strings: `federation.warningBadge`, `federation.popupTitle`, `federation.cuniverseLabel`, `federation.reasonLabel` — each with a hard-coded English `||` fallback in the markup (~`:7491`–`:7520`). The keys are **present in all 15 locales** (ar de en es fa fr he hi ja ko pt ru tr ur zh — verified), with native equivalents (ar: `كون فرعي` / `تحذيرات الاتحاد`).
- **Flag — hardcoded English not localized:** the *reason* text (`w.reason`) comes from Rust as a raw English string (`"search.db missing"`, `"ATTACH failed: …"`, `schema_incomplete: …` in `attach.rs`). It is displayed verbatim and is **not** routed through `$t()`. This violates the "everything localizes" standing order; bring-up must map reason codes to localized strings.
- **RTL:** the popup uses no explicit `dir`/`detectDir()`; in an RTL locale the badge/popup inherit document direction. **Unknown whether the path strings + reason render correctly RTL — verify in bring-up.**

## 7. Boot behavior
- **Runs at boot? YES — and unconditionally.** Two parts: (a) Rust spawns `attach_all` on a background thread from `ensure_search_db_ready` (never blocks boot, by design — Architect §3.3); (b) the frontend calls `getFederationWarnings()` during the post-hydration fan-out (~`:2042`) *regardless of any feature flag*, then re-polls once at +3 s.
- **Rule 8 status: MIXED — partial violation.** The *read* path is compliant: the badge **reads the persisted** `FederationContext.warnings` (a stored snapshot; the IPC is a Mutex lock + clone). But the `FederationContext` itself is **recomputed on every boot** — `attach_all` re-walks cUniverse roots, re-ATTACHes each `search.db`, and re-verifies schema every launch; nothing about the attach result is persisted across boots. For the warnings surface this is acceptable (transient, must reflect *now*), but the FTS5 page **pre-warm** (`federation_prewarm`, ~10–15 s/cUniverse) is a per-boot recompute that Rule 8's spirit would push toward a persisted/cached warm state. Flag for bring-up review.
- **Cost:** attach ≈ tens-to-low-hundreds ms per cUniverse (Architect §6.3, estimated); pre-warm ≈ **10–15 s per cUniverse** background (measured-class estimate from the MIG-058/059 comments, marked). Boot is not blocked; first federated searches during warm-up are slow (documented in search.rs). ATTACH cap = 25 (`ATTACH_CAP_V1`).

## 8. Flag / gate & bring-up position
- **Gate today:** **none — needs a NEW gate.** `getFederationWarnings()` and the `federation:ready` listener fire unconditionally at boot; there is no `enabledFeatures.federation` / `FEDERATION_ENABLED` guard (confirmed — no such key in the codebase). The master charter marks this "**unconditional → NEW**".
- **Bring-up phase: 6 (Federation, second screen, infra).** Depends on: the search DB being ready (`ensure_search_db_ready`), the universe/libraries resolver, and the status-bar/lens/search consumers it feeds. Bring-up must add a gate so a minimal/single-universe shell does not pay the unconditional federation poll + background attach.

## 9. Budget
- **Boot budget:** zero boot-thread cost — attach + pre-warm are off-critical-path background threads; the frontend poll is a single cheap IPC. Must stay non-blocking (Architect §3.3 hard invariant).
- **Interaction budget:** badge toggle is instant (local `$state`). Federated search inherits the search budget; first-query-during-warm-up slowness is the known cost (sub-second after warm). No `invoke()` on any hot path.
- **Regression guard:** measure boot `paint_ms`/`hydrated_ms` with N cUniverses vs zero; confirm the background attach never extends `hydrated_ms`; confirm the status-bar count converges to the federated total (the 1101→8751 case in the code comments) within the +3 s re-poll.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** linked cUniverses' notes appear in global search / lens / library stats / status-bar count; unreachable cUniverses surface as warnings (skip_unavailable, never a hard failure).
- [ ] **Serves Constellation's core purpose:** federation realizes **Connection** across independent universes; reads in place, never copies, never writes a child (File-Over-App).
- [ ] **Wires correctly to the Editor (the gate):** confirmed federation stays downstream-only — no attachment to the keystroke/save path; the Editor edits only the active universe.
- [ ] **Right-click present + correct:** decide whether cUniverse rows get a context menu; if yes, it uses shared `<ContextMenu>`/`buildContextMenu` (MIG-077), not hand-rolled; items enumerated and verified.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** badge/popup strings localized (already ×15); **`FederationWarning.reason` mapped to localized strings** (currently English-only); popup verified RTL.
- [ ] **Within budget:** background attach never extends `hydrated_ms`; status-bar count converges within +3 s; pre-warm stays off the critical path.
- [ ] **Obeys Rule 8:** warnings read from the persisted `FederationContext`; review whether the per-boot ATTACH + pre-warm recompute should be cached/persisted.
- [ ] **Holds its invariants:** read-only attach (`?mode=ro`); ATTACH cap = 25; generation-guard abandons stale work on universe switch; auto-migrate keeps lock-check + backup + atomic txn.
- [ ] **Boss-tested** per the Testing Instructions Rule (link two universes, kill one's `search.db`, confirm the badge counts it and search still returns the rest).

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—** (background-only by design; measure with N cUniverses before sign-off)
Notes: The defining bring-up task is **adding a gate** — federation's boot IPC + `federation:ready` listener fire unconditionally today (charter "unconditional → NEW"). Two honest debts: (1) `FederationWarning.reason` is raw English, not `$t()`'d; (2) cUniverse rows have **no** right-click menu while sibling library rows do — gap, not a regression. Rule 8 is satisfied on the *read* side (badge reads persisted warnings) but the per-boot ATTACH + ~10–15 s/cUniverse FTS5 pre-warm is a per-boot recompute worth a caching review. Exact context-menu item set and RTL-popup correctness are **unknown — verify in bring-up**.
