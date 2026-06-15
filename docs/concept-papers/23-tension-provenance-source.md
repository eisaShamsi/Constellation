# 23 — Tension, Provenance & Source Review (Concept Paper)

> Three sibling right-sidebar surfaces that turn the stored note graph into *reflective instruments*: where does my thinking conflict, where did a note come from, and what does the cataloger think this note is? They serve the **Tension** and **Connection** Acts. Each attaches to the Editor (the gate) read-only — none writes note content except through the user's explicit Accept.

## 1. Function in hand
Three panels under the right sidebar (and one full page):
- **Knowledge Health** — `src/lib/components/TensionPanel.svelte` (the `health` tab; the UI title is "knowledge health monitoring").
- **Provenance** — `src/lib/components/ProvenancePanel.svelte` (the `provenance` tab).
- **Source Review** — `src/lib/components/SourceReviewPanel.svelte` (the `sourceReview` tab; also embedded full-page in `CatalogerView.svelte`). MIG-021v2/v3 source+content-type classifier queue.

## 2. Purpose
- **Tension** serves the **Tension** Act directly — it surfaces contradictions, orphans, structural gaps, and single-points-of-failure so the user *sees where the knowledge base disagrees with itself or is under-connected*. This is the diagnostic-instrument idea made literal.
- **Provenance** serves **Connection** (and Conviction's evidentiary backing) — it traces a note's `derives-from` lineage and marks which ancestors carry an external source, answering "where did this come from, and how far back is it grounded?"
- **Source Review** serves **Connection / Synthesis** — it lets the user confirm or correct the cataloger ensemble's classification of each note along two axes (sources / content-type), turning raw notes into a navigable epistemic taxonomy.
All three justify their existence: each advances a named Act and none merely "stores." (Provenance and Tension currently lean on read-time recompute — see §7 — which is the bring-up's chief correction, not a reason to cut them.)

## 3. What it is NOT
- **Not** editors — they never mutate note *body* text. Source Review writes only frontmatter axis fields, and only on explicit Accept.
- **Not** Constellation Map / Sky View — no graph canvas; these are list/tree readouts.
- Tension is **not** a linter that auto-fixes; it reports.
- Provenance is **not** the full backlink graph — it follows only the `derives-from` typed link.

## 4. Wiring
- **Inputs (IPC read):** Tension → `detect_tensions(library_path, library_name)`; Provenance → `get_provenance_chain(library_path, note_path, max_depth)`; Source Review → `sources_list_pending_suggestions`, plus `classifier_suggest_for_note` (classify the open note), `getSummariesFor` (shared NSC summary store), `getHorizontalTaxonomy`/`getVerticalTaxonomy`.
- **Inputs (events):** Source Review listens for `constellation:classify-and-show` (DOM event from the right-click action) and Tauri events `classifier:scan` (live queue reload) + `sources:bulk_accept` (progress).
- **Outputs (IPC write):** Source Review only — `sources_set_manual`, `content_type_set_manual`, `cece_record_correction_for_card`, `sources_reject_suggestion`, `sources_accept_all_pending`, `sources_reject_all_pending`, `cece_resolve_disambiguation`. Tension and Provenance are **read-only** (no writes).
- **Consumers:** the host `+layout.svelte` owns `tensionReport`/`provenanceChain` state and feeds the panels via props; `CatalogerView` mounts a second Source Review instance and syncs both via the `constellation:classify-and-show` window event. `onNoteClick` opens the target as a tab.
- **Connection to the Editor (the gate):** all three are **displays** — they read the active note path (`sidebarTab?.path`) and call `openNoteTab` to hand navigation back to the Editor. Source Review's writes go through Rust commands that the Editor's reindex path observes; the panels never re-implement save/load. The right-click "Suggest sources" action originates from the shared menu and routes a note into the Source Review queue.

## 5. Right-click / context menu
- **The panels themselves have NO context menu** — grep of all three `.svelte` files shows no `oncontextmenu` / `on:contextmenu` / `ContextMenu` / `buildContextMenu`. Rows are plain `<button>`s (left-click opens the note).
- The **producer** action lives in the **shared** builder (`src/lib/components/contextMenuBuilder.ts`, MIG-077): `suggestSources` → label `sources.contextMenu.suggest` (✨), gated to markdown targets (`target.isMarkdown`). This is correctly shared, **not** hand-rolled. It dispatches `constellation:classify-and-show`, which Source Review consumes.
- **Gap to flag:** Source Review note-row cards, Tension result rows, and Provenance ancestor rows offer **only left-click**. Per the core paper §5 ("right-click should include every aspect of the app"), these rows arguably *should* expose a row-level menu (open in surface, reveal in tree, copy path; for SRP: Accept / Reject / Edit / "re-classify"). Today Accept/Reject/Edit are reachable only via on-card buttons, never by right-click. **Bring-up action:** add a shared `<ContextMenu>` to the rows, or consciously record "none-ok" — do not hand-roll.

## 6. Multilingual
- **Localized ×15 — verified.** Every user-facing string uses `$t('…') || 'English fallback'`. The `tensionPanel.*`, `provenancePanel.*`, `sources.review.*`, `sources.contextMenu.*`, and `cece.*` namespaces are present in all 15 locale files (ar de en es fa fr he hi ja ko pt ru tr ur zh) — confirmed by grep across the locale dir.
- **RTL / bidi:** Source Review root + summary use `dir="auto"`; Tension/Provenance use logical CSS (`padding-inline-start`, `margin-inline-start`) and flip the chevron under `[dir="rtl"]`. Source Review resolves taxonomy IDs, cataloger names, confidence enums, and rules-fired to localized labels via `cece.taxonomy.*` / `cece.cataloger.*` / `cece.confidence.*` / `cece.rule.*` (MIG-022 §E), with raw-string fallback.
- **No hardcoded English found in the rendered path** — English appears only as `||` fallbacks behind `$t()`, which is the established pattern. The inline hex colors (severity dots, origin colors, cataloger hues) are not user-facing strings.

## 7. Boot behavior
- **Runs at boot?** The panels do **not** run at boot — they fetch lazily on tab focus / note focus. The Source Review *queue producer* (`classifier_scan_start`) optionally runs at boot **only** if `cece.backgroundScan === 'on_startup'`, deferred 5 s, non-blocking, on a background thread — it does not block first paint.
- **Rule 8 status — the central finding (mixed):**
  - **Source Review = reads-persisted ✅.** `sources_list_pending_suggestions` reads suggestion rows already written to the DB by the classifier scan (write-time). Compliant.
  - **Tension = RECOMPUTES-on-read ⚠️ (partial).** `detect_tensions` runs the full contradiction/orphan/gap detection on every `health`-tab focus (guarded only by a per-library cache). Its *inputs* are persisted (loads `note_meta` + `note_links` from the DB; the old fs walk was retired), but the *report* is not stored — it is rebuilt each time. **Fix in bring-up:** persist the tension report, maintain it via the note-write triggers.
  - **Provenance = RECOMPUTES-on-read ⚠️ (full violation).** `get_provenance_chain` does a `scan_notes_recursive` **fs walk** of the whole library on every note focus, bypassing the DB. **Fix in bring-up:** read `derives-from` edges from `note_links` (already indexed) instead of re-walking disk; persist/cache the chain.
- **Cost:** unknown — verify in bring-up. Tension is `O(notes + links)` per focus from the DB (estimated cheap to mid on a 7,600-note universe); Provenance's fs walk is the suspect cost (estimated the worst of the three on a large library). Neither measured.

## 8. Flag / gate & bring-up position
- **Gate today:** Source Review → `enabledFeatures.cece !== false` (the Cataloger dock + queue). Tension and Provenance have **no dedicated feature flag** — they are right-sidebar tabs governed only by `panelPlacements.{provenance,…}`; minimal mode needs a **new gate** (e.g. `enabledFeatures.knowledgeHealth` / `enabledFeatures.provenance`) to flip them off cleanly.
- **Bring-up phase:** **Phase 5 (Curation / reflection surfaces)** — these are downstream reflective instruments, not core spine. Depend on: the Editor (active-note path + reindex), the search DB (`note_meta`/`note_links`), and — for Source Review — the CECE classifier subsystem + the dual-axis taxonomies. Tension/Provenance must have their Rule-8 recompute converted to persisted/triggered reads **before** re-enable.

## 9. Budget
- **Boot budget:** zero — none may run on the boot path. The optional CECE startup scan stays deferred + background; first paint must not wait on it.
- **Interaction budget:** tab/note focus → panel populated in < ~150 ms on a large universe (target; unmeasured). After Rule-8 conversion, focus should be a cheap indexed lookup, not a recompute. Accept/Reject must feel instant (the write is debounced/async; the card drops optimistically).
- **Regression guard:** focus the `health`, `provenance`, `sourceReview` tabs on a 7,600-note universe and measure focus-to-paint before/after; assert no `invoke()` on any keystroke path; assert Provenance no longer fs-walks. Source Review render is capped (`RENDER_BATCH = 80`) — keep the cap; verify scrolling a thousands-row queue stays smooth.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** Tension surfaces real contradictions/orphans/gaps; Provenance traces `derives-from` lineage with external-source marks; Source Review classifies + the user can Accept/Reject/Edit/disambiguate.
- [ ] **Serves Constellation's core purpose:** each maps to its Act (Tension → Tension; Provenance → Connection; Source Review → Connection/Synthesis) per [00](00-Constellation-Core-Concept-Paper.md).
- [ ] **Wires correctly to the Editor:** panels read the active note + `openNoteTab` back; Source Review writes only via Rust commands the reindex observes; no re-implemented save/load.
- [ ] **Right-click present + correct:** the "Suggest sources" producer is shared (MIG-077) ✅; **add** shared `<ContextMenu>` row menus (Accept/Reject/Edit; open-in-surface; reveal-in-tree) or record "none-ok" — never hand-rolled.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** verified for all three (re-confirm after any string change).
- [ ] **Within budget:** focus-to-paint measured on a large universe; no boot cost; SRP render cap holds.
- [ ] **Obeys Rule 8:** **Provenance** reads `note_links` instead of fs-walking; **Tension** report persisted + trigger-maintained; **Source Review** already compliant.
- [ ] **Holds its invariants:** read-only panels never write note bodies; Source Review writes only on explicit Accept and is reversible; queue producer is the classifier, not the panel.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unmeasured)** · Notes: Files confirmed (`TensionPanel`, `ProvenancePanel`, `SourceReviewPanel`; hosts `+layout.svelte` + `CatalogerView.svelte`). **Two Rule-8 corrections are the headline bring-up work:** Provenance still fs-walks the library on every note focus (full violation); Tension recomputes its report on every tab focus (partial — inputs are DB-persisted, report is not); Source Review reads persisted rows (compliant). Right-click producer is shared; panel *rows* have no right-click menu (gap to fill or consciously decline). Tension/Provenance lack a dedicated feature flag — a new gate is needed for minimal mode.
