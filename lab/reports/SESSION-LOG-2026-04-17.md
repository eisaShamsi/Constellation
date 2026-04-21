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

### 7. Custom FTS5 tokenizer — `constellation` registered via FFI

Next-phase work from the morning's open item. Landed same day. Approach approved by the user after a research pass (per LL-024 — no more reinventing the wheel).

**Research findings (agent pass):**
- `rusqlite` 0.31's `ffi` module re-exports the SQLite FTS5 C API (`fts5_api`, `fts5_tokenizer`, `Fts5Tokenizer`, `FTS5_TOKENIZE_*` flags, `FTS5_TOKEN_COLOCATED`, `sqlite3_bind_pointer`) even without the `vtab` feature — enough for a custom tokenizer.
- The FTS5 tokenizer is invoked **symmetrically** on write (document insert) and read (`MATCH` query). We stem once in the tokenizer and every `notes_fts` row + every search query key on the same stem space. No query interception needed.
- `snippet()` / `highlight()` keep working because FTS5 stores byte offsets **separately** from token bytes. We emit the stem as token content but report the original word's byte range.
- Bigrams are emitted as **colocated tokens** (share position with the preceding unigram), joined by `\x1f` (Unit Separator) sentinel. Phrase queries still resolve because SQLite re-tokenizes the query string through the same tokenizer.
- Three reference implementations surveyed: `sqlite-zstd` (too heavy, needs writable DB), `fts5-snowball` crate (English-only stemmer, no Arabic/Hebrew), **ColonelThirtyTwo gist** (~150 LOC of pure FFI glue, MIT-licensed, load-bearing pattern — the right shoulder to stand on).

**Files**

New: `src-tauri/src/fts5_tokenizer.rs` (~360 LOC)
- `pub trait Tokenizer` — the Rust-side interface (global state + `fn tokenize(reason, text, push_token)`).
- Four `unsafe extern "C"` shims (`c_xcreate`, `c_xdelete`, `c_xtokenize`, `c_xdestroy`) adapted from the gist. Panic boundary via `catch_unwind(AssertUnwindSafe(...))` so Rust panics become `SQLITE_ERROR`, not process crashes.
- `pub fn register_tokenizer<T: Tokenizer>(conn, name, global)` — binds the fts5_api pointer via `sqlite3_bind_pointer`, calls `xCreateTokenizer`. Leak-safe: if registration fails, the boxed Global is reclaimed (`let _ = Box::from_raw(boxed_global);` — fixed a latent leak in the upstream gist).
- `pub struct ConstellationTokenizer` — walks the input with `char_indices()` matching `libraries::tokenize_note_body` word-boundary rules, calls `libraries::process_word_for_fts` (new helper) for each word, and emits:
  - `stem` as the primary token at the word's original byte range;
  - previous-word + current-word bigram as a colocated token (`stem1\x1fstem2`) only if both are non-stopwords and same-script.
- `pub const BIGRAM_SEP: u8 = 0x1f` — `notes_vocab` consumers convert this back to space for display.

Changed: `src-tauri/src/libraries.rs`
- `build_stopwords()` and `is_same_script()` → `pub(crate)`.
- New `pub(crate) fn process_word_for_fts(word) -> Option<(stem, norm_lower)>` — length rules (Arabic 2-20, others max 40, min stem 2) + language routing to `process_arabic_word` / `strip_hebrew_prefix` / `stem_persian / russian / hindi / german / spanish / portuguese / french / turkish / english`. Single source of truth for per-word stemming.
- `read_index_entries` & `read_term_mentions`: `let mut conn`, `register_fts5_tokenizer(&mut conn)?;` before any `notes_fts` / `notes_vocab` query. Without this every fresh connection fails with "no such tokenizer: constellation" because the tokenizer registry is per-connection.
- `read_index_entries` additionally converts the `\x1f` sentinel to space for display (`entry.term`) and flags the row with `is_compound` so the Index panel can render bigrams distinctly.

