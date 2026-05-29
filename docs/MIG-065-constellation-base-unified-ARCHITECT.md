---
title: MIG-065 — Constellation Base, The Unified Progressive Base (Dual-World) — Architect
version: 1.0
date: 2026-05-29
status: Architect. Direction + 6 decisions locked by Eisa 2026-05-29 ("proceed as you see fit"). Governing principle: "Strong yet Simple, by default." Plan doc accompanies.
direction_holder: Eisa
drafter: Claude (Opus 4.8)
predecessor_vision: docs/Constellation-Base-Concept-Paper-v1.4.md (to be reconciled into v2.0 — see §9)
predecessor_build: MIG-055 (the lens engine — extended here, not replaced), MIG-056 (federation), MIG-060 (threading gestures)
research_grounding: two sourced web-research passes 2026-05-29 — Obsidian (Bases/Dataview/Datacore) + commercial leaders (Notion/Airtable/Coda). Findings summarized §3.
---

# MIG-065 — The Unified Progressive Base (Dual-World)

## §0. The one-line goal

**One Base feature that opens as a table any Obsidian/Notion user instantly recognizes, and grows — by adding a column, never by learning to code — into the only database that can be filtered by a link's confidence, a note's intellectual altitude, or where its knowledge came from.**

Eisa's governing constraint, recorded as the **top design principle for all Base work**:

> **Strong yet Simple, by default.** The default view is uncluttered and familiar. Strength is always present and one gesture away — but it never crowds the first screen. When any decision is in tension, the *default* leans Simple; the *ceiling* stays Strong.

## §1. Why this MIG exists — the two-Bases problem

Constellation today ships **two** disconnected Base features (verified by code read, commit `731e06cf`):

| | OLD MVP (`bases.rs`) | NEW lens engine (`lens/`) |
|---|---|---|
| Concept | Generic Obsidian clone — auto-detect frontmatter keys → columns | Curated cognitive "knowledge lens" |
| Sidebar | "Workspace Bases" | "Five Acts" |
| Format | `.base` JSON | inline ` ```base ` YAML |
| Engine | `query_base` — **live filesystem walk + frontmatter parse** (Rule-8 violation) | `execute_lens` — **SQL on `note_meta`** (Rule-8 clean), federated (MIG-056) |
| UI | `BaseView` + Table/Card/List + Filter/Sort builders (6 components) | `LensBlockWidget` in `livePreview.ts` (list view only) |
| Status | Active, federated this session (MIG-062 §D) | Shipped through Phase 1.5 |

This split is an accident of history (the clean-slate MIG-055 added the new engine *beside* the old MVP rather than replacing it). Eisa's decision 2026-05-29: **unify them into one progressive feature** — the familiar MVP becomes the *Simple* face; the lens cognitive dimensions become the *Strong* columns. One schema, one engine, one renderer.

## §2. What this Architect covers (umbrella) vs what MIG-065 builds (first slice)

This Architect is the **durable umbrella** for the whole Dual-World Base. It is implemented across several MIGs:

- **MIG-065 (this Plan): the Simple foundation.** The unified YAML schema + SQL engine + the familiar **table** view (arbitrary frontmatter columns, filter, sort, edit-in-place, standalone `.base` files), the tiered **add-column picker**, and retirement of the old live-scan engine. End state: an Obsidian user opens a `.base`, sees a familiar editable table — running on the clean, federated, Rule-8 engine.
- **MIG-066: the first Strong family — Living Links.** `link.confidence` / `link.weight` / `link.type` / `link.traversal_count` columns + filters (cheap — already `note_links` columns). The picker's "Constellation" power tier lights up.
- **MIG-067: Epistemic family — Source × Content-type.** `note.source` / `note.content_type` columns (cheap — already `note_meta` columns from MIG-021).
- **MIG-068+: Cognitive Engine family (Stratum / Maturity / Provenance).** **Requires a write-time-derivation step first** (these are not persisted today — §6 risk R3). This MIG includes that derivation, then the columns.
- **Later: aggregations (count/sum/avg/min/max over linked notes), card/board/calendar views, the remaining four Five Acts templates.**

The phasing literally enacts Strong-yet-Simple: **ship Simple first (065), then layer Strong one cheap family at a time (066, 067), deferring the families that need new derivation until their groundwork lands.**

## §3. Research grounding (what proven tools taught us)

Two sourced research passes (Obsidian; Notion/Airtable/Coda). The load-bearing conclusions:

1. **Progressive disclosure inside ONE object wins** — not a mode switch, not a separate query language. Airtable keeps one "+add column" door; Notion tiers types **Basics → Organizers → Power Tools**. Adopt both.
2. **Obsidian's mistake is the cliff we invert.** Their "powerful" path is a *different tool + a code language* (Bases → Dataview → Datacore/React); users fall off a cliff and run two engines forever. **Our power = "add a column," never "learn to code."** This is the whole thesis and the literal meaning of "Strong yet Simple."
3. **Adopt Obsidian's file shape.** A `.base` is YAML with `filters / formulas / properties / views`, columns living as an ordered list *inside each view*, mixing plain and computed columns via a **prefix namespace** (`note.` / `file.` / `formula.`). Our cognitive dimensions slot in as just more prefixed columns (`link.` / `note.cns.` / `note.cece.`).
4. **Mark computed columns read-only + visually distinct** (Airtable). Our cognitive columns are exactly this class.
5. **Our architecture turns the competitors' #1 weakness into our advantage.** Notion's *own docs* name read-time formula/rollup cascades as what stalls databases past ~2–3k rows. **We precompute in SQLite (Rule 8)** → rich cognitive columns at lookup speed where they choke. And relations are already ours (typed links exist) — no manual wiring.
6. **Opinionated defaults beat a blank canvas** (the Notion/Coda paralysis critique). The Base opens with the *right* columns already chosen (the note's own fields).

## §4. The unified design

### §4.1 One schema (extend `LensDefinition`, keep it YAML)

The `.base` is YAML (decision #1). Extend the shipped `LensDefinition` (`lens/definition.rs`) — **additive, not a rewrite**:

- `view: table | list | card` (today: `List` only → add `Table` in MIG-065; card later).
- A column may name **either**:
  - a **plain frontmatter field** — `property: status` → resolved by `json_extract(note_meta.properties_json, '$.status')`. **Editable.** This is the familiar/Simple half.
  - a **registered dimension** — `dimension: link.confidence` → fixed `sql_expression` from the registry. **Read-only/computed.** This is the Strong half.
- `filters` / `order` likewise accept `property:` (frontmatter) or `dimension:` (registered).

The clean **logic-vs-presentation split** (Obsidian's `properties` block) is preserved: a `display:` layer carries `displayName`, width, and (for us) `dir=auto` per column.

### §4.2 One engine (extend `execute_lens`, retire `query_base`)

- `sql_builder.rs` gains **dynamic property columns** (`json_extract(properties_json, '$.<key>')`) alongside the static registered dimensions. Table view selects all declared columns; federation (`build_federated_sql`) and the federated-then-fallback path are untouched.
- **`query_base`'s live filesystem walk is retired.** All Base reads become SQL on the write-time-derived index. This is the Rule-8 fix the 2026-05-25 handover flagged as the headline concern.
- **Edit-in-place** reuses the OLD MVP's `update_note_property` (writes a frontmatter key on disk) — but only for `property:` columns. `dimension:` columns are read-only (Airtable pattern).

### §4.3 One renderer, two entry points

- **Inline ` ```base ` blocks** already render via `LensBlockWidget` (`livePreview.ts`) — the host-note assemblage. Extended to render `view: table`.
- **Standalone `.base` files** open as a full-tab table (the Obsidian-familiar entry) — tab routing recognizes `.base` and mounts the same table renderer.
- The familiar table UI **reuses the OLD `BaseTableView.svelte` family** (it already does resizable/reorderable columns, inline edit, RTL) — re-pointed from `query_base` rows to `execute_lens` rows. *Reuse, don't rebuild* (CLAUDE.md "secure the winning").

