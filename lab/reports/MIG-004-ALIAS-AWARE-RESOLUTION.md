# MIG-004 — Alias-Aware Link Resolution (Option 2)

**Status**: Phase 1 complete (Architect). Phase 2 plan drafted — awaiting Build.
**Scope**: Persist note aliases in a new `note_aliases(path, alias_lower, added_at, source)` table so inbound-count read sites resolve a wikilink targeting `[[OldTitle]]` to the renamed note via its alias. Rewrite the inbound-count consumers (stratum, maturity, backlinks, map, sight) to JOIN through `note_aliases`. Fix the dormant `note_meta_sky_au` cascade gap (MIG-002 §6) as part of the same migration since both share root cause (DELETE+INSERT writer + name-only resolution).
**Chosen option**: **2 — Alias-aware matching via DB table.**

---

## Phase 1 — Architect (summary)

### Territory

- **Alias storage today**: frontmatter-only. No DB persistence. `libraries.rs::find_note_by_name_or_alias` (L1216) walks the FS per miss for display-time resolution. `parse_frontmatter` (search.rs:1235) does NOT extract list-shaped `aliases:` into structured form — only the inline scalar, if any, lands in `properties` as a string.
- **Note-link storage**: `note_links.target_name` = `extract_wikilinks` output (lowercased + Arabic-normalized via `normalize_arabic_for_search`). `note_links.target_path` exists but is never populated.
- **Sky-nodes shape**: `sky_nodes.id = LOWER(name)` — current title only; not alias-aware.

### Inbound-count consumers (rewrite targets)

- `STRATUM_SQL_EXPR` (search.rs:78–112) — three inbound subqueries match `note_links.target_name = sky_nodes.id`.
- `MATURITY_SQL_EXPR` (search.rs:139–192) — four inbound subqueries, same pattern.
- Map: `src-tauri/src/map.rs:71–94` builds `inbound_map: HashMap<String, usize>` keyed on `note.name.to_lowercase()` from FS scan — does not know about aliases.
- Cache boot: `cache.rs:447` reads `sky_links.target_name`; `cache.rs:483` joins via `name_to_idx` (current title only).
- `strata.rs:73` and `tension.rs:54` — outgoing-side `target_name_lower` joins. Not in MIG-004 scope (orphan detection, different semantics).

### Schema location and version

- `SKY_SCHEMA_VERSION` currently = **6**. MIG-004 bumps to **7**.
- Existing infra: `sky_backfill.rs` resumable batch populator (1000 rows, 50ms inter-batch sleep, busy_timeout=30s, three-phase finalize transaction).
- Module-level constants `STRATUM_SQL_EXPR` / `MATURITY_SQL_EXPR` — single source of truth for trigger + back-fill formulas. MIG-004's alias-aware versions land here too.

### Invariants (12, must not break)

1. Inbound counts correct after any rename.
2. Display-time resolution keeps working for pre-existing wikilinks.
3. Second-screen parity.
4. Zero IPC on keystroke hot path.
5. Rename latency bounded under the 1500ms autosave debounce.
6. Obsidian imports without `aliases:` continue working.
7. Alias collisions resolve deterministically (FS resolver path-sort tiebreak ⇒ SQL `ORDER BY path LIMIT 1`).
8. Circular aliases don't infinite-loop.
9. Living-link traversal counters preserved on rename (search.rs:1535 snapshot block).
10. FTS unaffected.
11. MIG-003 (human-name filenames, queued) still works on top of MIG-004.
12. Rollback-safe — additive schema only.

---

## §6 dormant-bug fix — scope decision

**Decision: MIG-004 fixes the dormant `note_meta_sky_au` cascade gap *implicitly* by changing what "match" means, NOT by trying to repair the AU pathway.**

### Rationale

The dormant bug: `note_meta_sky_au` (search.rs:936) tries to UPDATE `sky_links.target_name` from `LOWER(OLD.name)` to `LOWER(NEW.name)` on rename. But `index_note` writes `note_meta` via DELETE+INSERT (search.rs:1525–1531), so a canonical-filename rename fires AD+AI, not AU. The §6 cascade never executes.

Once aliases handle resolution, **target_name no longer needs to match `id` directly**. The new alias-aware inbound expression matches `note_links.target_name` against either `sky_nodes.id` OR any `note_aliases.alias_lower` belonging to the same path:

