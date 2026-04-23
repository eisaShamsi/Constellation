# Session Log — 2026-04-23

## § 63. Migration Rule + /migration skill

Encoded a principle for any change that crosses subsystem boundaries
(schema, data flow, cross-surface invariants, multi-file refactors).
Four-phase workflow: **Architect → Plan → Build → Audit**. Phase 4 is
three parallel agents (invariants, drift, migration path). Cost: ~30
min of agent time. Gain: prevents the regression-undo loop that costs
whole iterations.

- New skill: `.claude/skills/migration.md`
- New hard rule in `CLAUDE.md`: "The Migration Rule (major changes)"
- `/simplify` remains the skill for single-file refactors

**Commit**: `0229226`

## § 64. MIG-001 Phase 1+2: Sky View WTD plan

First application of the Migration Rule.

- **Phase 1** (Architect): 8 invariants (orphan preservation, dedup
  by source→target:type, name-based id, archived exclusion, second-
  screen parity, rename cascade, enrichment fields, typing latency),
  8 risks with mitigations, option comparison (A/B/C). User picked
  **Option C** (full WTD with dedicated tables + triggers).
- **Phase 2** (Plan): 11 steps with verification clauses, `/simplify`
  runs at steps 3/6/9, Phase-4 audit at step 11.

Plan doc: `lab/reports/MIG-001-SKYVIEW-WTD.md`

**Commit**: `8dbf304`

## § 65. MIG-001 Step 1 — SKY_SCHEMA_VERSION + schema gate

Version infrastructure only — no tables yet.

- `SKY_SCHEMA_VERSION = 0` constant with ledger comment
- New `schema_versions(module, version, updated_at)` generic table
- Init-time read + diag log line reporting current vs. target version

Decoupled from FTS `PRAGMA user_version` so each WTD surface can
migrate on its own cadence.

**Status**: ✅ Tested — app boots unchanged.
**Commit**: `0e3e72c`

## § 66. MIG-001 Step 2 — sky_nodes + sky_links tables (empty)

Schema only. Triggers land in Step 3+4.

`sky_nodes`:
- `path` PK (matches `note_meta`), `id` (lower-cased name), `name`,
  `library_name`
- `link_count`, `outgoing_count` (aggregates)
- `stratum`, `maturity`, `origin_type` (enrichment, populated in Step 7)
- `created_at`, `updated_at`
- Indexes: `library_name`, `id`

`sky_links`:
- `source_path`, `target_name`, `link_type` (name-based to match
  current SkyLink shape)
- `weight`, `count`
- `UNIQUE(source_path, target_name, link_type)` — dedup invariant
- Indexes: `source_path`, `target_name`, `link_type`

No foreign keys (back-fill order allows links before nodes; triggers
maintain integrity).

**Status**: ✅ Tested — app boots unchanged, tables empty, JS path
still drives UI.
**Commit**: `81e7143`

## § 67. Sky View: default camera pitch (orbital rotation)

User flagged that auto-rotation looked like a flat fan, not an
orbital spin around a vertical axis. Root cause: `camRotX = 0` meant
head-on view, so Y-rotation just shifted nodes left/right with no
depth arc.

Fix: `DEFAULT_PITCH = -18°` as the resting camera tilt. The "floor"
of the universe now tilts slightly away, so Y-axis rotation becomes
visible orbital motion — the imaginary vertical axis through the
center becomes the rotation axis. Front nodes swing toward viewer,
back nodes recede.

- `resetTilt()` returns to `DEFAULT_PITCH` instead of zero
- `isRotated()` compares against `DEFAULT_PITCH`, so axis gizmo only
  appears once the user has rotated beyond the orbital baseline
- Value tunable via the constant

**Status**: ✅ User-approved.
**Commit**: `87fb8b8`

## Session Summary — 2026-04-23

| § | Commit | Change |
|---|--------|--------|
| §63 | 0229226 | Migration Rule encoded + /migration skill |
| §64 | 8dbf304 | MIG-001 Phase 1+2 plan (Sky View WTD) |
| §65 | 0e3e72c | MIG-001 Step 1: SKY_SCHEMA_VERSION infra |
| §66 | 81e7143 | MIG-001 Step 2: sky_* tables (empty) |
| §67 | 87fb8b8 | SV: default camera pitch for orbital rotation |

### Still pending (MIG-001)

- Step 3: triggers on `note_links` → `sky_links` (next, with /simplify)
- Step 4: triggers on `note_meta` → `sky_nodes`
- Step 5: resumable back-fill populator
- Step 6: rename cascade (with /simplify)
- Step 7: strata enrichment triggers
- Step 8: new IPC `cache_boot_snapshot_sky`
- Step 9: frontend swap main window (with /simplify)
- Step 10: frontend swap second-screen
- Step 11: Phase-4 audit + cleanup

### Non-MIG pending

- Functional Panel Placement test walkthrough