Changed: `src-tauri/src/search.rs`
- Added `const FTS_SCHEMA_VERSION: i64 = 1;` (ledger: 0 = legacy `unicode61`, 1 = `constellation`).
- `register_fts5_tokenizer(conn)` helper — builds the stopword set once and hands an `Arc<HashSet<String>>` to the tokenizer's Global.
- `init_db` now registers the tokenizer **before** any `CREATE VIRTUAL TABLE notes_fts`; the fts5 VTable definition uses `tokenize='constellation'`.
- Migration: reads `PRAGMA user_version`. If less than 1, drops `notes_vocab` + `notes_fts`, re-creates them with the new tokenizer, runs `INSERT INTO notes_fts(notes_fts) VALUES('rebuild')` to re-tokenize every `note_meta.body_text`, writes the new user_version, then `PRAGMA wal_checkpoint(TRUNCATE)` — defense against the 3.1 GB WAL incident from earlier today.
- `reconcile_filesystem`'s walker connection also registers the tokenizer — the FTS5 AFTER-INSERT/UPDATE triggers on `note_meta` invoke `xTokenize` from that connection; without registration the trigger fails.

Changed: `src-tauri/src/lib.rs`
- `mod fts5_tokenizer;` registered alongside the other modules.

**Error fixed during build-check**
- `rusqlite::Error::ModuleError` is gated behind the `vtab` feature (we only enable `bundled`). Replaced with `rusqlite::Error::SqliteFailure(ffi::Error::new(ffi::SQLITE_TOOBIG), Some("token longer than c_int".into()))` — the overflow path that would fire if a single token exceeds ~2 GB (never).

**Where it still registers**
| Connection | File | Register? | Why |
|---|---|---|---|
| `init_db` main conn | `search.rs` | ✓ | creates `notes_fts`, runs rebuild |
| `reconcile_filesystem` walk conn | `search.rs` | ✓ | writes trigger `xTokenize` |
| `read_index_entries` | `libraries.rs` | ✓ | queries `notes_vocab` (uses tokenizer at query-time for some ops) |
| `read_term_mentions` | `libraries.rs` | ✓ | `MATCH` query |
| `cache.rs open_reader` | `cache.rs` | ✗ | read-only, plain `SELECT` only, no `MATCH` — tokenizer never invoked |

**Build status:** `cargo check` clean (0 errors, 8 warnings — all pre-existing in `embeds.rs` / `lens.rs` / `search.rs`).

**Pending user test** (see "Open items").

### 8. Uncapped Index + real virtualization (frontend)

After the tokenizer landed, the user built + booted the release on the 7,600-note Arabic-heavy trial Universe. Diagnostics confirmed the tokenizer is alive:

```
[read_index_entries] user_version=1 notes_vocab total=5,689,896 filtered(len>=2, cnt>=5)=516,563
arabic samples: [("عام", 22047), ("عرب", 21860), ("دول", 20029), ("عبد", 13845),
                 ("مدين", 13506), ("اول", 13086), ("سن", 13019), ("اخر", 11320),
                 ("اسلام", 10834), ("محمد", 10723)]
```

Arabic stems are present. The earlier `unicode61` bug that showed only Latin at the front of the list is gone because the `ORDER BY term LIMIT 50000` has been removed from `read_index_entries`.

**The remaining problem:** the panel hung on "Building index…" because Svelte's `{#each}` over 516k rows materialized half-a-million `<div>` nodes on the main thread. CSS `content-visibility: auto` (added as a first swing) skips paint/layout for off-screen rows but doesn't stop node creation — so it wasn't enough. The multi-column `columns: 280px auto` layout was also incompatible with row windowing.

User picked **(B)**: ship real virtualization, keep the `cnt >= 5` SQL threshold, keep no ceiling. Ruled out pulling `virtua` / `svelte-tiny-virtual-list` — the Index panel's shape (flat list + inline section headers + one-at-a-time expansion) is narrow enough that a purpose-built ~110 LOC component wins on RTL control, bundle size, and zero-dep discipline.

