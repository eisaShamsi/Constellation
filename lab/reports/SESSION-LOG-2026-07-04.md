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

*Boss pick: **F2′ then W**.*

---

# Function in hand: F2′ — notes created by the app must surface in the file tree

**Concept (the horse):** a note you create must appear where notes live — the file tree reflects creation instantly, from every creation door.

## Built (commit `9dc6f193`)
- **Class fix, not instance:** all frontend creation flows funnel through the ONE store `createNote` wrapper (exactly 4 callers: wikilink-create `NoteEditor.svelte:415`, Forge `ExpressionForge.svelte:143`, Canvas promote `SenseMakingCanvas.svelte:270`, createNoteWithTemplate `+layout:3991`). The wrapper now **emits `note-created`** after a successful create; nothing changes at the call sites — any future creation door gets tree visibility for free.
- **+layout:** the watcher's 300 ms coalescing flush extracted to `scheduleWatcherFlush` (same closure/sets — behavior-identical); a new `note-created` listener maps path→library via new `libraryIdForPath` (longest prefix WITH separator-boundary) and joins the same flush (tree refresh + loadAllStats + OrgChart dirty + 5 s cache refresh — correct for creations). Unlisten in cleanupFns (Rule 4).
- **Cross-window by construction:** Tauri `emit` crosses windows; the second screen mounts via `screen-entry.ts` (no +layout), so the main-window listener is the only tree owner — and a second-screen wikilink-create refreshes the MAIN tree.
- **Import leg:** `ImporterModal` now fires `onImportComplete(targetLibrary)` at import COMPLETION (not the Done button — Escape/✕/backdrop exits skipped it); +layout refreshes that library's tree + caches.
- **Second screen:** its own `note-created` listener (u5b) joins the existing u5 debounced reload — note list/dashboard stay in step (it never hears `library-changed` for gated creates).

