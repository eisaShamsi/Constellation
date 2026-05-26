---
title: MIG-055 — Constellation Base, Clean Slate (Architect)
version: 1.0
date: 2026-05-26
status: Architect doc. Awaiting Eisa's approval before Plan doc.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
predecessor: docs/Constellation-Base-Concept-Paper-v1.4.md (the design north star; unchanged)
supersedes: MIG-054 in spirit — MIG-054's §A–§I.0 code was reverted at commit 15c41504. MIG-054's docs (Architect / Plan / Audit) are kept as historical record but their build trajectory is abandoned.
explicit_break_from_old:
  - No reference to BaseView.svelte / BaseTableView / BaseCardView / BaseListView
  - No reference to the old BaseDefinition shape (source/columns/filters/sorts/view)
  - No reference to auto-detected YAML keys as columns
  - No "behavioral equivalence with the old MVP" obligation
---

# MIG-055 — Constellation Base, Clean Slate (Architect)

## §1. Premise — the fresh start

Eisa's correction on 2026-05-26 named the category error in MIG-054: the §A–§I.0 work tried to fix the previous Bases (the MVP from commit `c5b05f5c`, 2026-03-12), which was built on a different concept — a generic Obsidian-Bases-equivalent that auto-detects YAML frontmatter keys and renders them as table columns. The Concept Paper v1.4 articulates a structurally different shape: **the Constellation Base is a knowledge lens parameterized by curated cognitive dimensions** (Living Links / CE / CNS / CECE / Five Acts), **not** by free-form YAML.

The v1.4 §3 refused effect is the *structure-invitation effect* — exactly the UX the old auto-detect-all-keys default delivered. Patching the old code to look like the new concept perpetuates the wrong shape. Eisa stopped the cascade and directed a clean slate.

This Architect doc defines the **first concrete deliverable** of the fresh-start trajectory toward v1.4. It does not redesign the concept (that's v1.4's job); it specifies how the first thin slice gets built.

## §2. What this Architect designs

1. **The lens-definition schema** — a YAML format that names *cognitive dimensions* as first-class identifiers, not free-form keys.
2. **The query mechanism** — a new Tauri command surface that takes a lens definition and returns rows with the dimensions the lens names. SQL-backed against `note_meta` for write-time-derived dimensions per Concept Paper v1.4 §10.1.
3. **The user-visible surface** — host-note assemblage per v1.4 §7. The first deliverable is one note (a system-shipped Five Acts template) that embeds one ` ```base ` block and renders it.
4. **The first complete vertical slice** — *Recent Captures* (per v1.4 §8). Last 14 days, sorted by creation date, with NSC headlines visible. Exercises three of the six differentiators (§3 SQL query / §2 NSC headlines / §3 federation auto), and proves the wiring end-to-end.

## §3. What this Architect explicitly does NOT design

- The `BaseView.svelte` UI component or its `BaseTableView` / `BaseCardView` / `BaseListView` children — these embodied the old concept; not reused.
- The `BaseDefinition` Rust struct (`source/columns/filters/sorts/view`) — replaced by a new shape.
- The `parse_base_file` / `query_base` / `update_note_property` / `save_base_file` / `*workspace_base` Tauri command family — replaced by a new family.
- Generic filter operators on user-defined YAML keys (`is` / `contains` / `gt` / etc. operating on arbitrary string properties) — the new lens names CURATED dimensions, not free-form keys.
- Auto-detected `columns_detected` — Bases never auto-detects YAML keys as columns.
- Backward compatibility with old `.base` files — the old format is abandoned; existing `.base` files on disk get ignored by the new code.
- Phase 2 onwards (Living Link columns / CE / CNS / CECE Bridges / cell-edit on typed links) — those are subsequent MIG numbers per v1.4 §12 (with the phase numbers reassigned in this Architect's §7).

## §4. The lens-definition schema (the new `.base` format)

The new `.base` is a YAML block — either standalone in a `.base` file OR inline as a ` ```base ` code block inside a host note (v1.4 §7 assemblage). The schema names cognitive dimensions, not free-form properties.

