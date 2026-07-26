# Session Log — 2026-07-25

Continuation of the MIG-103 / safety-hardening arc. Two commits landed earlier in
the day (`a740a20f` destination propose+show; `bc8091e3` search.db un-deletable +
archive-reversal + scan de-freeze + sidebar dedup — logged in `SESSION-LOG-2026-07-24-B.md`).
This log covers the **third** commit (**`332f566d`**, pushed to `origin/main`): the
**PJ-140 Rust remediation + the Whole-Ecosystem file-tree fix + the library-icon system**,
plus two new top-principal laws.

**Verification (all green, run at close):** Rust **1161 passed / 0 failed** (10 ignored;
+9 new tests) · svelte-check **0 errors** (268 pre-existing unused-CSS warnings) ·
vitest **616/616**. Release binary rebuilt + re-embedded (18:32), Boss-tested.

---

## Phase A — The nested-library bug → the Whole-Ecosystem Fix Law

**Boss report:** *"the library 'Creating new library' is under 'Eisa Cognitive Knowledge'.
Yet the file tree shows it isn't. Also 'Eisa Test' exists in the file tree, but is missing
if I want to move a note to it."*

The `bc8091e3` sidebar-dedup fix touched only `read_library_tree` (the sidebar). The Boss
immediately hit the **Move picker** — a *parallel* walker of the same file-tree concern —
still inconsistent. **Boss dictated THE WHOLE-ECOSYSTEM FIX LAW** (now a top-principal in
CLAUDE.md + `feedback_whole_ecosystem_fix_law.md`):

> "You should be thorough when fixing anything… tackle everything related to the file
> tree/explorer in every function or aspect within the Constellation ecosystem. Consider
> this a law." — Eisa, 2026-07-25

**Remediation (one concern — "enumerate the file tree, honoring Library ≠ Folder" — every
surface):**
- **Two shared helpers** (`libraries.rs`): `nested_library_paths(libs, self_path)` (the
  exclude set — every registered library except the one at `self_path`) and
  `is_nested_library(dir, exclude)` (the one boundary check every walker uses). A shared
  helper so the surfaces cannot drift again.
- **Every walker now honors the exclude set** — `libraries.rs`: `read_library_tree`,
  `list_universe_folders`/`collect_folders` (the Move picker — the Boss-found gap),
  `scan_library_links`, `scan_library_tags`, `notes_by_tag`, `scan_note_stages`,
  `scan_library_index`; `tasks.rs`: `scan_library_tasks`, `scan_library_note_dates`;
  `search.rs`: `index_library_recursive` + `reconcile_filesystem`.
- **Per-file library attribution** replaces fixed-name stamping in the indexer
  (`index_library_recursive`, `reindex_library`, `move_item`, folder-rename) via
  longest-root-wins `library_name_for_path`. Root's walk no longer stamps nested-library
  notes with `universe_notes` (the "Eisa Test looks empty / 0 notes" cause).
- **Frontend resolver sweep** (`+layout.svelte`): new `libraryForPath(path)` (longest-root-
  wins) replaces the old first-match `$libraryStats.find(l => path.startsWith(l.path))` at
  **33 sites**. First-match always returned the root (whose path prefixes every nested
  library). Closes **PJ-141**.
- **Move-picker filter fix** (`+layout`/`MoveDialog`): a note living directly in a library's
  root no longer removes the whole library from the picker (the current-folder no-op filter
  now exempts library roots), so the library stays visible as a destination anchor.

**Self-heal backfill** (`library_attribution_backfill.rs`, new): a one-off column re-write
that corrects `note_meta.library_name` rows a *prior* reconcile mis-attributed — so a nested
library showing 0 notes self-corrects on boot. Versioned (`LIB_ATTR_VERSION=1`, `schema_versions`
`module='lib_attr'`), batched (500 rows / 40 ms sleep, keyset-paginated), scheduled off the
main thread after paint (`maybe_schedule`, wired in `ensure_search_db_ready` after the two
existing backfills), **stamps only after a completeness check** (an interrupted run re-runs
next boot). Pure Write-Time-Derivation shape — no note re-tokenized, `body_text`/FTS untouched.

**Boss test:** PASS — libraries show once in the tree, "Eisa Test" present as a Move destination.

---

## Phase B — PJ-140 Rust remediation ("everything fixable now")

The `libraries.rs +538` batch is a **PJ-140 Rust remediation**, not only the file-tree fix.
Per-build safety inspection (`wf_ae5d4d18`, whole-app — PJ-124 struck again) verdict:
**ZERO new silent-failures introduced.** **14 numbered findings + 5 unnumbered durability
findings closed** this batch:

**Path-integrity cluster** (the shared `migrate_note_db_paths` — migrates `note_meta`,
`note_links.source_path`, `note_aliases`, `note_embeddings`, and `review_schedule` (gated on
`is_stamped`) old→new, pre-deleting stale PK-on-path rows so a moved note can't be silently
orphaned; deliberately NOT `target_path`, the ~11 s dead-scan):
- **#2** — folder rename now cascades the DB path-migration + reindex to every descendant
  (was skipped entirely → descendants kept dead paths all session; watcher suppressed).
