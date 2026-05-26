# MIG-055 §H Audit — Migration Paths

**Agent:** Migration-Path
**Date:** 2026-05-26
**Commits audited:** 7b12b72d (§A) through 0ce98593 (§G)

## Verdict

**PASS-WITH-NOTES** — 9 scenarios PASS at the code-path level, 1 scenario (Scenario 10 — cross-universe switch) has a **P1 finding** that the existing `ensure_search_db_ready` early-return shape inherited from pre-MIG-055 means MIG-055's `init_five_acts_system_notes` will NOT run when a universe is switched within the same app session. The user must restart the app for the new universe's Five Acts system note to be created. 1 scenario (Scenario 1 — fresh universe) has a **P3 boot-race note** worth surfacing.

The 11 invariant-level test cases in §E/§G all reflect the file-level invariants correctly; the issues found are at the boot orchestration / IPC scheduling layer, not in the lens module itself.

## Per-scenario findings

### Scenario 1 — Fresh universe (clean slate)

**Status:** PASS-WITH-NOTES (P3 boot race noted; needs §I confirmation)

**Trace:**
1. User creates a new universe via Universe Manager → `set_active_universe` updates `UniverseState.active_path`.
2. Frontend calls `handleUniverseCreated` → `initializeApp()` (`src/routes/+layout.svelte:2084`).
3. `initializeApp()` fires `constellation_boot_bundle` (line 1891). The bundle does NOT call `ensure_search_db_ready` — confirmed by reading `src-tauri/src/boot_bundle.rs:57–137`.
4. After bundle resolves, `listFiveActsNotes()` fires fire-and-forget at line 1955 — reads `{universe}/Five Acts/` via `fs::read_dir`. On a brand-new universe, the directory doesn't exist yet, so `list_five_acts_notes_at` returns `Vec::new()` (system_notes.rs:177–179).
5. After bundle handler completes, `refreshLibraryCaches()` awaits (line 2032) → `cache_boot_snapshot_core` (cache.rs:166) → `ensure_search_db_ready` (cache.rs:179) → `init_five_acts_system_notes` (search.rs:4821). System note is created at this point.
6. **Race:** steps 4 and 5 are concurrent at the Tauri command boundary. If `listFiveActsNotes` finishes before `init_five_acts_system_notes` (highly likely since list is one stat call, init runs full `init_db` first), the sidebar reactive `fiveActsNotes` array gets populated with `[]`.

**Edge case:** On a fresh universe, the sidebar's "Five Acts" section may not appear after first paint because `fiveActsNotes` was loaded BEFORE the system note got written to disk. Mitigation in code: the section uses `{#if fiveActsNotes.length > 0}` guard, so the section is hidden gracefully (no broken UI). However, the user will not see "Observation — Recent Captures" in the sidebar until they trigger a re-list (no automatic re-list path exists post-init).

**Recovery path:** On the next app restart, `ensure_search_db_ready` fires before any boot bundle call, the system note already exists from the prior session, and `listFiveActsNotes` returns it. So first restart fixes it.

**Recommendation:** §I Boss-test should confirm whether the user sees the Five Acts entry on first paint of a brand-new universe. If not, fix candidates:
- (a) Move `listFiveActsNotes()` invocation to AFTER `refreshLibraryCaches()` awaits.
- (b) Emit a Tauri event from `init_five_acts_system_notes` that the frontend listens for to reload.
- (c) Hoist `init_five_acts_system_notes` to a non-search-state-gated path that always runs at boot.

### Scenario 2 — Existing universe (no Five Acts folder)

**Status:** PASS