## Adversarial review (wf_197b07af-29d — 3 lenses → per-finding verify): 4 confirmed, ALL fixed in-pass (WA#6)
1. P3 `libraryIdForPath` had no separator boundary — library `Research` would steal notes in sibling folder `Research Notes` → refresh targets the wrong tree (found by 2 lenses). Fixed (boundary check + trailing-separator trim).
2. P3 import refresh tied to the Done button only. Fixed (fires on completion).
3. P2 second screen blind to gated creates. Fixed (u5b listener).
4. One verifier CONTRADICTION adjudicated honestly: a refuter proved empirically that Windows parent-directory Modified events (never suppressed, pass the watcher's `is_dir()` filter) often fire during subfolder writes — so the watcher *sometimes* rescues subfolder creates. Non-blocking for F2′: the daily-note gap (MIG-080 §A.2, Boss-validated) proves the class is real at least at library roots; `note-created` makes appearance DETERMINISTIC instead of configuration-dependent. Noted, no code change needed.

svelte-check 0 errors ×2 (pre- and post-review-fixes). Binary rebuilt (19:35:48) + freshness-verified (`note-created` present in main chunks AND the second-screen bundle).

## Boss test — F2′
- **Stage 1 (wikilink-create, top-level + subfolder): PASS** ("Perfectly passed").
- **Stage 2: Test 1 Forge PASS** (screenshot: `Forge Birth Test` at Eisa Cognitive Knowledge top level, correctly alpha-sorted) · **Test 2 Canvas promote PASS** · **Test 3 import + Escape exit PASS** · **Test 4 second screen: BLOCKED — `Ctrl+Shift+2` doesn't work** + Boss design reminder: the second screen is an *extension* of the main screen (displays-not-domains).

## Boss-found defect: ALL Shift+digit shortcuts dead since birth (fixed, commit `1676a28f`)
- **Mechanism (utils.ts `eventToShortcut`):** physical-code normalization existed only for `'Key*'` (letters); a Ctrl+Shift+2 press carries `e.key === '@'` (layout's shifted char), producing `'Ctrl+Shift+@'` which can never match the stored `'Ctrl+Shift+2'` — on ANY layout. The `second-screen` default was the only Shift+digit default → dead since introduction; the Command-Palette/dock-button doors always worked.
- **Fix:** `'Digit2' → '2'` normalization, same branch shape and same non-Latin-layout rationale as letters. svelte-check 0.
- **Honest caveat (logged in commit):** a custom shortcut previously *recorded* as a shifted symbol (e.g. `Ctrl+Shift+@`) would stop matching; post-fix recordings store the digit form. Pre-release, acceptable.
- Second-screen concept check: the F2′ `u5b` listener adds NO operations to the second screen (displays-not-domains intact) — it only lets the extension's mirrored surfaces hear about creations, which an extension must, or it diverges from the main screen it extends.

## F2′ Test 4 re-run (Boss, second monitor connected): **PASS** — F2′ fully closed
Second screen picked up the app-created note; the `Ctrl+Shift+2` fix live. **Boss issued two governing SS design rulings at the pass:** (1) *"The SS should be contextual to the main screen."* (2) *"It shouldn't replicate what is already displayed on the main screen."* + ordered: read the entire SS history/concept, write a paper, park the rework in the PJ list.

## PJ-068 filed — Second Screen contextual-companion rework (PARKED)
- **Concept paper written:** `docs/concept-papers/PJ-068-Second-Screen-Contextual-Companion-Concept-Paper.md` — full history (birth `48eb3f01` 2026-03-13 → today), the ratified concept + the two rulings as governing law, the **replication audit** (Navigator companion REPLICATES; OrgChart mode REPLICATES + UNREACHABLE; Map/Index-term/Dashboard MIXED; fallback tab-strip editor NON-CONTEXTUAL; Sky-graph/Split/Editor-panels/Index-compare COMPLEMENT — keep), structural riders (editor-panels mode shadowing [static read], dead `screen:open-note` wire, Rule-8 re-walks, alias-blind buildSkyData, hardcoded English, no RC), and 5 doc-drift items. Source: history-dig workflow `wf_42ec73f0-794` (5 readers, 172 tool uses).
- **Pending Jobs v1.15** created (new file alongside v1.14) with the PJ-068 entry. Rework needs per-mode Boss rulings at reopen → /migration.

---

# Function in hand: Batch W — the panel fs-walk sweep (Boss: "If it passes, let's proceed to W" — Test 4 passed)

**Concept:** the user's hand on the app is never taken hostage — panel data loads must not park the dispatch thread.

## WA#4 caller verification FIRST (workflow `wf_b16f99ec-b6b`: 11 per-command walks + adversarial verifier, ~992k tokens)
Verifier corrected two agent verdicts before build: (1) `resolve_embed` "flip bare" REFUTED — post-flip, N cold embeds would run N CONCURRENT full-library walks (sync dispatch used to serialize them into 1 walk + N−1 cache hits) → Rust single-flight required; (2) `notes_by_tag` guard spec had a hole — the catch path also writes, so a stale FAILED call could blank a newer call's results → guards must cover try AND catch. Upheld: both writer analyses, all four dead-code claims, the skyviewGeneration token already correct.

## Batch W BUILT (commit `d9f8bd80`) — 11 commands off the dispatch thread
- **Bare flips:** `execute_dataview_query` (dataview.rs) · `repair_external_libraries_on_startup` (canonical.rs — one-shot boot caller, gated writes).
- **Flips + frontend stale-result seq guards** (repo convention; catch paths covered): `scan_library_tasks` + `scan_library_note_dates` (+layout `_calRefreshSeq`; GlobalTasksView `_loadSeq`) · `collect_library_notes_with_metadata` (NotebookNavigator `navLoadSeq`, onMount + refreshData) · `scan_library_links` (SecondScreenPage `epGeneration`/`scGeneration`) · `notes_by_tag` (DashboardView + SecondScreenPage `tagLoadSeq`) · `search_by_property` (NotebookNavigator `propSearchSeq` + onModeChange bump — also closes a pre-existing late-repopulation race) · `search_stars` (store.ts `_searchStarsSeq`; empty-query clear invalidates in-flight).
- **Flips + Rust serialization:** `resolve_embed` + `BUILD_LOCK` double-checked single-flight in `get_or_build_vault_index` (embeds.rs) · `cece_record_correction_for_card` + `RELIABILITY_LOCK` around the reliability.json RMW (reliability.rs, PULSE_LOCK idiom — also closes the pre-existing race via already-async `cece_resolve_disambiguation`).
- **Verification:** svelte-check 0 errors · cargo check green · 12 reliability tests pass (no lock re-entrancy). Binary rebuild in progress → Boss test (staged) next.
- Batch W status after this: the audit's W list is DONE. Remaining ranked: **L′** (6 canonical.rs walkers + federated hold hygiene) → **F1** (Focus, gated, alone) → **F3**.

## Batch W Boss test Stage 1 (2026-07-05): Tests 1–3 PASS + a Boss-found tag defect → FIXED (`9b64d1d6`)
- **Tests 1–2 (Global Tasks freeze-gone + toggle race; Calendar dots) PASS. Test 3 (tag A→B race) PASS** — but the Boss's screenshot showed a tag chip counting **127 notes returning an EMPTY list**, and imported tags rendering as `#"quoted"` chips.
- **Mechanism (pinned by reading all three tag paths):** THREE tag definitions coexisted. Chips are counted from the INDEX (`note_meta.tags_json` aggregate, cache.rs:1338); `notes_by_tag` fetched via an inline-`#tag`-only fs regex (frontmatter tags could NEVER match → 127 vs 0); `parse_frontmatter` (search.rs:4322) stripped quotes in the `tags: [a,b]` arm but NOT the block-list `- "tag"` arm (→ quoted tags in the index — the Wikipedia-import population); `scan_library_tags` counted inline OCCURRENCES and its YAML branch was dead code.
- **Fix — one tag authority:** `search::parse_frontmatter` now `pub(crate)` and the single definition; `notes_by_tag` + `scan_library_tags` delegate to it (per-note counts = chip semantics); block-list arm quote-strips; `notes_by_tag` normalizes the incoming chip label (pre-fix quoted index rows keep working); the inline-hashtag regex compiles once (OnceLock). Dead YAML branch deleted.
- **Honest transitional state:** index rows keep quoted chip labels until each note's next reindex; clicks work for both forms immediately. cargo test --lib **1006/0**.

## Batch W Stage 2 + tag re-test (Boss, 2026-07-05 morning): re-test PASS · T4 PASS · T6 PASS · T5 exposed 2 defects → ALL FIXED (`55919304`)
- **Tag re-test PASS** (the 127 notes list; screenshot) + **new Boss request:** the tag cloud and the tag-notes list must scroll INDEPENDENTLY → both columns now 70vh-bounded with own scrollbars (`DashboardView` `.tags-list-col`/`.tags-notes-col`; notes col sticky).
- **T5 (embeds) findings:** `![[Note]]` targeting a note in ANOTHER library rendered "File not found in vault". Two defects: (1) **cross-library gap** — `resolve_embed` searched only the note's own library while the `![[` picker offers universe-wide names → new fallback: on a bare-name miss, look the target up in the other registered libraries' cached filename indexes (strategy-5 lookup extracted to `lookup_in_library_index`; own-library path behavior-identical; mirrors wikilink cross-library precedent). (2) **"vault" terminology** (forbidden) + hardcoded EN on the missing card → `$t('embeds.notFound')`/`$t('embeds.lookupDetails')` added ×15 locales (each file's established library term verified — tr kütüphane, zh 库); diagnostic dump's "vault" wording → "library".
- svelte-check 0 · cargo check green · locale JSONs validated · binary rebuild in flight.
- **BOSS RULING (2026-07-05, design principle — record for every future resolver):** *"It is ONE universe, regardless of the libraries it contains"* — embedding (and by extension any name resolution: wikilinks, embeds, pickers) spans the ENTIRE universe **including libraries federated in through linked child-universes**. Verified satisfied by the shipped fix: the fallback iterates `load_all_libraries` → `resolve_universe_libraries` (libraries.rs:103), the federation-aware recursive walk — own libraries + all cUniverse children. Any future feature that resolves a note by name must use this set, never a single-library scope.

## NEW BOSS DIRECTIVE (2026-07-05): restructure the Notes Navigator — provisional MIG-090
> "I want to restructure the 'Note Navigator', either to merge it with the 'File Explorer', or redesign it to match Constellation philosophy. Today, it is slow and dumb, and it feels like a foreign body within the Constellation ecosystem."
- /migration-scale → **Architect phase opened** (read-only discovery workflow `wf_971f71e2-913`: navigator surface map · overlap matrix vs FileTree/Base/SearchHub/QuickSwitcher/tags · philosophy fit + WA#5 prior art · constraints/invariants + Rule-8 index-replacement feasibility · adversarial verify). Ammunition already in hand: Batch-W caller walk proved the mount reads EVERY note's full content per library (libraries.rs:4979 family) for previews; unvirtualized; PJ-068 ruled its SS mount REPLICATES. Architect doc + options to Boss next; no code before an approved Plan.

## MIG-090 Architect DONE + Boss rulings (2026-07-05)
- **Architect doc:** `docs/MIG-090-Architect-Notes-Navigator.md` (discovery `wf_971f71e2-913`, verifier upheld all findings, 3 precision corrections). Headlines: NO concept paper exists (02-file-tree explicitly carves it out — a carriage with no horse); mount = ~15,000+ full file reads (universe-root library overlap doubles the corpus; canonical files read twice; repeats per batch action); unvirtualized 7,600 rows ≈ 70k+ DOM nodes; **Folders mode NEVER worked** (invoke `libraryPath` vs Rust param `path`, error swallowed — static read, cite-with-repro caveat); fake created-sort; fake divider; SS single-click no-op; batch-tag YAML string-splice can corrupt inline-array frontmatter + bypasses the Editor gate; zero mutation listeners (the very reason the Boss parked its RC 2026-06-29); dead never-wired `navigator/data.svelte.ts` (253 lines, zero importers); Rule-8 feasibility = YES (note_meta already persists everything the list shows).
- **BOSS RULINGS (three, all banked):** (1) **Direction = Option B — the All-Notes Base**: the sidebar list mode becomes a built-in Base over the whole universe (lens engine: SQL over note_meta, federated, virtualized, search + letter rail + safe RC — the Boss-validated stack); old Navigator chain deleted; `enabledFeatures.notesNavigator` flag kept as rollback. (2) **Batch ops: PORT** to the new surface through proper shared write paths (never the YAML splice). (3) **SS branch: RETIRE in MIG-090** — the one PJ-068 Navigator-line ruling is hereby reopened and folded in; the rest of PJ-068 stays parked.
- Next: **Plan phase** (commit-sized steps + verification clauses) → Boss approval → build cascade.

## MIG-090 PLAN APPROVED (Boss: "Approved") — build cascade begins

### §0 — Predecessor → Replacement entries (Predecessor Lookup Rule; BEFORE any code edit)
| Predecessor (lives now) | Replacement (will live) |
|---|---|
| **Sidebar list mode** — `NotebookNavigator.svelte` two-pane browser, mounted at `+layout.svelte:6178-6185` (`sidebarMode === 'list'`), flag `$appSettings.enabledFeatures?.notesNavigator` (store.ts:3857, default true), mode button +layout:6151 (450px width-force). Pre-MIG lineage (no concept paper). | **The same button, same flag** → opens/focuses the auto-provisioned `All Notes.base` as a center **base tab** (BaseTab, `kind:'base'` — the +layout:4098/5549 open path). Width-force dies. |
| **Batch ops** — `NavBatchBar.svelte` (tag = YAML string-splice + unattested write_note loop, NotebookNavigator:237-320) | **BaseTab selection column + batch bar**, per-note SHARED handlers (tag via the shared add-tag routine + open-note reconciliation; move via `move_item`; delete via `deleteWithSetting`). |
| **Property search** — Navigator Properties pane → `search_by_property` (libraries.rs:797, sole caller) | **SearchHub `properties` category** (FTS5) — already shipped; the command retires. |
| **SS Navigator mount** — SecondScreenPage list branch (mode='second') | **Retired** (Boss ruling; the reopened PJ-068 Navigator line). No replacement on the SS — the main-window base tab is the surface. |
| **`collect_library_notes_with_metadata`** (libraries.rs:4986) + store wrapper `collectLibraryNotesWithMeta` (store.ts:3244) + `NoteWithMeta` type (store.ts:3233) | **`execute_lens`** over `note_meta` (existing engine; three new dimensions in §1). Command + wrapper + type deleted in §4. |
| **Navigator tags pane** (duplicate tag-tree builder, NavBrowserPane:33-54) | **TagsPanel** (right sidebar, tag_counts-backed) — already shipped. |
- **Folders-mode defect evidence (honest status):** statically certain (invoke sends `libraryPath`; the Rust param is `path` → Tauri rejects missing arg; `.catch(() => [])` swallows; every other caller passes `path`). Live-app confirmation NOT run — the pane is deleted by this MIG; recorded as static-certainty evidence, not a runtime repro.

### §1 BUILT (commit `de79bdf7`) — three All-Notes lens dimensions
`note.library` (Text, note_meta.library_name, sortable, full text ops) · `note.modified` (Timestamp, note_meta.modified, sortable, after/before/between/within) · `note.tags` (List = raw tags_json; NOT sortable/filterable v1 — exact-tag op deferred, Form-Aligns-To-Purpose). Frontend: labels (`lensBlock.colLibrary/colModified/colTags` ×15 — Tags MINED from each locale's `panels.tags`), ADDABLE list, tags non-sortable, renderCellValue (modified = locale date; tags = JSON array → ` · `-joined). Registry pinning tests updated to the new canonical 9 (+1 new test). **cargo lens suite 104/0 · svelte-check 0.** (Two earlier "could not compile" flashes were the recurring Windows file-lock race — clean on re-run, noted for the record.)

### §2 BUILT (commit `1e9481a0`) — the swap
- **`ensure_all_notes_base`** (bases.rs, `(async)`, idempotent): creates `All Notes.base` in the universe's workspace-bases dir on first use — scope ALL libraries (federated included — the ONE-universe ruling by construction), columns name·library·modified·tags·outgoing_count, order modified desc. NEVER overwrites (user's artifact). Registered lib.rs; store wrapper.
- **The sidebar list button** (same button, same `enabledFeatures.notesNavigator` flag) → `openAllNotesBase()` → opens the base as a center tab; the 450px width-force + preTreeWidth dance deleted.
- **Both NotebookNavigator mounts retired:** main `{sidebarMode==='list'}` branch + the SS list branch (the Boss-approved PJ-068 Navigator-line ruling executed). `'list'` trimmed from both mode unions (three-site lockstep clean; no window emits or branches on it).
- The condemned fs-walk loader (`collect_library_notes_with_metadata`) has **zero callers by construction** — the new surface reads note_meta via `execute_lens` only. Files deleted in §4.
- svelte-check 0 · cargo check green · binary rebuild in flight → **§2 Boss test next**.

## BOSS CORRECTION (Stop-On-Correction fired) — Option B REVOKED before the §2 test
> *"No, on second thought, if I want to have something similar I would use the Bases, the way I wanted. Let's re-write the Navigator concept, based on the original 'Note Navigator'. What we should do is to upgrade it to be smart, fast, and to launch from where others ended. It means that you have to do some digging and researching."*
- **Stopped the cascade** (§3–§5 cancelled). Changes-since-approval listed to Boss (§0 log, §1 dims, §2 swap, binary).
- **Corrected understanding (ratified by the rulings below):** a notes TABLE is what Bases are for — the Navigator stays the ORIGINAL two-pane notes-list paradigm, redesigned to be **smart + fast**, research-first: map where the world's best note-list tools ended (the namesake Obsidian plugin, Bear, Apple Notes smart folders, Craft, the PKM frontier) and launch beyond them, powered by what Constellation's index uniquely persists.
- **Boss rulings:** (1) **REVERT the §2 swap now** → done, `d03e2fd0` (old Navigator restored, BOTH mounts incl. the SS branch — the PJ-068 Navigator line returns to pending, tied to the new concept); (2) **KEEP §1** (the three lens dimensions serve all Bases). svelte-check 0 · cargo check green · binary relinking.
- Note: if a §2 click created `All Notes.base` in the universe, it remains as an ordinary user-owned base (harmless; delete at will) — the ensure command is reverted.
- **Research launched:** workflow `wf_50227623-e2e` — 5 researchers (the namesake plugin deep-dive · the note-list masters · the PKM frontier · interaction/perf patterns for huge lists · Constellation's launchpad inventory: what note_meta/review-pulse/strata/embeddings/tag_counts already persist to power "smart" facets) → frontier synthesis (table stakes / frontier / unsolved / ranked launch ideas). Deliverable: a rewritten Navigator concept paper for Boss ratification. MIG-090 docs stamped SUPERSEDED (Architect diagnosis §§1–5 stays valid input).
