# PJ-068 v2 — The Knowledge Cockpit — Build Plan (Phase 2)

*Follows the Architect: [PJ-068-v2 Concept Paper](PJ-068-v2-Second-Screen-Knowledge-Cockpit-Concept-Paper.md). Boss ratified the concept + 5 rulings 2026-07-09 (zone layout accepted · ship all three dial positions · all seven surfaces · retirements confirmed (re-validate later) · design Sight's complement now, build later). Frontend-only — an **event-vocabulary + mode-state-machine redesign**, no schema. Each phase lands as one commit, is Boss-testable, and runs the standing per-build checks (`/simplify` + diff-scoped `safety-inspection`).*

## Territory verified this session (supersedes the concept's migration sketch)

- **Operation-Map data is ready (Rule-8 clean):** `get_backlink_rows` / `get_outgoing_rows` (`cache.rs:597/648`, MIG-079 per-note index-seeks — NOT a walk); each row carries `link_type`, `confidence`, `weight`, `traversal_count`, `last_traversed`, `status`. Unlinked mentions: `scan_unlinked_mentions` (`libraries.rs:2828`, FTS-backed). `BacklinksPanel`/`OutgoingLinksPanel` are presentational (props-only) → reusable read-only.
- **Control-Dashboard data is ready:** note_links columns (weight/confidence/type/traversal/status) + **derived** stage `getLinkStage` (`store.ts:2255`; code vocabulary = `spark→birth→growth→maturity→dormancy→archival`) + review-due `get_note_review_status` (`review.rs:522`) / `get_due_notes` (`review.rs:68`).
- **Estimation-Map data is ready:** `sky_nodes` / `sky_links` ARE persisted + write-time-maintained (`maintain_sky_after_save` `search.rs:1504`; read via `cache_boot_snapshot_sky` `cache.rs:849`). **The only missing piece is stable x/y positions** (re-simulated per render) — a small P4 sub-decision, default = re-simulate a cheap locator layout (keeps this migration schema-free).
- **Navigator is ALREADY retired** (MIG-091 — no `NotebookNavigator` in the SS). Retirements shrink to **OrgChart** (dead, `SecondScreenPage.svelte:1860-1875` + import :39) and the **fallback tab-strip editor** (`:1890-1915`). The `DashboardView`-follow arm (`:1877-1888`) is a judgment call folded into P4.
- **The mode if-chain** is `SecondScreenPage.svelte:1199-1924` (~13 branches).
- **Doc-drift to fix in close-out:** `CLAUDE.md` Rule-8 line still calls Sky View "rebuilt on every boot" (stale since MIG-001/002); the SS help topic + User Manual still document full editing on the SS (contradicts read-only-always).

## Shape

Replace the 13-branch mode if-chain with a **focus-channel + three-zone state machine**. One `screen:focus` event carries the MS's current focus; the SS re-fills its three fixed zones (Estimation Map / Control Dashboard / Operation Map) from it. Read-only always; every zone reads **persisted** derived data (no re-walk). Behind a `COCKPIT_ENABLED` flag; the old mode-chain stays until P5 → one-line revert per phase.

---

## P0 — Reconcile the in-flight G3 diff to "read-only always" *(the foundation commit)*
- **KEEP:** the `readOnly` prop threading (NoteEditor/NotePane/PropertyEditor) — it IS read-only-always at the component layer; `adoptFreshDiskIntoSS` (`SecondScreenPage.svelte:719-750`) + `adoptCompanionTab` (`:756-760`) + the `onNoteSaved` adopt (`:866`) + the `cascade:rewrote` listener (`:875-880`) — the freshness sync a read-only complement still needs.
- **REMOVE (editable toggle):** `AppSettings.secondScreenEditable` (`store.ts:4174`) + `DEFAULT_SETTINGS` (`:4504`); the SettingsModal toggle (`:1029-1034`); the i18n keys `settings.editor.secondScreenEditable`(+`Desc`) in all 15 locales; simplify `ssReadOnly` (`SecondScreenPage.svelte:185`) → a constant `true` on the mounts.
- **REMOVE (§4 freeze — obsolete under read-only):** `secondScreen.ts` block `~382-402` (`CascadeFreezeData`/`emitCascadeFreeze`/`onCascadeFreeze`); `+layout.svelte` emits `:6213`/`:6266` + the import; `SecondScreenPage.svelte` import (`:52`) + helpers (`:762-807`) + the `uFreeze` listener (`:882-888`) + the cleanup call; drop the now-unused `markCascading`/`clearCascading`/`tabsInLibrary` SS imports.
- **Verify:** SS note-views are read-only; a main-window edit-and-save (or rename cascade) refreshes the SS view within ~1s; `svelte-check` 0 errors; `tests/mig-076/runtimeHarness.test.ts` — Recipe N's adopt-when-clean / cascade-reload assertions stay green; drop the editable-mode "dirty-refuses" expectation only if it becomes vacuous (a read-only model is always clean → always adopts). Diff-scoped `safety-inspection` (the two-sided residual is gone).

## P1 — Focus channel + three-zone shell + the Normal/Live/Locked dial
- **`screen:focus` event:** `{ kind: 'note'|'sky-node'|'map-arc'|'index-term'|'task'|'sight-region'|'none', path?|id?|term?, mode: 'selection'|'hover' }`. Emitted from `+layout.svelte` via ONE coalesced, ≥300 ms-debounced `$effect` (never on the keystroke path). The legacy per-mode emits stay in parallel during migration and are removed as each zone lands.
- **The SS shell:** the three fixed zones + the **dial** (Normal / Live / Locked; visible + releasable; persisted per library). Normal = reflect the committed focus; Live = reflect `mode:'hover'` focuses without disturbing; Locked = pin the last focus, ignore new ones until released.
- **Verify (Boss):** open the SS → the three fixed zones render; the dial switches; Locked pins a note while the MS navigates elsewhere.

## P2 — Note-editor complement (Operation Map + Control Dashboard) + retire the fallback tab editor
- **Operation Map (note):** reuse the store helpers over `get_backlink_rows`/`get_outgoing_rows` + `scan_unlinked_mentions`; render the typed-link inventory with `LinkTypePill` + lifecycle chips (reuse `BacklinksPanel`/`OutgoingLinksPanel` read-only). **Control Dashboard (note):** link health (stage via `getLinkStage` + weight/confidence/traversal from the rows) + `get_note_review_status`.
- **Retire the fallback tab-strip editor** (`:1890-1915`) + the SS-local `openTabs`/`activeTabId` lifecycle + the workspace save/restore around `openTabs` (`:937-964`).
- **Click-to-navigate:** click a backlink / typed relation → `sendNoteToMain` (MS opens it; SS never edits).
- **Verify (Boss):** open a note → SS shows its Operation Map + Control Dashboard (backlinks, typed links, health, review-due); click a backlink → the MS navigates; **Editor-Surface Gate item-7 (read-only)**: SS on-screen === disk after MS edits.

## P3 — Sky View + Constellation Map complements + retire OrgChart
- **Sky View focus:** Live (hover a bubble) → SS shows the node's detail (text preview + its link data-block) without moving the graph; Normal (select) → full detail. **Map focus:** Live (hover an arc) → SS lists the notes under it + the you-are-here breadcrumb, zero effect on the sunburst.
- **Retire OrgChart mode** (`:1860-1875` + import `:39`).
- **Verify (Boss):** hover a Sky bubble → SS peeks its card (graph unmoved); hover a Map arc → SS lists its notes; click a listed item → MS navigates/re-centers.

## P4 — Estimation Map + Index + Dashboard/Sight + Tasks + Sight-lens
- **Estimation Map — the ONE holistic universe view across past/present/future** (Boss ruling 2026-07-09; concept §3.1). Reuse persisted `sky_nodes`/`sky_links` (`cache_boot_snapshot_sky`) for the whole field, layered across time: **past** (creation timeline via `cid_cn` + link maturation history), **present** (active/load-bearing links, live clusters), **future** (`get_due_notes` review-due, decaying/dormant links, contested/tension links, orphans). The focus marker locates *within* it. **This is the marquee zone and gets a DEDICATED design pass BEFORE this phase** (research temporal/holistic knowledge visualizations + enrich concept §3.1: how to render past/present/future in one coherent map — time spine, temporal encoding, layered overlays). Data stays schema-free; a persisted `sky_layout` positions table remains a deferred option only if the rendering needs pixel-stable layout.
- **Index:** the term's mention-expansion + `via {lemma}` / `≈ similar` scope (reuse `read_term_mentions`). **Dashboard/Sight overview (idle state):** the universe Estimation Map + a **management-by-exception** Control-Dashboard strip (review-due via `get_due_notes`, decaying/dormant links, unresolved tensions/contested links, orphans). **Tasks:** the selected task's referenced notes/links + timeline. **Sight-lens:** the selected region/stratum's notes + taxonomy provenance, expressing ONLY that tradition's grammar (Form-Aligns-To-Purpose; no lens recompute). *(Sight is a disabled Wings plug-in — its complement is designed + wired behind the Sight flag, activating when Sight is on; Boss ruling 5.)*
- **Verify (Boss):** each surface's complement fills the zones + click-to-navigate works; the idle Dashboard state shows the exception triage.

## P5 — Unify + harden + close-out
- Remove the now-dead legacy `screen:*` per-mode emits/listeners superseded by `screen:focus`; dead imports; the mode if-chain fully replaced by the state machine; the `DashboardView`-follow arm folded into the Estimation-Map idle state.
- **Rule-8 / perf gate on a 7,600-note Universe:** boot + typing latency unchanged; zero `invoke()` on the keystroke path to feed the SS; focus pushes batched/debounced; no zone re-render stutters MS typing.
- **Editor-Surface Gate item-7 (read-only) full pass** + the two-window Boss test.
- **Docs:** SS help topic + User Manual ×15 rewritten to the cockpit (drop "full editing on SS"); CP-26 + PJ-068 reconciled; orientation v-bump; fix the `CLAUDE.md` Rule-8 Sky View line.
- **Per-cycle whole-app `safety-inspection`** + `/simplify`; PCS.

---

## Rollback
Frontend-only; `COCKPIT_ENABLED` flag; the old mode-chain retained until P5; one-line revert per phase; **no schema** (default Estimation-Map path re-simulates positions).

## Boss-test stops (staged tutorials per the Testing Instructions Rule)
P0 (read-only + fresh) · P1 (shell + dial) · P2 (note complement + click-to-navigate) · P3 (Sky + Map) · P4 (Index + Dashboard/Tasks/Sight) · P5 (full two-window pass). One stage sent at a time; each on a fresh release binary (built before the tutorial).

## Residual / not-in-scope
- A persisted `sky_layout` position view (only if the Estimation-Map locator needs pixel-exact parity with the main Sky View) — deferred sub-decision.
- The Rust-side derived-index staleness on link *re-type* (safety register MED, `search.rs:1447/1489`) — separate; not a cockpit concern.
