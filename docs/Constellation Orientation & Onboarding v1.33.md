# Constellation — Orientation & Onboarding

**Version 1.33 | 2026-05-04**

> **What changed in v1.33** (same day as v1.32; Boss "Proceed all" cascade): **THREE more MIGs closed back-to-back — MIG-011, MIG-012, plus a pre-existing script-filter bug fix and the note-stage-taxonomy-decision queue.** The Index function went from "mentions-side cross-language" (v1.32) to a full vocabulary search engine across all three retrieval layers: literal substring (always-on), lexical-bridge (M11 corpus, 20K concepts × 15 langs), semantic (multilingual-e5-small ONNX embeddings).
>
> **Pre-existing script-filter bug fix** (`5dbb43f`): typing Arabic in the Index filter while script-tab "All" was active returned 0 results until the user bounced through "عربي" once. Two layers — substring-direction-mismatch (FTS5 stores stems shorter than typed surface forms; the bidirectional `query.includes(term)` check was gated on comma-mode-only) and stale-letter-filter persistence (clicking a Latin letter then typing Arabic dropped Arabic terms via the active letter filter). Both fixed; bidirectional substring is now always active and the letter filter auto-clears when filtered entries don't match it.
>
> **MIG-011 closed — cross-language Index *filter* expansion.** Mirror of MIG-010 applied to the search box: typing "knowledge" surfaces Arabic terms `معرف` / `علم` with `via knowledge` badges; typing `معرفة` surfaces English `knowledg` / `cognit` with `via معرفة` badges. New Tauri command `lexicon_expand_for_filter`; frontend per-keystroke debounce 300ms + cancel-token + per-session cache; same Settings toggle drives both surfaces (one mental model, two behaviors). 5 build commits + simplify + audit. Boss verified PASS at G2.
>
> **Side-discovery during MIG-011 G2 testing** (`c95a0e6`): two i18n keys (`indexPanel.returnToIndex` + 6 Living Link lifecycle stages under `notePane.stage.*`) were rendering as raw literals in the Arabic interface — and audit showed they were missing in **all 15 locales**. Backfilled with full ar+en + English placeholders in 13 others. The deeper question — should Notes use Living Link lifecycle stages (`spark/birth/growth/maturity/dormancy/archival`) or Zettelkasten stages (`fleeting/literature/permanent/synthesis`)? — queued as `project_note_stage_taxonomy_decision.md` for Boss design call.
>
> **MIG-012 closed — Index Search Engine: search history + semantic search.** Boss-approved Q1.A + Q2.C + Q3.B (term-level embeddings, lazy-on-first-semantic-query bootstrap, SQLite-per-Universe history). Two new tables (`term_embeddings`, `index_search_history`) with idempotent `CREATE TABLE IF NOT EXISTS` for transparent migration. 4 new Rust IPCs for embeddings (`init_term_embeddings` with progress events, `cancel_term_embeddings`, `search_terms_semantic`, `term_embedding_status`) + 3 for history (`read_index_history`, `write_index_history_entry`, `clear_index_history`). Frontend: 2 new Settings toggles + Clear button, per-keystroke debounced semantic search (mirrors MIG-011 pattern), filter loop now matches across direct → bridge → semantic with priority, `≈ similar` cyan badge for semantic matches, history dropdown on filter focus, full Arabic translation. 8 build commits + simplify + audit + confirm-dialog fix. Boss verified PASS at all three G2 stages.
>
> **§Build.8 simplify caught 3 Tier 1 issues** that would have shipped to users: (1) `init_term_embeddings` held `EmbeddingState.engine` and `SearchState.db` for the entire ~10–20 min embed-all loop, freezing every concurrent IPC during the job — fixed via lock-per-iteration. (2) f32 LE BLOB encode/decode duplicated between note + term + read paths — extracted `vec_to_blob` / `blob_to_vec` helpers; existing `constellation_embed_notes` migrated to use them too. (3) `TERM_EMBED_CANCEL` was a process-global static; moved to `EmbeddingState` for per-app-instance scope. The simplify methodology earned its keep on this MIG.
>
> **§Build.8-fix (`8d98a3a`)**: Boss G2 stage 1 step 6 surfaced that the browser-native `confirm()` dialog couldn't honor app i18n — both message text and OK/Cancel buttons stayed English even on the Arabic interface. Replaced with the existing `ConfirmDialog.svelte` component for the Clear-history button; Arabic users now see fully-localized "حذف نهائي... / مسح / إلغاء". Pattern for any future confirmation surface.
>
> **Boss-approved follow-on workstreams (logged 2026-05-04, NOT yet started)**:
> - Note-stage taxonomy decision (Living Link lifecycle vs Zettelkasten) — `project_note_stage_taxonomy_decision.md`. Quick i18n fix shipped today; deeper architecture decision deferred.
> - Auto-trigger semantic-init when toggle flips on — Plan-promised but currently the init must be invoked explicitly. Manual trigger via DevTools available (`init_term_embeddings`). Logged for Build.7-fix-1.
> - Search history toggle: track this with the rest of the deferred items in the existing backfill workstream.
>
> **Lessons logged this round (LL-025)**: simplify pass with parallel review agents earns its keep on cross-subsystem migrations. The lock-per-iteration find on MIG-012 §Build.8 would have shipped a real ~20-min freeze to Stage 2 testers without the simplify check — caught before binary release. Lesson: **for any migration that adds a new long-running background job, `/simplify` is mandatory before the Boss G test.** Adding to the standing migration checklist.

> **What changed in v1.32**: **MIG-010 closed — Lexical Bridge integration into the Index panel.** Boss directive: "finish and implement the Index function." Build cascade ran §A (Phase A bug fix — register `read_cooccurring_terms` in `tauri::generate_handler!`, the chip-strip cooccurrence panel was silently broken pre-MIG-010) → Architect doc → Plan doc → §Build.1 (`pub(crate)` bridge helpers + parameterize `find_match_via_marked` for STX/ETX vs `<mark>` delimiter regimes) → §Build.2 (`read_term_mentions` extended with `expand_cross_language: Option<bool>`; new `via_lemma: Option<String>` on IndexMention; `build_term_match_clause` helper with 4 unit tests) → §Build.3 (Settings: new "Index" section + `indexExpandCrossLanguage: bool` toggle in 15 locales) → §Build.4 (IndexPanel reads setting, renders `via_lemma` badge with `dir="auto"`) → §Build.4-fix (G2 cosmetics: off-state visual contrast + RTL toggle slider mirror; latent G3 fix attempted) → §Build.4-fix2 (defensive expansion fallback + frontend error catch — diagnostic infrastructure) → §Build.4-fix3 (the actual G3 root cause: `$effect` in IndexPanel read `mentionsCache.size` making the cache its own dependency → Rule 2 violation — wrapped cache reads in `untrack()`) → §Build.5 (`/simplify` three-agent pass — fixed Tier 1 prop-coupling via `cacheKey?: unknown` rename, Tier 2 `LexicalExpansion::into_parts()` accessor + `fts_quote_phrase` extraction + flatten `match` block + `prepare_cached` + gated `eprintln`, Tier 3 magic-pixel comment) + Phase 4 Audit doc.
>
> **Boss verified PASS at G2 + G3** — screenshot showed Arabic notes ("2007", "2010", "428 هـ") with **`via علم`** badges + Spanish-language reference ("Ada Lovelace") with **`via conocimiento`** badge. The 7,600-note mixed Arabic/English library is now searchable by *concept* across languages, not just by literal lemma. Audit at `lab/reports/MIG-010-AUDIT.md` confirms all 11 invariants hold.
>
> **Phase D (boot perf)**: deferred `readIndexEntries()` from `graphReady` to first Index-panel open. ~tens of ms saved on every boot for users who don't open the Index that session. Cost paid on demand.
>
> **Phase E (docs)**: dedicated Index help page at `docs/help.uConstellation.World/Index/Index.md`. User Manual §7 + Arabic User Manual §8 updated with cross-language toggle subsection. 13 other locale User Manuals queued in existing `project_user_manual_13_locales_backfill.md`.
>
> **Phase G (guidance)**: teaching doc `docs/help.uConstellation.World/Index/Index Guidance — How to Read Your Vocabulary.md` — three reads (frequency profile, language-pair balance, cognitive adjacency), five common patterns + readings, weekly-practice ritual. Boss-pattern teaching doc, modeled after the queued 360.3D Stratification Matrix guidance.
>
> **Lesson logged (LL-024)**: `$effect` body must declare its dependencies explicitly. The §Build.4-fix3 root cause (cache-invalidation effect tracked the cache it managed → infinite-clear loop) is a CLAUDE.md Rule 2 violation that I shipped without an end-to-end IPC trace. New rule: for any cross-subsystem `$effect` work, run a console-level trace BEFORE the Boss test cycle. Working Agreement #4 self-correction.
>
> **Boss-approved follow-on workstreams** (logged 2026-05-04, NOT yet started):
> - **MIG-011** — cross-language Index *filter* (mirror of mentions expansion, applied to the search box). Today the filter does substring matching only; bridge-aware filtering is the next step.
> - **MIG-012** (eventually) — Index search engine: search history + semantic search powered by existing `embeds.rs` ONNX pipeline. Memory: `project_index_search_engine_history_semantic.md`.
> - Pre-existing Index script-filter bug ("All" hides Arabic terms until "عربي" bounce). Memory: `project_index_script_filter_all_hides_arabic.md`.
> - "Rebuild Index" button — explicitly **deferred** per Rule 8 (no `rebuild_*` commands; FTS5 triggers maintain the index at write-time). Memory: `project_index_rebuild_button_decision.md`.
>
> **Phase C status**: Settings → Boot-perf scorecard turned out to be ALREADY shipped (5-criterion view in `SettingsModal.svelte`); STATUS.md was stale on this. Rebuild Index button deferred per above.

> **What changed in v1.31**: **MIG-008 closed.** Build cascade ran §145 (CreateItemDialog component + i18n en+ar) → §146 (wire New Folder) → §147 (wire New Note) → §148 (wire New Base, replace NewBaseDialog) → §149 (wire New Library + new `create_new_library_at` Rust IPC) → §150 (orphan sweep — five state vars + two functions + `NewBaseDialog.svelte` deleted) → §151 (Boss-flagged context-menu gaps: folder right-click missing "New Base" + library-row right-click falling through to browser-default menu — both fixed) → §152 (Build.7 /simplify checkpoint: i18n backfill 13 locales, `create_new_library_at` async, IME composition guard, KIND_LABELS lookup, `parseFrontmatter` instead of hand-rolled regex, dropped `defaultName` prop + `lastOpenState` $effect, `baseSelectedSet` for O(1) lookup, plus four Boss-approved adds — right-click "New note" now applies folder templates the same way the toolbar does, `/libraries` route migrated to the dialog, path-traversal hardening on Rust create IPCs via `sanitize_name`) + docs commit (User Manual + 2 help articles + Arabic User Manual) + audit doc. Boss verified PASS across all 8 create scenarios on the §151 binary plus the four §152-specific verifications (templates, route migration, path traversal, IME). Audit at `lab/reports/MIG-008-AUDIT.md` confirms all 11 invariants (I1–I11) hold. Project memory `project_create_dialog_standardize.md` marked SHIPPED.
>
> **Logged for follow-up**: 13 User Manual translations (`project_user_manual_13_locales_backfill.md`); reserved-Windows-name + trailing-dot/space hardening on Rust create IPCs (pre-existing gap, not MIG-008-introduced); collision popup (`project_rename_collision_popup_wanted.md`) — pre-existing, will compose with the dialog when shipped.

> **What changed in v1.30** (MIG-008 starts; §142–§144 closed MIG-006 §4):
>
> **MIG-008 — Create-Dialog Standardization (Phase 1 Architect committed at `22839d4`)**. Boss directive 2026-05-03: "Whenever I created a folder it is created in the respective location under the name 'New Folder'. It shouldn't work this way. What I want it to do is to follow the standard way of any file system. A popup dialog box should emerge to name the new folder and to choose the location. Same thing should happen when creating new note, base or library." Architect plan at `lab/reports/MIG-008-CREATE-DIALOG-ARCHITECT.md`. Inventory found four inconsistent create flows (Folder rejects collisions / Note auto-increments / Base has its own `NewBaseDialog` / Library has folder picker only); 11 invariants (I1–I11) defined; three options enumerated (A: shared modal, B: inline tree-row input, C: rich modal with templates); **Option A approved by Boss**. Phase 2 Build cascade kicks off in 8 steps (§Build.1–.8): build shared `<CreateItemDialog>` component → wire each of the four affordances → drop orphaned auto-create handlers → /simplify → audit. Each step pauses for Boss-testable verification clause.
>
> **MIG-006 §4 closed (§142 + §144)**. Original gap from §3-redo Stage 1 testing: Outgoing Links / Backlinks panels stayed stale after wikilink rename cascade because the SQLite index wasn't reindexed and the frontend's `allLibraryLinks` `$state` was loaded once at boot and never refreshed. **§142** plugged the Rust side (cascade walker calls `reindex_single_note` for each rewritten path; new `library_name` parameter on the `update_links_on_rename` IPC). **§143** attempted a frontend-side targeted update of `allLibraryLinks` but only matched entries whose `target` equaled the rename's `oldName` exactly — after several renames in a session (Hub v4 → v5 → … → v8) the in-memory state had drifted further than any single rename's `oldName`, so the targeted match never fired. **§144** superseded §143 with the simpler drift-resistant fix: re-fetch `cache_boot_snapshot_graph` post-cascade and replace `allLibraryLinks` + `notePathToAliases` wholesale. Catches not just the just-rewritten target but any drift accumulated in the session. Boss tested PASS — Outgoing Links panel updates immediately after rename, no app restart, no manual rebuild.
>
> **Side discoveries during §144 testing**: (1) Pre-§140 cid_cn collision found in Boss's SourceA test note (title: Hub v6, cid_cn matching Hub v8) — §140's check prevents NEW path-reuse contamination but can't retroactively heal already-corrupted files. Boss self-healed via delete + recreate. Logged for future sessions: a one-time scrub utility for existing libraries is queued. (2) Unlinked Mentions panel matches frontmatter alias entries — the scanner reads full file content (frontmatter + body) so YAML alias entries (`- "Hub v6"` from rename history) surface as "unlinked mentions". Logged in project memory `project_unlinked_mentions_alias_bleed.md`; pair with the existing `project_unlinked_mentions_double_count.md` in a single Unlinked Mentions cleanup MIG.
>
> **Boss agenda items added today** (queued, not in scope of any in-flight MIG):
> - Standard OS-style create dialog (greenlit → became MIG-008).
> - One-time scrub utility for pre-§140 cid_cn collisions in existing libraries.
> - Outgoing Links display case fix (`hub v8` → `Hub v8`; cosmetic).
> - Unlinked Mentions / frontmatter alias bleed (project memory above).
> - NSIS bundling lock investigation — recurring `os error 32` when Constellation is running during build; not a tooling bug per se but worth a workaround.