### §4.1 Schema v1 (Phase 1 scope only)

```yaml
# A Constellation Base — knowledge lens definition (schema v1)
schema: 1
lens: "Recent Captures"      # Display name (or template ID for system-shipped lenses)
template: five-acts.observation   # OPTIONAL — names a system template; presence implies "this is a known shape, not a user composition"

# Scope: which notes the lens looks at
scope:
  libraries: all              # "all" (default) | list of library names
  federation: auto            # "auto" (default, per v1.4 §10.6) | "off"

# Filter: which notes the lens INCLUDES
# Each entry names a CURATED dimension; never a free-form YAML key.
# v1 supports a small filter vocabulary; future phases extend it.
where:
  - dimension: note.created_at
    op: after
    value: "now - 14 days"

# Sort: how rows are ordered
order:
  - dimension: note.created_at
    direction: desc

# Columns: which dimensions render as visible columns
# v1: name + NSC headline. Future phases add Living Link / CE / CNS / CECE columns.
columns:
  - dimension: note.name
  - dimension: note.headline   # NSC summary, unconditional per v1.4 §6.2

# Rendering shape
view: list                    # v1 ships "list" only. Future: table / card.
```

### §4.2 Why this shape (vs the old `BaseDefinition`)

- **Dimensions are named identifiers, not property strings.** `note.created_at`, `note.headline`, `link.confidence`, `note.stratum`, `note.cns.community_id`, `note.cece.source.primary` — each is a known dimension with a defined source, type, and renderer. The schema validates against the dimension registry; unknown dimensions are an error at parse time.
- **Filter operators are dimension-specific.** `note.created_at` accepts `after` / `before` / `between` / `within`. `link.confidence` accepts `>=` over the ordered enum (hypothesis/evidence/established/contested). `note.is_orphan` accepts only equality. The old generic `is`/`contains`/`gt`/`lt` set doesn't make sense across all dimension types.
- **`template` field marks system shapes.** When `template: five-acts.observation` is present, the lens IS a Five Acts template. Constellation can render template-specific affordances (e.g., the "Mark as processed" action on Observation captures). User-composed lenses leave `template` absent.
- **`scope.federation: auto` is the default** per v1.4 §10.6 / §13 #5. Future phases add `scope.federation: off` for per-lens opt-out.

### §4.3 What's NOT in schema v1

- Living Link dimensions (`link.confidence`, `link.weight`, etc.) — Phase 2 per v1.4 §12.
- CE dimensions (`note.stratum`, `note.maturity`, etc.) — Phase 2.5 per v1.4 §12.
- CNS dimensions (`note.cns.community_id`, etc.) — Phase 2.6 per v1.4 §12.
- CECE dimensions (`note.cece.source.primary`, etc.) — Phase 2.7 per v1.4 §12.
- The full Five Acts template set (Connection / Tension / Synthesis / Conviction) — Phase 5 per v1.4 §12. Only `five-acts.observation` (Recent Captures) ships in v1.
- View shapes beyond `list` (table / card / future shapes) — earned by future questions per v1.4 §5.1 Form-Aligns-To-Purpose.
- Inline aggregations / formulas — Phase 6+ per v1.4 §11.

## §5. The query mechanism

A single Tauri command replaces the old `query_base` family:

```rust
#[tauri::command]
pub fn execute_lens(
    app: tauri::AppHandle,
    lens_yaml: String,          // the YAML text from the .base block
    host_context: Option<HostContext>,  // currently-open note's metadata, for relative references
) -> Result<LensResult, String>;
```

Where:

```rust
pub struct LensResult {
    pub rows: Vec<LensRow>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub lens_name: String,
    pub template: Option<String>,
}

pub struct LensRow {
    pub note_path: String,
    pub library_name: String,
    pub library_path: String,
    pub dimensions: HashMap<String, DimensionValue>,  // keyed by dimension name
}

pub enum DimensionValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Timestamp(i64),
    List(Vec<String>),
    Null,
}
```

The `dimensions` map is keyed by **the dimension names declared in `columns:`**. The renderer doesn't need to know which dimensions exist universe-wide — only which ones the lens names.

