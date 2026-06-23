# MIG-084 — The Rich Reviewer (Architect)

*Status: Architect (Phase 1). Plan approved by Eisa 2026-06-23. Builds on the MIG-080 §F split (note-tab `ReviewStatusPanel` + left-dock `ReviewerView`) and MIG-083 (the write-time `review_schedule` + Mode-2 staleness).*

## 1. Concept (the horse)

> **The Reviewer is the constellation's TEMPORAL TRIAGE instrument — the physician's call-back list.**

It is the only core surface that is **push, not pull**, and the only one keyed on **time**. It answers one question no sibling answers: *"Of everything in my universe, what must I revisit **now**, and **why**?"* — then shrinks the universe to the **actionable few**, each carrying a plain-language reason, and hands the user off to the deeper instruments for the "why is it shaped this way."

**Governing design law (Eisa, 2026-06-23):** *"Just by looking at it, the user can UNDERSTAND and make the right decision."* Every element is self-explanatory — plain words, a full "why-now" sentence per row, named maturity (never a bare integer), actions that preview their consequence. If an element needs explaining, it is wrong. This sits with **Form-Aligns-To-Purpose** (rich = decision-relevant density, never filler) and the **Style-Setter full-center-zone** rule.

## 2. The six lenses

A note can appear once per lens (lenses are never merged into one score — the MIG-083 two-lens rule, generalized). Order = most-consequential first:

| Lens | Reason | Detection (all write-time / cheap — Rule 8, no FS walk) | Primary action |
|---|---|---|---|
| 🥀 **Stale** | a load-bearing dependency changed after last review | MIG-083 Mode-2 (`note_stale_status`); already in `DueNote` (`stale_trigger_name/type/changed_on`) | ✓ Reviewed |
| 🔄 **Due** | review interval expired | MIG-083 Mode-1/3 (`review_schedule.due_days`) | ✓ Reviewed |
| 🧠 **Checkpoints** | a mental-model note ("still hold this view?") | `reason='checkpoint'` (tags) | ✓ Reviewed |
| 🔗 **Orphan** | nothing links here yet — *connect me* | `note_meta.incoming_count == 0 AND word_count > 20` (reuse `inspector360.rs:186` `is_orphan` thresholds) | **🔗 Connect** (NEVER Dismiss) |
| ⚠ **Fragile** | many depend on it, little holds it up — *shore me up* | `incoming_count >= 5 AND derives_count <= 1` (reuse `inspector360.rs:188` `single_point_of_failure`); `derives_count` = count of active `derives-from` out-links in `note_links` for the small `incoming_count>=5` candidate set | ✓ Reviewed / Connect |
| 📝 **Never reviewed** | scheduled, never ✓-reviewed | MIG-083 (`last_reviewed IS NULL`) | ✓ Reviewed |