- **#16** — move MIGRATES rows instead of delete+reinsert (delete leg dropped review history
  + orphaned aliases/embeddings; reindex wrote a fresh default schedule — earned review
  state silently reset on every move).
- **#3** — folder delete snapshots descendants *before* the delete and purges each (exact-path
  DELETE matched zero rows for a folder path → deleting a folder purged nothing from the index).
- **#17** — `reindex_delete_note` also purges `note_aliases` + `note_embeddings` (else a future
  note at the same path inherited the dead note's alias→cid binding + orphan embedding).
- **#18** — `gate_rename` dest-exists guard UNDER the lock, returns `Err` (else a concurrent
  create at `new` between the outside-lock `exists()` check and `fs::rename` was silently replaced).

**Freeze / leak cluster:**
- **#27/#28** — `index_note` target-cid query seeks the write-time Unicode-folded `name_lower`
  column instead of ASCII-only `LOWER()` (non-ASCII titles never matched → cid NULL forever;
  and the predicate full-scanned `note_meta` dragging inline `body_text` per link per save —
  the PJ-066 22 s landmine).
- **#42** — `constellation_link_backfill_confidence` → `async` (two full-table `note_links`
  UPDATEs under the writer mutex, sync = UI freeze on the WebView2 dispatch thread).
- **#60** — `read_term_mentions` + `read_cooccurring_terms` → `async` (each re-tokenizes
  thousands of bodies per Index-panel expand on the dispatch thread).
- **#57** — WAL checkpoint daemon: newest-wins via `WAL_DAEMON_GENERATION` (each universe
  switch leaked an immortal daemon re-checkpointing a stale universe's `search.db`).
- **#61** — `perf_trace` bounded at `MAX_TRACE_ENTRIES=4096` (pushed on every IPC dispatch,
  no clear path → unbounded session-long memory growth).
- **#43** — `copy_dir_recursive` skips symlinks/junctions (a junction cycle recursed unboundedly).

**Silent-failure cluster:**
- **#53** — `cece_resolve_disambiguation` propagates DB `Err` via `?` instead of collapsing to
  `None` (a locked/corrupt DB read as "no prior card" → wrongly discarded a still-Split axis).
- **#33** — `set_active_universe` nested-universe consolidation logs a failed `.constellation`
  move and gates the registry repoint + dir removal on the move landing (else the universe was
  repointed to a config-less directory — silent breakage).

**Durability (unnumbered, atomic_write = fsync-before-rename):**
- `save_libraries` → `atomic_write` (`libraries.json` — power loss could land a zero-length file).
- `load_pulse_data` — a corrupt `review-pulse.json` is backed aside to `.corrupt-<ts>.json`
  before starting fresh (was a SILENT `default()` fallthrough discarding ALL review history);
  read errors treated as transient, left untouched. `save_pulse_data` → `atomic_write`.
- `read_universe_collections` legacy adoption → `atomic_write` for `collections.json`.

**New Rust test modules:** `tests_pj140_path_migrate`, `tests_nested_library_helpers`,
`tests_pulse_durability` (the +9 tests).

---

## Phase C — The library-icon system (planet → building)

**Boss:** *"change the library icon from a planet to a library… provide more than one option."*
Chose **D** (a library building — pediment + columns). Rulings captured:
- **Root** (Universe root) renders **NO icon** — "there won't be anything under the root, just
  cUniverse(s) and Libraries" (forward-compatible with MIG-105).
- **cUniverse** keeps a **planet-and-orbit** glyph (a whole other Universe, not a library).
- **Folders** unchanged.

**One shared component** (`LibraryIcon.svelte`, new — kind `library`/`cuniverse`/`root`; `size`
number|CSS|omitted; `strokeWidth`; `color`) so the mark can never drift. Wired to **every**
surface: sidebar (own + under-cUniverse, tinted by library colour, `var(--ft-library-icon-size)`),
the New Library toolbar button, MoveDialog (bold colour-tinted library rows), DashboardView,
LibraryPicker, StyleSetter tree preview.

**Follow-up fixes (each Boss-found, each fixed in-pass):**
- **New "Icon size" control** — `StyleSetter → Library → Icon size` (`--ft-library-icon-size`),
  matching the existing dock/sidebar/layout icon-size controls.
- **Toolbar icon "dead" / not size-controlled** — root cause was Svelte scoped-CSS: `.tb-btn svg`
  didn't reach the foreign component's SVG. Fixed to `.tb-btn :global(svg)` (verified in the
  built CSS), and `LibraryIcon` omits an inline width when `size` is omitted so the ambient
  `--sidebar-icon-size` rule can size it.
- **Move dialog icons match library colours** — each Move entry carries `iconKind` + `color`.
- **Glyph enlargement** ("enlarge the library icon by x0.5" → "the library icon only") — the
  building path was redrawn to fill more of the 24×24 box (renders ~1.4× larger at any size)
  WITHOUT changing any caller's `size` or the Style Setter control. My first pass over-reached
  (bumped the size props + control default across surfaces); reverted per the correction, kept
  only the glyph change. **Boss: Pass. Thanks!**