> **What changed in v1.29** (§135 + §136, same calendar day as v1.28):
>
> **§135 — `/simplify` checkpoint over §128-§134** (commit `fe9bf9e`). Three review agents (reuse / quality / efficiency) walked the MIG-006 §3 redo arc with Boss-supplied focus areas. Real-bug fixes shipped: refcounted `cascadingPaths` (Set → `Map<string, number>` so spam-renames in the same library don't pop each other's marks); killed the 1-second magic-timeout settle (orchestrator now `await`s `reloadTabsFromDisk(result.rewritten)` directly — real completion signal, no listener race, no wall-clock penalty on single-file renames); extracted `tabsInLibrary(libraryPath)` helper with separator-bounded prefix check (`/Foo/Bar` no longer falsely matches `/Foo/Bar2`). Efficiency wins: `reloadTabsFromDisk` batched + idempotent (parallel reads, single `openTabs.update`, skips bump when content matches); `watcher_suppress::was_recent` cheap-path lookup with opportunistic 256-threshold sweep (was O(N) `retain` on every watcher event); `CascadeResult.failed` capped at 100 entries with a `failed_truncated: usize` counter (defensive against pathological cascades bloating the IPC payload); consolidated `isCascading` WHY-comments at the three gate sites into one canonical docstring on `isCascading()` itself.
>
> **§142–§144 — MIG-006 §4 closed (write-time index propagation, both Rust + frontend halves)**. Boss surfaced the original gap in §3-redo Stage 1 testing: after rename, Outgoing Links panel kept showing the OLD target name (`foo`, lowercased) — the body cascaded but `note_meta.outgoing_links_json` and `note_links` weren't updated, so panels reading the index served stale data. **§142** plugged the Rust side: `update_links_on_rename` now calls `reindex_single_note` for each rewritten path after the cascade walk, with a new `library_name` parameter on the IPC. SQLite caught up. **§143** attempted a frontend-side targeted update of `allLibraryLinks` (the boot-snapshot `$state` the panels actually read from), but only matched entries where the in-memory `target` equaled the rename's `oldName` exactly — and after several renames in a session (Hub v4 → v5 → … → v8), the in-memory state had drifted further than any single rename's `oldName`, so the targeted match never fired. **§144** superseded §143 with the simpler drift-resistant fix: re-fetch `cache_boot_snapshot_graph` post-cascade and replace `allLibraryLinks` + `notePathToAliases` wholesale. Boss tested PASS on the §144 binary — Outgoing Links panel now updates immediately after rename. Closes the original Stage 1 observation. (§143's targeted update is left in the commit history as an "almost-fix" anchor — useful context for the next person who wonders why we don't do incremental updates.)
>
> **Tab/title corruption discovered + recovered during §144 testing**: a SourceA test file from earlier sessions had `title: Hub v6` AND a duplicate `cid_cn` matching Hub v8's identity — pre-§140 corruption that survived in the disk file. §140's `cid_cn` check prevents NEW path-reuse contamination but can't retroactively heal already-corrupted files. Boss self-healed by delete + recreate. Post-§140 the bug shouldn't reproduce on fresh notes. Logged for future sessions: existing libraries may carry pre-§140 cid_cn collisions; those need manual recovery (delete + recreate) or a one-time scrub utility.
>
> **Side discovery during §144 testing — Unlinked Mentions panel matches frontmatter alias entries** (logged: `project_unlinked_mentions_alias_bleed.md`). The scanner reads the full file content (frontmatter + body) when looking for the active note's name as a plain-text occurrence, so frontmatter `aliases:` entries surface as "unlinked mentions" of unrelated notes. Should split on the closing `---` fence. Pair with `project_unlinked_mentions_double_count.md` in a single Unlinked Mentions cleanup MIG.
>
> **§141 — `/simplify` checkpoint over §137-§140**. Three review agents (reuse / quality / efficiency) walked the §137-§140 diff. Real cleanups shipped: **(a)** new `normalizePathKey(p)` exported from `src/lib/utils.ts` — the `(p) => p.replace(/\\/g, '/').toLowerCase()` function was duplicated 7+ times across utils, store, and +layout. Single source of truth so a future filesystem-rule change (case-sensitive volumes, NFC normalisation) is one edit, not eleven. Every path-keyed Map operation now goes through this. **(b)** `WAB_LS_KEY = 'constellation-wab'` constant in store.ts — the localStorage key was hard-coded in five places. **(c)** Single `walkAuxStatePaths` walker shared by `migratePathKeyedAuxStateOnRename` and `clearPathKeyedAuxStateOnDelete` — both used to walk the same three structures (in-memory wab, in-memory recentWrites, localStorage wab) with identical norm-and-prefix matching. The walker passes the ORIGINAL key to the decide callback so folder-rename suffix preservation works on case-mixed Windows paths. **(d)** `openNoteTab`'s wab/disk choice extracted to `resolveNoteContent(filePath)` helper — the §140 inline check was three levels deep with three duplicated `clearWriteAhead` calls. The helper returns `{content, cursorPos, scrollTop}`: when wab is stale (cid_cn mismatch), drops the wab cursor/scroll too — they were for the OLD note, a subtle correctness improvement the inline §140 code missed. **(e)** `handleStageChanged(path, stage)` hoisted in +layout.svelte — the 3-line callback was inlined twice (main editor + split/second-screen path). **(f)** `extractCidCn` regex bounded to the first `---…---` frontmatter block — prior code matched against the full content, so a 10MB note made the lazy regex walk the whole body. **(g)** Stripped `// §139:` / `// §140:` inline anchor comments where they narrated what the code obviously does; kept multi-line docstrings on function declarations.
>
> **§140 — Cross-note content corruption via stale `writeAheadBuffer` (Rule 8 + the BUG-015 corruption class)**. Boss reported a **serious data corruption bug**: "Sometimes, when switching between notes after renaming or creating notes, I discover that a note replicates its contents, title, and cid_cn into another note. The victim note keeps its title in the file tree, but when I click it, it shows the culprit note (title, content, and properties)." Investigation pinpointed `writeAheadBuffer` (in-memory `Map<filePath, V>` + `localStorage` backup that survives app restarts). When a note is flushed, the editor's content is stashed under its file path so a later `openNoteTab` can substitute it for a disk read. **`renameItem` / `moveItem` / `deleteItem` migrate `openTabs.path` correctly but never touched the buffer** — so when a path was reused after a rename or delete (trivial with human-named notes: rename Foo → Bar, create new Foo, the new Foo lands at the old `…/Foo.md` path), `openNoteTab` hit the stale buffer entry and loaded the OLD note's content (cid_cn / title / body) into the new tab. The file tree kept showing the new note's title (driven by `display_title` from disk frontmatter — disk was correct) while the tab held the old note's content (in-memory only, until the user typed and triggered a `handleSave` that committed the corruption to disk too). Same Rule 8 / write-time-derivation gap §137 closed for `stageMap` / `maturityMap` — except corruption-class severity. §140 closes it three ways: **(1)** new helpers `migratePathKeyedAuxStateOnRename` and `clearPathKeyedAuxStateOnDelete` migrate / drop `writeAheadBuffer` + `recentWrites` entries (in-memory + localStorage backup), with folder-prefix support for folder rename / delete; **(2)** wired into `renameItem`, `moveItem`, `deleteItem`; **(3)** defense-in-depth in `openNoteTab` — when a wab entry hits, also read disk and compare the `cid_cn` signature; on mismatch, prefer disk and clear the stale buffer (handles historical localStorage entries from before §140). Self-healing via (3) for any user with stale localStorage from prior sessions.
>
> **§139 — Three production-binary bugs Boss caught (RTL arrows, recursive FileTree, SvelteMap reactivity)**. Boss installed the §138 production binary and reported three bugs from real-world testing: (1) Promote → / ← demote arrows inverted in RTL note context — the visual reading direction is right-to-left so `→` reads as "backward" in RTL; fix is to swap arrow characters when `dir === 'rtl'`. (2) Folder children in the file tree never receive `stageMap` / `maturityMap` — `<svelte:self>` recursion at `FileTree.svelte:102` was missing those two props from its prop list, so notes inside any folder rendered with default empty maps. (3) Promote/demote and "add Stage via property panel" updated the breadcrumb badge but **not** the file-tree emoji — the chain (handlePromote → onStageChanged → stageMap reassign) looked correct but the file tree didn't re-render. Root cause: the `$state(new Map())` + reassign-to-fresh-Map pattern has a Svelte 5 prop-propagation quirk visible specifically through this child-reads-via-prop path. Fix: switch `stageMap` and `maturityMap` to `SvelteMap` (Svelte 5's explicitly-reactive Map) — mutations are reactive at the operation level, no reassign-to-force-identity needed. Updated all six call sites (enrichNodesBackground, §138 toggleLibrary scans, §137 handleRenameComplete migrations, both onStageChanged callbacks) to use direct `.set()` / `.delete()`. New `migratePathKeyedMapInPlace<V>` helper in `src/lib/utils.ts` for SvelteMap targets in §137. `notePathToAliases` and `searchLinkCounts` stay on the original `$state(new Map())` pattern for now — narrow scope, only the user-visible drift surfaces converted.
>
> **§138 — Stage + maturity load on library expand (deeper Rule 8 fix)**. Boss tested §137 and reported: "the emoji is not visible, not before renaming or after it." The §137 path migration was correct but lit nothing because the upstream `stageMap` and `maturityMap` were both **empty on boot**. Audit found the cause: `enrichNodesBackground` (the only path populating these maps) was deliberately removed from the boot flow for boot-perf — comment at `+layout.svelte:2744-2757` explains "ZERO BOOT-TIME WALKS." Before §138, the only triggers were the Sky View legend's `onRequestEnrichment` button, the Settings → Rebuild Index path, and the first-ever-launch modal. None of those fire on a normal boot, so the file tree never showed stage emojis or maturity dots. §138 adds a third trigger: when the user expands a library in the sidebar (`toggleLibrary`, first-expand only), fire `scan_note_stages` + `compute_note_maturity` for that library and merge results into `stageMap` / `maturityMap`. Fire-and-forget so the expand isn't blocked; maps are reactive `$state` so the file tree re-renders when each scan returns. This respects the boot-perf discipline (no walks on boot) while restoring the Rule 8 expectation (every derived view present at the moment the user looks at it). Mutation guard: the merge only writes a fresh Map when at least one entry actually changed — Svelte doesn't fire spurious reactivity on no-op merges.
>
> **§137 — Rename propagates to path-keyed reactive state (Rule 8 reinforcement)**. Boss observation during Stage 5 testing: "we used to have the stage icon attached to the note title as a prefix — and we want Constellation to do it instantly when the user promotes, demotes, renames, or re-renames. That's why Constellation is unique and has its own prediction engine." Audit revealed: file-tree stage emoji + maturity dot + alias index + search-hub link counts are all `Map<path, V>` reactive `$state` in `+layout.svelte` (`stageMap`, `maturityMap`, `notePathToAliases`, `searchLinkCounts`). Promote/demote already kept them in sync via the `onStageChanged` callback chain; **rename did not**. After a rename, the renamed file's old path stayed in every map as an orphan, and the new path had no entry — so the file-tree showed the renamed note without its stage emoji until the next library scan. Direct violation of Rule 8 (Write-Time Derivation: "every computed view in Constellation is maintained at write time, not read time"). §137 adds `migratePathKeyedMap<V>(map, oldPath, newPath)` in `src/lib/utils.ts` (handles file rename, folder-prefix rename, and no-op canonical-file renames where the disk path stays the same; returns `null` to skip spurious reactivity when nothing migrated) and calls it from `handleRenameComplete` for all four affected maps. The renamed file's stage emoji, maturity dot, and aliases now follow the path the moment the rename lands.
>
> **§136 — Stage breadcrumb redesign + `handlePromote` cascade gate**. Boss observation: the breadcrumb Stage dropdown duplicated the property panel — same control, two surfaces. Homework on commit history showed why: the predecessor commit (`87d21d7`, CE Phase 6) added Stage to the breadcrumb as a one-click `Promote →` *verb* per `docs/CE-spec.md` Phase 6, then commit `6cbe87c` (40 minutes later) silently refactored the verb into a property-selector dropdown. Boss's "not LOGICAL" critique was reading the post-refactor state correctly. §136 restores the verb-distinct design: the breadcrumb now renders `[← demote] [stage badge] [Promote →]` with visual asymmetry — Promote prominent (accent border), demote subdued (faint arrow, no border, tooltip-only label). Demote is permitted (CE-spec one-way line was an oversimplification — knowledge revision is real research practice), but visually subdued to encode the frequency asymmetry. Removal of the stage property entirely stays in the property panel (verbs vs administration). Side fix: `NoteEditor.handlePromote` was the *other* drift surface the §134 audit missed — it bypassed the `isCascading` cascade gate the same way `PropertyEditor.saveTabContent` did. Added the gate at the top of `handlePromote`. Both stage-edit paths (breadcrumb verb + property panel) and both body-edit paths (`handleSave` + `handleFlush`) now share one consistent cascade gate. CE-spec Phase 6 updated to match (the "one-way" line is now historically annotated). i18n: added `notePane.demote` to all 15 locales; `notePane.promote` already existed from CE Phase 6.
>
> Stage 1-4 of the §3 redo Boss test cycle have all PASSED (basic cascade ✓, open-editor coherence ✓ — the headline win, pre-cascade-staleness ✓, multi-source watcher-loop ✓). Stage 5 (PropertyEditor / handlePromote cascade gate verification) and Stage 6 (spam-rename refcount) remain.

> **What changed in v1.28**: MIG-006 §3 redo lands clean (commits §128-§133). After the §115 attempt at §3-expanded ("open-editor coherence") burned BUG-015, MIG-006 §3 sat in `REVERTED` status for a week. Boss directed (via the 360.3D pattern) that a Concept Paper come first; that landed as §127 (`docs/Rename-Function-Concept-Paper-v1.0.md` + `lab/reports/MIG-006-3-REDO-ARCHITECT.md`). The redo itself shipped across §128-§133 as six landable steps + Phase 4 audit closure, all anchored to the eight P1-P8 invariants and Principle D6 (no `$effect` reads/writes value/editBody — that's BUG-015's class).
>
> **The redo (Concept Paper Option A — recreate via `{#key}` bump):**
>
> - **§128 (§3-redo.1)** — `flushAllTabsInLibrary(libraryPath)` helper in `store.ts`. Iterates open tabs in the affected library, writes any in-flight `writeAheadBuffer` content to disk via `writeNote`, marks each path as a recent write so the watcher's external-edit emit skips it. Closes F2-pre-cascade-staleness.
> - **§129 (§3-redo.2)** — new `src-tauri/src/watcher_suppress.rs` module: `mark(path)` / `was_recent(path)` with 2.5 s TTL. Cascade walker calls `mark` before each `fs::write`; the file watcher's emit path filters out recent writes. Closes F3-watcher-loop.
> - **§130 (§3-redo.3)** — `CascadeResult { rewritten, failed }` struct + `cascade:rewrote { paths }` Tauri event. Per Concept Paper D3, the cascade is per-file atomic but not transactional across files; failures collect into `result.failed` rather than rolling back successes.
> - **§131 (§3-redo.4)** — `OpenTab.reloadVersion?: number` field + `reloadTabFromDisk(path)` helper + `cascade:rewrote` listener in `+layout.svelte`. The listener re-reads each affected file from disk, updates `tab.content`, bumps `reloadVersion`. NoteEditor's `{#key}` includes `reloadVersion` so NotePane destroys + remounts with fresh content. Per Principle D6, this is the safe primitive — never an `$effect`-driven `view.dispatch`.
> - **§132 (§3-redo.5)** — `handleRenameComplete` orchestration: markCascading → flushAllTabsInLibrary → updateLinksOnRename → settle → clearCascading. NoteEditor's `handleSave` and `handleFlush` both gate on `isCascading(filePath)` and bail out for the duration. Closes F2-post-cascade-stomp.
> - **§133 (§3-redo.6)** — `/simplify` checkpoint cleanups: path normalisation in `cascadingPaths` Set + `flushAllTabsInLibrary` (Windows backslash vs forward-slash), parallelised `cascade:rewrote` listener (Promise.all), conditional 1 s settle (skip when `result.rewritten.length === 0`), opportunistic full-map GC in `watcher_suppress::was_recent`.
> - **§134 (§3-redo.7) — Phase 4 audit closure (this commit).** Three review agents found two HIGH/MEDIUM drift items shipped as fixes here:
>   - **PropertyEditor bypass (HIGH)** — `PropertyEditor.svelte` calls `saveTabContent` directly when the user edits a frontmatter property. Without an `isCascading` gate inside `saveTabContent`, a property edit during the cascade window would stomp the cascade's wikilink rewrite. Fixed by adding `if (isCascading(filePath)) return` at the top of `saveTabContent`. NoteEditor's gates on `handleSave`/`handleFlush` cover the body-save path; this gate covers the property-save path. Both routes now share the same protection.
>   - **Universe-switch leak (MEDIUM)** — `cascadingPaths` Set entries persisted across Universe switches. New `clearAllCascading()` helper called from `handleUniverseSwitch` so the new Universe starts with a clean slate.
>   - Concurrent renames + typing-during-cascade keystroke loss documented as known limitations; fixes deferred (concurrent renames need a `rename_id` serialization layer; keystroke loss is the input-block step that Concept Paper P4 explicitly accepts as out-of-scope for v1).
>
> **What MIG-006 §3 redo does NOT cover** (queued for §3-redo.8 onward, mapped to the original §4-§11 plan in `MIG-006-WIKILINK-CASCADE.md`):
> - Reindex via `index_note` (P7 — `note_links.target_name` reflecting disk).
> - Sync/async dispatch + progress events (P6 — hub-rename UX).
> - Atomic per-file writes via tempfile (P5 — kill-mid-cascade integrity).
> - Pre-MIG-006 backfill command for stale wikilinks.
> - Phase 4 audit (FULL — per-step audits ran inline; the cross-cutting audit happens at MIG-006 closure).
>
> **Migration table updated**: MIG-006 row now shows §1-§3 ✅ + §4-§11 ⏸.

> **What changed in v1.27**: Inline warning icons in matrix column headers (commit §125). Boss tested §124 on Abu Bakr and reported: "It is easy to identify the blind spot, but not the tensions. Is it in the Causes?" The §124 brown top border on Contradicts was being clipped by the matrix's `border-radius: 12px` + `overflow: hidden`. Boss's fix: "Maybe if we add the warning icons in their place, it will be easier."
>
> **§125 adds the same icon as the corresponding HUD chip directly above the column name** in the column header:
>
> - Blind spot column → ⚠ in red (alongside the existing full-red §122 treatment)
> - Fragile column (Derives From) → ⚠ in yellow
> - Tensions column (Contradicts) → ⚡ in brown (`#8b4513` light theme, `#c89875` dark theme)
>
> The icon is the primary signal; the §124 top border stays as a secondary cue (visible on middle columns even when the rounded corners clip the leftmost / rightmost). Visual continuity from HUD chip to column is direct: see ⚡ at the bottom, find ⚡ at the top of Contradicts.
>
> **No backend change in §125** — frontend template + CSS only.

