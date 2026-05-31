# User-Definable Link Types ("The Living Vocabulary") — Architect

**Phase 1 of the `/migration` workflow (Architect). Design map only — no code.**
**Date:** 2026-05-31 · **Author of facts:** Eisa · **Maintainer:** Claude.
**Folds in:** MIG-066 §C (render) / §D (rank-aware sort) / §E (reconcile surfaces) — they all change once types are user-defined, so finishing them for a fixed 8 then reworking would be waste.
**Number:** TBD by Eisa (proposed next MIG; the planned "Epistemics" MIG can renumber).

> **Function in hand:** let a user **add their own link types** — either **top-level** (a peer of the canonical 8) or **nested under one of the 8** as a sub-type — and give **each type its own sortable Base column**. The 8 stay the stable grammar of inquiry; the user's own structure grows on top. ("Thoughts are an ongoing process; ideas regenerate." — Eisa.)

---

## 1. The decision that shapes everything: ONE registry

Today the 8 types are hardcoded in **~25 places** across Rust + TS (parser, SQL rank, editor, colors, panels, Sight, i18n, the Base). They already **drift** (Inspector360 is missing `supersedes`; several color maps disagree). Adding a 9th type by editing 25 sites is untenable.

**So the spine of this migration is a single source of truth — a Link-Type Registry — that every surface reads from.** This simultaneously: (a) enables custom types, (b) **delivers MIG-066 §E** (all surfaces reconciled to one list), and (c) fixes the existing drift. Nothing else in this doc works without it.

---

## 2. The data model

A link type:

```
LinkTypeDef {
  id:       string            // "supports" | "empirically-supports" | "inspires"  (slug, lowercase-hyphen)
  label:    string            // display name (user types are user-labelled; the 8 localize via i18n)
  parent:   string | null     // null = top-level; else a canonical-8 id (v1: only the 8 can be parents)
  color:    string            // hex; the 8 seed from DEFAULT_SETTINGS.linkPills, customizable
  order:    number            // position within its tier (top-level among top-level; child among siblings)
  builtin:  boolean           // true = one of the 8 (semantics + id fixed); false = user type (fully editable/deletable)
  emoji?:   string            // optional (mirrors custom_stages)
  desc?:    string            // optional one-liner (the autocomplete "detail" + help)
}
```