---

## Two new top-principal LAWS (CLAUDE.md + memory)

1. **The Whole-Ecosystem Fix Law** — fix the whole concern across the entire ecosystem, every
   surface, in one pass; shared helper so they can't drift. Origin: the sidebar-only fix that
   left the Move picker inconsistent. (`feedback_whole_ecosystem_fix_law.md`.)
2. **No Guessing — Investigate to Build Awareness** — never guess/theorize to build awareness;
   read the file, trace the code, query real data; state findings with their source. Twin of
   *Don't Make Things Up*, applied to reasoning. Origin: I theorized why "Eisa Test" was missing
   from the Move picker instead of reading `libraries.json` + the builder. (`feedback_no_guessing_investigate.md`.)

---

## Phase D — The PJ-140 [0] HIGH content-loss fix (Backlinks "link it")

**Boss: "Fix the backlinks HIGH next."** The `wf_ae5d4d18` HIGH: `BacklinksPanel.linkMention`
turned a plain-text mention into a `[[wikilink]]` via a raw `invoke('write_note')` — three
silent failure modes: **open-model overwrite** (read disk, wrote behind the open model, whose
next autosave erased the link + any unsaved edits), **false success** (a `catch {}` swallowed a
failed write), **index divergence** (no reindex → the new backlink invisible until boot).

**The fix (Solve-the-Class — single content ownership):** a new shared store primitive
`linkMentionInNote(mentionPath, targetName)` on the proven `toggleTaskReconciled` body-edit shape —
gate (`markCascading`) → **flush the open model to disk first, or ABORT rather than clobber** →
mutate disk → model adopts the mutated disk (remount) → reindex. Longest-root-wins library
resolution (nested-safe); body-scoped regex (never corrupts frontmatter); **throws** on a genuine
write failure, surfaced via the existing save-health banner (`reportSaveFailure`).

**Whole-Ecosystem (the no-reindex half):** the four sibling raw-write-then-no-reindex sites all got
a `reindexNote`: `template_create` + `daily_template` (`+layout`), `ExpressionForge`, `SenseMakingCanvas`.

**The 5-10s delay (Boss-flagged, investigated not guessed):** NOT the write, NOT the reindex
(confirmed O(changed-edges), no re-embed). Root cause: nothing bumped `perNoteRefreshNonce`, so the
backlink appeared only when an *incidental* trigger re-ran the panel effect seconds later. Fix:
`linkMentionInNote` awaits the (fast) reindex → BacklinksPanel calls a new `onLinked` callback →
`applyMentionLinkedLocally` bumps the nonce → both the Backlinks and Unlinked-mentions effects
re-fetch at once. **Boss re-test: "almost instant."**

**Reproduce-First:** `tests/pj-140/backlinksLinkMention.test.ts` (7 cases) drives the real primitive
against a disk-backed mocked IPC bridge — incl. **T6** (open note, dirty model, locked file →
ABORTS with `false`, no clobber — the HIGH's crux) and T7 (open clean → safe link + reindex).

**Verification:** svelte-check **0** · vitest **623/623** (+7). Per-build safety inspection
(`wf_45def36d`, whole-app — PJ-124 again): **0 new in-diff findings** (8 pre-existing → PJ-140).
`/simplify`: one behavior-identical cleanup applied (`before` reuse); two follow-ups filed
(PJ-147 resolver consolidation, PJ-148 `createNote(initialBody)` for the two export sites); the
flush-gate-envelope extraction deferred to its 3rd occurrence (LL-014). **Boss-validated
end-to-end: Stage 1 · Re-test A (timing) · Stage 2 (open-note content-integrity) all PASS.**

*(Note: the `before`-reuse cleanup landed AFTER the Boss test; it is provably behavior-identical
— reuse an already-computed substring — so it cannot change the validated behavior; not re-tested.)*

---

## Open / carried forward

- **PJ-140** — ~37 findings remain (the [0] HIGH + the two index-divergence findings closed this
  phase). The editor-lifecycle cluster (NoteEditor handleSave re-entrancy drop, etc.) is its own
  migration; the rest await the Boss's sequencing ruling.
- **MIG-105 Architect** (root library vs flat Universe — Boss-directed, ready to run) — the
  data-model root cause behind the whole resolver class this session patched at the surface.
- **MIG-104 remaining questions** (durable earned-link home; location + re-type settled).
- PJ-142 bulk-accept end-to-end (needs a Tauri mock harness) · PJ-143 (`target_path` empty) ·
  PJ-144 (per-note scan reload) · PJ-124 (inspection ignores `args.files` — struck again).

**Doc-drift noted:** the translated help dirs (`help.ar`, `help.de`, …) are a partial subset,
not a 1:1 mirror of `help.uConstellation.World` (no "Libraries" topic in most). The library-icon
note was added to the English canonical help + User Manual; the translated subset was not
expanded for this minor visual note. Full 14-language help sync is a separate documentation job.