### §5.1 Backend implementation (Phase 1 scope)

The Phase 1 query exercises:
- `scope.libraries: all` + `scope.federation: auto` — selects all libraries across the active universe + any federated cUniverse children (per `resolve_libraries_recursive` in `universe.rs`).
- `where: [{ dimension: note.created_at, op: after, value: "now - 14 days" }]` — translates to `WHERE note_meta.created_at >= (unixepoch() - 14 * 86400)`.
- `order: [{ dimension: note.created_at, direction: desc }]` — `ORDER BY note_meta.created_at DESC`.
- `columns: [name, headline]` — fetches `note_meta.name` + the NSC summary via the existing `nsc_get_summaries_for_notes` IPC OR a direct join against `note_summaries.headline`.

The query DOES NOT touch `note_meta.properties_json`. That column may be corrupted (the pre-existing search.rs::parse_frontmatter bug); v1's lens definition never reads from it, so the corruption is irrelevant to v1.

### §5.2 Dimension registry

A new module `src-tauri/src/lens/dimensions.rs` defines the dimension registry. Each entry:

```rust
pub struct DimensionDef {
    pub name: &'static str,           // e.g., "note.created_at"
    pub kind: DimensionKind,          // Text / Number / Timestamp / Enum / Bool / List
    pub sql_expression: &'static str, // e.g., "note_meta.created_at"
    pub sortable: bool,
    pub filterable: bool,
    pub filter_ops: &'static [&'static str], // e.g., &["after", "before", "between"]
}
```

v1 registers FOUR dimensions:
- `note.name` — Text, sortable, not filterable in v1
- `note.headline` — Text from `note_summaries.headline`, not sortable, not filterable in v1
- `note.created_at` — Timestamp, sortable, filterable (after / before / between / within)
- `note.path` — Text, not sortable, not filterable (always-present implicit column)

Future phases extend the registry with Living Link / CE / CNS / CECE dimensions.

### §5.3 YAML parser

`serde_yaml` (already in Cargo.toml — used elsewhere) parses the YAML block into a `LensDefinition` struct. Schema validation catches:
- Unknown dimension names (referenced in `where` / `order` / `columns`)
- Filter operators not supported by the named dimension
- Unsupported `view` shapes

## §6. The first deliverable — Recent Captures

A system-shipped host note `Five Acts/Observation - Recent Captures.md` (or similar — exact path locked at Plan time). The note's body contains the prose explanation of the Observation Act + an inline ` ```base ` block:

```yaml
schema: 1
lens: "Recent Captures"
template: five-acts.observation
scope:
  libraries: all
  federation: auto
where:
  - dimension: note.created_at
    op: after
    value: "now - 14 days"
order:
  - dimension: note.created_at
    direction: desc
columns:
  - dimension: note.name
  - dimension: note.headline
view: list
```

