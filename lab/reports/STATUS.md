# Constellation — Status Index

**Purpose.** A single one-page pointer to where status lives for every Constellation subsystem. This file is an INDEX, not a tracker — it tells you which doc owns the authoritative status for each subsystem, with a one-line current-state summary so you don't have to open each doc just to see if anything needs attention.

**Last updated**: 2026-04-26 (commit reference: see SESSION-LOG-YYYY-MM-DD.md for the latest § entry).

**Maintenance rule** (Standing Order #5): when a subsystem's state changes meaningfully, update its row here in the same commit that lands the change. If a row's "Authoritative tracker" link starts disagreeing with reality, the discrepancy belongs in that day's session log under §STATE-OF-STANDING.

---

## Engines / Subsystems

| Subsystem | Authoritative tracker | One-line current state |
|---|---|---|
| **Cognitive Engine** (16 phases) | `docs/cognitive-engine-roadmap.md` | Layer 1 Phases 1–11 ✅ shipped (Phase 4 still pending large-library test). Layer 3 Phases 12–16 🔲 not started. |
| **Living Link Architecture** (P0–P5) | `docs/cognitive-engine-roadmap.md` (Layer 2 block) | All five phases ✅ shipped + user-validated. Out-of-scope items (LINK files on disk, inline annotation editor, dashboard legend) deliberately deferred. |
| **Constellation Arabic Engine** (M-numbered milestones) | Spread across SESSION-LOG-2026-04-{14,17,18}.md | M3 / M3-baker / M5 / M6 / M7 / M8 / M8b / M8c / M9 / M10 / M11-infra / M12 / M12-detect / M12-bench / M13 / M14 — all ✅ shipped. M9 perf follow-ons (string-intern `pattern_label`, mmap FST bytes, trim per-call `Arc::clone`) 🔲 pending. |
| **M11-data Lexicon corpus** | `lab/m11-data/README.md` | v2 Producer ✅ complete (~20K concepts across 499 shards). Follow-ons (synonyms, domain packs) 🔲 deferred. |
| **Canonical Filename Architecture** | `docs/CANONICAL-FILENAME-ARCHITECTURE.md` | Implemented for `NOTE` kind. MIG-003 (human-name filenames overlay) 🔲 not started. |

## Migrations

| ID | Authoritative tracker | One-line current state |
|---|---|---|
| **MIG-001** Sky View Write-Time Derivation | `lab/reports/MIG-001-SKYVIEW-WTD.md` | ✅ Closed (all 4 phases). One trailing item: release-run boot-perf trace not yet collected. |
| **MIG-002** Enrichment Persistence | `lab/reports/MIG-002-ENRICHMENT-PERSISTENCE.md` | ⏳ §1–§6 shipped + tested. §7–§10 🔲 pending: `enrichment_worker.rs` drain loop, derives-from triggers, frontend swap, Phase-4 audit. |
| **MIG-003** Human-Name Filenames | (no plan doc yet — design in `docs/CANONICAL-FILENAME-ARCHITECTURE.md`) | 🔲 Not started. User-flagged readability pain (canonical stems shown in Explorer). |
| **MIG-004** Alias-Aware Resolution | `lab/reports/MIG-004-ALIAS-AWARE-RESOLUTION.md` | ✅ Closed (all 4 phases). Audit deferrals (4B-1, 4B-2) folded into MIG-005. |
| **MIG-005** Alias-aware in-memory inbound consumers | (no plan doc yet) | 🔲 Not started. Scope: `strata.rs` / `maturity.rs` / `tension.rs` / `inspector360.rs` / `map.rs` / `LinkDashboard.svelte`. Read-side only. |
| **MIG-006** Wikilink Rename Cascade | `lab/reports/MIG-006-WIKILINK-CASCADE.md` | §1 ✅ verified. §2 ✅ shipped (regex walker, 11 unit tests). §3 expanded shipped at `3c4732d` then **reverted at `5afe0c2`** (BUG-015). §3 redo + §4–§11 🔲 pending. The "rename target while source visible" UX gap (BUG-013) is documented; user must switch tabs first. |

## Boot performance

| Item | Authoritative tracker | One-line current state |
|---|---|---|
| Criterion 1 (paint ≤ 2.5 s) | `lab/reports/HANDOFF-2026-04-15.md` + `lab/boot-perf/BOOT-BUDGET.md` | ✅ Verified production (`5cb4f94`). |
| Criterion 2 (hydrated ≤ 6 s) | same | ✅ Verified production. |
| Criterion 4 (post-UI sync sweep) | HANDOFF | 🔲 Not started. |
| Criterion 5 (kill-mid-index recovery) | HANDOFF | 🔲 Not started. |
| Stats persistence (star counts + recent notes in SQLite) | HANDOFF | 🔲 Not started. |
| Settings → Rebuild Index button | HANDOFF | 🔲 Not started. |
| Settings → Debug Boot Performance scorecard UI | SESSION-LOG-2026-04-22.md | 🔲 Not started. |
| CHANGELOG entry + version bump to 0.4.0 | HANDOFF | 🔲 Not started (your decision). |
| Release-run boot-perf trace | MIG-001 closure | 🔲 Not collected. |

## UI / UX features

| Item | Authoritative tracker | One-line current state |
|---|---|---|
| Panel Placement Tier 1 + 1b | SESSION-LOG-2026-04-{21,22}.md | ✅ Shipped (slot picker + drag-resize). |
| Panel Placement Tier 2 (drag-and-drop rearrange) | SESSION-LOG-2026-04-22.md | 🔲 Not started. |
| Panel Placement Tier 3 (detachable floating panels, multi-window) | SESSION-LOG-2026-04-22.md | 🔲 Not started. |
| "Note as organism" editor redesign | SESSION-LOG-2026-04-21.md (§47 closing notes) | 🔲 Design-only, not started. |

## Write-Time Derivation audit (CLAUDE.md Rule 8)

| Surface | Status |
|---|---|
| Sky View | ✅ MIG-001 closed. |
| Backlinks / Outgoing panels | ✅ Living Link P5. |
| Tag browser | ✅ §50 (`scan_library_tags` eliminated). |
| Sight dashboard (ConstellationSight2) | 🔲 Pending — recomputes Louvain + health on each toggle. Partial cache landed §55. |
| Sidebar star counts | 🔲 Pending. |
| Map | 🔲 Pending. |

## Outstanding bugs / cosmetic

| ID | One-line current state |
|---|---|
| **BUG-013** open-editor cascade race | Re-opened by §116's revert. Documented limitation: switch tabs before renaming a target whose source is visible. |
| **BUG-014** orphan `cid_cn` | ✅ Closed §118 (2026-04-25). |
| **BUG-015** target-body corruption | ✅ Vector removed from `main` at §116 (`5afe0c2`). Forensic snapshots in `lab/forensics/`. |
| Title-heading rename gap | 🔲 `NoteEditor.handleTitleChange` does not call `updateLinksOnRename`; only file-tree rename triggers cascade. |
| "Auto-update links on rename" toggle in wrong section | 🔲 Should be Knowledge Management, not Sky View & Links. |
| Sidebar active-item highlight lag (~10 s after wikilink nav) | 🔲 Pending. |

## Orthogonal / housekeeping

| Item | Status |
|---|---|
| `__navTrace` instrumentation dev-gate | 🔲 Permanent code today; should be dev-gated or removed. |
| Isolated throttle stress-test helper (P2 follow-up) | 🔲 Pending. |
| RTL alignment verification on Arabic docx | 🔲 Pending verification. |
| `buildSkyData` JS fallback | Intentionally retained (load-bearing for `sky.isReady=false` and SecondScreenPage). Deletion = its own migration. |

---

## Where the working state of the current session lives

The **most current** state of any in-flight work is in the latest `lab/reports/SESSION-LOG-YYYY-MM-DD.md`. Look for:

- `§NNN — <description>` entries for individual commits.
- `§STATE-OF-STANDING` blocks (Standing Order #5) for snapshots taken at pivot points.

This index is the long-lived layer; session logs are the high-frequency layer. They should not contradict — if they do, the session log wins, and this file should be updated in the same commit that lands the change.
