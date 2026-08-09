# Constellation — IPC Contract

> Every `invoke()` call between the frontend (JS/Svelte) and the Rust backend must be documented here.
> This is a **living document** — update it whenever a new command is added or removed.
> Consult before adding any new Tauri command.

---

## Design Rules

1. **Zero IPC on the keystroke hot path.** No `invoke()` inside CM6 ViewPlugins, input event handlers, or any code that runs on every keystroke.
2. **Debounce user-triggered queries ≥ 300ms.** Search, link resolution, and similar user-input-driven calls must be debounced. Cancel the previous call if a new one arrives.
3. **Saves are debounced ≥ 1500ms.** `write_note` is never called on every edit — only after idle time, on blur, or on close.
4. **Prefer push over pull.** Rust → frontend updates via Tauri events (`emit`/`listen`), not frontend polling `invoke()` in a loop or timer.
5. **Payload budgets.** File content payloads: unbounded (unavoidable). Metadata/index payloads: keep under 100KB per call. Search results: max 200 items per response.

---

## Command Registry

### File I/O
| Command | Caller | Trigger | Notes |
|---|---|---|---|
| `read_note` | `store.ts`, `+layout.svelte` | Tab open, file-watch event, template load | Never on keystrokes |
| `write_note` | `store.ts`, `+layout.svelte` | 1500ms debounce after edit, on blur, on close | Safe — debounced |
| `read_note_preview` | `store.ts` | Hover preview, backlink panel | On demand only |
| `create_note` | `store.ts` | User action (new note button / command) | One-shot |
| `create_folder` | `store.ts` | User action | One-shot |
| `rename_item` | `store.ts` | User action | One-shot |
| `delete_item` | `store.ts` | User action | One-shot |
| `read_library_tree` | `+layout.svelte`, `+page.svelte` | Library switch, startup, folder expand | Acceptable; not on hot path |
| `get_templates_dir` | `+layout.svelte` | Template open | One-shot |
| `list_templates` | `+layout.svelte` | Template picker open | One-shot |
| `save_clipboard_image` | `store.ts` | Paste event | One-shot |

### Library & Universe Management
| Command | Caller | Trigger | Notes |
|---|---|---|---|
| `add_library` | `store.ts`, `UniverseSetup.svelte` | User action | One-shot |
| `remove_library` | `store.ts`, `UniverseSetup.svelte` | User action | One-shot |
| `create_new_library` | `store.ts` | User action | One-shot |
| `list_libraries` | `store.ts` | Startup | One-shot |
| `resolve_universe_libraries` | `store.ts` | Startup | One-shot |
| `get_all_library_stats` | `store.ts` | Dashboard open | On demand |
| `pick_folder` | `store.ts`, `UniverseSetup.svelte` | User action (folder picker) | OS dialog — one-shot |
| `scaffold_starter_library` | `UniverseSetup.svelte` | First-run setup | One-shot |
| `read_library_appearance` | `store.ts` | Library switch | One-shot |
| `watch_library` | `store.ts` | Library open | One-shot |
| `unwatch_library` | `store.ts`, `+layout.svelte` | Library close | One-shot |
| `list_universes` | `universe/store.ts` | Startup | One-shot |
| `create_universe` | `universe/store.ts` | User action | One-shot |
| `set_active_universe` | `universe/store.ts` | User action | One-shot |
| `get_active_universe_path` | `universe/store.ts` | Startup | One-shot |
| `remove_universe_from_registry` | `universe/store.ts` | User action | One-shot |
| `rename_universe` | `universe/store.ts` | User action | One-shot |
| `add_child_universe` | `universe/store.ts` | User action | One-shot |
| `remove_child_universe` | `universe/store.ts`, `UniverseSetup.svelte` | User action | One-shot |
| `check_migration_needed` | `universe/store.ts` | Startup | One-shot |

### Settings & Persistence
| Command | Caller | Trigger | Notes |
|---|---|---|---|
| `save_universe_settings` | `store.ts`, `UniverseSetup.svelte` | Settings change | Debounced in caller |
| `save_universe_bookmarks` | `store.ts`, `UniverseSetup.svelte` | Bookmark change | One-shot |
| `save_universe_workspaces` | `store.ts`, `UniverseSetup.svelte` | Workspace change | One-shot |
| `save_universe_property_types` | `UniverseSetup.svelte` | Property type change | One-shot |

