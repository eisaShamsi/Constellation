# Safety Inspection — 2026-08-03 (per-build, ran whole-app)

Invoked diff-scoped over the 14 changed files; the workflow reported `mode: whole-app` (the
PJ-166 behaviour, now seen a fifth time) and swept all 14 scopes. **39 confirmed**, each
adversarially refuted before acceptance.

## Fixed before this commit

| File | What | Note |
|---|---|---|
| `store.ts` | **APP-KILLER — folder delete destroyed unsaved work.** `preserveWorkBeforeVacating` matched only the EXACT path while its two sibling steps matched descendants, so deleting a folder flushed nothing, then disposed every open note inside it and wiped their recovery nets. | Found by 4 independent hunters. Both halves now share `vacatedBy`. |
| `store.ts` | **Residual on that fix** — `vacatedBy` compared raw strings while the clear side normalises case/separators, leaving the clear side broader than the flush side. | Same normaliser now. |
| `style_presets.rs` | `save_style_presets` used plain `fs::write` — making the READ strict without making the WRITE atomic just relocates the failure. | `atomic_write`. |
| `NotePane.svelte` | The half-built `EditorView` was never disposed when init threw — listeners and the whole document leaked per re-open (Rule 4). | Disposed before the fallback. |

## Confirmed and FILED, not fixed in this pass

These are pre-existing defects in the swept files, not introduced by this diff. They join the
triage register (PJ-200 series) rather than being silently absorbed.

