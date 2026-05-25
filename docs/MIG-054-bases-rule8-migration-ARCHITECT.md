---
title: MIG-054 — Bases Phase 1, Rule 8 Migration (Architect)
version: 1.1
date: 2026-05-25
status: Architect doc with all 5 Q1-Q5 decisions folded in. Awaiting Eisa's approval before Plan doc.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
predecessor: docs/Constellation-Base-Concept-Paper-v1.4.md (Phase 1 of the roadmap §12)
phase_in_concept_paper: Phase 1 — Rule 8 Migration
mig_number_note: MIG-054 (the orientation v2.35 preamble and session log were originally drafted with MIG-049, allocated from the planned-but-never-built Mind cascade. Per Eisa's lock on §10 Q5 — uniqueness-aligned choice — we jumped to MIG-054 to preserve clarity of the audit trail. The orientation v2.35 references are corrected inline in the same commit that lands this Architect doc.)
---

# MIG-054 — Bases Phase 1, Rule 8 Migration (Architect)

## §1. Premise and Scope

Per the Constellation Base Concept Paper v1.4 §12, **Phase 1 is the architectural foundation** for everything that follows in the roadmap. It performs exactly one thing:

> Make `query_base` return in under 50ms on a 7,600-note Universe, by reading from **derived state maintained at write time** instead of walking the filesystem on every call.

This is the **dashboard-effect gate**. Effect §1 from v1.4 §3 — *"I see what's alive in my work right now"* — dies if the query is slow. Phase 1.5 (host-note assemblage + 3 threading gestures), Phase 2 (Living Link columns), 2.5 (360.3D Bridge), 2.6 (CNS Bridge), 2.7 (Cataloger Bridge), 3 (NSC headlines), 4 (federation auto-on), 5 (Five Acts templates) all build on a SQL-fast `query_base`. Without Phase 1, none of them feel right.

**Phase 1 does NOT** add new user-visible features. No new filter operators, no new view shapes, no new schema fields users can touch, no new column types. The user sees a Bases view they already had, returning instantly. That is the entire user-visible delivery — except for one quality-of-life addition adopted under Q2 (§5.8 below): the cell-edit refresh event, which removes the 1.5s file-watcher latency window.

This Architect doc maps the territory, enumerates design options with tradeoffs, lists the invariants that must not break, folds in Eisa's locked Q1-Q5 decisions, and recommends a path. The Plan doc (separate, follows after Eisa approves this) phase-decomposes the chosen path into landable commits with verification clauses.

---

## §2. Territory Mapping (current state, evidence-based)

### §2.1 What `bases.rs` does today (grounded read of the file)

`src-tauri/src/bases.rs` is 901 lines. Ten Tauri commands:

| Command | Line | Behavior |
|---|---|---|
| `parse_base_file` | 360 | Read a `.base` file from disk, parse as JSON, return `BaseDefinition`. (File extension is `.base` but content is actually JSON per `serde_json::to_string_pretty` at the create/save sites.) |
| `query_base` | 386 | **The hot path this MIG addresses.** Walks the filesystem of the active libraries, calls `parse_frontmatter()` per `.md` file, builds `Vec<BaseRow>`, applies filters + sorts in memory, returns `BaseQueryResult` with `query_time_ms`. |
| `create_base` | 525 | Create a new folder-scope `.base` file. |
| `save_base_file` | 592 | Persist a `BaseDefinition` to a folder-scope `.base` file. |
| `update_note_property` | 603 | Cell-edit-in-place → calls `update_frontmatter_property()` which rewrites the note's frontmatter on disk. **Q2 adds explicit refresh event after the write (see §5.8).** |
| `list_workspace_bases` | 717 | Enumerate `.base` files in `{universe}/.constellation/bases/`. |
| `create_workspace_base` | 755 | Create a new workspace-scope `.base`. |
| `save_workspace_base` | 805 | Persist a workspace-scope `.base`. |
| `delete_workspace_base` | 839 | Delete a workspace-scope `.base`. |
| `parse_workspace_base` | 861 | Read a workspace-scope `.base`. |

**Doc drift caught:** orientation v2.34 §4582 said "5 commands"; actual is **10**. Phase 1 corrects this inline.

The hot-path scan functions:
- `scan_folder` (bases.rs:210) — recursively `fs::read_dir` the library; for each `.md`, `fs::read_to_string` the whole file, call `parse_frontmatter`, build a `BaseRow`.
- `scan_by_tag` (bases.rs:262) — same shape, plus an inline check for `#tag` in body or `tags:` frontmatter list.
- `parse_frontmatter` (bases.rs:134) — line-by-line YAML-ish parser. Handles inline lists `[a, b, c]` and multi-line `- item` lists by joining with `, `. NOT a full YAML parser; key-value pairs only.

Plus the legacy `"vault"` source type at line 442 (Q1 retires it; see §5.7).

### §2.2 The `note_meta` gift (the architectural pivot)

`src-tauri/src/search.rs:1657` creates `note_meta` with this schema:

```sql
CREATE TABLE IF NOT EXISTS note_meta (
    path TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    library_name TEXT NOT NULL,
    modified INTEGER NOT NULL,
    content_hash TEXT,
    properties_json TEXT DEFAULT '{}',     -- ← THIS is the frontmatter, parsed at write time
    tags_json TEXT DEFAULT '[]',           -- ← extracted tags
    outgoing_links_json TEXT DEFAULT '[]', -- ← outgoing links
    headings_json TEXT DEFAULT '[]',
    body_text TEXT DEFAULT '',
    word_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER
);
```

**`note_meta.properties_json` is the parsed frontmatter, already maintained at write time.** Every write path that updates a note populates this. SQLite's `json_extract(properties_json, '$.<key>')` retrieves any individual frontmatter property in O(log n).

This means **Phase 1 does not need a new `bases_cache` table.** The Concept Paper §10.1 mandate ("table OR equivalent") is satisfied by `note_meta` itself — `properties_json` is the Bases equivalent of a cache.

This is the central pivot of the Architect doc: **the cache already exists; we just need to query it.**

### §2.3 The existing trigger pattern (the model)

`search.rs:1993` shows the canonical write-time-derivation pattern (`note_meta_ai` / `note_meta_ad` / `note_meta_au` AFTER INSERT/DELETE/UPDATE) for `notes_fts`. The same shape appears for `sky_nodes` (line 2382) and `sky_links` (line 2279). **Phase 1 does not need to write new triggers** if it queries `note_meta` directly — the write-time discipline is already in place upstream.

### §2.4 The `cell-edit-in-place` write path (Q2 lock applies here)

`update_note_property` (bases.rs:603) calls `update_frontmatter_property` which rewrites the note file on disk. **This does NOT directly update `note_meta`** — the file-watcher debouncer re-reads the file and re-populates `note_meta.properties_json` afterward (~1.5s window).

**Per Eisa's Q2 lock (uniqueness-aligned answer):** this latency window is rejected as productivity theater. The fix is explicit: `update_note_property` ALSO updates `note_meta.properties_json` directly in the same transaction, AND emits a Tauri event (`bases:note_updated` with the path). The frontend listens; the cell updates instantly. See §5.8.

### §2.5 Frontend wiring (verified from src/lib/bases/types.ts)

The TS types mirror the Rust structs cleanly. **One field-name drift noted:** TS uses `selectedLibraries`, Rust serdes use `selectedVaults` (with `#[serde(rename = "selectedVaults")]`).

**Per Eisa's Q1 lock (uniqueness-aligned answer):** the "Library" vocabulary is part of Constellation's identity. Phase 1 step §D unifies on `selectedLibraries` everywhere AND retires the legacy `"vault"` source type. See §5.7.

---

## §3. Invariants (must not break)

The Concept Paper v1.4 §5 Founding Principles + §10 Architectural Mandates yield the invariants for Phase 1. Each is testable.

1. **File-Over-App (§10.2).** `.base` files remain plain JSON-shape files on disk; nothing in Phase 1 moves Bases data into a proprietary container.
2. **Write-Time Derivation for per-note dimensions (§10.1).** `query_base` no longer scans the filesystem.
3. **Instant on 10k (§10.3).** `query_base` returns in under 50ms on a 7,600-note Universe.
4. **Multilingual Native (§10.4).** No regression on Arabic / Persian / Hebrew / mixed-script frontmatter property keys or values.
5. **Behavioral equivalence.** For every existing `.base` file on every existing Universe, the new SQL-backed `query_base` returns **the same `BaseRow` set** as the old filesystem-scan `query_base`. Central correctness invariant. See §4.E.
6. **No regression on `update_note_property` correctness.** Cell edits still rewrite the note's frontmatter file on disk. Behavior unchanged at the data layer; **latency window REMOVED per Q2** (see §5.8).
7. **No new IPC signatures.** The 10 existing commands keep their signatures (field rename `selectedVaults` → `selectedLibraries` is at the data shape level, not the command surface; backward compat reads handle the old name).
8. **Federation correctness preserved.** Multi-library queries via `selectedLibraries` continue to work.
9. **No `$effect` loop introduced.** Frontend changes (the cell-edit refresh listener) follow Svelte 5 runes discipline.
10. **Cross-library and workspace-base paths both work.** Folder-scope and workspace-scope `.base` files behave identically against the new query path.
11. **Backward compatibility with legacy `.base` files (Q1 implication).** Existing files using `selectedVaults` or `"vault"` source type are still parsed correctly; new files write the modernized form.

---

## §4. Design Options

### §4.A — Option A: New `bases_cache` table
Triggers on `note_meta` keep a mirror table. Cost: triggers + backfill + schema_version. Benefit: insulation from `note_meta` evolution.

### §4.B — Option B: Query `note_meta` directly (recommended)
`note_meta.properties_json` is already maintained. `query_base` becomes SQL over `note_meta` with `json_extract`. Zero new table, zero new triggers, zero backfill.

### §4.C — Option C: Hybrid — SQLite view on `note_meta`
`CREATE VIEW bases_cache AS SELECT ... FROM note_meta`. Naming separation; identical performance.

### §4.D — Comparison and recommendation

**Recommendation: Option B (query `note_meta` directly).** Concept Paper §10.1 says "table **or equivalent**"; `note_meta` IS the equivalent because `properties_json` is already write-time-maintained. Simplest correct change; zero first-boot cost; precedent in search.rs / sky.rs / nsc.rs / inspector360.rs which all read `note_meta` directly.

### §4.E — Behavioral equivalence verification (Q3 lock applies here)

Per Eisa's Q3 lock (uniqueness-aligned answer), the fixture set is designed as the **foundation suite that Phase 2 / 2.5 / 2.6 / 2.7 will extend**, not just a Phase 1 regression check.

**Locked fixture structure (~75 total):**

| Category | Count | Purpose for uniqueness |
|---|---|---|
| Source-type permutations (folder / tag / all) | 15 | Foundation for Phase 4 federation tests |
| Filter operators × representative property types | 16 | Foundation for Phase 2 Living Link filters (same operators, different property names) |
| Sort fidelity (text / numeric / date / multilingual) | 10 | Foundation for Phase 2.5 sort-by-stratum, Phase 2.6 sort-by-centrality, Phase 2.7 sort-by-content-type |
| Cross-library `selectedLibraries` | 10 | Foundation for Phase 4 federation auto-on |
| **Multilingual frontmatter** (Arabic / Persian / Hebrew property keys + values + mixed-script) | 10 | Locks Principle 5.5 (Language-First) at the foundation |
| Edge cases (empty results, all-empty properties, sparse properties) | 5 | Catches JSON-extract NULL-handling that future phases inherit |
| **Real Bases from Eisa's universe** | ~10 | The gold-master — whatever Eisa actually uses |

Each fixture: snapshot today's `query_base` output → land new implementation → diff every row, every property, every filter outcome. Fail loud on any mismatch.

**The same harness extends across phases.** Phase 2 adds Living Link fixtures; Phase 2.5 adds CE-dimension fixtures; Phase 2.6 adds CNS fixtures; Phase 2.7 adds CECE fixtures. The harness IS the regression suite for the entire roadmap.

**Test pattern follows LL-025 (Q4 lock):** test on a **copy** of Eisa's Cognitive Knowledge universe DB, not synthetic data. The copy-test passes 100% before the live binary swap. See §7 step §F.

---

## §5. The Recommended Design — Detailed

### §5.1 New `query_base` shape (pseudocode)

```rust
pub fn query_base(definition: BaseDefinition, library_paths: Vec<(String, String)>)
    -> Result<BaseQueryResult, String>
{
    let conn = open_search_db()?;
    let active_libs: HashSet<String> = if definition.source.selected_libraries.is_empty() {
        library_paths.iter().map(|(n, _)| n.clone()).collect()
    } else {
        definition.source.selected_libraries.iter().cloned().collect()
    };
    let library_path_map: HashMap<String, String> = library_paths.into_iter().collect();

    let (where_sql, where_params) = build_where_clause(&definition, &active_libs)?;
    let (order_sql, _) = build_order_clause(&definition.sorts);

    let sql = format!(
        "SELECT path, name, library_name, modified, properties_json
         FROM note_meta
         WHERE {where_sql}
         {order_sql}"
    );
    // ... execute, materialize Rows, total_count, columns_detected ...
}
```

### §5.2 Filter operator → SQL translation

| Operator | SQL translation |
|---|---|
| `is` | `LOWER(json_extract(properties_json, '$.<prop>')) = LOWER(?)` |
| `is_not` | `LOWER(json_extract(properties_json, '$.<prop>')) IS NOT LOWER(?)` |
| `contains` | `LOWER(json_extract(properties_json, '$.<prop>')) LIKE LOWER('%' \|\| ? \|\| '%')` |
| `not_contains` | `(json_extract(properties_json, '$.<prop>') IS NULL OR LOWER(...) NOT LIKE ...)` |
| `gt` | `CAST(json_extract(properties_json, '$.<prop>') AS REAL) > CAST(? AS REAL)` |
| `lt` | `CAST(json_extract(properties_json, '$.<prop>') AS REAL) < CAST(? AS REAL)` |
| `is_empty` | `json_extract(properties_json, '$.<prop>') IS NULL OR ... = ''` |
| `is_not_empty` | `json_extract(...) IS NOT NULL AND ... <> ''` |

**Special properties** (not in `properties_json`):
- `file_name` → `name` column (with `.md` trimmed in the result mapping)
- `modified` → `modified` column

### §5.3 Source type → SQL translation

| `source.type` | WHERE clause |
|---|---|
| `all` | `library_name IN (<active_libs>)` |
| `folder` | `library_name IN (<active_libs>) AND path LIKE '<library_path>/<folder>/%' [optional include_subfolders exclusion]` |
| `tag` | `library_name IN (<active_libs>) AND (json_extract(tags_json, ...) OR notes_fts MATCH '#<tag>')` |

**Legacy `"vault"` source type** — Phase 1 step §D retires it (Q1). The `parse_base_file` reads it as `"all"` with `selectedLibraries = [the named vault]`; the save path never writes `"vault"` again.

### §5.4 Tag matching via FTS5
Body-tag detection uses `notes_fts MATCH '#<tag>'`. Plan doc §B verifies the exact MATCH syntax for hashtag-prefixed tokens; fallback to `body_text LIKE` if FTS5 hashtag tokenization doesn't work as expected.

### §5.5 Sort translation
| Sort | SQL |
|---|---|
| `file_name` asc/desc | `ORDER BY name COLLATE NOCASE asc/desc` |
| `modified` | `ORDER BY modified asc/desc` |
| user-defined | `ORDER BY json_extract(properties_json, '$.<prop>') COLLATE NOCASE asc/desc` |

Numeric-aware sorting handled via the same numeric-cast attempt the existing `apply_sorts_fixed` uses.

### §5.6 Columns detected via `json_each`
```sql
SELECT DISTINCT key
FROM note_meta, json_each(note_meta.properties_json)
WHERE <same WHERE clause as the main query>
```
Single-query, cheap.

### §5.7 Field-name alignment + legacy retirement (Q1 lock)

Per Eisa's uniqueness-aligned answer to Q1, Phase 1 step §D performs:

1. **Backward-compat read.** `parse_base_file` reads both `selectedVaults` (legacy) and `selectedLibraries` (new) keys; precedence: `selectedLibraries` if both present, else `selectedVaults`.
2. **Modernized write.** `save_base_file`, `save_workspace_base`, and `create_*` paths only write `selectedLibraries`.
3. **Rust struct rename.** `BaseSource.selected_vaults` → `BaseSource.selected_libraries`. Serde annotation updated.
4. **TS type alignment.** `BaseSource.selectedLibraries` already exists in TS — no change.
5. **Legacy `"vault"` source type read-only.** `parse_base_file` translates `source.type = "vault"` to `source.type = "all" AND selectedLibraries = [the value]`. The save path never writes `"vault"`.

The migration is **strictly additive at the file format level** — old `.base` files continue to load. New `.base` files use the modern shape. Rollback: just stop writing the new shape; old files still work.

### §5.8 Cell-edit refresh event (Q2 lock)

Per Eisa's uniqueness-aligned answer to Q2, the 1.5s file-watcher latency window is removed in Phase 1:

```rust
#[tauri::command]
pub fn update_note_property(
    app: tauri::AppHandle,
    file_path: String,
    key: String,
    value: String,
) -> Result<(), String> {
    // ... security check ...
    let new_content = update_frontmatter_property(&content, &key, &value);
    fs::write(&file_path, new_content)?;

    // NEW (Q2): update note_meta.properties_json directly in the same transaction
    update_note_meta_property_immediate(&app, &file_path, &key, &value)?;

    // NEW (Q2): emit refresh event for any open Bases views
    app.emit("bases:note_updated", BasesNoteUpdatedPayload {
        path: file_path.clone(),
        changed_keys: vec![key.clone()],
    })?;
    Ok(())
}
```

Frontend: any Bases view listening for `bases:note_updated` either re-queries (cheap — SQL is instant) or updates the row in-place by patching the known property. The file-watcher's later re-parse becomes a no-op (`content_hash` matches what's already in DB).