**New file: `src/lib/components/VirtualList.svelte` (~110 LOC)**
- Svelte 5 runes-native, generic over `T`.
- `$derived.by` builds a `Float64Array` prefix sum of row heights via `getItemHeight(item, i)` — re-runs when `items` changes or when any `$state` read inside `getItemHeight` changes (Svelte's signal tracking crosses component boundaries).
- Binary search over the offsets array → first visible index; plus `overscan` on each side.
- Only the visible slice is rendered. Each rendered row is absolutely positioned (`translateY(offsetPx)`) inside an inner div sized to the full virtual height — scroll bar position stays accurate.
- `scrollResetKey` prop: whenever its value changes, the list scrolls back to top (used for filter / letter / sort switches).
- `ResizeObserver` on the container keeps `viewportHeight` accurate across pane resize; `onDestroy` disconnects per Rule 4.

**Rewrite: `src/lib/components/IndexPanel.svelte`**
- **Flattened model.** `VRow = { kind: 'header', letter } | { kind: 'entry', entry } | { kind: 'expanded', term }`. The `rows` derivation walks `groupedEntries` (alpha mode) or `freqEntries` (freq mode) and emits a single flat array with letter headers and optional expanded-mentions rows inlined after their owning entry.
- `getRowHeight` returns 30px for headers/entries; for expanded rows it's `max(32, 12 + mentions.length * 22)`. Because `mentionsCache` is a `$state`, expanding a term triggers a re-derivation of the prefix sum on the fly when mentions resolve — the expanded row smoothly grows to fit.
- **Removed**: the `visibleCount` lazy-load counter, the `handleScroll`-on-bottom auto-expand, the `listEl` ref, the `$effect` that reset `visibleCount` on filter change, the nested `{#each letter}{#each group}` rendering, the multi-column CSS (`columns: 280px auto; column-gap: 8px`), the `content-visibility: auto` / `contain-intrinsic-size` CSS experiment, and the `break-inside: avoid` / `break-after: avoid` rules that only made sense under multi-column.
- **Kept**: all filter/sort/script logic, alphabet bar, script tabs, context menu, anchor bar, and all RTL rules.
- **Dropped JS re-sort inside each letter group.** SQL's `ORDER BY term` (BINARY collation) already produces acceptable dictionary order within a single script, because the `constellation` tokenizer has already normalized Arabic prefixes and case at write time. Saves ~O(n log n) `localeCompare` calls on 100k-row groups.
- **Capped `maxCount`** at a 5000-item sample (was `Math.max(...scriptFilteredEntries.map(e => e.count))` — blew the call stack on 500k-entry arrays).
- **Capped export** at 5000 terms with an inline notice — `navigator.clipboard.writeText` on 500k entries would generate multi-MB strings and issue 500k `ensureMentionsLoaded` IPC calls.

**Verification**
- `npm run check`: only the pre-existing a11y warnings inherited from the original `<span onclick>` pattern — no new type or runtime errors. No issues in `VirtualList.svelte` or the refactored `IndexPanel.svelte`.
- Release build (`npm run tauri build`) in progress as of the SO.

**User test — PASS.** User reported: *"All pass, all work, with almost instant speed."* All six steps verified on the 7,600-note Universe with 516,563 filtered terms:
1. Panel opens immediately; no "Building index…" hang.
2. Smooth scroll top-to-bottom across the half-million rows.
3. Alphabet bar click scrolls + filters to the selected letter.
4. Filter search is lag-free on keystroke.
5. Expand/collapse term mentions works; no row overlap.
6. Alpha ↔ Frequency sort switch is instant.

## Open items

- **Test on the 7,600-note Arabic Universe.** Expected observations:
  - `SELECT COUNT(*) FROM notes_vocab WHERE LENGTH(term) >= 2` — was ~452k on `unicode61`, target 30-60k.
  - First boot after upgrade: `[search] notes_fts rebuilt with 'constellation' tokenizer in N ms` in the log. If N > 10_000 ms, move the rebuild to a background task post-paint (Rule 8 — first-time population runs after paint with progress bar).
  - Index panel: bigrams appear (space-joined, not `\x1f`), Arabic terms look like stems (e.g. `كتاب` instead of `الكتاب` / `كتابه` / `كتابي`), English terms stemmed (`run` covering `running/runs/runner`).
  - Boot time: no regression vs. 2026-04-17 morning measurement on the same Universe.
- **Phase after: port Write-Time Derivation to Sky View.** `skyNodes` + `skyLinks` currently rebuild on every boot from `allLibraryLinks`. Cache `sky_nodes` / `sky_edges` tables; maintain via note_links-change hooks.
- **Then**: Backlinks, Outgoing, Tag browser, Sight, sidebar stars, Map — same rule, same pattern.

## Commits

1. `9d62cd2` — **Index panel on FTS5 vocab + Write-Time Derivation rule.** Fixed the broken panel, documented the principle, set up the tokenizer phase.
2. `9187a62` — **Custom FTS5 `constellation` tokenizer + uncapped virtualized Index panel.** Arabic Light10 + multi-language stemmers + bigrams; versioned migration rebuilds `notes_fts` on first boot. New `VirtualList.svelte` renders all 516k terms smoothly at constant DOM cost. User-verified on the 7,600-note Arabic Universe ("almost instant speed").

---

## Phase: Index complementarity — snippet per mention, Arabic filter, multi-term compare

After the virtualized panel landed, the user locked in a **design rule** about the five core functions (Search Hub, OrgChart, SV, Map, Sight): each must **complement**, not overlap, the others within Cognitive Knowledge / Knowledge Formulation. Applied to the Index, that ruled out temporal sparklines (→ Sight) and thinking prompts (→ SV), leaving two Index-appropriate enhancements: **snippet per mention** (term in lexical context) and **co-occurring terms** (term-to-term adjacency).

User directive: *"Finish the index first, then the boot Criterion 2."*

### (a) Snippet per mention — PASS

**Rust — `src-tauri/src/libraries.rs`**
- Extended `IndexMention` with `snippet: Option<String>`.
- `read_term_mentions` SQL now joins `notes_fts` and calls `snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12)` — STX/ETX sentinels around matched tokens, 12-token window, ellipsis on clip. Column `-1` = all columns.
- Sentinels (U+0002 / U+0003) over literal `<mark>` so literal HTML typed in a note never reaches the DOM.
- Fixed two legacy `IndexMention { ... }` constructions in `scan_library_index` (added `snippet: None`).

**Frontend — `src/lib/libraries/store.ts` + `src/lib/components/IndexPanel.svelte`**
- Added `snippet?: string | null` to `IndexMention` TS interface.
- `splitSnippet()` helper parses STX/ETX into `{text, mark}[]` parts; text is interpolated through Svelte's default escaping; `mark` parts wrapped in `<mark class="gp-ref-hit">`.
- Mention row restructured: two-line flex column with `.gp-ref-name` (note title) + optional `.gp-ref-snippet` (`dir="auto"` for mixed-script context).
- VirtualList height logic: new constants `ROW_HEIGHT_MENTION_PLAIN = 22`, `ROW_HEIGHT_MENTION_SNIPPET = 40`. `getRowHeight` sums per-mention heights from the cache — a term whose mentions have mixed snippet/no-snippet renders correctly.

**User verification**: *"الفهرس يعمل جيدا"* ("The Index works well") — PASS on the 7,600-note Arabic Universe.

### (b) Arabic filter with definite article — user reported bug

User report: *"when I search for an Arabic term with the definite article 'ال' the result is often 0. The Index should be able to identify the root of the term, even if the searcher writes the word with the definite article."*

Root cause: the backend `constellation` FTS5 tokenizer normalizes and stems Arabic words via `process_word_for_fts` → `normalize_arabic` → `stem_arabic_light10`. So "الكتاب" indexes as "كتاب". When the user types "الكتاب" in the filter box, a raw JS substring match against "كتاب" returns zero.

**Fix — `src/lib/components/IndexPanel.svelte`**
- Added `normalizeArabicForFilter(s)` — an exact JS mirror of `stem_arabic_light10`:
  - Normalize: strip diacritics (U+064B..065F, U+0670, U+06D6..06ED), tatweel (U+0640); unify أ إ آ ٱ → ا; ى → ي; ة → ه.
  - 3-char prefixes وال فال بال كال — needs remaining ≥ 3 chars.
  - 2-char prefixes ال لل — needs remaining ≥ 2 chars.
  - 1-char prefix و — needs remaining ≥ 3 chars.
  - 2-char suffixes ها ان ات ون ين يه يت ته — needs remaining ≥ 2 chars.
  - 1-char suffixes ه ي — needs remaining ≥ 2 chars.
- `termMatchesQuery(termLower, queryLower)`: direct substring OR stemmed substring. Fast-gated by `ARABIC_RE.test(queryLower)` so Latin/CJK queries skip the regex passes.
- Wired into both comma-separated OR branch and the single-query branch of `filteredEntries`.

Only Arabic is mirrored here (highest payoff — ال is on virtually every noun). Multi-language query-side stemming via a shared IPC command is tracked as a separate broader follow-up.

### (c) Multi-term "Comparing" chips had no results — user reported

User report (screenshot of `indexPanel.comparing: ابريز × ابريل × ابشر × ... clearAll` bar): *"why when I ctrl many terms it doesn't display the results? It only highlights the selected terms."*

Root cause: `onTermSelect` updates `indexSelectedTerms` in the parent, which renders the chips, but no UI in IndexPanel ever computed or rendered the intersection.

**Fix — `src/lib/components/IndexPanel.svelte`**
- New `comparisonState` `$derived.by`:
  - `idle` when <2 terms selected.
  - `loading` when any selected term's mentions aren't in `mentionsCache`.
  - `ready` with the INTERSECTION of mention sets by `note_path` (notes containing ALL selected terms — "commonality").
  - Algorithm: sort mention lists smallest-first, build Sets from the rest, filter smallest by `every(s => s.has(...))`. O(|smallest| × k) — cheap even on thousands of mentions.
- New `$effect` (gated by `untrack()` per Rule 2) kicks off `ensureMentionsLoaded` for any selected term whose cache entry is missing — belt-and-braces for callers that pre-populate the set without going through the ctrl-click path.
- New `.gp-commonality` panel rendered between chips bar and term list: shows count ("N notes contain all K selected terms"), scrollable list of note buttons, or "No notes contain all selected terms." fallback. `max-height: 40vh; overflow-y: auto` so the main term list stays visible for adding more terms.
- i18n: added 7 new keys (`comparing`, `clearAll`, `loadingCommonality`, `noCommonality`, `noteWithAll`, `notesWithAll`, `selectedTerms`) to all 15 locale files (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh). JSON validated across all 15.

### Verification
- JSON sanity: all 15 locale files parse cleanly.
- Release build: PASS (exit 0, 8 pre-existing warnings, no new ones, both MSI + NSIS bundles produced; trailing `TAURI_SIGNING_PRIVATE_KEY` is the known benign auto-update-signing warning).
- **User test — PASS.** Both (b) Arabic definite-article filter and (c) multi-term commonality verified on the 7,600-note Arabic Universe: *"Great, both fixes passed."*

---

*Next session pickup: run the release binary on the trial Universe, capture `notes_vocab` row count + rebuild time + boot time, then commit the tokenizer. After that: Sky View port to Write-Time Derivation.*
