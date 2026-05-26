# MIG-055 §H Audit — Drift

**Agent:** Drift
**Date:** 2026-05-26
**Commits audited:** 7b12b72d (§A) through 0ce98593 (§G)

## Verdict
**PASS-WITH-NOTES**

The core SQL-injection / panic / memory / dependency surfaces are clean. No P1 findings. The notes below are P2 cross-cutting concerns inherited from existing architectural patterns plus a small P3 doc-drift item against the localization top-principal.

## Findings

### P1 (must fix before §I Boss-test)

**None.**

### P2 (should fix before §J PCS)

#### P2-1 — `init_five_acts_system_notes` does not re-fire on mid-session universe switch
- **Where:** `src-tauri/src/search.rs:4771-4826` (`ensure_search_db_ready`) and `src-tauri/src/universe.rs:546-721` (`set_active_universe`).
- **Issue:** `ensure_search_db_ready` early-returns at line 4775-4777 when `state.db.is_some()`, which means `init_five_acts_system_notes` (line 4821) only fires on the FIRST `ensure_search_db_ready` call after process start. `set_active_universe` does not reset `SearchState::db`, so when the user switches universes mid-session the Five Acts system note is NOT auto-created in the newly-active universe.
- **Impact:** A user who creates a new universe via `set_active_universe` (after the process already booted into a different universe) won't see the Five Acts sidebar populated for that universe until the app is restarted. The `listFiveActsNotes()` IPC reads `{universe}/Five Acts/` directly so it will return an empty array (the directory simply isn't there yet).
- **Severity rationale:** P2 because (a) it inherits a pre-existing architectural pattern shared by every consumer of `ensure_search_db_ready` (not a MIG-055 regression), (b) the first-boot path works correctly, (c) a process restart heals it, (d) the Boss-test in §I is likely to start from a fresh boot. But the §J PCS should at least document this in the help-file entry, ideally fix it by either calling `init_five_acts_system_notes` from `set_active_universe` directly OR clearing `state.db` on universe switch.
- **Suggested fix:** Add a direct call to `init_five_acts_system_notes` from `set_active_universe` after `lock = Some(final_path.clone());` (line 673). Five Acts init is idempotent + cheap (one filesystem stat); calling it twice on a fresh boot is harmless.

#### P2-2 — Federated-lens row click may not open notes from cUniverse libraries
- **Where:** `src/lib/components/LensBlock.svelte:54-67` (dispatches `constellation:open-note` with `libraryName` + `libraryPath` in detail) and `src/routes/+layout.svelte:2196-2202` (listener).
- **Issue:** LensBlock provides `libraryName` and `libraryPath` in the event detail, but the layout-shell listener ignores both fields and derives the library by `libs.find(l => detail.path!.startsWith(l.path))`. The `libs` store contains only the active universe's libraries (not cUniverse children), so a row from a federated cUniverse library will not match any `lib` and `openNoteTab` will not be called — the click silently does nothing.
- **Impact:** In v1 the canonical Recent Captures lens uses `federation: auto`, so cUniverse children DO populate rows; the user clicks a row from a federated note → nothing happens. The note shows in the list but doesn't open.
- **Severity rationale:** P2 because (a) it only affects users with cUniverses configured (a minority case for §I Boss-test), (b) clicking a non-federated row works fine, (c) the dispatch payload is already correct — just the listener needs to consult `libraryPath`/`libraryName` for fallback.
- **Suggested fix:** In `+layout.svelte:2196-2202`, when the path-startswith lookup misses, fall back to `detail.libraryPath` / `detail.libraryName` (provided by LensBlock). Keeps the non-federated path unchanged.