**Orphan is an ALARM, never disposable** (Eisa's correction). Its action is **Connect** (open the note positioned to add a link / locate in Sky). "Dismiss" survives only as *"this is an intentional hub, not an orphan"* — a false-positive exclusion, never a generic silence.

## 3. The prescription — complement, hand-off, never duplicate

The Reviewer **scans-and-queues**; the siblings **explain**. Cross-check outcome (workflow `wf_903c6389-5e7`):

| Plugin | Relation | Contract |
|---|---|---|
| **Knowledge Health** | receives-from / shared | KH owns the orphan **count** (whole-graph, `computeUniverseHealth`); the Reviewer owns the orphan **queue** (per-note, dated). Cross-linked, **single source** (`get_due_notes` / `review_schedule`) — no drifting numbers. |
| **360 Inspector** | hands-off-to | The deep per-note EXPLAIN. The Reviewer **reuses** its `is_orphan` / `single_point_of_failure` thresholds + exact vocabulary; **never** rebuilds the Stratification Matrix. Row → "See full context (360)". |
| **Cataloger / CeCe** | shared / hands-off-to | Each row's context line is CeCe's **NSC summary headline** (`getSummariesFor`, `src/lib/nsc/summaryStore.ts:64`) — reused, never regenerated. Reviewing a fresh note → "Classify". |
| **Living Links / CCS** | shared vocabulary | Staleness reads load-bearing-link semantics. **Invariant I2b:** opening a note from the Reviewer must NOT fire `constellation_link_traverse` (observe, don't feed circulation). |
| **Dashboard / Index / Sky** | receives-from / complementary | Dashboard due-count + Index summaries come from the same read; orphan rows may optionally "Locate in Sky View" (no embedded graph render). |
| **Global Tasks** | risk-of-duplication | Hard boundary: Global Tasks = explicit user TODOs; Reviewer = system-derived knowledge-maintenance prompts. Never blend. |

**Anti-duplication (law):** do NOT rebuild KH aggregates, the 360 matrix, the CCS link registers, CeCe summaries, Index term-discovery, the Sky graph render, Global-Tasks TODOs, or any FS walk (`scan_due_recursive` stays removed — MIG-083 §E).

## 4. Data inventory — what's cheap vs. new

**Already in the DB (no migration):**
- `DueNote` already carries: `note_path, note_name, reason, days_overdue, stratum, last_reviewed, stale_trigger_name/type/changed_on` (`review.rs`).
- `note_meta` already carries: `incoming_count`, `outgoing_count`, `word_count`, `created_at`, `modified` (`search.rs`). → orphan + fragile detection, connection counts, **and maturity** all derive from these.
- **Maturity needs NO denormalization migration.** The shared `maturity_sql` fragment (`search.rs:223-301`) already computes the vocabulary (seed/sapling/evergreen/canonical/wilting) from `note_meta` + inbound, with the time-arithmetic against `now` — read-time O(1), Rule-8-clean. The Reviewer **reuses that fragment** in its read (fresher than the trigger-maintained `sky_nodes.maturity` for the elapsed-day transitions). *(This collapses the originally-planned §E migration into a read-time reuse — strictly less risk, same outcome.)*
- Consequence previews + per-lens backlog counts = frontend-derived (interval ladder `review.rs`, the existing grouped arrays).

**The ONE schema change — Priority (§D):**
- `review_schedule.review_priority INTEGER NOT NULL DEFAULT 50` (0–100; user-set importance, separate from the schedule — SuperMemo's overload-by-ranking idea).
- Default 50 ⇒ **no back-fill computation** (existing rows inherit the column default). `upsert_schedule_row` must **preserve** it across review recompute (same pattern as `snoozed_until`).
- New command `set_review_priority(note_path, value)`; set from **both** the Reviewer detail-pane slider AND the note's Review tab (`ReviewStatusPanel`) — Eisa 2026-06-23.
- Surfaced as a `DueNote.priority` field; the queue can rank by it. Gets full /migration rigor (impact note + the §H audit) because it adds a column + a write path.

## 5. Layout — master-detail (Eisa-ruled)

A **two-column** full-page surface filling the center zone:
- **Master (left, ~360–460px):** the six-lens queue, collapsible section headers with per-lens counts, the existing order. Rich rows: name + reason glyph on line 1; the why-now sentence + maturity word + "N in · M out" + last-reviewed on a muted line 2. VirtualList fallback ≥80 rows per lens (Rule 3); **no "+N more" cap** (Boss).
- **Detail (right, fills the rest):** the selected note's full review context — the why-now sentence in plain language, the NSC headline, maturity, connection counts, last-reviewed — and the decision verbs (✓ Reviewed → next-in-Nd · 👁 Snooze 7d · 🔗 Connect for orphans · the priority slider) **plus hand-off buttons** (Open · See full context in 360 · Classify in Cataloger · Locate in Sky). It is a *triage card with exits*, NOT a mini-dashboard.

**Rejected as filler** (Form-Aligns-To-Purpose): body excerpts (Rule-8 FS read), heading lists, decorative charts/sparklines. The detail pane earns its width with the changed-dependency context, not whitespace.

## 6. Invariants that must not break
- **Rule 8 / Rule 3:** every read is a cheap indexed lookup; zero `.md` syscalls on the read path; VirtualList for any lens that can exceed 80 rows; `<100ms` `get_due_notes` on the 7,600-note universe.
- **Two-lens-never-merged:** six distinct collapsible lenses; a note appears once per lens.
- **CCS Invariant I2b:** opening a note from the Reviewer does NOT fire `constellation_link_traverse`.
- **Single source of truth:** Dashboard/KH counts read the same `get_due_notes`/`review_schedule` — no second count.
- **No boot/typing/IPC regression** measured before/after on the large universe.
- **Self-explanatory law** holds on every surface element.

## 7. Phase plan (each lands as one commit + a verification clause)

- **§A** Architect doc (this file) + WA#4 impact note. *(no code)*
- **§B** Cheap enrichment: `incoming_count`/`outgoing_count` + maturity (via `maturity_sql`) into `DueNote` + the Lens reads; stratum/maturity label frontend map. *Verify: `get_due_notes` returns the fields, `<100ms`, zero-FS; existing 21 tests green.*
- **§C** Orphan + Fragile lenses in the indexed read (new reasons from existing columns + the small derives count). *Verify: a known orphan + a known fragile note surface with the right reason; unit tests; `<100ms`.*
- **§D** Priority — the one schema change (defaulted column + `set_review_priority` + upsert-preserve + `DueNote.priority` + sort). *Verify: set priority persists + reorders; default needs no back-fill; impact note written.*
- **§E** *(collapsed — no migration)* maturity surfaced via reuse of `maturity_sql`; folded into §B. *Verify: maturity word shows; fresh across the day-transitions.*
- **§F** The master-detail rich surface (6 lenses, detail pane, hand-offs, Connect verb, NSC headline, consequence previews, designed empty states, `--rs-scale` text). *Verify: **Boss test** — rich, self-explanatory, hand-offs work.*
- **§G** Style Setter dedicated **Reviewer** category (text resize via the existing `--rs-text-scale-review` token + density + colours + show-connections toggle, live full-zone preview). *Verify: **Boss test** — live preview + persists across restart.*
- **§H** Audit (3 agents: invariants / drift / migration-path) + SO close-out (help + User Manual ×15, orientation v-bump, session log, MoCh).

## 8. Open decisions — RESOLVED
- Layout: **master-detail** (Eisa).
- Scope: **go big** — orphan + fragile both first-class lenses (Eisa).
- Priority set-point: **both** the Reviewer detail pane AND the note's Review tab (Eisa).
- Style Setter: **dedicated "Reviewer" category** (Eisa).
- Orphan action: **Connect**, never Dismiss (Eisa's ruling).