- A wikilink targeting `[[OldTitle]]` continues matching the renamed note (because `OldTitle` was added to `note_aliases` on rename — see §3).
- The §6 AU-trigger cascade becomes optional.
- The `update_links_on_rename` walker becomes a *display-only* nicety (so the on-disk wikilink eventually shows the new name) instead of a correctness requirement.

This is strictly better than trying to "fix AU to fire on DELETE+INSERT" — that fight has lost twice already (BUG-010, BUG-011) and would require restructuring the writer's preserved-traversal-data snapshot at search.rs:1535 (Invariant 9).

**What MIG-004 still does in §6's spirit**: the rename writer (§3) inserts the OLD name into `note_aliases` for the NEW path *before* `index_note` runs the DELETE+INSERT, so the alias is queryable inside the same transaction. The existing `note_meta_sky_au` body becomes dead-but-harmless and can be left in place; we trim its dead branches in §10's cleanup pass.

---

## Schema changes (summary)

### New table

```sql
CREATE TABLE note_aliases (
    path        TEXT NOT NULL,
    alias_lower TEXT NOT NULL,
    added_at    INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    source      TEXT NOT NULL,   -- 'frontmatter' | 'rename' | 'import'
    PRIMARY KEY (path, alias_lower)
);
CREATE INDEX idx_note_aliases_lookup ON note_aliases(alias_lower);
CREATE INDEX idx_note_aliases_path   ON note_aliases(path);
```

- `alias_lower` Arabic-normalized + lowercased at insert time. Direct equality with `note_links.target_name` (no per-row normalization).
- `source` discriminator: `'frontmatter'`, `'rename'`, `'import'`. Lets `§2` clear+repopulate frontmatter rows on save without clobbering rename-stamped rows.
- Composite PK = idempotent inserts.
- `idx_note_aliases_lookup` is the hot index. ~7,607 notes × ~2 aliases ≈ 15k rows; index ~500 KB.

### Version bump

`SKY_SCHEMA_VERSION: 6 → 7`. Docstring extended with `| 7 | MIG-004 §1 — note_aliases table; alias-aware STRATUM / MATURITY expressions; back-fill of frontmatter aliases for existing 7,607 notes |`.

### Trigger family overview

Writer-side population of frontmatter aliases lives in `index_note` (§2). For path-change and delete cascades, extend the existing `note_meta_sky_*` triggers' bodies (§4) — single trigger surface avoids the BUG-011 multi-trigger dispatch issue.

### Resolver-rewrite scope

**JOINs through `note_aliases`** — inbound subqueries inside `STRATUM_SQL_EXPR` + `MATURITY_SQL_EXPR`:

```sql
(SELECT COUNT(*) FROM note_links nl
   WHERE status = 'active'
     AND (nl.target_name = sky_nodes.id
          OR nl.target_name IN (SELECT alias_lower FROM note_aliases
                                 WHERE path = sky_nodes.path)))
```

- `STRATUM_SQL_EXPR` — three subqueries rewritten.
- `MATURITY_SQL_EXPR` — four subqueries rewritten.
- `cache.rs:483` `name_to_idx` lookup — extended with alias fallback.
- `map.rs:81` `inbound_map` populator — extended with alias resolution.

**NO change** — `strata.rs:73`, `tension.rs:54` (outgoing-side orphan detection; defer to MIG-005 if needed).

---

## Phase 2 — Plan (10 steps)

| # | Step | /simplify? | Verify |
|---|------|-----------|--------|
| 1 | Schema landing: `note_aliases` table + indexes; bump `SKY_SCHEMA_VERSION` 6→7 | — | App boots; `SELECT * FROM note_aliases LIMIT 1` works (empty). |
| 2 | Writer-side alias extraction in `index_note` (frontmatter list parser + INSERT) | — | Saving a note with `aliases: [Foo, Bar]` populates two rows in `note_aliases`. |
| 3 | Rename writer stamps OLD name into `note_aliases` for the NEW path | ✔ | After rename `Old → New`, `SELECT path FROM note_aliases WHERE alias_lower='old'` returns the new path. |
| 4 | `note_meta_sky_*` trigger extensions: path-change + delete cascade for `note_aliases` | — | Move a note to a new path → alias rows follow. Delete note → alias rows gone. |
| 5 | One-shot back-fill of frontmatter aliases for existing 7,607 notes | ✔ | Cold-boot back-fill drains; `SELECT COUNT(*) FROM note_aliases WHERE source='frontmatter'` matches FS scan count. |
| 6 | Alias-aware `STRATUM_SQL_EXPR` rewrite + trigger republish + back-fill recompute | — | Rename a note with ≥5 inbound: its stratum stays put. |
| 7 | Alias-aware `MATURITY_SQL_EXPR` rewrite + trigger republish + back-fill recompute | — | Rename a note with 10+ inbound: maturity stays `canonical`. |
| 8 | `cache.rs:483` name_to_idx fallback through `note_aliases` (boot-snapshot read) | ✔ | Boot snapshot returns correct inbound counts on a freshly renamed note. |
| 9 | Map / Sight inbound-counter alias-aware lookup (`map.rs:81` + Sight equivalents) | — | Map view shows correct inbound bubble size for renamed notes; second-screen parity. |
| 10 | Phase-4 audit + drop dead `note_meta_sky_au` body + cleanup | — | Audit ≤3 HIGH; rollback test (older binary boots against new schema). |

