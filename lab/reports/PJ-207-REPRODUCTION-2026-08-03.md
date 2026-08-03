# PJ-207 — Reproduction record: the repair pass that can never run

**Date:** 2026-08-03 · **Rule:** Reproduce-First (no defect-targeting change ships, or is even
designed, before the defect is reproduced on demand).

**Function in hand:** the app's index self-heal — `reconcile_filesystem`
(`src-tauri/src/search.rs`), the pass every other maintenance site in the codebase calls
"the authoritative self-heal".

**Concept (the horse):** *File Over App means the files can always restore the app.* When a
derived view drifts from the `.md` files on disk, the user must be able to make Constellation
re-derive it from the files — and see it happen. The pass exists. It has no door.

---

## 1. The reproduction, on the Boss's live universe

Not a theory and not a synthetic fixture — measured read-only against
`E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db`
(2,026,405,888 bytes, 7,824 `note_meta` rows) with Constellation **not running**.

| Measurement | Result |
|---|---|
| `note_meta` rows | **7,824** |
| Rows whose indexed `modified` == the file's mtime on disk | 7,764 |
| **Rows whose disk content is NEWER than the index** | **60** |
| Rows whose file is missing from disk | 0 |
| Of those 60, rows with body words on disk **absent from `note_meta.body_text`** | **57** |

Largest drift measured: **4,735,509 seconds ≈ 55 days** (`Anabolism.md`, `Carotene.md`,
`Photosynthesis.md`, `Babylonian astronomy.md` and others — a bulk external touch).
Smallest: 15 s (`Doi (identifier).md`).

Body-only comparison (frontmatter stripped from the disk file before comparing, ASCII words
≥ 6 characters only so Arabic normalisation and markdown-stripping cannot produce a false
positive):

```
Arcesilaus.md            words on disk, not in the index: carneades, skeptical, attachments
Vishnu Purana.md         words on disk, not in the index: mountains, planets, rivers
Arche.md                 words on disk, not in the index: archai, babylonian, reiteration
Indo-European languages.md   11 missing, e.g. assyrian, philologer, rudhir, naktis
Europe.md                disk body 165,074 chars · indexed body 161,205 chars
```

**Those words are not findable.** Read from the live database's own schema:

```sql
CREATE VIRTUAL TABLE notes_fts USING fts5(
  name, body_text, content=note_meta, content_rowid=rowid, tokenize='constellation')
CREATE TRIGGER note_meta_au AFTER UPDATE ON note_meta
  WHEN OLD.name IS NOT NEW.name OR OLD.body_text IS NOT NEW.body_text ...
```

`notes_fts` is an external-content table over `note_meta.body_text`. A word absent from
`body_text` is absent from search, from the Index panel's vocabulary, and from every surface
derived from them. This is not inferred — it is the schema in the running universe.

---

## 2. The named recipe (what the Boss can run)

1. **Close Constellation.**
2. Edit any note's `.md` outside it — Notepad, Obsidian, a `git pull`, a Syncthing sync — and
   add a distinctive word.
3. **Launch Constellation.** Search for that word.
4. **It is not found.** It stays not-found across restarts, forever.

This is exactly how the 60 rows above came to be.

---

## 3. The mechanism, read off the source (not theorised)

**Why the drift is never healed:**

| Boot step | Does it re-read changed files? |
|---|---|
| `refreshLibraryCaches()` | no |
| `reconcile::maybe_schedule` (`reconcile.rs:73`) | **no** — it heals *existence* drift only (relocate by `cid_cn` / re-adopt orphan / remove dead). It never compares mtime or content. |
| `reindex_library(onlyIfUnindexed: true)` (`+layout.svelte:2860`) | no — server-gated on a library with **zero** indexed notes |
| `cache_mark_search_ready` (`+layout.svelte:2905`) | no — explicitly the walk-free counterpart (`cache.rs:1544`) |
| the file watcher | no — it starts *after* boot and only observes events from then on; the edit already happened |
| the `initSearchIndex()` auto-recovery (`+layout.svelte:2891`) | **only when `totalIndexed === 0`** |

