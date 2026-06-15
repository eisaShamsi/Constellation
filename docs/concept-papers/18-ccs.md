# 18 — Constellation Circulatory System (CCS) (Concept Paper)

> A satellite of the gate. Reads the link circulation the Editor produces; never re-implements save/load/edit. See [00-Constellation](00-Constellation-Core-Concept-Paper.md) (the Five Acts, File-Over-App, Rule 8, the Editor is the gate) and [01-Note-Editor](01-Note-Editor.md).

## 1. Function in hand
The **Constellation Circulatory System (CCS)** — `src/lib/components/CCSView.svelte` (MIG-074, الجهاز الدوري للكوكبة). A full-overlay link dashboard: seven registers of "the pulse of your thinking" — Living Connections, Load-Bearing Reasoning, Cooling Inquiries, Conviction & Doubt, The Life of a Connection (lifecycle tiers), Retired Reasoning, and The Acts of Inquiry (typed-link distribution).

## 2. Purpose
Show, in one view, **which of your connections are alive, cooling, settled, contested, or retired** — so the circulation of reasoning across the whole universe is legible, not buried in per-note backlinks. It serves the Living Link Architecture's temporal layer: weight, confidence, traversal-count, last-traversed read off every link at once. Which Act: primarily **Connection** (the topology of links) feeding **Conviction** (the Conviction & Doubt register surfaces what is established vs. contested) — it is the diagnostic instrument applied to the link graph, not the note text. It justifies itself: without it, the eight link properties have no universe-scale read surface; the data exists but is invisible.

## 3. What it is NOT
- **NOT** a writer of links — it observes circulation; opening a note from a register **never** fires `constellation_link_traverse` (invariant I2b: CCS must not feed the metric it displays).
- **NOT** a per-note panel — that is Backlinks/Outgoing. CCS is universe-scoped.
- **NOT** Knowledge Health (KHD) or CNS — it cross-links to both but renders a distinct register set.
- **NOT** a recompute-on-open scanner — it reads a persisted snapshot (§7).

## 4. Wiring
- **Inputs:** one IPC read — `constellation_ccs_snapshot` → the MIG-073 `link_stats_cache` snapshot (8 cache keys: stats, lifecycle, ccs_living/load_bearing/cooling/contested/tiers/retired). Stores read: `linkTypesStore`, `linkTypeRegistry` (canonical type order/colors), `$t` (i18n). Event listened: `kh-snapshot-ready` (registered before first fetch; stale snapshot renders instantly, updates in place). "Show all" retired = one bounded live query `listArchivedLinks()` (the I1 user-initiated carve-out).
- **Outputs:** no write path of its own. Restore goes through the **existing** lifecycle command `unarchiveLink()` (`_link_unarchive`) — optimistic local row removal, cache true-ups on next refresh. Callbacks up to host: `onNoteClick` (open source note), `onClose`, `onOpenKnowledgeHealth`, `onOpenCns`.
- **Consumers:** `+layout.svelte` mounts it under `{#if showCCS}`; reachable from the left-dock button, command palette (`ccs`), and the `constellation:open-ccs` event (MIG-074 §D re-pointed Settings → Links here).
- **Connection to the Editor (the gate):** indirect and read-only. The Editor's saves drive the `note_links` write path; the snapshot cache is recomputed off that data; CCS reads the cache. Clicking a register row dispatches an open via `onNoteClick` — it does not edit. It is a pure downstream display, obeying "additional surfaces are displays, not domains."

## 5. Right-click / context menu
**No context menu.** Grep of `CCSView.svelte` for `contextmenu` / `oncontextmenu` / `ContextMenu` / `buildContextMenu` returns nothing. Register rows are left-click buttons (open the source note); Retired rows carry an inline **Restore** button. **Gap flagged:** per the core paper §5 ("right-click should include every aspect of the app"), CCS arguably should expose a shared `<ContextMenu>` (MIG-077) on each register row — candidate items: *Open source*, *Open target*, *Copy link annotation*, *Archive/Restore link*, *Set confidence*, *Reveal in Backlinks*. Today Restore is reachable only via its dedicated button (not a menu), and no per-row link operations exist beyond open. **Bring-up action:** add the shared context menu to register rows; do **not** hand-roll one. Marked as debt — `unknown which items the Boss wants — verify in bring-up`.

