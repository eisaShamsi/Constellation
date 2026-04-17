# Session Log — 2026-04-17

## Headline

**Index panel reborn on SQLite FTS5 vocab** after four failed hand-rolled attempts. New Performance Rule 8 added to CLAUDE.md: *Write-Time Derivation* — every computed view is maintained at write time, not read time. Plan for the next phase: register a custom FTS5 tokenizer to restore Arabic Light10 stemming + multi-language stemmers + bigrams.

## Work in order

### 1. Task D (Index panel investigation) — four failed approaches before the breakthrough

The Index panel had regressed in a prior session — broke when a Universe-walk tokenizer was pulled off the boot path. Symptom: panel stuck on "Building index…" with 0 terms.

Four attempts over the session, each failing for a different reason:

| # | Approach | Outcome |
|---|---|---|
| 1 | Whole-Universe SQL scan (one IPC, one big `HashMap<term, mentions>`) | OOMed on 7,600-note Arabic-heavy Universe — mentions Vec grew to hundreds of MB |
| 2 | Per-library sequential SQL scan (frontend loop, 17 batches) | Still OOMed on a single Arabic-heavy library (one library = 800+ notes × many shared terms) |
| 3 | Streaming per-note tokenizer, writes to `index_mentions` table, SQL GROUP BY per library | Ran 20+ min, thrashed disk. Root causes: (a) correlated subquery in finalize SQL (`ORDER BY count_in_note DESC LIMIT 1` inside GROUP BY — O(N²) over terms), (b) read query `index_terms LEFT JOIN index_mentions` returned 5M rows, ~1.5 GB of Rust structs, never returned |
| 4 | Tighter SQL, tune finalize, lazy-load mentions | Started the surgery but froze the app entirely on next boot — SQLite had to replay a **3.1 GB WAL** left by approach 3's committed writes into tables I then dropped |

### 2. Research — stopped reinventing the wheel

Dispatched a deep-dive research agent. The verdict was unambiguous: **SQLite FTS5's `notes_fts` table already has everything the Index panel needs**.

Key findings:
- `fts5vocab(notes_fts, 'row')` is a virtual table that exposes `(term, doc, cnt)` — exactly what `index_terms` was trying to be, maintained automatically on every `note_meta` insert / update / delete via the existing FTS5 triggers.
- `MATCH` on `notes_fts` gives the posting list for a term in O(log n) — exactly what `index_mentions` was trying to be, queryable without a custom table.
- Custom tokenization (Arabic Light10, per-language stemmers, bigrams) can be plugged in later by registering a custom FTS5 tokenizer from Rust; the spike over the existing `unicode61` tokenizer was the cheap, decisive first step.
- Tantivy would be over-engineering at this stage; FTS5 covers the use case with what's already in the binary.

### 3. The fix that shipped

**Rust** (`src-tauri/src/search.rs`):
```sql
CREATE VIRTUAL TABLE IF NOT EXISTS notes_vocab USING fts5vocab(notes_fts, 'row');
-- Drop leftover tables from aborted custom-index experiment:
DROP TABLE IF EXISTS index_mentions;
DROP TABLE IF EXISTS index_terms;
DROP TABLE IF EXISTS index_meta;
```

**Rust** (`src-tauri/src/libraries.rs`):
- Removed: `scan_index_populate_batch`, `tokenize_note_local`, `IndexBatchResult` struct.
- Rewrote `read_index_entries` as:
  ```sql
  SELECT term, cnt FROM notes_vocab
  WHERE LENGTH(term) >= 2 AND cnt >= 5
  ORDER BY term LIMIT 50000
  ```
  Measured 345ms / 810KB payload on 7,595-note Universe.
- Added `read_term_mentions(term, limit)` — `SELECT … FROM notes_fts MATCH ?1` with `JOIN note_meta`. Sub-10ms per call.

**Frontend** (`src/routes/+layout.svelte`):
- Deleted the batch-loop `$effect`, the `indexProgressDone` / `indexProgressTotal` state, and the status-bar progress indicator (CSS + HTML + i18n keys stay in place unused — they're so small it's not worth removing this session; revisit when the custom tokenizer lands).
- New `$effect`: single `await readIndexEntries()` on `graphReady`. One IPC round-trip.

**Frontend** (`src/lib/components/IndexPanel.svelte`):
- Added `loadMentions?: (term: string) => Promise<IndexMention[]>` prop.
- Added local `mentionsCache: Map<string, IndexMention[]>` + `loadingMentions: Set<string>` + `ensureMentionsLoaded(term)` helper.
- Replaced every `entry.mentions` read with `getMentions(entry.term)` — renders on expand, export, and onTermClick/onTermSelect handlers all route through the cache.
- `toggleExpand(term)` triggers `ensureMentionsLoaded(term)` fire-and-forget; rendering updates when the cache fills.

