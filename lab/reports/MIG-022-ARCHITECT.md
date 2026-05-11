# MIG-022 — Architect doc

**Phase:** 1 of 4 (`/migration` — Architect → Plan → Build → Audit)
**Date:** 2026-05-11
**Predecessor:** MIG-021v3 (CECE) — closed `407d79b`, ships
**Function in hand:** Map the territory + enumerate design options for the response to (a) the gap analysis (`docs/epistemic-content-gap-analysis.md`), (b) PJ-040 (UA partial-frontmatter), (c) PJ-041/042/043 (engine-output i18n gaps), and (d) audit F1 (legacy classifier dead-code cleanup). Surface decision points so Eisa can lock the MIG-022 scope before Phase 2 Plan.

---

## 1 · Sources reviewed

What I read in this Architect cycle:

- `docs/epistemic-content-gap-analysis.md` (339 lines, in full) — Eisa's analytical addendum identifying three structural gaps + five minor extensions + recommendations §6.1/§6.2/§6.3.
- `docs/Constellation Pending Jobs v1.9.md` — PJ-040, PJ-041, PJ-042, PJ-043 entries in full.
- `lab/reports/MIG-021v3-V3-§11-FINAL-INTEGRATION-AUDIT.md` — F1 dead-code finding context.
- `src-tauri/src/cece/cataloger.rs` (full, 200 lines) — confirmed `Confidence` enum at line 37-47 with `#[serde(rename_all = "lowercase")]`.
- `src-tauri/src/cece/synthesis.rs` (lines 92-167, 423-433) — confirmed `user_authority_short_circuit` shape + the test `user_authority_short_circuits`.
- `src-tauri/src/cece/catalogers/structural.rs:366` and `linguistic.rs:463` — confirmed `fn build_reasoning(...) -> String` exists with hardcoded English templates.
- `src-tauri/src/sources/vertical_taxonomy.rs` (head, 35 lines) — confirmed `VerticalNode { id, en, ar, parent_id, branch }` shape.
- `src-tauri/src/classifier/mod.rs` (head, 40 lines) — confirmed legacy `mod source_definitions; mod tier1_embedding; pub mod scan_job; pub mod tier1_rules; pub mod correction_log;` declarations still exist.

What I did **not** read in this cycle (called out so the Plan phase verifies):

- The full body of `linguistic.rs::build_reasoning`, `structural.rs::build_reasoning`, and the equivalent shapes in `semantic.rs` / `graph.rs` / `user_authority.rs`. PJ-041 says all six exist; I confirmed two by file:line and one (UA short-circuit) by reading `synthesis.rs`. The other three I'm trusting the PJ entry on.
- The full `classifier/mod.rs` IPC surface to confirm F1's "no longer reachable from any IPC" claim. The audit agent reported it; I peeked at the head and saw `#[tauri::command] pub fn classifier_suggest_for_note(...)` still declared. **F1 needs a verification step in its Plan / Build** — "is it actually unreachable, or just one of two parallel paths?"
- The companion epistemic-content papers (`epistemic-content-EN.md`, `epistemic-content-taxonomy.md`, the classifier paper). I have working knowledge of their structure from earlier sessions but did not re-read.
- `correction_log.rs` and `scan_job.rs` — the audit's F1 framing implies these may still be reachable. Need explicit reach analysis before any deletion.

---

## 2 · Territory map — six work clusters

I find six natural clusters in the inputs. They differ widely in scope, leverage, and risk. **MIG-022 should not try to swallow all of them in one cascade.** Section 7 below recommends scoping.

### 2.1 Cluster A — §6.1 YAML metadata extensions (gap-analysis recommendation #1)

**Scope.** Add the following optional frontmatter fields to the note schema, parsed and surfaced but not classified:

