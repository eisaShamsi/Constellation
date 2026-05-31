# User-Definable Link Types ("The Living Vocabulary") — Plan

**Phase 2 of the `/migration` workflow (Plan). Approve before any code.**
**Date:** 2026-05-31 · **Architect:** `docs/USER-DEFINABLE-LINK-TYPES-ARCHITECT.md` · **Governing:** Strong yet Simple; the 8 are a living seed, not a cage.
**MIG number:** TBD by Eisa (proposed next; "Epistemics" can renumber).

## Locked decisions (Eisa, 2026-05-31 — "all yes")
1. The 8: **color/order/label editable; id + meaning + existence locked.**
2. Scope: **per-universe** (`.constellation/link-types.json`, the `custom_stages` pattern).
3. Custom-type order: **user-draggable** (order = the canonical sort key).
4. Nesting: **one level** in v1 (a child under one of the 8).
5. Per-type columns: **leaf-only** in v1 (parent-sum columns → v2).
6. Analytic surfaces (**360.3D / CECE / tension**; Sight is out of core): **neutral default** for custom types in v1; user-assignable weight → v2.
7. Management UI: **Settings → Link Types** editor.

The migration **folds in MIG-066 §C/§D/§E** (render / rank-sort / surface-reconcile — all change once types are user-defined). MIG-066 §A/§B (columns + per-type counts) already shipped + Boss-confirmed.

---

## Invariant carried at EVERY phase
The app stays fully working after each commit, and **a universe with no custom types behaves byte-identically to today** (the 8 seeds reproduce the current behaviour). Each phase is additive; the hardcoded lists are removed only once their replacement reads the registry.

---

## Phases (each lands as one commit with a verification clause)

### §A — Backend registry + parser (the spine, additive)
- New `link_types` module (Rust): `LinkTypeDef { id, label, parent: Option<String>, color, order, builtin, emoji, desc }`; the **8 built-in seeds** (ids + semantics + derived order 1–8 + canonical colors from `DEFAULT_SETTINGS.linkPills`); `LinkTypeRegistry` = `merge(seeds, link-types.json deltas)`; an `Arc<RwLock<…>>` loaded at boot + a cheap id-set snapshot.
- `universe.rs`: `read_universe_link_types` / `save_universe_link_types` (the `property-types.json` path); init `link-types.json` (empty deltas) in `create_universe`; Tauri commands in `lib.rs`; `list_link_types` (resolved registry for the frontend).
- `extract_typed_links` / `parse_link_body` read the registry id-set instead of `const PARSER_LINK_TYPES` (the 8 still recognized; custom ids now too; `associative` default unchanged).
- **Verify:** Rust tests — the 8 seeds load with their derived order; a custom top-level + a child round-trip through `link-types.json`; the parser recognizes a seeded custom id and still defaults unknown to `associative`; an empty/corrupt file falls back to the 8 seeds. `cargo test` green; a no-custom universe parses identically.

### §B — Materialization from the registry (dynamic rank + JSON column + change-flow)
- `outgoing_aggregate_assignments` builds `LINK_TYPE_IN_LIST` + the rank `CASE` **from the registry order** (not the `const`); `outgoing_top_rank` sentinel = `registry.len()+1`.
- **ADD `note_meta.outgoing_link_types_json`** (`{"supports":358,…}`) materialized in the same assignment (idempotent ALTER). Display string `outgoing_link_types` unchanged in shape (still `type (count)`).
- **Vocabulary-change flow:** a `schema_versions`-style stamp of the registry hash; on boot (or on save) if it changed → `drop_outgoing_link_triggers` → `create_outgoing_link_triggers` (regenerated CASE) → `recompute_all_outgoing` (batched + lock-tolerant — already hardened). Re-uses the §A.2 mechanism.
- **Verify:** Rust tests — 8-only output identical to today + JSON populated `{type:count}`; adding a custom type to the registry → trigger CASE includes it + a recompute materializes it; the registry-hash stamp gates exactly one recompute. `cargo test` green; measure: no boot/typing regression (registry is one cached read).

### §C — Frontend registry store
- `src/lib/links/linkTypeRegistry.ts` (mirrors `propertyTypeRegistry.ts`): in-memory store, `seedFromBundle`, debounced persist; `universe/store.ts` IPC wrappers; `boot_bundle` carries `link_types`; `+layout.svelte` seeds it.
- **Verify:** the store exposes the resolved registry (8 + custom, ordered, nested) on boot; an edit persists + re-seeds; no extra boot IPC (rides the bundle).

