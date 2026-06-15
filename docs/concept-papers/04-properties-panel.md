# 04 — Properties Panel (Concept Paper)

> A satellite of the gate. Attaches to the Note Editor ([01](01-Note-Editor.md)) and serves the same `.md` file. Frames per the core paper ([00](00-Constellation-Core-Concept-Paper.md)): Five Acts, File-Over-App, Rule 8, the Editor is the gate.

## 1. Function in hand
The **Properties Panel** — the frontmatter (YAML) properties editor: `src/lib/components/PropertyEditor.svelte`. It renders one editable row per frontmatter key with a typed value control (text, number, date, datetime, list/tags, link, checkbox, nested-object-list), plus the special `stage` combobox and the `sources:` / `content_type:` taxonomy pickers.

## 2. Purpose
Edit a note's **frontmatter** — its structured metadata — without hand-typing YAML. One job: let the user attach typed attributes (tags, aliases, dates, links, stage, sources) to a note as first-class editable controls. It primarily serves **Connection** (tags/aliases/`link` typed values weave a note into the rest of the Library) and **Conviction** (the `stage` lifecycle field records where a note's thinking stands). It justifies itself: frontmatter is part of the `.md` source of truth (File-Over-App), and editing it raw is error-prone — the panel makes structured metadata safe and fast.

## 3. What it is NOT
- **Not** the body editor — it owns the YAML block only; the body is passed through untouched (`body` prop) and re-stitched on save.
- **Not** a derived-view computer — it does not recompute backlinks/tags/graph; it writes the file and the Editor's save path fires the reindex.
- **Not** a second-screen surface — `SecondScreenPage.svelte` does not mount it (verified: 0 references); the second screen is a display, not a domain.
- **Not** a Living-Link editor — typed *links between notes* (the 8 cognitive link types) live in the Outgoing/Backlinks panels; a frontmatter `link` value here is a plain wikilink scalar.