### §4.4 The add-column picker — where Strong-yet-Simple lives

`+ Add column` opens ONE tiered menu:

- **Your fields (Basics)** — the frontmatter keys actually present across the Base's scope (discovered cheaply via `json_each` over `properties_json`) + file intrinsics (name, created, word count). *This is the default vocabulary; it's all a casual user ever needs.*
- **Constellation (Power)** — the registered cognitive dimensions, grouped: **Links** (confidence/weight/type/traversal — MIG-066), **Epistemics** (source/content-type — MIG-067), **Cognitive** (stratum/maturity/provenance — MIG-068). Present, discoverable, clearly marked read-only — but **below** the fold, never crowding row one.

## §5. Invariants (must not break)

1. **Strong yet Simple by default** — opening any Base shows a clean, familiar table of the note's own fields. No cognitive column appears unless the user adds it.
2. **File-Over-App** — `.base` stays plain YAML on disk; the SQL index is an optimization, never the source of truth.
3. **Rule 8 — no live scan, ever.** Every Base read is a cheap SQL lookup on the write-time index. No `query_base`-style filesystem walk survives this MIG.
4. **Edit-in-place writes ONLY frontmatter** (`property:` columns), via `update_note_property`. A cognitive (`dimension:`) column is never editable — it's derived data.
5. **Federation parity** — the unified engine keeps MIG-056's federated-then-fallback; a Base spans cUniverses by default (decision #2 / Concept Paper §10.6).
6. **Language-first** — every column header, operator, and picker label renders in all 15 locales, bidirectional, `dir=auto` per cell. Day one.
7. **Performance on 7,600+ notes** — table open and every filter/sort returns well under the §10.3 budget (≤ ~50 ms server-side). Measured before/after.
8. **Backward-tolerant** — old `.base` *JSON* files are silently ignored (decision #1 / MIG-055 §11 Q4); no data loss, no silent conversion.

## §6. Risks

- **R1 — `properties_json` reliability (BLOCKER for the Simple half).** The familiar table reads arbitrary frontmatter from `note_meta.properties_json`, but the current lens engine *deliberately avoids* that column over a flagged `search.rs::parse_frontmatter` bug (MIG-055 §9 R4). **MIG-065 §B must verify and, if needed, repair the `properties_json` write path before the table can be trusted.** Mitigation: an audit step with a row-by-row check against on-disk frontmatter (per CLAUDE.md `feedback_walk_through_writes`).
- **R2 — Dynamic columns + federation.** `json_extract` columns must work identically across the federated `UNION ALL`. Mitigation: extend `build_federated_sql` symmetrically; test on the Eisa Universe + cUniverse.
- **R3 — Cognitive dimensions not persisted.** Stratum/maturity/provenance are compute-on-demand (no WTD trigger). They are explicitly **deferred to MIG-068**, which carries its own derivation+persistence step. MIG-065/066/067 only expose dimensions that are *already columns* (frontmatter, links, source/content-type, headline, word count). No read-time computation enters the Base path.
- **R4 — Old-MVP retirement surface area.** `query_base` + 9 sibling commands + `BaseView` routing are wired in `lib.rs` and the sidebar. Mitigation: §I retires them behind the new engine in one reviewed step; the federation work from MIG-062 §D (`list_workspace_bases`) is re-pointed, not orphaned.
- **R5 — Edit-in-place write races.** Concurrent cell edits → frontmatter writes. The OLD `update_note_property` had no locking (handover §6 Q2). Mitigation: debounced single-writer per note + reload-after-write; full concurrency hardening tracked as a follow-up if it surfaces.

## §7. Decisions locked (Eisa, 2026-05-29 — "proceed as you see fit")

| # | Decision | Lock |
|---|---|---|
| 1 | File format | **YAML `.base`** (matches Obsidian + our lens). Old JSON `.base` silently ignored. |
| 2 | Where a Base lives | **Both** — standalone `.base` file (full-tab table) **and** inline ` ```base ` blocks. One engine/renderer. |
| 3 | Power surface | **Curated dimension picker + finite aggregation menu.** No user formula language in v1. |
| 4 | Engine | **One engine** — extend `execute_lens` (SQL, write-time); retire `query_base` live-scan. Reuse old table UI. |
| 5 | v1 scope | **MIG-065 = Simple foundation** (familiar table); Living Links (066) and Epistemics (067) follow immediately. |
| 6 | Guiding light | **Concept Paper v2.0** reconciles "both worlds, one progressive surface" — written as Plan §A, before any code. |
| ★ | Governing principle | **Strong yet Simple, by default.** Tie-breaker for every downstream choice. |

## §8. Out of scope (this MIG line, v1)

- User-authored formula language (decision #3) — far-future opt-in.
- Cognitive Engine columns requiring new persistence (stratum/maturity) — MIG-068, with its own derivation.
- CNS network columns (community/centrality/bridges) — needs the freshness strategy from the Concept Paper §6.11; later.
- Card / board / calendar / gallery views — table + list first; shapes earned by questions (Form-Aligns-To-Purpose).
- NL → Base, generative suggestions, Wings column types — Concept Paper §11.

## §9. The Concept Paper reconciliation (decision #6)

Concept Paper **v1.4 §3 explicitly *refuses*** the generic/familiar Base (the "structure-invitation effect"). Eisa's Dual-World direction reconciles this: the familiar world is welcomed as the **on-ramp**, and the refusal narrows to its true target — *productivity theater / aspirational empty schemas*, not *a familiar editable table*. **Concept Paper v2.0** (Plan §A) will:
- Add the **Strong-yet-Simple-by-default** principle as §5.0 (above Form-Aligns-To-Purpose, since it governs the Base specifically).
- Reframe §3's refusal: we refuse *empty aspirational structure*, not *familiar surfaces*. The familiar table is the doorway; the cognitive columns are why they stay.
- Fold this Architect's unified design into the roadmap (§12), superseding the v1.4 phase numbering with the MIG-065+ sequence.
- Preserve v1.4 as historical record (per SO #6 — new file, never overwrite).

## §10. Predecessor & adjacent

- **Vision:** `docs/Constellation-Base-Concept-Paper-v1.4.md` → reconciled to v2.0 (Plan §A).
- **Engine extended:** `src-tauri/src/lens/` (definition / dimensions / parser / validator / sql_builder / query / system_notes).
- **UI reused:** `src/lib/components/Base{Table,Card,List}View.svelte`, `Base{Filter,Sort}Builder.svelte`; inline renderer `src/lib/editor/livePreview.ts` (`LensBlockWidget`, `baseLensField`).
- **Retired:** `src-tauri/src/bases.rs::query_base` (+ legacy command family as §I scopes).
- **Schema:** `note_meta` (properties_json, sources, content_type, word_count, created_at), `note_links` (confidence/weight/link_type/traversal_count), `note_summaries.headline` — all in `src-tauri/src/search.rs` / `nsc/` / `sources/`.
- **Successor:** the MIG-065 Plan doc (this commit set).

---

*End Architect. The umbrella is mapped; the Plan decomposes MIG-065 into landable, verifiable commits. No code until Eisa approves the Plan.*
