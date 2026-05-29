# HANDOVER — MIG-065 (Unified Progressive Base) §F.2/§G continuation

**Created:** 2026-05-29 (end of the Dual-World Base design + §A–§F build session).
**Hand-off to:** next session.
**Boss directive (verbatim):** *"Let's pick up §G fresh. Prepare the handover prompt!"*

> **Read order for the next session:**
> 1. `git pull origin main` (CLAUDE.md "Before Starting Work") — the MIG-065 §A–§F commits are pushed.
> 2. `docs/Constellation Orientation & Onboarding v2.44.md` — canonical state (it carries the MIG-065-in-flight note).
> 3. **This file** — the MIG-065 detail + what §G needs.
> 4. `docs/MIG-065-constellation-base-unified-ARCHITECT.md` + `…-PLAN.md` — the umbrella design + the step list.
> 5. `docs/Constellation-Base-Concept-Paper-v2.0.md` — the reconciled vision (§5.0 Strong yet Simple).

---

## 1. Function in hand

**Working on: Constellation Base — the Unified Progressive Base (Dual-World), MIG-065.** One Base that opens as a familiar Obsidian-style table and grows — *by adding a column, never by learning to code* — into the only database queryable by a link's confidence, a note's stratum, or its epistemic source. **Governing principle (Eisa, top of all Base decisions): "Strong yet Simple, by default."** The default view is uncluttered + familiar; strength is one gesture away and never crowds the first screen.

**Next concrete steps: §F.2 (standalone `.base`-file → full-tab table) + §G (the "+ Add column" picker).** They pair — §G's picker wants the full-tab table to host its toolbar.

## 2. Where we are — §A–§F shipped + Boss-validated

The *entire backend* + the familiar **inline** table are done, tested, committed, pushed. Commits (on `main`):

| § | Commit | Delivered |
|---|---|---|
| docs | `9daa5bb1` | Architect + Plan |
| A | `d8af1d5c` | Concept Paper **v2.0** — Dual-World reconciliation; §5.0 "Strong yet Simple"; §3 refusal reframed (refuse *empty aspirational structure*, not *familiar surfaces*); roadmap → MIG-065+ |
| B | `5197749e` | `properties_json` verified **faithful for scalar** frontmatter (6 characterization tests; RTL/empty/quotes/colons). Multi-line list/nested **dropped** — deferred (see §5). |
| C+D | `a89fc1a9` | Engine: `LensView::Table`; `prop.<key>` frontmatter columns; Text filters; federated symmetry |
| E | `3c411031` | `discover_base_properties` command + materializer fix (resolve_dim, not lookup_dimension) |
| F | `76de5ed7` | `LensResult.view`+`columns`; `LensBlockWidget._renderTable` — inline ` ```base view:table ` renders the familiar table |
| F-polish | `d3f9f3c3` | count badge (accent+white); RTL name cells (cell-level `dir`); header labels `lensBlock.col*` (en+ar) |

**97 lens tests pass.** Boss-tested §F: a 15-row table renders, RTL correct, click-to-open works, `prop.status` shows as a column.

## 3. Code reality (verified — don't re-map)

**Engine — `src-tauri/src/lens/`:**
- `definition.rs` — `LensDefinition { schema, lens, template, scope, where_clauses, order, columns, view }`. `LensView::{List, Table}`. `LensColumn/LensFilter/LensSort` each carry a `dimension: String`.
- `dimensions.rs` — registry of 4 dims (`note.name`/`note.path`/`note.created_at`/`note.headline`) + **`resolve_dim(name) -> Option<ResolvedDim>`**: registered dim OR `prop.<key>` → `json_extract(note_meta.properties_json, '$."<key>"')`. `PROP_PREFIX = "prop."`. Frontmatter columns are Text, sortable, filterable (`is/is_not/contains/does_not_contain/is_empty/is_not_empty`).
- `sql_builder.rs` — `build_sql` + `build_federated_sql` (UNION ALL across `main`+`cu0…`). `build_text_filter` for prop.*; `qualify_expr` schema-qualifies `note_meta`.
- `query.rs` — `execute_lens(app, lens_yaml) -> LensResult { rows, total_count, query_time_ms, lens_name, template, view, columns }`. `discover_base_properties(app) -> Vec<String>` (federated `json_each`; helper `discover_keys_federated`). Federated-then-single-schema fallback.
- Registered in `lib.rs invoke_handler`: `lens::query::execute_lens`, `lens::query::discover_base_properties`, `lens::system_notes::list_five_acts_notes`.

**Frontend:**
- `src/lib/lens/store.ts` — `LensResult`/`LensRow`/`DimensionValue` (untagged: `string|number|boolean|null`); now with `view`+`columns`. `executeLens(yaml)`, `discoverBaseProperties` not yet wrapped (add a wrapper for §G).
- `src/lib/editor/livePreview.ts` — `LensBlockWidget` (renders ` ```base ` blocks). `_renderResult` branches `res.view==='table'` → `_renderTable` (clickable name cell + declared columns, prop.* included; `_labelFor`, `_renderCellValue`, per-cell `dir`). CSS-in-JS `.cm-lens-table*` styles. `baseLensField` StateField detects the block.

