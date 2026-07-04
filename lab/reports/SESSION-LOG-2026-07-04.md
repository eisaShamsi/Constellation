# Session Log — 2026-07-04 (evening session)

## Session bring-up
- `git pull origin main` — already up to date (tip `1f54322d`).
- **Binary relinked** (the handover's ONE PENDING ACTION): app closed, `npm run build` (41.7 s) + `cargo build --release` (2 m 09 s) → `constellation.exe` mtime **2026-07-04 18:39:51** (after the 17:37 tip). Freshness verified per the standing rule: `build/` greps positive for the new `letterJump` Base key → the Base-search O(rows) fix + R2-instrumentation removal are IN the binary. Ready for Boss tests.
- Read: orientation v3.24 preamble, `docs/handover/Handover-2026-07-04-freeze-batch2-base.md`, tail of SESSION-LOG-2026-07-03.

## SO #8 cross-check — the freeze audit's deferred batches (workflow `wf_6de4f393-1dc`, 4 verifiers + 5 adjudicators, all adjudications upheld)

The handover's ranked batches were minted BEFORE Batch S/D landed; the cross-check read the current code. Verdicts:

### F2 — **OBSOLETE AS STATED; the live defect is the INVERSE. Re-scoped → F2′.**
- The stated goal ("stop refreshing the file tree on our own 1.5 s autosaves") is **already achieved end-to-end**: every gated write marks `watcher_suppress` (`write_gate.rs:249-250`, also gate_rename :582, gate_rmw_rename :709, gate_delete :747; TTL 2.5 s `watcher_suppress.rs:27`), and the Rust watcher filters marked paths BEFORE emitting `library-changed` (`watcher.rs:78`, emit skipped when nothing survives :82). Editor autosaves (`NotePane.svelte:459` 1500 ms → `write_note` → `gate_write`) **never reach** the frontend handler. The unconditional `pendingTreeRefresh.add` at `+layout.svelte:3053` still exists but only fires for external edits, directory create/remove events, and non-gated writes (grep: no ungated `.md` note writes exist).
- **The inverse, LIVE defect (the create-path verification found it): 3 creation flows never refresh the tree, and their watcher fallback is suppressed by the very mechanism above → newly created notes are INVISIBLE in the file tree**:
  1. **Wikilink-create-on-click** — `NoteEditor.svelte:411-418` (`createNote` → `openNoteTab`, no refresh).
  2. **Expression Forge export** — `ExpressionForge.svelte:143-145` (`createNote` + `writeNote`, no refresh).
  3. **Sense-Making Canvas promote** — `SenseMakingCanvas.svelte:270-278` (`createNote` + `writeNote` + `openNoteTab`, no refresh).
  - Import is PARTIAL: `onImportComplete` → `refreshLibraryCaches` (+layout:3471) refreshes boot caches, NOT `libraryTrees`; relies on unsuppressed DIRECTORY events / lazy expand.
  - Safe flows (explicit refresh already): createNoteWithTemplate (+layout:4040), quick capture (:4060-4061), daily note (:4285 — the MIG-080 §A.2 fix, whose comment states the class: "creation via gate_create_exclusive doesn't update the tree").
- **F2′ = add the explicit `refreshLibraryTree` to the 3 gap flows (+ decide import)** — the same fix MIG-080 §A.2 applied to daily notes. Small, high-confidence, user-visible correctness fix (WA#6: discovered → fix).

### W — **CURRENT, minus one strike.**
- **STRIKE `read_library_tree`** — already `#[tauri::command(async)]` (`libraries.rs:285`, Batch S comment above it; commit `44ae1325`).
- Still-sync, still to-do (attributes quoted by the verifier): `collect_library_notes_with_metadata` (libraries.rs:4979), `scan_library_tasks` (tasks.rs:379), `scan_library_note_dates` (tasks.rs:538), `scan_library_links` (libraries.rs:2378), `notes_by_tag` (libraries.rs:2744), `search_by_property` (libraries.rs:797), `search_stars` (libraries.rs:593), `execute_dataview_query` (dataview.rs:289), `resolve_embed` (embeds.rs:495 — cold/invalidated call walks the library; cached after).
- The 2 WA#4 writers confirmed: `repair_external_libraries_on_startup` (canonical.rs:1308 — walks every library + renames files) and `cece_record_correction_for_card` (classifier/mod.rs:239 — writer, **no walk**; its inclusion rests on WA#4 writer status only).
- Nuance: `embeddings.rs` family already async per PJ-066; the only genuine embed walk left sync is `resolve_embed`.

### L — **PARTIALLY STALE; re-scoped.**
- **STALE:** the `import_*` commands were **never** dispatch-thread parkers — all four are `pub async fn` since their introduction (`087a81d8`); an `async fn` command runs on the async runtime even with a plain `#[tauri::command]` attribute (the `(async)` marker is only needed for *sync* fns). **Audit-method lesson: counting "sync commands" by attribute alone over-counts — `async fn` bodies are already off-thread.**
- **STALE:** `cache_full_links` already `(async)` (cache.rs:372, Batch S).
- **Still to-do:** the six library-scale **canonical.rs sync walkers** — `canonicalize_preview` (:386), `canonicalize_execute` (:454), `auto_canonicalize_all` (:634, already emits progress events), `inject_cid_library` (:795), `de_canonicalize_library` (:902), `repair_external_libraries_on_startup` (:1308) — untouched by Batch S/D; plus the **federated_conn whole-read hold** in `cache_full_links` (guard acquired cache.rs:396, released only at return across both per-schema loops :432/:448 — no longer parks dispatch, but blocks every other federated_conn user for the full multi-schema read).

### F1 — **LIVE; scope narrowed since the audit.**
- CONFIRMED: focus mode fires a full compose + `writeNote` IPC on **every keystroke**, undebounced (`FocusPane.svelte:165-174` onchange per docChanged → `+layout.svelte:7228-7244` SINGLE_OWNERSHIP branch: editNoteBody + composeNoteModel + writeNote per keystroke). Violates Perf Rule 3 (zero invoke on the keystroke hot path; ≥1500 ms save debounce).
- Location correction: the vestigial unarmed `saveTimer` lives in `FocusPane.svelte:32` (cleared at :205, never armed) — not +layout.
- `write_note` is **still deliberately SYNC** (libraries.rs:334) and now routes through `gate_write` (:371) — its syncness is architecturally load-bearing per `write_gate.rs:617-620` (a SYNC write_note parking on a path lock that waits on the DB writer would re-freeze dispatch through the back door — hence the "no state.db inside path lock" hard rule). So F1's remaining scope: **(1) debounce the focus save path; (2) a deliberate ruling on write_note sync-vs-async** — /migration-grade, NOT a drive-by flip.
- Harness confirmed present: `tests/mig-076/` = currentBugRepro.test.ts, noteModel.test.ts, runtimeHarness.test.ts. F1 still ships ALONE through the Editor-Surface Gate.

## Corrected ranked queue (post-cross-check)
1. **F2′** — fix the 3 invisible-new-note creation flows (+ import decision). Small, discovered-defect class.
2. **W** — 9 sync fs-walk commands → async (+ the 2 WA#4 writers with caller walks).
3. **L′** — the 6 canonical.rs sync walkers + the federated_conn per-schema hold hygiene.
4. **F1** — focus debounce + write_note ruling; Editor-Surface Gate harness; ships alone.
5. F3 — unchanged (FileTree virtualization, Louvain worker, sky parse, second-screen reads).
- Unchanged also open: get_360_view index-read /migration; init_db phase-profiling MIG; switch-back-fast residual; MIG-088 Phases 6–10; Arabic callout End/Home caret.

*Awaiting Boss batch pick.*
