# MIG-014 §2F — Three-Agent Audit Report

**Date**: 2026-05-06
**Plan**: `lab/reports/MIG-014-NOTE-STAGE-PLAN-v4.md` §2F
**Architect doc**: `docs/Stages-Concept-Paper-v1.2.md`
**Status**: Audit complete. P0/P1 fixed in close-out commits. P2/P3 logged as memory follow-ups.

---

## Audit summary

| Audit | P0 | P1 | P2 | P3 | Result |
|---|---:|---:|---:|---:|---|
| Invariant | 0 | 0 | 0 | 0 | **PASS** — all 10 invariants hold |
| Drift | 3 | 2 | 1 | 0 | PASS WITH P0/P1 (close-out fixes) |
| Migration-path | 0 | 0 | 2 | 4 | PASS WITH P2/P3 (logged) |
| **Combined** | **3** | **2** | **3** | **4** | All P0/P1 fixed before close. |

All three agents ran in parallel against the primary `E:\مشاريع كلاود\Constellation` working directory.

---

## Invariant audit — clean

All 10 invariants verified as holding:

1. `LIVING_LINK_BASELINE.length === 6` — defined once at `src/lib/libraries/store.ts:1574` as a `ReadonlyArray<BaselineStage>` of 6 entries.
2. Promote/demote chain length = 6 — enforced in `nextStage` / `prevStage` via `LIVING_LINK_BASELINE.length - 1`.
3. No remaining references to dropped symbols (`customStages`, `CustomStage`, `addCustomStage`, `updateCustomStage`, `removeCustomStage`, `reorderCustomStages`, `isKnownStage`, two-arg `lookupStageEmoji`) — historical-record docs only.
4. M11 zero-diff: `git diff src-tauri/src/lexicon/` empty.
5. `stageLabel(stage, t)` single-signature.
6. `lookupStageEmoji(stage)` single-arg.
7. **Law 2.7** — single source of truth: NotePane uses `let currentStage = $derived(...)` (line 174). No file mirrors stage prop into local `$state`.
8. `UniverseMeta` has no `custom_stages` field; `lib.rs` has zero `*_custom_stage*` IPC handlers.
9. `BootBundle` has no `custom_stages` field.
10. IPC contract on stage commit path: `commitStage → updateValue → debouncedSave → setTimeout`. Zero synchronous IPC on the keystroke path.

---

## Drift audit — 3 P0, 2 P1, 1 P2 (all `[pre-existing]`)

### P0 — write paths still emitting dropped Zettelkasten values

All three were pre-existing — not regressions from MIG-014, but turned from "outdated" into "actively wrong" because they now write deprecated values that route through the legacy fallback display path.

| File:line | Before | After (Eisa's call) | Commit |
|---|---|---|---|
| `src/lib/components/FocusPane.svelte:246-247` | `🔗 Promote to Permanent` button writing `stage: permanent` | **Button removed entirely (Option B)**. `onexit` signature simplified from `(promote?: string) => void` → `() => void`. Caller in `+layout.svelte:5194` simplified. `focusPane.promote` i18n keys deleted. | (this commit) |
| `src/lib/components/ExpressionForge.svelte:137` | `stage: synthesis` | `stage: maturity` (composition is settled, depended-upon) | (this commit) |
| `src/lib/components/SenseMakingCanvas.svelte:266` | `stage: permanent` | `stage: growth` (canvas-promoted note is a defined-concept-in-progress) | (this commit) |

### P1 — read paths missing `spark` + `archived` typo

| File:line | Before | After |
|---|---|---|
| `src/lib/components/KnowledgeHealthDashboard.svelte:80-82` | Map of 5 keys: `birth, growth, maturity, dormancy, archived` (typo + missing `spark`) | All 6 lifecycle keys aligned with `LIVING_LINK_BASELINE`: `spark, birth, growth, maturity, dormancy, archival` |
| `src-tauri/src/search.rs:3741-3781` | Backend `lifecycle` aggregation buckets — same gap | Six buckets: spark (created < 7 days, untraversed), birth (≥ 7 days, untraversed), growth, maturity, dormancy, archival. DB enum stays `'archived'` for back-compat; bucket key uses `archival` to match the lifecycle name. |

### P2 — i18n backfill (queued via PJ-014)

The `notePane.stage.spark/birth/growth/maturity/dormancy/archival` and `inspector360.stage_*` keys are only populated in `en.json` + `ar.json`. The other 13 locales fall through to raw lowercase values. PJ-014 already covers this.

---

## Migration-path audit — clean for P0/P1, 2 P2 + 4 P3 logged

All 7 scenarios handled correctly:

1. **Fresh Universe** — `UniverseMeta` parses cleanly. Empty stage skips render.
2. **Pre-MIG-014 universe.json with leftover `custom_stages: [...]`** — Serde silently ignores unknown fields (no `deny_unknown_fields` anywhere). Field drops on next rewrite.
3. **Legacy Zettelkasten values** (`fleeting / literature / permanent / synthesis`) — render correctly on all 4 surfaces (PropertyEditor / NotePane / FileTree / Inspector360) via `LEGACY_ZETTELKASTEN_EMOJI`. Promote/demote arrows hidden because legacy values aren't in `LIVING_LINK_BASELINE`.
4. **Malformed stage values** — empty / trailing-dash / leading-dash / multi-dash / uppercase all handled gracefully (no crash). See P2/P3 below.
5. **Non-ASCII suffix** (`stage: spark-مفهوم`) — `splitStage` is Unicode-safe via JS `indexOf`; emoji + label render correctly.
6. **Mid-promotion interrupt** — disk file untouched until `writeNote` IPC completes. No persistent in-memory ghost state. Reopen reads disk.
7. **Rollback** — pure deletion + helper additions in §2A → §2D. Cleanly undoable; legacy code reads dash-encoded notes as flat custom values.

### P2 / P3 follow-ups (memory-logged, non-blocking)

Six items saved as `project_mig014_audit_p2_p3_followups.md` in memory for next PJ-NNN allocation:

- P2-A: leading-dash value (`-concept`) renders awkward `-Concept` label.
- P2-B: Concept Paper §6.1 says suffix can't contain `-`; `commitStage` doesn't enforce. Doc-vs-code drift.
- P3-a: stale `custom_stages: [...]` on disk (silently ignored).
- P3-b: trailing-dash on disk (`spark-`) graceful self-healing.
- P3-c: uppercase on disk (`SPARK-CONCEPT`) renders verbatim until re-commit.
- P3-d: NotePane stage badge `<span>` has no `dir="auto"`.

---

## Verification after close-out fixes

- `npm run check`: 1 error (the pre-existing LinkLifecycle dedupe error at `store.ts:2324`; not MIG-014's).
- `cargo build --release --lib`: clean (22 warnings, all pre-existing baseline).
- `git diff src-tauri/src/lexicon/`: empty (M11 zero-diff intact).

---

## MIG-014 closes

§2A → §2F all shipped. PJ-007 (Note Stage Taxonomy) — **shipped (per-note dash-encoded model)**. The §1A → §1D commits stay in `main` as the iteration record.

Five top-principal feedback memories now in the system (MoCh, orientation-inline, etc.). Law 2.7 (single source of truth) added as a durable rule. Stages Concept Paper v1.2 + Plan v4 are the canonical references for the new model.
