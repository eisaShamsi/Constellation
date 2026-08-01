# Verification inspection 2026-08-01 — the PRE-EXISTING register (triage feed)

Run `w484u96b7` — the per-build (diff-scoped) inspection over the 27 source files changed by
this session's remediation. 70/70 agents, all 14 scopes, **36 confirmed findings**.

They split two ways, and the split is what decides when each is fixed:

- **IN-DIFF (12) — fixed before the commit**, per WA#6 and the Charter's "never ship a known
  silent failure in the build that introduces it". These were regressions or asymmetries that
  *this session's own fixes* created. Recorded in `SESSION-LOG-2026-08-01.md` §6.
- **PRE-EXISTING (24) — this file.** Genuine defects the sweep surfaced that predate today's
  work. They are a NEW register, not a re-run of the 58 already closed. Same discipline as the
  2026-07-30 feed (`pj187-inspection-2026-07-30-remaining.md`): they are **filed, not
  deferred silently** — they need a Boss triage before they are scheduled, because between
  them they are another full remediation cycle and Stage-B is the standing priority.

**Dedupe note:** triage this against the 2026-07-30 feed (1 APP-KILLER, 10 HIGH, 13 MED,
3 LOW) and PJ-187's outstanding 19 M-cost register before scheduling. Several are likely the
same defect seen from a different scope.

---

## HIGH (13)

| # | Site | Defect |
|---|---|---|
| 03 | `bases.rs:406`, `tasks.rs:537`, `shape.rs:211` | `let _ = reindex_single_note(...)` after a gated frontmatter write, on the stated rationale "the watcher would catch it anyway" — **false** for a gated write (watcher-suppressed) and for boot (no re-walk of an indexed library). *(Fixed in-pass — see the session log; listed here because the rationale comment appears in more places.)* |
| 04 | `libraries.rs:1974` | `update_frontmatter_title` treats ANY non-inline-array `aliases:` value as a block-sequence opener, so a scalar `aliases: Foo` gets an indented `- "Old Title"` spliced after it → frontmatter that no longer parses. |
| 05 | `libraries.rs:1064`, `1873-1874`, `2016`; `store.ts:2473`, `2531` | YAML double-quoted scalars escape `"` but never the BACKSLASH, so a title containing `\` is written as an escape sequence. The correct escaper already exists in-repo: `canonical.rs:411 escape_yaml_string` (escapes `\` first, then `"`). |
| 07 | `search.rs:10129` | `reconcile_filesystem` — the authoritative self-heal every write-time maintenance path names as its safety net — has **no user-reachable trigger**, so any missed recompute is permanent rather than eventually-consistent. |
| 09 | `sources/mod.rs:501`, `1113` | `rewrite_frontmatter_sources` / `_content_type` skip the replaced block using ONLY `yaml_lines::is_seq_item`, omitting the `is_comment` + `is_block_value_line` pair every sibling writer uses → a comment or continuation line ends the skip early and orphans the remaining items under the PREVIOUS key. |
| 10 | `sources/mod.rs:286` | Any value in a note's `sources:` / `content_type:` that is not a taxonomy ID is **silently erased** from the `.md` the first time the note is accepted: `push_source` drops it during extraction, the PJ-091 merge never sees it, and the rewriter regenerates from validated IDs only. |
| 11 | `universe.rs:118` | `load_registry` collapses a READ failure (and a parse failure) of `universes.json` into an empty registry, and four write paths then atomically save that emptiness back — **silently deleting every registered Universe**. `libraries.json` got the "absent is a fact, unreadable is an unknown" fix (`try_load_libraries`); the universe registry never did. |
| 12 | `universe.rs:903` | `set_active_universe` holds the `UniverseState.active_path` mutex (never dropped) across `invalidate_search_state`, which blocks on `SearchState.db.lock()`. `active_path` is the mutex every path resolution goes through, so **the whole app freezes** for as long as the switch waits on the DB writer lock. |
| 13 | `NotePane.svelte:340` | `doSave()` clears the view-level `dirty` flag BEFORE handing text to the host, and `NoteEditor.handleSave` drops the request when a previous write is in flight (`if (saving) return`) — nothing re-arms. |
| 16 | `+layout.svelte:6918` | `handleRenameComplete` treats any non-`.md` path as a directory, so renaming a `.base` file builds a path with NO extension — the Base is destroyed as far as the app is concerned. |
| 17 | `+layout.svelte:6940` | The rename-collision **Overwrite** trashes the existing note with `moveToTrash`, which (unlike `deleteWithSetting`) leaves that note's tab and live model resident — the next debounced save recreates the file the user just replaced. |