**Edge case:** if `update_note_meta_property_immediate` fails (DB locked, etc.), fall through to the existing file-watcher path. Don't fail the user's edit; just lose the instantness for that one edit.

### §5.9 Library path map
Frontend already passes `library_paths: Vec<(String, String)>`. Phase 1 retains this; the new code converts to HashMap, joins each row by library_name. No backend change to libraries.rs needed. The frontend already includes federated cUniverse libraries when federation is opt-in; this continues to work in Phase 1. Phase 4 inverts to opt-out.

---

## §6. What Phase 1 Does NOT Do

Out of scope for MIG-054; defer to later phases per the Concept Paper v1.4 §12 roadmap:

- **Host-Note Assemblage + three threading gestures** (Phase 1.5 / §7). Phase 1.5 is its own MIG and ships after Phase 1 is verified. Open-in-360.3D, Open-in-CNS, and Open-in-Cataloger gestures all land there.
- **Living Link columns** (Phase 2 / §6.1).
- **Cognitive Engine dimensions** (Phase 2.5 / §6.10).
- **CNS network measurements** (Phase 2.6 / §6.11). Freshness strategy decision (α / β / γ) also lives at Phase 2.6.
- **CECE epistemic classifications** (Phase 2.7 / §6.12) — added v1.4 as a separate phase. Source × Content-type, regime state, disambiguation flags. No freshness wrinkle (classifications persisted).
- **NSC headlines as default column** (Phase 3 / §6.2).
- **Federation auto-on** (Phase 4 / §6.6). `selectedLibraries` continues to be the **opt-in** channel in Phase 1; Phase 4 inverts the default.
- **Five Acts templates** (Phase 5 / §8).
- **Cell-edit on typed links** (Phase 7 / §6.9).