**Schema (cheap columns available NOW):** `note_meta` (path/name/library_name/created_at/word_count/**properties_json**/**sources**/**content_type**); `note_links` (link_type/confidence/weight/traversal_count/…); `note_summaries.headline`. **Stratum/maturity/provenance are NOT persisted** (compute-on-demand) → MIG-068 must add write-time derivation first.

## 4. Locked decisions (Eisa)

1. YAML `.base`; old JSON `.base` silently ignored. 2. Both standalone files **and** inline blocks. 3. Curated picker + finite aggregations, **no formula language** v1. 4. **One engine** — extend `execute_lens`, retire `query_base`. 5. v1 = Simple foundation → then Living Links (MIG-066), Epistemics (MIG-067). 6. Concept Paper v2.0 first. **★ Strong yet Simple, by default.**
- **Implementation deviation (intentional, logged):** frontmatter columns use the `prop.<key>` prefix in the existing `dimension` field — NOT a separate `property:` YAML key as the Architect first wrote. Same user outcome, zero struct churn, aligns with the prefix-namespace research. The picker generates `prop.<key>`.

## 5. What's NEXT (the remaining Plan steps)

- **§F.2 — standalone `.base`-file → full-tab table.** Route opening a `.base` file to a full-screen table (reuse `BaseTableView.svelte` family OR generalize `_renderTable`), fed by `execute_lens`. This is the chrome §G's picker lives in. *Decision for the new session: reuse the old `BaseView/BaseTableView` Svelte components (they already do resizable/reorderable columns + inline edit) re-pointed to `execute_lens`, vs. a fresh component. Lean: reuse — "secure the winning" (CLAUDE.md).*
- **§G — "+ Add column" picker.** Tiered menu: **Your fields** (from `discover_base_properties`) + a **Constellation** power section (registered dims; today just the 4, grows MIG-066+). Selecting a field rewrites the `.base`/block `columns:` (append `- dimension: prop.<key>`). Mark computed/cognitive columns read-only + visually distinct (Airtable pattern). 15-locale labels.
- **§H — edit-in-place.** Cell edit on a `property:`/`prop.*` column → write the note's frontmatter via the OLD MVP's `update_note_property` (still registered in `bases.rs`); debounced single-writer + reload-after-write. `dimension:` (cognitive) columns are read-only.
- **§I — retire `query_base`.** Re-point `.base` routing + `list_workspace_bases` to the new engine; remove the live-scan path. Grep-confirm no Base read walks the filesystem.
- **§J** 3-agent audit · **§K** staged Boss test · **§L** PCS (orientation bump, 15-locale `lensBlock.col*` + the deferred items, push, milestone tag, ZIP).

## 6. Deferred / known limitations (PJ candidates)

- **Faithful list/nested `properties_json`** — the hand-rolled `search.rs::parse_frontmatter` drops multi-line YAML lists + flattens nested objects (scalars are fine). A proper fix (serde_yaml-with-fallback) needs a **re-index**; do it before advertising list/nested column types. Characterized by `tests_mig065_base_columns::known_limitation_multiline_list_is_dropped`.
- **`lensBlock.col*` in 13 locales** — only en+ar added; the rest land in §L (or the #13 batch).
- **`federation: off`** path — parser accepts it; `execute_lens` doesn't yet filter to single-universe (defaults to auto). Future.

## 7. Build / infra gotchas (will bite the new session)

- **LNK1104 file-lock** on the debug **test** binary (`constellation_lib-*.exe`) — Windows Defender holds the just-executed exe. **Workaround used:** run `cargo test` in a **background bash retry-with-backoff loop** (try → on LNK1104, `sleep 12`, retry ×6). Killing a background drive-scan and stale `constellation_lib`/`link` processes also helps.
- **`cargo check --lib --tests`** (no link step) surfaces *real* compile errors cleanly without the lock — use it to triage before running tests.
- **`npm run tauri build`** ends with `Error: A public key has been found, but no private key…` — that's the **benign** updater-signing step; the `.exe` + NSIS/MSI installers are already built. Detect success via `Built application at:` in the output, not exit code.
- Rebuilds may need to **replace a running `constellation.exe`** (LNK1104) — the build loop kills `constellation,constellation_lib` before retrying.

## 8. First actions for the new session

1. State the function in hand (§1 above).
2. `git pull`; skim orientation v2.44 + this file + the MIG-065 Architect/Plan.
3. Decide §F.2's component strategy (reuse `BaseTableView` vs new) — recommend reuse; confirm with Eisa if it turns into a real fork.
4. Build §F.2 (full-tab `.base` table) → then §G (picker) → §H (edit). Boss-test at each (Testing Instructions Rule; staged). Commit per §.
5. Honor Plan-Approval-Equals-Build-Approval — the Plan is approved; cascade, pausing only at the Boss-test clauses + genuine surprises.

---

*End of handover. MIG-065 §A–§F are done, tested, pushed. The new session resumes at §F.2/§G with full context — no re-discovery needed.*
