# MIG-022 §N — Final Integration Audit

**Date:** 2026-05-12
**Phase:** 4 of 4 (/migration discipline) — final integration audit.
**Status:** Audit complete. MIG-022 ships conditional on one P1 finding (trigger coverage gap) — Eisa's call below.

This report consolidates three parallel agent audits run during MIG-024 Architect drafting:

- **Invariants** (agent 1) — `lab/reports/MIG-022-§N-AUDIT-INVARIANTS.md`
- **Drift** (agent 2) — `lab/reports/MIG-022-§N-AUDIT-DRIFT.md`
- **Migration-path** (agent 3) — `lab/reports/MIG-022-§N-AUDIT-MIGRATION-PATH.md`

---

## §1 · Headline verdict

| Audit | Result | Read |
|---|---|---|
| **Invariants** | **10 HOLDS · 0 AT RISK · 0 VIOLATED** | Architecturally clean. Every cross-cutting invariant from MIG-022 Architect §3 verified against the shipped cascade. |
| **Drift** | **0 P0 · 1 P1 · 4 P2 · 3 P3** | "Exceptionally clean for its scope (~7,400 i18n insertions + new history subsystem). No fabricated guards, no orphaned schema, no broken IPC registrations." |
| **Migration-path** | **3 PASS · 1 FAIL · 3 UNVERIFIED** | One real FAIL on trigger coverage (adjacent finding, not in the original 7 scenarios but discovered while tracing the trigger fire-path). |

**The single P1 action item** is the trigger-coverage FAIL — discussed in §3 below.

---

## §2 · What MIG-022 shipped (recap)

5 work clusters across ~30 commits:

| Cluster | Status | Commits |
|---|---|---|
| §0 — Legacy classifier cleanup | ✓ shipped | `d626ae7` (-988 LoC) |
| §D — PJ-040 partial UA short-circuit | ✓ shipped | `c072700` (+2 tests) |
| §E — Engine-output i18n cascade (PJ-041/042/043/045) | ✓ shipped | 7 commits, ~4,775 translations across 15 locales |
| §A — YAML metadata + supersedes typed-link + ikhtilāf widget + 15-locale docs | ✓ shipped | 8 commits |
| §B.1–§B.4 — `note_state_history` Rust foundation | ✓ shipped | 4 commits (+11 history tests) |
| §B.5 — UI overlay on Sight v3 | **CONTRADICTED-AND-DEFERRED** | Sight v3 retired by Concept Paper v3.1; not part of v5 |
| §B.6 — i18n + help + UM for the §B.5 UI | Deferred (only meaningful if §B.5 ships in some form) | — |

Boss-Test Gates 1 + 2 + 3 all PASS. 5 PJs closed (PJ-040/041/042/043/045); 7 new PJs filed for §N polish backlog (PJ-044/046/047/048/049/050) + 1 housekeeping (PJ-051 — Mock B1 SVG follow-up edits).

---

## §3 · The one P1 — trigger doesn't catch direct YAML edits

### §3.1 The finding

The `note_state_history_au` trigger (shipped in `c63a2e3`/`5c4f1e5`) fires on `UPDATE note_meta`. But the canonical note-save path — `index_note` in `src-tauri/src/search.rs:3045-3054` — uses **`DELETE FROM note_meta` + `INSERT INTO note_meta`**. SQLite triggers do NOT fire on DELETE+INSERT.

**Consequence:** the trigger captures only the two explicit CECE classifier writes (`src-tauri/src/sources/mod.rs:338` and `:926`). When a user edits frontmatter via NotePane — changing `held_by`, adding an `ikhtilāf` row, retyping a stratum — the history table receives nothing.

### §3.2 Practical impact