**Doc drift items Phase 1 corrects inline:**
- Orientation §4582: "5 commands" → 10 commands.
- TS-Rust drift on `selectedVaults` / `selectedLibraries` — unified per Q1 (§5.7).
- Legacy `"vault"` source type retired per Q1.

---

## §7. Phase 1 Step Outline (high-level — Plan doc has details)

Indicative; the Plan doc breaks each into landable commits with verification clauses.

- **§A — `query_base` SQL skeleton.** Replace the filesystem-walking implementation with a SQL query against `note_meta`. Stub-handle all three source types (folder / tag / all). All eight filter operators. Sort. Behavioral-equivalence harness lands in this step (the snapshot side; the diff side lands in §F).
- **§B — Tag matching via FTS5.** Wire body-tag detection through `notes_fts MATCH '#<tag>'`. Verify against existing tagged universes.
- **§C — `columns_detected` via `json_each`.** Distinct-key extraction.
- **§D — Field-name alignment + legacy retirement (Q1 lock).** `selectedVaults` → `selectedLibraries` in Rust struct + TS types + serde; backward-compat read for old keys; legacy `"vault"` source-type read-only. New `.base` files write modernized shape.
- **§E — Cell-edit refresh event (Q2 lock).** `update_note_property` ALSO updates `note_meta.properties_json` directly + emits `bases:note_updated` Tauri event. Frontend listener re-queries or patches in-place.
- **§F — Behavioral-equivalence test pass on a copy of Eisa's universe DB (Q4 lock).** Snapshot 75 fixtures (Q3 lock structure). Diff every row, every property. Fail loud on any mismatch. **Test on a COPY of the real DB per LL-025**, not on the live one.
- **§G — Performance verification.** `query_time_ms` < 50 on 7,600 notes for all 75 fixtures.
- **§H — 3-agent audit** (`/migration` Phase 4): invariants / drift / migration-path.
- **§I — Boss-test gate.** Eisa opens his existing Bases views on the live universe (post-copy-test). Reports any anomaly. Returns < 50ms; same rows as before.
- **§J — PCS + Orientation v2.36** + help-doc note (existing Bases help doc gains a one-line "now instant" note + a brief "cell edits are now instant" note for Q2; no UX walkthrough change).