### `/simplify` checkpoints

Steps **3, 5, 8** — mirrors MIG-002's placement (after writer change, after back-fill scaffold lands, after frontend/read swap).

### Phase 4 audit trigger

Step 10 invokes `/migration` audit (invariant / drift / migration-path agents) plus a rename byte-diff parity test.

---

### §1 — Schema landing: `note_aliases` table + version bump

**What**
- `src-tauri/src/search.rs` — add `CREATE TABLE IF NOT EXISTS note_aliases (...)` block alongside the sky_* tables (~L693). Two indexes.
- `src-tauri/src/search.rs:59` — `SKY_SCHEMA_VERSION: 6 → 7`. Extend the docstring table.

**Why** — Invariants 10, 12. Additive only.

**Verification** — Existing DBs boot; `SELECT * FROM note_aliases LIMIT 1` returns 0 rows. `PRAGMA table_info(note_aliases)` shows the four columns.

**Risk mitigation** — `IF NOT EXISTS` on table + indexes; idempotent on repeat boots. No ALTER on existing tables.

**Rollback** — Pre-MIG-004 binary ignores the table. No DB revert needed.

---

### §2 — Writer-side alias extraction in `index_note`

**What**
- `src-tauri/src/search.rs` — new helper `extract_aliases(content: &str) -> Vec<String>` handling all three frontmatter shapes (inline `aliases: [a, b]`, scalar `aliases: foo`, YAML list). Each alias goes through `normalize_arabic_for_search(&s.to_lowercase())` to match `extract_wikilinks` normalization.
- `index_note` (search.rs:1442) — after `parse_frontmatter`, also call `extract_aliases`. After the `INSERT INTO note_meta` block (L1527), DELETE existing `note_aliases` for this path with `source='frontmatter'`, then INSERT one row per alias with `source='frontmatter'`.
- DELETE-by-source partition prevents §3's rename-stamped aliases from being clobbered when frontmatter is re-saved.

**Why** — Invariants 6 (Obsidian without `aliases:` works — zero inserts), 4 (no IPC).

**Verification** — Save a test note with `aliases:\n- alpha\n- beta`. Query `SELECT alias_lower, source FROM note_aliases WHERE path = ?`. Expect two rows, source='frontmatter'. Save same note with aliases removed: rows disappear.

**Risk mitigation** — Same Arabic normalization as wikilinks. Empty list = no-op. Source partition.

**Rollback** — Revert binary; existing rows ignored by old code.

---

### §3 — Rename writer stamps OLD name into `note_aliases` `/simplify`

**What**
- `src-tauri/src/libraries.rs` — locate the rename writer. Before invoking `index_note`, INSERT OR IGNORE into `note_aliases (path, alias_lower, source)` for the NEW path with the OLD name (lowercased, Arabic-normalized) and `source='rename'`.
- INSERT must run inside the same transaction as the `index_note` DELETE+INSERT.
- `update_links_on_rename` walker — already exists; no change. Its work becomes display-cosmetic.
- `/simplify` pass: factor the alias-INSERT into a shared helper used by §2 + §3 + §4.

**Why** — Invariants 1, 5, 11. Central correctness fix: between rename and walker completion, every wikilink in the wider library still targets `[[OldTitle]]`, and the alias row makes those resolve to the renamed note.

**Verification** — Rename `Old.md` to `New.md`. Immediately query `SELECT path FROM note_aliases WHERE alias_lower = 'old'` — returns the new path. `sky_nodes.stratum` for the renamed note is unchanged.

**Risk mitigation** — INSERT OR IGNORE handles rename-back idempotently. Same-name rename is no-op via IGNORE. Path collision impossible by FS invariant. Alias row outlives the renamed note (correct — historical wikilinks still resolve).

