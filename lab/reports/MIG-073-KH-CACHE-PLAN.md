# MIG-073 — Knowledge Health Write-Time Cache (Plan / Phase 2)

**Status: PLAN (Phase 2). Awaiting Eisa's approval → then cascade the build (Plan-Approval = Build-Approval).**
**Decisions locked (Eisa, 2026-06-10):** Option **B** (background-refreshed snapshot cache), **general scope**
(the circulatory-aggregate layer that serves KH now + CCS later). Architect: `MIG-073-KH-CACHE-ARCHITECT.md`.

## Locked design choices (from the Architect's open questions §7)
- **Scope:** general `link_stats_cache` key/value table (CCS-extensible), not KH-specific.
- **Pattern:** **stale-while-revalidate.** The panel ALWAYS reads the cached snapshot (instant); if the snapshot
  is older than a freshness window, it ALSO kicks a background recompute so the *next* open is fresh. First ever
  open (empty cache) shows "Computing diagnostics…" once, then renders on a `kh-snapshot-ready` event.
- **Recompute hook:** the existing `links_backfill` post-settle path (already recomputes `outgoing_*` in bulk
  after reconcile) + the stale-while-revalidate kick. This absorbs both write-driven and time-driven drift.
- **`most_connected`:** KEEP in KH for now (it's CNS topology per CCS §12, but KH shows it today — don't change
  KH's *content* in this migration; CCS later re-homes it).
- **DRY:** extract the existing query bodies into `&Connection`-taking helpers reused by both the live IPCs
  (kept as thin wrappers for back-compat) and the recompute — no duplicated SQL.
- **Federation/freshness window:** per active-universe `search.db` (matches KH today). Freshness window =
  **10 minutes** (time-driven buckets tolerate it; tunable).

---

## Phase 1 — Backend: cache table + recompute + snapshot IPC + first-time population
**One commit. Verified by me (backend; not yet Boss-visible).**

1. **Schema** (`init_db`, additive): 
   ```sql
   CREATE TABLE IF NOT EXISTS link_stats_cache (
       stat_key   TEXT PRIMARY KEY,   -- 'stats' | 'lifecycle' | 'fmt_emerging' | 'fmt_bias_check'
                                       -- | 'fmt_most_connected' | 'fmt_weak_foundations' (CCS adds more)
       payload    TEXT NOT NULL,       -- JSON of that aggregate's result
       computed_at TEXT NOT NULL DEFAULT ''
   );
   ```
2. **Extract helpers** (DRY): pull the SQL bodies out of `constellation_link_stats`, `constellation_link_decay`
   (read-only lifecycle counts), and `constellation_formulation_analysis` into `compute_link_stats(conn)`,
   `compute_lifecycle(conn)`, `compute_formulation(conn, query_type)`. The three existing IPCs become thin
   wrappers (lock + call helper) — existing callers unaffected.
3. **`recompute_link_stats_cache(conn)`**: run the 6 helpers ONCE, `INSERT OR REPLACE` each result as JSON into
   `link_stats_cache` with `computed_at` = now. This is the single full scan, run in the background.
4. **`constellation_knowledge_health_snapshot()` IPC**: read the 6 keys; if all present → return them bundled
   (+ `computed_at`); if stale (>10 min) → also spawn a background recompute + emit `kh-snapshot-ready`; if empty
   → spawn the recompute, return `{ ready: false }`.
5. **First-time population**: on the existing post-reconcile/`cache-reconciled` path, if the cache is empty, run
   `recompute_link_stats_cache` on the background thread (after paint), then emit `kh-snapshot-ready`.

**Verify (me):** unit-open the DB after boot → `link_stats_cache` has 6 rows; the snapshot IPC returns them in
<50ms (no `note_links` scan — confirm via timing on the 1.7 GB DB); a forced recompute refreshes `computed_at`.

## Phase 2 — Wire the KH panel to the snapshot (the freeze is gone) ★ Boss test
**One commit. Boss-testable: this is the milestone that kills the freeze.**

1. `KnowledgeHealthDashboard.svelte` `onMount`: call `constellation_knowledge_health_snapshot()` (one IPC)
   instead of the 6 raw `invoke`s. Render `stats` / `lifecycle` / the 4 formulation lists from the snapshot.
2. Cold-cache state: if `{ ready: false }`, show the existing "Loading diagnostics…" once, and `listen` for
   `kh-snapshot-ready` → re-fetch + render. Unlisten in cleanup (Perf Rule 4).
3. Keep the read-only `constellation_link_decay` etc. as fallout-free (still used by the store wrappers).

**Verify (Boss tutorial):** *What it is* — the Knowledge Health panel (the heart-pulse 'health' tab) shows your
whole library's link vitals. *What was broken* — it froze on "Loading…" for 10–20s because it recomputed
everything from scratch on every open. *Now* — it reads a kept-fresh snapshot, so it opens instantly.
**Steps:** (1) open any note, right sidebar → the **heart-pulse** tab. First time after this update it may say
"Computing diagnostics…" for a few seconds (one-off), then show the cards. (2) Switch to another tab and back
to **health** → it appears **instantly**, no "Loading…". (3) Close + reopen the app, open **health** again →
still instant. *Fail:* if it still hangs 10s+ every open, the panel isn't reading the cache.

## Phase 3 — Refresh hooks (writes + time-drift propagate) ★ Boss test
**One commit. Boss-testable: changes show up.**

> **AS-BUILT (2026-06-10) — four deviations from the text below, all within the approved architecture
> (no new write path, no hot-path work); surfaced to Eisa at the test stop:**
> 1. **`links_backfill` hook DROPPED** — verified it never mutates `note_links` (it derives
>    `note_meta.outgoing_*` FROM it), so hooking it adds nothing. The bulk settle point is the reconcile
>    walk; that hook (P1) flipped to **unconditional**.
> 2. **Freshness window 10 → 2 min** (the plan marked it tunable). The kick is open-driven + the recompute
>    is background on a dedicated connection — worst case one warm scan per 2 min of active use.
> 3. **The panel listens for `kh-snapshot-ready` the WHOLE time it's open** (registered before the first
>    fetch — no missed-event race): a stale snapshot renders instantly, then **updates in place** when the
>    background refresh lands.
> 4. The original test promised "add a link → wait a moment → updated" — wrong on two counts found in
>    build: single-note saves go through `reindex_single_note` (never `cache_reconcile`), and the frontend
>    has **no live Rebuild-Index invoke** (comment-level only). The honest propagation path for single-link
>    changes is SWR: next open past the window kicks the refresh; the in-place update makes it visible in
>    one open.