When the user opens this note, the markdown renderer:
1. Reads the body.
2. Identifies the ` ```base ` block.
3. Calls `execute_lens(yaml_text, host_context)`.
4. Renders the returned `LensResult.rows` as a list — each row showing `note.name` + `note.headline`.
5. Each row's name is clickable; clicking opens the cited note in a new tab.

**The user's takeaway:** they see what they've captured in the last 14 days, with one-line summaries. That's the Observation Act made operable.

## §7. Step outline — the new build cascade

Indicative; the Plan doc breaks each into landable commits with verification clauses.

- **Phase 1 (this MIG) — the thin slice.** Schema + dimension registry + `execute_lens` Tauri command + host-note embedding renderer + Recent Captures template + the system note. End-to-end vertical.
- **Phase 2** — Living Link dimensions added to the registry. Lens schema extended to express link.confidence / link.weight / link.lifecycle filters. UI rendering supports the new column types.
- **Phase 2.5** — Cognitive Engine dimensions (Stratum / Maturity / Stage / Provenance / structural flags / review pulse / word count). Per v1.4 §6.10.
- **Phase 2.6** — CNS dimensions (community / centrality / top-bridge / blind-spots). Per v1.4 §6.11. Freshness-strategy decision locked at this phase per v1.4 §13 row 9.
- **Phase 2.7** — CECE dimensions (Source axis × Content-type axis). Per v1.4 §6.12.
- **Phase 3** — The remaining four Five Acts templates (Connection / Tension / Synthesis / Conviction). Per v1.4 §8.
- **Phase 4** — "Open in 360.3D" / "Open in CNS" / "Open in The Cataloger" row gestures. Per v1.4 §7.2 / §7.3 / §7.4.
- **Phase 5** — User-composed lenses. UI for building a `.base` block from scratch (lens-name / scope / where / order / columns / view).
- **Phase 6** — Table + card view shapes (in addition to list). Earned by Form-Aligns-To-Purpose-justified questions.
- **Phase 7+** — Aggregations / NL→lens / Wings integration (per v1.4 §11 Out of Scope items).

Each phase is its own `/migration` MIG number (MIG-056, MIG-057, ...).

## §8. Phase 1 step decomposition (high-level — Plan doc has details)

1. **§A — Dimension registry.** New module `src-tauri/src/lens/`. `DimensionDef` + 4 dimensions (name / headline / created_at / path). Unit tests.
2. **§B — YAML schema parser.** `LensDefinition` Rust struct + serde_yaml parse + schema validation (unknown dimensions, unsupported ops). Unit tests covering each error mode.
3. **§C — `execute_lens` Tauri command.** SQL builder from `LensDefinition`. Federation-aware library scoping. NSC headline fetching. Returns `LensResult`.
4. **§D — Markdown ` ```base ` block extractor + renderer.** A new Svelte component `LensBlock.svelte` mounted by the existing markdown renderer when it encounters a `base` language fenced code block. Calls `execute_lens(block.text, ...)`, renders rows.
5. **§E — System note: "Observation — Recent Captures".** Ships in `{universe}/.constellation/system-notes/Five Acts/` (or equivalent — exact path locked at Plan). Created by a new `init_system_notes` function called from `init_db` if absent. Idempotent.
6. **§F — Sidebar entry.** A new "Five Acts" section in the sidebar (replaces the OLD "Bases" entry). Lists the shipped system notes. Click opens the host note.
7. **§G — Behavioral tests on synthetic universe.** ~10 test cases covering the lens parser / dimension validation / SQL generation / federation scoping / empty-result handling / multilingual content.
8. **§H — 3-agent audit (`/migration` Phase 4).**
9. **§I — Boss-test gate.** Eisa opens his universe → sees the new "Five Acts" sidebar entry → clicks "Observation — Recent Captures" → sees a host note with prose + the rendered list → confirms.
10. **§J — PCS + Orientation v2.37 + help-doc note.**

## §9. Risks

1. **Dimension registry surface lock-in.** v1 names 4 dimensions. Future phases add ~25 more (Living Links + CE + CNS + CECE). The naming convention chosen in §A becomes a contract. **Mitigation:** the §A naming convention is reviewed in v1.4 §6 (dimension lists) — `note.X` / `link.X` / `note.cns.X` / `note.cece.X`. Lock the prefix convention in §A; future phases follow.
2. **NSC headline fetch latency.** Each lens with `note.headline` column fetches NSC summaries for visible rows. NSC has a cache (`note_summaries` table); cold rows might be slower. **Mitigation:** the existing `nsc_get_summaries_for_notes` IPC already handles the cache-first-then-compute path. Reuse it.
3. **System-note creation idempotency.** §E creates the system note on first boot. If the user deletes or edits it, what happens on next boot? **Mitigation:** Plan §E locks the policy — re-create if absent? Diff and warn if user-modified? Lockdown to read-only?
4. **Pre-existing search.rs parse_frontmatter bug.** Still present, still corrupting `properties_json`. v1's lens schema never reads `properties_json` (the 4 v1 dimensions all come from other columns), so the bug is irrelevant to v1. But Phase 2+ will need to address it OR avoid `properties_json` entirely. **Mitigation:** document as PJ-NNN; Phase 2 design decides.
5. **The old `.base` files on disk.** Eisa's universe has files he created (e.g., "MIG-054 Bast Test"). After this MIG, they don't render via the new code (no longer routed to BaseView; no LensBlock parser for standalone `.base` files in v1). **Mitigation:** Plan locks behavior — silently ignored OR surfaced via a "legacy files" notice. Recommend silent ignore — they're artifacts of a discarded concept.
6. **The lens YAML schema being learned twice.** v1 ships schema v1 with 4 dimensions; later phases add more. The schema needs versioning (`schema: 1` field already declared). **Mitigation:** validation enforces schema version; future phases bump to schema 2/3.