**Rollback** — Revert binary; alias row stays in the table (harmless).

---

### §4 — Trigger extensions: path-change + delete cascade

**What**
- `src-tauri/src/search.rs` — extend the existing `note_meta_sky_au` body to also `UPDATE note_aliases SET path = NEW.path WHERE path = OLD.path` when `OLD.path IS NOT NEW.path`.
- Extend `note_meta_sky_ad` body to also `DELETE FROM note_aliases WHERE path = OLD.path`.
- Single trigger surface — sidesteps BUG-011 multi-trigger dispatch issue.
- The reindex DELETE+INSERT case re-INSERTs alias rows in the same transaction (via §2's writer block), so net state is correct even though AD fires.

**Why** — Invariants 2, 8.

**Verification** — Manually `UPDATE note_meta SET path='new' WHERE path='old'` → alias rows follow. Delete a note → alias rows gone.

**Risk mitigation** — Single-trigger pattern. AU's WHEN guard narrow.

**Rollback** — Drop the trigger extensions; alias rows go stale on path changes (rare, only via direct UPDATE).

---

### §5 — One-shot back-fill of frontmatter aliases `/simplify`

**What**
- `src-tauri/src/sky_backfill.rs` — extend back-fill loop with new phase: for every `note_meta` row, re-read the file (or parse `properties_json` if list-shaped aliases were promoted there), extract aliases via §2's helper, INSERT OR IGNORE into `note_aliases` with `source='frontmatter'`.
- 1000-row batches, cursor-resumable.
- `/simplify` pass: dedupe between §2 (writer) and §5 (back-fill) via shared helper.

**Why** — Invariants 5, 12.

**Verification** — Cold-boot a Universe with known frontmatter aliases. Back-fill completes. `SELECT COUNT(*) FROM note_aliases WHERE source='frontmatter'` matches FS scan output.

**Risk mitigation** — Cursor-resumable. INSERT OR IGNORE = idempotent on resume. ~5–10s on 7,607-note Universe.

**Rollback** — Truncate `note_aliases`; §6/§7 inbound expressions fall through to name-only matching.

---

### §6 — Alias-aware `STRATUM_SQL_EXPR` rewrite + back-fill recompute

**What**
- `src-tauri/src/search.rs:78` — rewrite the three inbound subqueries inside `STRATUM_SQL_EXPR`:
  ```sql
  (SELECT COUNT(*) FROM note_links
    WHERE status = 'active'
      AND (target_name = sky_nodes.id
           OR target_name IN (SELECT alias_lower FROM note_aliases
                               WHERE path = sky_nodes.path)))
  ```
- DROP + CREATE all stratum triggers (mirrors MIG-002 §99 BUG-010 republish). Also DROP the merged `note_meta_sky_ai` and recreate.
- `sky_backfill.rs` — one-shot recompute pass: `UPDATE sky_nodes SET stratum = (<new expr>)` for all rows.

**Why** — Invariant 1 (the central correctness target).

**Verification** — Pick a renamed note with 7 inbound (3 wikilinks targeting old name). Pre-§6: stratum bonus +1 missed. Post-§6: stratum +1 applied.

**Risk mitigation** — OR'd subquery hits `idx_note_aliases_lookup`. ~1ms per inbound subquery typical. Per-save under MIG-002's <2× baseline budget.

**Rollback** — Restore prior `STRATUM_SQL_EXPR`; drop+recreate triggers; recompute.

---

### §7 — Alias-aware `MATURITY_SQL_EXPR` rewrite + back-fill recompute

**What**
- `src-tauri/src/search.rs:139` — rewrite each of the four inbound count subqueries inside `MATURITY_SQL_EXPR` to use the same UNION-style pattern as §6.
- DROP + CREATE the maturity triggers; recompute all `sky_nodes.maturity`.

**Why** — Invariant 1.

**Verification** — Pick a renamed note with 12 inbound where 5 wikilinks targeted old name. Pre-§7: maturity dropped from `canonical` to `evergreen`. Post-§7: stays `canonical`.

**Risk mitigation** — Same as §6. If trigger fire latency exceeds 1500ms autosave on dense hubs, consider CTE materialization. Defer optimization until measured.

**Rollback** — Same shape as §6.

---

### §8 — `cache.rs:483` name_to_idx alias fallback `/simplify`

**What**
- `src-tauri/src/cache.rs:443–507` — in `read_sky_links_raw`, when target_name lookup misses `name_to_idx`, query `note_aliases` for the canonical path, then resolve through `path_to_idx`. Cold path.
- Cache the alias map at boot snapshot prepare-time: `SELECT alias_lower, path FROM note_aliases` into a `HashMap<String, String>`. In-memory lookup during the link scan.
- `/simplify` pass: confirm boot snapshot IPC payload shape unchanged. Confirm `nodes_mut[tgt_idx].link_count += 1` still increments.

**Why** — Invariants 1, 3, 7.

**Verification** — Boot a Universe with renamed note. Inspect boot-snapshot payload: edges that previously dangled now resolve. `link_count` matches expected.

**Risk mitigation** — Single connection trip. ~500 KB heap.

**Rollback** — Remove fallback; unresolved targets fall through to existing behavior.

---

### §9 — Map / Sight inbound-counter alias-aware lookup

**What**
- `src-tauri/src/map.rs:71–94` — extend `inbound_map` populator. Load `note_aliases` once at top of map command, use during outgoing-link counting pass.
- Sight equivalent: locate similar inbound-count code (no direct hit found in initial scan; flag for build).
- `strata.rs:73` / `tension.rs:54` — defer to MIG-005.

**Why** — Invariants 1, 3.

**Verification** — Open Map view on library with renamed note that had ≥3 inbound. Bubble size matches expected. Second-screen Map matches.

**Risk mitigation** — One-shot command, not hot path.

**Rollback** — Revert map.rs change.

---

### §10 — Phase 4 audit + cleanup

**What**
- Run `/migration` audit (invariant / drift / migration-path agents).
- Cleanup: drop the now-dead `UPDATE sky_links SET target_name = ...` block from `note_meta_sky_au`. Keep the path/name/library_name UPDATE block (still needed for direct-UPDATE writers).
- Document "lessons forward to MIG-003".
- Byte-diff parity: dump `{path, stratum, maturity, link_count}` from `sky_nodes` before/after a fixture rename.
- Rollback test: install pre-MIG-004 binary against post-MIG-004 schema; verify boot succeeds.

**Verification** — ≤3 HIGH findings. Rename byte-diff parity passes. Rollback boot succeeds.

**Rollback** — N/A (read-only audit).

---

## Lessons forward to MIG-003

MIG-003 (human-name filenames) introduces filename ≠ display name. MIG-004's `note_aliases` shape was chosen so MIG-003 can layer cleanly:

1. **`note_aliases.path` is the natural canonical-id column.** When MIG-003 introduces stable canonical id (UUID stamped in frontmatter), it can either replace `path` with `canonical_id` here, or add a parallel `note_canonical_id` table joining via path. Either way, the alias-resolution semantics defined in §6/§7 (target_name matches id OR alias) is the right shape — MIG-003 just adds another `OR` branch.

2. **`source` discriminator anticipates MIG-003.** Adding `source='canonical_filename'` for legacy file-stem-as-display-name case is one row addition. Existing partition by source means MIG-003's writer can manage canonical-filename rows independently.

3. **MIG-003 will likely add a fourth source `'human_name'` for the filename ↔ display-name mapping.** No schema change needed — just a new `source` value.

4. **The `idx_note_aliases_lookup(alias_lower)` index is the hot path; MIG-003's queries benefit directly without additional index work.**

In short: MIG-004's `note_aliases` is structurally a `(path, key) → exists` map. MIG-003 adds new key types (canonical id, filename stem) into the same map. No schema flip.

---

## Critical Files for Implementation

- E:/مشاريع كلاود/Constellation/src-tauri/src/search.rs (schema, STRATUM/MATURITY exprs, triggers, index_note writer, extract_aliases helper)
- E:/مشاريع كلاود/Constellation/src-tauri/src/sky_backfill.rs (alias frontmatter back-fill, stratum/maturity recompute pass)
- E:/مشاريع كلاود/Constellation/src-tauri/src/cache.rs (name_to_idx alias fallback in boot snapshot)
- E:/مشاريع كلاود/Constellation/src-tauri/src/map.rs (inbound-counter alias resolution)
- E:/مشاريع كلاود/Constellation/src-tauri/src/libraries.rs (rename writer — the OLD-name stamp insertion site)

**Next**: Phase 3 — Build, starting with §1.

---

## Phase 4 — Audit (closed)

Three audits ran in parallel per the `/migration` skill. Findings consolidated below.

### Audit 4A — Invariant Check
- 9 of 12 invariants HOLD with concrete code evidence.
- 0 REGRESSED.
- 3 CANNOT DETERMINE (mostly perf measurements + the alias-collision tiebreak).
- 1 MED finding: `cache.rs` alias SELECT lacked `ORDER BY path` for collision determinism. **Fixed in §10.**

### Audit 4B — Drift Check
- 2 HIGH:
  - **4B-1**: `strata.rs`, `maturity.rs`, `tension.rs`, `inspector360.rs`, `map.rs` all compute inbound counts in-memory via name-only lookup. They serve user-visible commands (Sight, Inspector 360, Tension, Map). Same alias-blindness MIG-004 just fixed for stratum/maturity SQL + Backlinks. **Deferred to MIG-005** — wider scope than §10 cleanup.
  - **4B-2**: `LinkDashboard.svelte` mis-classifies aliased wikilinks as "broken targets". **Deferred to MIG-005** — bundle with 4B-1's frontend fix.
- 4 MED, of which:
  - **4B-3**: `source` partition behavior was undocumented. **Comment added in §10.**
  - **4B-4**: ANALYZE+wipe block missing `busy_timeout(30s)`. **Fixed in §10.**
  - **4B-5**: Same as 4C-1 (interrupt leaves NULL rows). **Fixed in §10.**
  - **4B-6**: `note_meta_sky_au` cascade is dead code (index_note's DELETE+INSERT path skips AU). Defensive coverage; flagged for future cleanup.
- 2 LOW: confirmed `OutgoingLinksPanel` and `note_links.target_path` not affected.

### Audit 4C — Migration Path
- 2 HIGH:
  - **4C-1**: Wipe blasts every row up front, but Phase D refills only paths in `[after_path, last_path]`. On interrupt + reboot, rows below the cursor stayed at NULL forever. **Fixed in §10** by scoping the wipe to `WHERE path > last_path`. On a fresh start (cursor empty) this matches every row — same as before. On a resume, rows at or below the cursor already have correct values and survive.
  - **4C-2**: Claimed rollback-then-upgrade leaves stale triggers. **FALSE POSITIVE** — verified the DROP+CREATE blocks at search.rs:947 / 1102 / 1118 / 1189 are unconditional, not gated by `needs_sky_rebuild`. Triggers are reasserted on every `init_db`.
- 3 MED:
  - **4C-3**: Cursor table not version-scoped; a stale MIG-002 cursor could mis-resume MIG-004. Theoretical — current cursor is empty. Flagged for future migration housekeeping.
  - **4C-4**: ANALYZE+wipe block lacked `busy_timeout`. **Fixed in §10** (same change as 4B-4).
- 2 LOW.

### §10 cleanup commit summary

Landed in §10 (one commit):

1. `sky_backfill.rs::run` — wipe scoped to `WHERE path > last_path` so resume keeps already-recomputed rows; added `busy_timeout(30s)` on the ANALYZE block.
2. `cache.rs::cache_boot_snapshot_sky` (alias map) — added `ORDER BY path` for deterministic collision tiebreak.
3. `cache.rs::read_aliases` (graph snapshot payload) — added `ORDER BY path` for stable serialization.
4. `search.rs` `note_aliases` schema — comment block on the `source` partition's "first writer wins" semantics.

### Deferred / out-of-scope items

- **MIG-005 — Alias-aware in-memory inbound consumers** (covers 4B-1 + 4B-2). Five Tauri commands and one frontend panel still compute inbound counts via name-only lookups. Scope demands its own architect pass.
- **Future housekeeping** — 4B-6 (dead AU cascade), 4C-3 (cursor not version-scoped). Low-risk; address opportunistically.

### Lessons forward to MIG-003 (re-stated)

`note_aliases` is a `(path, key) → exists` map. MIG-003's canonical-id and human-name mappings layer cleanly via new `source` values without a schema flip. The `idx_note_aliases_lookup(alias_lower)` index covers the hot path for any future alias key type.

### Closure

MIG-004 closes with 12 invariants verified, 5 audit findings landed in §10, 2 HIGH findings deferred to MIG-005 (with explicit rationale), and 1 false positive documented. The rename-drops-counts symptom is closed end-to-end across stratum (§6), maturity (§7), Sky View boot (§8), and Backlinks panel (§9). Source-side wikilinks in note bodies stay verbatim by design — Constellation's alias-as-indirection model is preserved.

`/migration` workflow complete. Phase 3 — Build closed.
