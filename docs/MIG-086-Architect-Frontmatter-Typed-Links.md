# MIG-086 (Part 2) — Architect: Typed Links in Frontmatter

**Doc:** `docs/MIG-086-Architect-Frontmatter-Typed-Links.md` (net-new, 2026-06-24)
**Parent:** `docs/MIG-086-Architect-Reviewer-Link-Suggestions.md` + `docs/MIG-086-Plan.md`
**Branch / root:** `main` @ `E:\مشاريع كلاود\Constellation`
**Status:** Architect (Phase 1). Plan follows in `MIG-086-Plan.md` Part 2; build after Boss approval.

---

## 1. Why this exists (the trigger)

§C shipped a one-click connect that appended `[[type::Target]]` to the **end of the source note's body**.
Boss finding (2026-06-24): *"a typed link hanging there without context is illogical."* He is right — an
appended-at-EOF link is neither a **contextual** link (woven into an argument) nor honest **metadata**; it's
a dangling artifact. Boss ruling: **continue MIG-086 §C and fold a frontmatter-typed-links model into this
migration.**

## 2. Concept (the horse)

A typed relationship has two honest homes, answering two different questions:
- **Contextual link** — woven into prose, *where the thought happens* (`the cathedral's [[supports::nave]]…`).
  Stays in the **body**.
- **Declared relationship** — asserted *about* the note, not in its argument (the one-click connect, a TOC,
  a parent). Belongs as **frontmatter metadata** — the metadata block is exactly where contextless-but-
  structured relationships belong.

The one-click connect produces a *declared* relationship (the user judged "these relate" + chose a type; they
did not write a sentence). So it belongs in frontmatter. This resolves the dangling-link problem at the root.

## 3. Research basis (fact-checked — `wf_ce7372d6-d02`, 2026-06-24)

- **Obsidian Properties** (since v1.4, Aug 2023): a YAML property can hold quoted `"[[wikilinks]]"`; in
  *current* Obsidian these appear in **both backlinks and the graph** natively (the early-2024 "graph gap"
  is resolved as of 2026). Community "link-type properties" (`parent:`, `related:`, `source:`) are widespread.
- **Breadcrumbs** (the closest precedent): declares **typed, directional** edges as frontmatter properties
  (`parent: ["[[A]]"]`), and **auto-derives the reverse as an "implied" relation** (A parent B �implies B
  child A), distinguishing real vs implied. This is the model PJ-065's structural type will adopt.
- **Dataview**: body inline `key:: [[X]]` stays a real, rename-safe, graph-visible link; a quoted `[[X]]` in
  YAML is (in stock Obsidian/Dataview) more limited — but **Constellation owns its own parser + rename
  cascade**, so we make frontmatter links first-class on our side. "Mix frontmatter + inline" is the docs'
  own guidance.