- **The 8 canonical are built-in seeds** (`builtin: true`): ids + semantics fixed (Concept Paper §6–7), `parent: null`, ranks 1–8. Their **color/order/label-override** may be user-customizable; their **identity** is protected (can't be deleted/renamed — the grammar).
- **Custom types** (`builtin: false`): user id/label/color; `parent: null` (top-level peer) **or** one of the 8 (a sub-type that refines that act).
- **`associative`** = the null/untyped synonym; stays special (never listed as a "type" to add).

**Ordering (the canonical order generalizes):** the registry yields an ordered, possibly-nested list — the 8 in their derived order (§7), each canonical type optionally followed by its children, then custom top-level peers appended (or user-dragged). The integer **rank** used by materialization = the flattened index in this order. Children sort *within* their parent's slot (e.g. `supports` then `empirically-supports` then `theoretically-supports`, before `contradicts`).

---

## 3. Storage — `link-types.json` (the `custom_stages` / `property-types.json` pattern)

**Per-universe**, at `.constellation/link-types.json` (the vocabulary is universe-wide, like `custom_stages` in `universe.json`). It stores **custom types + user overrides of the 8** (deltas); the 8 seeds live in code, so a corrupt/edited file can never break the grammar. On load: `registry = merge(8 code seeds, file)`.

End-to-end path is the **exact** `property-types.json` model (verified):
- **Rust** (`universe.rs`): `read_universe_link_types` / `save_universe_link_types`; Tauri commands in `lib.rs`; initialized in `create_universe`.
- **Boot** (`boot_bundle.rs`): add `link_types` to the boot bundle (one IPC, no extra round-trip).
- **Frontend** (`universe/store.ts` + a new `links/linkTypeRegistry.ts` mirroring `propertyTypeRegistry.ts`): in-memory store, `seedFromBundle` in `+layout.svelte`, debounced persist on edit.
- **Rust index** keeps an `Arc<RwLock<LinkTypeRegistry>>` loaded at boot; the parser + materialization read it.

**Federation note:** each universe has its own vocabulary; a federated Base spanning cUniverses may see types that exist in one but not another — the registry read must union across attached universes (parallels how `discover_keys` unions schemas).

---

## 4. The seven design axes (options · effort · risk)

### 4.1 The registry (backend + frontend) — **the spine**
Centralize every hardcoded list (§6) to read the registry. **Effort: ◐◐◐ (broad — ~25 sites).** Risk: ✓ once done it's the single point; it's also §E. **Recommend: do it first, in two halves — Rust registry + IPC, then frontend store — each surface migrated + tested.**

### 4.2 Parsing — recognize custom ids
`parse_link_body` / `PARSER_LINK_TYPES` → `registry.is_known(id)`. **Effort: ◐ low.** Risk: ⚠ the parser runs at index time on a background thread — it must read the registry cheaply (an `Arc<RwLock>` or a cloned `HashSet` snapshot). **Recommend: snapshot the id-set at reconcile start.**

### 4.3 Materialization + rank — **the subtle one**
The `outgoing_link_types` "type (count)" string + `outgoing_top_rank` are built by `LINK_TYPE_RANK_CASE` (hardcoded 1–8) inside `outgoing_aggregate_assignments`, which is **baked into the triggers** at `init_db`. With a dynamic registry:
- Generate the rank `CASE` **from the registry order** at trigger-creation + back-fill time.
- **ADD a JSON column `outgoing_link_types_json`** = `{"supports":358,"contradicts":1,…}` alongside the display string — this is what per-type columns sort on (§4.5). Materialize both in the same assignment.
- **Vocabulary-change flow:** when the registry changes (add/reorder/delete a type), regenerate the CASE → **re-create the outgoing triggers** (`drop`+`create`, the §A.2 mechanism) → **`recompute_all_outgoing`** (now batched + lock-tolerant). i.e. "edit vocabulary ⇒ re-materialize," analogous to a mini schema bump.
**Effort: ◐◐ moderate.** Risk: ⚠ the re-materialize-on-change flow + keeping triggers and registry in lock-step (a `schema_versions`-style stamp of the registry hash gates a recompute on boot).

### 4.4 Editor colors — **the hard UI part**
Live-preview colors are **hardcoded CSS classes** `.cm-link-<type>` — you can't pre-write CSS for unknown ids. Options:
- **(a) Inline styles** — the decoration sets `style="color:<hex>"` from the registry instead of a class. **Recommend** (only way to color arbitrary types; pre-cache a `Decoration.mark` per registry id, rebuilt when the registry changes).
- (b) Inject a `<style>` block of `.cm-link-<id>` rules generated from the registry at runtime. Works but fights CM6.
**Effort: ◐◐ moderate.** Risk: ⚠ Rule-1 (typing hot path) — decorations must stay pre-cached per id, not built per keystroke; rebuild the cache only on registry change.

### 4.5 Per-type Base columns (dynamic dimensions)
`resolve_dim` already does runtime `prop.<key>` columns. **ADD `note.link.<typeid>`** → `COALESCE(json_extract(note_meta.outgoing_link_types_json,'$."<id>"'),0)`, `Number`, **sortable**. Sorting, federation (by-ordinal), and persistence (the `.base` YAML stores `note.link.<id>` as a column name) then work **with no further change** (validator + sql_builder already handle any `resolve_dim` hit). The picker gains a **"Link types" tier** that lazy-discovers the registry's types (like `discover_base_properties`). Sub-types: a column per leaf id; a parent column can sum its children (a `json_extract` sum) — **defer parent-aggregation to v2**. **Effort: ◐ low-moderate.** Risk: ✓ low (rides the proven `prop.*` rails).

### 4.6 Autocomplete + the editor write form
`completions.ts` lists the registry (id + `desc`), emits the canonical `[[type::target]]` form (already done for the 8). Sub-type ids are just longer slugs. **Effort: ◐ low.**

### 4.7 The other consumers (360.3D / Inspector360, CECE, stratum, tension)
These weight/score/analyze by type (CECE graph weights `cece/catalogers/graph.rs`; tension = `contradicts` `tension.rs`; stratum signals `strata.rs`). Most already have a `_ => default` fallback (custom types get neutral weight) — **acceptable v1**: custom types are first-class for *linking + columns*, and inherit a sensible default in the analytic surfaces (a v2 can let users assign a custom type's "semantic axis"/weight). **360.3D (Inspector360)** is the active visual analytic: `inspector360.rs` `ALL_LINK_TYPES` + `Inspector360.svelte` `TYPE_ORDER/TYPE_COLORS/TYPE_LABEL_KEYS` must read the registry (and gain `supersedes` — the existing gap). **Effort: ◐◐ (mostly mechanical).**

**OUT OF SCOPE — disabled "Sight" surfaces.** Sight (the whole-universe visual) is **no longer in core** — replaced by 360.3D (Eisa, 2026-05-31; cf. MIG-038 disable 2026-05-19, re-categorized as a "Constellation Wing"). Its files (`src-tauri/src/sight.rs`, `src/lib/sight/**`, `SightPanel.svelte`, `ConstellationSight.svelte`, `ConstellationSight2.svelte`) carry their own link-type color/weight maps but are **disabled / flags-off**, so — exactly like the `ConstellationEditor/` legacy subtree — they are **excluded from v1 reconciliation**. They'd read the registry only if/when Sight re-ships as a Wing. (The Explore sweep counted them in its "~25 surfaces"; the *active* count is lower.)

---

## 5. Concept-Paper reconciliation

The Concept Paper (RATIFIED) calls the 8 "the cognitive vocabulary." This migration **extends, not contradicts** it: the 8 remain the *derived, canonical grammar of inquiry* (the seed); user types are the operationalization of the paper's own thesis that **understanding is an ongoing, regenerating process**. Add a §: *"The vocabulary is a living seed, not a cage — the 8 are the stable grammar every thinker starts from; a thinker may grow their own acts (top-level) or refine the 8 (sub-types) as their inquiry demands."* Bump the Concept Paper to v1.1 in the same migration (SO #6). The canonical *order* still governs the 8; custom types order after/under them.

---

## 6. Invariants (the audit floor)
- **The 8 canonical ids + semantics + derived order are immutable** (built-in seeds; never deletable/renameable). A corrupt `link-types.json` cannot break the grammar.
- **Existing links keep working** — every current `[[type::target]]` (the 8) parses + materializes unchanged.
- **Rule 8** — per-type counts materialized write-time (the JSON column); no live graph walk on Base open.
- **Rule 1** — editor decorations pre-cached per registry id; registry reads off the keystroke path.
- **Boot / typing / IPC** unchanged on 7,600+ notes (the registry is one cached read; measure).
- **File-over-app** — the vocabulary is a portable `.constellation/link-types.json`; deltas only.
- **Localization** — the 8 localize (i18n); custom types use the user's own label (their language by definition).
- **Vocabulary-change consistency** — add/edit/delete a type ⇒ triggers regenerated + `recompute_all_outgoing`; a registry-hash stamp gates a one-shot recompute on boot if the file changed out-of-app.
- **Federation honesty** — a federated Base unions vocabularies; a type absent in one universe shows 0, not error.

---

## 7. Decisions for Eisa (before the Plan)

1. **Are the 8 themselves editable, or only extendable?** Recommend: **color + order + label-override editable; id + semantics + existence locked.** (The grammar is protected; the presentation isn't.)
2. **Scope = per-universe** (like `custom_stages`)? Recommend **yes**. (Alternative: per-library — more granular but more confusing across a federated Base.)
3. **Custom-type ordering:** appended after the 8 in creation order, or **user-draggable**? Recommend **user-draggable** (a small reorder UI; matters because order = the canonical sort key).
4. **Sub-type depth:** one level (child of a canonical 8) in v1, or arbitrary nesting? Recommend **one level** for v1 (covers your example; arbitrary trees are a v2).
5. **Per-type columns: leaf-only** in v1, with **parent-sums deferred to v2**? Recommend **yes**.
6. **Analytic surfaces (360.3D / CECE / tension weights) for custom types:** neutral default in v1, user-assignable "semantic weight/axis" in v2? Recommend **yes** (keeps v1 shippable). [Sight is out of core — see §4.7.]
7. **Where users manage the vocabulary:** a new **Settings → Link Types** editor (add/color/order/nest/delete), reusing the linkPills color UI? Recommend **yes**.

---

## 8. Decision point
End of Architect. On Eisa's answers to §7, I write the **Plan** (phase-by-phase, each one commit + a verification clause): registry (Rust) → registry (frontend) → parser+materialization+JSON column+vocabulary-change flow → §E surface reconciliation → editor inline colors + autocomplete → dynamic per-type columns + picker tier → Settings vocabulary editor → i18n/docs/Concept-Paper v1.1 → audit. Sources for the territory map: the three Explore sweeps (2026-05-31), logged in the session log.