### Search & Index
| Command | Caller | Trigger | Notes |
|---|---|---|---|
| `collect_library_notes` | `+layout.svelte` | Startup / library open | One-shot, runs in background |
| `scan_library_links` | `store.ts` | Library open, graph open | One-shot |
| `scan_unlinked_mentions` | `store.ts` | Unlinked mentions panel | On demand |
| `scan_library_tags` | `store.ts` | Tag browser open | On demand |
| `scan_library_index` | `store.ts` | Index panel | On demand |
| `search_stars` | `store.ts` | Star search input | ⚠️ **Must be debounced ≥ 300ms by caller** |
| `search_by_property` | `store.ts` | Property search | On demand |
| `resolve_wikilink` | `store.ts` | Link resolution | On demand |
| `resolve_wikilink_cross_library` | `store.ts` | Cross-library link | On demand |
| `get_note_headings` | `store.ts` | Outline panel | On demand |
| `update_links_on_rename` | `store.ts` | File rename | One-shot |
| `execute_dataview_query` | `dataview/store.ts` | Dataview block render | On demand — never on every keystroke |
| `constellation_search_init` | `repairReport.ts` (`submitRepair`) | **The repair door** — Settings → Index → Repair index, and Repair now on the drift band | One-shot. Single-flight: returns a typed outcome (`started` / `queued` / `alreadyRunning` / `blocked`) — never fire-and-forget. Runs on its OWN connection and its own thread; never blocks the caller |
| `index_repair_status` | `JobProgressStrip.svelte` | Strip mount (recover-on-mount) | One-shot per mount. NOT polled — live updates arrive on the `index-repair:progress` event |
| `index_repair_cancel` | `JobProgressStrip.svelte` | User presses Cancel | One-shot; idempotent (an atomic store). The walk stops at the next note boundary; the derived-view tail still completes |
| `index_repair_last_report` | `SettingsModal.svelte` | Settings → Index opened | One-shot. In-memory for the current run of the app — absent after a restart |
| `index_drift_report` | `+layout.svelte` | Drift band mount | One-shot READ. The value is also pushed on the `index-drift:report` event; the command exists for a surface that mounts after it fired |
| `derived_heal_status` / `derived_heal_cancel` | `JobProgressStrip.svelte` | PJ-228 catch-up strip | Same contract as their `index_repair` counterparts |

**Events (Rust → frontend, preferred over polling):** `index-repair:progress` and
`derived-heal:progress` both carry `{ phase, total, completed, error }`; `index-repair:done`
carries the run's report; `index-drift:report` carries the re-derived drift counts. Progress is
throttled to one event per 25 notes — never one per item (Performance Rule 3).

**`cache_reconcile` is NOT in this table by design.** It was removed from the boot path
(2026-07-08) and is not a command the frontend should call: the boot reconcile schedules itself
after paint, and the user-reachable route is the repair door above.

### Notes & Daily Notes
| Command | Caller | Trigger | Notes |
|---|---|---|---|
| `get_daily_note_path` | `store.ts`, `+layout.svelte` | Daily note open | One-shot |
| `get_file_metadata` | `+layout.svelte` | File properties panel | On demand |
| `quick_capture` | `store.ts` | Quick capture shortcut | One-shot |

### UI / Windows
| Command | Caller | Trigger | Notes |
|---|---|---|---|
| `open_second_screen` | `secondScreen.ts` | User action | One-shot |
| `close_second_screen` | `secondScreen.ts`, `+layout.svelte` | User action / close | One-shot |
| `constellation_show_in_folder` | `+layout.svelte` | Context menu | One-shot |
| `open_path` | `+layout.svelte` | Context menu | One-shot |

---

## ⚠️ Hot-Path Watch List

These are commands that could accidentally end up on a hot path. Audit periodically:

| Command | Risk | Mitigation |
|---|---|---|
| `read_note` | Called in file-watch handler | OK — event-driven, not polling |
| `search_stars` | User types in search box | **Must** be debounced ≥ 300ms in the calling component |
| `execute_dataview_query` | Called on dataview block visible | Must not be called on every `docChanged` — only when block enters viewport |
| `save_universe_settings` | Settings panel reactive | Debounce or batch — never one call per slider tick |

---

## Future Commands (Planned)

| Command | Purpose | Notes |
|---|---|---|
| `full_text_search` | tantivy-backed FTS | Replace JS-side search entirely; debounce 300ms |
| `graph_layout_tick` | Rust force simulation frame | Push via event stream, not invoke per frame |
| `get_index_stats` | Performance overlay | Dev-mode only |

---

*Last updated: 2026-03-31*
