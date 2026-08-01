# SESSION LOG — 2026-08-01

Continues 2026-07-30 (MIG-108 built through Slice 8; Stage-A Boss-passed; the Stage-B gate
written into `docs/HANDOVER-2026-07-30-mig108-gate.md`).

**The gate, in the Boss's order:** (1) the COMPLETE whole-app safety inspection, every
confirmed finding fixed; (2) the MIG-108 Phase-4 audit, likewise; (3) only then Slice 7 /
Stage-B against the real universe. Nothing touches
`E:\Constellation Universes\Eisa Cognitive Knowledge` until (1) and (2) are green.

---

## §1 — Gate step 1: the complete whole-app safety inspection

Run `wjqj3csw5` finished the sweep that the weekly limit had truncated on 2026-07-31
(that partial run reached 3 of 14 scopes; 11 of 28 agents died). The complete run:
**78/78 agents, all 14 scopes, 58 confirmed findings** — 2 APP-KILLER, 16 HIGH, 26 MED,
14 LOW.

**All 58 are fixed.** No finding was deferred, logged-and-shipped, or parked (WA#6). The
work was split across disjoint files so it could run in parallel; every fix follows a named
in-repo sibling pattern rather than inventing a new one.

### §1.1 The two APP-KILLERs

**`canonical.rs:279` — `merge_frontmatter` matched keys on the TRIMMED line.** Nested-map
children and block-scalar prose were treated as root keys during import/canonicalisation: a
nested `kind:` was REPLACED with a column-0 line (destroying the map and the user's value), a
nested `aliases:` opened the alias-injection block inside a foreign map, and a nested
`cid_cn:` set `has_cid` and silently suppressed minting the note's real identity. This is the
exact class the 2026-07-21 inspection fixed in `update_frontmatter_title` via
`is_top_level_key_line` — never swept into its sibling.
Fixed by applying that same guard to **every** key matcher in the file (sweep: 8 sites in
`merge_frontmatter`, 3 in `remove_canonical_fields`, plus `migrate_cid_to_cid_cn`,
`extract_created_from_frontmatter`, `extract_fm_field`). 6 new tests.

**`+layout.svelte:3438` — the `wasRecentlyWritten` filter discarded the Rust re-base
announce.** `announce_frontmatter_write` emits `library-changed` with `libraryId: ""` to tell
an open note to re-base after a gated frontmatter write. That event was passing through the
frontend's self-echo filter, so whenever the same note had been frontend-saved within ~2s the
announce was dropped — defeating the shipped 2026-07-22/24 APP-KILLER fix at *every* accept
seam (per-card Accept, disambiguation, bulk Approve-All). Accepting during typing erased the
accepted sources.
Fixed: `libraryId === ''` is now recognised as an ANNOUNCE — it bypasses `wasRecentlyWritten`
(that filter exists for frontend echoes, and an accept within 2s of a save is precisely when
the re-base matters most) and adopts IMMEDIATELY rather than after the 300ms flush debounce.

### §1.2 The Rust half (findings 3–11, 19–27, 45–50)

- **`libraries.rs` (13 findings).** Folder rename/move/delete now retarget or de-register a
  nested library's `libraries.json` entry (`retarget_registered_libraries`) instead of
  leaving it pointing at a dead path; `update_links_recursive` gained the symlink/junction
  guard every sibling walker already had; `move_item`'s DB tail is detached like
  `rename_item`'s (the §B2-4 stall class); the cascade reindex tail resolves the library name
  per file; `extract_frontmatter_title` got top-level-key discipline; `move_item`,
  `delete_path` and `rename_item`'s folder branch now `ensure_search_db_ready`-or-refuse
  **before** the fs op (they used to silently skip the index cascade during the cold-boot
  window); `collect_library_notes` is `(async)` and `extract_frontmatter_title_quick` finally
  honours its documented 1KB contract; the cross-device trash fallback claims its name
  exclusively; `resolve_embed_image` is `(async)`.
- **`search.rs` + siblings (7 findings).** Archive/unarchive now recompute the target's
  incoming aggregates and both notes' sky rows (nothing healed them — the coarse signature
  guaranteed no future save would); `incoming_signature` widened to include link TYPE, so
  re-typing `[[supports::B]]` → `[[contradicts::B]]` no longer recomputes nothing;
  `reindex_delete_note` propagates its four de-index writes and returns `Err` instead of
  `Ok` when the DB is absent (false-success); `reconcile_filesystem` persists a
  trigger-dropped marker so a crash mid-walk self-heals at the next boot; the incoming
  backfill gained the vocabulary-fingerprint gate its outgoing twin has;
  `compute_note_origins` is `(async)`.
- **The ambient-key family (`universe.rs`, `review.rs`).** `save_universe_settings` /
  `_workspaces` / `_collections` / `_property_types` and `mark_reviewed` / `snooze_note` /
  `dismiss_note` all resolved the target directory *ambiently at execution time*, so a write
  landing after a universe switch wrote universe A's data over universe B's file. All seven
  now take an optional explicit `universe_root`, mirroring the `session.json` precedent that
  was given this treatment for this exact reason. **Frontend wired to match**: the debounced
  settings (300ms) and property-type (500ms) writers capture the root when the debounce
  ARMS, and both flush on universe switch and on app close (they had no close flush at all —
  a change made inside the debounce window silently reverted next boot).

### §1.3 The frontend half (findings 12, 15–18, 28–44, 51–58)

- **The write-ahead net (finding 17).** `resolveNoteContent` destroyed the net on every
  manual open of a cid-less entry — and a cid-less entry is exactly what the app's own
  failed-save flow produces (templates are cid-exempt by design; for a regular note the cid
  injection fails under the SAME lock that made the save fail). See §3 — my first fix here
  was wrong and the tests caught it.
- **`discardFailedSave`** was the only one of nine `reloadTabsFromDisk` callers without the
  hazard-#6 `markCascading`/`markReseeding` bracket — the outgoing editor's teardown flush
  could re-poison the re-seeded model with the very content the user chose to discard.
- **Path normalisation.** `saveRecoveredCopy` built a mixed-separator path while every
  one-path-one-tab dedup gate compared raw strings — a later open via the canonical Rust form
  minted a SECOND tab and a SECOND model for one file. Fixed at the producer (join with the
  original separator) *and* at all three dedup gates + `adoptExternalChangeIntoTabs`, which
  had the same blind spot (an unmatched tab neither adopted an external edit nor reached the
  conflict-sidecar branch).
- **The swallowed-reindex class (30/51)** fixed at the choke point: `reindexNote` now does one
  bounded retry and surfaces a final failure (`indexHealthError`); every `.catch(() => {})`
  reindex call site in `store.ts`, `NoteEditor.svelte` and `flushAllForAppClose` routes
  through it. `flushAllTabsInLibrary` now uses `navFlushEnv` so a flushed-but-not-rewritten
  note reaches the index at all.
- **`moveItem` (36)** now aborts when the pre-move flush was not durable (it ignored the
  `SaveOutcome`, which never throws) and re-points the `saveHealth` entry to the new path.
- **The flush→RMW→reload recipe (38)** — five callers ignored the PJ-174 dirty-refusal and
  passed no conflict handler; all now pass `reportExternalConflict`.
- **`parseFrontmatter` flow lists (39)** split on raw commas, so `tags: [alpha, "beta, gamma"]`
  projected as three items — and that projection became the write the moment the key was
  edited. Replaced with a quote-aware scanner; 4 new tests.
- **Second screen (32/52/53)** now adopts external watcher edits (it only reloaded the flat
  note list), has stale-result generation tokens on its adopt reads, and disposes the previous
  universe's tabs/models on a universe switch.
- **Cascade failures (43)** — the Rust walker records per-file wikilink-rewrite failures in
  `CascadeResult.failed`; no frontend consumer had ever read them. Now surfaced, named, and
  journaled.
- **Unlinked mentions (44)** had no stale-result guard while its sibling effect did; two
  overlapping scans could leave note A's rows rendered under note B, and the "link it" button
  pairs a row with the LIVE active note — a click wrote a wikilink for the wrong target.
- **The surfaces that surfaced nothing (40/41/42, and two pre-existing).** Findings 40–42
  asked for the settings/workspaces write failures to stop being console-only. Replacing a
  `console.error` with a store is only a fix if something RENDERS the store — and a check
  showed that **none of the four** store-health conditions was read by any component:
  `collectionsError` and `workspacesError` (both pre-existing, from PJ-187) as well as the
  two added today. All four now feed one persistent, non-dismissible `.store-err` bar at the
  top of the app shell (worst-first: a refused write outranks a stale index). Not dismissible
  on purpose — unlike an action failure, the condition is still true after you look away, and
  every further change is also being lost. 4 new `storeHealth.*` keys × 15.
- Plus: `livePreview`'s two unbounded caches are now bounded LRUs; `yamlDoc`'s scalar
  fast-path no longer swallows `nested-object-list` (the user's structured rows never reached
  disk); `SenseMakingCanvas` surfaces write failures and flushes on destroy/canvas-switch;
  `ReviewerView`'s `catch {}` on priority (the PJ-187 defect left un-fixed in the sibling);
  tag/batch-move/traversal failures surfaced.

### §1.4 One latent APP-KILLER, handed over separately

`OrgChart.svelte` `onDrop` called raw `invoke('move_item')` instead of the store's `moveItem`
wrapper — skipping the flush-before-move envelope, the model repath and the aux-state
migration. **Refuted as unreachable and independently re-verified**: the drag-drop tree
renders only under `sidebarMode === 'skyview'`, and nothing anywhere assigns that mode
(`+layout.svelte:4314` only ever *exits* it). Fixed anyway — swapped to the wrapper, so it is
correct if the branch is ever re-wired. Whole-Ecosystem grep confirms there is now exactly
one `move_item` caller: the wrapper.

---

## §2 — Gate step 2: the MIG-108 Phase-4 audit

31 findings, 3 BLOCKERs — all in the crash-resume path, and all sharing one root: **the
engine assumed the crash-resume window is QUIET, and it is not.** Boot healers, reconcile and
watchers all run before the user can click Resume.

- **Adoption never GUESSES completeness.** A same-volume rename is atomic, so *source gone +
  dest present* proves done; a copy proves nothing by existence (a crash mid-copy leaves a
  partial directory that `is_dir()` happily accepts). Journal entries gained `started` /
  `copied` sub-states: a destination we never started is a hard collision (never deleted); our
  own partial is deleted and redone, count-verified.
- **Destination-prefix purge.** Reconcile can re-adopt moved files as fresh rows at their NEW
  paths inside the crash window; those rows collided with the cascade's UPDATEs and failed the
  rewrite deterministically on every retry. The purge deletes that junk (recomputable by
  construction) and the EARNED rows win.
- **In-transaction baseline captured AFTER the purge** — its own new test caught the ordering
  flaw ("aggregates diverged").
- **`mig108_restore`** — the promised rollback, now shipped and tested (valid up to
  `Moved`/`VerifyFailed`; after `DbRewritten` finishing is the only safe direction).
- **The dialog can no longer wedge the app**: "Not now" on the resume card, a "Put everything
  back" button when restorable, a corrupt-journal card (it was console-only — invisible in
  release), thaw-on-failure (the freeze envelope stopped every watcher; a failed run left them
  off), and a re-probe so a failed first run switches to resume mode instead of stranding the
  user at a Unify button that can only ever error.