---

## §8. Risks

1. **Behavioral mismatch on frontmatter edge cases.** Inline lists, multi-line lists, quoted scalars may serialize differently in `properties_json` vs the on-disk file. **Mitigation:** §A's behavioral-equivalence harness catches loudly. **Severity:** Medium.
2. **`note_meta.properties_json` not populated for some notes.** A note indexed before frontmatter parsing existed might have `'{}'` even though the file has frontmatter. **Mitigation:** verify on Eisa's universe; if non-zero, one-shot re-parse on those rows. **Severity:** Low.
3. **FTS5 hashtag tokenization.** Plan doc §B verifies; fallback to `body_text LIKE` if MATCH doesn't work for hashtags. **Severity:** Low.
4. **`update_note_property` immediate-write conflict with file-watcher.** Both could try to update `note_meta.properties_json` for the same path. **Mitigation:** the file-watcher's re-parse is idempotent (content_hash matches → no-op). **Severity:** Low.
5. **`library_path` join misses for cUniverse children.** Verify in §A. **Severity:** Low.
6. **Backward-compat read of `selectedVaults`** (Q1 implication). An old `.base` file with both `selectedVaults` AND `selectedLibraries` keys should pick `selectedLibraries`; an old file with only `selectedVaults` should map cleanly. **Mitigation:** explicit precedence logic in `parse_base_file`. **Severity:** Low.

