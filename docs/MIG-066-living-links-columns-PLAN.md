# MIG-066 — Living-Links Columns in the Base — Plan

**Phase 2 of the `/migration` workflow (Plan). Approve before any code.**
**Date:** 2026-05-30 · **Architect:** `docs/MIG-066-living-links-columns-ARCHITECT.md` · **Governing:** Strong yet Simple, by default.

## Locked scope (Boss decisions, 2026-05-30)

- **v1 columns:** **Outgoing count** + **Link types** (my lean — the two most legible).
- **Backlinks → v2** (the federated design is deferred; v1 is honestly outgoing-only).
- **Multi-type cell:** **list the types** (canonical order, localized).
- **Rank-aware sort:** **co-design here** — build the general mechanism, apply it to link types.

The two columns are **same-DB / outgoing-side**, so they materialize cleanly write-time (Architect §4–5, Option C). No backlink/cross-library work in v1.

---

## Phases (each lands as one commit with a verification clause)

### §A — Materialize the outgoing-link aggregates (backend, write-time)
- `note_meta` gains two additive columns (idempotent `ALTER TABLE … ADD COLUMN`): `outgoing_count INTEGER DEFAULT 0`, `outgoing_link_types TEXT DEFAULT ''` (the distinct `link_type`s where this note is the **source**, `status='active'`, stored in canonical order — see §D for the order key).
- Extend the existing `note_links` trigger family (AI/AD/AU) to recompute **the affected source note's** two fields on every edge change (a same-DB subquery over `note_links WHERE source_path = …`). Read-path stays a plain column read (Rule 8).
- Resumable **background back-fill** for existing `note_links` (status-bar progress; `schema_versions` gate so it runs once; chunked + WAL-friendly per the MIG-041 precedent).
- **Verify:** Rust test — insert/delete/archive `note_links` rows → assert `note_meta.outgoing_count`/`outgoing_link_types` update via the triggers; back-fill populates existing rows; re-run is idempotent. `cargo test` green.

### §B — Register the two dimensions + expose in the picker
- Add `note.outgoing_count` (Number) and `note.link_types` (Text) to the `resolve_dim` registry, `sql_expression` reading the materialized `note_meta` columns; both `sortable`.
- Add both to `ADDABLE_REGISTERED_DIMS` so they appear in the picker's **Constellation** group with labels.
- **Verify (Boss-testable):** open a base → **+ Add column** → the **Constellation** group now lists **Outgoing links** + **Link types** → add each → columns populate with live data.

### §C — Render the cells
- `outgoing_count` → plain right-aligned number.
- `link_types` → the materialized set rendered as a **localized, canonical-ordered list** (each typed-link name via `$t`, e.g. "supports, contradicts" → the locale's terms). Empty set = blank cell. RTL per-cell as usual.
- **Verify (Boss-testable):** the Link-types cell reads e.g. "supports, contradicts" in canonical order; switch language → the type names localize.

### §D — Rank-aware sort (general mechanism + link types)
- Generalize sorting so a dimension may declare a **canonical value order**; sorting by it emits a `CASE value WHEN … THEN <index> …` rank key (not alphabetical). **Reusable** for maturity/stage/stratum later (the MIG-068 need, satisfied here for links).
- For `link_types` (multi-value) the sort key is the **top-ranked type present** (lowest canonical index) — so a note that `supports` sorts ahead of one that only `exemplifies`.
- **Canonical link-type order — RATIFIED** (Living-Link Concept Paper v1.0 §7, 2026-05-30), derived from the inquiry arc (stance → explanation → abstraction → lineage → composition → succession):
  `supports · contradicts · causes · exemplifies · generalizes · derives-from · part-of · supersedes`. (`associative` = the null/untyped synonym, not in the semantic order.) The rank-aware sort uses this index.
- **Verify (Boss-testable):** sort the Link-types column → notes order by their top-ranked type in **canonical** order (not alphabetical); reverse flips it.

### §E — Reconcile EVERY core Living-Link surface to the Concept Paper (Boss directive)
The audit found **multiple inconsistent link-type lists**. §E reconciles them all to the ratified order + canonical 8 + `supersedes`, and — to stop drift recurring — introduces a **single shared source of truth** every surface references ("secure the winning — one source, used many times").
- **One canonical list** (frontend shared module + agree the Rust consts), ordered per Concept Paper §7.
- **Reconcile each surface:**
  - `Inspector360.svelte` `TYPE_ORDER` (360.3D matrix) — Camp B order, no `supersedes` → canonical order **+ add a `supersedes` column** (+ `TYPE_COLORS`/`TYPE_LABEL_KEYS`).
  - `inspector360.rs` `ALL_LINK_TYPES` — add `supersedes` (drives the matrix gap analysis).
  - `CodeMirrorEditor.svelte` `LINK_TYPES` — **legacy/wrong set** (`related-to`/`prerequisite`/`see-also`/`extends` — not Constellation types) → the canonical 8.
  - `editor/completions.ts` `LINK_TYPES`, `editor/livePreview.ts` `TYPED_LINK_TYPES`, `store.ts` `KNOWN_LINK_TYPES` → canonical 8 (+ `supersedes`).
  - Color/label maps (`KnowledgeHealthDashboard`, `SightPanel`) — cover all 8 + `supersedes`.
  - `Living-Links Guide §2` → canonical order; bump Guide to **v1.1** (SO #6, same commit).
- **Verify (Boss-testable):** 360.3D matrix reads in canonical order + has a `supersedes` column; the `[[Note|` autocomplete suggests only the canonical 8 (no `related-to`/`see-also`); Backlinks/Outgoing/KHD/Sight link rendering unregressed.

### §F — Localization + docs
- New column labels (**Outgoing links**, **Link types**) + the 8 typed-link names (shared by the Base column AND the matrix) if not already localized → fill **all 15 locales**.
- Update help **Bases** topic (+ the 14 translations), **User Manual** §15, and **orientation** (v-bump — MIG-066 v1 ships).
- **Verify:** every label localized (spot-check 2 languages); docs updated in the same commits.

### §G — Audit + PCS (`/migration` Phase 4)
- Three parallel agents: **invariants** (Rule 8 / boot / typing / federation honesty — the column is *outgoing-only*, must not imply backlinks), **drift** (new triggers/guards the system doesn't know about), **migration path** (first-boot, schema-mismatch, mid-backfill interrupt, rollback).
- Staged Boss test (per the staged-tests rule).
- PCS: push, milestone tag, ZIP backup.

---

## Invariants carried from the Architect (the audit floor)
Rule 8 (no live graph walk on open) · boot/typing/IPC unchanged on 7,600+ notes (measure before/after) · federation honesty (v1 columns are outgoing-only — never a silently-partial backlink number) · Living-Link semantics read-only (weight/confidence/lifecycle untouched) · the 8 typed-link vocabulary preserved · existing Base behavior (prop.*, sort/edit/reorder/virtualization) intact · additive + resumable + reversible schema.

## After approval
The canonical order is **ratified** (Concept Paper §7), so the only open input is closed. Plan-Approval = Build-Approval: on "approved" I cascade §A→§G, pausing only at the Boss-testable verification clauses (§B / §C / §D / §E) and logging each `§` commit.
