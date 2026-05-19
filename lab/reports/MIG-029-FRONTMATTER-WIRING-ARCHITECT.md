# MIG-029 — Per-note Frontmatter Wiring for Tradition-Kind Fields

**Status**: Architect + Plan combined.
**Phase**: 1+2 of 4 (/migration discipline).
**Date**: 2026-05-19
**Predecessor**: 8 TODO comments scattered across tradition modules:

- `habermas.ts:85` — `habermas_interest`
- `korean-songnihak.ts:72` — `songnihak_cell`
- `masadir.ts:119` — `masadir_source`
- `mencian-sprouts.ts:94` — `mencian_sprout`
- `mohist-san-biao.ts:105` — `mohist_zone`
- `pardes.ts:79` — `pardes_level`
- `peirce.ts:83` — `peirce_category`
- `pramana.ts:109` — `pramana_kind`

Plus `ibn-rushd-burhan.ts` which has a similar pattern (currently uses hash bucketing not frontmatter) — `burhan_kind`.

Total: **9 tradition-kind frontmatter fields**.

---

## §1 — Goal

Replace the 9 hardcoded constant-return functions with reads from per-note frontmatter, so that switching to e.g. `masādir` reshapes the dome into 4 sectors AND each note lands in its actually-classified sector (not all defaulting to `quran`).

**User-visible outcome**: a user can put `masadir_source: sunnah` in a note's YAML frontmatter, switch the chip to `masādir`, and that note lands in the sunnah quadrant.

Without this MIG, the 4 sectoral / 3 ladder / 5 sectoral / 4 ladder / etc. shape renderers correctly draw geometry but every star defaults to the first bucket — the chip is visually true but analytically inert for these 9 traditions.

## §2 — Territory map

### §2.1 Current state

- `LayoutCacheRow` (TS at `src/lib/sight/v6/types.ts:30` + Rust at `src-tauri/src/sight_v6.rs:49`) has 17 fields. No tradition-kind fields.
- `sight_v6_layout` table (SQLite, created at `sight_v6.rs:87`) has 17 columns matching the struct. Sentinel: `'mig025_sight_v6_layout_backfill_v1'`.
- Backfill query (`sight_v6.rs:215-283`) reads from `note_meta.properties_json` via `json_extract` for `stage` (`$.stage`) + `acts` (`$.act`). Adding more `json_extract` lines for tradition fields is mechanical.
- Each of the 9 tradition modules has a `xxxOf(_row)` function that returns a hardcoded constant. They take `LayoutCacheRow` as input but ignore it.

### §2.2 Out of scope (explicit)

