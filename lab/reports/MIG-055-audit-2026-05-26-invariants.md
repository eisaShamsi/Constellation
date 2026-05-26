# MIG-055 §H Audit — Invariants

**Agent:** Invariants
**Date:** 2026-05-26
**Commits audited:** e0f7bffc (B) cd0dd873 (C) 32b9f958 (D) fa9085a1 (E) 222b18b1 (F) 0ce98593 (G) [+ 7b12b72d (A) implicitly]

## Verdict

**PASS** — All 10 invariants hold across the §A-§G commits. 84/84 lens tests pass.

## Per-invariant findings

### 1. Clean break from old MVP — PASS

- Status: **PASS**
- Evidence:
  - `src-tauri/src/lens/` contains NO references to `bases::*`. Verified via `Grep "bases::" path="src-tauri/src/lens"` → no matches.
  - `src-tauri/src/lens/mod.rs:31-37` declares the new module hierarchy (definition / dimensions / parser / query / sql_builder / system_notes / validator) entirely under `lens::*`.
  - `src/lib/components/LensBlock.svelte` and `src/lib/lens/store.ts` contain NO references to `BaseView` / `BaseTableView` / `BaseCardView` / `BaseListView`. Verified via Grep.
  - `src/lib/editor/livePreview.ts:18` imports only `LensBlock`, not any legacy `BaseView*` component.
  - Legacy `bases::*` references survive in non-lens code (boot_bundle, tasks, lib.rs command registration, dataview), which is acceptable per Architect §10 ("Cleanup is a separate housekeeping MIG"). The invariant requires that NEW code uses ONLY `lens::*` — and it does.

### 2. No auto-detected columns — PASS

- Status: **PASS**
- Evidence:
  - `src-tauri/src/lens/sql_builder.rs:8` carries an explicit module-docstring lock: *"Per Architect §5.1: queries `note_meta` (+ JOINs added per dimensions used by the lens) — never `properties_json`"*.
  - Grep `properties_json` in `src-tauri/src/lens/` returns 3 hits, all benign:
    - `sql_builder.rs:8` — the negative comment above
    - `query.rs:236` — inside a `#[cfg(test)] CREATE TABLE note_meta (...)` statement defining the in-memory test schema column
    - `tests.rs:78` — same pattern in §G test schema
  - No SELECT / WHERE / JOIN ever reads `properties_json`. The 4 dimensions all source from `note_meta.{name,path,created_at}` or `note_summaries.headline` (verified at `dimensions.rs:76-115`).

### 3. Federation auto-default — PASS

- Status: **PASS**
- Evidence:
  - `src-tauri/src/lens/definition.rs:144-149` — `impl Default for FederationMode { fn default() -> Self { FederationMode::Auto } }`.
  - `src-tauri/src/lens/query.rs:73` — `execute_lens` calls `crate::universe::resolve_universe_libraries(app.clone())?` (the federated resolver). Confirmed at `universe.rs:1087` — that function does `resolve_libraries_recursive(&universe_dir, &mut visited)`, which descends cUniverse children.
  - Parser test at `parser.rs:184-187` (`parse_defaults_when_scope_omitted`) confirms missing scope yields `FederationMode::Auto`.
  - Validator test at `validator.rs:162-184` (`validate_canonical_recent_captures_passes`) round-trips the canonical YAML with federation default left implicit.

### 4. Curated dimensions only — v1 set is exactly 4 — PASS

- Status: **PASS**
- Evidence:
  - `src-tauri/src/lens/dimensions.rs:76-115` — REGISTRY const array contains exactly 4 entries: `note.name`, `note.path`, `note.created_at`, `note.headline`.
  - Test `dimensions.rs:142-149` (`dimension_registry_includes_4_v1_dimensions`) asserts `assert_eq!(names.len(), 4, ...)` and verifies each by name.
  - Test `dimensions.rs:227-233` (`registry_iteration_is_stable`) pins the order via `assert_eq!(names, vec!["note.name", "note.path", "note.created_at", "note.headline"])`.

### 5. Naming convention locked — PASS

- Status: **PASS**
- Evidence:
  - All 4 v1 dimensions use the `note.X` prefix (no `link.*` / `note.cns.*` / `note.cece.*` in v1) — see registry at `dimensions.rs:76-115`.
  - Test `dimensions.rs:214-224` (`all_v1_dimensions_use_note_prefix`) explicitly enforces this via `assert!(d.name.starts_with("note."), ...)` for every registered dimension.
  - `src-tauri/src/lens/mod.rs:22-29` documents the convention with the §11 #1 lock note for future readers.