---

## §9. Rollback Strategy

Phase 1 is **strictly additive at the data layer** — no schema change, no migration, no destructive operation. Rollback is a one-line revert of `bases.rs::query_base` back to the filesystem-scanning version. The disk filesystem and `.base` files are untouched.

The Q1 field-name rename (§5.7) is **backward-compat by design** — old files keep working; new files use the new shape; full rollback would require reading the new shape back, but a rollback window is built in by accepting both names on read.

The Q2 cell-edit event (§5.8) is purely additive — if the event isn't emitted (rollback), the old file-watcher path still works.

---

## §10. Decisions Locked 2026-05-25

Replacing the original v1.0 §10 Open Questions — Eisa's uniqueness-aligned answers.

| # | Question | Resolution | Where folded |
|---|---|---|---|
| 1 | Field-name alignment (selectedVaults → selectedLibraries) — in scope for Phase 1, or deferred? | **Fold into Phase 1 step §D. Also retire the legacy `"vault"` source type at the same time.** | §2.5, §5.3, §5.7, §7 step §D |
| 2 | `update_note_property` latency window — accept or fix? | **Fix. Wire explicit refresh event + immediate `note_meta` write. Dashboard effect doesn't tolerate a 1.5s window.** | §2.4, §5.8, §7 step §E |
| 3 | Behavioral-equivalence fixture set | **Design as foundation suite for Phase 2/2.5/2.6/2.7 — 75 fixtures across source-type, filter, sort, federation, multilingual, edge-case, and ~10 of Eisa's real Bases.** | §4.E |
| 4 | Boss-test Stage 1 universe | **Cognitive Knowledge universe, with LL-025 copy-test pattern as safety gate.** Copy-test passes 100% before live binary swap. | §4.E, §7 steps §F + §I |
| 5 | MIG number reuse (049 vs 054) | **Jump to MIG-054. Clean break from the reverted-Mind allocation. Audit-trail clarity > minor file-rename cost.** | filename, frontmatter, orientation v2.35 inline-fix |