| Severity | File:line | Summary |
|---|---|---|
| APP-KILLER | `src/lib/libraries/store.ts:3460` | A session-restored tab whose content came from the write-ahead net is seeded as a CLEAN model (`openNoteModel` + `setModelDiskBaseline`), so every dirty-refusal guard downstream (`reloadTabsFromDisk`'s `isNoteDirty` check, `adoptDisk`'s `is |
| APP-KILLER | `src/lib/libraries/store.ts:2460` | parseFrontmatter treats a SCALAR wikilink value (`key: "[[X]]"`) as a YAML flow sequence, so `[[X]]` projects as `[X]` — and the next write persists the stripped form, destroying one bracket pair per read→write cycle until the wikilink is g |
| APP-KILLER | `src-tauri/src/boot_bundle.rs:138` | The boot bundle collapses list_link_types' deliberately-strict Err into an EMPTY vocabulary via .unwrap_or_default(), defeating the 2026-08-02 fix at the one path that actually feeds the frontend at boot. |
| HIGH | `src/lib/libraries/store.ts:343` | The crash-recovery write-ahead net is an unbounded, never-pruned localStorage blob whose quota failure is swallowed by an empty `catch {}` — past a threshold every subsequent stash silently stops persisting while the save-health banner keep |
| HIGH | `src/lib/libraries/store.ts:1626` | flushOpenTabOrAbort writes a dirty model to disk with no isCascading gate — the exact gate its sibling flushOutgoing carries (store.ts:695) — so any of its three sidebar-reachable callers can flush a pre-cascade body over a file the rename  |
| HIGH | `src/lib/libraries/store.ts:2991` | openNoteTab performs an unguarded ensure_cid_cn_cmd disk write to the note's .md frontmatter with no displayOnlyWindow gate, so the read-only second screen writes to notes — and the write is a read-modify-write whose read is taken outside t |
| HIGH | `src-tauri/src/libraries.rs:1295` | migrate_note_db_paths pre-deletes only note_meta / note_embeddings / review_schedule / note_body / note_summaries / sources_suggestions / sight_v3_layout / note_state_history at the DESTINATION path, but never note_aliases — so a phantom ro |
| HIGH | `src-tauri/src/sources/mod.rs:503` | The `sources:` / `content_type:` block-strip stops skipping at the first COMMENT or indented continuation line and then re-emits the block's remaining `- item` lines at top level, producing frontmatter that no longer parses. |
| HIGH | `src/lib/libraries/linkTypeRegistry.ts:126` | The link-type registry is the only persisted store with no read-succeeded latch: seedFromBundle sets loaded = true unconditionally, loadLinkTypes' catch also sets it, and saveLinkTypes never checks it. |
| HIGH | `src-tauri/src/sources/mod.rs:488` | Accepting a classifier suggestion strips the note's ENTIRE `sources:` (and `content_type:`) frontmatter key — including values that are not in the 53-id taxonomy — because the PJ-091 "never subtract" union is computed over the ALREADY-FILTE |
| MED | `src-tauri/src/libraries.rs:398` | When a folder rename/move/delete cannot write the retargeted libraries.json, the failure is reported only via `eprintln!` — invisible in a release build — and the affected library silently vanishes from every list, tree and resolver. |
| MED | `src-tauri/src/libraries.rs:1418` | The FOLDER branch of `rename_item` runs the whole descendant DB cascade and the per-descendant reindex loop synchronously inside the awaited IPC, holding the app-wide SearchState writer mutex across the entire migrate loop — the exact unbou |
| MED | `src/routes/+layout.svelte:7124` | The rename wikilink cascade walks ONLY the renamed note's own library, so every `[[OldTitle]]` referrer living in any OTHER library of the same universe is silently left un-rewritten and never reported as failed. |
| MED | `src/lib/libraries/store.ts:3011` | `openNoteTab`'s cid-injection reindex passes the caller's UNTRUSTED `libraryName` argument, which the very same function distrusts 45 lines later — stamping `note_meta.library_name` with the wrong (or empty) library, durably and silently. |
| MED | `src/lib/components/FocusPane.svelte:330` | A title edited in Focus mode is committed ONLY from handleTitleBlur; the onDestroy teardown flushes the body (flushNow → onflush → commitFocusSave) but never routes the pending title through handleTitleBlur, so any exit that does not move D |
| MED | `src/lib/components/NotePane.svelte:1034` | Same title-commit hole at the other editor surface: NotePane's title is committed only from the input's onblur (handleTitleBlur, NotePane.svelte:1146), and onDestroy's doFlush() persists only the body — so a keyboard-driven tab change, in-p |
| MED | `src/lib/libraries/store.ts:1497` | linkMentionInNote's OPEN-note branch has no rename-cascade guard (the guard is deliberately scoped `!openTab`), so its JS-level read-modify-write can silently revert the walker's wikilink rewrite of that note. |
| MED | `src/lib/components/NotePane.svelte:887` | The task-checkbox mousedown listener dispatches a document change with no readOnly check, and the table toolbar (rendered at :1752, ungated) does the same via applyTableChange — so on the read-only second screen the edit visibly succeeds wh |
| MED | `src-tauri/src/link_life_restore.rs:397` | The boot ledger-restore flips note_links.status between 'active' and 'archived' but never recomputes the target's note_meta.incoming_* aggregates or either endpoint's sky_nodes.stratum/maturity — the exact recompute the archive/unarchive CO |
| MED | `src-tauri/src/search.rs:11136` | A failure of the write-time incoming / sky maintenance on the save path is reported only via eprintln!, which goes nowhere in a Windows GUI release build — and since boot is walk-free, the resulting divergence has no self-heal despite the c |
| MED | `src-tauri/src/canonical.rs:1316` | ensure_cid_cn renames ANY root-level `cid:` property to `cid_cn:` with no format validation, silently destroying a user's own `cid` property on disk and adopting its value as Constellation's durable note identity. |
| MED | `src/lib/libraries/propertyTypeRegistry.ts:112` | Property types is the one latched store whose read-refusal and write-failure are console-only — loadError is a plain module variable, not a store, so nothing can render it, and it is absent from the storeHealthError banner that covers colle |
| MED | `src-tauri/src/canvas.rs:76` | write_canvas overwrites the user's whole .canvas document with a plain truncate-then-write on a 1-second-debounced hot path, and the reader that hits the resulting torn file swallows the error with a bare `catch {}`. |
| MED | `src-tauri/src/cece/orchestrator.rs:153` | `run_one_safe`'s per-cataloger timeout is illusory — `std::thread::scope` joins the worker before returning, so `recv_timeout` only changes the RESULT (a fabricated abstain trail) and never bounds the DURATION; the ensemble silently records |
| MED | `src/lib/components/UniverseSetup.svelte:273` | `migrateLocalStorage` wraps all four legacy→universe migration writes in `try { … } catch { /* ignore */ }` and then UNCONDITIONALLY deletes every `constellation-*` localStorage key — a failed migration destroys the only copy of the user's  |
| LOW | `src/lib/components/NoteEditor.svelte:310` | Every save-path re-embed is fire-and-forget with `.catch(() => {})`, so `note_embeddings` silently keeps the note's pre-edit vector forever — the exact class the 2026-08-01 inspection fixed for `reindexNote` at the sibling line, left unfixe |
| LOW | `src-tauri/src/libraries.rs:1295` | The same destination pre-delete removes a note_meta row with a raw DELETE instead of via reindex_delete_note, so the phantom note's tags are never subtracted from the additive `tag_counts` table — which has no per-note provenance and theref |
| LOW | `src-tauri/src/props_reparse_backfill.rs:141` | The one-shot properties re-parse serializes the frontmatter HashMap directly (`serde_json::to_string(&properties)`), reintroducing the non-deterministic key order that MIG-104 Slice 2a removed from index_note — so the rows it 'fixes' are wr |
| LOW | `src-tauri/src/canonical.rs:1340` | ensure_cid_cn probes the WHOLE FILE for `\ncid_cn:` / `\ncid:` instead of the frontmatter block, so a body line beginning with either key permanently denies the note its durable identity, silently. |
| LOW | `src/lib/libraries/store.ts:4346` | buildDefaultFrontmatter concatenates `${key}: ${value}` without quoteIfNeeded — the third, unswept instance of the PJ-187 defect — so a default property whose value needs quoting makes every newly created note's frontmatter malformed from b |
| LOW | `src-tauri/src/arabic/overrides.rs:400` | save_to_path stages to a FIXED temp name (arabic-overrides.json.tmp) with no fsync before the rename — the exact PJ-087 defect universe.rs:183 documents as forbidden — while the module's own docs claim concurrent multi-window edits are made |
| LOW | `src/lib/components/PropertyEditor.svelte:1034` | The debounced property save writes the composed content into the tab-store entry BEFORE `commitAndSave`'s identity refusal runs, so on an in-place navigation the outgoing note's frontmatter is spliced onto the incoming note's `tab.content`  |
| LOW | `src/lib/components/ConflictMergeView.svelte:113` | `rebuild()` destroys the current MergeView, then awaits `tick()`, `readNote()` and a dynamic `import('@codemirror/merge')` before assigning the new one — with no destroyed/generation guard, so a close or unmount during those awaits orphans  |

## Scope coverage

| Scope | Confirmed |
|---|---|
| rename-move-delete-gate | 4 |
| note-save-index | 4 |
| notemodel-ownership | 2 |
| editor-lifecycle | 2 |
| rename-cascade-integrity | 2 |
| cross-window-integrity | 2 |
| derived-index-triggers | 5 |
| frontmatter-property-writes | 5 |
| persisted-json-state | 6 |
| cece-sources-derived | 2 |
| frontend-write-callers | 2 |
| boot-init-ordering | 0 |
| reactivity-concurrency | 1 |
| freeze-and-leaks | 2 |

69 agents · 1,879 tool calls · ~25 min.