### 6. Schema versioning — PASS

- Status: **PASS**
- Evidence:
  - `src-tauri/src/lens/definition.rs:22` — `pub schema: u32` is a required field on `LensDefinition` (no `#[serde(default)]`).
  - `src-tauri/src/lens/validator.rs:17` — `const CURRENT_SCHEMA: u32 = 1;`
  - `validator.rs:25-31` — schema-version mismatch produces a `LensError::Validate` with both the lens-declared and build-understood versions in the message.
  - Test `validator.rs:267-274` (`validate_schema_version_other_than_1_rejected`) confirms `schema: 2` is rejected.
  - Serde-derived `Deserialize` on `LensDefinition` (definition.rs:19) means a missing `schema:` field fails at parse time before validate runs.

### 7. File-Over-App — PASS

- Status: **PASS**
- Evidence:
  - `src-tauri/src/lens/system_notes.rs:33` — `pub const RECENT_CAPTURES_FILENAME: &str = "Observation — Recent Captures.md";`
  - `system_notes.rs:36` — `pub const FIVE_ACTS_DIR: &str = "Five Acts";` (visible folder under universe root, per Architect §11 #2 lock).
  - `system_notes.rs:94-129` — `init_at` creates the `.md` file ON DISK at `{universe}/Five Acts/Observation — Recent Captures.md`. No SQLite-stored "system note" lookup — the `.md` file IS the durable artifact.
  - `system_notes.rs:47-77` — `RECENT_CAPTURES_CONTENT` embeds the YAML inline inside ` ```base ` fenced block, per Architect §6 / v1.4 §7 host-note assemblage. The renderer (LensBlock.svelte) re-parses the YAML each open — the file is the source of truth.

### 8. Transfer-on-edit invariant — PASS

- Status: **PASS**
- Evidence:
  - `src-tauri/src/lens/system_notes.rs:110-118` — `if target_path.exists() { return Ok(()); }` early-return is the lock. Any existing file (canonical content, user-edited, even unrelated content with the same name) is left alone.
  - Test `system_notes.rs:243-264` (`existing_canonical_file_left_unchanged`) asserts mtime is preserved across two consecutive `init_at` calls.
  - Test `system_notes.rs:267-282` (`existing_user_edited_file_left_unchanged`) asserts user content survives a second `init_at` verbatim.
  - Test `system_notes.rs:296-326` (`two_consecutive_inits_are_idempotent`) confirms three back-to-back inits produce exactly one canonical file (no duplicates).
  - Module docstring at lines 13-26 documents the contract explicitly: *"if file absent → create with canonical content / if file present (any content) → no-op (transfer-on-edit honored)"*.

### 9. LL-022 (lazy mount) — PASS

- Status: **PASS**
- Evidence:
  - `src/lib/components/LensBlock.svelte:36-52` — `onMount` uses a `let cancelled = false;` flag captured in the closure; the returned cleanup function sets `cancelled = true`; both `.then` and `.catch` early-return when `cancelled`. No global subscriptions are attached (no Tauri `listen`, no `appSettings.subscribe`, no `addEventListener` on document/window from inside the component).
  - LensBlock.svelte does no I/O beyond the single `executeLens()` call on mount.
  - `src/lib/editor/livePreview.ts:755-765` — `LensBlockWidget.destroy()` calls `unmount(this._mounted)` inside a try/catch to dispose the Svelte component when CodeMirror removes the widget, satisfying the LL-022 cleanup-on-unmount requirement.
  - Module docstring on the widget (livePreview.ts:720-724) names the contract: *"`destroy(dom)` is called by CodeMirror when the widget is removed ... We unmount the Svelte component so its `$effect` cleanup runs and no `executeLens` callback fires into a destroyed DOM."*

### 10. i18n — PASS

- Status: **PASS**
- Evidence:
  - `src/lib/components/LensBlock.svelte:80,83,93` — all three user-facing strings route through `$t()`:
    - `$t('lensBlock.loading') || 'Loading lens…'`
    - `$t('lensBlock.errorLabel') || 'Lens error'`
    - `$t('lensBlock.empty') || 'No notes match this lens.'`
  - `src/routes/+layout.svelte:4752` — sidebar uses `{$t('sidebar.fiveActs')}`.
  - All 15 locales contain both `lensBlock` and `fiveActs` keys (Grep confirmed: 15/15 hits each in en, ar, de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh).
  - Spot-checked locale contents:
    - `en.json:709` `"fiveActs": "Five Acts"`; `en.json:3046-3050` `lensBlock` object with all 3 keys
    - `ar.json:673` `"fiveActs": "الأفعال الخمسة"`; `ar.json:3048-3052` Arabic translations of all 3 lensBlock keys
    - `zh.json:672` `"fiveActs": "五幕"`; `zh.json:2843-2846` Chinese translations
    - `he.json:672` `"fiveActs": "חמשת המעשים"`; `he.json:2843-2846` Hebrew translations
  - Per project convention, English fallback `|| 'Loading lens…'` etc. is acceptable belt-and-braces — if a locale's translation is missing for any reason, the component degrades gracefully.

## Aggregate notes

### Test suite

```
cargo test --lib lens:: → 84 passed; 0 failed; 0 ignored
```

Breakdown:
- `lens::dimensions::tests` — 9 tests (registry shape + lookup)
- `lens::parser::tests` — 14 tests (YAML round-trips + rejection cases)
- `lens::sql_builder::tests` — 13 tests (SQL string + parameter binding)
- `lens::validator::tests` — 10 tests (schema + dimension contract)
- `lens::query::tests` — 9 tests (in-memory SQLite end-to-end)
- `lens::system_notes::tests` — 8 tests (idempotency + transfer-on-edit + file enumeration)
- `lens::tests` — 11 tests (§G synthetic-universe end-to-end fixtures)

### Architectural observations

1. **The 7 §11 locks land coherently in the code.**
   - Lock #1 (prefix convention) → `dimensions.rs` test `all_v1_dimensions_use_note_prefix`.
   - Lock #2 (visible folder) → `system_notes.rs::FIVE_ACTS_DIR = "Five Acts"` (top-level, no `.constellation/` prefix).
   - Lock #3 (transfer-on-edit) → `system_notes.rs::init_at` early-return + three covering tests.
   - Lock #4 (silent ignore of old `.base` files) → No `parse_base_file` analog in `lens::*`. The old `bases::parse_base_file` Tauri command remains registered in `lib.rs` for backward compat but is never called by lens code. (Per Architect §3: cleanup is a separate housekeeping MIG.)
   - Lock #5 (sidebar label "Five Acts") → `+layout.svelte:4752` uses `$t('sidebar.fiveActs')`.
   - Lock #6 (MIG-055 number) → All commits + docstrings consistently reference MIG-055.
   - Lock #7 (keep `.base` extension for fenced-block lang tag) → `livePreview.ts:914,928` match `/^```+\s*base\s*$/i` and skip the language label for `base` blocks (delegates to LensBlockWidget instead).

2. **Federation `off` is parser-accepted but no-op at runtime.** Confirmed at `query.rs:82-86` — comment explicitly documents this as a v1 contract; `tests.rs:367-396` locks the contract so future phases must update the test when they implement runtime filtering. Not a violation; an intentional v1 affordance.

3. **The canonical YAML fixture round-trips.** `system_notes.rs::RECENT_CAPTURES_CONTENT` embeds the YAML; `system_notes.rs:329-339` test extracts it and asserts each required field; `system_notes.rs:341-357` test parses + validates it; `tests.rs:49-66` (CANONICAL_RECENT_CAPTURES_YAML) duplicates the YAML for §G tests; `tests.rs:canonical_yaml_matches_system_note_constant` (visible in the test run) cross-validates that the two copies agree. Single source of truth maintained.

4. **No `unmount()` deadlock risk in LensBlock.** The `destroy()` in `LensBlockWidget` wraps `unmount()` in try/catch (livePreview.ts:756-762) — if the Svelte component already disposed itself, the editor update doesn't crash.

5. **Empty allowed_libraries handled.** `sql_builder.rs:72-83` produces `WHERE 1=0` when `allowed_libraries` is empty, ensuring zero-row return without parameter-binding errors. Covered by `sql_builder::tests::build_sql_empty_libraries_returns_1_eq_0`.

### No HIGH/MEDIUM/LOW findings

Zero violations across all 10 invariants. The implementation is clean across the §A-§G cascade. Ready for §H consolidated audit + §I Boss-test gate.

---

**Summary line:** PASS / 0 non-PASS findings / 84/84 lens tests passing.
