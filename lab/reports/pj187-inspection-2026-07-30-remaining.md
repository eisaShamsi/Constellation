# PJ-187 — pre-commit inspection (2026-07-30): findings NOT in the sweep's diff

Run wf_860b6505-c41 · 85 agents · mode returned **whole-app** (invoked diff-scoped with
`args.files` — the PJ-166 defect, NINTH strike) · 34 confirmed, 25 verify agents lost to
server errors (those candidates died unverified — an under-count, not an all-clear).

**Six findings were in the sweep's own diff and are FIXED + RED-proven before commit**
(store.ts:624 ok:true→silent-discard APP-KILLER; :1820 latch never reset on universe switch;
:1833 stale-payload retry; :3098 drainCidEnsure missing reindex; :1737/:2865 aborted nav
consuming the incoming note's recovery net).

The rest are pre-existing at HEAD — this file is the triage feed for the ledger.

- **[APP-KILLER]** `src/lib/libraries/store.ts:4195` — renameItem's flush-FAILED branch repaths the model without re-basing it, so the model's G4 write-base still holds the PRE-rename frontmatter — the next (auto-retried) save re-emits it verbatim and silently reverts title 
- **[HIGH]** `src-tauri/src/search.rs:1639` — `incoming_signature` keys only on the DISTINCT lowercased TARGET NAME (search.rs:1690), so the save-path diffs `maintain_incoming_after_save` (1639) and `sky_affected_paths` (1675/1681) compute an EMPTY affected set when
- **[HIGH]** `src/routes/+layout.svelte:2779` — The "reconcile is the authoritative self-heal" contract asserted at five maintenance sites (search.rs:1627, 1694, 1734, 10145, review.rs:1309) is unreachable in normal operation: `reconcile_filesystem` — the sole product
- **[HIGH]** `src/lib/libraries/store.ts:2273` — `parseFrontmatter` only recognises a flow sequence that opens AND closes on the key's own line, so a MULTI-LINE flow list is projected truncated and mangled — and `immutableBlockKeys` leaves it editable, so the next tag/
- **[HIGH]** `src/lib/libraries/store.ts:2110` — A MULTI-LINE plain or quoted scalar is projected as only its first line (the continuation is skipped by the top-level-key guard) and is NOT held immutable — `immutableBlockKeys` guards only `|`/`>` block scalars — so edi
- **[HIGH]** `src-tauri/src/review.rs:814` — load_pulse_data's READ-error branch returns ReviewPulseData::default() with no backup, and its three callers are whole-file read-modify-write commands that immediately write that default back — erasing the user's entire 
- **[HIGH]** `src-tauri/src/sources/bulk_ops.rs:419` — Approve-All rewrites a note's `sources:` / `content_type:` frontmatter from a taxonomy-VALIDATED list, so any value in those keys that is not a Constellation taxonomy slug is silently deleted from the .md file on disk.
- **[HIGH]** `src-tauri/src/sources/mod.rs:491` — When the note's own `sources:` / `content_type:` key holds a YAML block scalar (`|` or `>`), the rewriter drops only the key line and leaves the indented continuation lines orphaned, producing frontmatter that no longer 
- **[HIGH]** `src-tauri/src/sources/mod.rs:602` — The PJ-187 re-base announce is emitted on the `library-changed` channel, which the frontend filters through `wasRecentlyWritten` — so an accept that lands within 2 s of the open note's own autosave is dropped and the not
- **[HIGH]** `src/lib/libraries/store.ts:6613` — A failed read of workspaces.json presents as 'no saved workspaces', and the next workspace save writes that emptiness over the file — the same load-guard defect the sweep just fixed for collections (store.ts:1867), at th
- **[HIGH]** `src-tauri/src/boot_bundle.rs:101` — `constellation_boot_bundle` swallows a `read_universe_settings` failure with `.unwrap_or({})`, and the frontend has no "did the read succeed?" guard — so a transient read/parse failure at boot leaves DEFAULT_SETTINGS in 
- **[MED]** `src/lib/libraries/store.ts:1470` — `flushAllTabsInLibrary` flushes every dirty open note to disk with a bare `standardSaveEnv` (no `onSaved`), so a flushed note that the rename cascade does not itself rewrite is left with `note_meta`/`notes_fts`/`note_lin
- **[MED]** `src-tauri/src/search.rs:8998` — `constellation_link_archive` / `constellation_link_unarchive` flip `note_links.status`, which is a direct input to `note_meta.incoming_*` and `sky_nodes.stratum/maturity`, but neither path recomputes either — no incoming
- **[MED]** `src-tauri/src/search.rs:10085` — In `reindex_delete_note`, the SKY stratum/maturity recompute for the deleted note's former targets (search.rs:10164) is nested inside `if let Some(targets) = inc_targets`, which is gated on `incoming_links_backfill::is_s
- **[MED]** `src/lib/libraries/store.ts:2274` — The inline-flow-list parser splits on every comma with no awareness of quoting, so one quoted list item containing a comma becomes two items — and the split is made permanent on the next write to that key.
- **[MED]** `src/lib/libraries/store.ts:6617` — persistWorkspaces() has no loaded-guard: loadWorkspaces swallows a read failure and leaves the store empty, so the next Save Workspace atomically replaces workspaces.json with a one-entry file.
- **[MED]** `src/lib/libraries/propertyTypeRegistry.ts:25` — The `loaded` flag is written but never read: a failed property-types load sets cache = {} and the next setRegisteredType persists {} plus one entry, wiping every library's property-type assignments.
- **[MED]** `src-tauri/src/style_presets.rs:36` — load_style_presets silently returns [] on a corrupt file (no backup, not even an Err), and save_style_presets writes non-atomically — a truncated write feeds itself back as "you have no styles" and the next save makes th
- **[MED]** `src-tauri/src/cece/orchestrator.rs:153` — The per-cataloger timeout is illusory: `std::thread::scope` joins the worker before returning, so `recv_timeout` bounds only the reported verdict, never the actual wait — a hung cataloger still parks the caller for its f
- **[MED]** `src/lib/libraries/store.ts:583` — Every app-driven save fires its FTS/note_meta reindex fire-and-forget with `.catch(() => {})`, and because the app's own writes are watcher-suppressed and boot does not walk, a single rejected reindex leaves the index pe
- **[MED]** `src/lib/components/SenseMakingCanvas.svelte:147` — The Sense-Making Canvas autosave swallows every write failure with a bare `catch {}` — no banner, no console, no write-ahead net, no save-health entry — so a canvas whose file cannot be written keeps accepting edits that
- **[MED]** `src/lib/libraries/store.ts:1590` — saveTabContent's post-save hook looks the tab up by path and SKIPS the reindex and the re-embed entirely when the tab is gone, so a property write that lands on disk after its tab closed never reaches the index at all.
- **[MED]** `src/lib/libraries/propertyTypeRegistry.ts:25` — `loadPropertyTypes`'s catch sets `cache = {}` AND `loaded = true`, and `constellation_boot_bundle` (boot_bundle.rs:131) degrades a failed property-types read to `{}` — so a read failure is indistinguishable from "no type
- **[MED]** `src/lib/components/SenseMakingCanvas.svelte:144` — The canvas's 1 s debounced save resolves `canvasPath`/`items` at FIRE time, so switching canvases inside the window writes the new canvas over itself and silently discards the pending edit — and `catch {}` hides every wr
- **[LOW]** `src/lib/components/PropertyEditor.svelte:1034` — In the 800 ms debounce body, `tab.content` is overwritten with `buildFullContent(editableProps, body)` using the LIVE `tabId` but this panel's STALE rows — and it runs BEFORE the PJ-187 `rowsBelongToTarget` refusal, so t
- **[LOW]** `src-tauri/src/style_presets.rs:51` — save_style_presets uses a plain truncate-then-write fs::write — the last persisted-state writer in the app with a crash window, on a file whose loader silently degrades to [].
- **[LOW]** `src-tauri/src/shape.rs:229` — `record_change` returns early (`let Some(conn) = guard.as_ref() else { return }`) when `state.db` is `None`, so the shape-history undo record — which lives ONLY in search.db — is silently dropped while the `.md` write it