**Ask still owed:** ~10 representative `.base` files from Eisa's Cognitive Knowledge universe to seed the §4.E fixture gold-master tier.

---

## §11. Predecessor and Adjacent Documents

- **Predecessor (vision):** `docs/Constellation-Base-Concept-Paper-v1.4.md` — Phase 1 is the first ship in the roadmap §12.
- **Predecessor (MVP):** `docs/BASES_MVP_SPEC.md` (commit `c5b05f5c`, 2026-03-12).
- **Adjacent — schema source of truth:** `src-tauri/src/search.rs` (note_meta + triggers + FTS5).
- **Adjacent — implementation target:** `src-tauri/src/bases.rs`.
- **Adjacent — TS types:** `src/lib/bases/types.ts`.
- **Adjacent — companion deep-read surfaces (out of scope for Phase 1; relevant for Phase 1.5+):** `src-tauri/src/inspector360.rs` (360.3D), `src-tauri/src/sight.rs` (CNS — internal v2 name preserved), `src-tauri/src/cece/` (The Cataloger).
- **Successor:** Plan doc — phase-decompose the chosen design into landable commits with verification clauses.

---

## §12. Closing

This Architect doc has **one architecturally consequential finding**: `note_meta.properties_json` already maintains frontmatter at write time, so Phase 1 is a query-rewrite, not a cache-build. The Concept Paper §10.1 mandate is satisfied; the "or equivalent" language in that mandate is what unlocks it.

Plus three uniqueness-aligned adjustments from Eisa's Q1-Q5 locks: legacy `"vault"` retirement (Q1 + Q5), instant cell edits (Q2), foundation-suite fixtures (Q3 + Q4).

Smaller deliverable, lower risk, faster ship, full Concept Paper alignment.

Awaiting Eisa's approval before drafting the Plan doc.

---

*End of MIG-054 Architect doc v1.1. To be updated only on substantive change of design; otherwise the Plan doc is the next document in this MIG.*