**Why the cure works:** `reconcile_filesystem` (`search.rs:10468`) →
`index_library_recursive` (`search.rs:7241`) → `index_note(conn, path, lib, force=false)`
(`search.rs:6569`), whose gate is:

```rust
if !force {
    let existing_mod: Option<u64> = conn.query_row(
        "SELECT modified FROM note_meta WHERE path = ?1", ...).ok();
    if existing_mod == Some(modified) { return Ok(()); }   // cache hit
}
```

It re-reads **exactly** the files whose mtime moved — the 60 — and skips the other 7,764.
It then runs `recompute_all_outgoing`, `recompute_all_incoming`, `recompute_all_sky`,
`tag_counts::recompute_all_in` and `review::recompute_all_in`.

**Why the user can never reach it.** `reconcile_filesystem` has exactly two command wrappers:

- `constellation_search_init` (`search.rs:10639`) → frontend `initSearchIndex()`
  (`store.ts:3703`), whose **only four** call sites are
  `+layout.svelte:2892` (gated on a completely empty index), `:4690` (new library),
  `:5971` (link an external library), `:5984` (bring a library in).
- `cache_reconcile` (`cache.rs:1511`), registered at `lib.rs:506` — **zero frontend
  callers.** A registered door with nothing on the other side of it.

**And the app tells the user to press a button that does not exist.** `storeHealth.index`
resolves to *"…Settings → Rebuild Index will restore it"* (`+layout.svelte:558`,
`en.json:4245`) — and the same promise is translated into **all 14 other locales**.
`SettingsModal.svelte` contains no such control: only an orphan CSS rule
`.semantic-status-rebuild` at `:2868` and a comment at `:110` recording that the old
"Rebuild Term Embeddings" button *was removed* by MIG-013.

---

## 4. Danger check on the cure (before designing the fix)

Running a re-index must not destroy the earned half of the Living Link Architecture, which
`CLAUDE.md` records as living **only** in `search.db`. Verified in source:
`index_note`'s link rebuild is an incremental **diff-edges** rebuild with an explicit
preserve predicate — `(traversal_count > 0 || weight != 1.0 || status != "active") && !structural`
(`search.rs:338`) — with tests `tests_archive_survives_save` (`search.rs:324`) and
`pj066_diff_edges_leaves_unchanged_rows_untouched` (`search.rs:1403`).
**An archived link stays archived and earned weight survives.** Reconcile is a mtime-gated
walk, never a drop-and-rebuild.

---

## 5. How long the repair holds the writer lock — the first measurement ever taken

The whole-ecosystem sweep reported that **no measurement of `reconcile_filesystem` exists anywhere**
in `lab/`, `docs/` or the session logs, and that the two figures in circulation must not be cited:
the "~1.0 s bulk walk" (`SESSION-LOG-2026-05-30.md:20-24`) is a synthetic two-table SQL benchmark
(`links_backfill.rs:636-760`) with no file read, no frontmatter parse and no FTS tokenization; and
the "30–60 s" in the 2026-05-04 decision memo is an estimate, not a measurement, predating the 2 GB
database.

Measured here on a **byte copy** of the live 2,026,405,888-byte `search.db` (moved outside the
universes tree; live database never touched). The SQL is verbatim from the Rust source, not
reconstructed.

| Pass | Source | Measured |
|---|---|---|
| `tag_counts::recompute_all_in` — `DELETE` | `tag_counts.rs:58` | 127 ms |
| `tag_counts::recompute_all_in` — `INSERT … json_each … GROUP BY` | `tag_counts.rs:59-67` | **13,040 ms** |
| commit | | 3 ms |
| **→ writer-lock hold, one transaction** (`search.rs:10579`) | | **13.2 s** |
| `review::recompute_all_in` — orphan sweep `DELETE … NOT IN` | `review.rs:1361` | 2,747 ms |
| `review::recompute_all_in` — materialize every body into a `Vec` | `review.rs:1366-1375` | **17,886 ms** (7,824 rows, **260 MB** of body text resident at once) |
| **→ writer-lock hold, one transaction** (`search.rs:10599`) | | **20.6 s**, before the Rust FNV hash + `backfill_one` loop |