## 4. Wiring
- **Inputs:** props from the host — `properties`, `body`, `tabId`, `filePath`, `libraryName`, `noteDir`, `collapsed`. Reads stores `appSettings` (pill shape, date format/script), `locale`, the per-library `propertyTypeRegistry` (`getRegisteredType`). Listens for the global `constellation:add-property` DOM event (Ctrl+;).
- **Outputs (writes/IPC):** debounced (800 ms) `saveTabContent(tabId, filePath, props, body)` → `buildFullContent` re-stitches YAML+body → the Editor's standard `write_note` + reindex path (no special IPC of its own). `setRegisteredType` persists a type choice library-wide. `onstagechange` callback bubbles a stage change to the host (→ `onpromote`). On destroy, flushes any pending save.
- **IPC it triggers (lazy only):** `sources_get_horizontal_taxonomy` / its vertical counterpart, via `getHorizontalTaxonomy`/`getVerticalTaxonomy`, fetched + cached **only** when the user expands a `sources:`/`content_type:` picker — never on mount.
- **Consumers:** the saved file (then everything downstream of the Editor's reindex: FTS5/tags/links). The host re-renders the panel from fresh `properties` on tab change.
- **Connection to the Editor (the gate):** mounted **inside** `NotePane.svelte` (line ~1122) and as the right-sidebar "Properties" tab in `+layout.svelte` (line ~6891). Both instances are the *same component* (one source of truth). It does not re-implement save/load — it routes through `saveTabContent`, the same store path the Editor uses, so the note's in-memory model stays the single authority.

## 5. Right-click / context menu
- **Has one? NO.** Grep for `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` in `PropertyEditor.svelte` returns **zero matches**.
- Row actions are exposed by **visible affordances only**: a drag handle (reorder), a type-icon dropdown, a per-row delete (`×`), key-suggestion and stage dropdowns, taxonomy pills with inline `×`. Nothing is reachable *only* by right-click — but nothing is reachable by right-click *at all*.
- **Gap flagged:** per the core paper §5 ("right-click should include every aspect of the app"), a property row arguably *should* offer a shared `<ContextMenu>` (e.g. Copy value, Change type, Convert to body, Delete, Move up/down) routed through `buildContextMenu()` (MIG-077). Today it has none. Bring-up decision: add a shared-menu contract or explicitly rule it out — do **not** hand-roll one.

## 6. Multilingual
- **Strong.** Every user-facing string flows through `$t('propertyEditor.*')`; the `propertyEditor` block is present in **all 15 locale files** (verified ar de en es fa fr he hi ja ko pt ru tr ur zh). The `ikhtilaf*` nested-object keys and `taxonomyTreePicker.expandAll` exist in en + ar.
- Type labels, key suggestions, and stage labels are localized; key suggestions carry a bilingual native equivalent (`labelAr`) and `selectKeySuggestion` picks the native key for RTL locales (ar/fa/ur/he) — e.g. inserts `الوسم` not `tags`. `SPECIAL_KEYS` recognizes Arabic keys (`الوسم`, `وسوم`, `أسماء بديلة`).
- **RTL:** `noteDir` prop drives date direction (`getDateDir`), the `↳` taxonomy connector mirrors under `[dir="rtl"]`, dropdowns flip (`left/right`), and date locale/numerals follow script.
- **Minor note (not a violation):** the `ikhtilaf*` and `expandAll` `$t()` calls carry English `|| 'School'`-style fallbacks. The keys *do* exist in en+ar, so the fallback never surfaces in practice; bring-up should still confirm those keys are backfilled across all 15 (not just en+ar) and drop the inline English fallbacks to avoid an English leak if a locale is missing the key. **No truly hardcoded-English user-facing string was found.**

## 7. Boot behavior
- **Runs at boot? NO.** The component only mounts when a note is open and the Properties surface is visible; nothing fires on app start.
- **Rule 8 status: ✅ compliant — reads-stored, does not recompute.** It reads the note's already-parsed `properties` prop (frontmatter the Editor extracted) and writes back through the save path; it recomputes no Universe-wide derived view. Taxonomy data is read from a cached IPC on demand, not rebuilt.
- **Cost:** negligible — render is O(number of properties on one note), typically <20 rows. Taxonomy fetch is one cached IPC on first picker-expand. No measured boot cost (it isn't on the boot path). *(estimated)*

## 8. Flag / gate & bring-up position
- **Gate today:** none of its own — it ships with the Editor's pane. No `enabledFeatures.*` / `SIGHT_*` flag guards it; the right-sidebar tab is gated only by `rightSidebarTab === 'properties'`. *(needs new gate if the bring-up wants it independently toggleable — verify in bring-up.)*
- **Bring-up phase:** **1 (with the Note Editor / core spine).** Depends on: the Editor's save path (`saveTabContent`/`buildFullContent`), the `propertyTypeRegistry` store, and `appSettings`/`locale`. The taxonomy pickers (`sources:`/`content_type:`) additionally depend on the CECE taxonomy IPC and may bring up **after** the bare panel.

## 9. Budget
- **Boot budget:** zero (not on the boot path).
- **Interaction budget:** typing in any value control must stay instant; saves are debounced (800 ms) — no `invoke()` on the keystroke path (verified: no direct `invoke()` in the component). Taxonomy expand is the only IPC and is user-initiated + cached.
- **Regression guard:** edit a property → confirm exactly one debounced `write_note` (not per-keystroke); tab-switch mid-edit flushes the pending save (onDestroy) with no body loss; the **content-integrity gate** (Editor-Surface checklist item 5: "PropertyEditor + stage promote") must stay green — on-screen frontmatter === on-disk after every transition.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** add/edit/delete/reorder a property persists to YAML; type changes round-trip; stage combobox + taxonomy pickers write correct lists.
- [ ] **Serves Constellation's core purpose:** advances Connection (tags/links) and Conviction (stage); frontmatter is real File-Over-App metadata, not app-only state.
- [ ] **Wires correctly to the Editor:** routes through `saveTabContent` (no re-implemented save/load); flush-on-destroy preserves body; never mutates body silently.
- [ ] **Right-click present + correct (shared, not hand-rolled):** currently **absent** — decide in bring-up: add a shared `<ContextMenu>` per MIG-077, or formally rule it out. No hand-rolled menu.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** all `propertyEditor.*` keys present in 15 locales; native keys for RTL; drop inline English fallbacks after confirming `ikhtilaf*`/`expandAll` backfill across all 15.
- [ ] **Within budget:** no per-keystroke IPC; debounced save; taxonomy IPC lazy + cached.
- [ ] **Obeys Rule 8:** reads stored frontmatter; recomputes no boot/read-time derived view.
- [ ] **Holds its invariants:** single content ownership preserved — the panel never becomes a competing authority over the note's body; on-screen === on-disk after edit, tab-switch, and stage-promote.
- [ ] **Boss-tested** per the Testing Instructions Rule (define the panel, then walk an add→edit→reorder→delete→stage cycle click by click).

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (not yet measured; not on boot path, expected trivial)**
Notes: One component, two mount sites (NotePane inline + right-sidebar tab), one source of truth — no copy-paste drift. **Two open items for bring-up:** (1) **no right-click menu at all** — the core paper's "right-clickable everywhere" contract is unmet here; add a shared `<ContextMenu>` or rule it out. (2) confirm `ikhtilaf*` / `taxonomyTreePicker.expandAll` keys are backfilled across all 15 locales (verified only en+ar) and remove the inline English `||` fallbacks. The taxonomy pickers (`sources:`/`content_type:`, MIG-021v2/MIG-022) and the `nested-object-list`/ikhtilāf widget are CECE-coupled — they may stage in after the bare typed-property panel.