- Frontmatter editor UI for these fields (user writes them in `.md` YAML manually). UX assist is a future polish item.
- Validation of frontmatter values at write time (NotePane doesn't gate). Invalid values fall back to default in the renderer.
- Settings flag to opt in/out per-tradition. Always on; defaults preserve current behavior.
- Bulk-edit / "classify all my notes" wizard. User-editable per-note only.

### §2.3 Invariants that must hold

I1. **Existing behavior preserved when field is absent.** Notes without the relevant frontmatter key fall to the same default they have today (`quran` for masadir, `pratyaksha` for pramana, etc.).

I2. **Schema additive only.** The 9 new columns are nullable; ALTER TABLE ADD COLUMN preserves all existing rows. No DROP / no breaking change.

I3. **Backfill is idempotent + resumable.** Sentinel version bumps v1 → v2 to trigger a one-time rebuild that fills the new columns. Mid-backfill interrupt rolls back the transaction (existing pattern).

I4. **Invalid frontmatter values fall back to default.** If a user writes `masadir_source: nonsense`, the lookup hits a `switch` default and returns `quran`. No crash; no rendering glitch.

I5. **Hot path performance unchanged.** Per-note tradition lookup is an O(1) string compare. The IPC payload grows by 9 nullable strings per note (most null) — for a 7,636-note universe that's at most ~150KB extra over the wire vs the current ~1.5MB. Negligible.

I6. **Cross-tradition isolation preserved.** Per-note frontmatter for tradition A doesn't affect tradition B's bucketing. Each lookup reads its own field.

I7. **Cache invalidation triggers fire correctly.** Existing AU trigger on `note_meta` UPDATE invalidates the cache row; next IPC re-derives with the new frontmatter value. No new trigger plumbing needed.

I8. **Frontend types stay aligned with Rust types.** TS `LayoutCacheRow` mirrors Rust `LayoutCacheRow` field-for-field (camelCase Serde).

## §3 — Design options

### Option A — Cache the values in `sight_v6_layout` (recommended)
Add 9 nullable TEXT columns to `sight_v6_layout`. Backfill query extracts via `json_extract(nm.properties_json, '$.<field>')`. Tradition modules read `row.<field>`. **Selected.**

Pros: O(1) read at chip-switch time; consistent with how `stage` + `acts` already work; cache-invalidation triggers handle update propagation.
Cons: schema migration needed (9 new columns); one-time backfill cost on existing universes.

### Option B — Lazy extraction at IPC time
Skip the schema change. Have the IPC handler extract the 9 fields from `note_meta.properties_json` on every Sight open.

Pros: no schema migration; no backfill.
Cons: every chip switch (or every Sight open) does 9 json_extracts × N notes; on 7,636 notes that's ~69k extra json calls per open. Measurable cost. Defeats the write-time derivation principle (CLAUDE.md Rule 8).

### Option C — Hybrid: cache the indexed ones, lazy the rest
Cache only the most-used (say `masadir_source` + `pramana_kind`); lazy-extract the rest.

Pros: smaller schema delta.
Cons: inconsistent behavior — some traditions feel snappy, others have a pause. Hard to reason about.

**Selection: Option A.** Matches write-time-derivation (CLAUDE.md Rule 8) and the existing `stage` + `acts` pattern. Schema migration is straightforward (9 nullable column adds; idempotent via PRAGMA check).

## §4 — Phased plan

### §4.ν.1 — Type contract
- TS `LayoutCacheRow` (`src/lib/sight/v6/types.ts`): add 9 optional fields
- Rust `LayoutCacheRow` (`src-tauri/src/sight_v6.rs`): add 9 `Option<String>` fields
- Both must stay aligned (camelCase Serde renames matter)
- Verification: TypeScript build + Rust check both pass

### §4.ν.2 — Schema migration + backfill
- Helper function in `sight_v6.rs`: `ensure_sight_v6_layout_tradition_columns(conn)` — runs `PRAGMA table_info(sight_v6_layout)`, parses column list, ALTER TABLE ADD COLUMN for any of the 9 missing. Idempotent.
- Wire into `init_db` (search.rs) AFTER `ensure_sight_v6_layout_table`.
- New sentinel: `'mig029_sight_v6_layout_tradition_fields_v1'` — when missing, run a one-time backfill update that refills the new columns from `note_meta.properties_json`. Skip if already stamped.
- Update the main backfill query (for new universes): the SELECT now includes 9 extra `json_extract` columns.
- Verification: fresh install + upgrade-from-MIG-028 install both populate the columns correctly.

### §4.ν.3 — Tradition modules
- Each of the 9 modules: replace the `xxxOf(_row)` constant-return with a switch on `row.<field>` + default fallback.
- Type the field strict (e.g. `MasadirSource = 'quran' | 'sunnah' | 'ijma' | 'qiyas'`) and narrow at read time.
- Update the existing TODO comment to a "fixed in MIG-029" historical note.
- Verification: vitest channel-isolation tests still pass (no regression to mini-dome tradition-agnosticism); tradition-perf tests still pass (no slowdown).

### §4.ν.4 — User Manual chapter
- New chapter: `docs/User Manual.md` + 14 locales — "Per-note tradition fields"
- Lists each tradition + frontmatter field name + allowed values + default
- 15-locale backfill via agent (same pattern as λ-fix-3)
- Verification: help-files include the chapter; orientation §17 list of help topics adds it

### §4.ν.5 — Build + Boss test
- NSIS build
- Boss test tutorial: create 4 test notes with each `masadir_source` value, switch to masādir chip, verify each star lands in its correct sector. Repeat for 2-3 other traditions sample.
- Verification: Boss-test PASS

### §4.ν.6 — Audit
- 3-agent /migration audit: invariants / drift / migration-path
- All findings folded into MIG-029 close-out

### §4.ν.7 — Close-out
- Commit chain: §ν.1 → §ν.7
- Milestone tag `milestone/mig029-frontmatter-wiring`
- ZIP backup
- Orientation v2.17 → v2.18 (MIG-029 marked Closed in §8)
- Pending Jobs ledger update

## §5 — Per-tradition field reference (build inputs)

| Tradition module | Frontmatter field | Allowed values | Default | TS type |
|---|---|---|---|---|
| `masadir.ts` | `masadir_source` | `quran` / `sunnah` / `ijma` / `qiyas` | `quran` | `MasadirSource` |
| `pramana.ts` | `pramana_kind` | `pratyaksha` / `anumana` / `upamana` / `shabda` | `pratyaksha` | `PramanaKind` |
| `ibn-rushd-burhan.ts` | `burhan_kind` | `burhan` / `jadal` / `khataba` / `shir` | `burhan` (or hash) | `BurhanKind` |
| `pardes.ts` | `pardes_level` | `peshat` / `remez` / `derash` / `sod` | `peshat` | `PardesLevel` |
| `peirce.ts` | `peirce_category` | `firstness` / `secondness` / `thirdness` | `firstness` | `PeirceCategory` |
| `habermas.ts` | `habermas_interest` | `technical` / `practical` / `emancipatory` | `technical` | `HabermasInterest` |
| `mencian-sprouts.ts` | `mencian_sprout` | `ceyin` / `xiuwu` / `cirang` / `shifei` | `ceyin` | `MencianSprout` |
| `mohist-san-biao.ts` | `mohist_zone` | `ben` / `yuan` / `yong` | `ben` (or hash) | `MohistZone` |
| `korean-songnihak.ts` | `songnihak_cell` | `li-sa` / `li-chil` / `qi-chil` / `qi-sa` | `li-sa` | `SongnihakCell` |

(`burhan_kind` and `mohist_zone` were originally hash-bucketing because frontmatter fields weren't extracted; with MIG-029 they read from the field and fall back to their hash-bucket only when the field is absent — preserves existing behavior.)

## §6 — Risks

- **Risk A**: Schema migration on Boss's existing 7,636-note universe blocks boot. Mitigation: the backfill update runs in a transaction; on a fresh install the existing v1 backfill already handles 7,636 rows in seconds. The v2 incremental update is even cheaper (UPDATE WHERE rather than INSERT).
- **Risk B**: User puts a typo in frontmatter (`masadir_source: sunna` missing the `h`) → falls to default. Mitigation: invariant I4 (graceful fallback). Future polish: a frontmatter linter could flag.
- **Risk C**: Mid-backfill interrupt leaves cache partially filled. Mitigation: sentinel-after-commit pattern (existing in v1 backfill, mirror for v2).
- **Risk D**: TS-Rust contract drift if one side updated without the other. Mitigation: type contract phase (§ν.1) lands BOTH in a single commit so they're never out of sync.

## §7 — Verification clauses (per phase)

- §ν.1 — `npm run check` (svelte-check) + `cargo check` both green; new fields visible in both type definitions.
- §ν.2 — On a fresh install, `SELECT COUNT(*) FROM pragma_table_info('sight_v6_layout') WHERE name IN ('pramana_kind', ..., 'songnihak_cell')` returns 9. On an upgrade, same query also returns 9 after first boot.
- §ν.3 — vitest `npm run test:sight-v6` 58/58 passes (the channel-isolation invariant is intact); a new ad-hoc test reads a few notes with mock frontmatter + verifies bucket assignment.
- §ν.4 — User Manual + 14 locale mirrors contain the per-tradition fields chapter.
- §ν.5 — Boss test PASS.
- §ν.6 — 3-agent audit zero P0/P1 findings.

---

**End of MIG-029 Architect+Plan.**

Per "Plan Approval = Build Approval" (Eisa direction "Proceed" 2026-05-19): cascading to Build now without per-step approval. Will surface user-testable Boss test at §ν.5.