| Field | Purpose | Maps to gap |
|---|---|---|
| `held_by` | "user" / scholar name / school | §1.3 contestation/agent (partial) |
| `warrant` | grade label (mutawātir / mashhūr / āḥād / etc.) | §1.2 justification (display only — no classifier yet) |
| `warrant_notes` | free text supporting the warrant grade | §1.2 |
| `domain` | list — fiqh / photography / overland-travel / etc. | §2.1 minor extension |
| `function` | reference / seed / actionable / shipped | §2.2 minor extension |
| `provenance_civilization` | sunni-usuli / analytic-western / etc. | §2.4 minor extension |
| `updated_at` | ISO date of last epistemic state revision | §1.1 temporal (single-snapshot only) |
| `supersedes` | note-id this one replaces | §2.5 logical relations |
| `contradicts` | list of note-ids this one disagrees with | §2.5 logical relations |
| `ikhtilāf` | list of `{ school, position }` objects | §1.3 contestation, structured |

**Touch surface.** Frontmatter parser (`src-tauri/src/notes/frontmatter.rs` or wherever YAML is parsed today — Plan phase verifies path), the Properties panel, the Source Review trail (display only — no new classifier), help docs + User Manual chapter on the new fields.

**Standard pattern.** Schema.org / Dublin Core / FOAF approach: add optional fields, document them, keep them inert until tooling consumes them. The gap analysis's §6.1 recommendation maps to this pattern exactly. **Battle-tested:** every metadata vocabulary on the web (RDFa, JSON-LD, schema.org) grew this way.

**Constellation-native variant.** Living Links already provide a typed-link relation primitive — `supersedes` and `contradicts` could become two more typed-link names rather than YAML scalars. This trades flat-YAML simplicity for graph-native consistency. **Decision point D-A1.** (See §6.)

**Big invariant at risk.** The YAML schema growth must NOT regress note-load performance for the 7,600+ note universe. Frontmatter parsing today is Rust-side fast; staying that way is cheap if we do it right.

**Effort.** Mid (~3-5 days agent time). Mostly schema work + UI + docs. No engine internals touched.

**Risk.** Low. Optional fields; existing notes work unchanged; no classifier change.

**Leverage.** Medium-high. Unlocks §1.2 (warrant display), §1.3 (ikhtilāf representation), §2.1-2.5 (all five minor extensions). Foundation for Cluster C eventually.

### 2.2 Cluster B — §6.3 Temporal axis via history layer (gap-analysis recommendation #3)

**Scope.** Per-note version history: every change to `source`, `content_type`, or any §6.1 epistemic field becomes a logged event. Queries like *"show me where my certainty has dropped in the last 6 months"* or *"show the evolution of my stance on this question"* become tractable.

**Touch surface.** Schema (new `note_state_history` table or a `.constellation/history/<note-id>/` event log), the file-save write path (must record state-change events without slowing typing), a query layer, Sight v3 (could overlay a timeline if the data exists), the Settings panel.

**Standard patterns.** Three approaches dominate:

| Approach | Used by | Trade-off |
|---|---|---|
| **Git-style commit log per file** | the gap analysis recommends this; Git itself; Fossil; Datasette | Most conservative — Constellation already encourages Git for sync. Indexing it for queries is the work. |
| **Bitemporal SQL** (valid-time + transaction-time) | Datomic, Crux/XTDB, SQL:2011 temporal tables | Richest query surface. Heaviest schema. Best when you want "what did the system know on date X". |
| **Event sourcing** (per-entity event stream) | Kafka apps, EventStoreDB, the CECE `composite_json` blob in spirit | Best when events have structure. Replays are cheap. Schema is just the event union. |

**Constellation-native variant.** A simpler middle path: an SQLite `note_state_history` table written by an `AFTER UPDATE` trigger on `note_meta` (write-time derivation, per CLAUDE.md Rule 8), keyed by `(note_id, captured_at, axis_changed, old_value, new_value)`. Cheap to maintain, reads are a single SELECT, no on-disk file proliferation. The `.md` file stays the source of truth; the table is the index.