> **What changed in v1.26**: Per-warning HUD chip colours + matching column-header overlays for fragile and tensions (commit §124). Boss confirmed §122 (red blind-spot column highlighting) on دمشق, then asked: "I want to have the same for the other warnings, like Orphan. But we have to choose a different color for each one."
>
> **Colour assignments**:
> - **Blind spots** (typed columns with 0 connections) — **red** (`var(--text-error)`). Existing §122 treatment; unchanged.
> - **Orphan** (no inbound links) — **orange** (`var(--color-orange)`). HUD chip only — no natural matrix counterpart, since "no one points at me" isn't a column-level signal.
> - **Fragile** (load-bearing on thin foundation) — **yellow** (`var(--color-yellow)`). HUD chip + 3 px yellow top border on the Derives From column header (the column whose under-population is what `single_point_of_failure` measures).
> - **Tensions** (active Contradicts links pointing at this note) — **brown** (Boss directive; brown isn't in the theme palette so hardcoded `#8b4513` for light theme and `#c89875` for dark theme). HUD chip + 3 px brown top border on the Contradicts column header.
>
> **Stacking precedence**: when a column is both a blind-spot and a fragile/tensions overlay candidate, blind-spot wins (red replaces everything). The `tensions-flag` and `fragile-flag` classes are only applied when `!isBlindSpot`. In practice tensions and blind-spot on Contradicts are mutually exclusive (tensions = inbound contradicts, which would make column count > 0); fragile + blind-spot on Derives From overlap only when the note has zero outbound derives-from while still being load-bearing-via-inbound — the red treatment is more important there.
>
> **No backend change in §124** — frontend CSS + classes only.

> **What changed in v1.25**: Stage 3.2 follow-up — blind-spot column highlighting (commit §122). Boss tested S3.2 on note دمشق, confirmed the column-totals row delivers the §4.2 Connection-Profile signal cleanly, then asked: "since the matrix identified the blind spots, it should highlight them within the matrix to help the user undertake the right measures."
>
> **Shipped in §122**: when a typed column's total is 0, the column header gets a warning treatment in addition to its normal type-coding:
>
> - Background gradient swaps from the soft type-colour tint (5%) to a `var(--text-error)`-mixed warning tint (14%).
> - Bottom border switches from the type colour to `var(--text-error)`.
> - The column name and the `0` count both render in `var(--text-error)`.
>
> Untyped is excluded from blind-spot detection — its 0 means "no plain wikilinks", not a typed-direction gap.
>
> Theme-aware via `var(--text-error)` (defined in `theme.css` as `--color-red`). With four-plus blind-spot columns, the visual is intentionally loud — the matrix is telling you which directions of reasoning haven't been declared for this note. The bottom HUD's `⚠ N blind spots` chip stays as a corroborating count.
>
> **No backend change in §122** — frontend CSS-only.

> **What changed in v1.24**: Three §120 retest follow-ups (commit §121). Boss flagged on the Arabic locale: (a) the `Untyped` column header still rendered in English because `typeLabels` derived skipped untyped via `if (lt === 'untyped') continue` — leftover from the §113 hardcode workaround; (b) the stage value `spark` (used in Boss's library) wasn't in the i18n stage map; (c) Arabic stratum-name terminology corrections — Boss's preferred terms: L3 رأي (vs قضية), L7 منظور (vs نموذج), L8 رؤية شاملة (vs رؤية كونية).
>
> **Three fixes shipped in §121**:
>
> 1. **Untyped column localized**. `typeLabels` in `Inspector360.svelte` no longer skips untyped — the loop now treats it uniformly, looking up `inspector360.untyped` (which §120 added to en + ar). With the §120 fallback chain, locales without that key fall through to en. Hardcoded English values stay as the final defensive fallback.
> 2. **`stage_spark` added** to en.json + ar.json. English: "spark"; Arabic: شرارة. Stage values are user-defined free-text (read directly from the YAML frontmatter `stage:` field by `extract_stage()` in `inspector360.rs`), so Boss's library uses lifecycle terminology beyond the four canonical Zettelkasten stages. Other lifecycle terms (birth/growth/maturity/dormancy/renewal) can be added on-demand if encountered.
> 3. **Arabic stratum corrections**: `stratum_name_3` قضية → رأي, `stratum_name_7` نموذج → منظور, `stratum_name_8` رؤية كونية → رؤية شاملة. Updated dependent help strings (`help_stratum_3/7/8`, `help_axis_stratum`, `help_dim_stratum`) to use the new terminology consistently.
>
> **No backend change in §121** — frontend + i18n only.

> **What changed in v1.23**: Three §119 follow-ups bundled (commit §120). Boss flagged on the §119 binary: (a) tooltip text for the dimension-strip `?` icons rendered ALL CAPS — inheriting `text-transform: uppercase` from the parent strip label; (b) tooltips near the right edge of the matrix were clipped because `transform: translate(-50%)` pushed half the tooltip off-screen; (c) "everything fully localized, like the Stratum, and the top row" — non-typed text in the matrix (stratum names, dim labels, maturity/origin/stage values, "Due", "Untyped") still rendered in English even on the Arabic locale, plus the new help text needed translations.
>
> **Three fixes shipped in §120**:
>
> 1. **HelpTip uppercase + edge-clip**. `.help-tooltip` now sets `text-transform: none` to override any uppercase ancestor; `font-weight: 400; letter-spacing: normal` for safety. `computeCoords()` clamps the tooltip's `x` coordinate to viewport bounds (190 px conservative half-width + 12 px margin), so triggers near the left or right edge no longer clip the tooltip.
> 2. **i18n fallback chain**. `t` derived in [`src/lib/i18n/index.ts:108`](src/lib/i18n/index.ts:108) now falls back to `en.json` when the active locale's lookup returns the literal key path (i.e. the key isn't in the active locale). Previously, missing keys in non-en locales returned the key string verbatim — a bug that forced the §104/§113 Untyped-label hardcode. With the fallback chain, missing keys display English instead, and partial translation stays graceful while translators backfill. Loaders cast each non-en locale through `unknown as typeof en` to bypass strict structural matching (the runtime fallback handles missing keys cleanly).
> 3. **Full Arabic + English localization of the matrix**. New i18n keys in `inspector360.*`:
>    - `untyped`, `stratum_name_1..8`, `dim_stratum/maturity/origin/stage/review/trails/lenses` (10)
>    - `maturity_seed/sapling/evergreen/canonical/wilting`, `origin_received/discovered/mixed/none`, `stage_fleeting/literature/permanent/synthesis/none`, `review_due/none` (16)
>    - `axis_stratum_label`, `axis_type_label` (2)
>    - `help_axis_stratum/type`, `help_stratum_1..8`, `help_type_*` (8), `help_dim_*` (7), `help_grand_total`, `help_hud_orphan/fragile/blind_spots/tensions` (4) — total 30 help strings
>    - All keys added to en.json (English source-of-truth) and ar.json (full Arabic translation, native-quality terminology). Other 13 locales fall back to English via the new chain — to be backfilled later.
>
> `Inspector360.svelte` updated: every previously-hardcoded label uses `tr($t(key), key, fallback)` where `tr()` is a small helper that returns the translation when present and the English fallback when `$t` returns the literal key. Static `STRATUM_NAMES`, `HELP_STRATUM`, `HELP_TYPE`, `HELP_DIM`, `HELP_GRAND`, `HELP_HUD`, `HELP_AXIS_*` constants removed; only `STRATUM_FALLBACK` retained as the in-component English fallback.
>
> **No backend change in §120** — frontend + i18n only.

> **What changed in v1.22**: Stage 3.1 follow-up — first-time-user `(?)` help affordances on the 360.3D matrix (commit §119). Boss S3.1 finding: "for the first-time user, we need to help them figure out what this matrix is all about. We need to explain each stratum, type, and/or every bit of detail within the 360.3D. By adding a (?) with each one of those elements."
>
> **Shipped in §119**:
>
> 1. **New reusable component** [`src/lib/components/HelpTip.svelte`](src/lib/components/HelpTip.svelte) — small `?` button that surfaces a styled tooltip on hover, and pins-on-click for accessibility / touch (outside-click dismisses). Tooltip uses `position: fixed` driven by `getBoundingClientRect()` so it escapes overflow boundaries. Theme-aware via `--background-secondary` / `--text-normal` / `--text-accent`.
> 2. **30 help markers wired** across the full-window matrix in [Inspector360.svelte](src/lib/components/Inspector360.svelte). Coverage:
>    - Corner cell: 2 (`▲ Stratum` axis legend, `Type →` axis legend)
>    - Column headers: 8 (one per typed direction + Untyped)
>    - Stratum row labels: 8 (L1 Datum → L8 Worldview)
>    - Dimension strip cells: 5 base + 2 conditional (Stratum, Maturity, Origin, Stage, Review, Trails, Lenses)
>    - Grand total Σ in the corner cell: 1
>    - HUD warnings: 4 (Orphan, Fragile, Blind spots, Tensions)
> 3. **Explanation text** authored as one-paragraph descriptions per element. Stratum text covers what kind of note lives at that altitude. Type text covers what the typed link asserts and shows the wikilink syntax. Dimension text covers the source-of-truth + how it's computed. HUD text covers when the warning fires and what it means cognitively. Axis-legend text in the corner cell explains how to read the matrix overall.
>
> **Compact scorecard untouched** — the sidebar widget is too narrow for `?` icons. First-time learning happens in the full-window matrix; once Boss is fluent, the scorecard reads at a glance.
>
> **No backend change in §119** — frontend-only.

> **What changed in v1.21**: Sky View inspect-mode lockout fix (commit §118). Bug Boss reported on 2026-05-01: in Sky View, click a node → app opens that note as a tab → close that tab via its own × (rather than via the "Return to Sky View" dismiss pill) → app locks; both sidebars refuse to open from their toggle buttons; only recovery is restarting the app.
>
> **Root cause**: clicking a Sky View node calls `handleSkyNodeClick` which (1) snapshots the current sidebar state to `sidebarSnapshots.get('skyInspect')`, (2) hides both sidebars, (3) sets `skyViewInspectMode = true`. The intended exit is a pill rendered next to the active tab — clicking its body returns to Sky View, clicking its `×` dismisses inspect mode and pops the snapshot. **But the pill only renders while `$activeTab?.path` is truthy** ([+layout.svelte:4439](src/routes/+layout.svelte:4439)), and the sidebar toggle handlers are guarded by `!skyViewInspectMode` ([+layout.svelte:1660-1661](src/routes/+layout.svelte:1660)). Closing the tab via its own × clears `$activeTabId` to `null` → pill disappears with the tab → flag stays `true` → toggles refuse to fire. Locked.
>
> **Fix shipped in §118**: a `$effect` in [+layout.svelte:586-590](src/routes/+layout.svelte:586) watches `skyViewInspectMode` and `$activeTabId`. When the tab goes null mid-inspect, it runs the same cleanup the dismiss × button runs — `popSidebars('skyInspect')` to restore the pre-SV sidebar layout, then sets `skyViewInspectMode = false`. Tab-close-via-X now exits inspect mode cleanly. Frontend-only fix; the dismiss pill itself is unchanged for users who use the intended path.

> **What changed in v1.20**: Verification B Check-2 follow-up (commit §117). Boss accepted §115's column-header text colour change but flagged the background tint as still too strong: "lower the tinted background more." §117 reduced the tint from 10 % type-colour mix to 5 %. Text colour and bottom-border colour kept the §115 values. One-liner CSS change.

> **What changed in v1.19**: Verification A retest fixes (commit §116). Boss tested the §115 list-of-titles and surfaced two issues:
>
> 1. **Cell expansion persisted across navigation.** Click a list item → matrix moves to new note → previously-expanded `(stratum, type)` cell stayed expanded on the new note. Boss: "It should collapse by default when we move to another node." Same on back-bar return: "When we are back, it should collapse automatically."
> 2. **Untyped should be expandable too.** Boss originally directed (S1.3.5 in §114) to exclude Untyped because dot-grid expansion at 800+ would balloon the matrix. §115 reworked expansion as a scrollable title list, which contains the size cleanly. Boss: "Let's have the 'untyped' expandable like the other type."
>
> **Fix shipped in §116** (frontend-only):
>
> 1. **Auto-reset on navigation**: a `$effect` watches `data?.note_path` and resets `expandedCells = new Set()` whenever it changes. Covers both forward (title-click → onNoteClick fires → parent updates `data` → effect runs → state clears) and backward (back-bar → onBack restores prior `data` → same path).
> 2. **Untyped exclusion removed** from `toggleCellExpand` and the template branch. The `+N` chip on Untyped is now a clickable button just like the seven typed columns. The list view caps at 240 px with internal scroll regardless of count, so Untyped's typically-large overflow is contained.
>
> **No backend change in §116** — frontend-only.

> **What changed in v1.18**: Stage 1 + Stage 2 retest follow-up bump (commit §115) — six refinements bundled into one rebuild after Boss walked all 6 + 6 sub-stages of the matrix tutorial.
>
> **Six fixes shipped in §115** (frontend-only):
>
> 1. **Expanded typed-cell renders as a list of note titles, not more dots.** S1.3.5 surfaced this: when the user clicked `+N` on a typed cell, §114's design just showed all the hidden dots — visually overwhelming for cells with 30+ connections, and the user still had to hover each dot to learn the name. New design: clicking `+N` switches the cell into a **vertical list of note titles**, each clickable to navigate. Dot bullet shows the type colour beside each name.
> 2. **Always-visible `×` collapse button** at the top-right of the expanded list. Replaces §114's `−` button which was at the *end* of the dots and easy to miss when the cell scrolled. Now positioned absolutely so it stays visible regardless of scroll.
> 3. **Max-height + internal scroll** (240 px) on the expanded list so very large typed cells (e.g. Abu Bakr's L7-Supports with 49 connections) don't balloon the row past the canvas. List scrolls inside the cell.
> 4. **Active-note name chip removed** from the row label. The note's name is already visible in the matrix header at the top; repeating it on the active stratum row was redundant. Active row is still highlighted in the theme accent (purple band + accented row number) — that signal is preserved.
> 5. **Column-header text contrast.** §113's gradient used 22 % type-colour tint with text in the same hue, which read as colour-on-same-colour. Reduced tint to 10 % and switched text colour to `color-mix(var(--col-color) 55 %, var(--text-normal))` so text stays type-coded but lifts off the background. Bottom border keeps the full-strength type colour for the visual signal.
> 6. **Grand total visible** in the top-right corner cell (the row-totals header). New layout stacks `Σ` symbol over the matrix-wide grand total of all (deduped per cell) connections. Confirms at a glance that column-totals sum equals row-totals sum equals this number.
>
> **No backend change in §115** — frontend-only. The §112 backend (`stratum: u8` on `LinkedNote` + `precompute_all_strata`) stays as-is.

> **What changed in v1.17**: a Stage 1.2 retest fix bump (commit §114). The §113 "2× sizes" directive overshot for the full-window matrix — Boss confirmed S1.1 (compact scorecard) but flagged S1.2 (full-window matrix) with two findings: "Minimize by 1" (sizes too big) and "L1 missing, L2 cut" (the bottom of the matrix was clipped by `overflow: hidden` because 8 rows × 110 px row-min exceeded the canvas height).
>
> **Fix shipped in §114** (frontend-only, full-window only — compact scorecard untouched):
>
> 1. **Full-window matrix scaled down ~25 %.** `360.3D` label 32 px → 24 px, brain icon 56 px → 40 px, header name 44 px → 32 px, strip label 22 px → 16 px, strip value 30 px → 22 px, column name 18 px → 14 px, column count 26 px → 20 px, row num 26 px → 20 px, row name 24 px → 18 px, active chip 20 px → 15 px, HUD font 28 px → 21 px, dot 16 px → 13 px. Padding tightened to match.
> 2. **Cell row min reduced from 110 px → 78 px** (and column min 120 px → 96 px, row-label column 280 px → 220 px, row-total column 100 px → 76 px). All 8 stratum rows now fit in a typical 1080p viewport without clipping.
> 3. **`min-height: 0`** on `.i360-matrix-wrap` so the matrix can shrink in tight viewports rather than getting clipped.
>
> **Compact scorecard unchanged**: Boss explicitly passed S1.1 at the §113 sizes (1.85rem name, 1.4rem pills, 14 px bar height), so those stayed.

> **What changed in v1.16**: a Stage-1-tutorial fix bump for the §112 Stratification Matrix (commit §113). Boss walked S1.1 → S1.6 in sequence and recorded seven refinements; all of them landed in one rebuild rather than commit-per-fix.
>
> **Fixes shipped in §113**:
>
> 1. **`Untyped` label hardcoded** in both the compact bar chart label and the matrix column header. The §104 fix had been preserved across the spherical line until §112 reverted it, and the i18n-key leak (`inspector360.unty…`) returned in Stage 1.1. The fix is the same as §104's: `$t('inspector360.untyped')` returns the literal key string when the translation is missing, which is truthy, so the OR fallback never fires; hardcode `'Untyped'` for that one type, keep `$t()` for the seven typed directions where the keys exist in en.json.
> 2. **Compact bars switched from max-normalised to percent-of-total.** Boss's "Abu Bakr" test note had Untyped=6,107 vs Supports=101 — max normalisation collapsed every typed bar to ~1.6% width and made them invisible. Each bar now fills its share of total connections and the right-hand number reads `X.X%` (or `—` for zero). The shape of the share, not the absolute count, carries the cognitive signal.
> 3. **Compact scorecard text and figures roughly doubled.** Card name 0.95rem → 1.85rem, pills 0.72rem → 1.4rem, bar height 8 px → 14 px, label column 90 px → 130 px, count column 28 px → 60 px to fit `100.0%`.
> 4. **Full-window background and chrome are now theme-aware.** Hardcoded `#060612` / `#0a0a1c` / `#060614` and `rgba(255,255,255,0.X)` greys replaced with `var(--background-primary)`, `var(--background-primary-alt)`, `var(--background-secondary)`, `var(--text-normal)`, `var(--text-muted)`, `var(--text-faint)`, `var(--text-accent)`, `var(--background-modifier-border)`. Active-row purple now derives from `--text-accent` via `color-mix`, so it follows the theme accent instead of locking to a single hex.
> 5. **Full-window `360.3D` header label doubled** (16 px → 32 px). Brain icon 28 px → 56 px. Active-note name 26 px → 44 px.
> 6. **Full-window matrix text and figures doubled.** Strip labels 11 px → 22 px, strip values 16 px → 30 px. Column headers 10 px → 18 px. Column counts 14 px → 26 px. Row labels 13 px → 24-26 px. Active chip 11 px → 20 px. HUD text 16 px → 28 px. Dot size 11 px → 16 px (subset; doubling fully would break 16-dot density per cell). Cell row height 72 px → 110 px. Row-label column 200 px → 280 px; row-total column 64 px → 100 px; column min 80 px → 120 px.
> 7. **Hover label moved from the fixed top-right of the matrix to a floating tooltip that sits directly above the hovered dot.** The previous placement (which I'd justified as "doesn't follow mouse, doesn't pop chrome on dense rows") forced the user to look away from the dot they were hovering. New placement uses `position: fixed` driven by the dot's `getBoundingClientRect()` so it escapes `overflow: hidden` on the matrix and works regardless of cell layout.
>
> **No backend change in §113** — frontend-only. The §112 backend (`stratum: u8` on `LinkedNote` + `precompute_all_strata`) stays as-is.
>
> **Process note**: I bundled S1.1 through S1.6 into one tutorial message and Boss flagged the staging violation early. The remaining sub-stages were sent one at a time (S1.2 alone, then S1.3, then S1.4, etc.). `feedback_staged_tests.md` interpreted strictly going forward — one focused test per turn, never a numbered list of tests in a single message.

> **What changed in v1.15**: the 360.3D Inspector redesign lands as code (commit §112). The concept paper (v1.0) was approved; the clean-slate redesign is the **Stratification Matrix**.
>
> **The matrix in one sentence**: an 8 × 8 grid where the **vertical axis is stratum** (L8 Worldview at the top → L1 Datum at the bottom) and the **horizontal axis is link direction** (the 7 typed directions + Untyped). Each connected note becomes a small dot in the cell at the intersection of its own stratum and the typed direction it shares with the active note. The active note's row is highlighted; **empty cells are visually present** (diagonal stripes) so absence reads as readily as presence — Concept Paper §4.3 "Absence is first-class."
>
> **Why this is the right shape (vs spheres / sectors)**: stratum is the dimension Constellation alone measures, and the matrix puts it on the dominant visual axis (vertical position = altitude in the knowledge hierarchy). Typed direction now has its own dedicated lane instead of competing with stratum on a polar layout. Counts read at a glance: column totals tell you which directions you over- or under-use; row totals tell you which strata your thinking spans. Gaps (empty rows = strata you haven't reached; empty cells = directions you don't use at this stratum) are part of the geometry, not afterthoughts.
>
> **Backend addition** ([`inspector360.rs`](src-tauri/src/inspector360.rs)): `LinkedNote` now carries `stratum: u8`. A new `precompute_all_strata()` helper computes every note's stratum once at the top of `get_360_view`, building an inbound-count + sources-of map up front so each `LinkedNote` can be stamped in O(1). Total cost stays O(N + total_links) — same big-O as before. The same rule set used for the active note (`compute_stratum_for_note`) is reused for connections.
>
> **Frontend rewrite** ([`Inspector360.svelte`](src/lib/components/Inspector360.svelte)): the spherical line — `SECTOR_MAP`, `polarToXY`, the three viz-mode toggle (Atmospheric / Neural / Cosmic), `ringsLayout`, `layoutMode`, `allNodes`, `vizMode` — is gone. Full-window mode is the matrix on an HTML/CSS grid (no SVG polar coordinates). Compact sidebar is now a **scorecard**: note name + stratum pill + maturity pill + ↑outbound/↓inbound/word counts + a per-type bar chart with explicit "—" markers for blind spots + a flags row. The matrix is too dense for a 280 px-wide sidebar; the scorecard is the right read at that scale.
>
> **Preserved from §107 / §109**: hover-only labels (no always-on names cluttering pattern reading), per-render `uniqueId` keying so empty-path collisions don't multi-highlight, multi-hop back-stack for click-to-navigate. Universe switch still resets the back-stack to `[]`.
>
> **Dropped permanently**: `vizMode` dropdown, polar / angular layout primitives, `SECTOR_THRESHOLD` hybrid logic, depth-based ring assignment, count-based ring assignment. The §110 binary (the previous "final iteration" of the spherical line) is no longer the latest runnable Inspector — the §112 binary is.

**Author of facts: Eisa ALSHAMSI (project owner, designer, IT Boss).**
**Maintainer: Claude (consultant / engineer / SME).**

---

## 0. How to use this document

**This is the first document any new Claude session reads.** It exists so a fresh AI can get to architectural fluency in one read instead of rediscovering the project from `git log` + screenshots over several frustrating turns.

**Maintenance is a Standing Order** (`CLAUDE.md` Standing Order #6). Whenever a fact below changes — a phase ships, a rule is added, a doc-drift item is fixed, a migration closes — update this file in the same commit that lands the change. Bump the version when the structure changes; date-stamp every section that updates. **The filename always carries its version suffix**: `Constellation Orientation & Onboarding v1.0.md`, `... v1.1.md`, `... v1.2.md`, etc. **Each new version is written as a NEW file alongside the existing ones — older versions are NEVER deleted.** They remain in `docs/` as a historical record the project owner uses to track how orientation evolved. A new session reads only the highest-version file, but the trail behind it is durable.

**This document is grounded.** Every claim cites the authoritative source (file:line, commit hash, or session log section). When two project documents disagree, I name both and don't pick a winner unless code-reading resolves it. When I don't know something, I say so explicitly in §17.

**Hard rule for every reader (human or AI) of this file**: if you find this document contradicts the actual codebase or a more recent session log, **trust the code and the session log first**, then update this file in the same session.

### v1.14 changelog (vs v1.13)

v1.14 was a clean-slate reset for the 360.3D Inspector (commit §111) on 2026-04-30. After five attempts (§104, §106, §107, §109, §110) at the spherical / orbital / compass-position layout — exceeding LL-014's three-attempts rule — Boss invoked the rule and directed a return to first principles.

Two artefacts shipped in §111 (no code change):

1. **Concept Paper v1.0** — `docs/360.3D-Concept-Paper-v1.0.md`. Defines what 360.3D is, why it exists, what cognitive dimensions it encodes, the three outputs the user should leave with (Position / Connection Profile / Absence), the eight design principles any 360.3D visualisation must satisfy, and what 360.3D is NOT (vs Sky View, Map, Sight, Index, OrgChart). Recommended starting axis: **stratum**.

2. **Orientation v1.14** — captured the reset and the pending clean-slate redesign.

The redesign itself shipped in §112 — see v1.15 above.

### v1.13 changelog (vs v1.12)

v1.13 was a sector-layout fix (commit §110) on 2026-04-30. The §109 depth-based rings didn't help "1902"-class data because `inspector360.rs::get_360_view` stamps every outbound and inbound link with `depth = 1`. §110 replaced depth-based with count-based ring assignment: typed groups sorted by count, distributed across the inner two rings (smallest typed → inner 160, largest typed → middle 270); untyped always on the outer ring 380. Three reliably distinct rings, no typed/untyped collision. **§110 is the final iteration of the spherical layout line — see v1.14 for the clean-slate reset.**

### v1.12 changelog (vs v1.11)

v1.12 was a sector-layout course-correction (commit §109) on 2026-04-30. **Restored depth-based sector rings** `[160, 270, 380]` (matching the compact widget). Each typed group's nodes cluster at their SECTOR_MAP compass angle with the widget's 8°-per-node spread; ring radius determined by note depth. **The §109 fix was insufficient for "1902"-class data** because the IPC always stamps typed links with depth=1, so every typed node piled onto the inner ring 160 and untyped depth-1 collided with them. §110 (v1.13) corrected this with count-based ring assignment.

### v1.11 changelog (vs v1.10)

v1.11 was a Stage 2B retest follow-up (commit §107) on 2026-04-30. Boss reported two findings on the v1.10 binary.

Two changes in §107:

1. **Single-ring sector layout** (interpreting "Distribute all nodes in one circle"): replaced §106's three depth-based rings with a single ring at `SECTOR_RADIUS = 290`. **This was an over-correction; §109 restored depth-based rings.**
2. **Hover label leak fix**: each rendered node now carries a `uniqueId`; hover state renamed `hoveredNode → hoveredId` keying on it instead of `node.path`. Fixes the empty-path collision (`inspector360.rs::get_360_view` returns `path: ""` for outbound links to notes outside the library). **This fix is preserved post-§109.**

### v1.10 changelog (vs v1.9)

v1.10 was a tuning bump for the Stage 2B sector layout (commit §106) on 2026-04-30. Boss reported during Stage 2B retest that the §104 sector mode rendered the test note "1902" too sparsely on the full-window canvas. Boss directive: "It has to be similar to the widget."

Two changes in §106:

1. **Sector spread formula switched** from §100's normalised cap to **the compact widget's exact formula** `(i - (n-1)/2) * 8`. Trade-off: large sectors bleed past their 50° semantic slot into adjacent compass directions. The widget shows this; Boss accepted.
2. **`SECTOR_THRESHOLD` raised** from 8 → **30**. Notes with up to 30 typed-link connections per group now use sector layout; Abu Bakr-class hubs still trigger ring-per-group.

### v1.9 changelog (vs v1.8)

v1.9 was a **CE Phase 12 hardening / refinement bump** (commits §96–§104, ten commits since v1.8 closed) on 2026-04-30. Phase 12 became user-testable on 2026-04-29; Boss tutorial-tested it across Stage 1 and Stage 2 over two days, and every iteration rolled into a fix-and-rebuild loop. Net result: the 360° Inspector surface that v1.8 announced as "enabled" is now the surface the Boss is actually using.

Highlights:

1. **Stage 1 hotfix (§96)** — clicking the new right-sidebar 360° tab routed the user back to Properties because a safety `$effect` (`+layout.svelte:1255`) was force-resetting `rightSidebarTab` to the first known visible tab. The `tabVisible` map and fallback `order` array missed `inspector360`. Fixed; tab now sticks.
2. **rs-tabs strip overflow fix (§97)** — adding the 11th tab pushed past the default 340 px sidebar width; the new tab clipped at the right edge. Pure CSS: replaced default `<button>` padding with explicit `padding: 0; flex: 1 1 28px; min-width: 24px; flex-wrap: wrap;`. Tabs now wrap to a second row instead of clipping.
3. **Compact-mode back-nav (§98 → §99)** — Boss requested a "back to source note" affordance inside the compact widget. Started as single-step (§98) then upgraded to a **multi-hop stack** (§99) per Boss directive: walks all the way back through any chain. State: `inspector360BackStack: $state<Array<{path, name}>>`. Universe switch resets the stack to `[]`.
4. **Stage 2 omnibus (§100)** — five Stage 2 findings: dock-button tooltip i18n leak (`ribbon.inspector360` key returned verbatim because `$t()` returns the key on miss); viz didn't fill canvas (removed `max-width: 1400px; max-height: 900px;` from `.i360-viz`); side panels + HUD doubled in size; tighter sector grouping `(i / (n-1) - 0.5) * 50`; full-window auto-close removed in favour of "Return to {previous}" header button.
5. **Sector → ring-per-group → hybrid (§101 → §102 → §104)** — three iterations on visualisation layout. §104 made the choice automatic: sector layout when max typed-group count ≤ `SECTOR_THRESHOLD = 8`, ring-per-group when above.
6. **Minimised nodes + hover-only labels (§103)** — node radii reduced 10/7/4 → 6/4/3. Always-on labels removed; hover-only with 13 px font + 3 px black SVG stroke. 6 px invisible hit-area expansion.
7. **Dedupe by path + Untyped label fix (§104)** — frontend dedup per-group in `ringsLayout` (the IPC returns the same note from outbound + inbound + second-order). Untyped label hardcoded `'Untyped'` to skip the broken i18n fallback.

**Boss's perf verdict on Phase 12**: first-fetch "almost instantly". **MIG-010 priority dropped to LOW** based on lived experience.

**Process violations recorded for the day**: (a) the over-long Stage 2 tutorial bundled 2.1–2.7 in one message — `feedback_staged_tests.md` rule. (b) Standing Order #6 violation: §96–§104 shipped without bumping the orientation in the same commit. **v1.9 was the catch-up bump.**

### v1.8 changelog (vs v1.7)

v1.8 captured three landings on 2026-04-29:

1. **MIG-003 integrated to main** via fast-forward of `claude/frosty-stonebraker-75c9bf` (the side branch that closed MIG-003 on 2026-04-28 but was never merged). `origin/main` moved from `6545b3e` (MIG-008/009 tip) to `8cb80ac` (MIG-003 handover). Three byte-identical "stranded" closure docs in main's working tree (the v1.7 file, SESSION-LOG-2026-04-28.md §85–§89, CANONICAL-FILENAME-ARCHITECTURE.md updates) became tracked. Source ↔ binary parity restored at main by copying the post-MIG-003 release artifacts from the frosty worktree.
2. **CE Phase 12 360° Inspector re-enabled** (§93 + §94 + §95). Backend `get_360_view` IPC was already shipped from earlier work; only the import + UI wiring was gated at `+layout.svelte:84`. Re-enable shipped both surfaces: a compact right-sidebar tab and a full-window overlay reachable from a new ribbon-dock button. IPC fetch debounced 200 ms with sequence-guard + last-fetched-key dedup; lazy-mount via `inspector360EverOpened`. The `get_360_view` IPC walks the full library on every call (acknowledged Rule-8 violation); MIG-010-scale work to cache `note_360_view` was queued, contingent on Boss's perf verdict.
3. **CE Phase 9 Multi-Lens approved for re-wire on Path B** (Rule-8 compliant) — queued after MIG-006 §3 redo. `lenses.rs::apply_lens` stays dead until that future MIG-010-scale migration.

### v1.7 changelog (vs v1.6)

v1.7 captured MIG-003 closure (Human-name Filenames) on the side branch `claude/frosty-stonebraker-75c9bf`. § 6 fully rewritten to reflect the inverted architecture: `cid_cn` is the immutable internal id (frontmatter only), filenames are human-readable. § 8 migration table updated to mark MIG-003 closed. The Canonical Filename Architecture design doc was given a Post-MIG-003 historical banner. Visible behavior change: every `.md` file on disk now has a human title as its filename; renames cascade through every dependent table (`note_meta`, `note_links`, `sky_nodes`, `note_aliases`, `note_embeddings`).

**Important context for any reader of v1.7**: at the time v1.7 was written, the seven MIG-003 commits + this v1.7 file itself + the closure session-log entries + the CANONICAL-FILENAME-ARCHITECTURE.md updates **only existed on the `claude/frosty-stonebraker-75c9bf` branch and as uncommitted/untracked files in `main`'s working tree**. They were not on `origin/main`. The stranded state was discovered and resolved at the start of the 2026-04-29 session via `git merge --ff-only` (see v1.8 note above). v1.7's "MIG-003 closed" claim was correct — but only on the side branch; the main-line integration arrived a day later.

### v1.6 changelog (vs v1.5)

v1.6 captures two cleanup migrations shipped on 2026-04-27 / 28:

**MIG-008 — Canonical Naming Cleanup** ✅ closed.

- Added shared helper `note_display_name(path, content_opt)` in [`libraries.rs`](src-tauri/src/libraries.rs) — smart enough to skip the file read for human-named files (file_stem IS the title) and only pay the I/O cost for canonical-named files.
- Patched ~14 sites across `map.rs`, `inspector360.rs`, `strata.rs`, `maturity.rs`, `provenance.rs`, `review.rs`, `lenses.rs`, `tasks.rs`, `tension.rs`, `libraries.rs::scan_index_words_recursive`, `trails.rs::find_note_recursive`, `universe.rs::collect_templates_recursive` — all switched from `path.file_stem()` to the helper so user-visible labels show frontmatter title instead of canonical filenames.
- Two of those changes are **correctness fixes**, not just label fixes: `inspector360.rs:88` (now matches incoming wikilinks for canonical notes) and `trails.rs::find_note_recursive` (canonical notes were unfindable by name lookup).
- User-verified across Stages 1, 3, 4a/4b, 5 (Constellation Map, Strata + Maturity + Provenance, Tasks, Review Pulse, Tension via Health). Stages 2 (Inspector 360) and 4c (Multi-Lens) skipped — surfaces are deliberately disabled or dead in current builds (see below).
- Phase 4 audit clean: invariant check / drift check / migration-path check all PASS.

**MIG-009 — Lens-to-Sight Naming Cleanup** ✅ closed.

- Renamed `src-tauri/src/lens.rs` → `src-tauri/src/sight.rs` to align the analytics module's filename with its UI surface (Constellation Sight, formerly Constellation Lens).
- Renamed Tauri commands: `constellation_lens_centrality` → `constellation_sight_centrality`, `constellation_lens_tag_edges` → `constellation_sight_tag_edges`. Frontend `+layout.svelte:3235` invoke updated atomically.
- Frontend JS variable names (`lensActive`, `toggleLens`, `lensCentrality`, `lensCommunities`, `lensCommunityAssignments`, `lensGaps`, `lensHealth`, `lensLoading`, `lensDataStale`, `availableLenses`, `activeLensId` — ~60 occurrences) intentionally **not** renamed; deferred as bookkeeping with no architectural payoff.
- `src-tauri/src/lenses.rs` (plural — CE Phase 9 Multi-Lens) **NOT renamed** — separate concern, deferred to whenever CE Phase 9 is resumed (see "dead-code finding" below).
- User-verified: Constellation Sight still renders centrality + community + gaps after rebuild.

**Dead-code finding** (catalogued, not fixed in this bump):

- `lenses.rs::apply_lens` has **zero frontend callers**. Verified by exhaustive grep on 2026-04-27. The Settings UI can still create + save lens definitions via `list_lenses` / `save_lenses`, but those definitions are never applied to anything. The Multi-Lens (CE Phase 9) IPC pipeline is dead-on-arrival.
- Decision deferred: either delete `lenses.rs` + the Settings lens-definition UI, or re-wire `apply_lens` into a real surface (Sight or a separate panel). Tracked in `project_lenses_apply_lens_dead_code.md` memory.
- MIG-008's patches to `lenses.rs::scan_property_recursive` and `scan_tags_lens_recursive` ship harmlessly but don't run today. Don't revert; the code is correct should the wiring be restored.

**UI / surface notes locked into memory this session:**

- Constellation Lens / Multi-Lens UI surface was renamed to **Constellation Sight** earlier (`feedback_lens_renamed_to_sight.md`). Internal Rust file was just renamed to match (MIG-009).
- 360° Inspector frontend component is deliberately disabled at [`+layout.svelte:84`](src/routes/+layout.svelte:84) — Rust backend (`inspector360.rs`) ships ready, but no UI surface mounts it today.

**New backlog items**:

- Decide fate of CE Phase 9 Multi-Lens (delete vs re-wire). Tracked.
- Decide fate of CE Phase 12 360° Inspector (re-enable vs withdraw).
- `docs/IPC-CONTRACT.md` is now even staler — missing the `constellation_sight_*` rename. Doc-drift item.

### v1.5 changelog (vs v1.4)

v1.5 is a focused-fix bump for the Unlinked Mentions panel (item 6 from the option-(e) backlog). User-verified 2026-04-27 ~18:00.

**§90 — Unlinked Mentions panel: scanner fix + frontmatter-title label**

Two bugs in `scan_unlinked_mentions` ([`libraries.rs:1665-1759`](src-tauri/src/libraries.rs:1665)) closed in one commit:

1. **Scanner false-positive on typed/aliased wikilinks.** The previous "skip source if `[[NoteName]]` substring is present" check was too narrow — every typed-link form `[[NoteName|supports]]`, every alias form `[[OldTitle]]`, and every embed `![[NoteName]]` slipped past it. The active note's title would then be matched as plain text *inside the wikilink markup* and counted as an unlinked mention. Fix: strip ALL wikilinks (regular + embed forms) from content before plain-text scanning. The regex `!?\[\[[^\]]*\]\]` removes them all in one pass.
2. **Source-row label was canonical filename, not human title.** Filename for canonical notes (`20260426T140940Z_NOTE_11B4`) is unreadable; users couldn't tell which note was being shown. Fix: prefer `extract_frontmatter_title()` (already used by the rename path), fall back to `path.file_stem()` only when title is missing.

**Side benefit.** Both fixes are upstream in Rust, so any future caller of `scan_unlinked_mentions` automatically gets correct behavior. No frontend changes needed; the existing `BacklinksPanel.svelte` Unlinked-Mentions section renders the corrected data unmodified.

**What this closes from §12 / §13 / backlog**:
- Item 6 (Unlinked Mentions double-count + canonical filename label) — both bugs fixed.
- The "(e) didn't fully cover item 6" gap I owned in v1.4 — now closed.

**Open items still in the backlog** (unchanged from v1.4 plus the snapshot-path mystery and second-screen alias):
- MIG-007 — Links Settings tab consolidation.
- Constellation Map: tooltip canonical-filename + search highlight + suspected memory leak (the canonical-filename label fix in §90 does NOT propagate to the Map — Map uses a different code path; that's still pending in `project_constellation_map_backlog.md`).
- SecondScreenPage.svelte buildSkyData calls still alias-blind.
- Architectural mystery: why is `cache_boot_snapshot_sky` bypassed at boot in builds that contain MIG-001/MIG-004 §8.

### v1.4 changelog (vs v1.3)

v1.4 captures the 2026-04-27 work session: MIG-005 Tutorial #1 testing, the Sky View edge regression fix (§88), the panel-dedupe fix (§89), and a basket of new backlog items the testing surfaced.

**Architecture / fixes shipped:**

- **§88** — `buildSkyData` fallback now alias-aware. The legacy graph-population path that runs when `cache_boot_snapshot_sky` is bypassed had no alias resolution; renamed-target wikilinks were silently dropped, leaving renamed notes as bubble-without-edges in Sky View. Fix at [`store.ts`](src/lib/libraries/store.ts) buildSkyData now accepts an optional `notePathToAliases` map and applies the same 3-tier resolution as `cache.rs::read_sky_links_raw`. User-verified.
- **§89** — Backlinks / Outgoing Links panel dedupe. A source note with both `[[Note]]` (regular) and `[[Note|supports]]` (typed) targeting the same active note used to render twice — once with no badge, once with the type badge. Now grouped by source path (Backlinks) / target name (Outgoing) into ONE row carrying a `linkTypes[]` array of all distinct typed-link badges. Helper `dedupeBySource` in `store.ts`. Same change includes annotation-redundancy suppression: when a typed-link annotation IS the typed-link keyword (e.g. `[[Note|supports]]` stores "supports" in both slots), the redundant italic prose underneath the badge is now suppressed.
- **Badge taxonomy update**: **M = Mutual link** confirmed by project owner 2026-04-27. Moved out of Unresolved into the link-relationship table in `Badge-Taxonomy.md`. **No more pending badge letters.** §13.1 here updated to match.

**New backlog items surfaced this session:**

- **Auto-update Links toggle is misplaced** under "Sky View & Links". Decision 2026-04-27: a new "Links" Settings tab will consolidate every link-related control. Will be **MIG-007** when greenlit. *(Reverses the v1.2 §12 entry that wrongly "corrected" v1.0's right call.)*
- **Constellation Map UX bugs**: tooltips show canonical filename instead of human title; search doesn't highlight matched arc; suspected memory leak / slowness. All filed in `project_constellation_map_backlog.md`.
- **Unlinked Mentions panel** double-counts wikilink occurrences as unlinked mentions (the scanner doesn't strip wikilink syntax before matching) AND shows source label as canonical filename instead of human title.
- **SecondScreenPage.svelte buildSkyData calls** still use the 2-arg form (alias-blind). Same rename-drops-edges symptom there until threaded.
- **Architectural mystery**: even with MIG-005/MIG-004 §8 in the binary, the alias-aware sky snapshot path (`cache_boot_snapshot_sky`) appears to be bypassed at boot — the legacy `buildSkyData` runs instead. The §88 defensive fix neutralizes user-visible impact, but the underlying "why" is unresolved. Filed for follow-up forensics.

**New top-principal rules / Standing Orders saved this session:**

- **Standing Order — staged tests**: split test tutorials into stages. Send Stage 1, wait for findings, then Stage 2. Never dump 6 tests at once. (Memory: `feedback_staged_tests.md`.)
- **Stage 0 — verify the running binary's mtime** before any test tutorial. The user runs an installed `.exe`, not the source on disk — confirm the binary contains the feature being tested. (Memory: `feedback_verify_binary_before_testing.md`. Earned by the 2026-04-27 incident where I burned hours testing against a binary that pre-dated the feature.)
- **Sky View vs Constellation Map vocabulary** — Sky View has bubbles (PIXI nodes); Constellation Map has sunburst arcs (D3). NOT interchangeable. Same correction had to be made twice. (Memory: `feedback_skyview_vs_map_vocabulary.md`.)

**§17 unknowns reduced:**

- **M = Mutual link** — resolved (see above). Removed from §17.
- Sidebar active-item highlight ~10 s lag — still unresolved.
- 2026-04-16 untracked-backup vs tracked log diff — still unresolved.

### v1.3 changelog (vs v1.2)

v1.3 is a focused correction round driven by [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md), the canonical badge reference dated 2026-04-15 (predates v1.0). I missed it on every prior orientation pass. Corrections folded in:

- **§13.1** badge table rewritten:
  - **W** = Wikilink (`[[target]]`), grey `#94a3b8` — was "unresolved" in v1.2.
  - **LT** = Link **Target** (this note links *to* the queried note), green — was "Link Type" in v1.2 (wrong).
  - **G** = deprecated, superseded by **#** — added to the table for posterity.
  - The badge set ships in **two** components per the source-of-truth invariant: [`ConstellationMap.svelte:80-84`](src/lib/components/ConstellationMap.svelte:80) **and** [`ConstellationSight2.svelte:79-83`](src/lib/components/ConstellationSight2.svelte:79). Both must agree letter→color.
  - Semantic clarification: badges indicate **where in the note the search query matched** (or what link relationship), not arbitrary note categories.
- **§14** "Where to read what" — new row pointing to `docs/Badge-Taxonomy.md`.
- **§17** unknowns — **W removed** (now resolved). M still pending owner clarification.

### v1.2 changelog (vs v1.1)

v1.2 closes the §17 unread list. Significant additions:

- **§3.2** corrected: `+layout.svelte` reactive declarations are now **155 $state, 29 $effect, 1 $derived** (was 77/17/19 in LL-002 / 2026-03-27 — file has roughly doubled).
- **§3.3** corrected: 32 Rust modules; ~120 commands.
- **§3.5** (NEW): full Rust module sizes — `search.rs` 4790, `libraries.rs` 3978, `universe.rs` 1472, `canonical.rs` 1401, `cache.rs` 824.
- **§4.2** enriched per-phase with the Rust file path, the actual aggregator details for Phase 12, and corrected Phase 9 lenses status.
- **§5** Arabic Engine: confirmed mmap is wired through ([`fst_bake.rs:323`](src-tauri/src/arabic/fst_bake.rs:323)), via `Arc<Mmap>` shared by both stripped + folded FSTs.
- **§5.5** (NEW): ai/, embeds/, embeddings/, tasks/, lens.rs (Brandes betweenness), inspector360.rs, sky_backfill.rs (BATCH_SIZE=1000, INTER_BATCH_SLEEP_MS=50), boot_bundle.rs.
- **§7.1** editor stack now described per-plugin from full reads. Added the LL-014 RULE A / RULE B in `calloutPlugin.ts`.
- **§7.4** (NEW): `store.ts` write-ahead buffer (memory + localStorage), navigation supersede tokens, `recentWrites` 2 s gate, save coalescing.
- **§7.5** (NEW): `secondScreen.ts` event API (12 main→screen, 4 screen→main, 1 bidirectional).
- **§9.3** (NEW): boot-bundle (10 IPCs → 1 round-trip) for early-boot data.
- **§11** LL list now grounded in verbatim text.
- **§12** drift list refreshed: `autoUpdateLinks` toggle is **correctly under "Sky View & Links"** (v1.0 misclaimed it as misplaced); `IPC-CONTRACT.md` still 4 weeks stale.
- **§13** badge taxonomy resolved: **T/C/P/S confirmed**; **#, ∅, W, M and LT/LF/⇄/LB/LA also defined** in `ConstellationMap.svelte:80-84`. **W and M letter meanings remain unresolved** (no doc found; honest).
- **§13** auto-update-links toggle confirmed at Settings → **Sky View & Links** (not "Files" as v1.0 wrongly suggested).
- **§14** corrected `lib.rs:233-432` line range.
- **§15.3** (NEW): collision tiebreak — name wins over alias; identical-alias multi-target is **first-write-wins, undefined order**.
- **§17** dramatically reduced — every Rust module read; every CM6 plugin read; every major Svelte component surveyed; `store.ts`, `secondScreen.ts`, `universe/store.ts` read; user manual + 24 help topics + BASES_MVP_SPEC + Concept Paper + Editor-Spec + eNotePane-development-record indexed; 14 translated User Manuals confirmed (ar = 1328 lines, others = 1120, parity confirmed); 20 session logs digested chronologically.
- **§17 remaining unknowns**: badge letters W and M (defined in code but undocumented); sidebar active-item highlight ~10 s lag origin (no reactive source isolated).

---

## 1. What Constellation IS

**Constellation is a Personal Knowledge Formulation desktop application.**

The distinction is fundamental — it is **not** PKM (Personal Knowledge Management):

> Knowledge Management asks: "Where did I put that?"
> Knowledge Formulation asks: "What can I BUILD from what I know?"
> *(`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md:13-17`)*

It is built on **standard Markdown files** (`.md` + YAML frontmatter) on the user's local filesystem, with a portable Universe-config layer above. Local-first, no telemetry, no cloud, no account.

- **Author**: Eisa ALSHAMSI
- **License**: MIT
- **Repository**: `github.com/eisaShamsi/Constellation`
- **Stack**: Tauri v2 (Rust backend) + SvelteKit + Svelte 5 + SQLite (rusqlite, bundled) + ONNX Runtime (`ort`) + CodeMirror 6 + PIXI v8 + D3 v7
- **Languages supported at launch**: 15 — `ar / de / en / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh`
- **RTL languages first-class**: 4 — Arabic, Hebrew, Persian, Urdu
- **Platforms**: Windows, macOS, Linux desktop. CI ships Windows builds today.
- **Mobile**: iOS/Android excluded via `cfg(not(any(target_os="ios", target_os="android")))` for `memmap2`. Not shipping mobile apps.

---

## 2. Universe / Library / Note hierarchy

Constellation has a **five-level knowledge hierarchy**:

```
Universe (root, named by user, contains universe.json)
  └── cUniverse (child universe — federation of libraries)
       └── Library (self-contained knowledge base, like Obsidian vault)
            └── Folder (subdirectory inside a Library)
                 └── Note (single .md file with optional YAML frontmatter)
```

- **Universe** = portable directory. Contains `.constellation/` subfolder with `universe.json`, `libraries.json`, `settings.json`, `bookmarks.json`, `workspaces.json`, `property-types.json`, `bases/`, `templates/`. Move it to another machine and the entire workspace follows.
- **Library** = first-class citizen with its own color/appearance/tags/links/index. Registered in `libraries.json`. Multiple libraries coexist in one Universe. Constellation reads them in place — never copies.
- **Folder ≠ Library**. Folders are organizational only.
- **Terminology**: use "Library" everywhere, **never** "vault" (except for Obsidian import compatibility).

### 2.1 Universe migration (legacy → current)

[`universe.rs::migrate_legacy_data`](src-tauri/src/universe.rs:1306) moves a v1 layout to v2:

- **From**: flat `universe.json` / `vaults.json` / `settings.json` at universe root; registry stored at `app_data_dir/vaults.json`; nested `name/name/` notes layout.
- **To**: `.constellation/` subdirectory; `vaults.json` renamed to `libraries.json`; registry moved to `app_data_dir/universes.json` (UniverseRegistry with `entries` and `active_id`); flat notes layout (Universe root IS the library, Obsidian-style).

`migrate_to_constellation` (line 133), `ensure_universe_notes_folder` (line 195), `set_active_universe` (line 545 — also consolidates same-name nesting `C:\Name\Name\` → `C:\Name\`).

### 2.2 Child-universe federation

[`universe.rs:425`](src-tauri/src/universe.rs:425) `resolve_child_universe_roots(parent)` reads `universe.json::children[]`, canonicalizes, filters directories. `resolve_libraries_recursive` (line 353) collects own + all child libraries, prevents circular refs, deduplicates by path. Frontend command: `resolve_universe_libraries`.

---

## 3. Architecture (one-page view)

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (SvelteKit / Svelte 5)                                │
│  src/routes/                                                    │
│    +layout.svelte (6872 lines — orchestrator, see §3.2)         │
│    +page.svelte (1 line — note viewing handled by layout)       │
│    libraries/+page.svelte (704 lines — library management)      │
│    skills/+page.svelte (219 lines — skills/onboarding)          │
│  Second window: static/screen.html (separate Tauri webview)     │
│  Editors: NotePane.svelte (388) / FocusPane.svelte (213)        │
│  Panels: Sky View (PIXI), Constellation Map (D3 sunburst),      │
│    Inspector 360, Tension, Sight, Lens, Bases, Tasks, Calendar, │
│    Backlinks, OutgoingLinks, IndexPanel, OrgChart, SearchHub    │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC (~120 commands, 32 Rust modules)                     │
│  - perf_trace (LL-021): every dispatch stamped at the boundary  │
│    via Box-typed closure wrapping generate_handler!             │
│  - 3 plugins: opener / process / updater                        │
│  - panic hook in run() writes constellation-crash.log           │
│    (NO panic-handler plugin — just std::panic::set_hook)        │
├─────────────────────────────────────────────────────────────────┤
│  Backend (Rust, src-tauri/src/, 32 modules — see §3.5)          │
│  - libraries.rs (3978) — file I/O, link extraction, cascade     │
│  - search.rs (4790) — SQLite, FTS5, Living Link triggers,       │
│    sky_nodes/sky_links triggers (Rule 8)                        │
│  - cache.rs (824) — boot snapshot, alias resolution             │
│  - canonical.rs (1401) — YYYYMMDDTHHMMSSZ_KIND_XXXX             │
│  - universe.rs (1472) — universe/cUniverse + legacy migration   │
│  - arabic/ (15 files) — 5-layer morphological engine, mmap'd    │
│  - lexicon/ (6 modules) — Lexical Bridge polylingual lemma graph│
│  - CE Layer 1: strata.rs / maturity.rs / tension.rs /           │
│    provenance.rs / inspector360.rs / lens.rs / lenses.rs /      │
│    review.rs / trails.rs / canvas.rs                            │
│  - bases.rs — .base file CRUD (read-time)                       │
│  - dataview.rs — DQL queries (read-time)                        │
│  - importers.rs — 7 source formats (one-off, async)             │
│  - watcher.rs — notify-rs file watch (must be async)            │
│  - boot_bundle.rs — 10 IPCs collapsed into 1                    │
│  - sky_backfill.rs — resumable populator, BATCH_SIZE=1000       │
│  - embeddings.rs — ONNX multilingual-e5-small (write-time)      │
│  - embeds.rs / fts5_tokenizer.rs                                │
│  - perf_trace.rs — IPC arrival tracer                           │
│  - ai/mod.rs — OpenAI/Anthropic/Gemini/Ollama                   │
├─────────────────────────────────────────────────────────────────┤
│  Storage                                                         │
│  - .md files on disk (source of truth)                          │
│  - SQLite DB at <universe>/.constellation/search.db              │
│    Tables: schema_versions, note_meta, note_embeddings,         │
│    note_links, note_aliases, sky_nodes, sky_links, notes_fts,   │
│    notes_vocab (fts5vocab), sky_backfill_cursor                 │
│  - boot-perf.latest.json — per-boot scorecard                   │
│  - .meta.json sidecars for non-markdown files (canonical)       │
│  - .constellation/review-pulse.json — Phase 7 schedule state    │
│  - .constellation/arabic-overrides.json — L5 user overrides     │
│  - kind_registry.json — auto-generated KIND codes (file_kinds)  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 Key dependencies (versions)

| Layer | Package | Version | Purpose |
|---|---|---|---|
| Rust | `tauri` | 2.x with `protocol-asset` feature | App runtime |
| Rust | `rusqlite` | bundled | SQLite |
| Rust | `ort` | ONNX Runtime | Semantic embeddings |
| Rust | `tokenizers` | HuggingFace (with `onig`) | Tokenizers |
| Rust | `fst` | BurntSushi | Arabic generative index |
| Rust | `memmap2` | 0.9 (desktop only) | mmap baked Arabic FST — **wired through** [`fst_bake.rs:323`](src-tauri/src/arabic/fst_bake.rs:323) |
| Rust | `notify` | File watcher | |
| JS | `svelte` | ^5.0 | UI framework (runes mode) |
| JS | `@sveltejs/kit` | ^2.9 | Routing |
| JS | `@codemirror/*` | 6.x (full set) | Editor |
| JS | `pixi.js` | ^8.17 | Sky View force graph (LL-019: `pixi.js/unsafe-eval` first) |
| JS | `d3` | ^7.9 | Constellation Map sunburst |
| JS | `@xenova/transformers` | ^2.17 | Frontend ONNX |
| JS | `katex` / `mermaid` / `marked` / `dompurify` | latest | Math / diagrams / markdown / XSS |

Plugins: `tauri-plugin-opener`, `tauri-plugin-process`, `tauri-plugin-updater`. **No panic-handler plugin** — the crash log path uses `std::panic::set_hook` in [`lib.rs:212-222`](src-tauri/src/lib.rs:212).

### 3.2 The `+layout.svelte` reactivity load (corrected counts)

`+layout.svelte` is the orchestrator. **6872 lines as of 2026-04-26.** Reactive declaration counts (verified by Grep this round):

| Kind | Count | LL-002 baseline (2026-03-27) | Change |
|---|---|---|---|
| `$state` | **155** | 77 | +78 |
| `$effect` | **29** | 17 | +12 |
| `$derived` | **1** (`allTagsList`) | 19 | −18 |

Growth drivers: multi-phase graph boot, second-screen sync effects, Tier 1 panel-placement state, child-universe sidebar expansion, lazy-mount flags. The drop in `$derived` count reflects intentional consolidation — derivations now live inside `$state`-bearing handlers or were promoted to module-level helpers.

`+page.svelte` is **a single-line comment** — the entire note-viewing UI is composed inside `+layout.svelte`. The `libraries/` (704 lines) and `skills/` (219 lines) routes are real pages.

**Lazy-mount flags** ([`+layout.svelte:569-572`](src/routes/+layout.svelte:569)): `mapEverOpened`, `orgChartEverOpened`. Both are sticky $state(false), set true via $effect on `showConstellationMap` / `showOrgChart`, **reset in `handleUniverseSwitch` at lines 1935-1936**. Used to gate `{#if mapEverOpened}` ... `{#if showConstellationMap}` two-tier rendering (LL-022 compliance).

**$effect violation candidates flagged** (audit-pending): line 498 (`lastSavedContent` async-race risk per LL-023), lines 781 / 837 / 1235 / 1353 / 1449 / 3480 (always-mounted IPC fan-out — index/sky scans run regardless of visibility).

### 3.3 Tauri command surface

[`lib.rs:233-432`](src-tauri/src/lib.rs:233) registers ~120 commands across 32 modules. The `invoke_handler` is wrapped in a Box-typed closure that records each dispatch via `perf_trace::record(invoke.message.command())` — the LL-021 IPC arrival tracer.

Two Tauri v2 type-system subtleties (from LL-021):

1. `generate_handler!` must be bound via `Box<dyn Fn(Invoke<Wry>) -> bool + Send + Sync + 'static>` to pin the macro's `R: Runtime` generic at the binding site.
2. `invoke.message.command()` returns `&str`; call `perf_trace::record` *before* forwarding to `inner(invoke)`.

**[`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) is significantly stale** (last updated 2026-03-31; lists ~80 commands of ~120). Until refreshed, [`lib.rs:233-432`](src-tauri/src/lib.rs:233) is the authoritative command registry.

### 3.4 Build / Release / CSP / Windows / Capabilities

**Versions** (in sync at 0.3.4):
- [`package.json`](package.json) — `"version": "0.3.4"`
- [`src-tauri/tauri.conf.json:4`](src-tauri/tauri.conf.json:4) — `"version": "0.3.4"`
- `src-tauri/Cargo.toml` — bumped per release workflow

**`tauri.conf.json` highlights**:
- `productName: "Constellation"`, `identifier: "world.uconstellation.app"`
- Two windows: `main` (1200×800) and `second-screen` (1200×800, `url: "screen.html"`, `visible: false` at startup).
- CSP: `default-src 'self'`; `script-src 'self' 'unsafe-inline'`; **no `unsafe-eval`** → LL-019 still applies (PIXI must use `pixi.js/unsafe-eval` side-effect import).
- Asset protocol enabled, `allow: ["**/*"]`, `requireLiteralLeadingDot: false`.
- Updater enabled, endpoint = public Gist (`gist.githubusercontent.com/.../latest.json`); minisign pubkey embedded.

**Capabilities** ([`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json)) — applies to both `main` and `second-screen` windows. Permissions: `core:default`, window controls, `core:webview:allow-create-webview-window`, `core:webview:allow-set-webview-zoom`, `opener:default`, `updater:default`, `process:allow-restart`.

**Second-window file**: [`static/screen.html`](static/screen.html) (built copy at `build/screen.html`).

**CI / release** ([`.github/workflows/release.yml`](.github/workflows/release.yml)) — `windows-latest` runner. Tag push `v*` or manual `workflow_dispatch` (bump `patch|minor|major` or `custom_version`). Bumps `package.json` + `tauri.conf.json` + `Cargo.toml` in lock-step, commits, tags, runs `tauri-action`. Post-release, downloads `latest.json` from release assets and `gh gist edit` updates the public Gist that the in-app updater polls.

**No frontend test harness** (no vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`). Rust unit tests only.

### 3.5 Rust module sizes (full census)

| File | LOC | Role |
|---|---|---|
| `search.rs` | 4790 | SQLite schema + FTS5 + Living Link triggers + search commands |
| `libraries.rs` | 3978 | File I/O + cascade walker + link extraction + 11 cascade tests |
| `universe.rs` | 1472 | Universe registry + child federation + legacy migration |
| `canonical.rs` | 1401 | Canonical filename generation + cid_cn migration + repair |
| `cache.rs` | 824 | Boot snapshots (core/graph/sky) + perf instrumentation |
| `embeds.rs` | 708 | Living embed resolver (`![[target]]`) — 7 resolution tiers |
| `inspector360.rs` | 517 | Aggregates 9 phase data per note (read-only); §112 added per-note `stratum` + `precompute_all_strata` |
| `lens.rs` | 419 | Brandes' betweenness centrality + tag-shared edges |
| `sky_backfill.rs` | 470 | Resumable populator (BATCH=1000, sleep=50ms) |
| `tasks.rs` | 495 | Task scanning (Tasks plugin emoji syntax) |
| `boot_bundle.rs` | 138 | 10 IPCs collapsed into 1 round-trip |
| `tension.rs` | — | CE Phase 4 |
| `provenance.rs` | — | CE Phase 5 (isnad-inspired) |
| `review.rs` | — | CE Phase 7 |
| `trails.rs` | — | CE Phase 8 |
| `canvas.rs` | — | CE Phase 10/11 (Cynefin) |
| `lenses.rs` | — | CE Phase 9 (Multi-Lens) — Rule 8 hybrid violation |
| `bases.rs` | — | .base file CRUD — Rule 8 read-time violation |
| `dataview.rs` | — | DQL queries — Rule 8 read-time violation |
| `importers.rs` | — | 7 source formats (Obsidian / Bear / Notion / Evernote / Markdown / HTML / Constellation backup) |
| `embeddings.rs` | — | ONNX e5-small (384-dim, 100 langs) |
| `watcher.rs` | — | Must be `async` (else Boot Criterion 2 dies) |
| `file_kinds.rs` | — | 3-layer kind classification |
| `fts5_tokenizer.rs` | 479 | Custom 'constellation' tokenizer (stemming + bigrams) |
| `perf_trace.rs` | 71 | TRACE_LOG mutex; record/get/clear |
| `strata.rs` | — | CE Phase 2 (8-level hierarchy) |
| `maturity.rs` | — | CE Phase 3 (5 states) |
| `map.rs` | — | Constellation Map (D3 sunburst data) — Rule 8 read-time |
| `arabic/mod.rs` + 14 files | — | 5-layer morphological engine |
| `lexicon/` | 6 files | Polylingual lemma graph |
| `ai/mod.rs` | 406 | 4-provider AI abstraction |

---

## 4. The Cognitive Engine (CE)

`docs/CE-spec.md` + `docs/cognitive-engine-roadmap.md` are the canonical specs. Two-layer architecture.

### 4.1 Seven epistemological foundations (`CE-spec.md:22-29`)

1. Knowledge is not information — value is in connections, not storage.
2. Knowledge has a vertical dimension — 8-level hierarchy (Datum → Worldview).
3. Knowledge has a certainty dimension — `ilm al-yaqin → haqq al-yaqin`.
4. Knowledge is organized by immutable principles — non-contradiction, causality, hierarchy.
5. Knowledge has diverse sources — sensory, rational, transmitted, experimental, intuitive.
6. Knowledge exists on a spectrum — received vs discovered.
7. The essence of knowledge is understanding-generative apprehension.

### 4.2 Layer 1 — Structural Cognition (zero AI). All shipped.

| # | Name | File | Rule 8 |
|---|---|---|---|
| 1 | Typed Links | `libraries.rs` + `search.rs` (note_links + triggers) | ✅ Write-time |
| 2 | Knowledge Strata (8-level) | [`strata.rs`](src-tauri/src/strata.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1137`](src-tauri/src/search.rs:1137)) |
| 3 | Maturity Lifecycle | [`maturity.rs`](src-tauri/src/maturity.rs) | ✅ Write-time (sky_nodes triggers, [`search.rs:1215`](src-tauri/src/search.rs:1215)) |
| 4 | Tension Detector | `tension.rs` | ⚠️ Partial — contradictions cached, structural gaps on read |
| 5 | Provenance Chain (isnad-inspired) | `provenance.rs` | ⚠️ Partial — frontmatter sources cached, traversals on read |
| 6 | Externalization | within `strata.rs` (word_count signal) | ✅ Write-time |
| 7 | Review Pulse | `review.rs` | Hybrid — `.constellation/review-pulse.json` |
| 8 | Trails | `trails.rs` | ✅ Write-time |
| 9 | **Multi-Lens Views** | `lenses.rs` | ❌ **Hybrid violation** — definitions write-time (`lenses.json`), results recomputed on read (`apply_lens` walks the tree) |
| 10/11 | Expression Forge / Sense-Making Canvas | `canvas.rs` | ✅ Write-time (JSON persisted) |
| 12 | 360° Inspector ✅ enabled v1.8 §93, hardened v1.9 §96–§104, **redesigned v1.15 §112 (Stratification Matrix)** | `inspector360.rs` (517 lines) | ⚠️ **Read-time aggregation, but actual perf is fine** — the per-fetch cost was theorised as 1–3 s but Boss's lived experience is "almost instantly". MIG-010 (cache `note_360_view` write-time) priority dropped to LOW. Frontend mitigations still in place: debounce 200 ms, sequence guard, last-fetched-key dedup, lazy mount, dedupe-by-path in the matrix. |

**Inspector 360° aggregator** ([`inspector360.rs:1`](src-tauri/src/inspector360.rs:1)): aggregates `Note360View` from typed/untyped links (7 types) + active-note stratum + maturity + contradictions + orphan/SPOF flags + provenance + stage + review + trail membership + lens groups + missing-link-types gap analysis. **Post-§112**: every `LinkedNote` (outbound, inbound, second-order) also carries `stratum: u8`, populated by `precompute_all_strata()` — a single pass that builds an inbound-count + sources-of map for the library, then runs the existing `compute_stratum_for_note` rule set against each note. O(N + total_links). Same big-O as before; constants higher but sub-second on the 7,600-note Universe per Boss's lived experience.

**Frontend Inspector 360 surface** (post-§112 — **Stratification Matrix**):

- Two display modes via the `compact` prop. Compact = right-sidebar tab (scorecard glance widget). Full-window = ribbon-dock button (deliberate-study matrix, replaces editor area).
- **Full-window = the matrix.** HTML/CSS Grid (no SVG polar coordinates). 8 rows (stratum L8 → L1, top-down) × 8 columns (`supports`, `contradicts`, `causes`, `derives-from`, `generalizes`, `exemplifies`, `part-of`, `untyped`) + a 200 px row-label column on the left + a 64 px row-totals column on the right. Each `(stratum, type)` cell holds the connected notes whose stratum matches the row, drawn as 11 px coloured dots (max 16 per cell, then `+N` overflow chip). Active note's row is highlighted (purple background gradient + bold `L{n}` chip showing the note's truncated name). Empty cells render diagonal stripes — gaps as first-class signal.
- **Compact = a scorecard.** Stratum pill (`L4 Concept`), maturity pill, ↑outbound/↓inbound/word-count line, per-type bar chart (label + filled track + count, with explicit `—` for blind spots and 50 % opacity to mark zero rows), and a flags row (orphan, fragile, gap count, due for review). No matrix — 280 px is too narrow.
- **Multi-hop back stack** shared between compact and full-window. State: `inspector360BackStack: $state<Array<{path, name}>>` in `+layout.svelte`. Forward node-click pushes current; back click pops one entry; bar shows `← {previous}` until empty. Universe switch resets to `[]`.
- **Hover-only labels** (preserved from §107). Hovering a dot reveals the connected note's name in a fixed top-right tooltip on the matrix canvas — does not follow the mouse, doesn't pop chrome on dense rows. The dot itself enlarges (`scale(1.6)`) and gains a colored glow (`box-shadow: 0 0 10px var(--dot-color)`) on hover.
- **Per-cell dedup** on path so the same note returned from outbound + inbound + second-order sources renders once per `(type, stratum)` cell.
- **Dimension strip** below the header surfaces the non-spatial dimensions: Stratum (with name), Maturity (color dot), Origin + trust depth (color dot), Stage (icon + name), Review (date or "Due"), Trails / Lenses (count) — only shown if non-empty.
- **Bottom HUD** keeps the existing `total_outbound` / `total_inbound` / `word_count` summary plus warning chips for orphan / fragile / blind-spots / tensions.
- **Dropped permanently**: `vizMode` dropdown (Atmospheric / Neural / Cosmic), `SECTOR_MAP`, `polarToXY`, `ringsLayout`, `layoutMode`, `allNodes`, `SECTOR_THRESHOLD`. Polar geometry is gone from the file; the design space the matrix occupies is grid + axis semantics.

### 4.3 Layer 2 — AI Discovery (5 phases, 🔲 all not started)

12. Hidden Pattern Discovery (ghost links via semantic engine).
13. Blind Spot Detection.
14. Cross-Domain Insight Generation.
15. Socratic Challenger.
16. Worldview Synthesis.

Local-LLM-first; cloud opt-in only. Existing infrastructure: `ai_send_message` Tauri command across 4 providers (OpenAI / Anthropic / Google Gemini / Ollama — [`ai/mod.rs:1-406`](src-tauri/src/ai/mod.rs:1)); embeddings via ONNX multilingual-e5-small (384-dim, 100 languages — `embeddings.rs`).

### 4.4 The Living Link Architecture (P0–P5 all shipped + user-validated)

`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` is the philosophy doc.

**8 link properties**: Type · Direction · Annotation · Weight · Confidence · Created · Last Traversed · Traversal Count.

**7 typed link types** (default `relates`/`associative`):
`supports` (blue) · `contradicts` (red) · `causes` (orange) · `exemplifies` (green) · `generalizes` (purple) · `derives-from` (gold) · `part-of` (gray).

**Syntax**: `[[Target|type]]` (pipe-after-target). The 3-part form `[[Target|alias|type]]` is parsed via `lastIndexOf('|')` ([`livePreview.ts:926-965`](src/lib/editor/livePreview.ts:926)).

**4 confidence levels**: `hypothesis` → `evidence` → `established` → `contested`. Auto-promote at traversal_count ≥3 → evidence, ≥10 → established. Manual override via right-click.

**Decay formula** (display-only — `weight` raw column never modified):
```
effectiveWeight = rawWeight × exp(−ln(2) × daysSinceTraversal / halfLifeDays)
```
Default half-life: 60 days.

**Storage**: dual-layer design (LINK files on disk + SQLite). **The on-disk LINK files layer was deliberately deferred** — implementation lives only in `note_links` SQLite table.

**Archive = soft-delete.** Reversible via Link Dashboard's Archived tab.

**Lifecycle commands** ([`search.rs:2330-2938`](src-tauri/src/search.rs:2330)): `_link_stats`, `_link_traverse` (updates weight via `1.0 + ln(1 + traversal_count)`), `_link_dormant`, `_link_decay`, `_link_set_confidence`, `_link_backfill_confidence`, `_link_archive` / `_unarchive` / `_archived`.

---

## 5. The Arabic Engine + Lexical Bridge

A native 5-layer morphological engine. Built from scratch, license-clean. **Not a port.**

### 5.1 Engine architecture (verbatim from [`arabic/mod.rs:16-37`](src-tauri/src/arabic/mod.rs:16))

```
[L1 normalizer]        — tashkeel / tatweel removal, hamza variants,
                          language detection; preserves surface form
   ↓
[L2 protected list]    — ~20K proper nouns + loanwords (hash lookup)
   ↓
[L3 generative FST]    — rolling-hash + FST over all (root × pattern)
                          combinations
   ↓
[L4 disambiguator]     — ranks multiple analyses by corpus frequency
   ↓
[L5 user overrides]    — per-Universe learning layer
```

**5 logical layers, 15 physical Rust files** in `src-tauri/src/arabic/`:

- `normalizer.rs` (484 lines) — L1: tashkeel/tatweel strip, aggressive folding (alif/ya/ta-marbuta), script detection (Arabic/PersianFamily/Hebrew/Latin/Other). Core test: `وائل` survives stripping (Light10 bug fix).
- `protected.rs` (551 lines) — L2: TSV-backed `HashMap<stripped, ProtectedEntry>` (~1196 entries). Categories: ProperNoun / Place / Loanword / Function. First-write-wins on dupes. M1e flagship: `وائل`, `محمد`, `إنترنت` return verbatim with confidence=1.0.
- `fst_index.rs` (598 lines) — L3: `GenerativeFst` wraps **two `fst::Map<FstBytes>`** (stripped + folded). Packing: FST value = `(offset u32 << 32 | count u32)`. ~300K distinct keys, ~1.1M forms at 7K-root scale, single-digit MB via prefix sharing.
- `fst_bake.rs` (991 lines) — M3-baker on-disk cache. **mmap wired through line 323**: `Mmap::map(&file)?` → `Arc<Mmap>` shared by both stripped + folded FSTs (single syscall + VMA). Cache filename: `arabic-fst-v{djb2(SEED_TSV) XOR CACHE_FORMAT_VERSION:016x}.bin`. Mobile fallback: heap `Vec<u8>`.
- `generator.rs` — Template substitution `(Root, Pattern) → surface`. Placeholders ف/ع/ل. Phonology passes: gemination fusion, hamza carrier picking, weak-radical rewrites (M2.c).
- `patterns.rs` — ~158 morphological patterns (verbal 50, verbal nouns 20, participles 22, broken plurals 27, etc.). All patterns carry full tashkeel.
- `roots.rs` — Root inventory (595 seed → 7K corpus). Classification: Hamzated / Geminated / Assimilated / Hollow / Defective / Sound (triliterals); Sound / Weak (quadriliterals).
- `affixes.rs` — Affix-peeling cascade (e.g., ال + كاتب).
- `disambiguate.rs` — L4 deterministic ranking (confidence → origin priority → POS → fewer affixes → alphabetic).
- `overrides.rs` — L5 per-Universe JSON store at `<universe>/.constellation/arabic-overrides.json`. Tauri commands: `read_arabic_overrides`, `add_arabic_override`, `remove_arabic_override`, `reindex_arabic_overrides`.
- `types.rs` — `Analysis`, `Root`, `Pattern`, `PartOfSpeech`, etc.
- `regression.rs`, `bench.rs`, `rss.rs` — test/bench harness (cfg-gated).

**Entry points** ([`arabic/mod.rs:129-564`](src-tauri/src/arabic/mod.rs:129)): `analyze`, `analyze_with_overrides`, `analyze_best`, `analyze_with_overrides_best`.

### 5.2 M-numbered milestones (NOT module boundaries)

The "M3-M14" series in session logs are **project milestones**. Engine is 5 layers (above). All M-milestones shipped:

- M3 FST-backed generative index + M3-baker cache.
- M5 502-case regression corpus, 100% pass.
- M6 FTS5 routes Arabic stemming through `analyze_best`. Closes flagship `وائل → "ائل"` mangle.
- M7 deterministic disambiguator.
- M8 + M8b + M8c — L5 user overrides + ACTIVE_STORE registry + Settings UI.
- M9 bench — ~130k words/sec, ~7.6 MiB cache.
- M10 Lexical Bridge architecture (15-concept seed).
- M11-infra Lexical Bridge baker.
- M11-data v1 (49-concept seed).
- **M11-data v2 Producer ✅ complete** — **20,000 concepts** across **499 thematic shards** in `lab/m11-data/concepts/` (verified by `wc -l lexicon_v1.tsv` = 20,015 lines incl. header).
- M12 query expansion plumbing (`escape_fts_term`, `build_match_expr`, `expand_to_match_expr`).
- M12-detect language detection (15-language classifier).
- M12-bench (mean 5.2 µs, p99 15.8 µs — 60–600× under 1 ms budget).
- M13 multilingual result badge (`match_via`).
- M14 lexical_search end-to-end bench gate.

### 5.3 Lexical Bridge (`src-tauri/src/lexicon/`, 6 modules)

**Polylingual lemma graph**, not a morphological tool: every lemma in any of the 15 languages can be looked up and yields its equivalents in any other.

- **graph.rs** — Node identity: `(lang, lemma, sense_id)`. Edge types: Equivalent / Synonym / Hypernym / Hyponym / UserLink. Storage: FST `{lang_code}:{normalized_lemma} → (first_node_idx u32 << 32 | sense_count u32)`. Core tier: ~20K concepts × 10 langs ≈ 200K nodes, ~800K edges.
- **expansion.rs** — Query expansion. `SynonymLevel`: None / Synonym / SynonymAndHypernyms (±1 hop). Pipeline: lemmatize → fetch equivalents → add synonyms/hypernyms → build FTS5 MATCH across selected languages. Cap 8 per language by default.
- **bake.rs** — TSV ingestion + binary cache (content-addressed, version-hash gated).
- **detect.rs** — Language detection (15-language Unicode classifier).
- **fts.rs** — FTS5 integration (escape, match expression assembly).
- **parse.rs** — TSV format parsing.

Source: `src-tauri/src/lexicon/data/lexicon_v1.tsv`. Built deterministically by [`lab/m11-data/build.py`](lab/m11-data/build.py) (Python 3) from 499 JSON shards.

**Coverage policy**: `en` + `ar` required per concept; target ≥8 of 15 languages. **No third-party sources** — all content original (WordNet / Wiktionary explicitly rejected per project policy in `lab/m11-data/README.md`).

### 5.4 Custom FTS5 tokenizer ('constellation')

[`src-tauri/src/fts5_tokenizer.rs`](src-tauri/src/fts5_tokenizer.rs) (479 lines). Wraps the Rust stemming pipeline: Arabic Light10 + Hebrew prefix stripping + Persian / Cyrillic / Devanagari / German / Spanish / Portuguese / French / Turkish / English stemmers + bigrams. Symmetric across `FTS5_TOKENIZE_DOCUMENT` (write) and `FTS5_TOKENIZE_QUERY` (read).

**Token emission**:
1. Primary token: stemmed form.
2. Bigram (colocated): `prev_stem \x1f cur_stem` (separator `0x1f` unmatchable in user text).
3. Stopwords/length-filtered: emit nothing, break bigram chain.
4. Bigrams form **only between tokens in the same script** (prevents Arabic↔English bigram noise).

All Arabic-side morphology delegates to `crate::libraries::process_word_for_fts` → `analyze_best()`.

### 5.5 Other Rust modules (read this round)

- **inspector360.rs** (517, post-§112) — see §4.2 row 12.
- **lens.rs** (419) — Brandes' betweenness O(VE), weighted by link_type (supports=1.0, causes=0.9, contradicts=0.8). **At >500 nodes**: approximate sampling (200 sources). Tag-shared edges command: weight 0.6 × shared_tag_count, top 500.
- **boot_bundle.rs** (138) — `BootBundle`: libraries + settings + bookmarks + workspaces + property_types + workspace_bases + child_universes + child_universe_lib_paths + per-step `timings_ms`. Replaces ~10 serialized IPCs.
- **sky_backfill.rs** (470) — MIG-001 §5 resumable populator. `sky_backfill_cursor` table stores `last_path`. `BATCH_SIZE=1000`, `INTER_BATCH_SLEEP_MS=50`. Per-batch phases: A (insert sky_nodes/links under lock) → B (read note files, compute word_count + created_at + aliases, no lock) → C (UPDATE note_meta) → D (UPDATE sky_nodes stratum/maturity). Idempotent via `INSERT OR IGNORE`. Final stamp: `schema_versions.sky = SKY_SCHEMA_VERSION`.
- **tasks.rs** (495) — `[- | * | +] [ ] | [x] | [X] text` pattern. Extracts: due_date (📅 YYYY-MM-DD or `[due:: …]`), priority (⏫🔼🔽), tags (#tag), created_date (➕), done_date (✅). Commands: `scan_library_tasks`, `scan_note_tasks`, `toggle_task`, `scan_library_note_dates`.
- **embeds.rs** (708) — Living embed resolver. 7-tier search order: relative-to-note → absolute-in-vault → explicit-attachment-folder (`.obsidian/app.json`) → fallback (attachments/ images/ assets/) → vault-wide index → vault root. `EmbedKind`: image / audio / video / pdf / canvas / excalidraw / note / generic / missing. URLs: data: if ≤4 MB, else `asset://localhost/{encoded_path}`. Digit normalization: Arabic-Indic (٠–٩) + Extended (۰–۹) → ASCII.
- **embeddings.rs** — ONNX runtime + multilingual-e5-small (384-dim, 100 langs), 100% offline. `constellation_init_embeddings`, `_embed_text`, `_embed_notes`, `_embedding_status`. Vectors persisted to SQLite.
- **importers.rs** — 7 formats async. `import_pick_source`, `_preview`, `_execute`, `_with_canonical`.
- **watcher.rs** — `notify` crate. **MUST be `#[tauri::command(async)]`** (recursive watch is blocking I/O; sync command runs on WebView2 UI thread → Boot Criterion 2 fails). Inline note at lines 19-38 explains the constraint.
- **dataview.rs** — DQL TABLE / LIST / TASK / CALENDAR + FROM + WHERE + SORT + LIMIT. Reuses bases.rs scan primitives. Read-time recompute on every `execute_dataview_query`.
- **bases.rs** — `.base` YAML CRUD. Live scans on `query_base`. 5 commands.
- **perf_trace.rs** (71) — `static TRACE_LOG: Mutex<Vec<(String, u64)>>`. `record(cmd)` / `get_perf_trace_log` / `clear_perf_trace_log`.
- **file_kinds.rs** (454) — 3-layer kind classifier. Layer 1: extension map. Layer 2 (markdown): explicit frontmatter `kind:` / `type:`, then heuristics (LINK = from+to fields; TMPL = `<%…%>` / `{{…}}` ≥3 occurrences or `template: true`; MARK = `url:` + body <500 chars; CLIP = `source:` + blockquotes; BASE = `schema:` / `dataview` blocks; default = NOTE). Layer 3: unknown extension → `auto_generate(ext)` → persist in `kind_registry.json`. 4 unit tests.

---

## 6. Filename + Identity Architecture (post-MIG-003, 2026-04-28)

> **Architecture inverted by MIG-003 (commits §85–§89). The legacy "canonical filename = primary key" design is preserved as historical record in `docs/CANONICAL-FILENAME-ARCHITECTURE.md` § 0 banner; the rest of that doc describes the pre-MIG-003 design.**

### 6.1 Two ids, two purposes

| | What it is | Where it lives | Mutability |
|---|---|---|---|
| **`cid_cn`** | Immutable internal id, namespace-safe ("Constellation Node id") | Frontmatter `cid_cn:` field + `note_meta.cid_cn` column + every dependent-table `_cid_cn` column | **Never changes** for the life of the note |
| **Filename** | Human-readable representation of the title | The on-disk `.md` filename + `note_meta.path` column | Changes when the user renames the note |

`cid_cn` format is still the canonical pattern (`YYYYMMDDTHHMMSSZ_KIND_XXXX`), but it is no longer used as a filename — only as an internal correlation key.

### 6.2 Frontmatter contract

```yaml
---
title: Agriculture System
cid_cn: 20260410T153045Z_NOTE_7F3A
kind: note
created: 2026-04-10T15:30:45Z
aliases:
  - Old Title (preserved on rename)
---
```

`title` is user-mutable and equals the filename stem in the steady state. `aliases:` accumulates old titles automatically on rename (so wikilinks targeting the old name still resolve). `cid_cn:` is the load-bearing internal id and is never edited by the user.

### 6.3 12 file kinds — unchanged

`NOTE` · `BASE` · `TMPL` · `LINK` · `MARK` · `CLIP` · `IMG` · `AUD` · `VID` · `ATT` · `CANVAS` · `DRAW` ([`file_kinds.rs:25-45`](src-tauri/src/file_kinds.rs:25)). Auto-generated for unknown extensions (e.g. `.blend` → `BLEND`). The kind is recorded in `cid_cn` itself (the `_KIND_` segment) and in frontmatter; classification logic is unchanged.

### 6.4 `cid_cn` generator

[`canonical.rs:49-93`](src-tauri/src/canonical.rs:49) — timestamp source priority: frontmatter `created:` → filesystem creation → modification → `Utc::now()`; XXXX is 4-char uppercase hex; collision avoidance tries 10 hex suffixes, fallback +1 second. Output is the cid_cn string written to frontmatter at note creation.

### 6.5 Rename flow (post-MIG-003 §89)

`rename_item` ([`libraries.rs:rename_item`](src-tauri/src/libraries.rs)) — unified single path for `.md` files:
1. Read current frontmatter title (for alias preservation).
2. Update frontmatter title + append old title to `aliases:`.
3. `fs::rename` old_path → new_path.
4. Cascade DB: `UPDATE note_meta.path` (fires `note_meta_sky_au` → propagates to sky_nodes/sky_links) + explicit UPDATE on `note_links.source_path/.target_path`, `note_aliases.path`, `note_embeddings.path`.
5. Stamp 'rename' alias row keyed to the new path (durable safety net independent of frontmatter edits).
6. Reindex the note at new path.
7. Frontend cascades `[[OldTitle]]` → `[[NewTitle]]` body rewrite via existing `update_links_on_rename`.

The legacy "canonical-detection special case" that updated frontmatter without renaming the file is **removed**. Folder rename keeps the legacy fs::rename-only flow (folder DB cascade is its own concern, deferred).

### 6.6 New-note creation flow (post-MIG-003 §89)

`create_note` ([`libraries.rs:create_note`](src-tauri/src/libraries.rs)) — single unified path:
1. Sanitize the user-supplied title via `note_display_filename()` (strips reserved chars, falls back to "Untitled" if empty).
2. Resolve filename collision via `resolve_filename_collision()` — auto-suffixes "Untitled" → "Untitled 1.md" → "Untitled 2.md".
3. Generate fresh cid_cn via `canonical::generate_canonical()`.
4. Write frontmatter with `title`, `cid_cn`, `kind`, `created`.

The previous `native` / `compatible` mode branching is removed. Every library creates human-named files; cid_cn lives only in frontmatter.

### 6.7 Wikilink resolution — unchanged shape, alias-aware

Wikilinks target **titles**, never cid_cn. Resolution order: `title exact → aliases → original_filename → broken (red)`. The alias table (`note_aliases`) is populated from frontmatter `aliases:` lists by the indexer plus explicit 'rename' rows stamped by `rename_item`.

### 6.8 The MIG-003 commit trail

| § | What landed |
|---|---|
| §85 (Step 1) | `cid_cn` column on `note_meta` + UNIQUE index `idx_note_meta_cid_cn` + backfill from frontmatter (7,610 rows; 38 + 4 collisions auto-resolved). Schema-versions module `note_meta` stamped to 1. |
| §86 (Step 2) | `cid_cn` columns on `note_links` (source + target) / `sky_nodes` / `note_aliases` / `note_embeddings` + per-table backfill via JOIN on existing path columns. Schema-versions module `dependent_tables_mig003` stamped to 1. |
| §87 (Step 3) | All 7 INSERT writers stamp cid_cn at write time. `note_meta_sky_ai` trigger updated to copy cid_cn. Boot-time soft re-backfill (cheap, 0 rows in steady state). The `target_cid_cn` bulk re-backfill was caught + omitted (would have hung the app at boot — Working Agreement #4 violation). |
| §88 (Step 4) | New module `mig003_step4.rs`. Walked 17 libraries, found 19 canonical-named .md files (only the user's "inbox" Universe Notes folder used canonical mode; the 16 declared libraries already had human filenames). Per-library transaction; audit log to `.constellation/mig003-step4-renames.tsv`. Schema-versions module `mig003_step4` stamped to 1. |
| §89 (Step 5) | Unified `create_note` + `rename_item` flows. Canonical-detection special case removed (dead code post-Step-4). |

### 6.9 What was deliberately skipped

- **Step 6** (promote `cid_cn` to formal PRIMARY KEY of `note_meta`, drop redundant path columns from dependent tables) — the dual-keyed schema is not a defect; path columns are still load-bearing for fs operations; the rebuild risk was judged not worth the cleanliness gain.
- **§89 alias-append** (preserve old canonical stem in frontmatter aliases of the 19 renamed files) — those files are all dev/test notes from this week's work, no external references existed; saved as wanted-feature memory if future external integration ever needs it.
- **User Manual + 14 i18n translations update** — the user-visible behavior change is small (filenames are now intuitive); separate doc-only commit when convenient, not a blocker.

### 6.10 Legacy commands still in the tree

- `canonicalize_preview` / `canonicalize_execute` / `auto_canonicalize_all` / `inject_cid_library` / `de_canonicalize_library` / `repair_external_libraries_on_startup` — these were the original architecture's tooling. Post-MIG-003 they are mostly dead code. `inject_cid_library` is harmless (just stamps cid_cn into frontmatter); `de_canonicalize_library` is a no-op in the new world (filenames are already human). Deletion candidates for a future cleanup migration; not urgent.

---

## 7. Editor (NotePane / FocusPane)

**Two editors**:

- **[`FocusPane.svelte`](src/lib/components/FocusPane.svelte)** (213 lines) — quick capture, plain text. Imports **only** `bidiPlugin` + base CM6. No markdown parser, no syntax highlighting, no decorations. Comment at line 201 codifies: "Tab switches destroy/recreate FocusPane with new value prop" — no $effect for value sync.
- **[`NotePane.svelte`](src/lib/components/NotePane.svelte)** (388 lines) — full WYSIWYG-like CodeMirror 6. Live preview decorations, callouts, code blocks, images, wikilinks, tables.

### 7.1 The shared editor stack — full per-plugin

`src/lib/editor/` — 11 plugins per the **Editor Parity Rule**.

- **activeEditor.ts** (24) — Singleton `lastView` registry; queried by emoji/icon picker.
- **bidiPlugin.ts** (209) — Per-line script detection (Arabic, Hebrew, Devanagari, CJK split into Hiragana/Katakana → Japanese, Hangul → Korean, else Chinese, Cyrillic, Latin). Theme rule `unicodeBidi:isolate` on `[dir]` lines. Empty-line RTL inheritance from preceding non-empty line. Viewport-only scan; debounced 300 ms.
- **calloutPlugin.ts** (420) — **LL-014 freeze-proof architecture** (lines 5-23 doc):
  - **RULE A**: `Decoration.replace` only when cursor on **different line**. Provably safe — cursor on line N cannot be inside replace covering line M (M ≠ N).
  - **RULE B**: Collapsed body lines use zero-length `Decoration.line({class})` at `line.from === line.from`. CSS `display:none` on `.cm-callout-body-collapsed` does the hiding; Decoration.replace never spans the collapsed region. Cursor never gets "inside" a replace → no CM6 nudge loop.
  - Fold state: `StateField<Set<number>>`. Line numbers remapped via `tr.changes.mapPos()` on docChanged so fold persists across edits.
- **completions.ts** (156) — Wikilink (20-item cap), tag (Unicode `\p{L}` regex, RTL-aware), typed-link (matches `[[note|type]]` and `[[note|alias|type]]` via `lastIndexOf('|')`), slash (14 commands incl. `/table 3x4`).
- **iconSets.ts** (173) — 4 libraries: Lucide (~1500), Phosphor (~1500), Heroicons (~300), Feather (~290). Lazy-load via single shared promise; cached afterwards. `wrapForInsertion` namespaces icon ids.
- **lineDecoPlugin.ts** (131) — Blockquote + fenced-code line-level borders/background. Syntax tree resolved once at viewport start (replaces O(N) forward scan). Callout detection: upward scan max 50 lines.
- **livePreview.ts** (1271) — Core inline-render plugin.
  - **Pre-cached Decoration objects** at lines 138-181: `headingDecos[0..5]`, `boldDeco`, `italicDeco`, `strikeDeco`, `codeDeco`, `linkDeco`, `replaceDeco`, 8 typed-link decos, 2 checkbox states (CR Rule 1).
  - **ViewPlugin update guard** (LL-002, lines 1046-1098): `contextChanged` branch detects path/attachment-folder/traversal-map state effects; `selectionSet` guard rebuilds **only when cursor crosses line boundary** (CR Rule 1); `docChanged` fast path maps decorations + debounces full rebuild 300 ms.
  - Image/embed resolution: 7-tier search; cached (`_imageCache`, `_embedCache`); circular-transclusion guard (`_transcludeStack`).
  - Widgets: ImageWidget, UniversalEmbedWidget (image/audio/video/pdf/canvas/excalidraw/note-transclusion/generic/missing), IconShortcodeWidget, CheckboxWidget, InlineHtmlWidget, AlignmentWidget, CodeBlockLabelWidget, DataviewLabelWidget. All implement `eq()` for memoization.
  - Living Link traversal chip (P4.2, lines 967-988): keyed on `sourcePathLower|targetNameLower`; emits `×N` widget on high-count links.
- **markdownHighlight.ts** (49) — Lezer extension for `==highlight==`. Adds `Highlight` and `HighlightMark` syntax-tree nodes.
- **shortcodeAutocomplete.ts** (167) — Loads 23 emojibase locale datasets in parallel. Combined emoji + icon ranking; per-set boosts (lucide 0, feather −1, heroicons −2, phosphor −3). Lazy-load on first `:` keystroke.
- **tableFormulas.ts** (163) — `=SUM/AVG/COUNT/MIN/MAX(A1:A5)`. A1 syntax with column-letter → 0-based index. Numeric-aware, fallback to `localeCompare` (Arabic-aware).
- **tableUtils.ts** (363) — `parseTable`, `formatTable`, `generateTable`, `detectTabularText` (TSV-first then CSV, ≥50% row consistency required), add/delete/move row/col, `setAlignment`, `sortByColumn` (numeric-aware).

### 7.2 Key NotePane spec rules (top-principal)

- **§2.1 — The Editor Owns Its Content.** After mount, CM6 owns the document. One-way: Editor → onchange(text) → Parent stores → Debounced save. Never Parent → Editor.
- **§2.6 — No `$effect` for Editor State.** No `$effect` reads or writes `value` / `editBody`. Only allowed: dir change (guarded by `prevDir`), font change (guarded by `prevFontKey`). **Violating §2.6 caused BUG-015** (see §8.1).
- **PaperOnDesk (PoD) layout**: gray desk `#e8e8ec`, white paper `max-width: 1200px`, `padding: 48px`.
- **Auto-title format**: code generates canonical `YYYYMMDDTHHMMSSZ_NOTE_XXXX` filename + `title:` field.

### 7.3 Audit-agent count (clarification)

Three sets exist; the umbrella is "14 audit agents":

- **[`lab/audit-agents.md`](lab/audit-agents.md) — 7**: PA / AA / MA / SCA / RA / UXA / CQA.
- **NotePane spec — 8**: above + **EA** (Environment Auditor), added 2026-03-27.
- **[`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) — 14**: 8 above + LA / SIA / SA / DIA / CFS / OGA.

Migrations use a different cohort: Phase 4 of `/migration` runs three parallel agents (Invariant Check / Drift Check / Migration Path).

### 7.4 `src/lib/libraries/store.ts` (write-ahead buffer, navigation)

**Stores**: `libraries`, `editingTabIds`, `openTabs`, `activeTabId`, `splitActive`, `focusedTabId`, `bookmarks`. Derived: `activeTab`, `universeNotesLibrary`, `selectedNote`, `focusedTab`, `libraryCount`, `totalStars`.

**Save discipline**:
- `saveLocks` map prevents concurrent writes per tab.
- `recentWrites` map (2 s TTL) gates the file watcher to ignore notes the app just wrote (prevents echo loops).
- **Write-Ahead Buffer**: in-memory + localStorage. `getWriteAhead()` checks memory first, falls back to localStorage (crash-safe). Cleared on tab close.
- `saveTabContent()`:
  - Auto-stamps "updated" / "حُدث" date if the property type === `date`.
  - Emits `screen:note-saved` for the second-screen window.
  - Async reindex via `constellation_search_reindex`.
  - Async semantic embed via `constellation_embed_notes`.
  - Tracks recent-edited in localStorage (20-deep) for second-screen dashboard.
  - **Does not dispatch to openTabs during autosave** — editor owns content, store re-syncs on tab switch.

**Navigation**: per-tab `_navTokens` prevent races on rapid Alt+Left/Alt+Right (newer click supersedes in-flight handler). 200-entry `_navTrace` ring-buffer exposed as `window.__navTrace`. Cross-library nav handled in `loadTabHistoryEntry`.

**Frontmatter parser**: multi-line YAML lists + inline `[a, b, c]`; type detection (list / link / checkbox / datetime / date / number / text); **Arabic property keys** recognized (`الوسم`, `وسوم`, `المجموعة`, ...); date normalization DD/MM/YYYY → YYYY-MM-DD.

### 7.5 `src/lib/secondScreen.ts` (12 events main→screen, 4 screen→main, 1 bidi)

**Window mgmt**: `openSecondScreen`, `openSecondScreenSmart` (auto-positions on secondary monitor at 80% size), `closeSecondScreen`, `isSecondScreenOpen`, `listMonitors`.

**Events**:
- **Main → Screen**: `screen:open-note`, `:universe-switched`, `:settings-changed`, `:context-changed` (editor/skyview), `:skyview-hover`, `:skyview-click`, `:sidebar-mode-changed`, `:split-mode-changed`, `:dashboard-open-note`, `:dashboard-tag-selected`, `:index-search`, plus workspace state restore.
- **Screen → Main**: `screen:open-in-main` (reverse-open), `:closed`, `:state-request` (workspace save), `:state-response` (restore).
- **Bidirectional**: `screen:note-saved` (both windows listen).

**Workspace State**: `ScreenState { mode: 'grid'|'star'|'detail'|'skyview'; linkedBrowsing; tabs; activeTabPath }`.

`src/lib/universe/store.ts` — 18 async invocation wrappers. **No local Svelte stores.** Pure IPC pass-through; Rust holds state.

---

## 8. Migrations (active state, 2026-04-28)

`/migration` — four-phase workflow: **Architect → Plan → Build → Audit**.

| ID | Plan | Status |
|---|---|---|
| **MIG-001** Sky View Write-Time Derivation | `lab/reports/MIG-001-SKYVIEW-WTD.md` | ✅ Closed. |
| **MIG-002** Enrichment Persistence | `lab/reports/MIG-002-ENRICHMENT-PERSISTENCE.md` | ⏳ §1–§6 shipped + tested. §7–§10 pending. |
| **MIG-003** Human-name Filenames | `lab/reports/MIG-003-HUMAN-NAME-FILENAMES.md` | ✅ Closed (2026-04-28). Steps 1–5 shipped (§85–§89); Step 6 (PK promotion) skipped by Boss decision; Steps 7–9 (docs + audit + PCS) shipped 2026-04-28. See § 6 of this orientation. |
| **MIG-004** Alias-Aware Resolution | `lab/reports/MIG-004-ALIAS-AWARE-RESOLUTION.md` | ✅ Closed. 9/12 invariants verified. |
| **MIG-005** Alias-aware in-memory inbound | `lab/reports/MIG-005-ALIAS-AWARE-INMEMORY.md` | ⏳ Steps 1–3 shipped (§121/§122/§123 — `map.rs` / `strata.rs` / `maturity.rs`). Tutorial paused after fabrication caught. Steps 4–8 pending. |
| **MIG-006** Wikilink Rename Cascade | `lab/reports/MIG-006-WIKILINK-CASCADE.md` | ⏳ §1 ✅. §2 ✅ + 11 cascade tests. §3 expanded shipped at `3c4732d`, **REVERTED at `5afe0c2`** (BUG-015). §3 redo + §4–§11 pending. |

### 8.1 The MIG-006 §3 / BUG-015 incident

- **§115** (`3c4732d`, 2026-04-25) shipped MIG-006 §3 expanded "open-editor coherence" — included a **value-prop → CM6 doc sync `$effect`** in NotePane that dispatched a doc-replace transaction on parent body-prop change.
- The `$effect` raced with `{#key tab.id+'|'+tab.path}` `onDestroy` on tab navigation. Click source → click target → reactivity propagated `tab.content` to target's body → OLD source NotePane's `value` prop changed → `$effect` replaced its own CM6 doc with target's body BEFORE `{#key}` ran destroy → destroy's `doFlush()` read the swapped doc → `handleFlush` wrote that swapped content to the OLD pane's `mountedFilePath`. Result: target file body overwritten with source body.
- **NotePane spec §2.6 explicitly forbade this pattern.** Spec wasn't read before commit.
- §116 (`5afe0c2`) reverted §115. §117 + §118 cleaned docs + recovered disk. BUG-014 closed as collateral.
- **Lesson**: per BASIC RULE + Working Agreement #4, every change touching write paths / lifecycle / reactivity / IPC contract MUST validate against the architecture before shipping. The MIG-006 §3 plan even documented a **fictional** "existing prop-change handler" that didn't exist — the plan misled itself.

---

## 9. Boot performance — 5 ship-gate criteria

`lab/boot-perf/BOOT-BUDGET.md`. Test corpus: **trial Universe (7,600 notes, 16 libraries, 656k typed links, 4k images on Windows 11 NTFS)**.

| # | Criterion | Status |
|---|---|---|
| 1 | UI visible ≤ 2.5 s | ✅ ~870 ms production (verified 2026-04-19) |
| 2 | Fully responsive (`hydrated_ms`) ≤ 6 s | ✅ closed at **811 ms** after Round 7 (LL-021) |
| 3 | Idle RSS ≤ 350 MB | 🔲 Not measured |
| 4 | Stat-sweep 50 externally-modified files ≤ 3 s, non-blocking | 🔲 Not implemented |
| 5 | Kill-mid-index recovery (no duplicate notes, no WAL corruption) | 🔲 Not implemented |

**Permanent diagnostic instrumentation** (kept after Criterion 2):
- **Five-stamp IPC diagnostic** (LL-021): `invoke_start_unix_ms` → `server_start_unix_ms` → per-phase `Instant::now()` → `server_return_unix_ms` → `client_recv_unix_ms`.
- **`perf_trace::TRACE_LOG`** at [`src-tauri/src/perf_trace.rs`](src-tauri/src/perf_trace.rs) — wraps `generate_handler!` to stamp every IPC dispatch arrival.
- **JS heartbeat** (max-gap from `boot:paint` to `boot:hydrated`).

### 9.1 What closed Criterion 2

`perf_trace` arrival tracer (Round 6) showed `constellation_map_universe` dispatched twice (~17.2 s gap), blocking `cache_boot_snapshot_core`. Round-7 fix: single attribute change `#[tauri::command]` → `#[tauri::command(async)]` on `constellation_map_universe`. `core_queue_ms` ~19.9 s → 4 ms; `hydrated_ms` 811 ms. **5,100× reduction.**

### 9.2 Other boot-perf primitives

- **Covering index** `idx_note_boot_snapshot ON note_meta(name, path, library_name)` — 100–1000× speedup (LL-020 corollary).
- **Paint-first UI** (LL-018): `appReady = true` synchronously; data hydrates after.
- **`LIBRARIES_CACHE`** (LL-016): in-memory cache for `load_all_libraries` invalidated by `save_libraries` + `set_active_universe`.
- **Always-mounted lazy-mount** (LL-022): `*EverOpened` flags for Map / OrgChart.
- **Watcher async** ([`watcher.rs:19-38`](src-tauri/src/watcher.rs:19) inline note): recursive watch is blocking I/O; sync command runs on WebView2 UI thread → Boot Criterion 2 fails.

### 9.3 Boot bundle — 10 IPCs into 1

[`boot_bundle.rs`](src-tauri/src/boot_bundle.rs) returns a single `BootBundle { libraries, settings, bookmarks, workspaces, property_types, workspace_bases, child_universes, child_universe_lib_paths, timings_ms[per step] }`. Replaces ~10 serialized invokes during `initializeApp`.

---

## 10. Standing rules (top-principal hierarchy)

### 10.1 BASIC RULE — Don't Make Things Up *(top of all rules)*

If I don't have a clue or information, I say **"I don't know."** No invented file paths, line numbers, function names, badge taxonomies, prior-art summaries, or any factual claim. **Fabrication is the worst class of error** — bugs are recoverable; trust isn't.

When tempted to add a "side note" — every claim in it must be sourced. If any claim isn't, the entire side note is cut.

Canonical violation prevented: 2026-04-26 tutorial fabricated T/C/P badge meanings as "Theory/Concept/Proposition." Actual: T = Title, C = Content, P = Property, with S = Semantic.

### 10.2 Working Agreement #1–#4

1. **Do the work yourself.** SQL, log greps, file inspection, build verification — Claude's job.
2. **One location: `E:\مشاريع كلاود\Constellation` on `main`.**
3. **The user is a non-technical IT Boss.** Plain language; tutorials per §10.4.
4. **Validate every change against the entire architecture before shipping.** Spawn parallel agents for any change touching write paths / lifecycle / reactivity / IPC. (BUG-015 is the canonical violation this rule prevents.)

### 10.3 Standing Orders

1. Update `lab/reports/SESSION-LOG-YYYY-MM-DD.md` after every phase / step / significant commit.
2. Update help files + User Manual + 14 translations on user-facing changes.
3. Session log = safety net for context loss.
4. `/simplify` (code review) after each phase.
5. **State-of-standing record before any pivot or major triage** — `§STATE-OF-STANDING` in the day's session log.
6. **Maintain `docs/Constellation Orientation & Onboarding vX.Y.md`** — filename always carries version suffix; rename in same commit on bump.

### 10.4 Tutorial Rule (top principal)

Every test instruction is a tutorial. Define the feature first (what / why / why it matters). Click-by-click walkthrough. Pre-state, action, post-state per step. Failure modes spelled out. Plain language only.

### 10.5 Plan Approval = Build Approval (top principal)

Once user approves a plan, Claude cascades through build steps autonomously. Stops only at: user-testable verification clauses, genuine architectural surprise, plan completion.

### 10.6 Migration Rule

Subsystem-crossing changes go through `/migration` four-phase workflow before any code is written. Single-file refactors → `/simplify`.

### 10.7 Performance Rules (8)

1. Every keystroke instant. Line-change guard for `selectionSet`. Pre-cache module-level Decorations.
2. No `$effect` loops. `$derived` for computed values.
3. No heavy work on the main thread. Vault indexing / search / file I/O → Rust. Debounce saves ≥1500 ms. **Zero `invoke()` on the keystroke hot path.**
4. No memory leaks. Every `setTimeout` / `setInterval` / `addEventListener` / `EditorView` / `listen()` / `requestAnimationFrame` → cleanup in `onDestroy`.
5. Minimal DOM. `display: none` not removal. No `:global()` cross-tree CSS.
6. No unnecessary imports. No `@codemirror/language-data` in FocusPane (500 KB+).
7. Test before commit. 10-char rapid type in NotePane + FocusPane after every change.
8. **Write-Time Derivation.** Every computed view maintained at write time. Persist + trigger on source-of-truth write path. Reads = cheap lookups. **No new feature may regress boot / typing / IPC** on the 7,600-note Universe.

### 10.8 Architecture principles

- **File Over App.** `.md` on disk = source of truth.
- **Local-First.** No telemetry, no cloud dependency.
- **Knowledge Formulation, not Management.**
- **The Living Link Architecture.**
- **Constraint as Design.** FocusPane has no toolbar — that IS the design.
- **Language-First by Design.** Bidi is architectural.
- **Constellation Knowledge Hierarchy** (5 levels).

### 10.9 Don't (hard "no" list)

- Don't use preview/screenshot tools unless essential.
- Don't add unnecessary abstractions.
- Don't use "vault" terminology in new code.
- Don't add a feature that makes the app slower.
- Don't commit `$effect` loops.
- Don't import heavy libraries in FocusPane.
- Don't use `position: absolute` for layout.
- Don't write CSS magic numbers without comment.
- **Don't patch the same bug more than three times** (LL-014).
- Don't create `Decoration.mark/replace/widget` inside builders — pre-cache.
- Don't call `invoke()` from a CM6 ViewPlugin or input event handler.
- **Don't duplicate working code by copy-paste-and-adapt** — extract.
- **Additional screens are displays, not domains.**

### 10.10 PCS Protocol

Push + Commit + Standing Order. Every milestone: verify build → commit → push → milestone tag → ZIP → session log → help files → 14 translations → SO.

### 10.11 Backup routine

`git tag milestone/<name> <commit>` + `git push origin --tags`. ZIP: `git archive --format=zip --prefix=Constellation-<name>/ HEAD -o "E:/Backups/Constellation/Constellation-<name>-YYYYMMDD.zip"`.

### 10.12 Versioned filename for this orientation doc — preserve every version

This file's name **always** carries its version suffix: `Constellation Orientation & Onboarding vX.Y.md`.

**Rule (corrected 2026-04-26):** when bumping the version, **write the new version as a NEW file**. Do NOT delete or overwrite the previous version. Older versions stay in `docs/` as a historical record — the project owner uses the trail to track how the project's architectural understanding evolved.

A new session reads only the highest-version file. But the trail behind it is durable.

---

## 11. Lessons Learned (LL-001 → LL-023, summary)

[`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) is canonical.

- **LL-001** Tauri IPC = #1 perf killer. Zero IPC during typing.
- **LL-002** `+layout.svelte` reactivity cascade. Direct mutation bypasses Svelte; never store-mutate from `onDestroy` or hot path. *(2026-03-27, file 3873 lines / 77/17/19. Today: 6872 / 155/29/1.)*
- **LL-003** Build passing ≠ working app.
- **LL-004** CM6 widget event handling — capture-phase `addEventListener` on editor DOM.
- **LL-005** `tauri dev` rewrites Cargo.toml. Use forwarding feature pattern.
- **LL-006** Phase-by-phase with user GO/NO-GO.
- **LL-007** Shared plugins in `src/lib/editor/` pay off.
- **LL-008** Session log = lifeline.
- **LL-009** Derive state, don't duplicate.
- **LL-010** Merge iteration loops over visible ranges.
- **LL-011** Tauri v2 asset protocol — 4 things: protocol-asset Cargo feature; assetProtocol enable+scope in tauri.conf.json; `http://asset.localhost` in CSP `img-src` AND `connect-src`; `https:` in `img-src`.
- **LL-012** `posAtDOM` unreliable for replacement widgets. Use `posAtCoords({x, y})`.
- **LL-013** `getCursorColumn` pipe-counting bug.
- **LL-014** **Three Strikes** — fix from root after 3 failed patches.
- **LL-015** Always test production before chasing dev-mode performance (~37 s/IPC dev overhead in Tauri v2 + Vite + DevTools).
- **LL-016** Cache at the call site when callers are unknown.
- **LL-017** When patching fails, spawn adversarial expert agents.
- **LL-018** **Paint-First UI** — never gate first paint on IPC.
- **LL-019** PIXI v8 + Tauri CSP — `import 'pixi.js/unsafe-eval'` as side-effect before any PIXI class. Never relax app-wide CSP.
- **LL-020** Wall-vs-server-time diagnostics. Plus covering-index corollary.
- **LL-021** Five-stamp IPC diagnostic + `perf_trace` arrival tracer. Methodology: Stage 1 stamps → Stage 2 plausible patches (stop after 2 fail) → Stage 3 cheap falsifiers → Stage 4 dispatcher tracer → Stage 5 named-culprit conversion.
- **LL-022** Always-mounted UI = always-running IPC. `*EverOpened` lazy-mount. Reset flags on context switch.
- **LL-023** Don't regress working features. 4-step verification: render → event → state → data path.

---

## 12. Documentation drift log

| Doc | Drift |
|---|---|
| [`docs/IPC-CONTRACT.md`](docs/IPC-CONTRACT.md) | Last 2026-03-31. Lists ~80 commands; actual ~120. |
| [`docs/CE-spec.md`](docs/CE-spec.md) | Body progress table at line 862-878 stale (says Phases 4 + 7 + 12-16 not started; roadmap and code show 1–11 done). |
| [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) | Says `cid`; code uses `cid_cn` namespace — see §6.1. |
| [`docs/Constellation-Editor-Spec.md`](docs/Constellation-Editor-Spec.md) | Describes a custom-built editor never built. CodeMirror 6 was used. **Aspirational.** |
| `lab/reports/MIG-006-WIKILINK-CASCADE.md:165-167` | The §3 plan claimed an existing prop-change handler that didn't exist. |
| Audit-agent count | `lab/audit-agents.md` = 7; NotePane spec = 8 (adds EA); `docs/AUDIT-SYSTEM.md` = 14. `lab/audit-agents.md` not updated to umbrella. |
| **CE Rule 8 audit-pending** | `bases.rs` (read-time `query_base`); `dataview.rs` (read-time); `lenses.rs` (hybrid violation: definitions write-time, results read-time on `apply_lens`); **Constellation Map** (`map.rs::constellation_map_universe` walks filesystem on every open). Sky View now write-time post-MIG-001. |
| **No frontend test harness** | No vitest / playwright / `*.test.ts` / `*.spec.ts` under `src/`. Rust unit tests only: 11 in `cascade_walker_tests`, 6 in `canonical.rs`, 4 in `file_kinds.rs`. |
| **No help topic for Constellation Map** | Sky View has [`docs/help.uConstellation.World/Sky View/Sky View.md`](docs/help.uConstellation.World/Sky%20View/Sky%20View.md). |
| Versioning | All three (`package.json`, `tauri.conf.json`, `Cargo.toml`) at 0.3.4 today. |
| Orientation v1.0 — auto-update toggle placement | v1.0 bug §13 said the toggle was wrongly placed under "Sky View & Links" and should be elsewhere. **The actual UI section is "Sky View & Links" and that's correct** (it's a links-cascade behavior, not a files-management one). v1.2 corrects: toggle is **correctly placed**. |

---

## 13. Outstanding bugs / cosmetic issues

| ID | Status |
|---|---|
| **BUG-013** open-editor cascade race | Open. Documented limitation: switch tabs before renaming a target whose source is visible. |
| **BUG-014** orphan `cid_cn` (collateral from BUG-012) | Closed §118 (2026-04-25). |
| **BUG-015** target-body corruption from §115 value-sync `$effect` | Vector removed at §116 (`5afe0c2`). Forensics in `lab/forensics/`. |
| Title-heading rename gap | **CONFIRMED**: [`NoteEditor.svelte:179-204`](src/lib/components/NoteEditor.svelte:179) handler calls `renameItem(filePath, newPath)` only — does **NOT** call `updateLinksOnRename`. The cascade is gated only by file-tree rename ([+layout.svelte:3807-3808](src/routes/+layout.svelte:3807) — conditional on `$appSettings.autoUpdateLinks && !isDir`). |
| Sidebar active-item highlight ~10 s lag | **Origin unresolved.** No reactive source / debounce / async refresh found that accounts for the 10 s; further forensics needed when it next reproduces. |

### 13.1 Badge taxonomy

Canonical reference: [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md). Render sites (must stay in sync per the source-of-truth invariant):
- [`ConstellationMap.svelte:80-84`](src/lib/components/ConstellationMap.svelte:80) — `CAT_COLORS` map; rendered at line 660 (current result) and line 711 (result list).
- [`ConstellationSight2.svelte:79-83`](src/lib/components/ConstellationSight2.svelte:79) — `CAT_COLORS` map.

**What badges mean.** A badge tells the user **where in the note the search query matched** (or what kind of link relationship the result represents). One result can carry multiple badges.

**Content / structural matches** (where in the note the match occurred):

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **T** | Title | Blue | `#3b82f6` |
| **C** | Content (body text) | Green | `#16a34a` |
| **P** | Property (frontmatter key/value) | Amber | `#f59e0b` |
| **S** | Semantic (embedding similarity) | Purple | `#7c3aed` |
| **W** | Wikilink (`[[target]]`) | Grey | `#94a3b8` |
| **#** | Tag / Hashtag (`#tag` or YAML `tags:`) | Pink | `#f472b6` |
| **∅** | Empty / Null result | Slate | `#64748b` |

**Link-relationship badges** (matched by virtue of how the result links to/from the queried note):

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **LT** | Link **Target** (this note links *to* the queried note) | Green | `#16a34a` |
| **LF** | Link From (this note is linked *from* the queried note) | Red | `#ef4444` |
| **⇄** | Bidirectional (mutual link in both directions) | Violet | `#8b5cf6` |
| **LB** | Link Back (backlink hit) | Light blue | `#0ea5e9` |
| **LA** | Link Alias (matched via the link's display alias rather than its target) | Pink | `#d946ef` |
| **M** | Mutual link (the queried note links *to* the source AND the source links *back*) | Cyan | `#06b6d4` |

**Deprecated**:

| Badge | Status |
|---|---|
| **G** | Earlier identifier for Tag/Hashtag. Superseded by **#**. Not present in current code. |

**Unresolved**: none. M was the last pending letter; resolved 2026-04-27 as Mutual link.

**Adding a new badge**: see `docs/Badge-Taxonomy.md` § "Adding a new badge" — must update both `CAT_COLORS` maps in lock-step + this section + Badge-Taxonomy.md.

### 13.2 Filter chips on Constellation Map ([`ConstellationMap.svelte:114-125`](src/lib/components/ConstellationMap.svelte:114))

These are **search-syntax helpers**, not letter badges:
`linksTo` (`links to [[`) · `linksFrom` (`links from [[`) · `orphans` · `tag` (`#`) · `supports` (`supports [[`) · `contradicts` (`contradicts [[`).

### 13.3 Auto-update-links toggle path

**[`SettingsModal.svelte:1395-1428`](src/lib/components/SettingsModal.svelte:1395)** — under section `activeSection === 'skyview'` (display label "Sky View & Links"). Toggle binds to `$appSettings.autoUpdateLinks`. Cascade trigger ([`+layout.svelte:3807`](src/routes/+layout.svelte:3807)):

```
if ($appSettings.autoUpdateLinks && !isDir) {
  await updateLinksOnRename(lib.path, oldName, newName);
}
```

---

## 14. Where to read what (index)

| Topic | Source |
|---|---|
| Why Constellation exists / vision | [`docs/Constellation — Concept Paper.md`](docs/Constellation%20—%20Concept%20Paper.md) |
| Living Link philosophy + 8 properties + 7 types + 6 lifecycle stages | [`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`](docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md) |
| Cognitive Engine 16-phase spec | [`docs/CE-spec.md`](docs/CE-spec.md) + [`docs/cognitive-engine-roadmap.md`](docs/cognitive-engine-roadmap.md) |
| Canonical filename + 12 kinds + import pipeline | [`docs/CANONICAL-FILENAME-ARCHITECTURE.md`](docs/CANONICAL-FILENAME-ARCHITECTURE.md) |
| NotePane editor rules | [`docs/NotePane-spec.md`](docs/NotePane-spec.md) |
| Audit system (7 / 8 / 14) | [`docs/AUDIT-SYSTEM.md`](docs/AUDIT-SYSTEM.md) + [`lab/audit-agents.md`](lab/audit-agents.md) |
| Migration four-phase workflow | [`.claude/skills/migration.md`](.claude/skills/migration.md) |
| PCS protocol | [`docs/PCS-PROTOCOL.md`](docs/PCS-PROTOCOL.md) |
| Working protocols / Tutorial Rule | [`docs/WORK-BEHAVIOR.md`](docs/WORK-BEHAVIOR.md) |
| Hard-won rules from real bugs | [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md) (LL-001 → LL-023) |
| Migration plans | `lab/reports/MIG-NNN-*.md` |
| Active boot-perf budget | [`lab/boot-perf/BOOT-BUDGET.md`](lab/boot-perf/BOOT-BUDGET.md) |
| What's in flight today | `lab/reports/SESSION-LOG-{latest-date}.md` |
| Subsystem status snapshot | [`lab/reports/STATUS.md`](lab/reports/STATUS.md) |
| User-facing feature docs | `docs/help.uConstellation.World/<topic>/<topic>.md` (24 topics) |
| Master User Manual (English, 25 chapters) | [`docs/User Manual.md`](docs/User%20Manual.md) |
| 14 translated User Manuals | `docs/help.{ar,de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}/User Manual.md` (ar = 1328 lines, others = 1120) |
| **Tauri command registry (authoritative)** | [`src-tauri/src/lib.rs:233-432`](src-tauri/src/lib.rs:233) |
| Tauri config / windows / CSP | [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) |
| Window permissions | [`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json) |
| Release workflow (CI) | [`.github/workflows/release.yml`](.github/workflows/release.yml) |
| Bases MVP | [`docs/BASES_MVP_SPEC.md`](docs/BASES_MVP_SPEC.md) |
| Badge taxonomy (canonical reference) | [`docs/Badge-Taxonomy.md`](docs/Badge-Taxonomy.md) |
| eNotePane build history | `docs/eNotePane-development-record.md` + `lab/experiments/phase-N-*.md` |
| Forensic snapshots | `lab/forensics/` |

---

## 15. Session-start protocol

1. **`git pull origin main`** to sync.
2. **`git log --oneline -10`** for recent work.
3. **Read `lab/reports/SESSION-LOG-{latest-date}.md`**. Look for `§STATE-OF-STANDING`.
4. **Read THIS document** (`docs/Constellation Orientation & Onboarding vX.Y.md`).
5. **Read [`docs/LESSONS-LEARNED.md`](docs/LESSONS-LEARNED.md)** — every rule was earned by a real bug.
6. **Read [`CLAUDE.md`](CLAUDE.md)** — top-principal rules + Working Agreement + Standing Orders.
7. **Read [`lab/reports/STATUS.md`](lab/reports/STATUS.md)** — one-page subsystem status index.
8. **Read memory files** at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\MEMORY.md` and linked entries.

If any contradict each other, ground in the code (`grep`) and update the stale doc in the same session.

### 15.1 Tools you'll need

- `gh` — GitHub CLI (release pipeline, PR ops).
- `git`, `npm`, `cargo`, `tauri` (`npm run tauri`).
- `sqlite3` for direct DB inspection (Rust side ships `rusqlite` bundled — no external sqlite3 required at runtime, but useful at dev time).

### 15.2 Boot pipeline summary

1. `paint:start` → `paint` (target ≤ 870 ms — Criterion 1) → app shell visible.
2. `cache_boot_snapshot_core` (note list, libraries, settings) — awaited.
3. `cache_boot_snapshot_graph` (links, tags, aliases) — deferred via `requestIdleCallback`.
4. `cache_boot_snapshot_sky` (pre-shaped sky_nodes + sky_links from triggers) — parallel with graph.
5. `boot:hydrated` (target ≤ 6 s — Criterion 2; achieved 811 ms).

### 15.3 Wikilink resolution + collision tiebreak

Three-tier resolution ([`cache.rs:553-588`](src-tauri/src/cache.rs:553)):
1. **`name_to_idx` hit** → use canonical id.
2. **`alias_to_path` hit** → resolve to canonical path → bump on canonical row.
3. **Unresolved** → fall back to lowercase comparison; orphan edge skipped.

**Tiebreak under collisions**:
- Two notes with identical title (case-insensitive): **Unresolved** — both match, no deterministic winner.
- Title equals another note's alias: **Name wins** (tier 1 precedes tier 2).
- Two notes share an alias: **First-write-wins** — `alias_to_path` is single-valued; insertion order undefined. Practical advice: avoid shared aliases.

---

## 16. Standing Order #6 (this document's maintenance contract)

Update this document in the same commit when:

- A migration starts, ships a step, or closes.
- A top-principal rule is added or reworded.
- A BUG-NNN opens or closes.
- A drift item from §12 is fixed (remove the row).
- A new LL-NNN is added.
- A boot-perf criterion changes or closes.
- A version bumps (`Cargo.toml`, `package.json`, `tauri.conf.json`).
- A subsystem ships a major feature.
- A help topic ships or restructures.

**Bump version (1.x → 1.y)** on structural changes. **Write the new version as a NEW file** in the same commit (filename always carries version suffix per §10.12). **Do NOT delete the previous version** — every version stays as a historical record. Date-stamp every section that updates.

The document **must remain readable in one pass.** If it grows past ~1500 lines, split into linked sub-documents in `docs/orientation/`.

---

## 17. What I (Claude) have NOT read in detail (v1.2 — significantly reduced)

This list is mandated by the BASIC RULE. If you need certainty on a claim that touches an "unread" file, **read it before acting**.

**Source code I have NOT read in full**:
- Some sections of `search.rs` (4790 lines), `libraries.rs` (3978) — read at section level, not line-by-line. Function signatures, schema, triggers, command surface confirmed.
- `+layout.svelte` (6872 lines) — structural map only (region table + $effect inventory + IPC list + component mount list). Not line-by-line.
- `libraries/+page.svelte` (704), `skills/+page.svelte` (219) — listed and counted, not read.

**Docs I have NOT read in full**:
- 14 translated User Manuals (parity confirmed: ar = 1328 lines, others = 1120; same chapter structure).
- `docs/User Manual.md` chapters beyond TOC + opening paragraphs.
- Binary docs (`docs/Constellation_Lens_Concept_Paper_Eisa.pdf`, `docs/GraphMind*.docx`, `docs/constellation_cognitive_engine_v2.1.pdf`) — text tools cannot extract reliably.

**Session logs partially read**:
- 2026-04-18 (1.46 MB): structural digest + sampled headlines (Arabic Engine M3-M14 milestone day).
- 2026-04-19 (99 KB): structural digest.
- All 20 logs digested chronologically (see §11 / §15 / §16 references throughout this doc).

**Specifics I do NOT know**:
- **Sidebar active-item highlight ~10 s lag origin** — no reactive source / debounce / async refresh isolates the lag. Reproduce-and-instrument needed.
- **Why the alias-aware sky snapshot path (`cache_boot_snapshot_sky`) is bypassed at boot** in builds that contain MIG-001 / MIG-004 §8 / MIG-005. The §88 defensive fix neutralizes user impact, but the underlying "why" is unresolved.
- **Whether `2026-04-16.UNTRACKED-BACKUP.md` (3.8 KB) and the tracked `2026-04-16.md` (13 KB) diverge in content** — sizes differ; backup may be checkpoint or partial draft. No content-level diff performed.
- **Whether the SECTOR_THRESHOLD = 8 cut-off feels right at the boundary** (v1.9 §104). The hybrid layout flips from sector to ring-per-group when the largest typed-link group exceeds 8 notes. Below 8 the sector layout looks balanced; above, the rings layout. The threshold itself is arbitrary; if Boss reports flips happening at the wrong moment for their data, the constant is one edit. Right now no data point either way.
- **Visualisation-mode distinctness (Stage 2E, deferred)** — at v1.9 commit time Boss had not yet flagged the three modes (Atmospheric / Neural / Cosmic) as too similar after the §103/§104 changes. The mode-specific decorations were redesigned to differentiate (rotating ellipses vs faint dashed rings vs solid coloured rings + sector lines + rim labels), but it's not Boss-confirmed. Triage only if flagged in 2E retest.

**Resolved during v1.9** (folded into §4.2 row 12 above, removed from §17):
- *Actual `get_360_view` latency on 7,600-note Universe.* Boss reports "almost instantly". MIG-010 priority dropped to LOW.
- *Inspector 360 first-fetch empty-state UX.* Confirmed not jarring in practice — the IPC is fast enough that the empty state barely shows.

**Resolved this session (2026-04-27):**
- **M = Mutual link** (was unresolved badge letter through v1.3). Confirmed by project owner; folded into §13.1 + Badge-Taxonomy.md.
- **W = Wikilink** (was unresolved through v1.1). Resolved earlier via Badge-Taxonomy.md.

**Future maintainers**: when you read one of the above and confirm a fact, update §17 to remove it AND fold the verified fact into the relevant section above. Keep §17 honest.

---

*End of v1.14. Maintained per Standing Order #6.*