1. ~~links_backfill hook~~ → the `cache_reconcile` walk-completion hook recomputes **unconditionally** (bulk
   settle), on the walker thread, dedicated connection — never the open path.
2. Stale-while-revalidate covers time-drift + single-link writes (traverse/archive/confidence/save): the next
   open past the **2-min** window kicks a background refresh and the open panel re-renders in place.

**Verify (Boss tutorial):** *What it is* — the vitals stay current as you work. **Steps:** (1) open **health**,
note **Total Links**. (2) In a note, add a new `[[wikilink]]` and save. (3) Wait ~2–3 minutes (do anything).
(4) Reopen **health** → it opens instantly (possibly with the old number), then within a few seconds the
numbers refresh in place — Total Links now reflects the new link. *Fail:* the number never updates across
two opens separated by 3+ minutes.

## Phase 4 — `/simplify` + Audit (Migration Rule Phase 4)
- `/simplify` the full MIG-073 diff.
- Three parallel audit agents: **invariants** (I1 no open-scan · I2 no hot-path write · I3 survives bulk reindex ·
  I4 reversible/rebuildable · I9 no re-decay of weight), **drift** (any new guard/trigger the system doesn't know),
  **migration-path** (first-boot empty cache · schema-mismatch on old DBs · mid-recompute interrupt · rollback =
  drop the cache table + revert the panel).
- Orientation §8 + a MIG-073 row, in the **same commit** as close-out (SO #6). Session log per phase (SO #1).

---

## Invariants carried from the Architect (§6): I1–I9. None may regress; measured before/after on the 1.7 GB universe.

## Rollback
The cache is purely derived. Rollback at any phase = drop `link_stats_cache` + revert the panel to the 6 raw
IPCs (still present as wrappers). No source-of-truth data is touched.

## Out of scope (flagged, not in MIG-073)
- The 1.7 GB `search.db` bloat (separate investigation — affects all cold reads).
- The CCS deep registers + retiring `LinkDashboard.svelte` (the CCS migration, built ON this layer).
- KH's final shape vs CCS (CCS Architect decides; this migration keeps KH as the cache's first reader).