- **Boot gate.** While an unfinished journal exists the watcher/reindex/session fan-out is
  held. It **waits** rather than returns — see §3.
- **The close guard.** The PJ-103 handshake proceeds after 5s no matter what; mid-engine that
  kills the process between two directory moves. The close is now REFUSED while the engine
  holds the world open, and the running screen says so. (A hung engine is still killable from
  the OS — that path is the journaled crash resume already handles.)
- **Backup hygiene**: a second run sets the previous backup aside as `mig108-backup.prev`
  instead of silently overwriting it (the summary promises it is kept); `mig108-backup*` and
  `mig108-journal.json` added to the `.constellation/.gitignore` contract — the journal is
  single-machine crash state and would surface a bogus resume dialog on another device.
- **`UniverseSetup`'s Add-Library dead-end** fixed: an external pick there now routes through
  the same `BringInDialog` + `bringInLibrary` the main window uses.

Rust: 22 mig108 tests + 4 new crash-window proofs.

---

## §3 — The fix that introduced its own bug (and how it was caught)

My first fix for finding 17 kept the write-ahead net on *every* rejection except a stale
snapshot or an empty-body resurrection. `npx vitest run` then failed
`tests/pj-181/…CONTROL — a net entry whose cid differs is still rejected in favour of disk`.

