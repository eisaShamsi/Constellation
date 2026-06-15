# 11 — Tags Panel (Concept Paper)

> Satellite of the Editor (the gate). The Tags Panel is a **read-only browser** over the universe-wide tag vocabulary — it never writes a note. Per [00-Constellation](00-Constellation-Core-Concept-Paper.md): tags are a derived view; the Editor emits the change, the panel displays the aggregate.

## 1. Function in hand
The **Tags Panel** — `src/lib/components/TagsPanel.svelte`, the federated tag tree shown in the right sidebar's **Tags** tab under the **"All tags"** sub-view. The same tab's **"This note"** sub-view is hand-rendered chips in `src/routes/+layout.svelte` (not this component); `TagsPanel` is specifically the universe-wide ("All tags") browser, fed by `allLibraryTags` and `handleTagClick`.

## 2. Purpose
Show every tag across the whole universe as a collapsible **`parent/child` hierarchy** with per-tag counts, sortable A→Z / Z→A / by-count and filterable, so the user can navigate into a tag's notes with one click. It serves **Connection** (the second Act): tags are a lightweight non-typed grouping that lets the user see "what have I gathered under this idea" and jump to it. Clicking a tag opens the federated **Search Hub** scoped to `#tag`. It justifies itself as a fast, zero-cost map of the user's own labelling vocabulary — a diagnostic surface, not storage.

## 3. What it is NOT
- **Not** a tag *editor* — it cannot rename, merge, delete, or create tags. It is display + navigate only.
- **Not** the "This note" tag list — that's a separate inline render in `+layout.svelte` (the `activeNoteTags` chips); this paper covers the **"All tags"** federated tree only.
- **Not** a typed-link surface — tags are flat labels, distinct from the 8 typed links (those are the Living Link Architecture, not tags).
- **Not** a writer of any kind — it dispatches a navigation intent, nothing else.

## 4. Wiring
- **Inputs (props):** `tags: Record<string, count>` (`allLibraryTags` in `+layout.svelte`) and `onTagClick(tag)` (`handleTagClick`). Pure props — the component reads no store and invokes no IPC itself.
- **Source of `allLibraryTags`:** the `cache_boot_snapshot_graph` IPC (`src-tauri/src/cache.rs`), which returns `{ links, tags, aliases }`. Also refreshed by `cache_boot_snapshot_graph2` and the federation-ready re-invoke.
- **Outputs:** none to disk/IPC. `onTagClick(fullPath)` → `handleTagClick` sets `searchHubInitialQuery = "#tag"` and opens Search Hub (`showSearchHub = true`), closing the other full-page overlays.
- **Consumers:** the Search Hub (receives the `#tag` query). Internal-only state: `sortMode`, `expanded` set, `filterQuery` — all local `$state`, no cross-surface effect.
- **Connection to the Editor (the gate):** indirect and correct. When the Editor saves a note it fires `constellation_search_reindex`, which updates each note's `tags_json` in `note_meta`; the next `cache_boot_snapshot_graph` re-aggregates the count map the panel renders. The panel never reads disk or the Editor directly — it only ever sees what the Editor's reindex wrote. No silent read path.

## 5. Right-click / context menu
- **Has one? NO.** Grep of `TagsPanel.svelte` for `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` returns **zero matches**. Every interaction is left-click (sort buttons, the expand chevron, the tag row).
- **Gap — flag it.** A tag browser is a natural right-click target: "Open in Search Hub", "Copy `#tag`", "Open in new pane", "Find/replace this tag across notes", "Rename tag" (a future write op) all belong on a context menu. Today none exist; the only action is the single left-click → Search Hub. **Bring-up should add a shared `<ContextMenu>` (MIG-077 pattern) per-tag**, not a hand-rolled menu. Recommended minimum: *Open in Search Hub*, *Copy `#tag`*. Rename/merge are write ops and out of scope until a tag-mutation path exists — do not invent them here.
- **Reachable only by right-click today:** none (there is no right-click).

## 6. Multilingual
- **Localized strings (✓):** `tagsPanel.noTags`, `tagsPanel.sortAz`, `tagsPanel.sortZa`, `tagsPanel.sortCount` all flow through `$t()` and exist in the locale files (the `tagsPanel` block is present in en/es/hi/pt/… — verify the full ×15 set in bring-up). The sort-button `title` tooltips use `$t(...) || 'A→Z'` fallbacks.
- **Hardcoded English — FLAG:** the filter input's `placeholder="Filter tags..."` is a **literal English string with no `$t()` key** (`TagsPanel.svelte` line 82). This must be moved to a locale key (e.g. `tagsPanel.filterPlaceholder`) and added to all 15 locales (ar de en es fa fr he hi ja ko pt ru tr ur zh). The on-row sort labels `A→Z` / `Z→A` / `#` are glyph buttons (script-neutral) — acceptable, but their `title` tooltips already localize.
- **RTL / bidi (✓):** `dir="auto"` is set on the filter input **and** on each `<span class="tp-name">` tag name, so mixed-script and Arabic/Hebrew tags align correctly. `padding-inline-start` is used for indentation (logical, RTL-safe). No `detectDir()` import is needed here because `dir="auto"` covers per-tag direction.

