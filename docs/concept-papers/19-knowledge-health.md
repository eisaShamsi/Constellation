# 19 — Knowledge Health Dashboard (Concept Paper)

> A per-function paper under [00-Constellation](00-Constellation-Core-Concept-Paper.md). It must trace its existence to the Five Acts and obey Rule 8. Template per [01 — Note Editor](01-Note-Editor.md).

## 1. Function in hand
The **Knowledge Health** dashboard — `src/lib/components/KnowledgeHealthDashboard.svelte`, a full-page overlay opened from the dock button / command palette (`ribbon.knowledgeHealth`). Its backend cache (MIG-073) is the `link_stats_cache` table, read via the `constellation_knowledge_health_snapshot` IPC. Sibling of the Circulatory System (CCS) panel, which shares the same cache.

## 2. Purpose
Show, at a glance, the **shape and vitality of the user's link graph**: lifecycle-stage counts, total/annotated links, distribution by cognitive type and by confidence, plus three curated lists — *Emerging ideas* (high-weight links), *Weak foundations* (hypothesis-confidence links), *Potential bias* (targets supported with no opposing/`contradicts` view). It serves **Tension** above all (surfacing weak foundations and one-sided support is the diagnostic that provokes the user to challenge their own structure), and supports **Connection** (it makes the link graph visible as a whole). It justifies itself under the core test: a PKF system measures *awareness of connections*, not storage volume — this is the panel that makes that awareness legible.

## 3. What it is NOT
- **Not** a per-note view — it is a universe-wide aggregate over `note_links`, not a single-note inspector.
- **Not** an editor of links — it is read-only display; it dispatches no writes and owns no note state.
- **Not** the topology/hub register — the Most-Connected card was retired (MIG-075 §B2); hubs live in CNS. The retired `most_connected` key is still emitted by the cache but not rendered.
- **Not** the same surface as CCS — CCS renders the 5-tier *usage* census from the same cache; KH renders the 6-stage life arc + the three curated lists.

## 4. Wiring
- **Inputs (IPC):** `constellation_knowledge_health_snapshot` (the one call on open). **Events:** listens for `kh-snapshot-ready` the whole time the panel is open (re-renders in place when a background refresh lands). **Stores read:** `linkTypesStore` / `linkTypeColor` / `getLinkType` (link-type registry, MIG-067) for bar colours + pill labels; `$t` for locale.
- **Outputs:** none to disk — read-only. Calls `onClose` (clears `showKnowledgeHealth`) and optional `onOpenCcs` (deep-link to CCS, present only when `enabledFeatures.ccs !== false`). The snapshot IPC may *kick* a background recompute (stale-while-revalidate), but the panel itself writes nothing.
- **Consumers:** none depend on it — it is a leaf display. Its cache (`link_stats_cache`) is shared *with* the CCS snapshot IPC (one reader, `read_link_stats_cache`).
- **Connection to the Editor (the gate):** indirect and downstream. The Editor's save fires the reindex that maintains `note_links`; bulk link changes settle through `cache_reconcile`, which then calls `kh_cache_recompute_blocking`. KH never reads the Editor's in-memory state and never re-implements save/load — it reads the persisted cache the gate's writes ultimately feed. It mounts as a top-level overlay in `+layout.svelte` (`{#if showKnowledgeHealth}`), not inside the editor pane.

## 5. Right-click / context menu
**None.** Grep of the component for `contextmenu` / `oncontextmenu` / `ContextMenu` / `buildContextMenu` returns no matches — there is no right-click handler on any row, card, or insight name. **Gap flagged:** the three insight lists (Emerging / Weak foundations / Bias) and the type/confidence bars render note names and link rows that the user can currently only *look at* — there is no way to open the note, open the link, traverse it, or jump to CCS for that row. Per the core paper §5 ("right-click should include every aspect of the app"), these rows *should* expose at least "Open source note / Open target note / Open link" via the **shared `<ContextMenu>` / `buildContextMenu()` (MIG-077)** — to be designed and added in bring-up. No hand-rolled menu debt exists (there is simply no menu); the bring-up must add the shared one, not invent a local one.

## 6. Multilingual
**Conformant.** Every user-facing string flows through `$t()` — `knowledgeHealth.{title,loading,annotated,byType,byConfidence,weakFoundations,bias,totalLinks,emerging,emergingEmpty,weakFoundationsEmpty,biasEmpty,supportsOnly,weightAbbrev}` + `knowledgeHealth.confidence.*`; type labels via `linkTypes.*` (registry fallback to user label / raw id), stage labels via `notePane.stage.*`. The `knowledgeHealth` block is present in **all 15 locale files** (ar de en es fa fr he hi ja ko pt ru tr ur zh) — verified by grep. **RTL:** note-name and bar-label spans carry `dir="auto"`; `.khd-insight-name` uses `text-align: match-parent` so a row aligns to the row's direction, not the name's resolved one (Boss fix, 2026-06-10). **No hardcoded English found in the component.** Minor note: the dock `title` and command-palette entry have an English `|| 'Knowledge Health'` fallback, but the primary path is `$t()`; verify the fallback never shows in any locale during bring-up.