The failure was mine, not pre-existing (a subagent had reported it as pre-existing, having
taken its baseline *after* my edit was already in the worktree — worth noting as a hazard of
parallel work). **"Not proven" and "DISPROVEN" are not the same fact:**

- **DISPROVEN** (both cids readable and different) — the entry belongs to ANOTHER note that
  used to live at this path. Keeping it would let a later resolve restore a different note's
  body into this file: cross-note contamination.
- **merely UNPROVEN** (either cid missing) — cannot be attributed or disowned, and this is
  exactly what a failed save under a file lock produces. Clearing it destroys what may be the
  only copy of the user's unsaved work.

The net is now cleared only for a stale snapshot, an empty-body resurrection, or a disproven
identity. Both behaviours are covered by tests.

Same shape, caught by the same reflex: the boot gate above originally `return`ed, which would
have left "Not now" with no watchers and no restored tabs until a restart — a worse and more
visible failure than the dirt the gate avoids. It now awaits a release that every dialog exit
resolves.

---

## §4 — Gates

| Gate | Result |
|---|---|
| `cargo test --lib` | **1318 → see §7 for the post-verification number**, 0 failed |
| `npx svelte-check --threshold error` | **0 ERRORS** (268 pre-existing warnings, mostly unused CSS) |
| `npx vitest run` (non-Sight) | **722 passed / 67 files, 0 failed** |
| `npx vitest run tests/sight-v6/` (PJ-172 serial lane) | **84 passed / 5 files, 0 failed** |
| frontend `npm run build` | clean; today's new strings verified present in `build/` |
| i18n | 16 new keys × 15 locales, verified present in all 15 |