**Big invariants at risk.**
1. **CLAUDE.md "File over app"** — the on-disk `.md` is the source of truth; history must be derivable / re-buildable from disk if needed.
2. **Performance Rule 8 (write-time derivation)** — history must be maintained at write-time, not recomputed on demand.
3. **No regression on boot time or typing latency** — the history table grows linearly with edits; queries must not stutter on a 7,600-note universe with 50,000+ historical events.
4. **Living Links interaction** — Living Links already carry partial temporal data (created, last_traversed, traversal_count, weight). The note history layer is for note-state changes; the Living Link history is for relation-state changes. They should NOT merge into one table; they should NOT duplicate each other.

**Effort.** Large (~2-4 weeks agent time, depending on scope). Schema migration, write-path wiring, query API, UI surfaces.

**Risk.** Medium-high. Touches the write path. Needs a backfill (existing notes have no history; one-time scan to seed `created` events). Performance bar is real.

**Leverage.** High but bounded — directly addresses §1.1 (temporal), and indirectly enables §1.2 (warrant changes over time can be tracked), §1.3 (a position attributed to a school in 2024 may have been attributed differently in 2010), and §2.5 (logical relations evolve). Foundation for many future features (timeline view, "recently changed" filter, undo across sessions).

### 2.3 Cluster C — §6.2 Warrant classifier (gap-analysis recommendation #2)

**Scope.** A NEW classifier head — separate from CECE's 6-cataloger Source × Content Type ensemble — that predicts a warrant grade for each note's claims. Sunni uṣūl vocabulary as the test bed: `mutawātir / mashhūr / āḥād / ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ` plus the conditions on each.

**Touch surface.** New cataloger architecture (or extension of existing), labeled corpus collection + curation, model training or LLM prompting, calibration evaluation, UI surfaces (warrant badge alongside source/content-type badges in Source Review), help docs.

**Standard patterns.** Closest analogues:
- **Academic citation grading** (Web of Science quartiles, Scimago H-index, Altmetric attention scores) — heuristic; doesn't directly transfer.
- **Information-retrieval document-quality scoring** (Lucene BM25, page-rank reputation) — measures document quality not warrant; partial transfer.
- **Provenance ontologies** (PROV-O W3C standard) — schema for *recording* provenance, not for *evaluating* warrant; useful for §6.1 metadata fields, not for the classifier.
- **Hadith authentication research** (Islamic studies CS — there are recent papers on automated isnad chain analysis using graph methods + NLP) — most directly relevant; small literature; not yet a turnkey solution.

**Constellation-native variant.** None — this is genuinely new ground.

**Big invariant at risk.** The CECE 6-cataloger architecture must NOT need restructuring. Warrant is a separate axis with separate semantics; it should be a parallel ensemble, not bolted into the existing one. The existing per-Library reliability layer can be extended for warrant catalogers symmetrically.

**Effort.** Multi-month research project. Comparable to the entire MIG-021v3 cascade in size — possibly larger because corpus curation is human-expert-bound, not agent-automatable.

**Risk.** High by nature of being research. Calibration is hard; ground truth requires uṣūl scholars; the literature is small.

**Leverage.** Highest of all clusters by scholarly significance. Lowest by tractability in a single MIG.

**Recommendation.** **Defer Cluster C from MIG-022 scope entirely.** Frame it as MIG-023 or a separate "Constellation Warrant Research" workstream with its own Concept Paper. The §6.1 YAML metadata work in Cluster A gives us the *display* slot for warrant grades that scholars enter manually — no classifier needed for v1.

### 2.4 Cluster D — PJ-040 (UA partial-frontmatter)

**Scope.** Refactor `synthesis.rs::user_authority_short_circuit` (synthesis.rs lines 120-167) to short-circuit ONLY the axes UA actually voiced on. For unfilled axes, fall through to `vote_on_axis` (the normal weighted-vote path) using the OTHER catalogers' trails.

**Touch surface.** Single function in `synthesis.rs` + the existing test `user_authority_short_circuits` updated + new regression test `partial_frontmatter_synthesizes_unfilled_axis_normally`. No schema, no IPC, no UI changes.

