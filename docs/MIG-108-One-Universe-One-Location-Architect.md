# MIG-108 — One Universe, One Location — Architect (Phase 1)

**Date** 2026-07-30 · **Status** Phase 1 complete · **D1–D4 RULED** (see the Plan) → Phase 2 awaiting approval
**Boss rulings (2026-07-29/30):** (1) one central `.trash` at the universe root — the scope
setting collapses to one option; (2) *"unify the whole 'Eisa Cognitive Knowledge' content under
E:\Constellation Universes\ … and make sure there's only one universe location that holds its
content, even if users imported it from other PKM apps."*
**Evidence base:** 7-slice territory workflow + completeness critic (run `wf_1e0af182-c53`,
maps archived in the 2026-07-30 session log entry) — every claim below carries file:line read
this session. Prior-art cross-check per WA#5 (Obsidian, Logseq, Joplin, Zotero, Lightroom,
Calibre, iTunes).

---

## 0 · Concept (the horse)

> **A universe is one directory that wholly contains its knowledge.** Owning, backing up,
> syncing, or walking away with your knowledge means owning ONE folder. Everything the app
> derives may point *into* that folder; nothing the user owns may live *outside* it.

The function (the carriage) is a one-time, app-mediated relocation of every external library
into the universe root — plus the standing constraints that keep the invariant true afterwards
(creation, linking, and import flows can no longer place or reference content outside the root).

**This REVERSES a documented invariant.** `CLAUDE.md` → Knowledge Hierarchy currently says:
*"Additional libraries can have any path (subfolders or external). Libraries are never copied —
Constellation reads them in place."* That sentence is repealed by this migration and must be
amended in the same commit that lands the new behaviour — never before.

**Prior-art verdict (WA#5).** The industry pattern is *self-contained root + one external
pointer* (Obsidian vault, Calibre library, Zotero data dir). No mature system bulk-rewrites
user-visible paths; where an index must be re-pointed, the winning shape is **Lightroom's
"Update Folder Location"**: one guided operation, the app rewrites its own rows transactionally,
every unresolved item visibly flagged, snapshot first. The documented horror stories (Lightroom
"5 years of edits lost", Zotero broken attachments, Logseq dead watchers, iTunes exclamation
marks) all share one root: derived/earned state lived only in the index and the move happened
outside the app. That is *exactly* Constellation's `search.db` exposure — which is why this
migration is app-mediated, journaled, and snapshot-first.

---

## 1 · Measured scope (the Boss's universe = the rehearsal)

| | |
|---|---|
| Libraries registered | 20 (2 already under root) |
| **To relocate** | **18** — all Boss-confirmed TEST libraries |
| Notes / bytes | 7,684 / 297.8 MB |
| Volumes | all on E: → **18 atomic directory renames**, no file copying |
| `note_meta` rows to rewrite | 7,672 of 7,825 |
| `note_links` rows (earned: weight/confidence/traversal/status) | ~234,000 |
| `sky_links` rows | 233,995 |
| `review_schedule` + `review-pulse.json` | 7,825 rows + 4 path-keyed maps |

The 18 include two special cases the pre-flight must classify (see §5-P):
`PJ-065-test-book` sits inside the **source repo** (`lab/`), and any external entry could in
principle be another universe's root or child (cross-registry check — critic finding #3).

---

## 2 · The two questions the whole design turns on

**Q-A. Rewrite absolute paths in place, or convert the stores to root-relative paths?**
Prior art says relative-to-root is the destination (it is what makes a universe truly portable —
move the folder, update one registry entry, done). But converting is a far larger change: every
equality-keyed lookup, every writer, every IPC boundary joins `root + relative`.
**Decision: Option A now — in-place absolute rewrite (the Lightroom shape), preserving every
byte of earned data; file the relative-paths end-state as its own future migration.** A is
complete in itself, and it does not foreclose B: after A, all paths share one root prefix, which
is precisely the precondition B needs.

**Q-B. When does the relocation run?**
The flatten-migration precedent runs silently on activation (`ensure_universe_notes_folder`,
universe.rs:285-364). Silently renaming 18 directories the user chose violates "never modify
silently" and The Constellation Way. **Decision: detect on activation → PROPOSE (a dialog
naming exactly what was found and where it will go) → the user clicks Unify → journaled run
with progress → verified summary.** Detection with zero external libraries shows nothing ever.