### §D — Reconcile the ACTIVE surfaces to the registry (= MIG-066 §E)
- Replace the hardcoded lists in the ACTIVE surfaces with registry reads: `editor/completions.ts`, `editor/livePreview.ts` (`TYPED_LINK_TYPES` set), `libraries/store.ts` (`KNOWN_LINK_TYPES`, `LINK_TYPE_NAMES`), `Inspector360.svelte` (`TYPE_ORDER/COLORS/LABEL_KEYS` — gains `supersedes`), `inspector360.rs` (`ALL_LINK_TYPES` — gains `supersedes`), the Rust `KNOWN_LINK_TYPES` in `strata.rs`/`tension.rs`/`libraries.rs`. **Exclude** the disabled Sight surfaces + the `ConstellationEditor/` legacy. Retire the dead `CodeMirrorEditor.svelte` legacy list.
- **Verify (Boss-testable):** the 8 still autocomplete / render / color / appear in 360.3D exactly as before; the existing drift is gone (360.3D now shows `supersedes`); `svelte-check` = only the 3 pre-existing errors.

### §E — Editor: inline colors + autocomplete for custom types
- `livePreview.ts`: per-type decoration color from the registry as an **inline style** (pre-cached `Decoration.mark` per registry id, rebuilt only on registry change — Rule 1); `completions.ts` lists the registry (id + `desc`), emits `[[type::target]]`.
- **Verify (Boss-testable):** type a custom link → it renders in its registry color + autocompletes; typing latency unchanged (10-char burst test); the 8 unchanged.

### §F — Dynamic per-type sortable columns
- `resolve_dim` accepts `note.link.<typeid>` → `COALESCE(json_extract(note_meta.outgoing_link_types_json,'$."<id>"'),0)`, Number, **sortable** (rides the `prop.<key>` pattern; validator + sql_builder + federation + `.base` persistence all already handle any `resolve_dim` hit). `BaseColumnPicker` gains a **"Link types" tier** that lazy-discovers the registry (a `discover_link_types` command); `tableModel.ts` `isSortable`/`columnLabel` handle `note.link.*`.
- **Verify (Boss-testable):** + Add column → Link types → pick `Supports` → a sortable count column; sort the Base by it (most-supports-first); add a custom type's column too.

### §G — Settings → Link Types editor (the management UI)
- A Settings section: list the registry (8 + custom, nested), **add** a type (id/label/color/emoji, top-level or child-of-one-of-8), **reorder** (drag), **recolor** (reuse the link-pill picker), **delete** (custom only; the 8 are locked except color/order/label). On save → persist `link-types.json` → triggers the §B re-materialize.
- **Verify (Boss-testable):** create a top-level type + a sub-type under `supports`, color + order them; they appear in autocomplete, the editor, 360.3D, and as columns; the 8 can be recolored/reordered but not deleted.

### §H — Localization + docs + Concept Paper v1.1
- The 8's labels localized (all 15 locales — `link_*`/`help_type_*` gain `supersedes`); custom types use the user's own label. Concept Paper gains the "living seed, not a cage" section + bump to **v1.1**. Help (Bases + a new "Link Types" topic) + User Manual + **orientation v-bump**.
- **Verify:** 8 labels localize (spot-check 2 languages); custom labels show as-authored; docs land in the same commits.

### §I — Audit + PCS (`/migration` Phase 4)
- Three parallel agents: **invariants** (the 8 immutable; no-custom = identical; Rule 1/8; boot/typing) · **drift** (every active surface reads the registry; nothing still hardcodes the 8) · **migration-path** (empty/corrupt `link-types.json`; registry change mid-reconcile; a federated Base across differing vocabularies; rollback).
- Staged Boss test; PCS (push, milestone tag, ZIP).

---

## Invariants (the audit floor)
The 8 ids/semantics/order immutable · no-custom universe byte-identical to today · existing `[[type::target]]` links unchanged · Rule 8 (per-type counts materialized) · Rule 1 (decorations pre-cached per id) · boot/typing/IPC unchanged on 7,600+ notes (measured) · file-over-app (portable `link-types.json`, deltas only) · localization (8 localized, custom user-labelled) · vocabulary-change ⇒ re-materialize (hash-gated) · federation honesty (union vocabularies; absent type = 0) · disabled Sight + `ConstellationEditor/` excluded.

## After approval
Plan-Approval = Build-Approval: on "approved" I cascade §A→§I, pausing only at the Boss-testable clauses (§D / §E / §F / §G) and logging each `§` commit. Open input that remains: the MIG number, and any rename of the canonical 8's default colors.