**Standard pattern.** This is just refactoring. The "voiced opinion per-axis, abstain per-axis" pattern is already the cataloger contract (`ReasoningTrail.voiced_opinion: bool` plus per-axis assignments) — UA's short-circuit was the one place that violated it.

**Big invariant at risk.** The full-short-circuit case (both axes have frontmatter) must produce IDENTICAL output to today. Synthesis method label can become `"user_authority_partial_short_circuit"` for the new case while keeping `"user_authority_short_circuit"` for the full case — preserves the existing test + reliability data semantics.

**Effort.** Small (~half a day, single PR / single commit).

**Risk.** Low. Test coverage is good; surgical change.

**Leverage.** Medium. Fixes a behavior gap visible to users on partially-filled-frontmatter notes (the `الخط العربي` case) — currently their CONTENT TYPE section silently disappears, which is a real UX bug. Doesn't unlock anything else.

### 2.5 Cluster E — PJ-041/042/043 (engine-output i18n gaps)

**Scope.** Three structural i18n gaps that V3-§10 could not address because the strings don't go through `$t()`:

| PJ | Gap | Effort | Translation volume |
|---|---|---|---|
| PJ-041 | Cataloger reasoning prose hardcoded English in Rust (`build_reasoning` + `format!()` patterns in 6 cataloger files + synthesis.rs) | ~3-5 hrs structural + translations | ~90 templates × 14 locales ≈ ~1,260 |
| PJ-042 | `Confidence` enum in `cataloger.rs:37-47` serializes as `"high"/"medium"/"low"/"abstain"`; rendered raw in trail | ~30 min + 4 keys × 14 locales = 56 | ~60 |
| PJ-043 | Vertical (~225) + Horizontal (~30) taxonomy node labels have only `en` + `ar` fields in `VerticalNode`/`HorizontalNode` structs | ~3,300 translations | ~3,300 |

**Touch surface (varies by PJ):**
- **PJ-041:** Refactor each cataloger's `build_reasoning` to emit `(template_key, params)` tuples. Frontend looks up `cece.reasoning.${key}` and substitutes params via `$t()`. Reliability data + composite blob format unchanged (the structured tuple goes alongside the prose).
- **PJ-042:** Add 4 keys `cece.confidence.{high,medium,low,abstain}`. Frontend wraps with a `confidenceLabel(c)` helper falling back to raw enum on missing key.
- **PJ-043:** Three sub-options (PJ doc names them):
  - **(a)** Extend the Rust struct with 13 more `&'static str` fields. Type-checked + zero-runtime-cost; large hand-edit.
  - **(b)** Move taxonomy data to per-locale JSON (`src-tauri/data/vertical_taxonomy.{locale}.json`) loaded lazily.
  - **(c)** Move labels to frontend `cece.taxonomy.*` keys in `src/lib/i18n/{locale}.json`; Rust keeps only IDs.

**Standard pattern.** All three are vanilla i18n problems. **Battle-tested** approach: ICU MessageFormat for templated strings (PJ-041), enum-to-key mapping (PJ-042), data-as-localized-files (PJ-043 sub-option b). Constellation already uses inline `$t()` calls — extending that pattern to engine output is the natural fit.