#### P2-3 — `lens::LensError` re-export is in `mod.rs:51` but never used by frontend bridge
- **Where:** `src-tauri/src/lens/mod.rs:51` (`pub use parser::{LensError, parse_lens_yaml}`) — TS bridge in `src/lib/lens/store.ts` does not consume `LensError` directly; errors from `execute_lens` round-trip as plain `String` via `Result<LensResult, String>`.
- **Issue:** The `LensError` re-export adds API surface that's not consumed externally; the only callers of `parse_lens_yaml` are inside `lens::query::execute_lens` (line 67) and the `#[cfg(test)]` modules.
- **Impact:** None at runtime — `pub use` of unused types is a benign API surface increase. Future maintainers might assume LensError is part of the public contract and break it.
- **Severity rationale:** P2 only because future-proofing matters; not blocking §I.
- **Suggested fix:** Either (a) keep as-is (the re-export documents what the lens module exposes for potential future external callers, in line with the §G `lens::tests::canonical_yaml_matches_system_note_constant` drift catch), or (b) make it `pub(crate) use` if no external consumer is planned. Decision is a style call.

#### P2-4 — `_ = def;` suppression in `execute_query` is a TODO placeholder
- **Where:** `src-tauri/src/lens/query.rs:169-170`.
- **Issue:** The comment `// Suppress unused warning until §D uses def for diagnostics` is a deliberate placeholder. The `def` parameter is passed in but never consulted in the materialization loop. Either `def` should be used (e.g., for a `template`-aware diagnostic header) or removed from the signature.
- **Impact:** Cognitive overhead reading the function ("why is `def` passed?"). No runtime cost.
- **Severity rationale:** P2 because it's a known short-term debt; either honor the TODO in §I or drop the parameter.

### P3 (nice-to-have for future MIG)

#### P3-1 — `FIVE_ACTS_DIR` and `RECENT_CAPTURES_FILENAME` are English-only on disk
- **Where:** `src-tauri/src/lens/system_notes.rs:33-36`.
- **Issue:** The Architect §11 #2 lock chose "Visible folder" + §11 #5 chose "Five Acts" as the sidebar label. The on-disk folder name `"Five Acts"` and file stem `"Observation — Recent Captures"` are hard-coded English strings. CLAUDE.md memory `feedback_full_localization_everything` (2026-05-18, TOP PRINCIPAL STANDING ORDER) says "when user switches language, EVERYTHING adapts — chip names, dome details, stratum labels, sector labels, every string" and "overrules §A.15 'brand names stay English' where they conflict".
- **Impact:** Arabic / Persian / Hebrew / etc. users see English folder + filename in their file explorer + sidebar (the sidebar `display_name` is the filename stem). The frontend `sidebar.fiveActs` i18n key (line 4752 of `+layout.svelte`) localizes the section heading but the file underneath stays English.
- **Severity rationale:** P3 because (a) Architect §11 #5 explicitly locked "Five Acts" as the sidebar label across all locales (treating it as a cognitive brand), (b) the §11 #2 lock is recent + intentional, (c) renaming the on-disk folder per-locale would break the transfer-on-edit invariant (renames on language switch are file mutations the user didn't request), (d) the in-app sidebar heading IS localized correctly. This is a design tension between two top-principals and warrants a Boss decision before any future change. **Recommend NOT changing in MIG-055** — surface it as a deferred decision.
- **Suggested action:** Either close as "intentional per Architect §11 lock" or open a follow-up PJ asking the Boss whether "Five Acts" should localize at the filesystem level.