- **Tana**: schema-on-write typed fields (most structured) — lesson: define the relation once, enforce on
  every instance. **Single canonical write path matters** (Logseq's two-syntax pitfall is the cautionary tale).
- **KR theory** (RDF triples / Neo4j property graphs / reification): a relationship carrying properties is a
  *reified labeled-property-graph edge*. Frontmatter is a fine home for a link's **birth** (type + target),
  but **not** for the rich 8 properties — those belong in the **index** (and, end-state, a first-class link
  object). Constellation's `note_links` table IS that property-graph engine.

## 4. Decisions (Boss-ratified 2026-06-24)

- **D1 — Shape: TYPE-AS-PROPERTY.** Each link type is a YAML key; value is a list of quoted wikilinks:
  ```yaml
  supports:
    - "[[Cellular respiration]]"
  derives-from:
    - "[[Krebs cycle]]"
  ```
  Reuses the existing PropertyEditor list-of-links UI; rename-cascade rewrites the values; PJ-065's `parent:`
  is just another key. (Rejected: a single structured `links:` object-list — YAML-heavy, new UI, less
  interoperable. Per-link confidence/weight/annotation are NOT in the file; they live in the index, as today.)
- **D2 — DUAL-SOURCE, non-destructive.** `index_note` reads typed links from **both** the body
  (`[[type::target]]`, contextual) **and** frontmatter (type-as-property, declared). Both feed `note_links`.
  Existing body links keep working; **no forced migration** of existing links.
- **D3 — `note_links` stays the single index + the earned-property store.** Single-writer invariant holds:
  the link is *born* as file text (body or frontmatter); `index_note` derives the row; confidence/weight/
  traversal are index-maintained at write-time. No code writes `note_links` directly.
- **D4 — The connect writes FRONTMATTER** (replaces §C's body append) via the **props save path**
  (`composeNoteModel`+`editNoteProps`+`saveTabContent` for open; `readNote`+`parseFrontmatter`+`writeNote`
  for closed). This is *safer* than the body-append cascade: `saveTabContent` already writes props to the
  model correctly, sidestepping the open-note body landmine. Confidence default `hypothesis` still comes from
  `index_note` (C-4, automatic).
- **D5 — Auto-implied reverse: DEFERRED to PJ-065.** For the 8 cognitive types the reverse is shown as a
  backlink (already how the index works); we do NOT auto-write a reverse typed link (direction carries
  meaning: "A supports B" ≠ "B supports A"). The Breadcrumbs implied-reverse mechanic is the heart of the new
  **structural** type (parent/child/TOC) and lands with PJ-065, not here.
- **D6 — Rename cascade extends to frontmatter.** `update_links_on_rename` must rewrite quoted
  `"[[oldname]]"` inside frontmatter typed-link properties, so renames never break a frontmatter link
  (the Obsidian "frontmatter links break on rename" failure mode must NOT exist in Constellation).

## 5. Invariants that must not break

1. **Single-writer** — links born as file text; `note_links` derived; never written directly (Living Link).
2. **Dual-source parity** — a relationship reads identically downstream whether authored in body or
   frontmatter; panels/Sky/Reviewer/maturity are source-agnostic (they read `note_links`).
3. **No double-count** — the same `type::target` declared in BOTH body and frontmatter yields ONE `note_links`
   row (dedup on `(source, target, type)`; the table's UNIQUE index already enforces this).
4. **Content-integrity (BUG-015 class)** — the connect goes through the props save path; on-screen===disk
   after every connect; the full 8-point Editor-Surface Gate.
5. **Rename safety** — frontmatter links survive rename (D6); linked-probe pair intact.
6. **Rule 8 / Rule 1+3** — frontmatter parse is part of the existing `index_note` write-time derivation; no
   new boot scan; suggest stays on-demand.
7. **Back-compat** — every existing body `[[type::target]]` link keeps working unchanged.
8. **i18n / RTL** — frontmatter type keys are canonical ids; their *display* localizes via `linkTypes.*`
   (the registry/pill), never the stored key.

## 6. Cross-surface impact (blast radius)

| Layer | Change | Severity |
|---|---|---|
| `search.rs::index_note` / `extract_typed_links` | NEW: parse frontmatter type-as-property links; merge+dedup with body links | HIGH |
| `store.ts::addLinkToNote` | REWRITE: write frontmatter property (props save path), not body append | MED |
| `libraries.rs::update_links_on_rename` (+ JS cascade) | EXTEND: rewrite quoted frontmatter wikilinks | MED |
| PropertyEditor / NotePane sidebar | DISPLAY frontmatter typed-link properties (type pill + clickable target) | MED |
| Backlinks / Outgoing / 360 / Sky / Reviewer / Knowledge Health / maturity | NONE (read `note_links`) | — |
| Body `[[` autocomplete (`completions.ts`) | NONE (body path unchanged) | — |

## 7. Migration / back-compat

- **First boot after ship:** `index_note` re-derivation picks up any frontmatter typed-links already present;
  body links untouched. No bulk migration job; no schema change to `note_links` (columns already exist).
- **Existing body links:** left as-is (dual support indefinitely). An optional future "normalize declared
  links to frontmatter" tool is out of scope (note for PJ-065 era).
- **Rollback:** the frontmatter parse is additive; disabling it falls back to body-only with no data loss
  (frontmatter links simply stop being indexed; the YAML stays on disk, human-readable).

## 8. Open questions for the Plan (none block Architect)

- Exactly how PropertyEditor renders a type-as-property link (pill + clickable) vs. a plain list property —
  a §FM-4 detail; the connect + index + rename (§FM-1..3) are independent of it.
- Whether the §D hosts (NotePane sidebar / 360 / Sky node) offer a **direction choice** for non-orphan notes
  (the earlier Boss question) — proposed: defer the direction toggle; declared links default to
  "in-hand note → suggestion" once we're outside the orphan lens. Confirm at §D.