**Big invariant at risk.**
1. **PJ-041 — Reliability data continuity.** The `composite_json` blob is read by `cece_record_correction_for_card` and the calibration view. Refactoring the reasoning emission must preserve every other field of the trail. Adding a parallel structured `reasoning_template + params` field alongside the existing `reasoning: String` is safer than replacing.
2. **PJ-043 — taxonomy validation.** If labels move to JSON (sub-option b or c), the Rust side must validate at load time that every node has either every-locale labels or a fallback rule. Today's `&'static str` gives us compile-time guarantees we shouldn't lose silently.
3. **All three — performance.** No `invoke()` added to the keystroke hot path. Translations resolved at render time, cached per-locale.

**Effort.** Mid (PJ-041) + small (PJ-042) + mid-to-large (PJ-043) ≈ 1-2 weeks for the full cluster.

**Risk.** Low to medium. PJ-041 has the most touch surface (six cataloger files); PJ-043 sub-options (a)/(b)/(c) need the choice locked before implementation; PJ-042 is half a day.

**Leverage.** Medium-high. Honest representation of CECE in non-en/non-ar locales — the engine output today is a mixed-language stew for any non-Arabic non-English user. This is the floor → ceiling work the V3-§11 close-out flagged. Direct user-experience improvement.

### 2.6 Cluster F — Audit F1 (legacy classifier dead-code cleanup)

**Scope.** Reachability analysis + safe deletion of `classifier/tier1_embedding.rs`, `classifier/tier1_rules.rs`, large parts of `classifier/source_definitions.rs`. Also confirm `classifier/scan_job.rs` and `classifier/correction_log.rs` are still in use or absorb them too.

**Touch surface.** `src-tauri/src/classifier/` directory and any caller. The audit's drift agent flagged these as no-longer-reachable, but I peeked at `classifier/mod.rs` and saw `#[tauri::command] pub fn classifier_suggest_for_note(...)` still declared. **The reachability claim needs verification before deletion.**

**Standard pattern.** Standard dead-code elimination: `cargo build` + `cargo test` + `grep` every `pub` symbol for callers. Rust's compiler will catch broken imports; the test suite catches behavior changes.

**Big invariant at risk.** Test coverage. If a test depends on `classifier::*` it must be updated or removed deliberately, not left as a silent regression.

**Effort.** Small (~half a day).

**Risk.** Low if reachability is verified rigorously. Medium if rushed.

**Leverage.** Low (cleanup; no user-visible improvement) but **important hygiene** — leaving large dead modules in the tree means every future grep returns false positives, and any new contributor (or session) wastes time understanding code that has no callers.

---

## 3 · Cross-cutting invariants (apply to ALL clusters)

These are the things any MIG-022 work must NOT break, regardless of which clusters get chosen:

