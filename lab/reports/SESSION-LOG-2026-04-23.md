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

## § 68. MIG-001 Step 3 — note_links triggers

AFTER INSERT/UPDATE/DELETE on `note_links` maintaining `sky_links`.
/simplify v2 findings applied before commit: WHEN guard on AU for
metadata-only writes, weight 1.0 default, plain INSERT, dropped
non-selective link_type index.

**Commit**: `03c21b4`

## § 69. MIG-001 Step 4 — note_meta triggers

AFTER INSERT/UPDATE/DELETE on `note_meta` maintaining `sky_nodes`.
AU WHEN guard limits fire to structural changes (path/name/library);
AD cascades outgoing sky_links. Orphan preservation intrinsic.
link_count/outgoing_count deferred to Step 8 read-time computation.

**Commit**: `39cc387`

## § 70. BUG-001 fix — phantom-duplicate on rename

Found during Step 4 verification. Rename of canonical note left a
second file on disk with stale content. Root cause: Rust's in-place
frontmatter rewrite kept the file at old_path but frontend assumed
the move happened. Fix: `rename_item` returns effective path; Rust
is now authoritative.

**Commit**: `26ba6aa`

## § 71. BUG-002/003 fix — title display + alias idempotency

- BUG-002: NotePane titleValue stuck at mount; added `$effect` to
  sync when title prop changes externally.
- BUG-003: Stale titleValue triggered rename with old==new, which
  appended current title to its own aliases. Added equality guard
  in `rename_item`.

**Commit**: `68f24ea`

## Session Summary — 2026-04-23

| § | Commit | Change |
|---|--------|--------|
| §63 | 0229226 | Migration Rule encoded + /migration skill |
| §64 | 8dbf304 | MIG-001 Phase 1+2 plan (Sky View WTD) |
| §65 | 0e3e72c | MIG-001 Step 1: SKY_SCHEMA_VERSION infra |
| §66 | 81e7143 | MIG-001 Step 2: sky_* tables (empty) |
| §67 | 87fb8b8 | SV: default camera pitch for orbital rotation |
| §68 | 03c21b4 | MIG-001 Step 3: note_links triggers |
| §69 | 39cc387 | MIG-001 Step 4: note_meta triggers |
| §70 | 26ba6aa | BUG-001 fix: phantom-duplicate on rename |
| §71 | 68f24ea | BUG-002/003 fix: title sync + alias idempotency |
| §72 | cfc3382 | MIG-001 Step 5: resumable sky_* back-fill populator |
| §73 | 4c8a974 | MIG-001 Step 6: rename cascade hardening + docs |
| §74 | c52b3f2 | MIG-001 Step 7: enrichment scope decision (defer MIG-002) |
| §75 | ae062f6 | SV: legend toggle + dead-code cleanup |
| §76 | b99b856 | SV: Stratum + Maturity color modes with legend |
| §77 | 6be6ee9 | BUG-004 fix: legend hides when mode has no data |
| §78 | 2f9a4c5 | SV: Compute-now button for on-demand enrichment |
| §79 | 5b0c305 | MIG-001 Step 8: cache_boot_snapshot_sky IPC |
| §80 | e2434ba | MIG-001 Step 8 perf: drop SQL JOINs, aggregate in Rust |
| §81 | 60b03a8 | MIG-001 Step 8 perf round 2: reuse pre-lowercased id |
| §82 | 0bf99d8 | MIG-001 Step 9: frontend swap to cache_boot_snapshot_sky |
| §83 | cf59401 | MIG-001 Step 9 fix: parallelize sky IPC with graph IPC |
| §84 | eab6aa9 | MIG-001 Step 10: audit-only, no second-screen swap needed |

## § 72-78 — MIG-001 Steps 5-7 + Sky View legend redesign

See individual commits for full context. High notes:

- **Step 5 (§72)**: Resumable back-fill populator on a background
  thread. 7,600 sky_nodes + 232,461 sky_links populated on first
  boot. Cursor table allows resume on crash. Subsequent boots
  skip via schema_versions.sky stamp. ✅ Tested.
- **Step 6 (§73)**: Rename cascade end-to-end — the actual chain
  is AD+AI fires (not AU) because index_note does DELETE+INSERT.
  INSERT OR IGNORE safety for back-fill race. ✅ Tested.
- **Step 7 (§74)**: Audit found enrichment is compute-on-demand,
  not persisted. Decision: defer enrichment WTD to MIG-002 to
  keep scope disciplined. ✅ No regression.
- **§75 Legend toggle**: Palette button to hide/show legend drawer.
- **§76 Stratum + Maturity modes**: New color modes pulled from
  ConstellationMap palettes; enrichment already flows via
  existing compute_note_strata / compute_note_maturity IPCs.
- **§77 BUG-004 fix**: Legend disappeared entirely when mode had
  no data, trapping user. Legend now always renders when visible.
- **§78 Compute-now**: Empty state exposes a button to trigger
  enrichNodesBackground on demand. Persistence deferred to MIG-002.

## § 79-84 — MIG-001 Steps 8-10

- **Step 8 (§79-81)**: `cache_boot_snapshot_sky` IPC reads sky_nodes +
  sky_links directly from SQLite. Perf pass 1 dropped SQL JOINs in
  favor of Rust HashMap aggregation (7.7s → 2.8s debug). Pass 2
  reuses the pre-lowercased `id` column instead of re-lowercasing
  per edge. Byte-diff identical to legacy buildSkyData output.
- **Step 9 (§82-83)**: Frontend swap in `+layout.svelte`. First cut
  (§82) ran sky IPC serially after graph IPC — regressed boot. §83
  parallelized via Promise.all so skyPromise kicks off before the
  graph await. ✅ Tested, user approved boot times.
- **Step 10 (§84)**: Audit-only close. SecondScreenPage's three
  buildSkyData sites operate on per-library ego networks, not the
  full universe graph. Invariant 5 (second-screen parity) is about
  the shared note data contract, which is unchanged. No code change.

## § Pending — MIG-001 Phase-4 Audit (Step 11)

Phase-4 audit launched 3 agents in parallel per /migration skill.
All three returned findings; **nothing committed yet**.

Release build completed (task `b7afncku9`, exit 0). Boot-perf
numbers not yet captured from output file.

**HIGH fixes pending**:
1. `+layout.svelte:2566` — graph-failure path skips sky assignment
   even when sky IPC succeeded. Guard needs `|| (sky && sky.isReady)`.
2. `cache_boot_snapshot_sky` — no gate on back-fill completion;
   users can see partial graph mid-migration. Gate on
   `schema_versions.sky = TARGET`; `isReady=false` → frontend falls
   back to buildSkyData, not partial render.

**MED/LOW fixes pending**:
3. Dedupe hand-synced SKY_SCHEMA_VERSION between `search.rs` and
   `sky_backfill.rs:41`.
4. Wrap `sky_backfill.rs` `finalize()` in one transaction.
5. Relabel "Compute now" button → "Compute for this session" (15
   i18n files).

Next session: apply HIGH #1 + #2 first commit, MED/LOW second
commit, capture boot-perf from release build output, write Phase-4
section in `MIG-001-SKYVIEW-WTD.md`, final MIG-001 closure commit.

### Non-MIG pending

- Functional Panel Placement test walkthrough
- MIG-002: persist enrichment (stratum/maturity/origin_type) to sky_nodes