**Today:** zero. No consumer reads `note_state_history` yet. The original consumer (§B.5's Sight v3 overlay) is contradicted-and-deferred.

**When MIG-025 (Layer 2 diagnostic) lands:** the "growth trajectory" health signal Sight v5 wants to compute over `note_state_history` would silently see only classifier writes, missing all human edits. Discovering the gap then would be late.

### §3.3 Three remediation options

| Option | What | Code cost | Touches | When |
|---|---|---|---|---|
| **(α) UPSERT** | Replace DELETE+INSERT with `INSERT ... ON CONFLICT(path) DO UPDATE SET ...` in `index_note`. Trigger fires correctly. | Small Rust diff; perf re-validation needed on 7,636-note save | Canonical write path | Now |
| **(β) Rust-side explicit diff** | Have `index_note` read old row, compute diff against new values, emit history events explicitly via a new helper. Preserves DELETE+INSERT pattern. | Medium Rust diff (new helper + diff logic); doesn't touch the trigger | New code only | Now or deferred |
| **(γ) Scope-clarification** | Document that the trigger captures classifier writes only; add a clarifying comment in `history.rs`. Defer comprehensive history to a future MIG when actually needed by a Layer 2 consumer. | Zero code | Docs only | Now (just docs) |

**Recommendation: (β) Rust-side explicit diff.** Reasons:

- Preserves the §B contract ("every epistemic-field state change is captured") — α also does this; γ rescopes it.
- Doesn't touch the canonical write path — α touches the hot path used by every note save.
- Well-bounded: new helper function in `search.rs` or `history.rs`; one diff-emission call inserted into `index_note`; trigger stays as-is for the CECE classifier writes (defense in depth).
- The diff computation is cheap (read 6-8 column values, compare, emit 0-N events) and amortizes against the existing DELETE+INSERT cost.

**Alternative reading**: if Eisa wants this fixed inside the MIG-024 cascade rather than as a separate mini-MIG, β slots cleanly as a §0 housekeeping cluster ahead of the Sight v5 visual work — same cluster shape as MIG-022 §0 (cleanup) was.

---

## §4 · Other findings (none block MIG-022 ships)

### §4.1 Drift findings

- **F1 (P1)** — §B.4 IPCs (`cece_get_note_history`, `cece_query_history`) registered in `lib.rs` but have **zero frontend consumers**. This is **deferred-by-design** (§B.5 was the planned consumer; §B.5 is contradicted). Acknowledged in commit message and Pending Jobs v1.11. Resolves when Layer 2 (MIG-025) reads the history table.
- **F2 (P2)** — Dead-code shim `_suppress_unused(_a: AxisAssignment)` at `synthesis.rs:455-458`; `AxisAssignment` is actually used by the test module. Cleanup candidate.
- **F3-F4 (P2)** — Magic numbers (`0.80` secondary cutoff, `1.0/0.7/0.4/0.0` confidence multipliers) without `const` extraction or calibration provenance comments.
- **F5 (P2)** — `compose_reasoning` builds English fallback alongside i18n template; no test enforces they stay in sync.
- **F6 (P3)** — Stale comment in `vertical_taxonomy.rs:18-21` references retired `classifier::source_definitions`. Historical note, accurate.
- **F7 (P3)** — Trigger SQL duplicated verbatim at `history.rs:107-133` and `:238-264`. Extract to `const TRIGGER_SQL`.
- **F8 (P3) — pre-existing drift, NOT MIG-022** — 137 i18n keys missing from 13 non-en/non-ar locales. Pre-existing from MIG-021/021v2; MIG-022 §A.4 + §E.2 + §E.3 cleanly hit all 15 locales. Flag for a future i18n-completeness sweep.

### §4.2 Migration-path UNVERIFIED items (need empirical test, not code fix)

- **Scenario 6 (i18n)** — locale files present and parse-valid; RTL rendering of new Properties panel needs visual check; key-completeness across 13 backfilled locales beyond Spanish/German (the Gate 3 sample) needs locale-by-locale visual spot-check.
- **Scenario 7 (large-universe perf)** — single bulk `INSERT ... SELECT` + covering index; code is right; no wall-clock measurement on Eisa's 7,636-note universe yet.

Neither blocks ship. Both are empirical test items the next Boss-test session can spot-check.

### §4.3 Verified clean

- Zero `tier1_*`/`tier2_*` references in production code (§0 cleanup is complete).
- Zero TODO/FIXME/XXX/HACK in any audited file.
- All §B init_db migrations in correct order (table → trigger → backfill).
- All §B.4 IPC registrations present in `lib.rs:345-347`.
- All `pe-ikhtilaf-*` CSS classes referenced by templates.
- `supersedes` typed-link added consistently across `livePreview.ts` + `store.ts`.
- All 9 invariants from MIG-022 Architect §3 hold (10/10 including the boot/typing one).

---

## §5 · MIG-022 §N close-out verdict

**MIG-022 ships, conditional on one decision** — how to handle the P1 trigger-coverage gap.

| Decision | Options | Recommendation |
|---|---|---|
| **D-N1 — trigger-coverage P1** | (α) UPSERT in `index_note` · (β) Rust-side explicit diff · (γ) Scope-clarify and defer | **β** — preserves the §B contract, doesn't touch the hot path, well-bounded, slots cleanly as MIG-024 §0 housekeeping if you want it before Layer 2 |
| **D-N2 — timing** | (a) Fix inside MIG-024 §0 (one cluster before visual-foundation work) · (b) Stand-alone mini-MIG before MIG-024 · (c) Defer to MIG-025 (Layer 2 will discover when it tries to read history) | **a** — co-located with the next MIG; same agent context; same /migration discipline already in flight |

Other findings (F2 / F3 / F4 / F5 / F6 / F7 / F8) are P2/P3 polish — candidates for a future cleanup MIG, but not gating MIG-022.

---

## §6 · What ships when MIG-022 closes

When you lock D-N1 + D-N2:

- **MIG-022 status →** Done (with the chosen D-N1 remediation either folded inline or queued).
- **Pending Jobs →** v1.12 bumps to note MIG-022 §N closed; F1 P1 logged as resolves-with-MIG-025; F2-F8 logged as polish backlog (one PJ each or batched as PJ-052).
- **Orientation →** v2.01 (minor bump — same release context as v2.00) marking MIG-022 §N close-out.
- **MIG-024 Architect →** unchanged; the 7 D-V decisions still wait on your lock; D-N1 just adds an optional §0 housekeeping cluster if you pick D-N2.a.

---

## §7 · Three full agent reports

- **Invariants** — `lab/reports/MIG-022-§N-AUDIT-INVARIANTS.md` (10 HOLDS / 0 AT RISK / 0 VIOLATED)
- **Drift** — `lab/reports/MIG-022-§N-AUDIT-DRIFT.md` (0 P0 / 1 P1 / 4 P2 / 3 P3)
- **Migration-path** — `lab/reports/MIG-022-§N-AUDIT-MIGRATION-PATH.md` (3 PASS / 1 FAIL / 3 UNVERIFIED)

---

**End of MIG-022 §N audit.** Awaiting Eisa's lock on D-N1 + D-N2 to officially close MIG-022.