## §10. Out of scope for MIG-055

- All v1.4 §11 Out of Scope items (vertical templates / aggregation formulas / NL→query / generative lens suggestions / real-time collaboration / etc.).
- The dataview.rs Tauri command — Untouched. Its filesystem-walk path remains; it's a separate subsystem with its own concept (DQL queries). When/if it's redesigned, that's a separate MIG.
- The pre-existing `search.rs::parse_frontmatter` bug — not Phase 1's concern. Logged as PJ-NNN if needed.
- The old MVP files (BaseView.svelte / BaseTableView.svelte / etc.) — left on disk as dead code. Cleanup is a separate housekeeping MIG.

## §11. Open questions for Eisa

1. **Dimension prefix convention.** I propose: `note.X` for per-note (name, path, created_at, stratum, maturity, etc.) / `link.X` for Living Link properties / `note.cns.X` for CNS measurements / `note.cece.X` for CECE classifications. Lock this convention or propose alternative?
2. **System-note location.** The "Observation — Recent Captures" host note needs a path. Candidates: `{universe}/.constellation/system-notes/Five Acts/Observation — Recent Captures.md` (hidden) OR `{universe}/Five Acts/Observation — Recent Captures.md` (user-visible folder). My recommendation: visible folder — the user SEES the Five Acts as a teaching artifact, browseable like any other note.
3. **System-note edit policy.** If the user edits the system-shipped Observation host note, do we (a) leave their edits (treating it as a user-owned copy after first edit), (b) reset on next boot, (c) prompt? My recommendation: (a) — first edit transfers ownership; the system stops touching it. New universes get a fresh copy.
4. **Old `.base` files on disk.** Silent ignore OR surfaced via a notice? My recommendation: silent ignore (they're discarded-concept artifacts; surfacing them implies they're recoverable).
5. **Sidebar label.** The new sidebar entry that replaces the OLD "Bases" — what's it called? "Five Acts" (the cognitive frame) OR "Lenses" (the technical frame) OR something else? My recommendation: "Five Acts" — anchors the user-facing vocabulary to the cognitive model.
6. **MIG number.** I went with MIG-055 — clean break from MIG-054's reverted code. Confirm or pick a different number.
7. **The `.base` file extension itself.** Keep `.base` (familiar) OR switch to something new (`.lens`?) given the concept shift? My recommendation: keep `.base` — the user already knows it; the format inside is the change, not the extension.

## §12. Predecessor and Adjacent Documents

- **Predecessor (vision):** `docs/Constellation-Base-Concept-Paper-v1.4.md` — the design north star. Unchanged.
- **Predecessor (reverted approach):** `docs/MIG-054-bases-rule8-migration-ARCHITECT.md`. The Architect for the rejected approach. Kept as historical record. Its §A–§I.0 code was reverted at commit `15c41504`.
- **Adjacent — schema source of truth:** `src-tauri/src/search.rs` (the `note_meta` schema, `note_summaries` schema, federation resolver).
- **Adjacent — concept reference:** `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` (Living Link Architecture, Five Acts).
- **Successor:** Plan doc — phase-decompose §8 into landable commits with verification clauses.

---

## §13. Closing

This Architect doc breaks from MIG-054 cleanly. **No carryover from the old MVP concept.** First user-visible deliverable: Recent Captures, a single Five Acts template rendered via the host-note assemblage pattern. Three of the six differentiators exercised end-to-end (SQL query / NSC headlines / federation auto). Future phases add the rest one dimension family at a time.

Awaiting Eisa's approval before drafting the Plan doc.