---

## 3 · What is already proven safe (the wins the maps banked)

1. **Zero note-body rewrites.** Links resolve by NAME at click/render time — `note_links.target_path`
   is never populated (libraries.rs:1100-1101); the rename cascade fires only on TITLE changes
   (libraries.rs:6052, :1264-1266); structural links and embeds are name/relative-based. Moving a
   library directory rewrites **no user file**. The `.md` corpus is untouched by construction.
2. **FTS is untouched.** `notes_fts` is rowid-keyed, content=note_meta, and its trigger is gated
   on name/body change (search.rs:3692-3698, :3722-3729). A pure path rewrite costs zero
   retokenization.
3. **The per-note cascade already exists.** `migrate_note_db_paths` (libraries.rs:1102-1240)
   covers all 11 path-bearing tables with FK-deferral, destination pre-deletes, stamp gates and
   trigger-covered exclusions — the bulk rewrite replicates its statement set prefix-batched, it
   does not invent a new one.
4. **`earned.jsonl` is immune.** Keys are cid-first with a NAME fallback — no path ever enters a
   ledger key (link_life.rs:249-260; the module doc's claimed "path fallback" was never
   implemented — critic finding #7; fix the doc comment in this migration).
5. **Per-library sidecars travel free.** `.constellation/canonical`, reliability, `.obsidian/`
   appearance are location-relative — intact the moment `libraries.json` is re-pointed.
6. **Library IDs and NAMES survive** — appearance is id-keyed, property-types are name-keyed,
   `library_name` attribution keys off `libraries.json` roots; none of these change.
7. **cUniverse federation is out of scope by construction.** The under-root rule is per-universe;
   children are referenced by root path only and are never in the parent's `libraries.json`
   (universe.rs:443-500) — scoping relocation to own `libraries.json` entries is automatically
   correct.

---

## 4 · The hazards the maps and critic pinned (each becomes a plan clause)

| # | Hazard | Evidence | Mitigation |
|---|---|---|---|
| H1 | **FK refusal**: `UPDATE note_meta SET path` refused for any note with summaries/history/suggestions rows | search.rs:13884-13917; libraries.rs:1117-1141 | `BEGIN IMMEDIATE` + `PRAGMA defer_foreign_keys=ON`, parent+children in one tx (proven pattern) |
| H2 | **O(N²) trigger storm**: `note_links_outgoing_au` ungated (+17 s measured at 216k); `note_links_sky_au` → 468k redundant ops | search.rs:1442-1473, :4079-4094 | drop both trigger sets → bulk rewrite → recreate → `recompute_all_outgoing` once (reconcile_filesystem precedent, search.rs:1481-1489) |
| H3 | **Byte-exact matching is unsound**: stored paths vary in separator (Rust `\` vs JS `/`) and NFC is live on Arabic names | move-machinery Q5 (7 divergent normalizers); libraries.rs:6013-6017 | enumerate rows and match via normalized compare (the `delete_rows_under_prefix` pattern, search.rs:10372-10396) — never SQL `replace()`/`LIKE` (`_` wildcard app-killer, search.rs:10360-10363) |
| H4 | **Crash mid-move has NO net**: boot reconcile hard-aborts above max(200, 10%) stale — 7,684 stale rows is 10× the cap | reconcile.rs:56-59, :161-167 (critic #1) | a **migration journal** (`.constellation/mig108-journal.json`): per-library intent→moved→rewritten states; resume on boot; never lean on reconcile |
| H5 | **WAL-blind snapshot corrupts the backup** | WAL confirmed in 8 modules; no VACUUM INTO/backup primitive exists (critic #2) | `wal_checkpoint(TRUNCATE)` → copy `search.db` (+ sidecars if present) → verify by opening read-only + row-count; JSON stores copied alongside |
| H6 | **Cross-registry entanglement**: an external entry can BE another universe's root/child | libraries.rs:427-432 has no cross-check; primitives exist at universe.rs:1324-1341 (critic #3) | pre-flight classifier: such entries are **skipped + reported**, never moved; Boss decides their fate case by case |
| H7 | **Basename collisions + UNIQUE merges**: two libs can share a folder name; `UNIQUE(source_path,target_name,link_type)` merges silently under `INSERT OR IGNORE` | libraries.rs:434-437; search.rs:3820, :3931 (critic #4) | pre-flight collision scan; destination de-collide via `free_trash_name`-style numbering; post-rewrite row-count invariants catch any merge |
| H8 | **Two walkers lack the nested-library exclusion** — post-migration they double-walk all 18 libraries | canonical.rs:1190-1219/:1361-1388; embeds.rs:155-215 (critic #5) | give both `nested_library_paths` exclusion **before** the relocation ships (pre-work slice); embed index invalidate + first-wins precedence pinned by test |
| H9 | **Second screen is outside every freeze channel** | no freeze listener in SecondScreenPage (critic #6) | migration runs with the SS window force-closed (it is reopenable); simplest correct envelope |
| H10 | **Watcher re-registration no-ops on same id** — old path keeps being watched | watcher.rs:65-67 | explicit `unwatch_library(id)` → move → `watch_library(id,newPath)`; all watchers down during the run; `reindex_changed_paths` (drop-and-rebuild heal) must never fire |
| H11 | **Stale-path caches** | sight_v3 hash includes library paths (sight_layout.rs:189-190); link_stats_cache embeds paths (search.rs:7995-7997); 4 path-ordered backfill cursors; embeds index keyed by path string | wipe v3 rows, recompute link_stats, reset cursors, invalidate embed indexes — all in the same run |
| H12 | **JSON stores with earned/precious data**: review-pulse (4 path-keyed maps + an orphan-sweep that would PURGE old-path rows), workspaces (named snapshots, zero healing), session, collections (cid-less members), settings `folderTemplates` keys | json-state map ranking | rewritten in the same journaled run via `atomic_write`; review-pulse **before** any review reconcile can run |

---

## 5 · The migration run (design)

**P · Pre-flight (read-only, produces the proposal).** Classify every `libraries.json` entry:
under-root (skip) · external own library (relocate) · another universe's root/child (skip +
report, H6) · repo-resident (`PJ-065-test-book` — Boss ruling) · missing on disk (report).
Detect basename collisions (H7), same-volume vs cross-volume per entry, open-handle probe.
Output = the proposal dialog's content: *what*, *where to*, *what is skipped and why*.

**S · Snapshot.** H5 procedure. Journal created. Backup paths recorded in the journal.

**F · Freeze.** Flush all dirty tabs (the committed sweep's machinery); `markFreeze` +
`markCascadingLibrary` per library; force-close the second screen (H9); `unwatch_library` ×N
(H10). From here to V, no watcher, no autosave, no editor input on the moved subtrees.

**M · Move (per library, journaled).** `gate_rename(old_dir, new_dir)` — one atomic rename on
the same volume; cross-volume entries use copy+remove with the symlink-skipping recursive copy.
Journal: `intent → moved`. A failed rename (open handle) halts cleanly: journal shows exactly
which libraries moved; resume re-runs from the first unmoved entry.

**R · Rewrite (one DB transaction + the JSON stores).**
Inside `BEGIN IMMEDIATE` + `defer_foreign_keys=ON`, triggers dropped (H2): for each moved
library, enumerate + rewrite via normalized-prefix match (H3) across the 11-table set (the
`migrate_note_db_paths` statement set, prefix-batched), letting `note_meta_sky_au`… no — sky
triggers are dropped with the rest; sky_nodes/sky_links/note_aliases rewritten explicitly in
the same tx; recreate triggers; `recompute_all_outgoing` once. Then cursors/caches (H11).
Then the JSON stores (H12) + `libraries.json` re-pointed (the pivot) via `atomic_write`.
Journal: `rewritten` per store. Verification queries INSIDE the tx before COMMIT (see V).

**T · Trash consolidation.** Every `<library>/.trash` top-level entry → root `.trash` via the
shared de-collide pair; empty source `.trash` dirs removed. (`.trash` contents include folders,
`.base`, `.conflict-*.md.txt` sidecars — moved as units; index needs nothing, dot-paths are
filtered everywhere.)

**V · Verify (hard gate, inside/around COMMIT).** Invariants: zero rows under any old prefix ·
per-library row counts preserved · `SUM(weight)`, `COUNT(*)` over note_links and
`COUNT(review_schedule)` byte-identical to pre-move · FK check clean · every moved library's
dir exists at its new path with the same file count. Any failure → ROLLBACK the tx; the journal
+ snapshot make the fs moves reversible (rename back).

**W · Wake.** Rewatch all libraries at new paths; unfreeze; refresh caches; session restore
now finds rewritten paths. Show the summary (what moved, what was skipped, where the backup is).

---

## 6 · The standing constraints (what keeps the invariant true)

1. `add_library` (the single choke point — all 5 UI surfaces funnel there, import-linking §1.1):
   external paths are no longer registered in place. New flow: **copy-in** (walk → copy under
   root → register the copy) or **move-in**, per Boss ruling Q3 below. Rejection alone is not
   enough — the user's intent ("bring this into my universe") must still be servable.
2. `create_new_library_at` / `pick_folder`: destination constrained under the root (the Pick…
   affordance scoped or removed for kind 'library'); legacy `create_new_library` retired.
3. Importer: already copy-in; restrict target-library list to OWN libraries (today it offers
   read-only federated cUniverse libraries — a real hole found in passing).
4. `link_library_as_universe` conforms already (in-place = its own root); fix its double-entry
   registration shape (import-linking §3.3) in the registry-normalization pass.
5. Settings collapse: `trashFolderScope` removed (type, default, UI rows, 60 i18n keys, plus an
   explicit `delete parsed.trashFolderScope` purge — the load-path spread resurrects stale keys
   otherwise, trash-settings §D); `resolveTrashDestination` loses its 'library' arm; PJ-192's
   Rust `move_to_trash` command retired and its single caller re-pointed to `delete_path`.
6. Old-doctrine strings rewritten ×15 ("Link Existing Library", "Connect an existing folder",
   `app.tagline` "A Vault of Vaults" — which also violates the no-"vault" terminology rule).
7. `CLAUDE.md` Knowledge Hierarchy amended; orientation, User Manual ×15, help topics updated.

---

## 7 · Invariants that must not break (the Audit phase's checklist)

I1 zero `.md` bodies modified · I2 earned aggregates byte-identical (note_links weight/count,
review_schedule, review-pulse maps re-keyed not dropped) · I3 cid_cn never re-minted · I4
library ids + names unchanged · I5 FTS rowids untouched · I6 no drop-and-rebuild path executes
(reindex_changed_paths, reconcile, reindex_library forced) during the run · I7 crash at ANY
point → journal resume completes or rolls back; never a half-state without a recorded next step
· I8 collections cid-adopted members still resolve; cid-less + folder/search members re-pointed
· I9 macOS-neutral (no Windows-only APIs; NFC handled; `#[cfg]` for anything platform-specific)
· I10 boot time / typing latency / IPC unregressed after migration (measure on the 7,600-note
universe before/after per the Performance Rules).

## 8 · Rehearsal protocol (the disposable-universe advantage)

Before the Boss ever clicks Unify on the live universe: script a **full copy** of
`Eisa Cognitive Knowledge` + the external trees to a scratch root, run the entire migration
there (pre-flight → verify), assert every §7 invariant mechanically, then diff the rehearsal
universe's behaviour (search, links, review, collections, Sky) against expectations. Only after
a green rehearsal does the proposal dialog run on the real thing. The rehearsal harness is a
deliverable of the Plan, not an afterthought.

## 9 · Boss decision points (Phase 2 cannot start without these)

| # | Decision | Options | Recommendation |
|---|---|---|---|
| D1 | Target layout under the root | flat `<root>/<name>` · container `<root>/Libraries/<name>` · preserve source grouping (`<root>/العالم العربي/<name>`…) | see AskUserQuestion |
| D2 | "Bring in an external folder" semantics | Copy (original untouched) vs Move (take ownership), or ask-per-use | Copy default, Move offered |
| D3 | `PJ-065-test-book` (lives inside the source repo) | unregister vs relocate | unregister |
| D4 | Timing | this migration next, or after MIG-104 Slice 8 | next (it is the Boss's standing ruling and the sweep landed its prerequisites) |

## 10 · What this migration deliberately does NOT do (filed instead)

- **Root-relative paths everywhere** (the full portability end-state — move a universe by
  moving one folder + one registry entry): filed as a follow-up PJ; MIG-108 creates its
  precondition (one shared prefix).
- The in-app **Trash browser** (PJ-193) — unchanged in scope.
- A general "move a library between universes" feature — out of scope; the pre-flight's
  cross-registry SKIP is the guard.