## 7. Boot behavior
- **Runs at boot?** The component itself does not. Its data (`allLibraryTags`) arrives via `cache_boot_snapshot_graph`, which is **deferred** (fired via `requestIdleCallback` after `boot:hydrated`, per `cache.rs` header §29-33) — it is **not** on the initial-paint critical path.
- **Rule 8 status: PARTIAL VIOLATION — RECOMPUTES-on-read.** The per-note tag arrays ARE persisted write-time (`note_meta.tags_json`, written by the Editor's reindex). But the **`tag → count` map the panel consumes is re-aggregated on every graph snapshot** by scanning every `note_meta` row and summing `tags_json` (`read_tags_in_schema`, `cache.rs:1027`). There is **no persisted `tag_counts` table maintained by a trigger**. This is exactly the §Rule-8 audit-pending shape ("Tag browser scanned on open"). The fix: persist a `tag_counts(tag, count)` derived table, maintain it via the same `note_meta_ai/ad/au` trigger family that keeps FTS5 current, and have `cache_boot_snapshot_graph` read it directly.
- **Cost:** one `SELECT tags_json FROM note_meta` full scan + JSON parse per note per schema. On a 7,600-note universe this is the dominant term in `read_tags` (estimated low-tens of ms; **measure in bring-up** — exact figure unknown). Off the paint path, so it does not regress first paint, but it does re-run on every graph refresh.

## 8. Flag / gate & bring-up position
- **Gate today:** none in the component. It is unconditional core UI, surfaced only when the right-sidebar Tags tab is set to **"All tags"** (`tagView === 'all'` in `+layout.svelte`) and `allLibraryTags` is non-empty. No `enabledFeatures.*` / `SIGHT_*` flag wraps it.
- **Bring-up phase:** **Phase 2 (satellites that read the graph snapshot)** — alongside Backlinks/Outgoing/Index, all of which depend on `cache_boot_snapshot_graph`. Depends on: the Editor's reindex write path (Phase 1) + the deferred graph snapshot. Phase 3 (graph/Sky View) is downstream and unrelated.

## 9. Budget
- **Boot budget:** zero on first paint (deferred). The graph-snapshot tag aggregation must stay within the post-hydration idle window; target ≤ the existing `read_tags` timing on a 7,600-note universe (baseline to be captured in bring-up).
- **Interaction budget:** sort / expand / filter are pure in-memory `$derived` recomputes over an already-loaded map — must be instant (Rule 1). No `invoke()` on any interaction (verified: the component issues none).
- **Regression guard:** load the "All tags" view on a 7,600-note universe; toggle all three sort modes and type in the filter — no perceptible lag. Confirm `cache_boot_snapshot_graph` stays off the paint path. Re-measure `read_tags` before/after any tag-count change.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** all universe tags render as a `parent/child` tree with correct counts; sort A→Z / Z→A / by-count works; filter narrows the tree; click opens Search Hub scoped to `#tag`.
- [ ] **Serves Constellation's core purpose:** it is a **Connection** surface — a fast map of the user's own labelling, navigable in one click; no storage-management framing.
- [ ] **Wires correctly to the Editor:** a tag added in a note (Editor → reindex → `tags_json`) appears in this panel after the next graph snapshot; counts are accurate; no direct disk read.
- [ ] **Right-click present + correct:** add a **shared `<ContextMenu>`** (MIG-077), NOT hand-rolled, with at least *Open in Search Hub* + *Copy `#tag`*; currently **absent — must be added**.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** move `"Filter tags..."` to a `$t()` key in all 15 locales; verify `tagsPanel.*` keys complete in all 15; confirm `dir="auto"` renders Arabic/Hebrew/CJK tags correctly.
- [ ] **Within budget:** off the paint path; interactions instant on a 7,600-note universe.
- [ ] **Obeys Rule 8:** **currently does not** — persist a trigger-maintained `tag_counts` table and read it, instead of re-aggregating `tags_json` on every snapshot.
- [ ] **Holds its invariants:** read-only (never mutates a note); counts equal the sum of per-note `tags_json`; `expanded`/`filter` state is local and never leaks across notes.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unmeasured)**
Notes: Two real debts surfaced and verified in code, not assumed: (1) **Rule 8** — the tag-count map is recomputed on every graph snapshot (`read_tags_in_schema`, `cache.rs:1027`) rather than read from a trigger-maintained table; this is the audit-pending "tag browser scanned on open" item from CLAUDE.md §Rule 8. (2) **i18n** — the filter placeholder `"Filter tags..."` is hardcoded English (`TagsPanel.svelte:82`). One more gap: **no right-click menu** at all — a shared `<ContextMenu>` should be added in bring-up. The component is otherwise clean: pure props, no IPC of its own, `dir="auto"` on tag names + filter, logical-property indentation. Exact `read_tags` cost on a large universe is **unknown — verify in bring-up**.