## MED (8)

| # | Site | Defect |
|---|---|---|
| 21 | `link_types.rs:516` | `read_deltas` maps a read/parse FAILURE of `link-types.json` to `Vec::new()` — indistinguishable from "no custom types" — and the Links editor's save then whole-file-replaces from that emptied view, destroying every custom link type and seed override. |
| 22 | `link_types.rs:540` | `save_universe_link_types` writes with a plain non-atomic `std::fs::write` (truncate-then-write, no fsync, no temp+rename) — the only persisted-JSON store not using `universe::atomic_write`. |
| 24 | `ConfidencePicker.svelte:62` | A failed link-confidence write and a failed link-archive write are both swallowed by `catch { /* ignore */ }` after the popover has closed — and confidence/archived live ONLY in `search.db`. |
| 25 / 33 | `PropertyEditor.svelte:1034` | The 800 ms debounce writes `tab.content = buildFullContent(...)` into the live store tab BEFORE `commitAndSave`'s `rowsBelongToTarget` refusal runs, so on an in-place navigation the OUTGOING note's frontmatter is spliced onto the INCOMING note's body in the incoming tab's `content`. |
| 26 | `PropertyEditor.svelte:604` | `onDestroy` clears the pending 800 ms timer then returns early when reseeding/cascading — a typed-but-uncommitted property edit is discarded with no model write, no write-ahead stash, no surface. |
| 27 | `PropertyEditor.svelte:466` | The seed effect applies the registry OVER the parsed type, and the taxonomy branch precedes the `nested-map` branch — both re-open the PJ-136 "invitation": a read-only nested block renders editable and regains its delete button. `composeFrontmatter`'s `immutableBlockKeys` still refuses the write (bytes survive), but the refusal is BELOW the model, so nothing reports it. |
| 28 | `UniverseSetup.svelte:273` | The legacy-migration path deletes every `constellation-*` localStorage key unconditionally after four writes whose errors are swallowed — and one of those writes (`save_universe_bookmarks`) **is not a registered Tauri command at all**, so it fails 100% of the time. |
| 29 | `store.ts:2955` | `openNoteTab` invokes `ensure_cid_cn_cmd` — a gated, watcher-suppressed, un-announced write into the note's frontmatter — with no `displayOnlyWindow` guard, so the read-only second screen writes to the user's `.md` and the main window is never told. |
| 31 | `+layout.svelte:9440` | The right-sidebar TasksPanel checkbox swallows a failed write into `console.error` only, leaving the box visually ticked while the `.md` is unchanged — the exact defect PJ-187 fixed in its sibling `GlobalTasksView`. |

## LOW (3)

| # | Site | Defect |
|---|---|---|
| 34 | `SecondScreenPage.svelte:132` | The SS subscribes to `onNoteMutation` with only `onAnyChange`; it never wires `onRenamed`/`onMoved`/`onDeleted` to repath or close its OWN tabs, so after a main-window rename it holds tabs at dead paths — and reports them into the persisted workspace snapshot. |
| 35 | `lens/BaseTab.svelte:256` | `updateNoteProperty` resolved normally when its pre-write flush aborted, so `commitEdit`'s catch never fired and the cell painted as saved. *(Fixed in-pass — see the session log.)* |
| 36 | `+layout.svelte:780` | `secondScreenOpen` is initialised `false` at main-window mount and never resynced from the authoritative `is_second_screen_open`, so after any main-window reload the still-visible second screen goes permanently blind to every sync gated on that flag. |

---

### Standing observations from this run

- **The two APP-KILLERs it found were both of the same species as the ones it was checking.**
  One was mine, introduced hours earlier (`adoptExternalChangeIntoTabs` half-normalised); one
  was the last unswept site of a class fixed five separate times before
  (`remove_frontmatter_contains_item`). A class is not closed until *every* site is swept —
  the grep is the fix, not the patch.
- **The POSIX-path blind spot is systemic.** The entire watcher-adopt suite drove `/n.md`
  paths, where `normPath` is the identity function, so a Windows-only total failure of the
  external-change subsystem kept the suite green. Any test that exercises a path comparison
  must use a backslash path — Windows is the platform Constellation ships on today.
- `reconcile_filesystem` is cited by many fixes as "the authoritative self-heal" while having
  no user-reachable trigger (#07). Several MED findings' severity rests on that assumption
  being true. Worth resolving early, because it silently upgrades a family of "eventually
  consistent" defects into "permanent".