## 7. Boot behavior
- **Runs at boot?** Only the **one-off first population**: `cache_mark_search_ready` calls `spawn_kh_cache_recompute(app, only_if_empty = true)`. On every later boot that is a single `COUNT(*)` on `link_stats_cache` that finds the cache non-empty and returns immediately — no `note_links` scan, honoring MIG-067's zero-boot-walks rule.
- **Rule 8 status:** ✅ **reads-persisted.** The panel reads `link_stats_cache` (6 tiny rows) on open; the snapshot IPC never re-walks `note_links` on read. The derived view is maintained off the write path (post-reconcile recompute) + stale-while-revalidate (2-minute freshness). This *is* the canonical Rule 8 fix — MIG-073 replaced the old read-time path where the panel fired six live aggregates and cold-read a 1.7 GB table (~11 s under the DB mutex).
- **Cost:** open = one cheap row read (estimated sub-millisecond on a warm cache). First-ever population = one background recompute over `note_links` (cost scales with link count; runs off-thread, panel shows its loading state). Boot adds one `COUNT(*)` (negligible). *Not measured on the 7,600-note universe — verify in bring-up.*

## 8. Flag / gate & bring-up position
- **Gate today:** **no dedicated flag.** There is no `enabledFeatures.knowledgeHealth`; the panel is an unconditional dock button + command-palette entry driven only by `showKnowledgeHealth` state. The only `enabledFeatures` check is the optional `ccs` deep-link. **Needs a new gate** (`enabledFeatures.knowledgeHealth` or a `SIGHT_*`-style guard) so the bring-up can flip it on/off independently.
- **Bring-up phase + dependencies:** a **curation / dashboard** function (phase 5-class — display-of-derived). Depends on: the search index + `note_links` populated (the Editor's write path), the MIG-073 `link_stats_cache` (its Rule-8 store), the link-type registry (MIG-067), and `cache_reconcile`/`cache_mark_search_ready` boot hooks. Re-enable after the core spine + link-cache are proven.

## 9. Budget
- **Boot budget:** add nothing measurable beyond one `COUNT(*)`; first population stays off the paint thread. Must not regress the MIG-067 zero-boot-walks baseline.
- **Interaction budget:** open ≤ one cheap cache read; no `invoke()` loop, no per-row IPC; background refresh must not block the UI thread. Single-recompute-in-flight guard (`KH_RECOMPUTE_IN_FLIGHT`) prevents pile-up.
- **Regression guard:** open the panel on a large universe (7,600+ notes / ≥200k links) — first open shows loading then resolves via `kh-snapshot-ready`; a second open is instant; verify no `note_links` scan fires on read (only on recompute). Confirm no DB-mutex stall during open.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** lifecycle cards, type/confidence bars, and the three curated lists render correct counts against a known fixture.
- [ ] **Serves Constellation's core purpose:** advances **Tension** (weak foundations + bias) and **Connection** (graph made visible) — traces to [00-Constellation](00-Constellation-Core-Concept-Paper.md) §2.
- [ ] **Wires to the Editor correctly:** reads only the persisted cache the gate's writes feed; dispatches no writes; re-implements no save/load (display, not domain).
- [ ] **Right-click present + correct:** the **gap is closed** — insight rows expose Open source / Open target / Open link via the **shared `<ContextMenu>` / `buildContextMenu()` (MIG-077)**, not a hand-rolled menu.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** all strings `$t()` in 15 locales; `dir="auto"` + `match-parent` alignment verified in an RTL locale; English fallbacks never surface.
- [ ] **Within budget:** large-universe open is instant on a warm cache; no boot walk; no DB-mutex stall.
- [ ] **Obeys Rule 8:** read path touches only `link_stats_cache`; no read-time `note_links` aggregate.
- [ ] **Holds its invariants:** completeness judged on the 6 KH keys only (missing `ccs_*` keys never break this panel); `{ready:false}` self-heals via background populate; retired `most_connected` stays unrendered.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (not measured on the large universe — verify in bring-up)**
Notes: Rule-8 status is its headline — MIG-073 already converted this from a read-time aggregate (the 1.7 GB / ~11 s cold-read) into a persisted-cache read, so the central bring-up question is *settled* here. Two open items for bring-up: (1) add the **shared right-click menu** on insight rows (currently no context menu at all); (2) add a **dedicated feature gate** (`enabledFeatures.knowledgeHealth`) so it can be flipped independently. Most-Connected card retired (MIG-075 §B2 — hubs live in CNS).
