# MIG-073 — Knowledge Health Write-Time Cache (Architect / Phase 1)

**Status: ARCHITECT (Phase 1). Plan approved by Eisa (Option B, general scope); build under way.**
**Date: 2026-06-10. Function in hand: the Knowledge Health panel (`KnowledgeHealthDashboard.svelte`) —
universe-wide link diagnostics that freeze on open.**

> **§2-correction (found during P2 build, 2026-06-10):** `KnowledgeHealthDashboard` is the **full-screen
> overlay** (command palette "Knowledge Health" 🧠 / dock button → `showKnowledgeHealth`). The right-sidebar
> **'health' TAB renders `TensionPanel`** (fed by `detect_tensions`, `+layout:3293`) — a different surface
> this doc originally conflated with KHD. The tab's stuck-"Loading…" is a **separate pre-existing issue**
> (`detect_tensions` errors → `tensionReport = null` → permanent loading state), out of MIG-073 scope.

> Predecessor context: the compounding-decay corruption + the boot/periodic 11s scans were already fixed
> (commit `aa9941ee`, `constellation_link_decay` made read-only). What remains is the **read-time cold scan**
> that this migration removes.

---

## 1. The problem (FACT, measured)

The KH panel's `onMount` fires **6 read-time aggregates over `note_links`**: `constellation_link_stats`,
`constellation_link_decay` (now read-only — lifecycle counts), and `constellation_formulation_analysis` ×4
(`emerging`, `bias_check`, `most_connected`, `weak_foundations`). All 6 serialize on the single `state.db`
mutex. On Eisa's universe — **`search.db` = 1.7 GB, `note_links` = 234,061 rows** — whichever query hits the
table first **cold-reads it off disk (~11s)**; the rest run warm. Net: **~12–20s frozen "Loading…"** on a
cold open, with the DB lock held (stalling the rest of the app). This is the canonical **Perf Rule 8**
violation: a derived view recomputed at read time over the whole universe.

---

## 2. The scope-defining finding — KH **is** CCS's circulatory domain

`KnowledgeHealthDashboard.svelte` renders (verified):

| KH renders | = CCS §6 register |
|---|---|
| Lifecycle census (spark·birth·growth·maturity·dormancy·archival) | **The Life of a Connection** |
| Confidence Distribution (`by_confidence`) | **Conviction & Doubt** |
| Link Types (`by_type`) | **The Acts of Inquiry** |
| `most_connected` (per-target incoming) | topology — **CNS's** (CCS §12 says so) |
| `emerging` / `weak_foundations` / `bias_check` | formulation insights — CCS register territory |

So the panel that freezes is **universe-wide link circulatory aggregates** — exactly the data the ratified
**CCS Concept Paper §8** says must be *"write-time-maintained — no graph walk on open (Perf Rule 8)."*

**Consequence:** the cache that fixes the KH freeze **is CCS's write-time data layer.** Building it KH-specific
would be throwaway when CCS arrives. Building it as a **general circulatory-aggregate layer** fixes KH now
*and* lays the foundation CCS §13 assumes (which is why CCS could call itself "frontend-mostly").

**Flag for Eisa:** CCS §9/§15 ruled "*coordinate, not subsume* Knowledge Health — it does not absorb its
**note-quality counts**." But the *actual* `KnowledgeHealthDashboard` is **not** note-quality counts — it's
link circulatory aggregates (a proto-CCS). So either (a) KH stays and becomes the cache's first consumer
[coordinate], or (b) KH's content largely migrates into CCS later and KH shrinks to true note-quality counts.
**This migration doesn't force that choice** — it builds the shared data layer + points today's KH at it. The
CCS Architect revisits KH's final shape.

---

## 3. Territory (FACT — from the write-path/cache recon)

**11 write paths mutate `note_links`** (trigger points a cache must hook):
`index_note` DELETE+INSERT (`search.rs:4105`), reindex-delete (`6340`), full `reconcile_filesystem` walk
(`6251`), rename cascades source+target (`libraries.rs:985/989`), MIG-003 bulk rename (`mig003_step4.rs:204`),
`constellation_link_traverse` (`search.rs:5191`), confidence set + backfill (`5596`/`5617`), archive/unarchive
(`5653`/`5675`). Bulk paths DELETE-then-reinsert whole source-sets.

**The write-time-cache pattern is already deeply established** — 12 triggers on `note_links` maintain three
derived surfaces:
- **`sky_links`** — sparse copy of *active* edges (`note_links_sky_ai/ad/au`).
- **`note_meta.outgoing_*`** — per-note aggregates (`note_links_outgoing_ai/ad/au`), the closest precedent.
- **`sky_nodes.stratum/maturity`** — SQL-formula derivations.
The **FTS `note_meta_ai/ad/au` triggers** (`search.rs:2247`) are the canonical blueprint (separate AI/AD/AU,
`WHEN` gate on meaningful changes). `PRAGMA recursive_triggers=ON`.

**Critical discipline (must honor):** `reconcile_filesystem` **drops** the `outgoing` triggers before a full
walk and **bulk-recomputes + recreates** them after (`6281`/`6290`) to avoid O(N²) per-edge cost. Any KH cache
must do the same, or a reindex of 234k links fires 234k trigger recomputes.

---

## 4. The crux — write-driven vs time-driven metrics

Not all KH metrics can be trigger-maintained:

- **Write-driven** (change *only* on edge INSERT/UPDATE/DELETE): `total`, `by_type`, `by_confidence`,
  `with_annotation`, `most_connected`, `bias_check`, `weak_foundations`, and the `growth/maturity/dormancy/
  archival` lifecycle counts. → **trigger- or recompute-on-write maintainable.**