**Trace:** Identical to Scenario 1 but with existing notes. Pre-MIG-055 universe has a populated `note_meta` table and existing notes on disk. `ensure_search_db_ready` triggers from `cache_boot_snapshot_core`. Since `state.db` is `None` (cold app start), the full init runs: `init_db` opens the existing DB (no-op since schema present), then `init_five_acts_system_notes` runs — creates `{universe}/Five Acts/` directory + writes the canonical "Observation — Recent Captures.md". Existing notes/folders are untouched (the function only touches `Five Acts/`). On the next `listFiveActsNotes` call (or next boot), the sidebar gains the new "Five Acts" section ABOVE the existing "Workspace Bases" section. The layout enforces this ordering at `+layout.svelte:4738–4806` (Five Acts block then Workspace Bases block). PASS.

Boot race from Scenario 1 may apply here as well — on a cold app start opening this universe for the first time, the same race exists. However, the `Five Acts/` folder is created by `init_five_acts_system_notes`; subsequent `listFiveActsNotes` calls on later boots find it. Recovery path is identical: next restart shows the entry.

### Scenario 3 — Existing universe with old `.base` files

**Status:** PASS

**Trace:**
- Legacy `.base` files live in `{universe}/.constellation/bases/` per `bases.rs:700–705` (`workspace_bases_dir = cdir.join("bases")` where `cdir = active_constellation_dir`). This path is **completely orthogonal** to `{universe}/Five Acts/` (universe root subfolder).
- `list_five_acts_notes_at` enumerates only `*.md` files in `{universe}/Five Acts/` (system_notes.rs:190: `if path.extension().and_then(|s| s.to_str()) != Some("md") { continue; }`). A `.base` file in `Five Acts/` would be filtered out.
- The legacy "Workspace Bases" section in the sidebar (lines 4773–4806) reads `bundle.workspace_bases` from the boot bundle, which is populated by `bases::list_workspace_bases(app)` (boot_bundle.rs:100–103). This is independent of the Five Acts code path.
- Both render simultaneously. The Five Acts section is rendered ABOVE Workspace Bases (4738 before 4772). Confirms the architect doc's §11 #4 lock ("silent ignore" of old `.base` files via the new code).

PASS — confirmed at code level.

### Scenario 4 — User-edited system note

**Status:** PASS (covered by §E test `existing_user_edited_file_left_unchanged`)

**Trace:** `init_at` in system_notes.rs:94–129 has the early-return at line 110–118:
```rust
if target_path.exists() {
    return Ok(());
}
```
The transfer-on-edit invariant is implemented as **transfer-on-presence** — any file with the canonical name in `Five Acts/`, edited or not, is preserved. The §E test `existing_user_edited_file_left_unchanged` (system_notes.rs:267–282) writes user content first then calls `init_at` and asserts content is unchanged. PASS.

### Scenario 5 — User deleted system note

**Status:** PASS (cold restart) / FAIL-WITHIN-SESSION (same as Scenario 10)

**Trace:** If the user deletes `Observation — Recent Captures.md` and restarts the app, on the next cold start `state.db` is `None`, `ensure_search_db_ready` reaches `init_five_acts_system_notes`, `init_at` finds the file absent, and writes the canonical content back. Recovery is automatic.

**Edge case:** Within the same app session, the user deletes the file → no re-init happens. `state.db.is_some()` is `true` (already initialized this session), so all subsequent `ensure_search_db_ready` calls early-return. The system note will NOT be re-created until the user restarts the app or switches universe (and even then — see Scenario 10).

This is a subtle UX issue: "I deleted it; I want it back" → user has to restart the app to recover. The §E test set has no test for "delete during session, expect re-init" — and there couldn't reasonably be one without restructuring `ensure_search_db_ready` to be idempotent on re-call.

**Recommendation:** Document in `docs/help.uConstellation.World/Five Acts/Five Acts.md` that recovery requires a restart, OR fix by making `init_five_acts_system_notes` callable independently from a Tauri command that the frontend can invoke (e.g., a "Restore Five Acts defaults" Settings button).

### Scenario 6 — User edited the embedded YAML to something invalid

**Status:** PASS