**≈34 seconds of writer-lock hold in two transactions** — and these two are the only tail passes
that are *not* batched. Their three siblings (`recompute_all_outgoing` / `_incoming` / `_sky`) walk
in 500-row windows with busy-retry precisely because a single whole-table UPDATE "silently failed
under boot DB contention… the 2026-05-30 overnight blank" (`links_backfill.rs:259-263`).

**Why that matters for a button.** The walk's connection has `busy_timeout(30 s)`
(`search.rs:10496`); `state.db` — the connection every user save goes through — has **5 s**
(`search.rs:3634`). A save landing inside either transaction parks on `BEGIN IMMEDIATE`, times out
after 5 s, and does so **while holding the single `state.db` mutex that 71 call sites need**. So the
repair as it stands does not merely take time: it can freeze the window and fail a save while doing
it.

One correction to the sweep: it warned that the review pass materializes "the 123 MB note" cited in
`docs/MIG-078-Phase-BL-Design.md`. Not in this universe — the largest single body here is **0.3 MB**.
The 260 MB is the aggregate, which is the real cost.

## 5b. The Full re-read floor, and the foreign-copy oscillation — measured / verified 2026-08-03

**Full re-read is minutes, not seconds.** Reading every note in the live universe from disk, with no
parsing at all:

| | |
|---|---|
| Files read | 7,824 |
| Bytes | 298 MB |
| Wall clock | **49.0 s** (6.27 ms/note) |

That is **I/O only**. Frontmatter parsing, markdown stripping, Arabic normalisation, wikilink and
heading extraction, the `note_links` diff and FTS re-tokenisation all sit on top of it. Any
confirmation dialog for "Full re-read" must quote a measured figure, and 49 s is its **floor**.
This is why the ruling was *measure before exposing it*.

**The foreign-copy removal would have oscillated — and the cure already exists.** Charter W2-9 is
open because `reconcile_filesystem` walks the *federated* library set
(`search.rs:10515` → `load_all_libraries` → `resolve_universe_libraries` →
`resolve_libraries_recursive`, `universe.rs:1465-1469`) and writes foreign cUniverse notes into the
active universe's `search.db`. Verified today: **the boot reconcile does the same** —
`reconcile.rs:91` takes its roots from the identical recursive call, and `reconcile.rs:280-300`
**re-adopts** any `.md` under an accessible root that has no `note_meta` row, via
`reindex_single_note`. So deleting the foreign rows would have them re-created on the next launch,
with a `link_life` ledger append and fsync every cycle, forever.

The cure is a helper that already exists and was written for exactly this discipline:
`universe::own_libraries_for_root` (`universe.rs:1479`), documented at `:1471-1478` as
*"NON-recursive — deliberately WITHOUT the federated cUniverse libraries… an edit must never land on
a read-only cUniverse file."* **Both** passes must route through it — scoping only the walk is the
half sweep.

One caveat to carry into the build: `own_libraries_for_root` reads `libraries.json` with
`unwrap_or_default()`, so an unreadable or corrupt file yields an **empty list**. For the boot
reconcile that is safe (no roots → do nothing). For a repair it would mean *walk nothing and report
success* — the "couldn't read it → you have none" class PJ-200 closed elsewhere. It needs the same
read-succeeded discipline.

## 6. What this record does **not** establish

- The duration of the walk itself, or of the three batched tail passes. Their SQL is assembled from
  the live link-type registry (`incoming_aggregate_assignments`, `stratum_sql_expr`), and I will not
  reconstruct it by hand — a reconstruction that drifted would be a measurement of the wrong thing.
- Any of it under real contention. All figures above are from an idle copy; they are a **floor**.