- **Time-driven** (drift with the wall clock, *no write event*): the `spark`↔`birth` split (created vs 7 days),
  cooling/stale (last_traversed vs 90 days), `emerging` (recently traversed). → **a per-write trigger can
  never fire for these** — a spark silently becomes a birth at the 7-day mark with no DB write.

This is the design fork: a pure-trigger cache leaves the time-driven buckets stale.

---

## 5. Design options

### Option A — Pure trigger-maintained counter table
Triggers increment/decrement a `link_stats` key/value table on every edge change. Reads are O(1).
- **Speed (reads):** instant. **Effort:** medium (incremental-delta triggers are fiddly — archive flips
  status, retype moves a by_type bucket, etc.). **Risk:** med-high — delta drift over 234k rows + 11 write
  paths; **cannot maintain time-driven buckets** (spark/birth/cooling go stale until a write touches them).
- Verdict: rejected as the whole answer; viable only for the pure write-driven counts.

### Option B — Background-refreshed snapshot cache  ★ RECOMMENDED
One `link_stats_cache` table holding the full aggregate snapshot (counts + the top-N insight lists as JSON).
Recomputed **off the open-path**: (a) **debounced after link writes settle** — reuse the existing
`links_backfill` / post-reconcile hook that already recomputes `outgoing_*` in bulk; (b) a **low-frequency
idle/timer refresh** to absorb time-drift (spark→birth, cooling). The panel reads the snapshot → **instant,
no scan**. The recompute is the *only* full scan — amortized, background, resumable (the Rule-8 first-time-
population pattern: run after paint, status-bar progress).
- **Speed (reads):** instant. **Effort:** medium — one table, one `recompute_link_stats()` fn (≈ today's 6
  queries, run once in the background), wire to the backfill hook + a timer. **Risk:** low — a recompute can't
  drift (it's a full snapshot, not deltas); worst case is *staleness* (bounded by refresh cadence), never
  *wrongness*. Survives bulk reindex trivially (recompute after).
- **Handles time-driven** naturally (the periodic refresh re-evaluates the clock-based buckets).
- Aligns with CCS §8 and Rule 8's letter ("persist the derived view; reads are cheap lookups").

### Option C — Hybrid (triggers for write-driven + read-time covering-index for time-driven)
Trigger-maintain the write-driven counts; compute the 2–3 time-driven buckets at read time via new covering
indexes (so they're index-only, not full-table scans).
- **Speed:** fast if the planner uses the covering indexes (unverified on the 1.7 GB DB). **Effort:** high
  (both mechanisms). **Risk:** med — most moving parts; a mis-planned covering query still scans → partial
  freeze returns. Adds indexes to an already-1.7 GB DB.

---

## 6. Invariants (must not break)

- **I1** No full scan of `note_links` on panel open (the whole point).
- **I2** No new write on any hot path — the cache recompute is background/debounced, never on a keystroke or IPC-open.
- **I3** Survive bulk reindex — honor the drop/recompute/recreate discipline (`reconcile_filesystem`); never fire 234k per-edge recomputes.
- **I4** Reversibility — the cache is *derived*; droppable + rebuildable from `note_links` at any time (it is never a source of truth).
- **I5** First-time population — background after paint, status-bar progress, resumable (Rule 8).
- **I6** Federation-transparent — universe-scoped; cUniverse aggregation behaves as the panel does today (open question §7).
- **I7** Facts rest — if/when CCS consumes the layer, signals never flag a resting fact (inherited from CCS §7); not triggered by this migration but the data shape must not preclude it.
- **I8** Locale + theme aware — panel concern only; the cache is language-neutral data.
- **I9** The corruption fix stays — decay remains display-only; the cache stores *raw* weight aggregates, never re-decays.

---

## 7. Open questions for the Plan (Eisa decides at the Phase-2 gate)

1. **Cache scope** — general circulatory-aggregate layer (serves KH now + CCS later) **[recommended]**, or KH-specific?
2. **KH's fate** — KH stays as the cache's first consumer [CCS §15 "coordinate"], or is KH's content
   earmarked to fold into CCS later (KH → true note-quality counts)? *(Doesn't block this migration; sets the doc framing.)*
3. **Design** — Option B (recommended), A, or C?
4. **Refresh cadence** — which hook (the `links_backfill` post-settle path? a dedicated debounce?) + the
   idle/timer interval + acceptable staleness for the time-driven buckets (minutes? on-focus?).
5. **`most_connected`** — keep it in KH (it's CNS topology per CCS §12), or drop it from the circulatory cache?

---

## 8. What this migration is NOT

- **Not the CCS migration.** CCS = the deep left-dock registers + retiring `LinkDashboard.svelte`. MIG-073 = the
  **data layer** + pointing today's KH panel at it. CCS is built *on* this.
- **Not a `note_links` schema change** to the source of truth — only an additive derived cache table + triggers/hook.
- **Not a fix for the 1.7 GB DB bloat** — flagged separately (it slows every cold read, not just KH; likely
  FTS bloat / a `VACUUM` candidate; LL-XXX history). Worth its own investigation.

---

## 9. Recommendation

**Option B, scoped as the general circulatory-aggregate cache.** It fixes the KH freeze with the lowest risk
(snapshot can't drift), handles the time-driven buckets the others can't, reuses the existing backfill hook,
and is the exact foundation CCS §8 already assumes — so the work is not throwaway. KH stays and becomes its
first reader; CCS's Architect later decides KH's final shape. Phase 2 (Plan) follows Eisa's answers to §7.