**Frontend** (`src/lib/libraries/store.ts`):
- Dropped `scanIndexPopulateBatch`, `IndexBatchResult` interface.
- Added `readTermMentions(term, limit)` wrapper.

### 4. User database rescue — 3.1 GB WAL

The user's `search.db-wal` had ballooned to 3,095 MB during the earlier failed streaming run. Every boot SQLite replayed the WAL, freezing the app. Fixed externally via Python's stdlib `sqlite3`:

```python
conn.execute('PRAGMA wal_checkpoint(TRUNCATE)')  # 100ms
conn.execute('VACUUM')                            # 65s
```

Result: `search.db` 770 MB → 711 MB, WAL: 2,959 MB → gone.

### 5. Verified test pass

User confirmed:
- Boot: no indexing bar, fully responsive.
- Index panel: populates in ~2 seconds with terms, filter-as-you-type works, expansions load notes instantly.
- Reboot: still instant.

### 6. CLAUDE.md Rule 8 added

Formalized the lesson into a standing rule:

> **Write-Time Derivation.** Every computed view in Constellation is maintained at write time, not read time. When a note changes, every derived surface that depends on it updates in the same transaction. The app does not recompute on boot. It does not recompute on panel open. It reads what's already stored.

Canonical example cited: FTS5's `notes_fts` triggers on `note_meta`. Canonical use case cited: the new Index panel via `notes_vocab`. Audit list of surfaces still violating the rule: Sky View, Backlinks, Outgoing, Tag browser, Sight dashboard, sidebar star counts, Map. Each must be ported in future phases, with before/after measurement on a 7,600-note Universe.

## Lessons

- **LL-021 (proposed): Don't reimplement a sorted on-disk dictionary that SQLite FTS5 already ships.** Before writing custom tokenization-to-tables schemas, check whether `fts5vocab` does it. It does. Four failed attempts × days of work could have been one research pass.
- **LL-022 (proposed): WAL checkpointing is not automatic in failure scenarios.** A large aborted write (millions of rows into a table that later gets dropped) leaves the WAL with committed-but-unchecked-pointed frames. Next boot replays them serially. Add a periodic `PRAGMA wal_checkpoint(TRUNCATE)` on some cadence or after large writes; consider `PRAGMA journal_size_limit` to cap growth.
- **LL-023 (proposed): Test with the real Universe, not an estimate.** 50k terms was my estimate. Reality was 452k (10×). Arabic without stemming explodes the vocabulary. For any performance-sensitive SQL pattern, count the real rows before shipping.
- **LL-024 (proposed): Research before reinvention.** When three attempts fail for three different reasons (LL-014 says stop), the instinct to add more tricks is wrong. The right move is to step back and research the domain. 30 minutes of reading FTS5 docs would have saved all four failed attempts.

## Files touched

- `src-tauri/src/search.rs` — added `notes_vocab` virtual table, dropped `index_*` leftover tables.
- `src-tauri/src/libraries.rs` — removed ~360 lines of custom-index code, added `read_index_entries` (fts5vocab scan) + `read_term_mentions` (MATCH query).
- `src-tauri/src/lib.rs` — swapped command registrations.
- `src/lib/libraries/store.ts` — swapped frontend wrappers.
- `src/routes/+layout.svelte` — simplified `$effect`, removed progress-bar state/CSS/HTML, wired `loadMentions` prop.
- `src/lib/components/IndexPanel.svelte` — added `loadMentions` prop + local mentions cache + lazy-load on expand.
- `CLAUDE.md` — added Rule 8: Write-Time Derivation.
- *External*: user's `search.db` at `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\` — vacuumed (WAL truncated).

## Open items

- **Phase next: custom FTS5 tokenizer.** The Index panel currently uses `unicode61 remove_diacritics 2` — no Arabic Light10 prefix stripping, no multi-language stemmers, no bigrams. Reference implementation: `greentechapps/sqlite3-arabic-tokenizer`. Plan: register a custom tokenizer from `rusqlite` that wraps Constellation's existing `tokenize_note_body` / `process_arabic_word` / `stem_english` / … pipeline. Create a second FTS5 table (contentless) using it, re-point `notes_vocab`. Expected effect: 452k terms collapse to ~30–60k, easily under the 50k cap.
- **Phase after: port Write-Time Derivation to Sky View.** `skyNodes` + `skyLinks` currently rebuild on every boot from `allLibraryLinks`. Cache `sky_nodes` / `sky_edges` tables; maintain via note_links-change hooks.
- **Then**: Backlinks, Outgoing, Tag browser, Sight, sidebar stars, Map — same rule, same pattern.

## Commits expected

1. **Index panel on FTS5 vocab + Write-Time Derivation rule** — today's change. Fixes the broken panel, documents the principle, sets up the next phase.

---

*Next session pickup: register a custom FTS5 tokenizer wrapping the existing Constellation tokenization pipeline (Arabic Light10 + multi-language stemmers + bigrams), create a second FTS5 table using it, re-point `notes_vocab` to that table.*
