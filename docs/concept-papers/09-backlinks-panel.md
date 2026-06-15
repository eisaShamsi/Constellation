# 09 — Backlinks Panel (Concept Paper)

> A satellite of the Editor (the gate — see [01-Note-Editor](01-Note-Editor.md)). Pairs with [10-Outgoing Links](10-outgoing-panel.md): Backlinks is the *incoming* half, Outgoing the *outgoing* half. Both read the same in-memory link list; neither writes the link graph.

## 1. Function in hand
The **Backlinks Panel** — `src/lib/components/BacklinksPanel.svelte`, mounted by `src/routes/+layout.svelte` (right-sidebar tab and, in Sky-View inspect mode, as a left/right editor *flank*). It renders two sections per the active note: **Linked mentions** (notes that wikilink to it) and **Unlinked mentions** (notes that name it in plain text but don't link yet).

## 2. Purpose
Show, for the note in hand, *which other notes point at it* — incoming Living Links with their type badges, confidence, traversal "wear" (`×N` chip + tier gradient), and an NSC summary headline — plus *who mentions it without linking*, each with a one-click "Link it" action that inserts `[[ActiveNote]]` into the mentioning note. It serves the **Connection** Act directly (and feeds **Tension/Synthesis** by surfacing `contradicts`/`supports`-typed incoming links). It justifies itself: in a Personal Knowledge Formulation system, "what already references this idea?" is a first-order question, and the unlinked-mention → link affordance is how latent connections become first-class.

## 3. What it is NOT
- **Not** the source of truth for links — that's the LINK files on disk + the `note_links` SQLite table. The panel is a *read surface*.
- **Not** the link *editor* — it can set confidence and archive a link (right-click), but it does not create typed links or edit annotations; that's the Editor + Property/Link surfaces.
- **Not** the Outgoing panel — outgoing links live in [10-outgoing-panel](10-outgoing-panel.md). Same data array, opposite direction.
- **Not** the graph — Sky View / Map render the whole web; this panel is the active note's incoming neighbourhood only.

## 4. Wiring
- **Inputs (props from `+layout.svelte`):** `backlinks` (= `getBacklinks(effectiveLibraryLinks, tab.name, …)`), `unlinkedMentions` (= `scanUnlinkedMentions(...)`), `activeNoteName`, `activeNotePath`, `libraryColorMap`, and two callbacks `onConfidenceChange` / `onArchive`. `effectiveLibraryLinks` derives from `allLibraryLinks`, populated at boot from `cache_boot_snapshot_graph`.
- **Inputs (stores/IPC read directly):** `appSettings` (pill shape), `getSummariesFor(paths)` (MIG-044 NSC headlines, cache-first batched).
- **Outputs (IPC writes):** `setLinkConfidence(sourcePath, targetName, level)` and `archiveLink(...)` on right-click; `write_note(filePath, content, origin:'link_mention')` when "Link it" rewrites a mentioning note to insert the wikilink.
- **Outputs (events):** opens notes via `openNoteTab(...)` (row click; ctrl/middle-click = new tab).
- **Consumers:** none downstream — it is a leaf read surface. Parent re-points `currentBacklinks` after edits (`applyConfidenceLocally`/`applyArchiveLocally` mirror the DB write into memory so the row updates without a rescan).
- **Connection to the gate:** it never reads disk itself; it learns of changes only because the Editor's save fired the reindex that refreshes `allLibraryLinks` (via the boot/federation-ready graph snapshot). The panel is downstream of the Editor's mandatory dispatch.

## 5. Right-click / context menu
- **Has one — HAND-ROLLED, flag the debt.** Right-clicking a *linked-mention* row (`oncontextmenu` → `openConfMenu`) opens a bespoke fixed-position popover (`.conf-menu`) with: the four confidence levels (hypothesis / evidence / established / contested) and an **Archive link** action. This is **not** the shared `<ContextMenu>` / `buildContextMenu` (MIG-077) — it's a local `confMenu` `$state` + hand-built overlay, duplicated in the Outgoing panel ("mirrors BacklinksPanel"). **Debt:** two hand-rolled copies of the same confidence menu that should be one shared component.
- **Actions reachable ONLY by right-click:** setting link confidence and archiving a link — there is no other entry point to either from this panel.
- **Unlinked-mention rows have no context menu** (only the inline "Link it" button). Gap to weigh in bring-up: should unlinked rows offer "Link it" / "Ignore" via the shared menu too?

## 6. Multilingual
- **Localized (✓ in all 15 locales):** section headers `backlinksPanel.linkedMentions` / `unlinkedMentions`, empty state `backlinksPanel.noBacklinks`, and the confidence menu (`linkConfidence.setConfidence` / `.rightClickHint` / the four levels / `.archive`). Type-pill labels read in the **note's** language via `tIn(noteLoc(), …)` (`LinkTypePill`), not the UI's. Filter input and NSC headline carry `dir="auto"`.
- **Hardcoded English — flag:** the filter placeholder `"Filter..."` (no `$t()`), the "Link it" button `title="Link it"`, and the traversal-chip tooltip built from `fmtTraversed()` (`"today"`, `"yesterday"`, `"2d ago"`, …) + `` `Traversed N times · tier · Last: …` ``. The confidence strings use `$t(...) || '<English>'` fallbacks — localized, but the inline English fallback should be removed once keys are confirmed present (they are).
- **Bring-up action:** route `Filter...`, `Link it`, and the relative-time / traversal tooltip through `$t()` and add the keys to all 15 locales; confirm RTL of the filter + headline.

## 7. Boot behavior
- **Runs at boot?** Not as a panel mount — it paints only when its sidebar tab/flank is shown. Its **data** rides the existing `cache_boot_snapshot_graph` call that `+layout.svelte` already makes for Sky View; no panel-specific boot IPC.
- **Rule 8 status — linked half: ✅ reads-persisted.** `cache_boot_snapshot_graph` → `read_links_in_schema` is a `SELECT … FROM note_links WHERE status='active'` against the always-current SQLite index. `getBacklinks` is a pure in-memory `filter`/`sort`/`dedupe` over that array on tab change (debounced 500 ms) — a cheap projection, not a universe re-walk.
- **Rule 8 status — unlinked half: ⚠️ recomputes-on-read, but bounded.** `scanUnlinkedMentions` calls `scan_unlinked_mentions` on each tab change (debounced 500 ms). It no longer walks the tree: it narrows candidates with one FTS5 phrase `MATCH` on the title, then re-reads only those few files for the exact word-boundary/wikilink-strip gate. Acceptable today; the *purest* Rule-8 end-state would persist unlinked-mention candidates write-time, but the FTS-narrowed read is within budget.
- **Cost:** linked projection — sub-millisecond per tab (in-memory). Unlinked — one FTS query + a handful of file reads; **measure in bring-up on a 7,600-note universe** (estimated low-tens of ms, marked).

## 8. Flag / gate & bring-up position
- **Gate today:** none of its own — it ships unconditionally as a sidebar tab; its flank placement is gated by `appSettings.panelPlacements.backlinks` (`left-of-note` / `right-of-note`) **and** `skyViewInspectMode`. No `enabledFeatures.*` / `SIGHT_*` flag wraps it. Needs a **new** satellite gate to flip off in minimal mode.
- **Bring-up phase:** **Phase 2 (link surfaces)** — depends on the Editor (Phase 1, the gate), the boot graph snapshot (`cache_boot_snapshot_graph`), and the Living-Link write path (confidence/archive IPCs). Brings up alongside its twin, [10-Outgoing](10-outgoing-panel.md). The graph/Sky-View read is a shared dependency, **not** Phase 3 graph rendering.

## 9. Budget
- **Boot budget:** zero added boot IPC (rides the existing graph snapshot); no regression to `paint_ms` / `hydrated_ms`.
- **Interaction budget:** tab-switch → panel refresh debounced 500 ms; linked projection sub-ms; unlinked FTS read must stay off the keystroke path (it is — fires on tab change, not on typing) and complete well under one frame's worth of perceptible lag.
- **Regression guard:** open a note with many incoming links on a 7,600-note universe — section renders instantly; right-click → set confidence → row updates without a full rescan; "Link it" rewrites exactly one mentioning file and the row migrates from Unlinked to Linked after reindex. Measure unlinked-scan latency before/after any change to `scan_unlinked_mentions`.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** incoming links + unlinked mentions render correctly for the active note; "Link it" inserts `[[ActiveNote]]` and the row promotes.
- [ ] **Serves Constellation's core purpose:** makes the **Connection** Act visible and actionable (latent mention → typed link).
- [ ] **Wires correctly to the Editor:** updates only via the Editor-triggered reindex/graph refresh; never reads disk on its own for the linked half.
- [ ] **Right-click present + correct, SHARED not hand-rolled:** confidence/archive menu migrated to `<ContextMenu>`/`buildContextMenu` (MIG-077), deduped with the Outgoing panel.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `Filter...`, `Link it`, relative-time/traversal tooltips routed through `$t()`; pill labels in note language; filter + headline RTL verified.
- [ ] **Within budget:** instant render + right-click on a large universe; unlinked-scan latency measured and acceptable.
- [ ] **Obeys Rule 8:** linked half reads persisted `note_links`; unlinked half's FTS-narrowed read confirmed within budget (or persisted if it regresses).
- [ ] **Holds its invariants:** archived links excluded; alias-aware backlinks resolve (MIG-004 §9); same source never double-rendered (dedupe by source path).
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unlinked-scan latency unmeasured)**
Notes: Linked half is Rule-8-clean (persisted `note_links` via `cache_boot_snapshot_graph`). Two debts to close before re-enable: (1) the confidence/archive context menu is **hand-rolled** and duplicated with the Outgoing panel — fold into the shared `<ContextMenu>`; (2) **hardcoded English** in `Filter...`, `Link it`, and the relative-time/traversal tooltips. The unlinked-mention scan recomputes on read but is FTS-narrowed (no tree walk) — acceptable, verify latency on a 7,600-note universe in bring-up.