1. **CLAUDE.md "File over app"** — `.md` files on disk stay the source of truth. Schema growth does not introduce a parallel store of record.
2. **CLAUDE.md Performance Rule 1** — zero perceptible lag on keystroke; nothing in MIG-022 may add IPC calls or computation to the hot path.
3. **CLAUDE.md Performance Rule 8 (write-time derivation)** — every new derived view (history table, warrant index, etc.) is maintained at write time via triggers / hooks, not recomputed on read.
4. **CLAUDE.md Editor Parity Rule** — any new metadata UI must work identically in NotePane and any other editor view.
5. **CLAUDE.md Living Link Architecture** — `supersedes` / `contradicts` (gap analysis §6.1) overlap with the existing typed-link names. Decision D-A1 below resolves the overlap.
6. **CECE 6-cataloger contract** — adding warrant classifiers (Cluster C) does NOT restructure the existing ensemble; warrant becomes a parallel ensemble.
7. **i18n parity** — anything new added in Cluster A or beyond ships with en + ar at minimum; 13-locale backfill follows V3-§10's translation honesty pattern (disclaimer headers).
8. **Reliability data continuity** — Cluster E (PJ-041) refactor preserves every existing `composite_json` field; existing `cataloger_reliability.json` files keep working.
9. **Migration from CECE v1 universes** — every Cluster lands additively. Existing notes work unchanged; new fields default to absent / inert.
10. **No regression on Boot time / typing latency** measured against a 7,600+ note Universe (Eisa's primary library is the reference benchmark).

---

## 4 · Decision points for Eisa

These are the questions Phase 2 Plan needs answered before it can lay out steps. Numbered so we can refer back to them.

### D-A1 — Living Links overlap with §6.1 `supersedes` / `contradicts`

The gap analysis's §6.1 schema includes flat YAML fields:
```yaml
supersedes: "note-id-447"
contradicts: ["note-id-921"]
```

But Constellation's Living Link Architecture already has `contradicts` as one of the 7 typed-link names (the `[[contradicts: target|annotation]]` syntax) plus `supports`, `causes`, `exemplifies`, `generalizes`, `derives-from`, `part-of`, `associative`. Adding `supersedes` as a typed link would make 8.

**Two options:**
- **(D-A1.alpha)** Treat `supersedes` and `contradicts` as **YAML scalar fields** per the gap analysis's literal wording. Pro: matches the gap analysis exactly, simpler to surface in Properties panel. Con: parallel representation of relationships that the typed-link system already models.
- **(D-A1.beta)** Treat them as **typed-link names** (extending the existing 7 → 9, or 7 → 8 if `contradicts` is reused). Pro: graph-native consistency, gets backlinks / weight / lifecycle for free. Con: small departure from the gap analysis's literal schema.

**Recommendation: D-A1.beta.** The Living Link architecture is the right home for relations between notes; flat YAML is the right home for properties of a single note. (The other §6.1 fields — `held_by`, `warrant`, `domain`, `function`, etc. — are properties; only `supersedes` / `contradicts` are relations.)

### D-A2 — Should Cluster A include the warrant *display* surface, or defer all warrant UI to Cluster C?

The gap analysis §6.1 includes `warrant: "mutawātir"` as a flat scalar. Cluster C is the warrant classifier proper. But surfacing a manually-entered warrant value in the Source Review panel + Properties panel is doable in Cluster A without any classifier.

**Two options:**
- **(D-A2.alpha)** Include warrant display in Cluster A. Users can hand-enter `warrant:` in frontmatter; Source Review shows a pill; Properties panel has a dropdown. Forward-compatible with Cluster C: when the warrant classifier ships, it populates the same field.
- **(D-A2.beta)** Defer ALL warrant surface to Cluster C. Cluster A treats `warrant:` as inert.

**Recommendation: D-A2.alpha.** Display surface is cheap; it gives Eisa a usable feature today without waiting on the classifier; the classifier project just slots into the existing slot when ready.

### D-A3 — Schema versioning

If Cluster A lands and a user adopts the new fields, then Cluster C ships and changes the warrant vocabulary, the user's `warrant: "mutawātir"` should still parse. The gap analysis §7 names v1.0 / v1.1 / v1.2 / v2.0 explicitly.

**Question:** does MIG-022 ship a `taxonomy_version: "1.1"` frontmatter field that the parser uses to choose validation rules? Or does the field remain implicit (parser accepts unknown fields gracefully)?

**Recommendation:** **start implicit** (graceful unknown-field acceptance, which YAML parsers do by default). Add `taxonomy_version` only if/when a breaking change to the taxonomy actually requires it. Premature versioning is its own pathology.

### D-B1 — Temporal axis storage substrate

Three options (§2.2 above): Git-style commit log, bitemporal SQL, or event sourcing in an SQLite history table.

**Recommendation:** **SQLite `note_state_history` table** maintained by triggers on `note_meta` writes. Reasoning:
- Constellation already has an SQLite layer (`init_db`, the `note_meta` table, the FTS5 indexes). Adding one more table fits the existing shape.
- Write-time derivation (Rule 8) is naturally a SQL trigger.
- Queries are cheap SELECT.
- Git-style on-disk versioning means a parallel `.constellation/history/` tree that grows linearly with edits — file-system pressure on long-lived universes.
- Bitemporal SQL is overkill for "track changes to a few epistemic fields"; the user doesn't need transaction-time queries.

But this is a real choice with real trade-offs; D-B1 needs Eisa's lock-in.

### D-B2 — Temporal axis scope: which fields get history?

A spectrum from "every frontmatter field" to "only the epistemic fields".

**Recommendation:** **only the epistemic fields** — `source`, `content_type`, plus the §6.1 epistemic additions (`held_by`, `warrant`, `confidence_score`, `function`). Title changes and tag adds don't need to feed the temporal-epistemic-axis story; they're better tracked in Git itself if needed.

### D-B3 — Cluster B in MIG-022 or deferred?

Cluster B (temporal axis) is a 2-4 week project. MIG-022 might or might not include it.

**Recommendation:** **defer to MIG-023** as a focused workstream. MIG-022 lands Clusters A + D + E + F (the YAML metadata + UA fix + i18n gaps + dead-code cleanup), which is itself ~3 weeks of work. Temporal axis is a sibling MIG with its own Architect doc (it deserves the depth).

### D-C1 — Cluster C scope and timing

This is the warrant classifier — multi-month research project per §2.3.

**Recommendation:** **defer entirely from MIG-022**, as the gap analysis §7 itself recommends. Frame as **"Constellation Warrant Research"** workstream with its own Concept Paper (parallel to the Sight v3 Concept Paper). Land Cluster A's warrant *display* in MIG-022 so the slot is ready when the classifier ships.

### D-E1 — PJ-043 sub-option choice

(a) struct extension / (b) per-locale JSON / (c) frontend i18n keys.

**Recommendation:** **(c) frontend i18n keys** — `src/lib/i18n/{locale}.json::cece.taxonomy.{nodeId}`. Reasoning:
- Matches existing i18n pattern; one place to update labels.
- Translation team (whether human or LLM-assisted via the V3-§10.D pattern) already knows this surface.
- Rust side stays minimal — only IDs + parent_id + branch needed; English label can stay as a fallback in the struct.
- 14 locales × ~255 nodes = ~3,570 translations. Significant but parallelizable via the V3-§10.D agent pattern.

But (c) loses Rust-side validation — Rust no longer guarantees "every node has a label". Mitigation: ship a `cargo test` that loads the en.json, checks every taxonomy ID has a `cece.taxonomy.${id}` key. Same test for the other locales (looser — warn on missing instead of fail).

### D-E2 — Cluster E bundling

Three PJs that share a theme. Should they be one MIG or three?

**Recommendation:** **one MIG (MIG-022 §E)** with three sub-phases (E.1 = PJ-042 first because smallest; E.2 = PJ-041 second; E.3 = PJ-043 last because largest by translation volume). Single Architect / Plan / Audit cycle covers all three; phases land as separate commits. Saves ~2 days of overhead vs. three separate MIG cycles.

### D-F1 — Cluster F (legacy classifier) scoping

**Recommendation:** **fold into MIG-022 §0 housekeeping** as the first cascade phase. It's pre-work that should happen before any other touching of `classifier/` or `cece/`. Reachability verification is part of the Plan; deletion follows verification; tests guard the change.

---

## 5 · Recommended MIG-022 scope (the synthesis)

Based on the above decisions:

| Phase | What | Effort | Risk |
|---|---|---|---|
| **§0 — Cleanup** | Cluster F (legacy classifier dead code) | ½ day | Low |
| **§A — YAML metadata extensions (Cluster A)** | All §6.1 fields + warrant display + ikhtilāf rendering | 3-5 days | Low-medium |
| **§D — UA partial-frontmatter fix (Cluster D, PJ-040)** | Single `synthesis.rs` refactor + tests | ½ day | Low |
| **§E — Engine-output i18n (Cluster E)** | Three sub-phases E.1 (PJ-042) → E.2 (PJ-041) → E.3 (PJ-043) | 1-2 weeks | Low-medium |

**Total MIG-022 effort:** ~3 weeks of agent time. Two Boss-test gates (one after §A lands, one after §E lands).

**Explicitly deferred from MIG-022:**
- **Cluster B (temporal axis)** → MIG-023, separate Architect cycle.
- **Cluster C (warrant classifier)** → Constellation Warrant Research workstream, separate Concept Paper.

---

## 6 · Summary of decisions Eisa needs to make

To unblock Phase 2 Plan, I need explicit answers to:

1. **D-A1** — `supersedes`/`contradicts` as YAML scalars (alpha) or typed-link names (beta)? **Recommend beta.**
2. **D-A2** — Warrant display in Cluster A (alpha) or all warrant UI deferred (beta)? **Recommend alpha.**
3. **D-A3** — Ship `taxonomy_version` field now, or stay implicit? **Recommend implicit.**
4. **D-B3** — Cluster B (temporal axis) in MIG-022, or split to MIG-023? **Recommend split — MIG-023 with its own Architect.**
5. **D-C1** — Cluster C (warrant classifier) in MIG-022, or workstream-deferred? **Recommend workstream-deferred — separate Concept Paper.**
6. **D-E1** — PJ-043 storage: (a) struct fields / (b) per-locale JSON / (c) frontend i18n keys? **Recommend (c).**
7. **D-E2** — Cluster E as one bundled MIG or three separate? **Recommend one MIG with three sub-phases.**
8. **D-F1** — Cluster F as MIG-022 §0 or its own mini-MIG? **Recommend §0.**

If Eisa accepts every recommendation: **MIG-022 = §0 Cleanup + §A YAML metadata + §D UA fix + §E i18n** (~3 weeks, 2 Boss-test gates). MIG-023 (temporal axis) opens its own Architect after MIG-022 closes.

If Eisa wants a smaller MIG-022 (e.g., just §0 + §D + §E without §A): also valid. The §A YAML work is the largest piece and the most user-visible; it's reasonable to want to ratify the schema separately.

If Eisa wants a bigger MIG-022 (e.g., add Cluster B): I'd push back. Cluster B's storage choice (D-B1) deserves its own Architect cycle; cramming it in shortcuts the cross-check that Working Agreement #5 calls for.

---

## 7 · Composes with / depends on

- **MIG-021v3 (CECE)** — closed `407d79b`. MIG-022 §A extends the frontmatter schema CECE reads; §D fixes a CECE synthesis behavior; §E translates CECE engine output.
- **PJ-005 (Links Settings tab, MIG-007)** — MIG-022 §A (D-A1.beta) adds typed-link names; the Links Settings tab UI may need to surface them. Composes naturally.
- **Living Link Architecture** — MIG-022 §A's `supersedes`/`contradicts` (per D-A1.beta) become typed-link names. Living Links P2-P5 (CE Phase 2 onward) is independent work but shares the typed-link surface.
- **Sight v3 (PJ-038, MIG-019)** — MIG-023 (temporal axis, deferred from MIG-022) would feed Sight a timeline overlay. Not blocking either direction.

---

## 8 · What the Plan phase needs to verify

Before Phase 2 Plan can lay out steps, the following Architect-phase claims need code-level verification:

1. **F1 reachability** — is `classifier::*` actually unreachable? `classifier/mod.rs` declares `#[tauri::command] pub fn classifier_suggest_for_note(...)`. Plan must reach-analyze every `pub` symbol in `classifier/` for callers in the rest of the codebase. If any caller exists, the deletion scope shrinks.
2. **`build_reasoning` in 6 catalogers** — I confirmed structural.rs:366 + linguistic.rs:463. The other four (semantic.rs, graph.rs, user_authority.rs, reasoning.rs) are stated by PJ-041 but not yet directly verified.
3. **Frontmatter parser path** — I named "`src-tauri/src/notes/frontmatter.rs` or wherever YAML is parsed" but did not verify the path. Plan finds the actual file.
4. **Properties panel path** — Cluster A surfaces new fields in the Properties panel; the Plan finds the actual Svelte component path.
5. **Schema migration testing on Eisa's primary universe** — every Cluster A field addition needs a "does the existing 7,600-note universe still load fast" check.

---

## 9 · The honest framing

This Architect doc maps the territory faithfully; it does NOT pre-decide MIG-022's scope. The recommendations in §6 are **defensible defaults** based on the gap-analysis's own ranking + Constellation's existing constraints, but every one of them is a decision Eisa retains.

The work that ships with MIG-022 will be the work Eisa picks. The work that's deferred to MIG-023 (temporal) and the Warrant Research workstream is work that deserves its own depth — not work being avoided.

Once Eisa locks the eight decisions in §6, Phase 2 Plan lays out commits + verification clauses + Boss-test stages. Phase 3 Build cascades through the Plan. Phase 4 Audit runs the three-agent integration check the way V3-§11 just did.

---

*Filed at MIG-022 Phase 1 (Architect). Awaiting Eisa's decisions on §6 to proceed to Phase 2 (Plan).*