## 6. Multilingual
Conformant. Every user-facing string in the component flows through `$t()`; the `ccs` block exists in **all 15 locale files** (ar de en es fa fr he hi ja ko pt ru tr ur zh) with **native equivalents** (Arabic: "الجهاز الدوري للكوكبة", not transliteration). Note names carry `dir="auto"`; the row line aligns to the row's direction (the KHD RTL lesson, `text-align: match-parent`). Label resolvers fall back locale → registry label → raw id (no English literal leaks). No hardcoded English string found **inside CCSView**. Note: the dock-button `title` and command-palette name in `+layout.svelte` use `$t('ccs.title') || 'Constellation Circulatory System'` — the fallback is an English literal, but it only fires if the locale key is missing (it is present in all 15). `bar-num`/dates use `toLocaleString()` / `toLocaleDateString()`.

## 7. Boot behavior
- **Runs at boot?** No render at boot — the component exists only while `showCCS` is true ({#if} unmount, LL-022: closed CCS does zero IPC). The cache it reads is **first-populated** in the background after paint via `spawn_kh_cache_recompute(app, only_if_empty=true)` from `cache_mark_search_ready` (MIG-073 one-off backfill, dedicated connection, never blocks boot).
- **Rule 8 status:** **reads-persisted** (compliant in spirit) — `constellation_ccs_snapshot` reads the stored `link_stats_cache`; it does **not** scan `note_links` on open. Caveat for the bring-up: the cache is maintained by **stale-while-revalidate full recompute** (on first population + when older than `KH_CACHE_FRESH_MINUTES`), **not** by a per-link write-time trigger like the FTS5 canonical example. So it is a persisted-snapshot read, not a trigger-maintained derivation. Verify in bring-up whether the recompute should be wired to the `note_links` write path to fully match Rule 8's trigger model.
- **Cost:** open = one indexed cache read (estimated low single-digit ms; `unknown — measure in bring-up`). Background recompute walks `note_links` once on a worker thread — cost scales with link count; measure on the 7,600-note universe before re-enable.

## 8. Flag / gate & bring-up position
- **Gate today:** `$appSettings.enabledFeatures?.ccs !== false` — a **default-ON Core Plug-in**. Dock button, command-palette entry, and `constellation:open-ccs` listener all check it. No SIGHT_* flag involved.
- **Bring-up phase + depends on:** **Phase 5 (curation / link surfaces).** Depends on: the Editor (gate) driving the `note_links` write path; the MIG-073 `link_stats_cache` layer being populated; the link-type registry; the lifecycle commands (`_link_unarchive`). Bring up after the core spine + search/links are proven.

## 9. Budget
- **Boot budget:** zero boot cost (not mounted at boot). The one-off backfill must stay off the boot pipeline (MIG-067 walk-free boot intact).
- **Interaction budget:** open → render from cached snapshot with no perceptible lag; Restore is one IPC, optimistic UI. No `invoke()` on any hot path; the only per-open IPC is the single snapshot read.
- **Regression guard:** opening CCS must fire exactly one `constellation_ccs_snapshot`; closing fires zero IPC (LL-022); opening a row must **not** fire `constellation_link_traverse` (I2b); measure backfill cost on a 7,600-note universe before re-enable.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** the seven registers render correct counts/rows from the snapshot; Restore round-trips a link back into Backlinks/Outgoing.
- [ ] **Serves Constellation's core purpose:** advances Connection→Conviction (the link circulation made legible); no register is decorative filler (Form-Aligns-To-Purpose).
- [ ] **Wires correctly to the Editor:** read-only downstream; opening a row opens the note without editing; never re-implements save/load.
- [ ] **Right-click present + correct:** add the shared `<ContextMenu>` to register rows (per-link actions); **not** hand-rolled — currently **absent (gap)**.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** all `ccs.*` keys in 15 locales; `dir="auto"` rows; native equivalents; remove/justify the `+layout.svelte` English fallback literals.
- [ ] **Within budget:** zero boot cost; one IPC per open; zero IPC when closed.
- [ ] **Obeys Rule 8:** reads persisted snapshot (✔); decide in bring-up whether to wire the recompute to the `note_links` write-time trigger for full conformance.
- [ ] **Holds its invariants:** I2b (open ≠ traverse); I2 (no new write path; Restore via existing command); I1 (Show-all is the only live query, bounded).
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (open-cost unmeasured)** · Notes: Component shipped under MIG-074, default-ON gate `enabledFeatures.ccs`. Two items for bring-up: (1) **no right-click menu** — add shared `<ContextMenu>` on register rows (gap vs. core §5); (2) **Rule 8 is persisted-read but recompute-based, not trigger-maintained** — confirm whether to wire it to the `note_links` write path. Multilingual and the no-traversal invariant (I2b) are clean in the source as read. Open-IPC cost and backfill cost on the 7,600-note universe are `unknown — measure in bring-up`.