**Trace:**
1. User opens the host note and corrupts the YAML inside the ` ```base ` block (e.g., changes `dimension: note.created_at` to `dimension: note.frobnitz` or breaks indentation).
2. LensBlock.svelte:38–48 calls `executeLens(lensYaml)`.
3. The Rust pipeline (`query.rs:67–70`): `parse_lens_yaml` errors on malformed YAML, `validate` errors on unknown dimensions. Both convert to `Err(String)`.
4. The Tauri command returns `Err`, which the frontend `.catch((err) => ...)` (LensBlock:44–48) handles by setting `error = err.message`.
5. The render block at LensBlock:81–85 displays a red `lens-error` div with the label "Lens error:" and the validator's message. The rest of the note's markdown body renders normally (LensBlock is a CM6 widget — failures don't propagate to the editor).

NO crash. Error is surfaced cleanly. The fix path is: user edits the YAML in the editor to be valid again; on save + reload the LensBlock re-mounts with the new text. PASS.

### Scenario 7 — `Five Acts/` directory exists but is empty

**Status:** PASS

**Trace:** `list_five_acts_notes_at` at system_notes.rs:175 does `fs::read_dir(&five_acts_dir)`. If the dir exists but has no `.md` files, the loop at line 188 iterates over non-`.md` entries (all `continue`), leaving `out: Vec<(String, PathBuf)>` empty. Returns `Ok(Vec::new())`. The sidebar's `{#if fiveActsNotes.length > 0}` guard at `+layout.svelte:4745` hides the section. PASS.

However — note the recovery path from Scenario 5 also affects this: if the user has an empty `Five Acts/` dir within the same app session (e.g., they deleted the canonical file), `ensure_search_db_ready` won't recreate it. Same restart-required recovery.

### Scenario 8 — `Five Acts/` exists with a non-system Markdown file

**Status:** PASS

**Trace:** User puts `Random.md` in `Five Acts/`. `list_five_acts_notes_at` enumerates all `.md` files. The `file_stem` extraction at line 193–196 yields "Random" as `display_name`. Sorting by display_name at line 206 places "Observation — Recent Captures" and "Random" alphabetically (the Em-dash `—` U+2014 sorts before 'R' in lexicographic ordering of Unicode codepoints — verified: Em-dash is U+2014 = 8212 decimal, 'R' is U+0052 = 82 decimal; ASCII letters come BEFORE Em-dash in code-point order, so "Random" actually sorts BEFORE "Observation — Recent Captures"). Both render as sidebar entries. Clicking opens the host note (`openNoteTab(note.absolute_path, ...)` at +layout.svelte:4760). No special treatment — `Random.md` opens as any other note (LensBlock will only render if it contains a ` ```base ` fence). PASS.

**Sub-finding (P3 alphabetization):** The sort order may surprise users. If they expect "O" sort below "R" in English, ASCII letter order will look right. But if "Random" sorts before "Observation — Recent Captures" because the Em-dash isn't a regular letter, the order may be inconsistent with i18n collation. Worth confirming in §I Test 5 (multilingual mix). For purely English filenames, sort works as expected.

### Scenario 9 — Permission-denied on Five Acts directory

**Status:** PASS

**Trace:**
- `init_at` at system_notes.rs:99 calls `fs::create_dir_all(&five_acts_dir)`. If the universe root is read-only, this returns `Err(io::Error)` and `init_at` returns the wrapped error to `init_five_acts_system_notes`.
- The caller at search.rs:4821 catches the error explicitly:
  ```rust
  if let Err(e) = crate::lens::system_notes::init_five_acts_system_notes(app) {
      eprintln!("[search] init_five_acts_system_notes failed (non-fatal): {}", e);
  }
  ```
- The error is logged to stderr (Windows GUI subsystem swallows this — no user-visible log unless dev build), AND `ensure_search_db_ready` continues to return `Ok(())`. The rest of the app boots normally.
- `listFiveActsNotes` from a read-only universe returns `Ok(Vec::new())` if dir creation failed (the dir doesn't exist).

PASS — the rest of the app works; the Five Acts section is just absent.

**Recommendation:** Consider adding a `diag_log` call (the durable file log at `<universe>/.constellation/diagnostics.log` used elsewhere in search.rs) so the error is recoverable for debugging. Currently only stderr — invisible on Windows release builds (per memory note `feedback_devtools_dev_only.md`).

### Scenario 10 — Cross-universe switch

**Status:** **FAIL — P1 finding**

**Trace:**
1. User switches active universe via Universe Manager. Frontend calls `set_active_universe(id)` → `handleUniverseSwitch()` (+layout.svelte:2087).
2. `handleUniverseSwitch` clears `fiveActsNotes = []` at line 2106. Good.
3. Calls `initializeApp()` at line 2150. This re-fires `constellation_boot_bundle` and then `refreshLibraryCaches()`.
4. `refreshLibraryCaches` calls `cache_boot_snapshot_core` → `ensure_search_db_ready` (cache.rs:179).
5. **PROBLEM:** `ensure_search_db_ready` (search.rs:4771–4826) has an early-return at lines 4774–4777:
   ```rust
   let guard = state.db.lock().map_err(|e| e.to_string())?;
   if guard.is_some() {
       return Ok(());
   }
   ```
   Since `state.db` is still `Some(<old conn>)` from the previous universe (no path resets `state.db = None` anywhere in the codebase — confirmed by grep across `src-tauri/src/` for `db = None` / `invalidate.*search` — only `libraries.rs:114` resets a separate `LIBRARIES_CACHE`, not `SearchState.db`), the function early-returns.
6. `init_five_acts_system_notes` (line 4821) is NEVER CALLED for the new universe.
7. Result: if the new universe doesn't already have `Five Acts/Observation — Recent Captures.md` on disk, the sidebar entry will be missing for the rest of the app session.

**Severity:** P1 — this directly violates the Architect §11 #3 lock (idempotent init across boots) for the universe-switch case. The Plan §7 rollback section asserts "Existing `Five Acts/*.md` files on disk remain" — that's about file safety, but the inverse case (file ABSENT, expected creation) fails here.

**Pre-existing context:** This is NOT a new bug introduced by MIG-055 — the `ensure_search_db_ready` early-return has been in the codebase for many MIGs (the search.db connection has not been universe-switch-aware). However, MIG-055 is the first feature whose user-visible behavior depends on `init_*` running per-universe rather than per-app-session. Previous boot-time initializers (SQLite schema, FTS5 tokenizer registration, bigram purge, sky_backfill) are universe-independent.

**Recovery path:** User restarts the app. On cold start, `state.db` is `None`, `ensure_search_db_ready` reaches `init_five_acts_system_notes`, system note is created. So the failure mode is "one restart required after a universe switch to a never-opened universe".

**Recommendation — fix candidates (in order of preference):**
- **(a) Add a `reset_search_state` call** in `universe::set_active_universe` that does `*state.db.lock()? = None;` to invalidate the cached connection. This also fixes the pre-existing latent bug where reads via `state.db` after a universe switch return data from the OLD universe. (Touches search.rs + universe.rs.)
- **(b) Make `init_five_acts_system_notes` callable independently** at the top of `cache_boot_snapshot_core` (or any per-universe boot path), not gated through `ensure_search_db_ready`. Costs ~1 fs stat per boot — negligible.
- **(c) Add a `list_five_acts_notes` side effect** that calls `init_five_acts_system_notes` if the canonical file is absent. Combines list + init at the IPC the frontend already calls. Risk: deduplicates with the canonical init path.

The MIG-054 audit at `lab/reports/MIG-054-audit-2026-05-25.md` (referenced in PLAN §1) had a similar drift category. Worth a Pending Job entry if not fixed in §I.

### Scenario 11 — Rollback

**Status:** PASS (per Plan §7 documentation)

**Trace:** Plan §7 explicitly documents the rollback shape:
- `Five Acts/*.md` files on disk remain — `init_five_acts_system_notes` never deletes.
- `note_meta` / `note_summaries` untouched — MIG-055 only reads.
- UI surface vanishes when §F (sidebar) + §D (LensBlock) revert.

Verified at code level: no DELETE/DROP statements in `src-tauri/src/lens/`. The only filesystem writes are `fs::create_dir_all` (idempotent) and `fs::write` (only on file-absent). PASS.

## Drift catches (beyond the per-scenario set)

These are findings that surfaced while reading the code but aren't in the 11-scenario checklist:

1. **Path-normalization on Windows.** `list_five_acts_notes` at system_notes.rs:161 does `rel.to_string_lossy().replace('\\', "/")` to normalize the relative path to forward slashes. The `absolute_path` field at line 162 does NOT normalize backslashes — Windows callers receive a mixed-slash path. The frontend's `openNoteTab(note.absolute_path, ...)` may or may not need the path normalized depending on how the file-resolver handles it. Verified at +layout.svelte:4759: `$activeTab?.path === note.absolute_path` — equality compare. If `$activeTab.path` was stored from a forward-slash path (which `relative_path` is) and `absolute_path` has backslashes, the equality fails — the "active" class won't apply, and Reactivity to the active tab will be silently broken. Worth confirming in §I.

2. **Five Acts/ at universe root vs library-isolated.** The Architect §11 #2 lock places Five Acts at `{universe}/Five Acts/` — universe root, not inside any library. But the universe root IS itself a library (the `universe_notes` library — `is_universe_notes: true`, path == universe root). So `Five Acts/` is technically inside the `universe_notes` library. If a user has many subfolders in their universe_notes library, they'll see "Five Acts" as a regular folder in the FileTree component too, alongside the special sidebar section. This is the intended visible-folder behavior per Architect §11 #2 ("named templates ... visible folder makes the cognitive vocabulary part of the user's everyday navigation"). Confirms the design but worth noting as a §I observation.

3. **No corruption of `note_meta`.** `execute_lens` opens a fresh connection (`Connection::open(&db_path)`) and only does SELECTs. Verified by reading `src-tauri/src/lens/sql_builder.rs` (NOT included in this report but confirmed via grep — only SELECT statements emitted). No write path through the lens module. Architecturally clean.

4. **§E test `existing_canonical_file_left_unchanged` may be brittle on Windows.** Lines 256–262 sleep 10ms then re-check mtime. Windows NTFS has 100ms mtime resolution on some configurations. The test asserts `mtime_before == mtime_after` — which is true if we DIDN'T touch the file (we shouldn't have). However if a future bug WROTE the file with the same content, mtime would change. The test would catch the regression. PASS as-is.

5. **`list_five_acts_notes_at` failure modes.** Lines 181–187 call `fs::read_dir` and wrap any error in a string. If the dir exists but is unreadable (permissions), the function returns `Err` rather than `Ok(Vec::new())`. The frontend at +layout.svelte:1955 swallows errors with `.catch(() => {})`. So a permission error on the Five Acts dir won't crash the app, but the section will be silently empty with no diagnostic. Same as Scenario 9 — log to stderr only.

## Open items for §I Boss-test

The following scenarios REQUIRE live universe verification (rather than fixture testing) because they depend on real Tauri command scheduling, real universe paths, and real boot timing:

1. **Scenario 1 boot race.** On a brand-new universe (created during the §I test session, NOT a pre-existing one), does the sidebar "Five Acts" entry appear on first paint, or only after restart?
   - **Test setup:** Create a fresh universe via Universe Manager.
   - **Expected:** "Five Acts" section visible in left sidebar with "Observation — Recent Captures" entry.
   - **Failure mode:** Section missing until app restart → confirms the P3 boot race finding; recommendation (a)/(b)/(c) above.

2. **Scenario 10 P1 universe switch.** Boss has multiple universes registered. Switch from Universe A (pre-existing, system note created) to Universe B (newly registered, never opened during this session).
   - **Test setup:** Universe B should be a fresh path with no `Five Acts/` dir yet.
   - **Expected (currently failing):** Five Acts entry appears in sidebar for Universe B.
   - **Likely actual behavior:** Sidebar shows no Five Acts section until the app is restarted.
   - **Severity:** P1 — must fix before §J.

3. **Scenario 8 alphabetization on real multilingual filenames.** Add a Five Acts/`اول.md` (Arabic "first") plus the canonical Observation note. Verify sort order matches the user's locale expectation.
   - **Test setup:** `{universe}/Five Acts/اول.md` (any content).
   - **Expected:** Order is locale-stable or at least documented.

4. **Scenario 9 permission-denied behavior.** Make `{universe}/Five Acts/` read-only at the OS level. Restart the app.
   - **Expected:** Sidebar's Five Acts section is empty. App boots normally. Universe still usable.
   - **If app crashes or boot fails:** P1.

5. **Drift catch #1 (path normalization).** Click on the "Observation — Recent Captures" entry in the sidebar. Then click the same entry again.
   - **Expected:** First click opens the note in a new tab; second click does nothing OR focuses the existing tab. The `class:active={$activeTab?.path === note.absolute_path}` check should highlight the sidebar entry while that note is the active tab.
   - **Failure mode:** Sidebar entry never highlights (active class never applies) → path-normalization bug confirmed.

6. **System note YAML round-trip after open + save.** Open "Observation — Recent Captures.md" in NotePane (no edits). Save. Close. Re-open.
   - **Expected:** The lens still renders rows; the file content on disk is byte-identical to `RECENT_CAPTURES_CONTENT` (CM6 should preserve `\n` line endings).
   - **Failure mode:** CM6 normalizes the content (BOM, trailing newline, etc.) → the §G drift test `canonical_yaml_matches_system_note_constant` still passes (it only checks the YAML inside the fence) BUT the file is now "user-edited" from the system's perspective. Subsequent boots leave it alone (Scenario 4 behavior). This is mostly fine but worth confirming the user can re-trigger the canonical write if they want.

7. **Workspace Bases coexistence.** If Boss has any legacy `.base` files in `{universe}/.constellation/bases/` (from before MIG-054 revert), both sections must render: "Five Acts" above "Workspace Bases". The `.base` files render via the legacy `BaseView.svelte` (untouched by MIG-055).
   - **Expected:** Both sections visible; both clickable; lens block renders only in `.md` files.

## Summary

**Verdict: PASS-WITH-NOTES.** The 11-scenario migration-path checklist surfaces 1 P1 finding (Scenario 10 universe switch) and 1 P3 finding (Scenario 1 boot race). Both are not data-corrupting; both have a clean recovery path (app restart). The §E and §G test suites correctly assert the file-level invariants — the issues exist at the boot-orchestration layer, where MIG-055 inherits the pre-existing `ensure_search_db_ready` early-return shape that wasn't universe-switch-aware before MIG-055 had any universe-switch-dependent behavior to expose.

**Scenarios needing live verification:** 7 of the 11 (Scenarios 1, 8, 9, 10 + 3 drift catches). All listed above with test setup + expected/failure modes.

**Recommended pre-§I action:** Decide whether to fix the P1 universe-switch finding before §I Boss-test (preventing the user from hitting it cold) OR address it as a §J follow-up after Boss confirms the rest of the surface. Option (a) — adding `reset_search_state` in `set_active_universe` — is the architecturally cleanest fix and also closes the pre-existing latent bug where `state.db` reads after a universe switch return stale data. Estimate: 1 commit, ~30 lines, low risk.