Two notes on reading these honestly:

- A first `cargo test` attempt failed with the known transient Windows LNK1104; clean on retry.
- Running the FULL vitest suite in one pass reports 1–6 failures that vary between runs — all
  of them Sight v6 **wall-clock budget** assertions (`≤32 ms`, `≤16 ms`) missing under
  concurrent load. That is the documented PJ-172 split, which is why Sight has its own serial
  lane; both lanes are green when run as designed, and nothing in this session touches Sight.
  Recorded rather than hidden: a suite that only passes when sharded is a real (if known)
  weakness in the harness.

---

## §6 — Gate step 1b: the per-build verification inspection — and it was right to run

Run `w484u96b7`, diff-scoped over the 27 source files this session changed. 70/70 agents, all
14 scopes, **36 confirmed** — **2 APP-KILLER, 15 HIGH, 14 MED, 5 LOW**.

**Both APP-KILLERs were of the same species as the ones the session had just been fixing.**

### §6.1 `store.ts:1186` — mine, introduced hours earlier

Fixing finding 37 I normalised `adoptExternalChangeIntoTabs`'s key-BUILDING half
(`byPath.set(normPath(...))`, `openPaths`, `targets`) and left all three consumption sites
raw: `byPath.get(t.path)` at 1167, 1186, 1215. On Windows every tab path is a backslash
string from `read_dir_recursive`, and `normPath` rewrites `\` → `/` — so the lookup missed for
**every open tab** and the entire external-adopt/conflict arbitration silently no-opped: no
adopt, no `.conflict` sidecar, no banner, and the next keystroke's save overwrote the external
edit. All three ingress paths funnel through it (watcher flush, the Rust re-base announce,
second-screen saves), so it also killed the announce fix from §1.1 of this same session.

**All three gates passed it.** The entire watcher-adopt suite drives POSIX `/n.md` paths,
where `normPath` is the identity function — so a Windows-only total failure of the subsystem
kept 803 tests green. Fixed, and `tests/mig-076/watcherAdoptStore.test.ts` gained three
backslash-path cases. **RED-proven**: reverting the arbitration lookup fails exactly those 3
and leaves the 11 POSIX cases passing — the blind spot, demonstrated.

### §6.2 `libraries.rs:2111` — the last unswept site of a five-times-fixed class

`remove_frontmatter_contains_item` matched `contains:` on the TRIMMED line. A nested
`contains:` child under a user's own map was consumed as the note's structural list and
**deleted**; an indented inline array was re-emitted at column 0 as a DUPLICATE root key,
which makes the document unparseable — after which `composeFrontmatter`'s H1 branch passes
the frontmatter through verbatim and **every later property edit on that note is silently
discarded while every save reports success.**

This is the "indentation is data" class fixed in `update_frontmatter_title` (2026-07-21),
`set_frontmatter_parent` (2026-07-22) and six sites earlier *today* — and PJ-182 had already
been through this very function, routing only its seq-item half through `yaml_lines`. Fixed
with the same `is_top_level_key_line` guard; 3 new tests (nested child, nested inline array,
block-scalar prose).

### §6.3 The rest of the in-diff set (12 total, all fixed before commit)

`reindex_single_note` still returned `Ok(())` on a None DB while its sibling
`reindex_delete_note` was fixed today (the asymmetry was the finding) · `rename_item`'s `.md`
branch missed the `ensure_search_db_ready` guard its three siblings got · today's new
vocabulary-fingerprint gate stranded targets behind the re-materialize cursor · today's
reconcile crash-marker covered only OUTGOING, leaving INCOMING and sky stranded ·
`retarget_registered_libraries` (added today) loaded via the empty-on-failure loader ·
`settings.json` was the last persisted store with no read-succeeded latch — **and the boot
bundle is its only load path**, so the latch had to be wired there or every save would be
refused · the property-type registry survived a universe switch and would write universe A's
registry over universe B's file · `updateNoteProperty` resolved normally when its pre-write
flush aborted, so the Base table painted an unsaved cell as saved · and **my own boot gate
could park forever**: its single release site was reachable only from a button inside
`{#if visible}`, and the dialog has paths that finish invisible. Now every dialog exit
releases it, plus a 30 s belt.

### §6.4 The 24 pre-existing findings

Filed to `lab/reports/inspection-2026-08-01-remaining.md` — **not deferred silently**; they
need a Boss triage before scheduling, and they should be deduped against the 2026-07-30 feed
and PJ-187's 19 M-cost register first. Notable: `universe.rs:118` collapses an unreadable
`universes.json` into an empty registry and four write paths save that emptiness back
(silently deleting every registered Universe — the fix `libraries.json` already has);
`set_active_universe` holds the path mutex across a DB lock (whole-app freeze); and
`reconcile_filesystem`, named by many fixes as "the authoritative self-heal", has no
user-reachable trigger at all.

---

## §5 — Open / filed

- **Pre-existing i18n drift** — using the union of all locales as the reference, every locale
  is missing keys (de/es/fa/fr/hi/pt/tr/ur ~196 each, he/ru ~169, ja/ko/zh ~223, en ~94),
  including `plurals.*` families that `$tn()` depends on. NOT introduced today; today's keys
  are complete in all 15. Filed as a standalone task.
- **`$t(key) || 'fallback'` is a dead idiom** — `i18n/index.ts:186` returns the KEY on a miss,
  which is truthy, so the `||` fallback never fires and a missing key renders as a raw dotted
  string. `+layout.svelte` now has a local `tOr()` helper that compares against the key. Worth
  promoting to the i18n module itself.
- **Stage-B is still un-run.** It is gated on this session's verification inspection
  (`w484u96b7`, diff-scoped over all 27 changed source files) coming back clean, then a
  release build, then the Boss's live run.