#### P3-2 — `current_unix_seconds()` swallows `SystemTime::duration_since` errors silently
- **Where:** `src-tauri/src/lens/sql_builder.rs:276-281`.
- **Issue:** The fallback `unwrap_or(0)` returns Unix epoch (1970-01-01) on `SystemTime` failure (could happen if system clock is set before 1970). Lens filters like `now - 14 days` would then evaluate to ~1970-01-01, returning zero rows on a "recent" lens silently.
- **Impact:** Near-zero in practice. Modern systems do not have pre-1970 clocks. But the silent fallback violates a Working Agreement (#4) lesson about proven-safe failures.
- **Severity rationale:** P3 because the failure mode requires a broken system clock — but documenting it explicitly with a log line costs nothing.
- **Suggested fix (deferrable):** Replace `unwrap_or(0)` with an `eprintln!`-on-error fallback, or return `Err` from `parse_time_value` if `current_unix_seconds()` fails.

#### P3-3 — `replace('\\', '/')` only fires once for path normalization
- **Where:** `src-tauri/src/lens/system_notes.rs:161` (`rel.to_string_lossy().replace('\\', "/")`).
- **Issue:** Mac/Linux paths never contain backslashes, but Windows paths do. The single-pass replace is correct, but it's a magic-coupling assumption (Windows-specific) without a comment.
- **Severity rationale:** P3 documentation/clarity only.
- **Suggested action:** Add a `// Normalize Windows path separators for frontend consumption` comment, or use `std::path::MAIN_SEPARATOR` explicit handling.

## Aggregate notes

### What I verified explicitly

1. **SQL injection paths — CLEAN.** All `format!` calls in `sql_builder.rs` consume only static-registry strings (`dim.sql_expression`, `dim.requires_join`). User-controlled `col.dimension` / `filter.dimension` / `filter.value` / `filter.op` / `allowed_libraries` are never interpolated into SQL — only used as registry lookup keys (where unknown values are rejected at validation time) or bound to `?` placeholders. The library-list `IN (?, ?, ?)` placeholder construction at `sql_builder.rs:75-83` is sound. The `parse_time_value` path correctly converts user time strings → integers BEFORE binding. Verified by re-reading every `format!` in `sql_builder.rs:62,76,100,126,132,149,157,166,174,184,187,192,204,219,223,248,252,256,263,273`.

2. **Panic paths in production code — CLEAN.** Every `unwrap()` / `expect()` / `panic!` in `src-tauri/src/lens/` is inside a `#[cfg(test)]` module. The grep confirmed: zero panics in production paths. Production code uses `unwrap_or_default` / `unwrap_or(DimensionValue::Null)` / `ok_or_else(|| format!(...))` for graceful degradation.

3. **Error message hygiene — CLEAN with note.** `query.rs:124` includes the full SQL string in error messages ("Failed to prepare lens SQL: ...\nSQL: {}"). This leaks internal SQL structure to the frontend on error. For v1 the SQL is constructed from registry values only (not user secrets), so the leak is benign — but if user-supplied LIBRARY NAMES are ever sensitive, the SQL would include them. Note for §I documentation but not flagging as P1/P2 — current attack surface is empty.

4. **Dimension-registry leaks — CLEAN.** `REGISTRY: &[DimensionDef]` is a `const` array (line 76 of `dimensions.rs`). `lookup_dimension` is the only entrypoint. `all_dimensions()` is `#[allow(dead_code)]` and returns the same `&'static` slice. No runtime mutation paths exist. Exactly 4 dimensions registered; `dimensions::tests::dimension_registry_includes_4_v1_dimensions` pins this.

5. **Tauri command registrations — CLEAN.** `src-tauri/src/lib.rs:496` registers `lens::query::execute_lens`; `src-tauri/src/lib.rs:500` registers `lens::system_notes::list_five_acts_notes`. The comment at lines 489-495 correctly explains why the full module path is used (Tauri macro resolves `__cmd__` at definition site, not via re-export). No missing registrations.

6. **Re-export consistency — CLEAN.** `src-tauri/src/lens/mod.rs:44-53` re-exports `LensDefinition`, `LensColumn`, `LensFilter`, `LensSort`, `LensView`, `LibrariesSelector`, `FederationMode`, `SortDirection`, `DimensionDef`, `DimensionKind`, `all_dimensions`, `dimension_names`, `lookup_dimension`, `LensError`, `parse_lens_yaml`, `execute_lens`, `DimensionValue`, `LensResult`, `LensRow`, `validate`. Frontend `src/lib/lens/store.ts` mirrors `LensRow`, `LensResult`, `DimensionValue`, `FiveActsNoteEntry` — all consistent.

7. **Cargo.toml dependencies — CLEAN.** `serde_yaml = "0.9"` is at `src-tauri/Cargo.toml:38-39` with a MIG-055 §B annotation. No version conflicts; existing deps cover the rest.

8. **Hidden guards / undocumented constants — minor.** Two constants in `system_notes.rs:33-36` (`RECENT_CAPTURES_FILENAME`, `FIVE_ACTS_DIR`) — both `pub const &str`, documented in module comments. The `livePreview.ts::LensBlockWidget` carries no hidden magic — its `eq()` uses YAML string equality (the right invariant for widget reuse). The cursor-line check `!cursorInBlock` at line 900/914 reads exclusively from `firstLine.text.trim()` + `cursorInBlock` (a local computed at line 812), no async/IPC.

9. **Memory leaks (frontend) — CLEAN.** `LensBlockWidget.destroy()` (livePreview.ts:755-765) calls `unmount(this._mounted)` and nulls out the reference. The `try/catch` around `unmount` is a defensive shield for the destroyed-tree case — correct pattern. `LensBlock.svelte:36-52` returns a `cancelled` flag from `onMount` so an in-flight `executeLens` promise can't `result =` a destroyed component. No `setTimeout`/`setInterval` in either component; no `addEventListener` adds (event listening is via the layout shell). Verified clean.

10. **CLAUDE.md Rule conformance:**
    - **Rule 1 (instant keystrokes):** FencedCode handler at `livePreview.ts:896-935` is read-only — just regex-matches the info string, slices the YAML text out, builds a widget object. No IPC, no parsing, no async. The `update()` function at `livePreview.ts:1131-1183` has the line-change guard (line 1162) AND the 300ms debounce + fast-path `decorations.map()` (lines 1170-1182). Conforms.
    - **Rule 2 (no $effect loops):** `LensBlock.svelte` uses `onMount` exclusively. Zero `$effect` blocks. The `$state` writes are guarded by `if (cancelled) return;`. Conforms.
    - **Rule 3 (no heavy work main thread):** `execute_lens` is a Rust IPC call (correct). Rust does parse + validate + SQL + materialization + returns serialized JSON. Frontend only renders. Conforms.
    - **Rule 4 (no memory leaks):** Verified in finding #9 above. Conforms.

### Test summary
`cargo test --lib lens::` — 84/84 PASS in 0.02s. No warnings introduced by MIG-055 (`cargo build --lib` shows 42 pre-existing warnings, none in `lens/`).

### Cross-reference check
- `system_notes::RECENT_CAPTURES_CONTENT` matches `tests::CANONICAL_RECENT_CAPTURES_YAML` (verified by `lens::tests::canonical_yaml_matches_system_note_constant`).
- The 4 v1 dimensions in `dimensions.rs:76-115` match Architect §4.1 / Plan §A exactly.
- The §11 #1 lock (prefix convention) is honored: every v1 dimension uses `note.X` (pinned by `dimensions::tests::all_v1_dimensions_use_note_prefix`).
- The §11 #3 lock (transfer-on-edit) is honored: `system_notes::init_at` checks `target_path.exists()` and no-ops when present, regardless of content (verified by `system_notes::tests::existing_user_edited_file_left_unchanged`).
- The §11 #2 lock (visible folder) is honored: `FIVE_ACTS_DIR = "Five Acts"` directly under universe root, not `.constellation/`.

### Closed checks (non-findings)
- The "P3" magic-number `300` in `livePreview.ts:1181` is the 300ms debounce window, a known top-principal from CLAUDE.md §Performance Rule 3. Not a drift.
- The "P3" silent failure of `listFiveActsNotes` via `.catch(() => {})` in `+layout.svelte:1955` is a deliberate fire-and-forget pattern documented in the comment ("Not yet part of the bundle IPC ... fire-and-forget here keeps boot time unchanged"). Not a drift.
- The "P3" suppression `let _ = def;` — duplicated as P2-4 above since the upstream comment names it a deliberate TODO.
- The `unwrap_or_default()` chain in `system_notes.rs:201` (the filename-to-string conversion) is the standard pattern for non-fatal filesystem path handling on Windows where `Path::file_name().to_str()` can return None on invalid Unicode. Correct.
