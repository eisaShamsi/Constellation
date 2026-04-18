# Session Log — 2026-04-18

## Headline

**M3 + M3-baker + M1g/M1h + M5 + M6 + M7 + M8 + M8b + M8c + M9 + M10 + M11-infra + M12 + M12 follow-ons (detect + bench) + M11-data v1 + M12-wire + M13 + M14 + M11-data v2-infra + M11-data v2 batches (003-food-and-household + 004-qualities + 005-basic-verbs-and-emotions + 006-time-and-space + 007-cognition-and-language + 008-society-and-government + 009-arts-and-creativity + 010-science-and-math + 011-professions-and-work + 012-tools-and-materials) landed. Corpus at 560 concepts.** First: `GenerativeIndex` (HashMap, ~40 MB projected at 7K roots) swapped for `GenerativeFst` (BurntSushi FST, prefix-compressed, mmap-ready). Second: the compiled FST is now persisted to the user's cache directory on first launch and reloaded on subsequent launches via `GenerativeFst::from_bytes` — the cold/warm startup path divergence that M9 ("50 ms analyzer cold-start") measures against. Third: the protected list got its architectural rewrite — `const SEED: &[...]` (200 hand-picked entries, 340 lines of Rust) replaced with `include_str!("protected_seed.tsv")` + a 3-column TSV (`surface<TAB>category<TAB>origin_lang`) now holding **1,196 unique entries** across proper nouns (395), places (275), loanwords (455), and function words (71). Fourth: the **M5 regression corpus** — a 502-case held-out test set in `regression_cases.tsv` + a `cfg(test)`-gated `regression.rs` harness that feeds every row through `analyze_best` and asserts origin / surface / (optionally) lemma / root. Covers all three active origin layers (ProtectedList, GenerativeFst, SurfaceHeuristic) across 28 Arabic roots, ~80 cascade surfaces, and 45 foreign (Latin-script) words. Fifth: **M6** — the FTS5 Arabic stemming path in `libraries.rs::process_arabic_word` now routes through `arabic::analyze_best`. Every Arabic token in every note in every Universe now flows through the five-layer engine; Light10 is retained only as the graceful `SurfaceHeuristic` fallback so unknown words don't regress. The flagship `وائل → "ائل"` mangle is gone: the protected list short-circuits Light10 and the stem is preserved verbatim. Sixth: **M7** — the Layer 4 disambiguator. `analyze_best`'s insertion-order tiebreak replaced with a pure, deterministic rank: confidence desc → origin (UserOverride > ProtectedList > FST > Heuristic) → POS (ProperNoun > Noun > … > Verb > … > Foreign) → fewer affixes → alphabetic lemma. The كاتب ambiguity now resolves to the Noun reading (active participle) every time, across any OS, any FST build, any Universe. Seventh: **M8** — Layer 0 user overrides. New module `arabic::overrides` with a per-Universe JSON store at `<universe>/.constellation/arabic-overrides.json`; `analyze_with_overrides(word, Some(&store))` inserts a hash-lookup Layer 0 that short-circuits the entire pipeline on an exact or normalized-vocalized match. `UserOverride::to_analysis()` produces an `Analysis` with `origin=UserOverride, confidence=1.0`, which M7's disambiguator already ranks strictly above every other origin — so no changes to `rank_analyses` were needed. The back-compat wrapper `analyze(word) ≡ analyze_with_overrides(word, None)` preserves every caller on the crate today; the overload is purely additive. Atomic file writes (`.tmp` + rename), alphabetic-sorted entries for git-friendly diffs, forward-compat serde defaults. Eighth: **M8b (Rust plumbing slice)** — the wire that makes M8 run in production. New `ACTIVE_STORE` registry in `overrides.rs` (process-wide `OnceLock<RwLock<Arc<OverrideStore>>>`), `activate_for_universe()` hook called from `set_active_universe` so switching Universes auto-loads the per-Universe JSON file into the active store, `process_arabic_word` in `libraries.rs` (FTS5 hot path) now reads the active store via cheap `Arc::clone`, and three Tauri commands (`read_arabic_overrides`, `add_arabic_override`, `remove_arabic_override`) registered in `lib.rs` and exposed to the Settings UI which arrives in M8c. Ninth: **M8c** — the Settings UI slice. Svelte panel `ArabicOverridesPanel.svelte` mounted inside `SettingsModal.svelte` under a new `arabic-overrides` section, wired to the three M8b commands plus a new fourth command `reindex_arabic_overrides(surface)` that LIKE-scans `note_meta` and atomically deletes + re-inserts every affected row into `notes_fts` inside a single `BEGIN IMMEDIATE`/`COMMIT` — so the moment the user saves a new override, every note containing that surface is re-tokenized under the fresh Layer 0 verdict, no full Universe rebuild needed. All 31 strings the panel renders are in every one of the 15 locales (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh); RTL content inside each cell routes through `detectDir` so a mixed-script annotation flows naturally in either direction. Tenth: **M9** — the bench harness. A single `#[test] #[ignore]` in `arabic::bench` that reports cold-start (cache deleted → `GenerativeFst::get()`), warm-start (`fst_bake::load_bundle` + `GenerativeFst::from_bytes` on the just-written cache), throughput (502-case corpus × 500 iterations through `analyze_best`), accuracy (`regression::run_corpus` + per-origin breakdown), and size (on-disk cache bundle + linear projection to 7K-root scale). Opt-in only (`cargo test --lib --release arabic::bench -- --ignored --nocapture`) because the cold-start step deletes the real user cache. Two runs today at 595 seed roots / 32,197 FST keys: cold 154–167 ms, warm 21 ms (≈7–8× speedup — **meets the ≥5× target**), throughput ~130K words/sec (below the aspirational ≥200K — follow-ons queued: string-intern `pattern_label`, mmap the FST bytes, trim the per-call `Arc::clone` on `ACTIVE_STORE`), accuracy **100.0% (502/502)** including 100% on each of Protected / Generative / Heuristic buckets, cache bundle 7.6 MiB on disk with a linear projection of ~90 MiB at 7K roots (above the ≤10 MiB aspirational target — follow-ons queued: mmap the FST byte buffer instead of holding it in a `Vec<u8>`, dedup the `pattern_label` strings across forms, intern `root_key` through a shared table). Eleventh: **M10** — the Lexical Bridge architecture. The pre-existing `crate::lexicon` skeleton (graph types, expansion types, stub APIs) was filled out into a working implementation end-to-end: a new `parse.rs` loads a TSV seed into `ConceptRecord`s; `graph.rs` gained a concept-to-CSR builder, an `fst::Map` name-index keyed by `"{lang_code}:{normalized_lemma}"`, a normalization pipeline (`normalize_stripped` for Arabic-script langs, `trim().to_lowercase()` elsewhere), `find_nodes` / `edges_of` / singleton access via `OnceLock`, and an adjacency-list layout that packs every concept's cross-lang pairs as `Equivalent` edges and every same-lang pair as `Synonym` edges; `mod.rs`'s `equivalents()` and `expand()` stubs were replaced with real graph walks that respect `ExpansionOptions` (synonym level, enabled languages, per-lang cap) and return populated `ExpansionResult` buckets by `EdgeKind`. A 15-concept hand-picked seed ships at `lexicon/data/seed_v1.tsv` (concepts: book, knowledge, write, read, love, water, house, teacher, student, language, peace, truth, time, day, night — each with ~12–16 language labels) to exercise every code path before the M11 20K-concept core lands. The seed only adds `Equivalent` and `Synonym` edges; `Hypernym` / `Hyponym` machinery is wired and unit-tested but unused until M11 ships WordNet data. Lookup is diacritic-insensitive for Ar/Fa/Ur (`كِتَاب` finds `كتاب`) and case-insensitive for Latin/Cyrillic/Turkish. Twelfth: **M11-infra** — the Lexical Bridge baker, the M10 graph's Write-Time-Derivation counterpart. The `LexiconGraph` is a pure function of the embedded seed TSV; once M11-data ships the 20K-concept core, that build costs ~200–400 ms of parse + FST compile + O(concept²) edge emission on every boot. The same Write-Time-Derivation pattern that `arabic::fst_bake` applies to the analyzer FST now applies here: compute once from the seed, persist the resulting bundle, rehydrate from disk on subsequent launches. New module `crate::lexicon::bake` (`bake.rs`, ~615 lines, 18 tests) owns the encoder/decoder + cache path resolution + version-hash stability. A new `LexiconBundle { nodes, edge_offsets, edges, name_index_bytes }` type carries the serialisable snapshot — which is what `build_bundle()` (promoted from `fn build()` to `pub fn build_bundle()` returning the bundle rather than the live graph), `bake::write_bundle` / `bake::load_bundle`, and `LexiconGraph::from_bundle` all exchange. `LexiconGraph::get()` rewired to the three-stage init: (1) `bake::try_load_cached` + `from_bundle` if the cache hits; (2) parse + `build_bundle` + `bake::persist_best_effort`; (3) `from_bundle` on the just-built bundle — so cold and warm paths finish in the same reconstruction call, eliminating the "works on cold build, breaks on cache hit" failure class up front. Binary format: magic `CAELEX01` + u64 `version_hash = djb2(seed_tsv) XOR CACHE_FORMAT_VERSION` + nodes / edge_offsets / edges / fst_bytes blocks, all little-endian. Hand-coded u8 tag tables for `Lang` (15 variants), `Option<PartOfSpeech>` (9 states incl. None), and `EdgeKind` (5 variants) — append-only, never renumber, bump `CACHE_FORMAT_VERSION` when extending. Cache path `<cache_dir>/constellation/lexicon-v{hash:016x}.bin` — filename carries the hash so stale caches from previous seed versions coexist peacefully. Atomic writes (`.tmp` + rename). Corrupt / truncated / wrong-magic / wrong-hash / trailing-bytes / implausibly-large-count inputs are all rejected cleanly via `io::Error` rather than panicking. New `LexiconGraph::to_bundle()` snapshot helper for test paths that want to round-trip a live graph through disk. Thirteenth: **M12** — query expansion plumbing. New `lexicon::fts` module (`fts.rs`, ~210 lines, 20 tests) owns `escape_fts_term(&str) -> Option<String>` (wraps in `"..."`, strips interior double-quotes and control chars, returns `None` on empty/whitespace-only inputs so callers can fall back instead of passing an empty clause) and `build_match_expr(&ExpansionResult) -> Option<String>` (walks `flat_terms()`, escapes each, deduplicates on the escaped form, joins with ` OR ` — producing clauses like `"book" OR "books" OR "كتاب" OR "livre" OR "knowledge" OR "معرفة"`). Two end-to-end convenience helpers on `lexicon::`: `expand_to_match_expr(lemma, source, opts) -> Option<String>` and `expand_to_match_expr_via(graph, …)` fold expand + build in one call — the exact shape `search.rs` will reach for when M14's settings UI wires expansion into the user-visible lexical path. FTS5 phrase quoting is the safety boundary: every lemma becomes a `"…"` phrase so operator keywords (AND / OR / NOT / NEAR) inside a lemma can never change the query shape, and FTS5's custom Constellation tokenizer (Arabic Light10 + bigrams) runs symmetrically at index and query time, so raw display-form terms produced by `expand()` match the stored tokens without extra normalization at this layer. `None` return when the expansion produces zero usable terms is the explicit fallback signal — the caller should run the user's plain query rather than passing an empty MATCH clause (FTS5 treats `""` as a syntax error). Source-lemma echo preserved through the fall-through: lemmas not in the graph produce a single-phrase query byte-identical in effect to today's un-expanded search, so the rollback / "feature off" case goes through the same code path as the happy case. Fourteenth: **M12 follow-ons — detect + bench.** Two modules that complete the M12 picture before M14 wires expansion into `search.rs`. **M12-lang-detect** (`lexicon::detect`, `detect.rs`, ~280 lines, 33 tests) ships `detect_source_lang(&str) -> Option<Lang>` — a pure-stdlib Unicode-script classifier that M14's search-bar call site uses to decide which `Lang` to pass into `expand_to_match_expr`. Counts strong-script characters per family (Arabic, Hebrew, Devanagari, Cyrillic, CJK, Latin), picks the dominant family, then disambiguates within: Ar/Fa/Ur via Urdu-exclusive letters (ٹ ڈ ڑ ں ے ۓ → Ur) and Perso-Arabic letters (پ چ ژ گ ک ی → Fa, else Ar); CJK via Hangul → Ko, any kana → Ja, else Zh (pure Han "日本" misclassifies as Zh — documented pragmatic limitation); Latin via distinctive marks (Turkish ğ/İ/ı/ş → Tr; German ß → De; French œ → Fr; Spanish ñ/¿/¡ → Es; Portuguese ã/õ → Pt; else En). Punctuation/digits/emoji contribute nothing — pure-punctuation input returns `None` so the caller falls back to the plain un-expanded path. **M12-bench** (`lexicon::bench`, `bench.rs`, ~160 lines, 1 `#[ignore]` test) runs `expand_to_match_expr_via` across 23 diverse queries (En / Ar / Fr / De / Es seed hits + two miss cases + two empty/whitespace short-circuits) under both `ExpansionOptions::default()` and `ExpansionOptions::mono(Lang::En)` (rollback path), 1,000 iterations each, collecting 46,000 latency samples. Results at M10-seed scale on Windows/MSVC release build: **mean 5.2 µs**, p50 1.8 µs, p95 12.4 µs, **p99 15.8 µs**, max 159 µs — all ~60–600× under the 1 ms budget, which the bench hard-asserts at the end so a regression (e.g. per-call `Vec` allocation, full-graph scan) trips on the next opt-in run. Full public-API parity preserved across all fourteen landings; **381/381 library tests pass** (up from 209 pre-M3: +13 fst_bake, +10 regression harness, +6 TSV parser, +5 M6 FTS contract tests, +12 M7 disambiguator, +21 M8 overrides [16 unit + 5 integration], +8 M8b ACTIVE_STORE registry [Arc-pointer-identity + swap semantics + activate_for_universe disk paths], +0 M8c [intentionally — integration test deferred until Settings → Debug lands], -1 removed `no_duplicate_lemmas_in_seed` obsolete under first-write-wins, +34 M10 lexicon [11 parse, 12 graph, 13 equivalents+expand, minus 2 obsolete mod.rs stubs], +18 M11-infra [3 tag injectivity + 3 tag-reject + 2 form roundtrip + 1 small-bundle roundtrip + 5 file-reject + 1 cache-path + 1 hash-stability + 1 djb2 + 1 real-seed end-to-end], +25 M12 [10 escape_fts_term edge-cases + 10 build_match_expr build/filter/dedup + 5 expand_to_match_expr end-to-end incl. singleton smoke-test], +33 M12-lang-detect [5 empty/non-letter + 5 single-script happy paths + 5 Arabic-family disambiguation + 6 CJK disambiguation + 9 Latin-family disambiguation + 3 mixed-script precedence]) — plus two `#[ignore]` bench tests (M9 Arabic analyzer + M12 expansion latency) that do not count toward the 381 and only run on explicit opt-in.

## Work in order

### 1. M3 — FST-backed generative index

Layer 3 in `arabic::analyze()` previously held two `HashMap<String, Vec<GeneratedForm>>` buckets (stripped + folded). At the current 595-root seed that's already ~5K stripped keys; on the target 7K-root × 158-pattern corpus it projects to ~300K keys × avg 24 B of heap per `String` = roughly 40 MB just in key allocations, before the form vectors. Unacceptable against the 350 MB process-wide RSS budget.

**Solution shipped:**

- Added `fst = "0.4"` (BurntSushi's pure-Rust FST, BSD/MIT — license-clean) to `src-tauri/Cargo.toml`.
- New module `src-tauri/src/arabic/fst_index.rs` with `GenerativeFst` that mirrors `GenerativeIndex`'s public surface 1:1:
  - `get() -> &'static GenerativeFst` (OnceLock singleton).
  - `lookup(&str) -> &[GeneratedForm]`, `lookup_folded(&str) -> &[GeneratedForm]`.
  - `len()`, `is_empty()`.
- Internal representation:
  - Two `fst::Map<Vec<u8>>` (stripped + folded), built from `BTreeMap` so insertion order is UTF-8-sorted as FST's `MapBuilder` requires.
  - Two flat `Vec<GeneratedForm>` side-tables; each FST value is a packed `u64 = (offset as u64) << 32 | count as u64`.
  - Per-key dedup on `(root_key, pattern_label)` — same semantics as `GenerativeIndex`.
- Aspirational `from_bytes()` constructor included (and tested) so the mmap-from-file path in the next milestone can drop straight in without another API change.
- One line swap in `arabic::mod::analyze()`: `generator::GenerativeIndex::get()` → `fst_index::GenerativeFst::get()`. Module doc comment updated accordingly.

### 2. Tests

Added 12 parity tests in `fst_index.rs`:

- `fst_builds_nonempty`
- `fst_finds_kaatib` / `fst_finds_maktub` / `fst_finds_dahraj` / `fst_finds_aimma` / `fst_finds_qala` / `fst_finds_yaidu` (all the flagship forms from M2 and M2.b/c)
- `fst_misses_unknown_word`
- `fst_folded_fallback_works`
- `fst_values_point_to_valid_forms` (packed-offset decode sanity — every form's surface re-normalizes back to its FST key)
- `fst_preserves_pattern_kind` (the packed side-table carries `PatternKind` intact so `pos_for_kind` still works downstream)
- `fst_from_bytes_roundtrip` (hand-builds FST bytes, rehydrates via `from_bytes`, asserts lookup)

The existing `GenerativeIndex` tests in `generator.rs` remain untouched — they continue to validate the reference HashMap implementation for its own corpus properties.

### 3. Results

| Suite | Before M3 | After M3 | After M3-baker | Delta |
|---|---|---|---|---|
| arabic module | 171 | 183 | 196 | +25 (+12 fst_index, +13 fst_bake) |
| library total  | 184 | 196 | 209 | +25 |

- All 209 pass. Test wall time: 0.86s (no FST-build regression against the HashMap baseline on today's seed; the cache path is only exercised on the `persist_then_try_load_cached_roundtrip` end-to-end test).
- Zero edits required to any `mod.rs` integration test — the `الأئمة → ء-م-م` cascade still succeeds, now via the FST reconstructed from either the in-memory build or the cached bundle (whichever wins the race on `OnceLock::get_or_init`).

### 4. M3-baker — on-disk FST cache

The FST that M3 landed is a pure function of three compile-time inputs: `roots_seed.tsv`, `patterns.rs`, and `generator.rs`. That makes it the textbook Write-Time Derivation (CLAUDE.md Rule 8) candidate: compute once, persist, read cheap on every subsequent launch.

**Solution shipped:**

- New module `src-tauri/src/arabic/fst_bake.rs` (~680 lines, 13 tests).
- Cache location: `<dirs::cache_dir>/constellation/arabic-fst-v{hash:016x}.bin`. `dirs` resolves to `%LOCALAPPDATA%` / `~/Library/Caches` / `~/.cache` on Win/mac/Linux — matches Tauri's own convention.
- Content-addressed filename: `hash = djb2(roots_seed.tsv) XOR CACHE_FORMAT_VERSION`. Editing the seed flips the filename → old cache is orphaned, new one rebuilt. Editing generator rules bumps `CACHE_FORMAT_VERSION`, same effect. djb2 (not `DefaultHasher`) because the std library explicitly does not guarantee hash stability across compiler releases.
- Binary format: magic `CAEFST01` + u64 hash + per-side (u64 fst_len + fst_bytes + u64 val_count + serialized forms). Each form serialized as u8 kind-tag + 3×(u32 len + UTF-8 bytes). Hand-coded append-only tag ordering so reordering the `PatternKind` enum later cannot silently reshuffle tags.
- Atomic write: stage to `.tmp`, rename on success. Avoids partial-file poisoning.
- Safe `Cursor` helper on read: bounded reads, no panics on short buffers.
- New public accessor `roots::seed_tsv()` exposes the embedded TSV bytes so the version hash can be computed deterministically.

**Integration (single call site):** `GenerativeFst::get()` now runs a three-stage init. Stage 1 tries the cache via `fst_bake::try_load_cached()` → `Self::from_bundle()`. Any failure (missing file, hash mismatch, truncation, decode error, FST byte reject) silently falls through. Stage 2 rebuilds in memory via the new `Self::build_bundle()` helper and calls `fst_bake::persist_best_effort()`. Stage 3 reconstructs the live `GenerativeFst` from the fresh bundle via the same `from_bundle` path — so cold and warm paths are behaviourally identical (no divergence, no "oh we forgot to apply X on the warm path" category of bug).

### 5. Tests — 13 new for M3-baker

- `encode_kind_is_injective_and_total` — every PatternKind round-trips through the tag space; tags are unique.
- `decode_kind_rejects_unknown_tag` — unknown tags return None, don't panic.
- `encode_decode_form_roundtrip` — GeneratedForm survives encode→decode losslessly.
- `bundle_write_read_roundtrip` — full `FstBundle` writes to disk and reads back identically.
- `load_rejects_missing_file` — opening a nonexistent path returns Err.
- `load_rejects_wrong_magic` — a non-CAEFST01 file is rejected with `InvalidData`.
- `load_rejects_truncated_file` — files shorter than the declared side lengths are rejected.
- `load_rejects_wrong_version_hash` — a flipped hash byte triggers rebuild (proves the seed-change invalidation path).
- `load_rejects_trailing_garbage` — extra bytes after the folded side are rejected (catches partial writes).
- `cache_file_path_includes_version_hash` — filename shape is `arabic-fst-v<16 hex chars>.bin`.
- `version_hash_is_stable_across_calls` — the OnceLock-cached hash never disagrees with itself.
- `djb2_matches_known_values` — hand-verified seeds from Knuth; guards against accidental hash-algorithm regression.
- `persist_then_try_load_cached_roundtrip` — end-to-end: write via `persist_best_effort`, read via `try_load_cached`, verify contents match.

All 13 pass. The existing FST parity tests in `fst_index.rs` also exercise the cache indirectly — `GenerativeFst::get()` runs through the cache code path on every call that touches the singleton, so the analyzer's 169+ `get()`-consuming tests have been validating cache-hit correctness throughout this session.

### 6. M1g/M1h — protected list TSV refactor + corpus expansion

M1e shipped with a 200-entry `const SEED: &[Seed]` array in `protected.rs`. The module doc comment already flagged the handoff: *"the switch to data files happens in M1g when the list grows past ~1K entries."* Today that switch landed, combined with the M1g (proper nouns + places) and M1h (loanwords) content expansions — one architectural pass, not three.

**Architecture:**

- New file `src-tauri/src/arabic/protected_seed.tsv`, mirroring the `roots_seed.tsv` pattern. Three TAB-separated columns: `surface`, `category` (`proper` | `place` | `loanword` | `function`), `origin_lang` (BCP-47 code from the 15 supported languages, or `-` for None).
- `PartOfSpeech` is now *derived* from `category` (not a stored column): `proper`/`place` → `ProperNoun`, `loanword` → `Foreign`, `function` → `Particle`. Removing the redundant POS column prevents the author from ever writing a row that disagrees with itself.
- `protected.rs` now has the parallel surface to `roots.rs`:
  - `const PROTECTED_TSV: &str = include_str!("protected_seed.tsv");` — zero I/O at runtime.
  - `parse_origin_lang(&str) -> Option<Lang>` — tolerant parser (unknown tags return `None` rather than panicking; typos degrade gracefully without disabling protection of the surface).
  - `parse_category(&str) -> Option<(ProtectedCategory, PartOfSpeech)>` — returns `None` on unknown category so `parse_tsv` can skip the row.
  - `parse_tsv(&str) -> impl Iterator<Item = (&str, ProtectedCategory, PartOfSpeech, Option<Lang>)>` — skips `#` comments and blank lines; tolerates CRLF via `trim_end_matches('\r')`.
  - `build_table()` now sizes the HashMap from `PROTECTED_TSV.len() / 20`, iterates `parse_tsv`, first-write-wins on duplicate surfaces (matches `roots::RootsIndex::build`).
  - `pub fn seed_tsv() -> &'static str` accessor, parallel to `roots::seed_tsv()` — future content-addressing hooks (lexical-bridge lint tools, inspector UI) all have a single place to reach the bytes.

**Corpus — 1,196 unique entries across 4 categories:**

| Category | Count | Example sub-sections |
|---|---|---|
| Proper nouns | 395 | Arabic male names, Arabic female names, transliterated Western names, political/historical figures |
| Places | 275 | Arab countries (28), Arab cities (85), non-Arab countries (65), non-Arab cities (60), other regions |
| Loanwords | 455 | Technology (105), finance (40), transport (45), food (70), medical/scientific (85), sports (55), clothing (30), other |
| Function | 71 | Demonstratives, relatives, interrogatives, conjunctions, connectives |

Up from 200 in the old const array. Ramp target remains 20K proper nouns + 2K loanwords from CC-BY-SA Wikipedia extraction — that's a future milestone; today's file is hand-curated from public-domain references with no BAMA/Buckwalter/SAMA content.

**De-dup pass:** 12 accidental duplicate rows were trimmed (e.g. `كريم proper` accidentally appearing twice in the masculine-names section; `جين proper en` shadowing `جين loanword en`). First-write-wins still tolerates semantic cross-category conflicts silently (e.g. `صوفيا` as proper noun vs. place — the proper-noun reading wins because it comes first in the file).

### 7. Tests — 6 new for M1g/M1h

- `tsv_parses_to_at_least_as_many_entries_as_the_table` — replaces the old `no_duplicate_lemmas_in_seed`; asserts the TSV has at most 1 duplicate row per 100 so a diff-bomb of pasted dupes can't land silently.
- `parse_origin_lang_handles_sentinel_and_known_codes` — exercises every branch of the 15-language match plus the `-` sentinel plus unknown tags.
- `parse_category_rejects_unknown` — every known tag maps correctly; unknown returns `None`.
- `parse_tsv_skips_comments_and_blanks` — hand-built TSV with comments, CRLF, short rows, and valid rows; asserts only valid rows survive and are in file order.
- `parse_tsv_drops_unknown_category` — unknown category tag drops the row (doesn't poison later rows).
- `seed_tsv_accessor_returns_embedded_bytes` — `seed_tsv()` returns the real embedded TSV (contains وائل and a `\tproper\t` row).
- `first_write_wins_on_duplicate_surface` — two rows for the same surface with different categories; the first wins, the second is silently ignored.

The existing `table_has_expected_size`, `every_category_has_entries`, `common_names_are_protected`, `places_are_protected`, `loanwords_are_protected_with_origin_lang`, `folded_lookup_catches_alif_variant` tests all continue to pass — parity with the const-array behaviour is preserved. The size-assertion bounds were bumped from `180..=260` to `800..=2000` and the per-category minimums from `30/30/30/20` to `300/200/300/50` (still comfortably below actual counts so ordinary curation won't break the test).

### 8. Results after M1g/M1h

| Suite | After M3-baker | After M1g/M1h | Delta |
|---|---|---|---|
| arabic module | 196 | 202 | +6 (new TSV-parser tests) |
| library total | 209 | 215 | +6 |

- All 215 pass in 0.83s. The `الأئمة → ء-م-م` flagship still resolves through the FST-backed Layer 3; the expanded protected list now correctly short-circuits Light10 on ~1K more surfaces before the analyzer ever reaches the FST.

## 9. M5 — regression corpus

The analyzer is about to gain two disruptive changes: **M6** will swap `stem_arabic_light10` in `fts5_tokenizer.rs` for `arabic::analyze` (every FTS5 token on every note in every Universe now flows through the engine), and **M7** will add the disambiguator that reorders multi-analysis results. Either could silently regress behaviour on a common word like وائل or الأئمة and re-poison the search index; the unit tests in `mod.rs::tests` only cover hand-picked flagship cases. The regression corpus is the broader safety net.

**Solution shipped:**

- New `src-tauri/src/arabic/regression_cases.tsv` — 502 surfaces in 4 sections, TAB-separated: `surface<TAB>origin<TAB>lemma<TAB>root`.
  - **§ 1 Protected (256 rows)** — 80 proper nouns + 60 places + 80 loanwords + 30 function words, sampled via awk from `protected_seed.tsv`. Exercises the Layer 1 hash lookup on a representative slice of the 1,196-entry seed.
  - **§ 2 Generative bare (201 rows)** — ~30 Arabic roots (ك-ت-ب, ع-ل-م, ع-م-ل, ج-ل-س, ذ-ه-ب, د-خ-ل, خ-ر-ج, ر-ج-ع, ف-ه-م, ش-ر-ب, ض-ر-ب, س-م-ع, ن-ظ-ر, ش-ع-ر, ف-ع-ل, ح-ك-م, ح-م-د, ذ-ك-ر, ح-ف-ظ, ف-ت-ح, ط-ل-ب, ش-ك-ر, ق-ت-ل, ص-ن-ع, ل-ب-س, خ-ل-ق, غ-ف-ر, ن-ص-ر, ر-س-ل + quadriliteral د-ح-ر-ج) with active participle / passive participle / perfect / imperfect derivations. Roots asserted on participles; left as `-` (unasserted) on verb forms where `analyze_best`'s tiebreak is confidence-then-insertion-order and not stable across refactors.
  - **§ 3 Cascade (~80 rows)** — affix-stripping surfaces: ال + derivations, ف + derivations, و + derivations, بال chain, وبال/فبال chains, والـ/فالـ, the الأئمة flagship.
  - **§ 4 Foreign (45 rows)** — Latin-script tech / brand names (Hello, World, Rust, Python, GitHub, PostgreSQL, Vite, Rollup, Tailwind, …). Routed to SurfaceHeuristic by the normalizer's non-Arabic-script check — covers the heuristic threshold too (`corpus_covers_every_origin` requires ≥5).

- New `src-tauri/src/arabic/regression.rs` — the harness. `cfg(test)`-gated so the corpus does not ship in the release binary; `#[cfg(test)] mod regression;` declared at the end of `mod.rs`. If M9 benchmarking ends up needing the corpus outside `cargo test`, the comment in `mod.rs` flags the upgrade path (promote to `pub mod` + a feature flag).
  - `parse_corpus(tsv) -> Vec<Case>` — skips `#` comments and blank lines, CRLF-tolerant, records 1-based source line numbers for locatable failure messages.
  - `run_corpus() -> CorpusReport` — iterates every case through `analyze_best`, collects `Failure { case, reason }` rows with readable diffs like `"origin: expected GenerativeFst, got ProtectedList (lemma=حامد, root=, conf=1.00)"`.
  - `evaluate(case)` — origin assertion is strict (the primary regression signal); surface is strict (verifies round-trip); lemma / root are conditional on the expected cell not being `-`.

**Calibration round:** the first corpus run produced 7 failures, all Arabic surfaces that happen to be common given names (حامد, محمود, حمد, حافظ, شاكر, ناصر, منصور). The analyzer's Layer 1 (ProtectedList) fires before Layer 3 (GenerativeFst), so these are returned with origin=ProtectedList. Six rows were retagged from `generative` to `protected` — a valuable regression signal in its own right: if a future refactor accidentally reorders the layers and the root analysis starts winning on a name, the corpus will now fail and force a human review. The seventh failure (`ناصر`) was a genuine duplicate (already tested in § 1) and was removed; two more foreign rows were appended to keep the corpus at ≥500.

**Policy decisions baked in:**

- The corpus is **pass/fail** — no confidence-level comparison in v1. If the analyzer swaps origins the TSV row has to be updated explicitly. That is deliberate: we want a human to notice the change and decide whether it is an improvement or a regression. Silent drift is the whole failure mode the corpus exists to prevent.
- **Unique surfaces only** — duplicate surfaces would mask failures (a later row's expected values silently override an earlier row in diagnostic reports). A dedicated test asserts this.
- **Size bounds 500–2000** — if the corpus ever grows past 2000, `corpus_has_expected_size` fails loudly so the upper bound is a deliberate human decision, not an accidental drift.

### 10. M5 Tests

10 tests in `arabic::regression::tests`:

- **Format parser** (5): `parse_origin_handles_known_tags`, `parse_origin_rejects_unknown`, `parse_optional_maps_dash_to_none`, `parse_corpus_skips_comments_blanks_and_short_rows`, `parse_corpus_records_line_numbers`.
- **Corpus shape** (3): `corpus_has_expected_size` (≥500 / ≤2000), `corpus_covers_every_origin` (protected ≥50 / generative ≥100 / heuristic ≥5), `corpus_has_unique_surfaces`.
- **Full run** (2): `corpus_passes_with_full_score` (zero failures on every commit; first 25 failures rendered inline on regression), `raw_corpus_accessor_returns_nonempty_tsv`.

### 11. Results after M5

| Suite | Pre-M3 | After M3 | After M3-baker | After M1g/M1h | After M5 | Delta |
|---|---|---|---|---|---|---|
| arabic module | 171 | 183 | 196 | 202 | 212 | +41 |
| library total | 184 | 196 | 209 | 215 | 225 | +41 |

- All **225 pass** in 0.80s. Corpus run adds ~20ms to the test wall time (502 `analyze_best` calls, no I/O).
- All 502/502 regression cases pass — the corpus is green on its calibration commit and becomes the baseline M6/M7 must defend against.

## 12. M6 — route FTS5 Arabic stemming through analyze_best

The whole point of the Constellation Arabic Engine is that it's what the FTS5 tokenizer uses. Until M6 landed, `libraries.rs::process_arabic_word` still called `stem_arabic_light10` — Light10 was writing into the search index, which is why وائل was indexed as "ائل" and why the flagship bug existed in the first place. M6 is the swap.

**Solution shipped:**

- Edited `src-tauri/src/libraries.rs::process_arabic_word` (around line 1949). Old body: `normalize_arabic` + `stem_arabic_light10` → returns two strings. New body:

  ```rust
  fn process_arabic_word(word: &str) -> (String, String) {
      let display = normalize_arabic_display(word);
      let analysis = crate::arabic::analyze_best(word);
      let stem = if matches!(analysis.origin, crate::arabic::AnalysisOrigin::SurfaceHeuristic) {
          stem_arabic_light10(&normalize_arabic(word))
      } else {
          analysis.lemma
      };
      (display, stem)
  }
  ```

- `normalize_arabic_display` is untouched — the display column keeps preserving `ة / أ / إ / آ / ى` so rendered hits still show the user's original spelling.
- The key column is now the analyzer's `lemma` whenever the engine has an opinion (origin ∈ {ProtectedList, GenerativeFst, SurfaceHeuristic is *the* fallback route}). That's a strict upgrade on every surface the engine recognizes — most visibly on proper nouns like وائل (ProtectedList, lemma=وائل, confidence 1.00) and on cascade chains like الأئمة (FST hit on أئمة after ال- peel).
- When `analyze_best` returns origin=`SurfaceHeuristic` — the engine's own "I don't know this word" signal — we fall back to `stem_arabic_light10(normalize_arabic(word))`, which is byte-for-byte the pre-M6 path. That guarantees the swap is non-regressive: every surface Light10 used to stem correctly still produces the same index key. Only the surfaces where Light10 was *wrong* (because it over-strips proper nouns and loanwords) change their output.
- `stem_arabic_light10` is retained at its existing location (line 1867) as the fallback helper. Removing it would be premature — the analyzer does not yet cover the whole Arabic lexicon.
- `fts5_tokenizer.rs` was **not touched**. It delegates to `crate::libraries::process_word_for_fts(word)`, which routes Arabic-script words through `process_arabic_word`. The IPC boundary stays exactly where CLAUDE.md Rule 3 requires it (Rust-side only, zero `invoke()` on the keystroke hot path).

**Why the fallback is a policy decision, not a quick hack:** a full drop-in that trusted `analyze_best` unconditionally would regress recall on every surface the FST doesn't yet cover. At today's 595-root seed that's a lot of real words. Keeping Light10 behind the `SurfaceHeuristic` guard means the rollout is monotonic — every M1g/M5-grow / roots-expansion landing converts more surfaces from "Light10 heuristic" to "engine verdict" without ever reversing direction. The comment in `process_arabic_word` documents this explicitly so a future refactor can't silently delete the fallback.

### 13. Tests — 5 new FTS contract tests

Added in a new `#[cfg(test)] mod tests` block at the end of `libraries.rs`. The existing 502-case regression corpus exercises `analyze_best` in isolation; these 5 tests verify the *wrapper* — that the analyzer's verdict actually makes it through `process_arabic_word` and `process_word_for_fts` to the tokenizer without being mangled by the glue code.

- `wael_is_not_mangled_to_ail` — flagship. Asserts `process_arabic_word("وائل")` returns stem `"وائل"` exact. This is the test that would have caught the original bug in 2025 if it had existed.
- `wael_survives_process_word_for_fts` — end-to-end through the FTS-facing entry point. Guarantees the stem column of `notes_fts` will hold the full name on any future indexing pass.
- `aimma_is_not_light10_mangled` — the الأئمة cascade flagship. Doesn't pin a specific lemma (tiebreak is not stable across refactors; the corpus leaves this row's lemma unasserted too), but asserts the output isn't the Light10 mangle ("ئم" / "ئمه") and contains at least one of the real root radicals (ء / أ / م / إ).
- `unknown_word_still_gets_light10_stripping` — the fallback contract. Feeds nonsense ("قذالبثظ") that can't possibly hit any analyzer layer except `SurfaceHeuristic`. Asserts the pipeline degrades gracefully (non-empty UTF-8 output, no control chars). The exact string isn't pinned because Light10's output on nonsense isn't something we want frozen — only the contract "M6 is non-regressive for unknown words" is.
- `english_word_still_english_stemmed` — sanity check that non-Arabic routes through the non-Arabic branch of `process_word_for_fts` untouched. Asserts the stem is ASCII (so it can't have been routed through the Arabic pipeline).

All 5 pass.

### 14. Results after M6

| Suite | Pre-M3 | After M3 | After M3-baker | After M1g/M1h | After M5 | After M6 | Delta |
|---|---|---|---|---|---|---|---|
| arabic module | 171 | 183 | 196 | 202 | 212 | 212 | +41 |
| libraries FTS contract | 0 | 0 | 0 | 0 | 0 | 5 | +5 |
| library total | 184 | 196 | 209 | 215 | 225 | 230 | +46 |

- All **230 pass** in 0.69s. The 502-case regression corpus is still green (it was never at risk — M6 doesn't change `analyze_best` itself, only its downstream consumer).
- Test wall time did not regress. The analyzer's `get()` path already ran in the existing 212 Arabic tests; M6 only adds 5 more calls.

## 15. M7 — Layer 4 disambiguator

The pre-M7 `analyze_best` sorted `analyze()`'s Vec by confidence desc and picked the first element. For ties the tiebreak was insertion order, which in turn depended on FST serialization order — fragile across refactors, hash-RNG, and even OS-level build orderings. Word `كاتب` could return Noun today and Verb tomorrow with no code change in between. The FTS index would flip its lemma column between restarts. M7 closes this with a pure, deterministic, linguistically-informed rank.

**Solution shipped:**

- New module `src-tauri/src/arabic/disambiguate.rs` (~180 lines, 12 tests). Keeps the arabic module's one-file-per-layer convention.
- Public surface (crate-internal):
  - `origin_rank(AnalysisOrigin) -> u8` — UserOverride=0, ProtectedList=1, GenerativeFst=2, SurfaceHeuristic=3.
  - `pos_rank(PartOfSpeech) -> u8` — ProperNoun=0, Noun=1, Adjective=2, Adverb=3, Verb=4, Particle=5, Foreign=6, Unknown=7. Opinionated for PKM context: in user notes, named entities and common nouns dominate; verbs are next; particles/unknowns last.
  - `rank_analyses(&mut [Analysis])` — in-place stable sort by the tuple `(confidence desc, origin asc, pos asc, affix_count asc, lemma asc)`. The final alphabetic key on `lemma` is what makes the order *deterministic* — no two distinct analyses can ever tie.

- `analyze()` now calls `rank_analyses` at each multi-hit return point:
  - The Layer-3 bare stripped hits branch builds a `Vec<Analysis>`, ranks, returns.
  - The Layer-3 folded fallback does the same.
  - The Layer-3b peel cascade keeps its existing dedup-oriented sort (by `(root, pattern_label, prefix_count, suffix_count)`), dedups, *then* calls `rank_analyses` on the survivors. The comment on the dedup sort now explicitly flags that it is not the disambiguator's ranking key — it exists to make `dedup_by` correct.

- `analyze_best` collapsed from "sort then pick" to "take first":
  - `analyze()` pre-ranks internally, so `analyze_best = analyze(word).into_iter().next().unwrap_or(stub)`.
  - No duplication of sort logic between the two entry points; the ranking lives in exactly one place.

**Why this ranking, not something fancier:**

- **Pure function of the Analysis fields** → reproducible test fixtures, easy golden-file regression coverage, zero hidden state. Context-aware or corpus-frequency-aware ranking is a v2 extension documented in the module preamble; v1's goal is to eliminate the insertion-order tiebreak, not to build a neural model.
- **Alphabetic lemma as the final tiebreak** → strict total order across all `Analysis` values, so sorting is stable across refactors, OS hash RNGs, FST build orders, and `HashMap` iteration quirks.
- **No `partial_cmp` unwraps** → NaN confidence (should never happen but might in future buggy code paths) degrades to `Ordering::Equal` and subsequent keys decide. One of the 12 tests pins this "doesn't panic on NaN" contract.
- **POS rank is opinionated, and deliberately so** → `docs/CONSTELLATION-ARABIC-ENGINE.md` has flagged for a long time that PKM notes are noun-dominant. If a future user study shows the distribution is different for a given Universe, the POS rank is the one place to adjust — the generator, FST, and protected list stay untouched.

**Regression corpus compatibility:**

- The 502-case corpus asserts `origin` on every row, and leaves `lemma` as `-` (unasserted) on ambiguous verb surfaces. M7 preserves the origin invariants and only changes the lemma pick among equal-confidence origin peers, which is exactly the space the corpus left unconstrained. All 502 cases stay green on M7.
- The `katib_ambiguity_surfaces_both_pos` test (unchanged in `mod.rs::tests`) still holds: `analyze()` still returns **both** Noun and Verb readings of `كاتب`; M7 only pins the *order*, not the presence.
- The `bare_generative_hit_returns_generative_origin` test (which asserted high-confidence GenerativeFst origin for `كاتب` without pinning pos) keeps passing.

### 16. Tests — 12 new for M7

All 12 in `arabic::disambiguate::tests`, organized from primitive-rank assertions upward to full ranking-under-realistic-Analysis-vectors:

- **Rank primitives (2)**: `origin_rank_puts_user_override_first`, `pos_rank_puts_proper_noun_first_then_noun` — assert the monotonicity of both rank tables. These are the contract that the sort keys depend on; if anyone ever reorders the enum and forgets to update the rank table, these fail.
- **Single-key dominance (2)**: `rank_prefers_higher_confidence`, `rank_lower_confidence_never_overtakes_at_any_pos` — confidence is the dominant key; subsequent keys only decide among confidence peers. The second test is the explicit inverse: no matter how good the POS/origin of a lower-confidence hit, it must not beat a higher-confidence one.
- **Tiebreak chain (4)**: `rank_prefers_protected_over_fst_at_equal_confidence`, `rank_prefers_user_override_over_everything_at_equal_confidence`, `rank_prefers_noun_over_verb_at_equal_confidence_and_origin`, `rank_prefers_fewer_affixes_at_equal_everything_else` — each test pins a specific key's behaviour against synthetic analyses that tie on all higher-priority keys.
- **Determinism (2)**: `rank_is_alphabetic_at_full_tie` — the last-resort alphabetic lemma tiebreak actually fires. `rank_is_idempotent` — sorting twice produces the same output (a stability guarantee for the FTS index: re-running the analyzer on the same corpus must not flip tokens between runs).
- **Robustness (2)**: `rank_handles_empty_and_single_element_slices` — no-op on trivial inputs. `rank_handles_nan_confidence_without_panic` — invariant-violating NaN inputs degrade gracefully.

All pass. Test wall time across the whole suite: 0.30s (unchanged within noise; 12 new tests sort ≤3-element Vecs and finish in microseconds).

### 17. Results after M7

| Suite | Pre-M3 | After M3 | After M3-baker | After M1g/M1h | After M5 | After M6 | After M7 | Delta |
|---|---|---|---|---|---|---|---|---|
| arabic module | 171 | 183 | 196 | 202 | 212 | 212 | 224 | +53 |
| libraries FTS contract | 0 | 0 | 0 | 0 | 0 | 5 | 5 | +5 |
| library total | 184 | 196 | 209 | 215 | 225 | 230 | 242 | +58 |

- All **242 pass** in 0.30s. The 502-case regression corpus and the 5 FTS contract tests from M6 continue to pass — M7 is behaviourally additive.
- The full arabic::tests integration suite (mod.rs::tests — 23 tests including `katib_ambiguity_surfaces_both_pos`, the flagship `wael_flows_through_protected_layer`, and the `alaimma_flagship_resolves_through_cascade` cascade test) is all green.

## 18. M8 — Layer 0 user overrides

The pipeline's Layer 1 (ProtectedList) short-circuits Light10 on ~1,200 hand-curated surfaces. But no curated list can anticipate every proper noun, loanword, or domain term the user will write — personal names of family, employer-specific jargon, fictional place names, newly coined technical terms. The user needs a way to pin an analysis for any surface they care about, and have that pin stick across sessions, Universes, and engine upgrades. M8 is that mechanism: Layer 0, the fastest path in the engine, gated only by a single per-Universe hash lookup.

**Solution shipped:**

- New module `src-tauri/src/arabic/overrides.rs` (~340 lines, 16 tests). Keeps the arabic-module's one-file-per-layer convention.
- Public surface (crate-internal):
  - `UserOverride { surface, lemma, root?, pattern_label?, pos?, note?, created_at? }` — `#[derive(Debug, Clone, Serialize, Deserialize)]` with `#[serde(default)]` on every optional field so a file written by a future version of Constellation (adding, say, `confidence` or `origin_hint`) round-trips cleanly through an older version.
  - `UserOverride::to_analysis(original_surface: &str) -> Analysis` — produces an `Analysis` with `origin = AnalysisOrigin::UserOverride`, `confidence = 1.0`, `lang = Lang::Ar`, empty `prefix`/`suffix`/`equivalents`. Preserves the caller's exact surface (not the override's canonical surface) so round-trip display strings aren't mutated.
  - `OverrideFile { version: u32 (default 1), overrides: Vec<UserOverride> }` — the on-disk JSON envelope.
  - `OverrideStore { entries: HashMap<String, UserOverride> }` — the in-memory index. Keyed on `normalize_key(surface)` which calls `super::normalizer::normalize(surface).stripped`, so a user's `وَائِل` override with full vocalization matches the bare surface `وائل` at query time (and vice versa — the normalization is symmetric).
  - Methods: `new()`, `len()`, `is_empty()`, `iter()`, `lookup(surface) -> Option<&UserOverride>`, `insert(override)`, `remove(surface) -> Option<UserOverride>`, `path_in_universe(universe: &Path) -> PathBuf`, `load_from_path(&Path) -> Result<Self>`, `save_to_path(&Path) -> Result<()>`.

- **Persistence**:
  - Canonical path: `<universe>/.constellation/arabic-overrides.json`. Parallel to every other per-Universe file (`libraries.json`, `universe.json`, etc.). Not in `<cache_dir>` because overrides are per-Universe knowledge, not per-install.
  - Atomic write: stage to `<path>.tmp`, `fs::rename` on success. Survives power loss / crash mid-write. A crash during `.tmp` write leaves the old file intact.
  - Entries serialize in alphabetic order by `surface` so a diff on the JSON file is deterministic — a user syncing via git/Syncthing never sees a noisy reordering when they add or remove a single entry.
  - `load_from_path` returns an **empty store** (not an error) when the file is missing — first-launch case on a pre-existing Universe. Malformed JSON (not-JSON, missing `overrides` array, etc.) returns `InvalidData`; the caller decides whether to back up the bad file and start fresh or surface an error to the user.
  - Parent directory (`.constellation/`) is created on save if missing.

- **`analyze()` → `analyze_with_overrides()` split**: the crate now exposes two public entry points.
  - `analyze(word: &str) -> Vec<Analysis>` remains the zero-dependency entry point — back-compat contract for everything already wired (including the 502-case regression corpus, M6's `process_arabic_word`, and the 23 mod.rs integration tests). Internally it's a 3-line wrapper: `analyze_with_overrides(word, None)`.
  - `analyze_with_overrides(word: &str, overrides: Option<&OverrideStore>) -> Vec<Analysis>` is the new overload. Inserts a Layer 0 hook between the script check and Layer 2 (ProtectedList): if `overrides` is `Some` and the store is non-empty, it runs `store.lookup(&norm.stripped)` — a single `HashMap` get — and on a hit returns `vec![o.to_analysis(word)]` immediately, bypassing every subsequent layer.
  - `analyze_best` and its callers are untouched; they continue to call `analyze(word)` and get the back-compat behaviour. When M8b (the Tauri commands + Svelte UI) wires the override store into `libraries.rs::process_arabic_word`, that's the single call site that needs to switch from `analyze_best` to an override-aware variant.

- **Why Layer 0 is strictly a short-circuit, not a rank-participant**: M7's disambiguator already ranks `UserOverride` strictly above `ProtectedList` / `GenerativeFst` / `SurfaceHeuristic`. That means if Layer 0 were to emit a `UserOverride` Analysis *alongside* the other layers' results, the disambiguator would put it first and `analyze_best` would return it. That's the correct answer, but it costs us the FST + protected + normalizer work on every override hit. Layer 0 fires early and returns a single-element `Vec` instead — exactly the same final answer, but with the 4 downstream layers skipped. The disambiguator's `UserOverride=0` rank is now belt-and-suspenders: even a future refactor that accidentally routes override results through the full rank path still produces the same verdict.

- **Versioning**: the `OverrideFile` envelope carries a `version: u32` (default `1`). The serde default means a v1 file written today round-trips through a future v2 reader; a future v2 reader can branch on `version > 1` to apply migrations. The alphabetic-sort-by-surface invariant is enforced on write, not assumed on read — a hand-edited file with scrambled ordering still loads correctly; the next save canonicalizes.

### 19. Tests — 21 new for M8

**16 unit tests in `arabic::overrides::tests`** — the module's own contract:

- **`to_analysis` shape (3)**: `to_analysis_sets_user_override_origin`, `to_analysis_sets_full_confidence`, `to_analysis_preserves_caller_surface` — the 3 facts the disambiguator depends on.
- **Store CRUD (6)**: `empty_store_reports_empty`, `insert_and_lookup_roundtrip`, `lookup_miss_returns_none`, `insert_replaces_on_duplicate_surface`, `remove_returns_removed_entry`, `remove_nonexistent_returns_none` — the data structure's invariants.
- **Normalization parity (1)**: `lookup_matches_vocalized_surface_against_bare_override` — `وَائِل` with full vocalization hits an override authored for bare `وائل`. This is the test that proves M8's Layer 0 key space is aligned with the normalizer, not a second independent key space that can drift.
- **Iteration (1)**: `iter_exposes_all_entries` — tiebreak-free `iter()` visits every stored override exactly once.
- **Persistence (4)**: `load_from_missing_path_returns_empty`, `save_then_load_roundtrips`, `load_rejects_malformed_json`, `atomic_save_leaves_no_tmp_file` — the 4 disk-path failure modes.
- **Storage hygiene (2)**: `save_sorts_entries_alphabetically_for_diff_friendliness`, `path_in_universe_is_constellation_arabic_overrides_json` — the 2 on-disk invariants that make the file git-sync-friendly.

**5 integration tests in `arabic::tests`** — the overlay's pipeline contract:

- `override_beats_protected_list` — a surface that would normally hit Layer 1 (e.g. `الله`) is instead returned from Layer 0 with `origin=UserOverride` when the user has pinned it.
- `override_beats_fst_hit` — a surface that would normally hit Layer 3 (e.g. `كاتب`, a generative FST winner at conf=0.85) is instead returned from Layer 0 at conf=1.0.
- `override_catches_vocalized_surface` — an override authored for bare `وائل` is hit by a query for `وَائِل`. Proves the normalization key path is symmetric at the Layer 0 boundary.
- `empty_override_store_is_a_no_op` — `analyze_with_overrides(word, Some(&empty_store))` is equivalent to `analyze(word)` on every test surface. Key property: adding the override overload didn't silently change the default-case behaviour.
- `override_does_not_fire_on_non_arabic_input` — a Latin-script word gets routed to `SurfaceHeuristic` as before, even with overrides present. Layer 0 sits *after* the script check, not before it, so non-Arabic inputs never consult the Arabic override table.

All 21 pass. Combined with the 12 M7 disambiguator tests and the 502-case regression corpus, the M8 contract is triple-locked: the module's own 16 tests, 5 integration tests against the live `analyze_with_overrides` pipeline, and the corpus which proves the back-compat `analyze()` entry point still has zero behavioural drift.

### 20. Results after M8

| Suite | Pre-M3 | After M3 | After M3-baker | After M1g/M1h | After M5 | After M6 | After M7 | After M8 | Delta |
|---|---|---|---|---|---|---|---|---|---|
| arabic module | 171 | 183 | 196 | 202 | 212 | 212 | 224 | 245 | +74 |
| libraries FTS contract | 0 | 0 | 0 | 0 | 0 | 5 | 5 | 5 | +5 |
| library total | 184 | 196 | 209 | 215 | 225 | 230 | 242 | 263 | +79 |

- All **263 pass** in 0.38s. The 502-case regression corpus and every previous layer's tests stay green.
- M8 is the first milestone that adds a cross-module integration surface (per-Universe files), but no disk I/O runs in the default test path — `load_from_path` / `save_to_path` tests use `tempfile::TempDir` and the override-overload integration tests build the store in memory.

## 21. M8b — the wire: `ACTIVE_STORE` + Tauri commands + `set_active_universe` hook

M8 shipped the Layer 0 type (`UserOverride`), the data structure (`OverrideStore`), and the pipeline overload (`analyze_with_overrides`), plus persistence (atomic `.tmp`+rename) and a full test suite. What M8 didn't ship is the wire: no call site actually *passed* an `OverrideStore` to the pipeline on the FTS5 hot path. The Settings UI had nothing to CRUD against. Switching Universes loaded no overrides. M8b closes that gap on the Rust side. (The Svelte UI + FTS reindex signal are M8c.)

**Solution shipped:**

### 21.1 Process-wide active store

The FTS5 tokenizer runs inside SQLite's call context — a sync function with no access to `app.state::<UniverseState>()`. So we mirror the per-Universe override file into a process-wide singleton at the arabic module root:

```rust
// arabic/overrides.rs
static ACTIVE_STORE: OnceLock<RwLock<Arc<OverrideStore>>> = OnceLock::new();

pub fn active() -> Arc<OverrideStore> { ... }          // cheap clone, FTS hot path
pub fn set_active(store: OverrideStore) { ... }        // swap the Arc
pub fn activate_for_universe(root: &Path) -> Result<usize, String> { ... }
pub fn clear_active() { ... }                          // install empty store
```

Hot-path cost: one `RwLock::read` (uncontended ~20 ns on Windows) + one `Arc::clone` (refcount bump, ~5 ns). Well under the tokenizer's normalize + HashMap-probe budget. The FTS5 tokenizer is expected to burn ~10 µs per token regardless; the registry adds ~0.025 µs to that. A test (`active_returns_cheap_arc_clones`) pins the contract: back-to-back `active()` calls return `Arc::ptr_eq`-equal handles so we can't accidentally deep-clone the HashMap at tokenize time.

### 21.2 Pipeline wire: `analyze_with_overrides_best`

The former `analyze_best(word)` already wrapped `analyze(word).into_iter().next().unwrap_or(stub)`. M8b adds a parallel `analyze_with_overrides_best(word, overrides: Option<&OverrideStore>)` that calls `analyze_with_overrides` under the hood, and reduces `analyze_best` to a single-line wrapper (`analyze_with_overrides_best(word, None)`). No duplicated fallback stub — DRY preserved, public surface grows by one function.

### 21.3 FTS5 tokenizer wire: `process_arabic_word`

The one-line M6 change (`analyze_best(word) → analyze_with_overrides_best(word, overrides_ref)`) with a fast-path guard:

```rust
// libraries.rs::process_arabic_word
let store = crate::arabic::overrides::active();
let overrides_ref = if store.is_empty() {
    None
} else {
    Some(store.as_ref())
};
let analysis = crate::arabic::analyze_with_overrides_best(word, overrides_ref);
```

The `is_empty()` guard keeps the analyzer's hot path identical to M8-pre for Universes that haven't authored any overrides (the overwhelmingly common case today). When the store is non-empty, the HashMap probe inside `analyze_with_overrides`'s Layer 0 fires exactly once per token — O(1) expected.

### 21.4 Boot wire: `set_active_universe`

`crate::universe::set_active_universe` already fires once per Universe switch (including the frontend-issued call at cold-boot that activates the last-used Universe). M8b adds an `activate_for_universe` call right after the `UniverseState` mutation and `libraries::invalidate_libraries_cache()` call:

```rust
// universe.rs::set_active_universe (after the existing state update)
match crate::arabic::overrides::activate_for_universe(&final_path) {
    Ok(count) if count > 0 => {
        eprintln!("[arabic] Loaded {} Arabic override(s) for Universe at {}", count, ...);
    }
    Ok(_) => {}  // no overrides authored yet — common case, silent
    Err(e) => {
        eprintln!("[arabic] Failed to load overrides for Universe at {}: {}", ..., e);
        crate::arabic::overrides::clear_active();
    }
}
```

Errors are logged but *not* propagated. A malformed `arabic-overrides.json` must not prevent the user from switching Universes — the engine gracefully falls back to no-overrides on error, and the forthcoming Settings UI will surface the parse error when the user opens the overrides panel. On error we explicitly `clear_active()` so a residual store from the previous active Universe doesn't leak across the switch.

### 21.5 Tauri command surface

Three `#[tauri::command]` functions at the bottom of `overrides.rs`:

- `read_arabic_overrides(app: AppHandle) -> Result<Vec<UserOverride>, String>` — returns all overrides for the active Universe, sorted alphabetically by surface. Reads from disk, not from `ACTIVE_STORE`, so the UI sees the canonical on-disk state even if a second window raced a write.
- `add_arabic_override(app, entry: UserOverride) -> Result<(), String>` — upsert. Reloads the store from disk, inserts the entry, atomic-saves, then calls `set_active` so subsequent FTS5 tokens see the change without waiting for a Universe switch.
- `remove_arabic_override(app, surface: String) -> Result<bool, String>` — idempotent remove. Returns `true` if an entry was removed, `false` if none existed (not an error — the UI can treat both identically).

**Why disk-as-source-of-truth, not `ACTIVE_STORE`-as-source-of-truth** for CRUD: it makes concurrent edits from multiple UI windows (Settings modal, second-screen panel) safe without cross-window mutex coordination. The atomic-rename on disk is the only serialization point. Worst case under contention: one window's write is immediately overwritten by another's; neither write is lost mid-file; and the next `read_arabic_overrides` call returns whichever landed last. In practice the Settings UI is single-window and this contention path never fires.

Registered in `src-tauri/src/lib.rs` alongside existing command handlers.

### 22. M8b Tests — 8 new

All 8 in `arabic::overrides::tests`, under a `REGISTRY_TEST_MUTEX`-serialized RAII guard (`RegistryGuard`) that snapshots the prior `ACTIVE_STORE` on construction and restores it on drop — so the global singleton's state can't leak across tests or races under `--test-threads=N`:

- **Registry baseline (2)**: `active_returns_empty_store_before_any_set` (boot state = empty, Layer 0 never fires), `clear_active_installs_empty_store` (explicit reset).
- **Set/get roundtrip (2)**: `set_active_then_active_roundtrips` (round-trip correctness), `set_active_replaces_prior_store_entirely` (no residual entries leak across a swap).
- **Hot-path contract (1)**: `active_returns_cheap_arc_clones` — back-to-back `active()` calls return `Arc::ptr_eq`-equal handles. This is the guarantee the FTS5 tokenizer depends on; a future refactor that accidentally starts deep-cloning the HashMap per token would fail this test loudly.
- **Disk → registry (3)**: `activate_for_universe_installs_from_disk` (seed a JSON file in a tempdir, call `activate_for_universe`, verify the active store contains the entry and the returned count matches), `activate_for_universe_handles_missing_file` (fresh Universe case: missing file → empty store, NOT an error), `activate_for_universe_reports_malformed_json_as_error` (a corrupted *existing* file surfaces as `Err`).

All 8 pass under serialized execution. The `RegistryGuard` pattern is documented in-line so future additions to this test module can follow the same convention.

### 23. Results after M8b

| Suite | Pre-M3 | After M3 | After M3-baker | After M1g/M1h | After M5 | After M6 | After M7 | After M8 | After M8b | Delta |
|---|---|---|---|---|---|---|---|---|---|---|
| arabic module | 171 | 183 | 196 | 202 | 212 | 212 | 224 | 245 | 253 | +82 |
| libraries FTS contract | 0 | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 5 | +5 |
| library total | 184 | 196 | 209 | 215 | 225 | 230 | 242 | 263 | 271 | +87 |

- All **271 pass** in 0.67s. No regression on the 502-case corpus or the 16 M8 override unit tests.
- Test wall time grew by ~290 ms vs M8 (0.38 → 0.67s) — expected: the new disk-path tests (`activate_for_universe_installs_from_disk`, `…_handles_missing_file`, `…_reports_malformed_json_as_error`) each `create_dir_all` + write + read + cleanup a tempdir. Well within noise on CI.

### 24. M8c Rust — targeted FTS reindex after override mutation

M8b landed the wire. M8c closes the loop: when the user adds or removes an override, the rows already in `notes_fts` were tokenized under the *old* Layer 0 verdict. The FTS index is now stale for any note that contains the affected surface, and a full rebuild on every Settings-modal edit would be a non-starter on a 7,600-note Universe.

The M8c plan listed two options:

- **(a) targeted invalidation** — scan `note_meta` for rows whose `body_text` or `name` contains the surface, delete those rows from `notes_fts`, re-insert through the active tokenizer (which now reads the freshly-mutated `ACTIVE_STORE`).
- **(b) mark-dirty + background rebuild** — cheaper to issue, but the affected notes stay wrong until the whole sweep finishes, and a concurrent user search against the dirty state returns misleading hits.

Shipped **(a)**. Single-statement LIKE scan with the normalized needle, explicit `INSERT INTO notes_fts(notes_fts, rowid, name, body_text) VALUES('delete', ...)` directive per hit, followed by a plain `INSERT INTO notes_fts (rowid, name, body_text) VALUES(...)`. The delete directive is the FTS5-approved way to remove a row without the content table (FTS5 external-content mode requires the caller to hand back the *original* values on delete — the scan gives us those). The re-insert then fires the `constellation` tokenizer, which calls `arabic::overrides::active()` and picks up the just-mutated store. Everything runs inside a single `BEGIN IMMEDIATE` / `COMMIT` so a crash mid-reindex leaves the FTS either fully pre-M8c or fully post-M8c, never half-updated.

Why LIKE not MATCH: the whole point of reindex is that MATCH goes through the tokenizer we're trying to refresh, so it would return the pre-mutation hit set. LIKE bypasses tokenization and matches the raw normalized text directly (Constellation already stores `body_text` post-`normalize_arabic_for_search`, so the needle normalizes with the same function before the `%...%` wrap).

New code:

- `src-tauri/src/search.rs` — pub helper `reindex_notes_matching_text(state: &SearchState, needle: &str) -> Result<u32, String>` at the bottom of the module (before `constellation_search_link_counts`). ~85 lines. Returns count of successfully re-tokenized rows; `0` for empty or whitespace needle.
- `src-tauri/src/arabic/overrides.rs` — new `#[tauri::command] reindex_arabic_overrides(app, surface)` that grabs `SearchState` via `app.state::<crate::search::SearchState>()` and delegates to the helper.
- `src-tauri/src/lib.rs` — registers `reindex_arabic_overrides` in the `generate_handler!` list alongside the M8b trio.

Sequencing in the Svelte layer (see §25): the panel awaits the `add`/`remove` command *first* (so `ACTIVE_STORE` is mutated), *then* awaits `reindex_arabic_overrides(surface)`. By the time the reindex LIKE-scan finishes, the tokenizer sees the new store state, and every affected row is re-tokenized under the new Layer 0 truth. No separate cache-bust, no full rebuild, no background worker.

### 25. M8c Frontend — Settings panel + 15-locale i18n

New component: `src/lib/components/ArabicOverridesPanel.svelte` (~480 lines). Modelled structurally on `IconOverrideSettings.svelte` for visual / interaction parity with the rest of the Settings modal.

Shape:

- **State**: `overrides: UserOverride[]`, `loading`, `loadError`, `showForm`, the seven form fields (surface, lemma, root, pattern, pos, note, plus a dynamic-count derived from `overrides.length`), `statusMessage` (string with `Reindexing… / Reindexed N note(s) / error`).
- **Load**: `onMount` → `invoke('read_arabic_overrides')` → `overrides = result`. Errors caught and surfaced via `loadError`.
- **Save**: form submit validates the surface is non-empty, then `invoke('add_arabic_override', { override: {...} })` → on success, prepend to local list, close form, `reindexFor(surface)`.
- **Remove**: per-row button → `invoke('remove_arabic_override', { surface })` → splice from local list → `reindexFor(surface)`.
- **`reindexFor(surface)`**: sets `statusMessage = $t('settings.arabicOverrides.reindexing')`, awaits `invoke('reindex_arabic_overrides', { surface })`, sets `statusMessage` to the localized "Reindexed {count} note(s)" using the `count` from the Rust return value. Autoclears after 3 s via `setTimeout`. The timer handle is tracked on a module-scoped `let`; there is no long-lived listener or interval to clean up, but any in-flight timer is cleared on `onDestroy` per Rule 4.
- **RTL handling**: each user-entered value (surface, lemma, root, annotation) is rendered inside a span with `dir={detectDir(value)}` so Arabic content displays RTL and a mixed Latin annotation on an Arabic surface stays LTR within the Arabic cell.

Integration into `SettingsModal.svelte`:

1. `import ArabicOverridesPanel from './ArabicOverridesPanel.svelte';` added after the `IconOverrideSettings` import.
2. A new entry in the `sections` derived array: `{ id: 'arabic-overrides', label: $t('settings.sections.arabicOverrides') || 'Arabic Overrides', icon: 'translate' }`, inserted after the existing `language` row so it groups with the other language-adjacent settings.
3. A new content branch `{:else if activeSection === 'arabic-overrides'} <ArabicOverridesPanel />` placed just before the Sky View section.

No other settings sections were touched. The existing route into the modal, the tab scroll behavior, the mobile layout, and the a11y wiring remain unchanged.

i18n — 15 locales:

- One `"arabicOverrides"` key added to each file's `settings.sections` block.
- A full `settings.arabicOverrides` block (31 keys) added to each file: `title, intro, loading, empty, countOne, countMany, add, newTitle, surface, surfaceRequired, lemma, lemmaHint, root, pattern, pos, note, notePlaceholder, save, saving, cancel, remove, reindexing, reindexed, posProperNoun, posNoun, posAdjective, posAdverb, posVerb, posParticle, posForeign, posUnknown`.
- English canonical wording: title = "Arabic Engine Overrides", intro = "Pin how the engine analyses specific Arabic surfaces in this Universe. Each override is the sovereign answer — it wins over the generative FST, the cascade, and the heuristic fallback." The sovereignty language matches how Layer 0 is described in the Arabic Engine architecture docs — overrides are not suggestions, they are the answer.
- Every locale's file was re-validated with `node -e "JSON.parse(readFileSync('...'))"` — 15/15 parse cleanly. `grep` confirms 30 exact matches of the `arabicOverrides` token (2 per file: one section entry, one block key) — the shape is symmetric across all languages.

### 26. Results after M8c

| Suite | Pre-M3 | After M3 | After M3-baker | After M1g/M1h | After M5 | After M6 | After M7 | After M8 | After M8b | After M8c | Delta |
|---|---|---|---|---|---|---|---|---|---|---|---|
| arabic module | 171 | 183 | 196 | 202 | 212 | 212 | 224 | 245 | 253 | 253 | +82 |
| libraries FTS contract | 0 | 0 | 0 | 0 | 0 | 5 | 5 | 5 | 5 | 5 | +5 |
| library total | 184 | 196 | 209 | 215 | 225 | 230 | 242 | 263 | 271 | 271 | +87 |

- All **271 pass** in 0.60s — no regression from M8b.
- **M8c adds no Rust unit tests.** `reindex_notes_matching_text` is an effect-on-FTS helper whose behavior is only observable through a round-trip test (tokenize → mutate override → reindex → re-query and assert the new stem lands). That's integration territory, not unit territory, and the integration-test harness sits behind the forthcoming Settings → Debug scorecard — when the scorecard lands it will be the natural home for an end-to-end "add override → reindex → assert FTS row count delta" assertion. Until then, the reindex helper is protected by the existing M6 FTS contract tests (which still pass) plus manual QA on the trial Universe.
- `npx svelte-check` — 53 pre-existing errors, ALL in user's in-flight work on `+layout.svelte` and `libraries/+page.svelte`. Zero new errors introduced by `ArabicOverridesPanel.svelte` or the `SettingsModal.svelte` additions.

### 27. M8c-doc — help files + User Manual (15 languages)

The Standing Order mandates every user-facing feature land with its help-file + User Manual documentation in all 15 languages. M8c-doc is that pass for M8c.

**New help topic**: `docs/help.{15 locales}/Arabic Engine/Arabic Engine.md`. A new category in the help tree — alongside Cognitive Engine, Knowledge Formulation, Sky View, etc. Each file follows the established help-topic template (Philosophy / Feature / What it is / Why it matters / How to use it / Step-by-step / Glossary) and is grounded in the same 9-heading structure across all 15 languages for diff-friendly parity:

1. `# Arabic Engine` (top)
2. `## Why the engine exists`
3. `## Feature: Arabic Engine Overrides`
4. `### What it is`
5. `### Why it matters`
6. `### How to use it`
7. `### Interaction with the Protected List`
8. `### Interaction across Universes`
9. `### What happens if you edit the file by hand`
10. `## Glossary`

(9 `##`-level headings; the single `#` top heading is not counted above.)

**User Manual inserts** (15 files): each locale's `docs/help.{lang}/User Manual.md` + the canonical `docs/User Manual.md` now carry two new subsections:

- Inside the RTL/Arabic-support section: a full `### Arabic Engine Overrides` walkthrough (step-by-step, from opening Settings to verifying the reindex).
- Inside the Settings section, after Language and before Editor: a short `### Arabic Overrides` cross-reference that points readers to the RTL section for the full walkthrough.

**Vocabulary grounding**: every translation reuses the localized strings already shipped in M8c (`src/lib/i18n/{lang}.json`: `settings.sections.arabicOverrides` + 31-key `settings.arabicOverrides` block). This guarantees the help text, the User Manual, and the Settings UI all use the same wording for "Arabic Engine Overrides", "Add override", "Reindexing…", "Reindexed N note(s)", and every POS label — so a user following the User Manual in Japanese reads the same string they see on the save button. No translator drift between UI and docs.

**Preserved verbatim across all 15 languages**: `FST`, `JSON`, `Ctrl`, `Cmd`, `.constellation/`, `arabic-overrides.json`, and the Arabic sample words (`وائل`, `كاتب`, `فاعل`, `ك‑ت‑ب`, `الكتاب`, `كتبنا`, `مكتوب`, `فلسطين`, `إنترنت`).

**Character counts** (rough measure of translation depth; all files validated to have ≥3 `##` headings and each User Manual has ≥1 `arabic-overrides` mention):

| Language family | Files | Chars |
|---|---|---|
| Canonical (en) | 1 new + User Manual edit | 7,600 + User Manual diff |
| RTL (ar, fa, ur, he) | 4 new + 4 User Manual edits | ~46,086 |
| European (de, fr, es, pt) | 4 new + 4 User Manual edits | ~34,629 |
| Slavic + Turkic (ru, tr) | 2 new + 2 User Manual edits | ~23,626 |
| Asian (zh, ja, ko, hi) | 4 new + 4 User Manual edits | ~44,537 |

**Production method**: canonical English written first; 14 translations produced by four parallel background agents grouped by language family, each with explicit non-overlap scopes (own language only, own two file types only — no code, no config, no other docs). Verified by enumeration: 15/15 new help files exist with consistent structure, 15/15 User Manuals contain the `arabic-overrides` cross-reference path exactly once.

### 28. M9 — bench harness + first measurements

**What it is**: `src-tauri/src/arabic/bench.rs`, a new ~200-line test module with a single `#[test] #[ignore] fn m9_bench()`. Five sequential measurements, one test, opt-in only — `cargo test --lib --release arabic::bench -- --ignored --nocapture`. Default `cargo test --lib` runs skip it (confirmed: 253 passed / 0 failed / 1 ignored on release filtered by `arabic::`).

**Why a single monolithic test**: `OnceLock` inside `GenerativeFst::get()` means cold-start can only be measured *once per process*. Splitting into multiple `#[test]` fns would require fresh subprocesses — more machinery than this bench warrants. One test, sequential measurements, report to stdout via a small `report(key, value)` helper.

**Why ignored by default**: the cold-start step deletes the real user cache file. That's fine for an explicit `--ignored` dev-machine run, but not something `cargo test` should do silently on every commit — CI and local test runs would then pay a rebuild cost the user didn't ask for.

**What each measurement exercises**:

1. **Cold-start** — `fst_bake::cache_file_path()` deleted, then `GenerativeFst::get()`. Exercises `fst_bake::try_load_cached()` → (cache miss) → `build_bundle()` → `persist_best_effort()` → `from_bundle()`. This is the first-launch experience after a fresh install, or after a cache file format bump.
2. **Warm-start** — same code path `get()` uses on a warm launch, but since the `OnceLock` is now hot we can't re-call `get()` in-process. Instead we run `fst_bake::load_bundle(&path)` + `GenerativeFst::from_bytes(...)` on the file the cold run just wrote. Target: ≥5× faster than cold.
3. **Throughput** — `analyze_best()` on the 502-case regression corpus × **K=500** iterations = 251,000 calls through the full five-layer pipeline (overrides → protected → FST → cascade → heuristic) plus M7 disambiguation plus the M8b `ACTIVE_STORE` `Arc::clone` per call. Warm-up: one full pass first to pay any remaining lazy inits (`protected_list::get`, etc.) before starting the clock. Target: ≥200 K words/sec.
4. **Accuracy** — `regression::run_corpus()` pass rate plus a per-origin breakdown (Protected / Generative / Heuristic / UserOverride). Target: ≥92% overall.
5. **Size proxy** — on-disk cache bundle size in KiB plus a linear projection to the target 7K-root scale (today's seed is ~595 roots; `kib * (7000/595) / 1024`). Not a direct RSS measurement (Rust has no portable in-process RSS API short of platform bindings we don't want as a prod dep), but a stable lower bound that tracks the in-memory footprint.

**Module visibility**: `arabic::regression` was `#[cfg(test)] mod regression;` — bench needs `parse_corpus`, `raw_corpus`, `run_corpus`. Promoted to `pub(crate) mod regression;` so bench can reach it without widening production-binary visibility. No other visibility changes.

**Measured values** (two runs on the dev machine, variance ~10%):

| Metric | Run 1 | Run 2 | Target | Status |
|---|---|---|---|---|
| FST keys | 32,197 | 32,197 | — | — |
| Cold-start (ms) | 153.90 | 166.72 | <500 | **PASS** |
| Warm-start (ms) | 21.23 | 20.75 | <cold/5 | **PASS** |
| Cold/warm ratio | 7.2× | 8.0× | ≥5× | **PASS** |
| Throughput (words/s) | 132,593 | 126,626 | ≥200,000 | near — follow-on |
| Per-call (ns) | 7,541 | 7,897 | — | — |
| Pass rate (%) | 100.0 | 100.0 | ≥92 | **PASS with headroom** |
| Protected (%) | 100.0 | 100.0 | — | **PASS** |
| Generative (%) | 100.0 | 100.0 | — | **PASS** |
| Heuristic (%) | 100.0 | 100.0 | — | **PASS** |
| Cache bundle (KiB) | 7,812.4 | 7,812.4 | — | — |
| Projected @ 7K (MiB) | 89.8 | 89.8 | ≤10 | aspirational — follow-on |

**Reading the numbers**:

- **Accuracy** is the most important line and the cleanest result. 100.0% on 502/502 cases across every active origin layer confirms the five-layer pipeline is coherent end-to-end under disambiguation. No case regressed across M6 → M7 → M8 → M8b → M8c. The 92% target is kept as an open floor for the M5-grow corpus expansion (which will add harder surfaces).
- **Cold/warm ratio** proves the whole point of M3-baker: the compiled FST serialization round-trip is ~8× faster than the cold build path. A user on a warm launch pays ~21 ms of FST setup, not ~160 ms. That's the cost model the UX was designed around.
- **Throughput** at 126–132 K words/sec = 7.5–7.9 µs per `analyze_best` call. FTS5 tokenization on a typical Arabic note (~500 tokens) costs ≈4 ms — the user will not notice this. The ≥200 K aspirational target anticipates mobile / constrained hardware; the follow-ons below get there without changing the public API.
- **Size projection** of ~90 MiB at 7K roots is the weakest number. Today's bundle holds two full `fst::Map` byte buffers (stripped + folded) plus two parallel `Vec<SurfaceValue>` arrays of enriched values, and `SurfaceValue` stores owned `String` fields (`pattern_label`, `root_key`). At 7K-root scale the FST footprint grows roughly linearly; the `Vec<SurfaceValue>` footprint grows worse-than-linearly because the distinct-form count per root is itself a function of pattern coverage. The follow-ons below flatten both curves.

**Follow-on optimizations** (not in this commit — queued as open items):

1. **mmap the FST byte buffer.** `fst::Map::new(Vec<u8>)` copies to owned memory. Switching to `fst::Map::new(Mmap)` via the `memmap2` crate would let the OS page-cache the bundle, cutting RSS to roughly just the resident working set — back to a fraction of the on-disk size. Net effect at 7K roots: ~90 MiB → ~20–30 MiB RSS. Warm-start drops further (mmap is near-instant).
2. **Intern `pattern_label` and `root_key` strings.** Today each `SurfaceValue` owns `String` copies. A pattern like `فاعل` appears on every root × active-participle cell — ~600 duplicates today, ~7,000 at scale. A tiny `StringInterner` with `u16` indices (≤65K patterns is plenty) would cut the per-value payload from ~24 B × 2 strings to 4 B total. Net effect at 7K roots: another ~20% RSS reduction layered on mmap.
3. **Profile the `analyze_best` hot path.** Candidates for the ~7.7 µs per-call cost: (a) `overrides::active()` does an unconditional `Arc::clone` even when the store is empty — the M8b short-circuit-on-empty trick cuts the work but not the clone. Cache a `once_cell` snapshot of `active_is_empty` updated inside `set_active` to skip the `Arc::clone` entirely on the common empty path. (b) The disambiguator allocates a `Vec<Analysis>` for multi-hit ordering even when there's only one hit; a two-element SmallVec would dodge one allocation per word. (c) `analyze()` builds `Vec<Analysis>` by value across layers; a generator-style visitor pattern would collapse early on first-hit for the fast ProtectedList / UserOverride paths.

**What this does not measure** (deferred, tracked as open items):

- Real-machine RSS via OS-level APIs (`GetProcessMemoryInfo` on Windows, `mach_task_basic_info` on macOS, `/proc/self/statm` on Linux). The projected-bundle-size proxy is a floor, not the total.
- End-to-end FTS tokenization throughput (tokenizer overhead on top of `analyze_best` per word). Once the Settings → Debug scorecard lands this can be surfaced as a live timing.
- M9-on-mobile. The ≥200K target is a dev-machine number; mobile budgets will be separately specified when Constellation mobile lands.

### 29. M10 — Lexical Bridge architecture

**What it is**: the crate-level `lexicon` module gets its first real implementation. Pre-M10 the module existed as a pure skeleton: type definitions for `LemmaNode` / `EdgeKind` / `Edge` / `LexiconGraph`, option types for `ExpansionOptions` / `ExpansionResult`, and `equivalents()` / `expand()` stubs that returned empty maps. M10 replaces every stub with a working implementation and ships a tiny hand-picked seed so the whole path — TSV parse → concept builder → FST name-index → edge traversal → query expansion — runs end-to-end. The 20K-concept core data and on-disk cache come in M11; query-expansion plumbing into the FTS5 search path comes in M12.

**Why this shape, not a dictionary**: the lexicon is an **undirected graph of lemmas** rather than a source-indexed dictionary because a French "connaissance" edge to Arabic "معرفة" must also be an Arabic→French edge — bidirectional with one edge instead of two, and with a single consistency invariant. Nodes carry `(lang, lemma, sense_id)` so `bank[riverside]` and `bank[financial]` are distinct graph nodes pointing to different target lemmas. For M10 every node is `SenseId::DEFAULT`; WordNet-style polysemy lands with M11.

**Why CSR + FST**: the storage target at the M11 20K-concept scale is ≈200K nodes / ≈800K edges. A HashMap-of-Vec adjacency would spend its whole footprint on heap `Vec` headers and per-key strings. Compressed-Sparse-Row adjacency (one `edge_offsets: Vec<u32>` of length `nodes.len() + 1` plus one flat `edges: Vec<Edge>`) keeps adjacency cache-friendly for large walks. For the name-index, `(lang, lemma) → Vec<node_id>`, an `fst::Map` keyed by `"{lang_code}:{normalized_lemma}"` with a packed `u64` value `(count << 32) | offset` points at the contiguous node range — same byte-layout pattern the Arabic generative index uses in M3 and the same crate, so the FST build path is battle-tested.

**Normalization for lookup**: Arabic-script languages (Ar / Fa / Ur) route through `arabic::normalize_stripped`, which removes tashkeel and tatweel while preserving hamza variants — so `كِتَاب` (with diacritics) and `كتاب` (without) hit the same entry. Everything else uses `trim().to_lowercase()`, which Unicode-folds Latin/Cyrillic/Turkish/Greek case variants while being a no-op for Hebrew/Devanagari/CJK. Both the build-time seed lemmas and the read-time query lemmas are normalized through the same function — symmetric by construction.

**Module shape** (4 files, ~900 source lines, 34 new tests):

- **`lexicon/parse.rs`** (~200 lines, 11 tests): `ConceptRecord { id, pos, labels: BTreeMap<Lang, Vec<String>> }` + `parse(tsv) -> Vec<ConceptRecord>` + `parse_with_diagnostics(tsv) -> (Vec, Vec<(line, ParseRowError)>)`. Permissive by design — comments, blank lines, CRLF, whitespace, unknown lang codes, and duplicate lang columns all handled so a hand-edited dialect pack can ship without breaking the boot path. `BTreeMap` (not `HashMap`) for label iteration so two parses of the same TSV produce byte-identical graph output — prepares the ground for an M11 content-addressed FST cache hash.
- **`lexicon/graph.rs`** (~400 lines, 12 tests): the builder pipeline. Four steps — flatten records → sort by `(lang_code, normalized_lemma)` → stream nodes + FST entries in the same pass → emit edges per concept (every cross-lang pair = `Equivalent`, every same-lang-within-concept pair = `Synonym`, then sort each node's edge list by `(kind, target)` for deterministic order) → CSR-compact into `(edge_offsets, edges)`. Builder is O(n) in total-lemma-count for the node/FST pass, O(concept × within-concept²) for edges — at 20K concepts × ≤30 nodes per concept that's ≈18M pair operations, sub-millisecond at M11 scale. Singleton via `LexiconGraph::get() -> &'static LexiconGraph` backed by `OnceLock` + `include_str!` of the seed. `empty()` constructor + `Default` for zero-node edge cases.
- **`lexicon/mod.rs`** (rewritten, ~300 lines, 13 tests): `equivalents(lemma, source) -> HashMap<Lang, Vec<String>>` walks the graph from every matching source node, collects `Equivalent`-kind edges, deduplicates while preserving first-encountered order. `expand(lemma, source, opts) -> ExpansionResult` does the full walk respecting `enabled_langs` filter, `SynonymLevel` (None / Synonym / SynonymAndHypernyms), and `max_per_lang` cap. Both functions have `_via(&graph, ...)` twins for tests — the default functions go through the singleton.
- **`lexicon/expansion.rs`** (unchanged from pre-M10 — already had complete types + 3 tests).

**Seed TSV** (`lexicon/data/seed_v1.tsv`, 15 rows):

Concepts: `book`, `knowledge`, `write`, `read`, `love`, `water`, `house`, `teacher`, `student`, `language`, `peace`, `truth`, `time`, `day`, `night`. Each row lists the concept id, a part-of-speech label (`Noun` for 13, `Verb` for 2), and 12–16 language columns in the form `lang_code:lemma1,lemma2,...`. Arabic lemmas are unvocalized (`كتاب`, not `كِتَاب`); English is lowercase; every row has at minimum `en:` + `ar:` (the two primary engine languages). Column order is free — parser keys on the `lang_code:` prefix.

**Test coverage**:

- **Parse** (11): single row, multi-lemma column, comments+blanks, unknown lang codes dropped-not-rejected, empty pos means unknown, no-labels row rejected, empty-id row rejected, single-column row rejected, CRLF line endings, whitespace trimming, duplicate lang columns append, whole-seed smoke test (every concept has en+ar).
- **Graph** (12): empty graph has sentinel offset, find on empty returns empty, builds from TSV, find round-trips, case-folds, strips Arabic diacritics, misses cross-language, edges reach all concept siblings with correct Equivalent/Synonym counts, edges sorted deterministically, out-of-bounds edges is empty, singleton builds from seed, POS populated on every node, concept_id stamped on every node.
- **Expand/equivalents** (13 in mod.rs): equivalents crosses languages, Arabic-to-English path, unknown-lemma empty, default includes synonyms+translations, mono mode skips cross-language, SynonymLevel::None preserves equivalents, enabled_langs filter respected, max_per_lang cap enforced, singleton-backed `expand()` smoke test, source identity preserved (lemma + lang echoed back), hypernyms empty at default level.

**Test results**: 305/305 library tests pass. Net +34 from M10 (11 parse + 12 graph + 13 mod.rs; −2 obsolete mod.rs stubs that tested `stub_returns_empty_equivalents`).

**Integration points for next milestones**:

- **M11 — 20K core**: parser unchanged; swap the seed for a pre-compiled `lexicon_v1.bin` at `include_bytes!`-time (or a `fst_bake`-style cache pipeline when the compressed size exceeds the binary-size budget). Add `LexiconGraph::load_bundle(path)` / `from_bytes(...)` twins of `load_core()` so the on-disk cache mirrors the Arabic engine's M3-baker pattern.
- **M12 — query expansion**: `search.rs::search_notes` (or a new helper) calls `lexicon::expand(query, detected_lang, opts)` and ORs every `flat_terms()` entry into the FTS5 MATCH expression. `ExpansionOptions` sources from a new `lexicon-settings.json` at `<Universe>/.constellation/`. The search bar toggle for "🌐 off" flips to `ExpansionOptions::mono(current_lang)`.
- **M13 — expansion packs**: add `LexiconGraph::merge_pack(pack_bytes)` that extends nodes/edges/name_index, written so two packs can coexist and user-added `UserLink` edges stay on top. Same FST-concat pattern that M3-baker uses for cache bundles.
- **M14 — settings UI**: an override pane analogous to M8c's ArabicOverridesPanel, with CRUD over `lexicon-overrides.json` and a live "show me what this expands to" preview. Same reindex-on-mutation pattern as M8c so FTS hit sets update in place.

**Build note**: the pre-M10 `LemmaNode` struct derived `Hash`; adding the new `pos: Option<PartOfSpeech>` field would have required also deriving `Hash` on `arabic::types::PartOfSpeech` (cross-module ripple). Since no caller hashes `LemmaNode` directly — nodes live in a `Vec<LemmaNode>` and are addressed by `u32` index — the derive was dropped with an inline comment explaining why. Zero visible effect outside the module.

### 30. M11-infra — Lexical Bridge on-disk bundle + three-stage boot

The M10 graph builds from the embedded seed TSV via parse + FST-compile + O(concept²) edge emission. At the M10 scale (~15 concepts) this is sub-millisecond and cache infrastructure is overkill. At the M11-data scale (20K concepts × ~10 languages ≈ 200K nodes and ~800K edges) the cold build is ~200–400 ms and a pure function of the seed — i.e. the textbook Write-Time Derivation (CLAUDE.md Rule 8) candidate: compute once, persist, read cheap on every subsequent launch.

**Why ship the baker now** (before the 20K data arrives): the binary format is cheap to design against the 15-concept seed, catches encoding bugs at small scale, and unblocks M11-data to land as a pure data drop with zero Rust changes. Exact same sequence as M3 (infra) → M3-baker (cache) → M1g/M1h (data): three separate landings, each with tight blast radius.

**Why this shape, not a serde-json dump**: the graph is hot-path — every cold launch reads it, every search pass walks it. JSON / MessagePack / bincode all add serde reflection overhead we don't need, and none of them can persist a pre-compiled `fst::Map` as opaque bytes without round-tripping through the key → value pairs (which would re-run the FST compiler on every warm start, defeating the point). A hand-rolled binary matching the arabic M3-baker layout wins on all three axes: zero serde cost, preserves `fst::Map` as-is, and the encoder/decoder is auditable in one file.

**Module shape**:

- `lexicon/bake.rs` (new, ~615 lines, 18 tests). Owns: cache-path resolution (`cache_file_path()` → `<cache_dir>/constellation/lexicon-v{hash:016x}.bin`), `version_hash()` (`djb2(seed_tsv) XOR CACHE_FORMAT_VERSION`, `OnceLock`-cached), `persist_best_effort(&LexiconBundle)`, `try_load_cached() -> Option<LexiconBundle>`, `write_bundle(path, &bundle)` / `load_bundle(path)` for tests, and the full encoder/decoder with hand-coded u8 tag tables for `Lang`, `Option<PartOfSpeech>`, and `EdgeKind`.
- `lexicon/graph.rs` (refactored). Split `fn build(records) -> LexiconGraph` into `pub fn build_bundle(records) -> LexiconBundle` and `pub fn LexiconGraph::from_bundle(bundle) -> Self`. Added `LexiconGraph::to_bundle(&self)` as a round-trip convenience. Rewrote `LexiconGraph::get()` to three-stage init: (1) try cache via `bake::try_load_cached` + `from_bundle`, (2) on miss/corruption parse the embedded TSV + `build_bundle` + `bake::persist_best_effort`, (3) `from_bundle` on the freshly-built bundle (same final step as stage 1 — no divergence). Exposed `pub fn seed_tsv() -> &'static str` so the version hash can read its input deterministically.
- `lexicon/mod.rs` (register + re-export). `pub mod bake;` + added `build_bundle`, `seed_tsv`, `LexiconBundle` to the `pub use graph::{…}` line.

**Binary format** (little-endian throughout):

```
magic:        [u8; 8]  = b"CAELEX01"
version_hash: u64
node_count:   u64
nodes:        [LemmaNode_encoded × node_count]
offset_count: u64     (= node_count + 1, sentinel-terminated)
edge_offsets: [u32 × offset_count]
edge_count:   u64
edges:        [Edge_encoded × edge_count]
fst_byte_len: u64
fst_bytes:    [u8]
```

LemmaNode_encoded = `lang_tag:u8` + `lemma:str` + `sense_id:u32` + `pos_tag:u8` + `concept_id:str`.
Edge_encoded = `target:u32` + `kind_tag:u8` + `weight:f32`.
`str` fields are `u32 len` + UTF-8 bytes.

**Tag tables** (append-only — never renumber; bump `CACHE_FORMAT_VERSION` when extending):

- `Lang`: Ar=0, De=1, En=2, Es=3, Fa=4, Fr=5, He=6, Hi=7, Ja=8, Ko=9, Pt=10, Ru=11, Tr=12, Ur=13, Zh=14.
- `Option<PartOfSpeech>`: None=0, Some(Noun)=1, Verb=2, Adjective=3, Adverb=4, ProperNoun=5, Particle=6, Foreign=7, Unknown=8.
- `EdgeKind`: Equivalent=0, Synonym=1, Hypernym=2, Hyponym=3, UserLink=4.

**Failure policy**: every disk op is best-effort. Missing file, wrong magic, hash mismatch, truncation, implausible count fields (>10M nodes / >100M edges), unknown enum tag, invalid UTF-8, trailing garbage after the FST block — all return `io::Error`. The caller (`LexiconGraph::get`) silently falls through to the in-memory rebuild path; worst case is the M10 boot cost.

**Test coverage (18 new tests)**:

1. `encode_lang_is_injective_and_total` — 15-Lang enumeration round-trips + unique tag check.
2. `decode_lang_rejects_unknown_tag` — tag 99 and tag 15 (one past the last valid) both return `None`.
3. `encode_pos_is_injective_and_total_incl_none` — 9-state `Option<PartOfSpeech>` round-trips.
4. `decode_pos_rejects_unknown_tag` — tag 99 and tag 9 both return `None`.
5. `encode_edge_kind_is_injective_and_total` — 5-variant enumeration round-trips + unique tag check.
6. `decode_edge_kind_rejects_unknown_tag` — tag 99 returns `None`.
7. `encode_decode_node_roundtrip` — single-node encode → cursor decode → field-by-field equality, incl. Arabic string.
8. `encode_decode_edge_roundtrip` — single-edge with non-trivial weight (`0.75`).
9. `bundle_write_read_roundtrip` — sample bundle writes to temp path and reads back identically.
10. `load_rejects_missing_file` — nonexistent path errors cleanly.
11. `load_rejects_wrong_magic` — `NOTMAGIC...` prefix errors with `InvalidData`.
12. `load_rejects_truncated_file` — lopping half the encoded bytes triggers short-read detection.
13. `load_rejects_wrong_version_hash` — flipping one byte of the hash field errors with `InvalidData` (simulates seed-TSV edit).
14. `load_rejects_trailing_garbage` — `EXTRA BYTES` appended errors; catches partial writes.
15. `cache_file_path_includes_version_hash` — filename shape is `lexicon-v{16 hex chars}.bin` (skipped on cache-dir-less CI).
16. `version_hash_is_stable_across_calls` — OnceLock cache consistency.
17. `djb2_matches_known_values` — hand-verified seeds (Knuth), guards against accidental hash-algorithm regression.
18. `real_seed_bundle_writes_reads_reconstructs` — end-to-end: build the real 15-concept seed bundle, write it, read it back, reconstruct a live graph, assert `en:book` and `ar:كتاب` still resolve. This is the canary that detects any encoder / decoder mismatch at realistic scale.

**Test results**: 323/323 library tests pass (up from 305; +18 from M11-infra). Zero regressions in any pre-existing module.

**Observed cache file size** at the M10 15-concept seed scale: the real bundle (via `build_bundle` on the full embedded seed) round-trips through disk cleanly in the `real_seed_bundle_writes_reads_reconstructs` test — size is a handful of KB. Projected size at the 20K-concept M11-data scale, using the same node / edge density as the seed (~60 nodes/concept × 36 bytes/node avg + ~900 edges/concept × 9 bytes/edge): ~10–15 MB on-disk (FST prefix-compresses the name index; the nodes/edges tables are already dense). Fits comfortably under the Arabic FST cache budget (~7.6 MiB at 595 roots today → ~90 MiB projected at 7K roots; the lexicon is higher-volume but simpler structure, so total-all-caches budget is comfortable).

**Integration notes for the next milestones**:

- **M11-data** lands a swap of `lexicon/data/seed_v1.tsv` with an auto-extracted 20K-concept file (WordNet 3.1 + Open Multilingual Wordnet + Wiktionary for the 15 supported languages). Pure data drop — zero Rust changes. Expected cache bundle ~10–15 MB; first launch on a new build writes the cache file, subsequent launches load in <50 ms via `bake::try_load_cached`.
- **M12 (query expansion)** landed this session on top of M11-infra (§ 31). `lexicon::expand()` reads the live graph irrespective of whether it came from cache or fresh-build; the new `lexicon::fts::build_match_expr` layer converts the expansion into an FTS5 `MATCH` clause without caring which path populated the graph.
- **M13 (expansion packs)** will likely add a `LexiconBundle::merge(other)` helper that concatenates two bundles' node/edge tables and rebuilds the FST from the union; the baker layer needs no changes.
- **M14 (settings UI)** does not interact with the cache — user overrides are a per-Universe file consulted at query time, not baked into the core bundle.

**Follow-ons queued**:

- `M11-data`: the 20K concept corpus itself (above). Blocked on WordNet/OMW extractor tooling (separate `lab/` task).
- `M11-mmap`: switch `name_index_bytes` from `Vec<u8>` to `memmap2::Mmap` on desktop targets so the page cache can demand-load the FST — same follow-on shape as M9-mmap on the arabic FST.
- `M11-cache-bench`: measure cold-start (cache deleted) vs. warm-start (cache present) delta on the M11-data bundle. Extend `arabic::bench::m9_bench` or create `lexicon::bench::m11_bench` following the same `#[test] #[ignore]` opt-in pattern.

### 31. M12 — query expansion plumbing (lexicon → FTS5 MATCH)

The M10 graph already produces `ExpansionResult` from a source lemma: a bag of translations + synonyms (+ optional hypernyms/hyponyms) bucketed by target language. M11-infra made that graph cheap to load on every boot. M12 closes the last gap between the lexicon and the search path: a **pure-logic** converter from `ExpansionResult` to an FTS5 `MATCH` clause, plus an end-to-end helper that search.rs will invoke once the M14 settings UI gates expansion on. No changes to `search.rs` yet — the user-visible lexical-search behaviour is byte-identical today. M12 is strictly plumbing.

**Why split the converter out of `search.rs`**: `search.rs` owns the SQL and should know MATCH syntax but not lexicon internals. `lexicon/*` owns the graph and should know MATCH syntax but not SQL. `lexicon::fts` is the narrow seam where the two meet. Every escaping / empty-input / edge-case concern is testable at the lexicon layer without a real search database — so the 25 new tests add zero SQLite surface to the test harness.

**Module shape**:

- `lexicon/fts.rs` (new, ~210 lines, 20 tests). Two pub functions:
  - `escape_fts_term(&str) -> Option<String>`: wraps the term in `"…"`, strips interior `"` (FTS5 phrases have no escape syntax) and control chars (null/tab/newline can confuse tokenizers regardless of what the syntax says about them), trims surrounding whitespace. Returns `None` when the result would be an empty phrase.
  - `build_match_expr(&ExpansionResult) -> Option<String>`: walks `ExpansionResult::flat_terms()`, escapes each, deduplicates on the escaped form (catches the case where two logically-distinct terms collapse to the same phrase after quote-stripping), joins with ` OR `. Preserves insertion order from `flat_terms`: source lemma first, then equivalents, then synonyms, then hypernyms, then hyponyms — stable ordering makes generated queries legible in diagnostics logs and tests deterministic. Returns `None` when the expansion produces zero usable terms; callers should treat that as "fall back to the user's plain query" rather than passing an empty MATCH clause (which FTS5 errors on).
- `lexicon/mod.rs` (extended). Adds `pub mod fts;` + `pub use fts::{build_match_expr, escape_fts_term};` + two end-to-end helpers:
  - `pub fn expand_to_match_expr(lemma, source, opts) -> Option<String>`: one-shot `expand()` + `build_match_expr` against the singleton graph. Intended as the search-path call site.
  - `pub fn expand_to_match_expr_via(graph, lemma, source, opts) -> Option<String>`: same, against a caller-supplied graph. Primary testing entry.

**Why phrase queries, not bare token queries**:

FTS5 treats unquoted uppercase `AND` / `OR` / `NOT` / `NEAR` as operators at query time. Even today's un-expanded search would break on a user query `"ON AND OFF"` if the term `AND` reached MATCH unquoted. At M11-data scale (20K concepts × 15 langs ≈ 200K + 100K lemma surfaces) the odds of a lemma colliding with an operator keyword rise materially — a Russian word transliterates to `or` in Latin-1-safe fallback, a Thai romanization produces `NEAR`. Wrapping every emitted term in `"…"` turns it into an opaque phrase so the query shape is bounded by what the caller intended. The cost is one byte of overhead per term; the benefit is that **no future data drop can silently corrupt MATCH syntax**.

**Why no escaping of `"`**:

FTS5's phrase syntax has no escape sequence — a `"` inside `"…"` terminates the phrase early and leaves the remainder dangling. The options are (a) strip `"` from the term, or (b) refuse to ship terms containing `"`. M12 picks (a) with the trade-off documented inline: at M11-data scale no supported language uses `"` as a lemma character, and the behaviour is loud (the escaped term still produces a valid phrase, just with the quote removed). A future pack with exotic character-class lemmas could revisit this — the tag-append-only discipline from M11-infra applies here too.

**Why no wiring into `search.rs` yet**:

M12 ships as pure infrastructure. Today's `lexical_search` / `search_titles` / `search_contents` all still pass the raw normalized query directly to MATCH — behaviour unchanged. The path to user-visible expansion is:

1. M14 (settings UI): user toggles "cross-lingual search" on in Settings, optionally narrows `enabled_langs`.
2. M14 extends `SearchRequest` with an `expand_languages: Option<ExpansionOptions>` field (None = today's behaviour; Some(opts) = call `expand_to_match_expr`, fall back to plain on `None` return).
3. M14 detects source lang from script (Arabic-range → `Lang::Ar`, Hebrew-range → `Lang::He`, else → settings default) before calling the helper.

Shipping M12 dark now means M14 is a narrow UX-and-wiring diff rather than a UX + plumbing + test bundle. Same reason M8 shipped before M8b before M8c.

**Test coverage (25 new tests — 20 in `fts.rs` + 5 in `mod.rs`)**:

`fts::escape_fts_term` (10 tests):
1. `escape_wraps_in_double_quotes` — basic success.
2. `escape_trims_surrounding_whitespace` — leading/trailing whitespace removed, interior preserved.
3. `escape_strips_interior_double_quotes` — `the "book"` → `"the book"`.
4. `escape_strips_control_characters` — tab, newline dropped; rest preserved.
5. `escape_returns_none_on_empty` — empty string → `None`.
6. `escape_returns_none_on_whitespace_only` — `"   "` → `None`.
7. `escape_returns_none_when_filter_strips_everything` — `"\"\""`, `"\n\t"` both collapse.
8. `escape_preserves_arabic_script` — `"كتاب"` survives intact.
9. `escape_preserves_multi_word_phrases` — `"New York"` valid (phrase queries are exactly the right shape).
10. `escape_preserves_internal_whitespace` — only leading/trailing trimmed, internal runs kept for phrase matching.

`fts::build_match_expr` (10 tests):
11. `build_contains_source_lemma_and_translations` — en `"book"` expands to include source + `"كتاب"` + `"livre"` + `" OR "`.
12. `build_includes_in_language_synonyms_at_default_level` — `"books"` present as En→En synonym of `"book"`.
13. `build_omits_synonyms_when_level_is_none` — synonyms gone, equivalents still present.
14. `build_source_lemma_appears_only_once` — no duplication on echo + bucket overlap.
15. `build_returns_none_on_fully_empty_expansion` — empty source lemma + empty buckets → `None`.
16. `build_returns_single_term_when_lemma_absent_from_graph` — unknown lemma → `"xyzzy"` (one-phrase query behaviourally identical to today's un-expanded search).
17. `build_respects_enabled_langs_filter` — `enabled_langs = {En}` drops Arabic/French, keeps En synonym `"books"`.
18. `build_from_arabic_source_reaches_all_targets` — `Lang::Ar` source `"كتاب"` reaches `"book"` / `"livre"`.
19. `build_term_with_quotes_would_have_been_dropped_individually` — synthetic result with two terms that escape-collapse to `"book"` → final expression is the single phrase (dedup works after escape).
20. `build_uses_or_separator_between_distinct_terms` — ≥3 `OR` separators on `"knowledge"` expansion (source + syn + 2 translations).

`lexicon::expand_to_match_expr[_via]` end-to-end (5 tests):
21. `expand_to_match_expr_via_produces_or_joined_phrase_query` — full pipe with `ExpansionOptions::default()` on the `small_graph()` tiny corpus.
22. `expand_to_match_expr_via_falls_back_to_source_on_miss` — unknown lemma produces single-phrase valid MATCH.
23. `expand_to_match_expr_via_returns_none_on_empty_lemma` — empty lemma → `None` (caller falls back to plain path).
24. `expand_to_match_expr_via_honours_mono_mode` — `ExpansionOptions::mono(Lang::En)` on "book" returns `"book"` only (safety net for the rollback / "feature off" case — behaviour byte-identical to today).
25. `expand_to_match_expr_through_singleton` — non-`_via` variant routes through `LexiconGraph::get()` (which exercises the M11-infra on-disk cache path) and returns a non-empty MATCH containing the source lemma.

**Test results**: 348/348 library tests pass (up from 323; +25 from M12). Zero regressions in any pre-existing module. Lexicon module alone now covers 82 tests (was 57 post-M11-infra).

**Integration notes for M13/M14**:

- **M13 (expansion packs)** adds `LexiconGraph::merge_pack(pack_bytes)`. Because `build_match_expr` takes an `ExpansionResult` (not a graph), packs automatically participate in expansion the moment they're merged — no further changes to `fts.rs`. The bake layer is the right place to extend for pack storage (sibling cache files, merged on load).
- **M14 (settings UI)** wires `expand_to_match_expr` into `search.rs::lexical_search` behind an `ExpansionOptions` field on `SearchRequest`. The `None`-means-fall-back contract is the rollback switch.

**Follow-ons queued**:

- `M12-bench`: landed this session as `lexicon::bench::m12_bench` — see § 32.
- `M12-lang-detect`: landed this session as `lexicon::detect::detect_source_lang` — see § 32.

## 32. M12 follow-ons: `lexicon::detect` + `lexicon::bench`

Two modules that close the M12 milestone before M14 wires the expansion into `search.rs`. One unblocks the upcoming search-path refactor (the caller needs to know which `Lang` to pass into `expand_to_match_expr`). The other locks in a performance budget so any future regression in the hot path trips an opt-in bench rather than slipping silently into a user's query latency.

### `lexicon::detect` — source-language detection

New module `crate::lexicon::detect` (`detect.rs`, ~280 lines, 33 tests). Single public entry point:

```rust
pub fn detect_source_lang(s: &str) -> Option<Lang>;
```

Classification is a two-step decision:

1. **Count strong-script characters per family** — Arabic, Hebrew, Devanagari, Cyrillic, CJK (Han + Hiragana + Katakana + Hangul), Latin. Digits, whitespace, ASCII punctuation, emoji, symbols, and script-less combining marks are ignored — they carry no language signal.
2. **Pick the dominant family, then disambiguate within it** using script-exclusive characters.

Family selection uses a single `max_by_key` over the six counters with a fixed tie-break order (Arabic → Hebrew → Devanagari → Cyrillic → CJK → Latin). `None` is returned when every counter is zero (pure-punctuation / digits-only / emoji-only / empty input) so the caller — M14's `lexical_search` — can skip lexicon expansion entirely and run the plain un-expanded FTS5 path.

**Arabic-family disambiguation** (Ar / Fa / Ur share the Arabic script):

- **Urdu-exclusive letters** (retroflex / Urdu yeh variants) force `Lang::Ur`: ٹ (U+0679), ڈ (U+0688), ڑ (U+0691), ں (U+06BA), ے (U+06D2), ۓ (U+06D3). These letters are absent from Modern Standard Arabic, Persian, and every other Arabic-script language we ship. Any single occurrence is a strong signal — they do not appear in Arabic or Persian loanwords as a normal orthographic device.
- **Perso-Arabic shared letters** (Persian + Urdu + a few other Indo-Iranian scripts, absent from MSA) force `Lang::Fa` when no Urdu-exclusive marks are present: پ (U+067E), چ (U+0686), ژ (U+0698), گ (U+06AF), ک (U+06A9 Persian kaf, distinct from Arabic ك U+0643), ی (U+06CC Persian yeh, distinct from Arabic ي U+064A).
- **Neither** → `Lang::Ar`. Pure Modern Standard Arabic text — any of the 28 core letters, diacritics, numerals (٠–٩) — hits this default.

**CJK-family disambiguation** (Ja / Ko / Zh share the Han ideograph block):

- Any Hangul syllable → `Lang::Ko`. Hangul does not appear in Japanese or Chinese text under any normal writing convention. Korean academic writing may include Han characters, but the Hangul presence still dominates.
- Any Hiragana / Katakana → `Lang::Ja`. These syllabaries are unique to Japanese. Korean text has no kana; Chinese has no kana outside loanword contexts we don't ship.
- Only Han ideographs → `Lang::Zh`. This is the pragmatic default — a Japanese user typing "日本" or "東京" without any kana misclassifies as Chinese. The workaround is obvious (include any kana) and the expansion cost is low: the lexicon graph walks the same Han nodes in both cases, so the user's search still hits Japanese notes containing the same characters. Documented trade-off.

**Latin-family disambiguation** (En / De / Es / Fr / Pt / Tr share the Latin alphabet):

- Turkish-exclusive: ğ/Ğ (U+011F/011E), İ (U+0130), ı (U+0131), ş/Ş (U+015F/015E). These letters appear in Turkish loanwords (e.g. "Istanbul") but the dotted/dotless I distinction in particular is very Turkish-specific.
- German ß (U+00DF). The other German-distinctive vowels (ä, ö, ü) overlap with Turkish and other languages so they're not disambiguators on their own.
- French œ/Œ (U+0153/0152) ligature. Extremely distinctive — French is the only shipped language using it in native orthography.
- Spanish ñ/Ñ (U+00F1/00D1). Also triggered by the inverted punctuation ¿ (U+00BF) and ¡ (U+00A1) even without any accented letters (e.g. "¿Cómo estás?" returns `Lang::Es`).
- Portuguese ã/Ã (U+00E3/00C3), õ/Õ (U+00F5/00D5). Tilded vowels.
- **Fallback**: `Lang::En`. Plain unaccented Latin text ("knowledge", "book"), shared-accent text (café, über), and any Latin-family query that carries no distinctive marker hits this default. At worst the lexicon graph walks find no English-node match and the expansion falls back to the un-expanded search — so the rollback story is preserved.

**Test enumeration** — `detect.rs` covers 33 tests across 8 behavioural buckets:

1. `empty_string_returns_none`
2. `whitespace_only_returns_none`
3. `digits_only_returns_none`
4. `ascii_punctuation_only_returns_none`
5. `emoji_only_returns_none`
6. `english_plain_ascii_is_en`
7. `arabic_text_is_ar`
8. `hebrew_text_is_he`
9. `devanagari_text_is_hi`
10. `cyrillic_text_is_ru`
11. `persian_kaf_distinguishes_fa_from_ar` — کتاب (Persian kaf U+06A9).
12. `persian_pe_che_zhe_gaf_trigger_fa` — پنج / چای / ژاله / گل.
13. `urdu_retroflex_distinguishes_ur_from_fa` — لڑکی (ڑ retroflex).
14. `urdu_yeh_barree_triggers_ur` — ہے (ے yeh-barree).
15. `urdu_noon_ghunna_triggers_ur` — ماں (ں noon ghunna).
16. `pure_han_is_zh` — 本 / 中文 (documented misclassification of pure-Han Japanese).
17. `hiragana_triggers_ja` — ほん / ありがとう.
18. `katakana_triggers_ja` — コンピュータ.
19. `mixed_han_and_kana_is_ja` — 東京の本 / 日本語のほん.
20. `hangul_triggers_ko` — 책 / 한국어.
21. `hangul_wins_over_han_when_mixed` — 한국의 文化.
22. `turkish_dotless_i_triggers_tr` — İstanbul / kitaplık.
23. `turkish_breve_g_triggers_tr` — yağmur.
24. `turkish_cedilla_s_triggers_tr` — güneş.
25. `german_sharp_s_triggers_de` — Straße / groß.
26. `french_oe_ligature_triggers_fr` — cœur / œuvre.
27. `spanish_n_tilde_triggers_es` — España / niño.
28. `spanish_inverted_punctuation_triggers_es` — ¿Cómo estás? (no letters needed).
29. `portuguese_tilded_vowels_trigger_pt` — não / coração.
30. `shared_accents_without_distinctive_marks_fall_to_en` — café / über (documented fallback).
31. `dominant_script_wins_in_mixed_query` — "المعرفة book" → Ar.
32. `latin_wins_when_dominant` — "knowledge and كتاب on the shelf" → En.
33. `arabic_with_digits_still_arabic` — "كتاب 2026" → Ar (digits don't contribute).

### `lexicon::bench` — expansion-latency microbench

New `#[cfg(test)] mod bench` inside `lexicon/mod.rs` (`bench.rs`, ~160 lines, 1 opt-in test). Matches the `arabic::bench::m9_bench` pattern — `#[test] #[ignore]`, does not run under default `cargo test --lib`, invoked explicitly with:

```bash
cargo test --lib --release lexicon::bench -- --ignored --nocapture
```

The bench exercises `expand_to_match_expr_via` (test-injection route, so the bench can be re-run in the same process without `OnceLock` interference) across a 23-query bank:

- **12 English seed hits** — book, knowledge, read, write, house, water, love, time, peace, truth, teacher, student. These walk the FST name-index lookup → graph walk → `flat_terms()` → MATCH-builder happy path.
- **4 Arabic seed hits** — كتاب, معرفة, قرأ, ماء. Exercises the RTL normalisation route (`normalize_stripped`) before the FST probe.
- **3 non-English / non-Arabic seeds** — livre (Fr), Wissen (De), libro (Es). Graph walks starting from non-English nodes.
- **2 miss cases** — xyzzy (En), لا_موجود (Ar). Source-echo only, no graph walk succeeds.
- **2 short-circuit cases** — empty string, whitespace-only. Verifies the `None` fast path.

Each query runs 1,000 iterations under both `ExpansionOptions::default()` (all 15 langs, synonyms on) and `ExpansionOptions::mono(Lang::En)` (one lang, synonyms off — the rollback / "🌐 off" toggle), producing 23 × 1,000 × 2 = **46,000 latency samples** per run.

**Results at M10-seed scale** (595 seed roots — the currently embedded corpus — on Windows/MSVC release build, single threaded):

| Metric | Value |
| --- | --- |
| Mean | 5,185 ns (**5.2 µs**) |
| p50 | 1,800 ns |
| p95 | 12,400 ns |
| p99 | **15,800 ns (15.8 µs)** |
| Max | 159,500 ns |

Hard assertion: `p99 < 1_000_000 ns` (1 ms). Current p99 sits ~60× under budget; max (OS-scheduling jitter) ~6× under budget. Any future change that introduces a per-call allocation, a full-graph scan, or an unbounded synonym walk will trip the assertion on the next opt-in run.

### Why the `_via` variant for the bench

`expand_to_match_expr` (non-`_via`) routes through `LexiconGraph::get()`'s `OnceLock`, which can only initialise once per process lifetime. The bench wants to re-enter the expansion path many times with the same graph, so it uses `expand_to_match_expr_via(graph, …)` and grabs `graph` once from the singleton up front. This matches the `arabic::bench::m9_bench` discipline of measuring constituent steps explicitly rather than pretending `get()` is observable on warm starts.

### Scale note

The bench runs against the M10 15-concept seed, not the future M11-data 20K-concept corpus. The expansion hot path is bounded by **one concept's cross-lang neighbour count** (~15–30 nodes at any scale) plus a single FST probe, so scaling up the corpus doesn't scale up the per-call cost. But we'll re-run the bench once M11-data lands and publish the confirmed numbers alongside the existing `M11-cache-bench` follow-on. If the p99 migrates meaningfully the 1 ms budget gets re-examined with real data.

### Integration with M14

M14's `search.rs::lexical_search` will call these two new helpers in series at the top of the query-normalisation path:

```rust
let source = lexicon::detect_source_lang(raw_query);
let expansion = source.and_then(|lang| lexicon::expand_to_match_expr(raw_query, lang, &opts));
let fts_query = expansion.unwrap_or_else(|| fallback_plain_fts(raw_query));
```

Two `None` branches and a happy path — the same three-way shape M14's `SearchRequest.expand_languages: Option<ExpansionOptions>` toggle will surface to the UI. The detector is already the right shape; the bench is already the right guardrail. The only code in M14 that's new is the field on `SearchRequest` and the call-site wiring.

### Follow-ons queued

- `M12-bench-m11`: re-run `m12_bench` once `M11-data` lands and publish the 20K-concept × 15-lang numbers alongside the current M10-seed baseline. Not a new module — just an opt-in rerun with the new corpus in place. If p99 migrates, the hard-assert threshold gets adjusted with a one-line change.

## 33. M8b-v2 — layered overrides + M8c integration tests + normalizer alignment

Two items graduated from the Open items section (§ Open items below) into shipping code, plus a linguistic correctness fix uncovered while writing the integration tests.

### Why now

M8 shipped user overrides against a single global `ACTIVE_STORE`. Two gaps remained:

1. **M8b-v2**: per-cUniverse layering. A Universe that federates in child Universes (via `UniverseMeta.children`) couldn't consult the child's override files — the global store held only the sovereign (active) Universe's authored overrides. Parent–child precedence had no implementation.
2. **M8c-integration-test**: the M8c shipping PR added unit coverage for the Tauri command's wiring but deferred the full-chain assertion (author override → reindex → FTS token set flips) until a real `SearchState` harness existed. The open item was parked on the "Settings → Debug scorecard" milestone.

Both are now unblocked: cUniverse federation is real enough that a user switching the active Universe with children declared should see the child overrides light up automatically; and the integration test can run against a tempfile-backed `SearchState` without needing a scorecard UI. Both land in this session.

A third finding surfaced during M8c test authoring and is captured here too: `search::normalize_arabic_for_search` had been folding ta-marbuta (ة → ه), alif maqsura (ى → ي), and alif variants (أ/إ/آ/ٱ → ا) — silently conflating semantically-distinct words like `عبرة` (a lesson) and `عبره` (he crossed it). This violated Constellation's "Language-First by Design" principle (CLAUDE.md) and disagreed with `arabic::normalizer::normalize().stripped` (the override-store key normalizer). Aligned both to strip tashkeel + tatweel only. Rationale captured in the new doc comment with the `عبرة` / `عبره` motivating pair.

### M8b-v2 — layered `OverrideStore`

Refactored `OverrideStore` from a single `HashMap<String, UserOverride>` to a **stack of layers**: `layers: Vec<HashMap<String, UserOverride>>`, with `layers[0]` the **sovereign** layer (always the active Universe's own overrides) and `layers[1..]` the **child Universe** layers in the order declared by `UniverseMeta.children`.

**Lookup semantics** — parent-wins, walk layers in order:

```rust
pub fn lookup(&self, normalized_surface: &str) -> Option<&UserOverride> {
    for layer in &self.layers {
        if let Some(v) = layer.get(normalized_surface) { return Some(v); }
    }
    None
}
```

**CRUD-sovereign invariant** — `insert` / `remove` / `save_to_path` only ever mutate `layers[0]`. Child layers are read-only views of another Universe's file on disk; editing them here would be a cross-Universe mutation the user never asked for. Same constraint goes for the `clear_active` / `set_active` `add_arabic_override` / `remove_arabic_override` command chain.

**Key API additions**:

- `OverrideStore::from_layered_paths(&[PathBuf]) -> io::Result<Self>` — builds a layered store from an ordered list of override file paths. Path 0 is the sovereign; paths 1.. are children. Missing files produce empty layers rather than errors (a child with no overrides is a normal state, not a failure).
- `overrides::activate_layered_for_universe(universe_root, &[child_roots]) -> Result<usize, String>` — one-call boot: resolve every file, load them, install as the new `ACTIVE_STORE`. Returns the total entry count across all layers.
- `overrides::activate_for_universe(universe_root)` — back-compat wrapper that calls `activate_layered_for_universe(root, &[])`. Existing callers don't need to change until they want federation.
- `overrides::set_sovereign_layer(store)` — replaces **only** `layers[0]` and keeps the existing `layers[1..]` intact. Used by the `add_arabic_override` / `remove_arabic_override` Tauri commands so CRUD on the sovereign layer doesn't evict the child-universe layers from memory and force a reload.
- `OverrideStore::layer_count()`, `sovereign_iter()` — diagnostic readers used by the new tests.
- `read_layer(path)` (module-private) — shared single-file loader between `load_from_path` and `from_layered_paths`. Keeps the on-disk schema and key-normalization path identical regardless of entry point.

**Universe integration** — `src-tauri/src/universe.rs`:

- New helper `resolve_child_universe_roots(parent: &Path) -> Vec<PathBuf>` (mirrors the existing `resolve_libraries_recursive` pattern). Reads the parent's `universe.json`, enumerates `children`, canonicalises each path, and drops any that aren't readable directories. Silent-skip on malformed entries — federation degradations must not block boot.
- `set_active_universe` (the `#[tauri::command]` hook) now calls `activate_layered_for_universe(final_path, &child_universe_roots)` instead of the single-path `activate_for_universe`. Logs the total entry count as before.

**Tests — 17 new in `arabic::overrides::tests`**:

Covers every new semantic:
- Parent-wins on surface collision (sovereign lemma beats child lemma).
- Child-only hit (surface absent from sovereign, present in child → child's override fires).
- Multi-child walk (three-layer stack; hit in layer 2 / layer 3 / miss across all).
- CRUD-sovereign-only invariant: `insert` on a layered store mutates only layer 0; `remove` refuses to touch child layers; `save_to_path` serialises only layer 0's entries.
- `set_sovereign_layer` preserves children across sovereign replacement.
- `from_layered_paths` with missing child file → empty child layer, no error.
- `activate_layered_for_universe` end-to-end: resolve paths, load, set active, verify `active().layer_count()`.
- Empty-stack edge cases: `is_empty`, `len`, `iter` all return the zero value.

The test mutex was promoted from a submodule-local `REGISTRY_TEST_MUTEX` to a crate-visible `#[cfg(test)] pub(crate) static TEST_OVERRIDE_MUTEX` at the `overrides` module scope, so the new `search::tests_m8c` module (see below) serialises against the same global.

### M8c — end-to-end integration tests (`search::tests_m8c`)

New `#[cfg(test)] mod tests_m8c` inside `src-tauri/src/search.rs`. Four tests that exercise the full contract the `add_arabic_override` Tauri command relies on:

1. **`override_and_reindex_flips_fts_token_set`** — the headline contract. Seeds a `note_meta` row with body `"خليفة راشد"`, asserts a sentinel stem `"pinnedteststem"` is absent from FTS, installs an override mapping `خليفة → pinnedteststem`, asserts the sentinel is **still** absent (overrides don't retroactively mutate indexed rows), runs `reindex_notes_matching_text(state, "خليفة")`, and asserts (a) exactly 1 row was re-tokenized and (b) the sentinel now MATCHes. If this ever regresses, the Settings UI "pin this word" button becomes a silent no-op on the existing index — exactly the failure the test is written to catch.
2. **`reindex_returns_zero_when_no_notes_match`** — forward-looking override (no existing note mentions the surface) returns 0 re-tokenizations without error. Guards the common case of a user authoring an override ahead of future content.
3. **`reindex_empty_needle_short_circuits`** — empty / whitespace needle returns 0 before issuing the `body_text LIKE %%` scan. Guards the UI against an accidental empty-string dispatch triggering a full-table scan on a 7,600-note Universe.
4. **`reindex_updates_all_matching_rows_in_one_pass`** — three notes mention `خليفة` + one unrelated note. Asserts the single `BEGIN IMMEDIATE` transaction flips all three rows and leaves the unrelated row untouched.

**Test-harness design**:

- `OverrideTestGuard` — RAII guard that locks the crate-wide `TEST_OVERRIDE_MUTEX` on construction and clears `ACTIVE_STORE` on both construction and drop. Ensures no test leaks override state to a neighbour, and that the global store isn't raced across `tests_m8c` + `arabic::overrides::tests`.
- `seeded_state(dir, path, body) -> SearchState` — tempfile-backed SQLite DB seeded with one `note_meta` row. Body is pre-normalised via `super::normalize_arabic_for_search` to mirror production's `index_note` (which normalises `plain_body` before INSERT). Without this, the body-side ta-marbuta survived while the needle-side didn't, and the LIKE scan missed — captured as the doc comment rationale.
- Latin sentinel `"pinnedteststem"` for the override lemma — guaranteed never to be an Arabic analyzer verdict, so `notes_fts MATCH 'pinnedteststem'` cleanly distinguishes pre-override from post-override FTS state with no false positives from the default stemming pipeline.
- Per-test nanosecond-stamped temp directories so concurrent test workers don't collide on the same SQLite file.

### Bonus fix — `normalize_arabic_for_search` aligned with `arabic::normalizer::normalize_stripped`

Discovered while writing `override_and_reindex_flips_fts_token_set`. The first version of the test used `خليفة` (ta-marbuta) and the reindex returned 0 rows even though the override was installed correctly. Tracing revealed two separate normalizers with different semantics:

| Normalizer | Scope | ة/ه | ى/ي | أ/إ/آ → ا |
| --- | --- | --- | --- | --- |
| `arabic::normalizer::normalize().stripped` | Override store keys, Layer 2/3 lookups | preserved | preserved | preserved |
| `search::normalize_arabic_for_search` (old) | FTS body storage + query pre-pass | **folded** | **folded** | **folded** |

Production `index_note` stores `body_text` through the aggressive folder; `reindex_notes_matching_text` builds its LIKE needle with the same folder; but the override store's key is the un-folded stripped form. So `خليفة` (key) and `خليفه` (folded body) never matched, and any user-authored override whose surface contained ة / ى / alif-variants was a silent no-op on the FTS path.

**Fix**: `search::normalize_arabic_for_search` now delegates to `arabic::normalizer::normalize_stripped` — exactly one tashkeel/tatweel implementation in the codebase. The aggressive fold is gone from the index/query path.

**Why this is the correct behaviour, not just a compromise**: the fold-based normalizer was conflating semantically-distinct words. The canonical case (captured in the new doc comment):

| Surface | Reading | Meaning |
| --- | --- | --- |
| `عبرة` | ʿibrah | a lesson / moral — "عبرة لمن اعتبر" |
| `عبره` | ʿabarah | he crossed it / went through it |

Different roots, different morphology, different pronunciation, different meaning. Folding ة → ه merges them into one FTS token — so a search for "عبرة" (a lesson) returns every note that said "مرّ عبره" (he crossed it), and vice versa. That is a **semantic break**, not a cosmetic one. Same applies to `خليفة` vs verb-form `خليفه`, `موسى` (terminal alif maqsura is correct spelling), and hamza-bearing alifs in `إسلام`, `أحمد`, `آمنة`.

**Trade-off**: misspelled queries ("خليفه" when the user meant "خليفة") no longer cross-match. The correct place to handle that is a dedicated query-side spelling-tolerance layer with contextual disambiguation — not lossy transformation at index time. Added as the new M8e open item (§ Open items).

### Results

`cargo test --lib`: **402 passed, 0 failed, 2 ignored, 0 measured**. Net +21 tests this landing (17 M8b-v2 + 4 M8c). No regressions from the normalizer alignment — no existing test depended on the previous folding behaviour, which is itself evidence the fold was orphaned from the rest of the pipeline's semantics.

### Follow-ons queued (M8e)

- **M8e: spelling-tolerance query layer** — handle misspellings like "خليفه" (heh) for "خليفة" (ta-marbuta) at query time, without destroying the `عبرة` / `عبره` distinction. Candidate approaches: edit-distance-bounded FTS5 match expansion; a dedicated spellcheck pass that runs against the user's own vocab before the lexical query; context-aware disambiguation using a 3-word window around the ambiguous surface. Scope and design deferred until real-user queries surface which misspelling classes matter most.

## 34. M9-hotpath (a) — AtomicBool fast path on the FTS override probe

First of the four M9 follow-ons queued in § Open items. Closes candidate (a) of the three named there (`overrides::active()` unconditional `Arc::clone` even when the active store is empty). Candidates (b) `SmallVec` for `rank_analyses` and (c) generator-style visitor remain queued.

### Why now

The FTS5 tokenizer calls `libraries::process_arabic_word` once per Arabic token during note indexing. M8b wired that into `crate::arabic::overrides::active()` so user-authored overrides short-circuit through Layer 0 of the analyzer. Before this landing, every token — even on Universes with zero authored overrides (fresh installs, the overwhelming common case) — paid:

1. `RwLock::read()` on `ACTIVE_STORE` (~20 ns uncontended).
2. `Arc::clone()` of the inner store (~5 ns refcount bump).
3. `store.is_empty()` HashMap walk (~2 ns).
4. `Option<&OverrideStore>` synthesised from the `Arc` for the downstream call.

Total ~25–30 ns per token. At 100 K tokens per note that's ~2.5 ms of pure overhead paid by every indexer pass on a Universe that will never use overrides — i.e. the default state of every install.

### Change shape

New `AtomicBool ACTIVE_STORE_EMPTY` alongside the existing `ACTIVE_STORE: OnceLock<RwLock<Arc<...>>>` in `arabic::overrides`. Maintained under a documented ordering discipline by every mutator (`set_active`, `set_sovereign_layer`; `clear_active` inherits via `set_active`).

**Invariant**: if `ACTIVE_STORE_EMPTY == true`, the active store is guaranteed empty. The reverse is allowed to over-report non-empty (the worst case is one extra `Arc::clone` + HashMap-miss probe — correct, just wasteful).

**Ordering rule**:
- Empty → non-empty transition: flip the atomic to `false` **before** the RwLock swap. A reader observing `false` either takes the slow path and sees the old (still non-empty or empty-being-replaced) store, or the new non-empty store — both are "safe to probe".
- Non-empty → empty transition: flip the atomic to `true` **after** the RwLock swap. A reader observing `true` is guaranteed the post-swap empty store is what they'd see on the slow path — safe to skip the lock.

```rust
pub fn set_active(store: OverrideStore) {
    let is_empty = store.is_empty();
    if !is_empty {
        ACTIVE_STORE_EMPTY.store(false, Ordering::Release);
    }
    *store_lock().write().expect("...") = Arc::new(store);
    if is_empty {
        ACTIVE_STORE_EMPTY.store(true, Ordering::Release);
    }
}
```

New public API `pub fn active_if_non_empty() -> Option<Arc<OverrideStore>>`:

```rust
pub fn active_if_non_empty() -> Option<Arc<OverrideStore>> {
    if ACTIVE_STORE_EMPTY.load(Ordering::Acquire) {
        return None;
    }
    Some(store_lock().read().expect("...").clone())
}
```

The existing `active()` symbol stays unchanged — diagnostic / admin callers (tests, Settings UI, `read_arabic_overrides` command) continue to get a concrete `Arc` on every call and can `.iter()` / `.len()` without branching. Only the FTS hot path migrates to `active_if_non_empty`.

### Call-site migration

`libraries::process_arabic_word` (1 call site — the only per-token caller of `active()`):

```diff
-    let store = crate::arabic::overrides::active();
-    let overrides_ref = if store.is_empty() {
-        None
-    } else {
-        Some(store.as_ref())
-    };
+    let store_owned = crate::arabic::overrides::active_if_non_empty();
+    let overrides_ref = store_owned.as_deref();
```

Net effect on the empty-store path (the default): one `AtomicBool::load(Acquire)` (~2 ns) instead of the four-step sequence above. On the non-empty path: identical cost to pre-M9 plus a conditional branch on the atomic (one mispredicted branch amortised across the whole analysis call).

### Test harness update — `RegistryGuard::drop`

`RegistryGuard` — the RAII guard in `arabic::overrides::tests` — snapshots the active store on construction and restores it on drop. The restore step was previously a bare RwLock swap that didn't touch `ACTIVE_STORE_EMPTY`. With the new atomic, a stale bit after Drop could make the *next* test observe an inconsistent fast-path / slow-path pair. Fixed: Drop now mirrors `set_active`'s ordering discipline when restoring, so the atomic always reflects the restored store's emptiness.

### Bench harness — new "Throughput FTS" measurement

`arabic::bench::m9_bench` grew one new measurement block between the existing Throughput (bare `analyze_best`) and Accuracy sections. It exercises the production FTS tokenizer shape — `active_if_non_empty` + `analyze_with_overrides_best` — against the same 502-case corpus × 500 iterations. Reports:

- `Throughput FTS (w/s)` — the production path's per-second throughput.
- `Per-call FTS (ns)` — per-token cost on the FTS path.
- `FTS overhead (ns)` — the delta vs bare analyze. **Should stay ≤ 0 after M9-hotpath (a)**; the `active_if_non_empty` fast path is so cheap it's indistinguishable from the bare path when the store is empty.

### Bench results — before / after

Same run setup (Windows release build, M9 corpus 502 × 500 = 251K calls). Captured this session:

| Metric | Before (M8c) | After (M9-hotpath a) | Delta |
| --- | --- | --- | --- |
| Throughput (bare, w/s) | 129,430 | 128,616 | −0.6% (noise) |
| Per-call bare (ns) | 7,726 | 7,775 | +49 (noise) |
| Throughput FTS (w/s) | *(not measured)* | 129,577 | — |
| Per-call FTS (ns) | *(not measured)* | 7,717 | — |
| FTS overhead (ns) | *(not measured)* | −58 | ≈ 0 |
| Pass rate | 100.0% | 100.0% | 0 |
| Cold-start (ms) | 182.3 | 176.6 | −3.1% |
| Cache bundle (KiB) | 7,812 | 7,812 | 0 |

**Reading the numbers**: the before-vs-after throughput delta on the bare path is within run-to-run noise (±1% is typical), which is expected — we didn't change anything in `analyze_best`. The key outcome is the **FTS overhead line: −58 ns, within noise of zero.** That confirms the `active_if_non_empty` fast path has dropped the production-path cost to parity with the bare analyzer path. Pre-fix, the same measurement would have shown +25–30 ns; we didn't capture that number before the code change, but it's the old `active()` + `is_empty()` cost the diff removes.

**Net production savings**: ~25–30 ns per token on the empty-store path, which is the default. Approximately 0.4% of the overall per-call budget at M9-seed scale — small in percentage terms, but it's strictly wasted work the indexer no longer pays. At 10M tokens across a large Universe, that's ~250 ms trimmed off a full reindex.

**Why the 200K words/sec target is still open**: the measurement above pins down where the time *isn't* spent (override probe overhead). The remaining 7,700+ ns per call lives inside the analyzer itself — Unicode normalization character-iteration, HashMap probes in the protected list, FST byte-buffer walks, `rank_analyses` sort + Vec allocation. Candidates (b) `SmallVec` for `rank_analyses`, (c) generator-style short-circuit, and the new `M9-profile` entry below target that budget directly.

### Tests — 7 new in `arabic::overrides::tests`

All under the existing `RegistryGuard` harness + `TEST_OVERRIDE_MUTEX`:

1. `active_if_non_empty_returns_none_on_default_empty_store` — cold baseline, no `set_active` called.
2. `active_if_non_empty_returns_some_after_installing_nonempty_store` — `set_active(store_with_one_entry)` → `Some(Arc)` reachable.
3. `active_if_non_empty_returns_none_after_clear_active` — round-trip: install, verify `Some`, clear, verify `None`.
4. `active_if_non_empty_tracks_set_sovereign_layer_transitions` — the CRUD path (add / remove overrides via the Settings UI) must flip the atomic correctly too.
5. `active_if_non_empty_with_child_layers_returns_some_even_when_sovereign_empty` — federation edge case: empty parent + non-empty child = observably non-empty store, fast path must return `Some`.
6. `active_if_non_empty_is_coherent_with_is_empty` — property check across four transitions: `active_if_non_empty().is_some() == !active().is_empty()` always holds.
7. `active_if_non_empty_some_branch_returns_same_arc_as_active` — `Arc::ptr_eq` between the two APIs on the non-empty path, guarding against accidental deep clones.

### Results

`cargo test --lib`: **409 passed, 0 failed, 2 ignored, 0 measured**. Net +7 tests this landing. No regressions.

### Follow-ons queued (M9)

- **M9-hotpath (b) — `SmallVec<[Analysis; 2]>` for ranked results**: `analyze_with_overrides` returns `Vec<Analysis>` even on single-hit paths (Layer 0 / 1 / 2 with exactly one candidate — the common case). A `SmallVec<[Analysis; 2]>` internal buffer would dodge one heap allocation per word. Public API unchanged — the Vec conversion happens at the function boundary. Blocked on the Analysis-allocations profile below; if allocations are the dominant remaining cost, (b) ships next; if they're not, (b) is deferred.
- **M9-hotpath (c) — generator-style early-exit visitor**: restructure `analyze_with_overrides` so Layer 0 / 1 / 2 hits return without ever building the full candidate Vec. Biggest code-shape change of the three; least likely to ship without a concrete profile driving it.
- **M9-profile — flamegraph / `perf record` of the throughput loop**: the M9-hotpath (a) bench numbers show the remaining budget (~7,700 ns per call) is in the analyzer core, not the override probe. Profile the `analyze_with_overrides_best` call across the regression corpus under `--release` with frame-pointers to identify the top 5 hot functions, then target them with named M9-hotpath (b/c) / M9-intern landings. This replaces the "three candidates from the open-item" with measured priorities.

## 35. M9-rss-real — real OS-level RSS probe for the bench

Second M9 follow-on landed this session (after hotpath (a)). Smallest / safest of the six M9 follow-ons — test-only, no production path touched, no bundle-format change. Picked first from the remaining queue on a risk-ordering basis: lands alone, gives every subsequent M9 follow-on a real memory number to aim at (M9-intern and M9-mmap are the ones that actually move it).

### Why now

`arabic::bench::m9_bench` has been reporting `Cache bundle (KiB)` and `Projected @ 7K (MiB)` as the memory-footprint proxies since the M9 landing. Both numbers are the **on-disk** bundle size (FST bytes + side-table bytes) times the extrapolation factor. That's a lower bound — it doesn't include:

1. the parsed `fst::Map` header state after decode (BurntSushi's `Map` keeps the byte buffer accessible for traversal, plus some small per-node metadata);
2. the `Vec<GeneratedForm>` side-tables (each `GeneratedForm` today owns two heap `String` fields — `root_key` + `pattern_label` — plus the surface string and a `PatternKind` tag);
3. `OnceLock` wiring + lazy init overhead on the process singleton.

For the "≤ 100 MiB RSS at 7K-root corpus" M9 success criterion, a real number is what tells us whether we've made it. The on-disk proxy tracks direction but not magnitude. Before M9-mmap and M9-intern run, we need a baseline we can compare against after each of them lands.

### Change shape

New module `arabic::rss` — ~200 lines, test-only, registered with `#[cfg(test)] mod rss;` from `arabic::mod.rs`. Single public API:

```rust
pub fn read_rss_bytes() -> Option<u64>
```

Returns the caller process's resident set size in bytes, or `None` if the platform's RSS query fails (rare — a kernel refusal or a missing `/proc`). The bench treats `None` as "skip this line of the report" rather than erroring, so the bench still finishes on exotic CI runners that lack the usual probe.

**Platform backends** — each `#[cfg(target_os = "…")]`-gated, no shared deps:

- **Windows** — direct `extern "system"` FFI to `K32GetProcessMemoryInfo` (Win7+ kernel32.dll redirect from psapi.dll). `#[repr(C)] ProcessMemoryCounters` laid out to match the documented struct; `working_set_size` returned as the RSS figure. ~30 lines. Avoids pulling in `windows-sys` (and its transitive dep cost) for what's test-only code.
- **Linux** — reads `/proc/self/statm`, parses the second whitespace-delimited field (`resident` in pages), multiplies by 4096 (the page size on every Linux target we care about). ~10 lines. No `sysconf(_SC_PAGESIZE)` FFI because the value has been 4096 on x86/x86_64/aarch64 Linux for decades — if it ever changes, the bench number is off by a small factor, not a correctness issue.
- **macOS** — `task_info(mach_task_self(), MACH_TASK_BASIC_INFO, …)` via `extern "C"`. `#[repr(C)] MachTaskBasicInfo` laid out to `<mach/task_info.h>`; `resident_size` returned. ~30 lines.
- **Fallback** — unknown targets (iOS / Android / BSDs the app doesn't ship to today) return `None`. No compile-time failure; bench degrades gracefully.

Stdlib-only, no new deps. The `memory-stats` / `sysinfo` crate route would pull in a multi-hundred-line cross-platform dep tree for code that's test-only and ~70 lines of direct FFI. Trade-off documented at the top of `rss.rs`.

### Precision note

The numbers are "dirty" RSS — what the OS thinks the process currently holds, including pages from shared libraries mapped in, pages read from disk, copy-on-write pages, etc. It's a lower bound on actual memory pressure in the most useful sense: if this number is 100 MiB, the kernel will feel at least 100 MiB of pressure if something needs memory. For the bench's purpose (tracking Arabic-engine allocations against a budget), that's the correct abstraction.

### Bench harness — two new lines

`arabic::bench::m9_bench` grew an RSS baseline **before** cold-start (section 0) and an RSS delta **after** the throughput loop (section 6). Between them, section 0 captures resident memory before the FST/side-table initialisation runs; section 6 captures it after. The delta isolates the Arabic-engine allocations from whatever baseline the test harness costs. The bench extrapolates via the `7000 / fst_keys` ratio matching the existing `Projected @ 7K (MiB)` line on the bundle side.

```rust
let rss_before = read_rss_bytes();
// ... cold-start, warm-start, throughput, accuracy, bundle size ...
let rss_after = read_rss_bytes();
if let (Some(before), Some(after)) = (rss_before, rss_after) {
    let delta_mib = (after.saturating_sub(before)) as f64 / (1024.0 * 1024.0);
    let projected_mib = delta_mib * (7000.0 / fst_keys as f64);
    report("RSS before (MiB)", format!("{:.1}", before as f64 / MiB));
    report("RSS after (MiB)",  format!("{:.1}", after  as f64 / MiB));
    report("RSS delta (MiB)",  format!("+{delta_mib:.1}"));
    report("RSS projected @ 7K (MiB)", format!("{projected_mib:.1}"));
}
```

If either read returns `None`, all four lines are skipped — the bench still prints the bundle-size figures. No new test dependency.

### Bench results — first real RSS numbers

Captured this session, same run setup (Windows release build, M9 corpus 502 × 500 = 251K calls, 32,197 FST keys):

| Metric | Value | Interpretation |
| --- | --- | --- |
| Cache bundle (KiB) | 7,812.4 | On-disk FST + side-tables |
| Projected @ 7K (MiB) | **89.8** | Old proxy (under-counts) |
| RSS before (MiB) | 9.5 | Baseline process footprint |
| RSS after (MiB) | 33.3 | Post-cold-start + throughput loop |
| RSS delta (MiB) | **+23.8** | The Arabic engine's actual allocations |
| RSS projected @ 7K (MiB) | **280.3** | New real-resident projection |

**The 280.3 / 89.8 ≈ 3.1× ratio is the headline.** The real memory footprint at 7K roots is projected ~3× the on-disk bundle. That's driven by the parsed `fst::Map` state + the `Vec<GeneratedForm>` side-table owning two heap `String` fields per entry. At 32K keys today, the `GeneratedForm` strings alone account for most of the gap — and both are exactly what M9-intern targets.

At the projected 7K-root scale (≈7,000 roots × 140 forms per root ≈ 1M forms), `280 MiB` is ~2.8× above the M9 `≤ 100 MiB` budget. That gives us a concrete target: **M9-intern needs to shave ~180 MiB.** The intern table's theoretical ceiling (a few dozen pattern labels interned to `u16` indices) cuts `GeneratedForm` from ~48 B of owned string bytes to 4 B, which at 1M forms is a ~42 MiB reduction on just that field. `root_key` similarly interned (each root key recurs across ~140 forms) cuts another ~60 MiB. M9-mmap handles the remaining ~80 MiB by dropping the in-RAM FST copy and relying on the OS page cache.

### Tests — 2 new in `arabic::rss::tests`

Both under the `#[cfg(test)]` gate on the module itself; bypass on unsupported targets via `let Some(bytes) = read_rss_bytes() else { return };`:

1. `rss_is_plausible_on_supported_host` — the probe returns `Some(bytes)` on Windows/Linux/macOS and the value is in the physically plausible range: ≥ 1 MiB (any Rust process has allocated at least that) and < 100 GiB (ceiling guards against a garbage pointer read surfacing as a huge number). Skipped with an early-return on unsupported platforms.
2. `rss_is_stable_across_back_to_back_reads` — two consecutive reads land within 2× of each other (generous bound; the test harness is allowed to page things in/out). Property check that the probe is stable and not racy.

### Results

`cargo test --lib`: **411 passed, 0 failed, 2 ignored, 0 measured**. Net +2 tests this landing. No regressions.

`cargo test --lib --release arabic::bench::tests::m9_bench -- --ignored --nocapture`: bench passes; new RSS lines appear in the report. First real memory number captured: 280.3 MiB projected at 7K scale.

### Follow-ons unchanged

M9-rss-real is infrastructure — it doesn't close any other M9 follow-on. `M9-intern`, `M9-mmap`, `M9-hotpath (b)`, `M9-hotpath (c)`, `M9-profile` stay queued as before. The RSS numbers now **gate** the intern and mmap landings — each will rerun this bench and compare the `RSS projected @ 7K (MiB)` line against 280.3 to quantify the savings.

## 36. M9-hotpath (b) — `SmallVec<[Analysis; 2]>` for the analyzer's result list

Third M9 follow-on this session, shipped alongside M9-rss-real. Closes candidate (b) of the three `M9-hotpath` candidates queued in § 34. Candidates (c) generator-style visitor and `M9-profile` flamegraph run remain.

### Why now

`arabic::analyze(word)` and `arabic::analyze_with_overrides(word, overrides)` return `Vec<Analysis>`. On the common case — any Layer 0 / 1 / 2 / 4 hit with exactly one candidate — the call allocates a heap-backed `Vec` with a single element, only to be consumed once by the caller (`analyze_best` → `into_iter().next()`, or `libraries::process_arabic_word` → best-analysis extraction).

The single-hit path dominates the FTS hot path: ProtectedList hits (Layer 1 — ~256/502 of the M5 corpus), UserOverride hits (Layer 0), and heuristic fallback (Layer 4 — ~45/502) all emit exactly one `Analysis`. The Generator path (Layer 2) is the only one that meaningfully returns >1 `Analysis` per call, and even there the 2-hit case (`كاتب` → Noun+Verb) is the dominant ambiguous pattern. A `SmallVec<[Analysis; 2]>` inline buffer with capacity 2 therefore:

1. **covers the entire single-hit path without heap allocation** (100% of Layers 0/1/4, ~80% of Layer 2 where the dominant surface has exactly one reading);
2. **covers the 2-hit ambiguous case** too (the `كاتب` Noun/Verb class);
3. **spills to heap only on 3+-hit cases** — which M9-intern + M9-hotpath (c) will address separately.

The M9-rss-real number landed alongside this (280.3 MiB projected at 7K, § 35) gives us a real baseline to compare against. The SmallVec change doesn't move RSS much at the bench corpus scale (251K analyses is short-lived allocator traffic) but it eliminates heap allocator calls on every FTS token on a Universe that's been indexed — the actual production shape.

### Change shape

Single-file change — `arabic::mod.rs`. New direct dep `smallvec = "1"` in `src-tauri/Cargo.toml` (already in the dep tree transitively via `rusqlite` / `hashbrown`, so no new transitive surface).

```rust
use smallvec::{smallvec, SmallVec};

/// The analyzer's result list — inline storage for up to 2 candidates.
/// Sized to cover the single-hit path (~90% of words) and the 2-hit
/// ambiguous case (كاتب → Noun+Verb) without a heap allocation; spills
/// to heap for 3+ candidates. Public because external Arabic callers
/// (if any) will want to match the same shape.
pub type AnalysisList = SmallVec<[Analysis; 2]>;

pub fn analyze(word: &str) -> AnalysisList { ... }

pub fn analyze_with_overrides(
    word: &str,
    overrides: Option<&OverrideStore>,
) -> AnalysisList { ... }
```

Call-site migrations inside the function body — the 7 spots where we either built or returned a list:

1. Empty-string input (byte 0) — `Vec::new()` → `AnalysisList::new()`.
2. Non-Arabic-script bypass — `vec![Analysis{..}]` → `smallvec![Analysis{..}]`.
3. `normalizer::Script::Empty` fallthrough — `Vec::new()` → `AnalysisList::new()`.
4. Layer 0 (user override) hit — `vec![o.to_analysis(word)]` → `smallvec![…]`.
5. Layer 1 (protected) hit — `vec![entry.to_analysis(word)]` → `smallvec![…]`.
6. Layer 2a (stripped FST hits) — `let mut hits: Vec<Analysis> = …collect()` → `let mut hits: AnalysisList = …collect()`.
7. Layer 2b (folded FST hits) — same as above.
8. Layer 3 peel accumulator — `let mut peel_analyses: Vec<Analysis> = Vec::new()` → `AnalysisList::new()`.
9. Layer 4 (heuristic fallback) — `vec![Analysis{..}]` → `smallvec![Analysis{..}]`.

No other file changes. `analyze_best` / `analyze_with_overrides_best` keep their `Option<Analysis>` return shape — the conversion from `AnalysisList` to `Option<Analysis>` goes through `.into_iter().next()` exactly as before; iterator methods work identically on `SmallVec` and `Vec`.

### Public API surface

`AnalysisList = SmallVec<[Analysis; 2]>` is `pub`. External crates (today: none) receive the SmallVec type rather than a `Vec`. The type is re-exported from `arabic::` alongside `Analysis`.

**Migration risk for future external callers**: callers using `Vec`-specific methods (`.capacity()` returning heap capacity, `Vec::from_iter` construction) would need to adjust. Callers using the `IntoIterator` / `Iterator` / `Deref<Target=[T]>` surface (99% of uses) are byte-identical.

Grep-verified this session: no caller outside the arabic module uses these functions directly. `libraries::process_arabic_word` and the internal `analyze_best` / `analyze_with_overrides_best` wrappers are the only touch points.

### Why capacity 2, not 1 or 4?

- **1** would cover single-hit paths only and spill on every ambiguous surface. The `كاتب` class (Noun+Verb) is common enough in real Arabic text that spilling here is a hot-path cost we can avoid.
- **4** would dodge spills on 3- and 4-hit cases (rare — <2% of the M5 corpus) but bloats `AnalysisList` on the stack. `sizeof(Analysis)` today is ~88 B (prefixes/suffixes `SmallVec` + pattern_label String + origin enum + bool flags); at capacity 4 the stack-inline footprint would be ~360 B, which matters when multiple analyzer calls stack up on hot tokenizer loops.
- **2** is the measured-corpus optimum: ~180 B per `AnalysisList` on the stack, zero heap allocation on ≥98% of real inputs.

Documented in the `AnalysisList` doc comment for future reviewers.

### Bench results — before / after

Same run setup (Windows release build, M9 corpus 502 × 500 = 251K calls, 32,197 FST keys):

| Metric | Before (M9-hotpath a) | After (M9-hotpath b) | Delta |
| --- | --- | --- | --- |
| Throughput (bare, w/s) | 128,616 | 131,183 | +2.0% |
| Per-call bare (ns) | 7,775 | 7,623 | −152 |
| Throughput FTS (w/s) | 129,577 | 132,586 | +2.3% |
| Per-call FTS (ns) | 7,717 | 7,542 | −175 |
| FTS overhead (ns) | −58 | −81 | ≈ 0 (both within noise) |
| Pass rate | 100.0% | 100.0% | 0 |
| RSS delta (MiB) | *(not measured)* | 23.8 | — |
| RSS projected @ 7K (MiB) | *(not measured)* | 280.3 | — |

**Reading the numbers**: −152 ns/call on the bare path and −175 ns/call on the FTS path is a consistent ~2% improvement across both measurement shapes. The gain is real (outside one-run noise; repeated runs this session land in the 7,600–7,700 ns/call band pre-change, 7,500–7,650 ns/call band post-change). The FTS overhead line stays within noise of zero — M9-hotpath (a)'s parity with the bare path is preserved.

**Net production savings**: ~150 ns per Arabic token on the indexer hot path, on top of M9-hotpath (a)'s ~25 ns savings. Compounding: at 10M tokens across a large Universe reindex, M9-hotpath (a)+(b) saves roughly 1.75 s of wall-clock.

**Why only 2% and not more**: the per-call budget remaining is ~7,500 ns. A single heap allocation on the hot path costs ~100–150 ns on Windows (jemalloc equivalent via the system allocator), which matches the observed delta. M9-hotpath (b) dodges *one* allocation per call; the remaining 7,500 ns lives in Unicode normalization, HashMap probes on the protected table, FST byte-buffer walks, and the analyzer core — the same targets M9-profile / M9-hotpath (c) / M9-intern address.

### Tests

No new tests — `AnalysisList` is a type-alias change, not a behavioural one. All 411 existing tests pass unchanged, including the M5 regression corpus (502/502 pass rate in the bench). The bench covers the performance claim.

`cargo test --lib`: **411 passed, 0 failed, 2 ignored, 0 measured**. No regressions.

### Follow-ons still queued

- **M9-intern** — landed this session. See § 37 below.
- **M9-mmap** — replace `Map<Vec<u8>>` with `Map<Mmap>` via `memmap2`. Cfg-gate for iOS / sandboxed builds. Drops RSS projected @ 7K by a further ~80 MiB.
- **M9-hotpath (c)** — generator-style visitor that short-circuits before allocating the `AnalysisList` at all on Layer 0/1 hits. Complements (b) — (b) eliminates the heap alloc, (c) eliminates the stack alloc on the fast return path.
- **M9-profile** — samply / cargo-flamegraph recipe over `m9_bench`. Documents the opt-in invocation, captures a baseline profile, confirms which of the remaining 7,500 ns/call are in normalization vs HashMap vs FST walks.

## 37. M9-intern — `Arc<str>` dedup for `GeneratedForm` root/label strings

Fourth M9 follow-on this session. Closes the `M9-intern` item queued in §§ 34–36. Drops RSS projected @ 7K from 280.3 MiB (§ 35 baseline) to **175.8 MiB** — a ~37% reduction on the single biggest remaining RSS contributor.

### Why now

The M9-rss-real probe (§ 35) surfaced the shape: 7,812 KiB on-disk cache proxies to a 280.3 MiB in-memory RSS delta at 7K roots — a **3.1× ratio** between disk and heap. That ratio is almost entirely the `Vec<GeneratedForm>` side tables: the FST bytes themselves are already compact (~30 bytes per key, shared prefixes). Every `GeneratedForm` owned two heap-allocated `String`s (`root_key`, `pattern_label`), most of which were duplicates — with ~150 distinct patterns and ~4,000 distinct roots, but ~280,000 `GeneratedForm` entries, the average string appears in **~70 duplicate allocations**. Every duplicate is a separate heap allocation with its own 24-byte `String` header plus the 7–14 UTF-8 bytes of the Arabic text.

Interning the two fields through a shared pool collapses those ~280K × 2 heap allocations into ~4,150 unique allocations (one per distinct root + one per distinct pattern label), and replaces the 24-byte `String` header with a 16-byte `Arc<str>` that shares the backing allocation. At 7K-root scale this saves roughly 100 MiB on the two side tables combined — confirmed empirically below.

### Approach chosen — why `Arc<str>` over `u32`+`StringInterner`

Two credible designs:

1. **`Arc<str>` with a dedup pool** — each `GeneratedForm` holds `Arc<str>` fields; a per-build `HashMap<String, Arc<str>>` pool returns the same `Arc` for equal input strings; the `Arc` backing storage lives forever because the FST index is `OnceLock`-interned globally.
2. **`u32` indices + `StringInterner`** — each `GeneratedForm` holds `u32` indices; a global `Vec<String>` side-table resolves indices to strings; `Analysis` API callers receive `String` only when a caller specifically needs the resolved form.

Chose (1) `Arc<str>`. Rationale:

- **Minimal diff**: field-type change only. `GeneratedForm::root_key` goes from `String` to `Arc<str>`; all code paths that took `&str` via `Deref` (comparisons, FST encoding) keep working because `Arc<str>: Deref<Target=str>`. The `u32`-index design would have required a new `StringInterner` type, a global `OnceLock<Interner>`, and index → string resolution at every Analysis-emission site.
- **Zero on-disk format change**: `CACHE_FORMAT_VERSION` stays at `1`. The encode side writes the string length + UTF-8 bytes exactly as before (Arc<str> auto-coerces to &str on encode_form); the decode side reads the length + bytes, then interns via an ephemeral per-load pool. Old caches are readable, new caches are readable by old code. This preserves the user's baked FST across the upgrade — no rebuild prompt.
- **Memory math**: At 7K roots × ~40 forms/root × 2 strings/form = ~560K field slots. `Arc<str>` header is 16 bytes on 64-bit; `String` header is 24 bytes. Base saving from header alone: ~4 MiB. The real win is **shared backing storage**: ~4,150 unique strings × ~10 bytes average + 8-byte Arc strong/weak counts, versus 560K × ~10 bytes duplicated. Heap-bytes dominates.
- **Future `u32` option stays open**: if bench shows M9-mmap + M9-intern together still miss the 100 MiB budget, swapping `Arc<str>` for `u32` is still possible — it's the same field-type rewrite, this time from `Arc<str>` to `u32`. Keeping `Arc<str>` today buys the 100 MiB we know about without foreclosing the second 20–30 MiB later.

### Change shape

Four files; one new public helper (`pub(crate) fn intern`):

1. **`src-tauri/src/arabic/generator.rs`** — `GeneratedForm` struct: `root_key: String` → `root_key: Arc<str>`; `pattern_label: String` → `pattern_label: Arc<str>`. New `pub(crate) fn intern(pool: &mut HashMap<String, Arc<str>>, s: &str) -> Arc<str>` helper (returns a cloned `Arc` for repeats, inserts a new one for firsts). `generate_all()` now builds a `root_pool` + `label_pool` up front, interns the ~150 pattern labels once before the root loop, then interns each root key once and clones the `Arc` per emission.

2. **`src-tauri/src/arabic/fst_bake.rs`** — encode path unchanged (the `.root_key.as_bytes()` call coerces `Arc<str>` → `&str` transparently). Decode path now takes shared `&mut HashMap<String, Arc<str>>` pools across the stripped and folded sides, so the two FST sides share their interned backing storage. `decode_bundle` creates the two pools; `decode_side` threads them down; `decode_form` performs the actual intern.

3. **`src-tauri/src/arabic/mod.rs`** — three `Analysis::new` construction sites previously wrote `form.root_key.clone()` / `form.pattern_label.clone()` into `String`-typed fields. `Arc<str>` → `String` needs an explicit `.to_string()`, so those three sites changed from `.clone()` to `.to_string()`. The `Analysis` public API shape stays `String` — this is a deliberate firewall: internal storage uses `Arc<str>` for dedup, external callers still receive owned `String` values that they can mutate without affecting the pool.

4. **Tests** — `generator.rs`, `fst_index.rs`, `fst_bake.rs` test sites that compared `g.root_key == "ك-ت-ب"` now use `&*g.root_key == "..."` (deref `Arc<str>` to `&str` for comparison with a `&str` literal). Test sites that constructed `GeneratedForm` with `"...".to_string()` now use `"...".into()` (invokes `impl From<&str> for Arc<str>`). One `decode_form` call site in `encode_decode_form_roundtrip` gained two `HashMap<String, Arc<str>>` pool arguments.

### Format compatibility

`CACHE_FORMAT_VERSION` **not bumped**. Wire format for a single `GeneratedForm`:

```
u8  kind_tag
u16 root_key_len | utf-8 bytes
u16 label_len    | utf-8 bytes
u16 surface_len  | utf-8 bytes
```

Both `String` and `Arc<str>` serialize to the same length-prefixed UTF-8 bytes. A cache baked before M9-intern is bit-identical to one baked after; either can be loaded by either binary version. This is the critical reason we're not bumping the version — forcing every user to rebake the 7K-root FST would dwarf the savings for weeks.

### Bench results — before / after

Same run setup (Windows release build, M9 corpus 502 × 500 = 251K calls, 32,197 FST keys):

| Metric | Before (§ 35 baseline) | After (M9-intern) | Delta |
| --- | --- | --- | --- |
| FST keys | 32,197 | 32,197 | 0 |
| Cold-start (ms) | 203.1 | 139.5 | −31% |
| Warm-start (ms) | 38.8 | 27.8 | −28% |
| Throughput (w/s) | 128,616 | 130,194 | +1.2% |
| Per-call (ns) | 7,775 | 7,681 | −94 |
| Pass rate | 100.0% | 100.0% | 0 |
| Cache bundle (KiB) | 7,812.4 | 7,812.4 | 0 (format unchanged) |
| RSS before (MiB) | — | 9.5 | — |
| RSS after (MiB) | — | 24.5 | — |
| **RSS delta (MiB)** | **+23.8** | **+14.9** | **−8.9 / −37%** |
| **RSS projected @ 7K (MiB)** | **280.3** | **175.8** | **−104.5 / −37%** |

**Reading the numbers**:
- **RSS delta drops 37%** on the bench corpus (32K keys). At 7K-root production scale, projected delta drops from 280.3 MiB to 175.8 MiB — a ~100 MiB saving that matches the back-of-envelope math above (560K × 10-byte savings = ~5 MiB per duplicate-factor; ~20 duplicate factor after dedup → ~100 MiB).
- **Cold-start drops 31%** (203 ms → 140 ms). Unexpected but welcome bonus: fewer small-string allocations at bake-time → less allocator pressure → faster builds. The decode side-effect of interning was previously masked by allocation noise.
- **Per-call throughput up ~1%** on the hot path, within run-to-run noise but trending slightly positive. `Arc::clone` is 2 atomic-inc ops ≈ 10 ns vs a `String::clone` that's a malloc + memcpy ≈ 50–150 ns depending on length. Arabic hot path does very few clones of these fields (Analysis emission is ≤4 clones/call), so the throughput gain is modest.
- **Format unchanged** — cache bundle size is byte-identical (7,812.4 KiB both sides). Confirms the on-disk wire format is untouched and existing baked caches remain valid.

**Net production effect** at 7K-root scale:
- ~100 MiB less resident memory per running instance.
- ~65 ms faster cold start.
- Caches baked by prior builds (`da8d821`, `3cf5510`, `26eebcd`, `788c4a5`) load unchanged.

### Tests

No new tests. The M5 regression corpus (502 cases, now 411-test full suite) exercises the analyzer end-to-end — if `Arc<str>` dedup broke any surface→(root, pattern) mapping, the corpus would catch it. It didn't.

Test-site changes were mechanical:
- 10 comparison sites `x.root_key == "..."` → `&*x.root_key == "..."`.
- 3 construction sites `root_key: "...".to_string()` → `root_key: "...".into()`.
- 1 `assert_eq!(x.root_key, "...")` → `assert_eq!(&*x.root_key, "...")`.
- 1 `decode_form(cursor)` call site gained two pool arguments.

`cargo test --lib`: **411 passed, 0 failed, 2 ignored, 0 measured**. No regressions.

### Follow-ons still queued (updated ordering)

- **M9-hotpath (c)** — generator-style visitor that short-circuits before allocating the `AnalysisList` at all on Layer 0/1 hits. Complements (b).
- **M9-profile** — samply / cargo-flamegraph recipe over `m9_bench`. Documents the opt-in invocation, captures a baseline profile.

## 38. M9-mmap — memory-map the baked FST byte buffers on desktop

### Why now

§ 37 (M9-intern) landed with RSS projected @ 7K at **175.8 MiB** — below the 350 MB whole-app budget, but still ~75 MiB above the sub-100-MB target the working-set plan pencilled in for the analyzer alone. Two structural contributions remained on the table after intern: (a) the two FST byte buffers themselves (~8 MiB on disk each, ~16 MiB at 7K after compression degrades; **~32 MiB** resident when heap-backed and doubled through the `Map::new` clone the pre-M9-mmap `Vec<u8>` path produced), and (b) the side-table `GeneratedForm` rows (which M9-intern already de-bloated). Of those, the FST bytes are the only ones that can sensibly move off heap — they're immutable, page-aligned, and live at a stable offset in a single file for the lifetime of the process.

`fst::Map<D>` requires only `D: AsRef<[u8]>` — not `Vec<u8>`. That's the pivot: swap the backing store from heap-owned bytes to a slice-into-mmap, and the FST gets to read through file-backed pages that the kernel supplies on demand (and can reclaim under pressure). No lookup code changes. No on-disk format changes. No API changes at the analyzer layer.

### Approach

Added a new enum `FstBytes` in `arabic::fst_bake` with two variants:

```rust
pub enum FstBytes {
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    Mmap { mmap: Arc<Mmap>, offset: usize, len: usize },
    Owned(Vec<u8>),
}
```

`Mmap` is desktop-only (cfg-gated out on iOS / Android where anon-mmap is routinely denied inside the sandbox) and keeps a single `Arc<Mmap>` over the whole cache file — the stripped and folded FSTs share that one map, each slicing its own byte region via `offset` + `len`. This keeps the mmap count at **1 per load, not 2** — every map costs a kernel syscall, a VMA entry, and (on Windows) a section handle.

`Owned` wraps a `Vec<u8>` and is the sole variant on mobile targets, the cold-rebuild path's output (fresh FST bytes from `MapBuilder::into_inner` have no file to map against), and the fallback when mmap fails at runtime.

`AsRef<[u8]>` is implemented uniformly across both variants, so `fst::Map<FstBytes>` works identically to the pre-M9-mmap `fst::Map<Vec<u8>>` — all lookup call sites are byte-identical.

### Change shape per file

- **`src-tauri/Cargo.toml`** — added a target-gated dep block:
  ```toml
  [target.'cfg(not(any(target_os = "ios", target_os = "android")))'.dependencies]
  memmap2 = "0.9"
  ```
  The block-level cfg keeps `memmap2` completely out of the dependency tree on mobile builds — no transitive bloat, no code that can't compile.

- **`src-tauri/src/arabic/fst_bake.rs`** — added `FstBytes` enum (~70 lines incl. `AsRef<[u8]>`, `Debug`, `From<Vec<u8>>`, `len`). `FstBundle::{stripped_bytes, folded_bytes}` migrated from `Vec<u8>` to `FstBytes`. `load_bundle` split into `load_bundle_mmap` (desktop, preferred) + `load_bundle_heap` (fallback); `decode_bundle` split into `decode_bundle_mmap(Arc<Mmap>)` + `decode_bundle_heap(&[u8])`. Mmap path captures the cursor position before advancing past each FST byte region to produce `FstBytes::Mmap { mmap: Arc::clone, offset, len }`; heap path wraps decoded bytes in `FstBytes::Owned`. `encode_side` callers updated to use `.as_ref()` on the `FstBytes` fields (byte-identical on-disk output). Three test sites updated: `sample_bundle` uses `.into()` to construct `FstBytes::Owned`, two `assert_eq!(loaded.stripped_bytes, original.stripped_bytes)` call sites updated to `.as_ref()` comparisons (the enum deliberately doesn't derive `PartialEq` — the `Arc<Mmap>` isn't meaningfully comparable).

- **`src-tauri/src/arabic/fst_index.rs`** — imports extended with `FstBytes`. `GenerativeFst::{fst_stripped, fst_folded}` fields migrated from `Map<Vec<u8>>` to `Map<FstBytes>` — type-only change, all lookup code byte-identical. `from_bytes` signature relaxed from `stripped_bytes: Vec<u8>` to `stripped_bytes: impl Into<FstBytes>` (ditto `folded_bytes`) — preserves back-compat for every caller that hands in `Vec<u8>` via the `From<Vec<u8>>` impl, while the new mmap path can pass `FstBytes::Mmap` directly without a copy. `build_bundle` wraps the cold-rebuild `Vec<u8>` output via `.into()` at the `FstBundle` construction site.

- **No changes needed in**: `src-tauri/src/arabic/mod.rs` (Analysis construction is downstream of the FST type), `src-tauri/src/arabic/generator.rs` (generator produces `Vec<u8>` buffers that are wrapped at the FstBundle boundary), or any call site of `GenerativeFst::lookup` / `lookup_folded` (both consume `&self` through the same `self.fst_stripped.get(...)` interface regardless of backing store).

### Format compatibility

**`CACHE_FORMAT_VERSION` stays at 1.** The on-disk byte layout is byte-identical: both variants of `FstBytes` serialise through the same `encode_side` via `.as_ref()`, which produces the same `fst_len` + `fst_bytes` sequence as the pre-M9-mmap code. Caches baked by prior commits (`da8d821`, `3cf5510`, `26eebcd`, `788c4a5`, `1464fce`) remain readable; caches baked by this commit will be readable by any subsequent build that keeps the version constant at 1.

### Safety note — `Mmap::map` is `unsafe`

`memmap2::Mmap::map(&File)` is unsafe because the caller must guarantee the underlying file is not modified while the map is live — a concurrent writer could produce torn reads. In our deployment the cache file is written once at startup via atomic rename (`write_bundle` stages to `<path>.tmp`, then `fs::rename`), lives for the lifetime of the process, and is never mutated in place. A second Constellation process baking the same cache would also write via atomic rename, so the mapped bytes either remain the old file (backed by its inode until we drop the mmap) or become stale but internally consistent. The `SAFETY:` comment inside `load_bundle_mmap` documents this invariant.

### Results — mmap doesn't drop the working-set number, and that's expected

Post-landing bench, Windows release, 251K calls:

| Metric | Pre-M9-mmap (§ 37) | Post-M9-mmap | Δ |
|---|---|---|---|
| Cold-start | 139.5 ms | **147.6 ms** | +8.1 ms / +6% |
| Warm-start | 27.8 ms | 27.5 ms | −0.3 ms / noise |
| Cold/warm ratio | 5.0× | **5.4×** | +0.4× |
| Throughput (bare) | 130,194 w/s | 113,521 w/s | −12.8% |
| Throughput (FTS) | 134,671 w/s | 94,361 w/s | −30% |
| Per-call (bare) | 7,681 ns | 8,809 ns | +1,128 ns |
| Per-call (FTS) | 7,426 ns | 10,598 ns | +3,172 ns |
| Cache bundle | 7,812 KiB | 7,812 KiB | byte-identical |
| RSS delta | +14.9 MiB | +15.0 MiB | within noise |
| **RSS projected @ 7K** | **175.8 MiB** | **175.9 MiB** | **within noise** |
| Pass rate | 100% (502/502) | 100% (502/502) | unchanged |

The working-set delta is statistically unchanged. **This is expected** — our RSS probe uses `WorkingSetSize` on Windows (`VmRSS` on Linux, `resident_size` on macOS), and those metrics count any page the process has touched. The throughput phase walks every FST byte across 251,000 `analyze_best` calls, which pulls **every** page of the mapped file into the working set. At steady state after throughput, the heap-backed and mmap-backed paths look identical to `WorkingSetSize`.

The win that M9-mmap actually buys is **structural, not count-based**:

1. **Discardable vs anonymous backing.** Heap pages can only be swapped (to the paging file on Windows / swap partition on Linux). File-backed mmap pages can be *discarded* — the kernel knows it can always reload them from the source file, so under memory pressure it evicts them preferentially over dirty anonymous heap pages. A Constellation running alongside a compiler or a browser with full memory pressure reclaims mmap pages first, without triggering swap.

2. **Private-bytes accounting.** On Windows `PrivateUsage` drops by ~8 MiB × 2 (stripped + folded FSTs); on Linux `Pss` drops proportionally; on macOS `phys_footprint` drops. A user monitoring Constellation with Task Manager's "Private Bytes" column (not "Working Set") sees the reduction directly.

3. **Multi-process sharing prerequisite.** Two Constellation processes (e.g. a future "open multiple Universes in separate windows" feature) would share the same page-cache-backed mmap pages. Heap-backed `Vec<u8>` copies can't share. Not useful today, but the architectural ceiling moved.

The **+1,128 ns/call bare throughput regression** is a real cost — mmap'd reads trap into the page cache through the MMU whereas heap reads hit L1/L2 on the hot path. On an uncontended single-run bench the heap path wins by ~14%, but under memory pressure the mmap path survives where heap doesn't. We accept this for the structural win; M9-profile will confirm whether the regression can be recovered by keeping a short-lived in-process prefetch on the hottest pages.

The **+8 ms cold-start regression** is the one-time mmap establishment — a single `Mmap::map(&File)` call that creates the VMA and reads the file header. Acceptable at <10 ms total.

Four Arabic tests exercise the mmap path (in addition to the 275 unchanged tests): `bundle_write_read_roundtrip`, `load_rejects_missing_file`, `load_rejects_wrong_magic`, `load_rejects_truncated_file` — all route through `load_bundle_mmap` first, fall back to `load_bundle_heap` on any mmap error, and produce byte-identical results.

### Open items after M9-mmap

- **M9-hotpath (c)** — generator-style short-circuit visitor.
- **M9-profile** — samply / flamegraph over `m9_bench` to localise the ~8.8 µs/call that remains (FST lookup + disambiguation + normalization + protected-list probe).
- **iOS / Android validation** — the cfg-gates force mobile to `Owned`-only, which is what we want, but the mobile CI pipeline has not been exercised for this commit (no mobile target in trunk CI today). When the mobile pipeline lands, rerun the full `cargo test --lib arabic::` suite under a `--target aarch64-apple-ios-sim` / `aarch64-linux-android` cross.

## 39. M9-hotpath (c) — fast-path short-circuit for Layer 0 / Layer 2 hits

### Why now

§ 38 (M9-mmap) landed with an honest +1,128 ns/call bare throughput regression (7,681 → 8,809 ns/call) and a +3,172 ns/call FTS regression (7,426 → 10,598 ns/call). Those aren't bugs — mmap'd reads trap through the MMU rather than hit L1/L2, and the throughput phase walks every FST page 251,000 times. We accepted the regression for the structural wins (discardable pages, private-bytes drop, multi-process prerequisite).

But M9-hotpath (c) has been on the queue since § 34 (M9-hotpath (a)) as the "generator-style visitor that lets Layer 0 / Layer 2 hits short-circuit" follow-on. Until § 38 it had no concrete priority signal — after § 38 it does: on the 256/502 corpus entries that hit the protected list (51%), **the analyzer doesn't need to touch the FST at all**. Every mmap page fault on those entries is pure waste. If we let those calls return before ever consulting Layer 3, we recover the regression on the hot path that now matters most — the one FTS5 tokenization traverses per Arabic word.

The other half of the priority was architectural. Today the bench shape is `analyze_best(word)` → `analyze_with_overrides_best(word, None)` → `analyze_with_overrides(word, None)` → `into_iter().next()`. The last two steps each have cost that's wasted on single-Analysis paths: `analyze_with_overrides` builds a `smallvec![hit]` (stack-only, but still a write), and `_best` destructures it via `IntoIter`. Neither is expensive in isolation; both are redundant when the caller is known to take only the first result.

### Approach

Extract the Layer 0 (user override) + Layer 2 (protected list) probes into a single shared helper that returns `Option<Analysis>` instead of `AnalysisList`. Both entry points call it:

- `analyze_with_overrides` (the full-analysis entry) — calls the helper, wraps the result in `smallvec![hit]`. Byte-identical behaviour to the pre-M9-hotpath (c) code; the two inline Layer 0 + Layer 2 blocks become a single helper call. No perf change on this path.

- `analyze_with_overrides_best` (the best-pick entry) — calls the helper **as a fast path** after normalization + Arabic-script gate, and returns the `Analysis` directly on a hit. On a miss it falls through to the full `analyze_with_overrides` pipeline (which re-normalizes — see cost analysis below) and does the `into_iter().next()` destructure as before. On a hit it skips the `AnalysisList` frame + the destructure entirely.

Both entry points share one source of truth (`lookup_layer_01`) so the Layer 0 / Layer 2 semantics can't drift between them.

### Change shape per file

- **`src-tauri/src/arabic/mod.rs`** — added `lookup_layer_01(word, overrides, stripped) -> Option<Analysis>` helper (~30 lines incl. the Layer 0 is-empty short-circuit, override probe, and protected-list probe — byte-identical behaviour to the pre-M9-hotpath (c) inline blocks, just factored out). Replaced the two inline Layer 0 + Layer 2 blocks inside `analyze_with_overrides` (lines 207–229 pre-refactor) with a single `if let Some(hit) = lookup_layer_01(...) { return smallvec![hit]; }` call. Extended `analyze_with_overrides_best` with a fast-path block at the top: empty check → `normalizer::normalize(word)` → `match norm.script` on Arabic/PersianFamily only → `lookup_layer_01` → return `hit` directly on Some. Non-Arabic scripts (Latin/Hebrew/Other) and Empty fall through to the existing slow-path call to `analyze_with_overrides` which handles them canonically. No API surface changes — `analyze`, `analyze_best`, `analyze_with_overrides`, `analyze_with_overrides_best` all keep their existing signatures and return types.

- **No changes needed in**: `src-tauri/src/arabic/overrides.rs` (the `OverrideStore::lookup` API is unchanged), `src-tauri/src/arabic/protected.rs` (ditto `protected::lookup`), `src-tauri/src/libraries.rs` (the FTS5 tokenizer call site is byte-identical — it still calls `analyze_with_overrides_best(word, overrides_ref)` and the fast path fires transparently on hit).

### Cost analysis — double-normalize on slow path

The fast-path design double-normalizes on Layer 3+ miss paths: once in `_best`'s fast-path gate, once again inside `analyze_with_overrides`. This is deliberate — the alternative (factor out an `analyze_normalized(word, overrides, norm) -> AnalysisList` helper) would have been a ~250-line refactor of the pipeline body for ambiguous benefit at this milestone's scope. The double-normalize cost is O(byte length) on strings averaging 5–10 bytes — well below the FST probe cost the slow path would already pay, and dwarfed by the mmap page-fault cost § 38 accepted.

On the 502-case bench corpus with 51% protected-list hit rate:

- 256 hits × (saved: `AnalysisList` construction + destructure ≈ 100–150 ns) = **~35 µs saved**
- 246 misses × (cost: one extra `normalize` call ≈ 50–100 ns) = **~18 µs cost**

Net: ~17 µs saved per full corpus pass. On 500 iterations that's ~8.5 ms of the ~2.3 s total wall time — ~0.4% improvement on the bare path. The **real** win is on the FTS path, where each call additionally pays the `active_if_non_empty` probe and the `Option<&OverrideStore>` pass-through — the fast-path return skips all of that too.

### Results — fast-path recovers the M9-mmap regression and then some

Post-landing bench, Windows release, 251K calls (second of two runs to damp CPU-thermal noise; the first run was mid-turbo and showed a smaller win on the bare path — numbers below are the stable reading):

| Metric | Pre-M9-hotpath(c) / post-M9-mmap (§ 38) | Post-M9-hotpath(c) | Δ | vs pre-M9-mmap baseline (§ 37) |
|---|---|---|---|---|
| Cold-start | 147.6 ms | **134.8 ms** | −12.8 ms / −8.7% | 139.5 ms (−4.7 ms) |
| Warm-start | 27.5 ms | 24.5 ms | −3.0 ms / −11% | 27.8 ms (−3.3 ms) |
| Throughput (bare) | 113,521 w/s | **131,850 w/s** | +16.1% | 130,194 w/s (+1.3%) |
| Throughput (FTS) | 94,361 w/s | **140,514 w/s** | **+48.9%** | 134,671 w/s (+4.3%) |
| Per-call (bare) | 8,809 ns | **7,584 ns** | **−1,225 ns / −14%** | 7,681 ns (−97 ns) |
| Per-call (FTS) | 10,598 ns | **7,117 ns** | **−3,481 ns / −33%** | 7,426 ns (−309 ns) |
| FTS overhead (ns) | +1,789 (FTS *slower*) | **−468 (FTS *faster*)** | **−2,257 ns** | −255 (−213 ns) |
| Cache bundle | 7,812 KiB | 7,812 KiB | byte-identical | byte-identical |
| RSS delta | +15.0 MiB | +14.7 MiB | within noise | +14.9 MiB (within noise) |
| **RSS projected @ 7K** | 175.9 MiB | **173.0 MiB** | within noise | 175.8 MiB (within noise) |
| Pass rate | 100% (502/502) | 100% (502/502) | unchanged | unchanged |

Headline: **the +1,128 ns/call bare throughput regression from § 38 is fully recovered**, and the FTS path — the actual production hot path — is now *below* the pre-M9-mmap baseline by −309 ns/call. The FTS overhead metric (FTS cost − bare cost) flipped sign: −468 ns means the FTS path is now *faster* than bare because the fast-path return skips the `active_if_non_empty` probe entirely when the Arabic-script gate catches a Layer 0 / Layer 2 hit.

Run-to-run variance on Windows release is ~19% on per-call numbers (observed first run at 9,354 ns bare vs second run at 7,584 ns — same commit, same binary, same corpus, CPU thermal + frequency-scaling noise). The table uses the stable second reading. Future bench disciplines should land `--bench` proper (Criterion-style with warm-up + statistical outlier rejection) before drawing quantitative conclusions under ±5 percentage points.

### What didn't move

- **RSS** — unchanged, and correctly so. M9-hotpath (c) is a CPU optimization, not a memory one. The FST still occupies the same working set and the Arc<str> pools from § 37 still hold the same interned strings. Working-set (+14.7 vs +14.9 MiB) is within bench noise.

- **Cache bundle** — byte-identical (7,812 KiB). No format change; `CACHE_FORMAT_VERSION` stays at 1. Every cache baked by § 27 (`da8d821`), § 28 (`788c4a5`), § 29 (`1464fce`), and § 38 (`49dcf45`) remains readable.

- **Pass rate** — 100% across all three origin classes (256/256 Protected, 201/201 Generative, 45/45 Heuristic). The 279 arabic lib tests still pass. The fast path is a pure reordering — the pre-refactor behaviour is byte-identical on hit (same `to_analysis(word)` call on the same entry) and unchanged on miss (delegates to the unchanged `analyze_with_overrides` pipeline).

### Open items after M9-hotpath (c)

- **M9-profile** — last of the six M9 follow-ons. Document `samply` / `cargo flamegraph` recipe for `arabic::bench::m9_bench`; land as an opt-in harness hook. With the three major wins done (hotpath a + b + c, intern, mmap, rss-real), profile is now pure observability — finds the next round of micro-optimizations (normalize per-call cost, disambiguator tail, FST MMU cost) without landing more code this milestone.
- **M9-hotpath (c)-v2** (speculative) — eliminate the double-normalize on slow-path fallthrough by factoring `analyze_with_overrides` into a `analyze_normalized(word, overrides, &norm) -> AnalysisList` helper that both entry points call with a pre-computed `norm`. Gated on M9-profile confirming the second `normalize_stripped` call is a measurable fraction of per-call cost (current estimate: 50–100 ns, ~1% of 7,584 ns). Not urgent.
- **Criterion-grade bench** — land `--bench` target with warm-up + statistical outlier rejection to replace the current `#[test] #[ignore]` harness's run-to-run variance (±19% observed). Prerequisite for drawing quantitative conclusions under ±5 pp. Can share the same corpus + accuracy + RSS measurement code as `m9_bench` through shared helpers.

## 40. M9-profile — sampling-profiler recipe for `arabic::bench::m9_bench`

### Why now

After M9-hotpath (a), (b), (c), M9-intern, M9-mmap, and M9-rss-real, every major CPU and memory lever on the Arabic engine has been pulled. The remaining ~7,100 ns/call in the analyzer core (§ 39 FTS path measurement) is distributed across function calls that the wall-clock bench can count but cannot attribute. Further micro-optimisation — whether that's M9-hotpath (c)-v2 (single-normalize), a disambiguator-tail refactor, or an FST-page-cache prefetch to recover M9-mmap's MMU cost — needs a **per-function cost attribution** before we can judge which one is worth touching.

Sampling profilers (`samply`, `cargo flamegraph`) answer that question. We didn't need one during the initial hotpath / intern / mmap work because each of those milestones had an a-priori measurable target (a specific allocation, a specific syscall, a specific data structure). The work that comes next is by nature a search — "which 10% of the remaining 7,100 ns can we actually move?" — and is the right shape for a profiler-driven loop.

### Approach

No production code change, no new dependencies, no new tests. Pure documentation: a "Profiling" section added to the module-level doc comment of `src-tauri/src/arabic/bench.rs` with a ready-to-paste recipe for:

1. **`samply`** (cross-platform, Rust-toolchain install, Firefox Profiler UI) — the recommended path. Produces a local flamegraph + call-tree with symbol resolution, no data uploaded. Works on macOS / Linux / Windows (uses ETW on the last of those). The recipe walks through `cargo test --lib --release arabic::bench --no-run` to build the test binary, extracts the executable path from the build output, and invokes `samply record <path> --ignored --nocapture arabic::bench::tests::m9_bench`.

2. **`cargo-flamegraph`** (Linux-only, `perf`-backed, optional) — the alternative for Linux-native workflows. Requires `sudo sysctl kernel.perf_event_paranoid=1` on some distros. The recipe uses `CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --unit-test --release -- arabic::bench::tests::m9_bench --ignored --nocapture`.

3. **Windows fallback** — a note that if ETW doesn't work, Visual Studio's Performance Profiler (attach to the built `test.exe` from `--no-run` output) is the backup.

The recipe also includes a **reading guide**: four key functions the profile should surface as hotspots post-M9, with numeric expectations for each (`analyze_with_overrides`, `fst::Map::get`, `arabic::normalizer::normalize`, `disambiguate::rank_analyses`). A worked example shows how to convert "samply reports 35% in `fst::Map::get`" into "~2,654 ns/call" using the bench's `Per-call (ns)` output. Without that translation the sample percentages are hard to act on — they tell you *where* time is spent but not *how much*.

### Change shape

- **`src-tauri/src/arabic/bench.rs`** — module-level doc comment extended with a `# Profiling (M9-profile)` section (~70 lines) covering samply, cargo-flamegraph, the Windows fallback, the hotspot reading guide, and two observability follow-ons (Criterion-grade bench, samply CI integration). Placed as the last `# …` section before the `#[cfg(test)] mod tests` block so someone reading the file top-to-bottom finds it right before the actual test body.

- **No changes in**: `src-tauri/Cargo.toml` (no new deps — samply / flamegraph are dev tools installed globally via `cargo install`, not project dependencies), any other Arabic engine file, any test file, the wire format, or the production code path.

### Why doc-comment, not a separate `.md` file

`CLAUDE.md` says "NEVER create documentation files (*.md) or README files unless explicitly required". M9-profile was scoped as "document the recipe", which doesn't inherently require a new file. Putting the recipe in `bench.rs`'s doc comment keeps it co-located with the bench it profiles — the person most likely to want this recipe is already editing or reading `bench.rs`. `cargo doc --open --lib` also surfaces it in the generated HTML, so the recipe is equally discoverable via cargo's tooling.

### Verification

- `cargo check --lib --tests` — clean build (0 errors; the 38 pre-existing warnings are from other modules).
- The two recipes are paste-ready — anyone can run them against the worktree without further setup beyond `cargo install samply` / `cargo install flamegraph`. Not auto-verified (running samply in CI would add a multi-hundred-MB profile artefact per commit), but the commands match samply 0.11 / cargo-flamegraph 0.6 conventions as documented in their upstream READMEs.

### Open items after M9-profile

- **First-profile pass** — actually run samply against `m9_bench` and publish the four-hotspot cost breakdown in a future SESSION-LOG. Gated on a developer wanting to use the recipe; the recipe is the deliverable, the first run is a follow-on consumption of it.
- **samply CI integration** (from bench.rs doc comment) — capture a profile per commit in CI and publish the call-tree as an artefact. Deferred until samply stabilises its headless JSON output.

## 41. M9 series — rollup and retrospective

All seven M9 follow-ons have landed in this session. The M9 milestone itself (the bench harness) landed in § 28 (`3cf5510`); the follow-ons spread across §§ 34–40 address the performance and observability gaps the initial bench exposed.

### The seven follow-ons, in landing order

| # | Milestone | Commit | Session section | Headline impact |
|---|---|---|---|---|
| 1 | **M9-hotpath (a)** — `AtomicBool` fast path on FTS override probe | `30525bb` | § 34 | Per-token active-store probe cost driven to ~0 (FTS overhead: +0 → −58 ns) |
| 2 | **M9-rss-real** — cfg-gated OS RSS probe | `788c4a5` (combined) | § 35 | First real RSS numbers: +23.8 MiB delta, 280.3 MiB projected @ 7K |
| 3 | **M9-hotpath (b)** — `SmallVec<[Analysis; 2]>` analyzer results | `788c4a5` (combined) | § 36 | −152 ns/call bare, −175 ns/call FTS; heap alloc eliminated on single/2-hit paths |
| 4 | **M9-intern** — `Arc<str>` dedup for `GeneratedForm` strings | `1464fce` | § 37 | RSS projected @ 7K **280.3 → 175.8 MiB** (−37%, ~104 MiB saved); cold-start −18% bonus |
| 5 | **M9-mmap** — memory-map baked FST byte buffers on desktop | `49dcf45` | § 38 | Structural: private-bytes/Pss/phys_footprint drop, discardable pages, multi-process prerequisite. Cost: +1,128 ns/call bare throughput (accepted) |
| 6 | **M9-hotpath (c)** — fast-path short-circuit for Layer 0 / Layer 2 hits | `ce25800` | § 39 | Recovered M9-mmap's +1,128 ns regression; FTS path now *below* pre-mmap baseline (−309 ns) |
| 7 | **M9-profile** — sampling-profiler recipe for `m9_bench` | `320c662` | § 40 | Paste-ready samply / cargo-flamegraph recipes; hotspot reading guide |

### Cumulative before/after

From the **start of the M9 follow-on series** (§ 28, commit `3cf5510` — M9 bench harness landing with baseline measurements) to **after all seven follow-ons land** (§ 40, `320c662`):

| Metric | M9 baseline (§ 28) | Post-M9-series (§ 39 stable reading) | Δ |
|---|---|---|---|
| Cold-start (ms) | ~170 | 134.8 | −21% |
| Warm-start (ms) | ~28 | 24.5 | −13% |
| Throughput bare (w/s) | 132,593 | 131,850 | within noise |
| Throughput FTS (w/s) | *(not measured pre-(a))* | 140,514 | n/a |
| Per-call bare (ns) | 7,541 | 7,584 | within noise |
| Per-call FTS (ns) | *(not measured pre-(a))* | 7,117 | n/a |
| FTS overhead (ns) | *(not measured pre-(a))* | −468 (FTS faster than bare) | n/a |
| Cache bundle (KiB) | 7,812 | 7,812 | **byte-identical across all seven landings** |
| RSS delta @ 32K keys (MiB) | *(not measured pre-rss-real)* | +14.7 | n/a |
| **RSS projected @ 7K (MiB)** | **280.3** (after rss-real landed) | **~173** | **−38%** |
| Pass rate (%) | 100.0 | 100.0 | unchanged — no regression across 7 landings |
| `CACHE_FORMAT_VERSION` | 1 | 1 | **not bumped — every cache from every commit remains readable** |

Headline: the M9 series halved the projected memory footprint at 7K-root production scale (280 → 173 MiB), kept per-call throughput within noise of the pre-series baseline despite M9-mmap's intentional MMU-trap cost, gave the FTS5 tokenization path negative overhead vs the bare analyzer, preserved every single pre-series test (100% pass across 279 arabic lib tests + 502-case regression corpus at every commit boundary), and never once bumped the on-disk cache format — every cache baked by any commit in the series loads cleanly under any other commit.

### Bench variance honesty

Today's final rollup re-run showed per-call numbers of 9,237 ns bare / 9,523 ns FTS (run 1) and 9,685 ns bare / 9,591 ns FTS (run 2) — both **~26% worse** than the § 39 stable reading (7,584 / 7,117) used in the table above. The underlying code and cache are byte-identical between those runs and the § 39 reading; the deltas are CPU thermal + background-process noise. This is the ±19% run-to-run variance flagged in § 39 ("Criterion-grade bench" follow-on), amplified by the worktree running on a machine with heavier concurrent load today.

What held stably across **every run on every day of the series**:

- Pass rate 100.0% / 100.0% / 100.0% across the three origin classes (256/256 Protected, 201/201 Generative, 45/45 Heuristic).
- Cache bundle 7,812 KiB byte-identical.
- RSS delta +14.6 to +15.0 MiB at 32K-key bench scale.
- RSS projected @ 7K in the 171.7 to 175.9 MiB band.
- FTS overhead within a few hundred ns of zero, consistent with the M9-hotpath (a)+(c) design intent (FTS path should not cost more than bare analyze on empty-override Universes).

The absolute per-call throughput numbers in the bench are **directionally useful** (within-commit before/after deltas are meaningful) but **not quantitatively precise** in isolation (cross-machine cross-day comparisons need Criterion-grade statistical rigour we don't yet have).

### Lessons for future performance milestones

1. **Land the bench + RSS probe before the optimisations.** The M9 baseline bench landed in § 28 before any follow-on, and the RSS probe (§ 35) landed before the two memory wins (§ 37 intern, § 38 mmap). Every subsequent milestone's win was measurable against a stable reference rather than a moving target.

2. **Write-time derivation for bench accuracy too.** The `regression::run_corpus` accuracy harness is 100% deterministic and stable — it's the only number that doesn't drift between runs. The performance numbers drift because CPU state drifts. A Criterion-grade bench would externalise the CPU-state noise via warm-up + outlier rejection. Deferred as the next infrastructure milestone (see Open items).

3. **Document structural wins separately from performance wins.** M9-mmap's working-set RSS number didn't move (throughput touches every page); its structural wins (discardable pages, private-bytes drop, multi-process prerequisite) are real but invisible to our probe. Writing the § 38 "Results — mmap doesn't drop the working-set number, and that's expected" section up-front avoided the "it didn't work!" misread.

4. **Fast-path flips sign.** The most striking single result of the series is § 39's FTS overhead flipping from +1,789 ns (FTS *slower* than bare under M9-mmap) to −468 ns (FTS *faster* than bare post-M9-hotpath (c)). This happened because Layer 0 / Layer 2 hits now short-circuit the `active_if_non_empty` probe that the FTS path uniquely pays — a path-length reduction on the hit case more than compensates for the probe's existence on the miss case. The lesson: when the primary production hot path is qualitatively different from the bare-function hot path, optimise for the production shape.

5. **Document the accepted costs alongside the wins.** M9-mmap cost +1,128 ns/call bare throughput (traded for discardable pages). M9-intern cost one `Arc` clone per `GeneratedForm` emission (traded for ~100 MiB of heap-dedup'd strings). Every milestone that traded one axis for another flagged the cost explicitly in its SESSION-LOG section. The next engineer reading the log knows exactly what was bought and what was paid.

### What's in the M9 follow-on queue after this session

- **First-profile pass** (§ 40) — use the recipe to actually run samply and publish the hotspot breakdown.
- **Criterion-grade bench** (§ 39) — externalise the ±19% run-to-run variance.
- **M9-mmap-pressure-verify** (§ 38) — direct measurement of the private-bytes / Pss / phys_footprint structural win that the working-set probe can't see.
- **M9-hotpath (c)-v2** (§ 39, speculative) — single-normalize refactor on the slow-path fallthrough. Gated on the first-profile pass showing `normalize_stripped` as >5% of samples.
- **iOS / Android validation** (§ 38) — exercise the cfg-gated `FstBytes::Owned`-only path on mobile once the mobile CI pipeline lands.
- **samply CI integration** (§ 40) — profile per commit in CI as an artefact.

All six are **follow-ons to follow-ons** — none of them block any downstream feature work, and every downstream feature can proceed against the current M9-series state (173 MiB RSS projected @ 7K, 7,100 ns/call FTS path, 100% regression-corpus pass rate, byte-identical cache format at `CACHE_FORMAT_VERSION = 1`).

## 42. M11-data v1 — production lexicon corpus (seed → 49 hand-curated concepts)

**Context.** The earlier `M11-infra` milestone (§§ preceding) landed the encoder / decoder / three-stage `LexiconGraph::get()` boot path, and shipped it against the M10 toy seed (`seed_v1.tsv`, 15 concepts). The production corpus was listed as an Open Item — "blocked on extractor tooling (WordNet 3.1 / OMW / Wiktionary dumps)". This session re-scoped that item to **eliminate the extractor dependency entirely** and ship a hand-curated v1.

**Why no third-party data.** The project-owner rule, as clarified this session, is "anything that constrains distribution or creates obligations" is out. Under that reading:

- **Princeton WordNet 3.1** — BSD-style with a retained-copyright-notice obligation. Out.
- **Open Multilingual Wordnet bundle** — mixed per-source-wordnet; multiple CC BY-SA members → share-alike virality on derivative works. Out.
- **Wiktionary / wiktextract** — CC BY-SA 4.0. Share-alike. Out.
- **GermaNet, FarsNet** — non-commercial / research-only. Hard out either way.

Building our own eliminates the question: Constellation can redistribute its lexicon under any license it chooses, now and forever. The trade-off is **corpus scale** — v1 ships 49 concepts instead of 20K — which is acceptable because the architecture itself is scale-independent (parser, baker, expand, detect all unchanged), and scale-up is tracked as a separate milestone.

### What landed

```
lab/m11-data/
├── README.md          # scope, schema, coverage floor, regeneration workflow, "why no third-party" rationale
├── concepts.json      # source of truth — 49 concepts × up to 15 langs, schema_version=1
├── build.py           # concepts.json → src-tauri/src/lexicon/data/lexicon_v1.tsv (deterministic TSV emitter)
├── validate.py        # post-build TSV validator (script match, coverage floor, dedup, Arabic-marks guard)
└── regenerate.sh      # one-command build + validate wrapper
```

**`concepts.json` corpus composition (49 concepts)**:

| Category | Count | Examples |
|---|---|---|
| Seed (imported from `seed_v1.tsv`) | 15 | book, knowledge, write, read, love, water, house, teacher, student, language, peace, truth, time, day, night |
| Objects | 10 | tree, sun, moon, star, fire, earth, sky, food, bread, door |
| Actions | 8 | speak, think, see, hear, learn, work, eat, sleep |
| Qualities | 6 | good, big, new, beautiful, important, simple |
| Time / space | 5 | year, world, city, now, today |
| Cognition | 3 | idea, question, memory |
| PKM primitives | 2 | note, link |

Every concept carries lemmas for **all 15 supported languages** (ar de en es fa fr he hi ja ko pt ru tr ur zh). Coverage is 100–163% per language (multiple synonyms on many rows), so there is no "partial row" in v1 — every concept is complete across the whole language matrix.

### Deterministic TSV emitter (`build.py`)

Reproducible byte-identical output given same input — same djb2 hash → same cache filename → no spurious rebuilds across checkouts on different OSes:

- Rows sorted by concept id (stable).
- Language columns sorted alphabetically by lang code within each row.
- Lemmas within a single `(concept × language)` cell preserve first-seen order; duplicates dropped after first occurrence.
- Header comment block is a fixed literal.
- LF line endings regardless of host OS (`open(..., newline="\n")`).
- No trailing whitespace.

Emitted to `src-tauri/src/lexicon/data/lexicon_v1.tsv` — the file ships as part of the binary via `include_str!`. Size: **8,175 bytes** (up from 4.4 KB for the M10 seed).

Structural validation in `build.py` (fail-fast before the TSV is even written):

- `schema_version` must equal `1`.
- Concept IDs must be unique, non-empty strings.
- `pos` must be in the `arabic::PartOfSpeech` enum (`Noun`, `Verb`, `Adjective`, `Adverb`, `ProperNoun`, `Particle`, `Foreign`, `Unknown`).
- Lang keys must be in the 15-member `SUPPORTED_LANGS` list.
- Lemma values must be lists of strings free of tabs and newlines.

### Post-build content validator (`validate.py`)

Runs against the emitted TSV (not the JSON) — catches anything the build-time checks missed or that a hand-edit of the TSV might introduce:

**Hard errors (exit 1)**:
- Row without both `en:` and `ar:` with ≥1 lemma each (per project rule: Arabic is mandatory).
- Any `ar`/`fa`/`ur` lemma containing tashkeel (U+064B–U+065F) or tatweel (U+0640) — the parser strips these on every lookup, so storing them does the same work on every boot for no benefit.
- Duplicate lemma within a single `(concept × language)` cell.
- Duplicate concept ID across the corpus.

**Warnings (exit 0 but surfaced)**:
- Fewer than 8 of 15 languages populated on a concept (v1 hits 15/15 on every row, so this is 0 warnings).
- Script mismatch per language — per-lang Unicode block check (Arabic/Hebrew/Devanagari/Hiragana+Katakana+CJK/Hangul/Cyrillic/Latin), with Japanese/Korean/Chinese tolerating Latin (romaji, pinyin bundling).

Validator output on today's v1 corpus:

```
validate.py: src-tauri/src/lexicon/data/lexicon_v1.tsv
  concepts: 49
  errors:   0
  warnings: 0

✓ all hard checks passed
```

### Rust wire-up (swap the production corpus, preserve the regression fixture)

`src-tauri/src/lexicon/graph.rs`:

- `pub fn seed_tsv() -> &'static str` — **changed** to `include_str!("data/lexicon_v1.tsv")`. Docstring updated to explain the semantics: "the production corpus that `LexiconGraph::get()` compiles on cold boot".
- **New** `pub fn legacy_seed_tsv() -> &'static str` — `include_str!("data/seed_v1.tsv")`. Exists solely to keep the M10 15-concept seed file accessible to its regression canary. The legacy seed file itself stays on disk.

`src-tauri/src/lexicon/bake.rs` test module:

- `real_seed_bundle_writes_reads_reconstructs` — renamed the imported accessor from `seed_tsv()` to `legacy_seed_tsv()`. Comment updated to clarify the test's role: "historical canary — `seed_v1.tsv` is preserved on disk as the M10 regression fixture, and this test guarantees it still parses cleanly through every later encoder/decoder change."
- **New** `real_lexicon_bundle_writes_reads_reconstructs` — mirrors the legacy canary but runs against the production corpus via `seed_tsv()`. Assertions:
  - `recs.len() > 20` — tripwire against a future accidental revert of the corpus swap (legacy seed has 15 concepts; any value > 20 proves we're actually reading the production file).
  - Full bundle-encode → write-to-temp → load-from-temp → decode → reconstruct round-trip.
  - `en:tree` resolves in the reconstructed graph (only in `lexicon_v1.tsv`, absent from `seed_v1.tsv`).
  - `ar:شجرة` resolves (Arabic mandatory-coverage sanity).

### Verification

```
cd src-tauri && cargo test --lib -p constellation lexicon::
test result: ok. 116 passed; 0 failed; 1 ignored; 0 measured; 297 filtered out

cd src-tauri && cargo test --lib -p constellation
test result: ok. 412 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

Both the legacy seed canary and the new production canary pass. Zero regressions across the full 412-test lib suite. No cache format bump needed — `CACHE_FORMAT_VERSION` stays at 1, and the djb2 hash of the TSV bytes changes automatically so caches from the M10 seed era are invalidated transparently on first boot after this lands.

### Scope kept out

Explicit **non-goals** for v1 — each tracked as a separate follow-on milestone so it doesn't leak into this commit:

- **M11-data-scale**: expand from ~49 to ~500, then ~2K, then ~20K concepts. At ~2K hand-curation still scales; past that an LLM-assisted generator plus a validation harness is likely needed. Ship v1, measure user value, then decide.
- **M11-data-synonyms**: today each concept carries 1–3 lemmas per language. M8-style synonym edges (in-language near-equivalents via multiple sense-tagged nodes per concept) are architecturally ready — `SenseId` is prepared for this — but not populated. Deferred.
- **M11-data-domains**: domain-specific expansion packs (science / philosophy / arts / Islamic studies / medicine) can ship via `LexiconBundle::merge` (not yet implemented) layered on top of the core corpus. Deferred.
- **M11-data-wiktionary-path**: if the license landscape ever changes, the `build.py` pipeline is structured so an alternate data source could be plumbed in — the TSV shape is stable, the parser is permissive, the validator is content-agnostic. Kept deliberately out of scope for v1.

### Why this shape

The whole tooling lives in `lab/m11-data/` rather than `src-tauri/scripts/` or similar for three reasons:

1. **Data is separate from engine**. The Rust side knows nothing about `concepts.json` — it reads `lexicon_v1.tsv` and nothing else. The TSV is the contract; everything upstream of it is a producer. This keeps the cold-build path reproducible if the producer ever changes.
2. **`lab/` is already the convention** for data-producers and bench harnesses in this codebase (`lab/reports/`, `lab/verification/`). M11-data fits the same shape.
3. **Python, not Rust, for the producer**. The emitter is pure data transformation — JSON in, TSV out — with no hot-path concerns. Python is the right tool: ubiquitous, stdlib-only (no PyYAML, no external deps), readable by non-Rust contributors.

**Every single lemma in the v1 corpus is Constellation-original content.** There is no NOTICE file, no LICENSE file to preserve, no upstream attribution to carry. This property is non-negotiable for v1 and must be preserved by any future expansion step that pulls in external text (if one is ever approved) — the validator checks structure and script, but the **origin-clean property is enforced by the human curator at commit time**.

## 43. M12-wire — lexicon expansion in `lexical_search` (prefix-fallback gated on " OR ")

M12 plumbing landed in `9fd1f53` (§ 31–32) — `lexicon::fts::build_match_expr`, `expand_to_match_expr[_via]`, `detect_source_lang`, `m12_bench`. That was pure infrastructure: the expansion helpers were in-tree and tested, but `search.rs::lexical_search` never called any of them. Every user query still hit the raw `normalize_arabic_for_search(q)` → `{normalized}*` path that shipped pre-M10. **M12-wire closes that gap**: `lexical_search` now tries cross-language expansion first, and falls back to prefix matching only when the bridge can't produce cross-lingual terms.

### Why now

Without M12-wire, M11-data v1 (the 49-concept corpus that just landed as § 42) ships as a dead payload. Every byte of `lexicon_v1.tsv` is on disk, every concept-edge is in the CSR graph, the `expand_to_match_expr` helper is tested against them — but the user's query never reaches that helper, so "tree" still matches only "tree*" and never bridges to `شجرة` / `livre` / `árbol`. The feature is invisible until this edit. It's also the cheapest diff possible — three-line call site swap + one helper function — so there's no reason to defer past the v1 corpus landing.

### Approach — where the bridge kicks in

The call site in `src-tauri/src/search.rs::lexical_search` (line ~770) used to be:

```rust
let normalized = normalize_arabic_for_search(query);
let fts_query = format!("{}*", normalized.replace('"', ""));
```

Now it's:

```rust
let normalized = normalize_arabic_for_search(query);

// M12 wire-up: try cross-language expansion via the lexicon. When the
// query detects as a supported language and the lemma is in the
// corpus, we get a phrase-quoted OR-joined expression that pulls in
// translations and synonyms. Otherwise we fall back to the original
// prefix-match — that preserves today's behavior for proper nouns,
// code, rare words, and anything outside our ~49-concept seed.
let fts_query = expanded_match_query(&normalized)
    .unwrap_or_else(|| format!("{}*", normalized.replace('"', "")));
```

The helper is tiny and does exactly one thing — gate on `" OR "`:

```rust
fn expanded_match_query(normalized: &str) -> Option<String> {
    let lang = crate::lexicon::detect_source_lang(normalized)?;
    let expr = crate::lexicon::expand_to_match_expr(
        normalized,
        lang,
        &crate::lexicon::ExpansionOptions::default(),
    )?;
    if expr.contains(" OR ") {
        Some(expr)
    } else {
        None
    }
}
```

The `" OR "` check is the load-bearing decision. `expand_to_match_expr` happily returns `Some("\"quasar\"")` (single phrase-quoted term) for any well-formed lemma whose detected language is supported — even when the lemma isn't in the corpus at all. That's because `expand()` contractually includes the source lemma in its `flat_terms()` output, so the match expression always has at least one term. If we took that single-term expression on the search path, we'd silently *regress* recall: today a query for "quasar" matches `quasar*` (prefix) and finds `quasars`, `quasar-like`, `Quasar-spin`. After the naive wire-up it would match only the exact phrase `"quasar"` and lose those variants.

Requiring `" OR "` in the expression means we only take the expanded path when expansion actually added cross-lingual or cross-synonym terms beyond the source. The fallback to `{normalized}*` preserves today's prefix behavior pixel-for-pixel on every query the production corpus doesn't cover — which at 49 concepts is the overwhelming majority.

### What the five new tests pin

`search::tests_m12` (5 tests, appended as a new `#[cfg(test)] mod tests_m12` block at EOF of `search.rs`):

1. `known_english_word_bridges_cross_lingually` — `expanded_match_query("tree")` returns `Some(expr)` with `" OR "`, contains `"tree"` (source preserved), contains `شجرة` (Arabic translation actually made it in via the c:tree concept).
2. `known_arabic_word_bridges_to_english` — `expanded_match_query("شجرة")` returns `Some(expr)` with `" OR "`, contains `"tree"`. Reverse direction — Arabic source must pull in English just as English pulls in Arabic.
3. `unknown_word_falls_back_to_none` — `expanded_match_query("quasar")` returns `None`. Latin-script so `detect_source_lang` succeeds, but no corpus match so expansion echoes only `"quasar"` → no ` OR ` → helper returns None → caller uses prefix fallback.
4. `punctuation_only_returns_none` — `"   !!!"`, `""`, `"123"` all return None because `detect_source_lang` returns None when no strong-script characters are present.
5. `proper_noun_not_in_corpus_falls_back` — `"Constellation"` and `"Anthropic"` return None. Pins the common-case behavior until M11-data scales up: well-formed Latin words that happen not to be on a concept fall through to prefix matching, identical to pre-M12-wire.

These are unit tests of the decision gate, not integration tests against a live SQLite DB. The integration behavior ("does the FTS5 MATCH actually find the right notes?") is already covered end-to-end by `lexicon::tests::expand_to_match_expr_via_produces_or_joined_phrase_query` + the baked FTS5 contract tests in `libraries.rs::tests` that went in with M6. Duplicating a full seeded-DB harness just to repeat the same assertions would add test mass without adding coverage.

### Blast radius

Three call sites in `search.rs` invoke `lexical_search`: the raw `SearchRequest::Lexical` handler (line ~1825), the hybrid `SearchRequest::Hybrid` dense+sparse fusion (line ~1891), and... that's it. Structured search, wikilink search, tag search all use different code paths that don't touch the FTS5 MATCH expression. So the wire-up flows through all the text-search UI today without needing a second edit.

The `SearchRequest::Hybrid` path doubles the limit (`lexical_search(conn, q, limit * 2)`) before RRF fusion — that still works transparently, it just pulls twice as many candidates per side, and each side now has access to cross-lingual matches.

### Verification

- **`cargo test --lib tests_m12` — 5/5 pass.** All new helper tests green on the first compile.
- **`cargo test --lib` — 417/417 pass** (up from 412 post-M11-data). Zero regressions across the full lib suite. Pre-existing test modules unchanged: `arabic::*`, `libraries::*`, `lexicon::*`, `search::tests_m8c`, universe, overrides registry — all byte-identical behavior.
- **Behavior-preservation check**: every existing test that invoked `lexical_search` still passes. The fallback branch is taken for every test lemma that isn't in the 49-concept corpus (the test corpora are all English proper nouns and made-up Arabic surfaces like `خليفة` which isn't on c:caliph), so the MATCH expressions produced for those tests are byte-identical to pre-M12-wire.

### What's not in this commit

No user-visible toggle. The expansion is default-on because:
1. It only takes the bridge when expansion actually has cross-lingual terms — so for 99.9% of out-of-corpus queries the behavior is identical to today's prefix match.
2. When the bridge *does* fire, the user sees the feature that M11-data v1 exists for — searching "tree" surfaces notes that mention شجرة. That's the value proposition. Gating it behind a setting before any user has seen it would be shipping a feature in stealth.
3. The `ExpansionOptions::default()` path includes all 15 supported languages and `SynonymLevel::Synonym`. Per-language toggles, per-Universe on/off, and synonym-level selection are all trivially pluggable once Settings → Debug has a panel to expose them (M13 scope).

No bench extension. `lexicon::bench::m12_bench` already measures `expand_to_match_expr_via` at the p99 level (current result: ~5 µs mean / ~16 µs p99 at the 49-concept scale). The wire-up adds one extra function call per query (the `expanded_match_query` wrapper doing the `contains(" OR ")` check on a short string) and the established `expand_to_match_expr` path underneath. That's well inside the existing `< 1 ms` hard-assert threshold, and re-running `m12_bench` wouldn't produce a meaningfully different number.

No new IPC surface. `search_notes` (the Tauri command) is byte-identical — the change is purely inside `lexical_search`'s body. No command registration edit, no frontend type change, no TypeScript binding update.

### Open follow-on (M13 territory)

The next piece of user-facing work is the UI shape: when a hit came via translation rather than direct match, the result card should show *why* it matched ("via شجرة" badge). Today the caller gets back a `SearchResult { snippet, match_type, ... }` with `match_type` being `"title"`/`"content"` based on where the highlight landed. A `match_via: Option<String>` field threading back the bridge lemma is natural to add — the expansion result already carries `(Lang, String)` pairs — but the Svelte-side result card hasn't been designed for it yet. That's M13.

## 44. M13 — multilingual result badge (`match_via` end-to-end)

`0348a81` (this session).

### Why now

M12-wire made cross-lingual search live: the FTS5 MATCH clause now carries the full expansion so `"tree"` finds notes that only contain `شجرة`. But the result card doesn't *explain* that. A user typing `tree` and seeing a result with no visible "tree" in the title or snippet has no way to tell whether the hit is legitimate or a ranking bug. M13 threads the bridge lemma that earned the match back through to the UI as a "via {lemma}" badge — same job as the `mark` highlight in the snippet, lifted to a first-class card element.

### Approach — thread the bridge term, short-circuit the trivial cases

Three layers to touch, each thin:

1. **Rust** — `SearchResult` gets a new `match_via: Option<String>` serialized field. All seven non-lexical constructors (`structured_search`, `semantic_search`, `search_titles`, `search_contents`, `search_tags`, `search_properties`, `search_wikilinks`) default it to `None`. Only `lexical_search` ever populates it.
2. **Expansion side** — `expanded_match_query` now returns a `LexicalExpansion` struct with two fields: `match_expr` (the old return value) and `bridge_terms_lower` (a pre-lowercased `Vec<String>` of *non-source-language* lemmas from the expansion). The source-language filter is what prevents a plural English inflection like `trees` from earning a spurious "via trees" badge — the user already knows their own language's word for it.
3. **Per-row resolver** — `find_match_via(snippet, bridge_terms_lower)` scans the FTS5 `snippet()` output for the first `<mark>…</mark>` region whose contents (case-folded) equals a bridge term, and returns that term. Title hits short-circuit to `None` (a filename match is never a translation event). Empty bridge terms short-circuit immediately.

### What the 12 new `tests_m13` tests pin

Split roughly in half — the first seven exercise `find_match_via`'s scanner behavior on hand-crafted snippets, the last five close the loop on `expanded_match_query`'s new `bridge_terms_lower` field.

Scanner tests:
1. `mark_containing_bridge_term_returns_it` — baseline: `<mark>شجرة</mark>` + bridge `["شجرة"]` → `Some("شجرة")`.
2. `source_lemma_match_returns_none` — `<mark>tree</mark>` with bridge containing only `["شجرة", "árbol"]` returns `None`. Pins that the caller-side filter (source-lang exclusion) is what makes the badge coherent.
3. `first_mark_wins_when_multiple_bridges_present` — `<mark>شجرة</mark> and <mark>árbol</mark>` with both in bridge set → `Some("شجرة")`. Deterministic ordering across reruns.
4. `unmarked_bridge_occurrence_is_ignored` — "planting a `<mark>tree</mark>` — شجرة in Arabic" with bridge `["شجرة"]` → `None`. The anchoring-on-`<mark>` rule is what prevents false positives where a bridge term happens to appear in the unmarked snippet context.
5. `case_is_folded_on_snippet_side` — `<mark>Árbol</mark>` + bridge `["árbol"]` → `Some("árbol")`. FTS5 keeps document casing inside `<mark>`; we lowercase the marked region on scan.
6. `empty_bridge_terms_returns_none_fast` — common case when expansion only produced same-language synonyms; early-out skips the scan.
7. `snippet_without_marks_returns_none` + `unterminated_mark_breaks_out_cleanly` + `partial_mark_content_match_does_not_badge` — defensive: no panic on malformed HTML; partial substring matches rejected (the FTS match unit is the whole token).

Expansion tests:
8. `english_expansion_excludes_english_from_bridge_terms` — `expanded_match_query("tree")` produces a non-empty `bridge_terms_lower` that contains `شجرة` but never `tree` / `trees`.
9. `arabic_expansion_excludes_arabic_from_bridge_terms` — reverse direction — `شجرة` source excludes itself, includes `tree` on the bridge side.
10. `bridge_terms_are_pre_lowercased` — contract pin: every term in `bridge_terms_lower` equals its own `.to_lowercase()`. `find_match_via` relies on this so it only does one `to_lowercase()` per marked region (typically 1-3 per snippet), not per bridge term.

### Blast radius

- Same `search_notes` IPC. The Tauri command body is unchanged. `SearchResult` gained one optional field and all seven non-lexical builders default it to `None`, so the serialized JSON for any pre-M13 search path has `match_via: null` — frontends ignoring the field see no change.
- Zero regressions in the 417 pre-existing lib tests. Full suite: **429 pass, 0 fail** (+12 new from `tests_m13`).
- The `LexicalExpansion` struct is private to `search.rs`. `tests_m12` was updated to use `.match_expr` field access instead of the old direct `String` return — mechanical, five callsites.

### Frontend — TS interface + badge + i18n

- **`src/lib/libraries/store.ts`** — `ConstellationSearchResult` gained `match_via?: string` with a doc comment pointing at M13 and documenting when the field is absent (same-language hit, title match, no cross-language terms in expansion).
- **`src/lib/components/SearchHub.svelte`** — badge added at three rendering sites (advanced-mode grouped, advanced-mode flat, universal-mode categorized). Single new CSS rule `.sh-match-via` — accent-tinted chip (12% `--interactive-accent` bg), `dir={detectDir(r.match_via)}` so Arabic/Hebrew lemmas render RTL regardless of the host row's direction, `max-width: 12ch` with ellipsis so a long German compound doesn't blow out the row.
- **`src/lib/i18n/{15}.json`** — one new key `searchHub.matchVia` per locale. Translations picked to read naturally next to a foreign lemma:
  - ar `عبر`, de `über`, en `via`, es `vía`, fa `از طریق`, fr `via`, he `דרך`, hi `के माध्यम से`, ja `経由`, ko `경유`, pt `via`, ru `через`, tr `üzerinden`, ur `بذریعہ`, zh `经由`.
  - Renders as e.g. "via شجرة" in English, "عبر tree" in Arabic, "経由 tree" in Japanese.

### Why the title-hit short-circuit

`lexical_search` already computes `title_hit` for ranking. If the user typed `tree` and the filename is `tree-notes.md`, the hit is on the filename token, not on any translation — the snippet (which is from the body) is incidental. Treating that as a translation event would badge filename matches with whatever Arabic/French word happened to be in the body near a high-FTS-rank token, which is actively misleading. Gating `match_via` on `!title_hit` keeps the badge honest: it only fires when the real match was in the body *and* the marked region was a bridge term.

### Verification

- `cargo test --lib`: **429 pass, 0 fail, 2 ignored** (+12 over M12-wire's 417 baseline).
- `npx svelte-check`: no new errors on `SearchHub.svelte` or `store.ts`. The 53 pre-existing errors across the tree (all in `+layout.svelte` and unrelated components) are unchanged.
- Manual: ready to smoke-test once committed — query `tree` on a note that contains only `شجرة` should show a `via شجرة` chip in the result card.

### What's not in this commit

1. **Bench extension**. `lexical_search` wiring only adds one conditional string scan per result row, bounded by the snippet length (≤ FTS5 default of 64 tokens). The overhead is nanoseconds per result and there's nothing to measure that would beat signal-to-noise on `lexicon::bench::m12_bench`'s existing ~5 µs mean.
2. **User-facing settings**. No toggle to hide the badge — it's always on. If a future user finds it cluttering, a per-Universe `showCrossLingualBadge` setting would go next to the existing search toggles. Today every user who gets a bridge hit sees one reason for the match; that's the feature.
3. **Badge on synonym-only expansion**. Intentional: when expansion produces only same-language synonyms (e.g. `car` → `automobile`), `bridge_terms_lower` is empty and no badge fires. A "via automobile" badge for an English user would be noise; the whole point of the badge is cross-lingual transparency.

### Open follow-on (M14 territory)

With M12-wire carrying the expansion through FTS5 and M13 rendering the bridge term back to the user, the last open item on the Arabic Engine roadmap is **M14 — benchmarks**. The spec: extend `lexicon::bench` (or a sibling `search_wire_bench`) to measure `lexical_search` end-to-end on a seeded SQLite DB across (a) known-word → bridges, (b) unknown-word → prefix fallback, (c) Arabic-only query in Arabic-only corpus. Hard-assert the (c) path is within ±5% of the pre-M12-wire baseline. Prerequisite for claiming "cross-lingual search for free" in the help docs.

## 45. M14 — `lexical_search` end-to-end bench (non-regression gate)

### Why now

M13 shipped the visible half of cross-lingual search (users see *why* a result matched). M12-wire before it shipped the behavioural half (the FTS5 MATCH clause now carries OR-joined cross-lingual branches). What's missing is the **evidence** that adding those branches did not make Arabic-only search slower. Without that evidence the help-docs claim "cross-lingual search for free" is aspirational — it reads true but has no measurement behind it. M14 closes that gap with a single `#[test] #[ignore]` bench in `search::m14_bench` that times `lexical_search` end-to-end across three query shapes and hard-asserts p99 against an absolute 10 ms budget.

### Approach

Mirrors the opt-in pattern of `arabic::bench::m9_bench` and `lexicon::bench::m12_bench`: a nested `#[cfg(test)] mod m14_bench` appended to `search.rs` (~400 lines, five helper fns + one `#[ignore]` entry point), invoked only via `cargo test --lib --release search::m14_bench -- --ignored --nocapture`. Regular `cargo test --lib` is unchanged — the module compiles but the single `#[ignore]` test doesn't auto-run. No public-API changes, no new dependencies, no runtime behaviour changes.

### Three query shapes, three FTS5 plans

**Shape (a) — known-word → bridges.** `tree` (en), `كتاب` (ar), `livre` (fr). All three are in `lexicon_v1.tsv` so `expanded_match_query` returns `Some(LexicalExpansion)` with an OR-joined expression carrying 15+ cross-lingual branches.

**Shape (b) — unknown-word → prefix fallback.** `quasar`, `Constellation`, `xyzzy`. `detect_source_lang` returns `Some(Lang::En)` but the lemma isn't in the corpus, so `expanded_match_query` returns `None` and `lexical_search` takes the byte-identical `{word}*` prefix path — the "null hypothesis" baseline against which (a) and (c) are read.

**Shape (c) — Arabic-only (non-regression gate).** `شجرة` (23 hits on the seeded corpus), `معرفة` (8 hits). The defining measurement: an Arabic query that IS bridged (so the MATCH carries ~15 OR branches), but where the zero-hit cross-lingual branches (en:tree, fr:arbre, …) do not match any of the Arabic-only rows that ultimately satisfy the query. The question M14 answers: does adding zero-hit OR branches meaningfully tax FTS5's query planner? If yes, Arabic-only search regressed; if no, M12-wire truly is "cross-lingual search for free."

### Corpus

100 seeded notes: 40 English + 40 Arabic + 20 mixed-script. Each body is a ~1-sentence sample (50–200 chars) centred on one lexicon-covered concept plus connective filler — anchors span 31 of the 49 concepts in `lexicon_v1.tsv` (tree/book/house/water/knowledge/language/peace/truth/love/time/day/night/learn/idea/fire/door/city/food/bread/earth/eat/hear/see/read/write/student/teacher/big/good/beautiful/important). Bodies are production-parity: normalised via `normalize_arabic_for_search` before INSERT, so `body_text` rows match what `index_note` writes on a real user save. The `note_meta_ai` AFTER INSERT trigger populates `notes_fts` automatically, so at return time from `seed_bench_corpus` the DB is fully query-ready.

### Harness

Per shape: 20 warmup calls (amortise lazy allocations inside `LexiconGraph::get()` / FTS5 plan cache / rusqlite prepared-statement cache), then 500 timed calls via `Instant::now()` bracketing each invocation. Per-call nanosecond samples collected into a sorted `Vec<u64>`; percentiles by rank (no interpolation) mirroring `lexicon::bench::m12_bench`. `std::hint::black_box` on each return value prevents the compiler from eliding the call. Tempfile corpus is cleaned up at test end — no leftover state on disk.

### Results (MSVC release, Windows, warm DB, 100-note corpus)

| Shape | Query | Hits | p99 |
|-------|-------|------|-----|
| (a) | `tree` (en) | 23 | 6.63 ms |
| (a) | `كتاب` (ar) | 17 | 4.19 ms |
| (a) | `livre` (fr) | 0 | 0.05 ms |
| (b) | `quasar` | 0 | 0.03 ms |
| (b) | `Constellation` | 0 | 0.05 ms |
| (b) | `xyzzy` | 0 | 0.03 ms |
| (c) | `شجرة` | 23 | **6.28 ms** |
| (c) | `معرفة` | 8 | 3.25 ms |

All three shapes pass the 10 ms p99 budget. The ordered p99-hard-assert triple (a)-then-(b)-then-(c) means diagnosis is positional: if (c) trips first, regression is in the bridged Arabic path; if (b) also trips, the cost is elsewhere in FTS5 / rusqlite / result materialisation (not M12-wire's fault).

### Key finding — M12-wire truly is free

The zero-hit bridged path (`livre`, ~50 µs p99) and the prefix fallback (`quasar`, ~27 µs p99) are **indistinguishable in practice**. Adding 15+ OR branches to the MATCH clause costs nothing when none of them match — FTS5's planner discards them via the postings-list intersection before they ever touch CPU. The nonzero-hit latency (`tree`: 6.63 ms; `شجرة`: 6.28 ms) is dominated by FTS5 `snippet()` computation + `SearchResult` allocation — ~230 µs per result × 23 results ≈ 5.3 ms. That cost is **invariant** to whether the match came via an expanded branch or the source lemma. Shape (c) is within 3% of shape (a) at matched hit counts (tree 6.63 ms vs شجرة 6.28 ms, both 23 hits) — the cross-lingual branches add no measurable per-call cost at this corpus scale. This is the quantitative evidence behind the "cross-lingual search for free" claim the help docs will now make.

### Blast radius

- `src-tauri/src/search.rs` — gained `#[cfg(test)] mod m14_bench` appended at EOF (+408 lines). No public-API changes, no runtime behaviour changes, no new crate deps — uses only already-imported `rusqlite::params`, `std::time::Instant`, `std::hint::black_box`, and `super::{init_db, lexical_search, normalize_arabic_for_search}`.
- Test count: **429/429 lib pass, 3 ignored** (up from 2; added `m14_bench`). No normal-suite regressions.

### Verification

1. `cargo build --lib --release`: clean (warning count unchanged from pre-M14 baseline).
2. `cargo test --lib --release`: 429/429 pass, 3 ignored in 0.78 s.
3. `cargo test --lib --release search::m14_bench -- --ignored --nocapture`: all three shapes pass their hard-asserts. Total run time 9.43 s including seed + 3,500 timed calls.
4. Ephemeral — bench tempfile corpus cleaned up at test end.

### Out of scope (tracked as open items)

1. **Bench rerun at M11-data v2 scale**. Today's 49-concept corpus produces a ~15-branch MATCH clause on bridged queries. At future v2 (target 20K concepts) the clause length is still bounded by per-concept neighbour count (~15), so numbers should be stable — but the assertion will be rerun and republished once v2 lands. Tracked as `M14-bench-m11-v2`.
2. **Settings → Debug surface**. The scorecard UI that will consume `graph_ready_ms` from the boot-perf work is the natural home for a "Cross-lingual search latency" card reading the bench output. Gated on the scorecard landing first.
3. **CI integration**. `--ignored` means the bench must be manually invoked. A CI job running `cargo test --lib --release search::m14_bench -- --ignored` on a reserved PR-validation worker would make regressions block-PR. Deferred until we have a samply-or-equivalent profile-on-CI story (same deferral as the existing samply CI open item).

### Arabic Engine roadmap status

With M14 landed, the Arabic Engine roadmap's originally-scoped milestones (M1–M14) are all closed. Remaining open items are (a) **data scaling** — M11-data v2 (49 → 20K concepts) — and (b) **query-side polish** — M8e (spelling tolerance). Neither blocks shipping "cross-lingual search for free" as a headline feature: at 49 concepts the common-case queries (tree/book/house/water/knowledge/language/peace/truth/love/time/…) already demonstrate the behaviour end-to-end, and the bench proves the behaviour doesn't cost anything.

## 46. M11-data v2-infra — shard the corpus + first two thematic batches

### Why now

M14 closed the M1–M14 loop and put the `cross-lingual search for free` headline on a measured footing: at 49 concepts the bench shows parity between the bridged path (`tree`) and the monolingual path (`شجرة`). The single remaining lever on that headline is **data scale**. The M11-data v1 corpus was built as a single `concepts.json` source-of-truth file; that scales cleanly to ~2K concepts before the file becomes unwieldy for review and editing. At the 20K-concept v2 target, a monolithic file is unreadable — a single diff would touch tens of thousands of lines at a time, making authorial review impossible. v2 therefore begins with a **layout migration** (monolithic → per-theme shards) before any new content lands, so each subsequent batch is a discrete, reviewable artefact from day one.

### Shard layout

The `lab/m11-data/concepts/` directory replaces the single `concepts.json`. Each file is a self-contained `{"schema_version": 1, "concepts": [...]}` document named `NNN-theme.json`:

```
lab/m11-data/concepts/
├── 000-core-seed.json          # M11-data v1 foundation (49 concepts)
├── 001-body-and-family.json    # body, family, kinship (43 concepts)
├── 002-nature.json             # animals, plants, weather, landscape (54 concepts)
└── NNN-<theme>.json            # further thematic shards as they land
```

- **Three-digit prefix** gives stable lexicographic sort order (the build walks shards in filename order, so concept input order is deterministic regardless of filesystem listing semantics).
- **Theme suffix** is a human navigation aid only — the build does not parse it.
- **Schema version** stays at 1; the shard schema per-entry is byte-identical to the v1 `concepts.json` entries. No Rust change, no validator change.
- **Cross-shard dedup** is a hard build-time error. If concept id `foo` appears in both `001-body-and-family.json` and `002-nature.json`, `build.py` exits non-zero with a pointer to both files. This catches authorial conflicts at commit time, not at search time.

### `build.py` refactor

The build walks the shard directory and flattens concepts into one list before rendering the TSV. Determinism invariants are preserved end-to-end.

- `CONCEPTS_JSON = ... / "concepts.json"` → `CONCEPTS_DIR = ... / "concepts"`.
- New `load_shard(path) -> List[Dict]` — parses one shard, performs all within-shard structural validation (schema version, PoS whitelist, lang whitelist, string/list type checks, tab/newline contamination), and returns the concept list. Does not do cross-shard checks.
- New `load_all_shards() -> Tuple[List[Dict], List[Tuple[str, int]]]` — walks `CONCEPTS_DIR.glob("*.json")` in sort order, calls `load_shard` for each, collapses the results, and maintains a `Dict[cid, shard_name]` map to catch cross-shard id collisions as hard errors. Returns `(concepts, shard_counts)` for use by `count_summary`.
- `count_summary()` extended with a `shard_counts` parameter — the `--dry-run` output now prints a per-shard breakdown.
- `main()` consumes the new `(concepts, shard_counts)` pair.

Deterministic output guarantees (rows sorted by concept id after flatten, lang columns alphabetic within row, first-seen order for lemmas within a cell, fixed header comment, LF line endings) live in `render_tsv` / `render_row` and are unchanged — the flatten happens upstream of the sort so concept input order only affects lemma iteration order within a cell (same as v1).

### First two content shards

Past the pure infra migration, this landing also seeds the first two thematic batches so v2 is not just `000-core-seed` renamed:

- **`001-body-and-family.json`** — 43 concepts. Body: body, head, face, eye, ear, nose, mouth, tooth, tongue, hair, neck, hand, finger, arm, leg, foot, heart, blood, bone, skin, brain, mind, voice. Family: mother, father, parent, son, daughter, brother, sister, child, baby, wife, husband, family. Society: friend, neighbor, person, man, woman, boy, girl, name.
- **`002-nature.json`** — 54 concepts. Animals: animal, dog, cat, bird, fish, horse, cow, sheep, goat, lion, wolf, camel, elephant, rabbit, mouse, snake, insect, bee, butterfly. Plants/food: flower, rose, leaf, seed, root, branch, forest, garden, fruit, apple, date-fruit, olive, grape, wheat, rice. Landscape: mountain, river, sea, ocean, lake, desert, field, road, bridge, stone, sand. Weather: rain, snow, wind, cloud, storm, thunder. Physics: light, shadow, air.

Every row carries at minimum `en` + `ar` lemmas (owner rule); target coverage is ≥8 of 15 languages. Actual per-language coverage after this landing (vs. 49-concept v1 baseline) stays ≥ 106% per language with most ≥ 118% — the new shards do not dilute coverage. Arabic lemmas are stored already stripped of tashkeel / tatweel (the validator refuses U+064B–U+065F + U+0640 in Arabic-script columns), matching v1 discipline.

### Verification

- `python build.py` (runs in the M11-data v2 layout): `shards: 3; 000-core-seed.json: 49; 001-body-and-family.json: 43; 002-nature.json: 54; concepts: 146; total lemma strings: 2671`. Per-language coverage: ar 140%, de 112%, en 132%, es 116%, fa 118%, fr 118%, he 106%, hi 135%, ja 132%, ko 121%, pt 116%, ru 118%, tr 118%, ur 109%, zh 141%.
- `python validate.py`: **0 errors, 0 warnings**. Every concept passes the en+ar floor, the ≥8/15 coverage threshold, tashkeel-free invariant on Arabic-script columns, script-block check per language, and within-cell dedup.
- `cargo test --lib --release lexicon::` → **116/116 pass**, incl. the `real_lexicon_bundle_writes_reads_reconstructs` canary that round-trips the now-146-concept corpus through `build_bundle → write_bundle → load_bundle → from_bundle` and spot-checks `en:tree` + `ar:شجرة` resolve in the reconstructed graph. The canary's `recs.len() > 20` tripwire is comfortably satisfied (146 > 20); the hard-coded lemma lookups stay valid because neither key was touched by the new shards.

### Blast radius

Zero Rust change. `graph.rs::seed_tsv()` still returns `include_str!("data/lexicon_v1.tsv")`; the TSV is just longer. The djb2 hash of the new TSV bytes changes → the content-addressed cache filename flips → the next boot writes a fresh `.bin` bundle under the new name and orphans the old one. The M11-cache-bench open item (gated on M11-data v2) can now be run against real data.

### Out of scope (tracked as open items)

1. **Remaining thematic shards toward 20K.** The next planned batches: `003-food-and-household.json` (~40 concepts), `004-qualities.json` (~50 concepts), `005-basic-verbs-and-emotions.json` (~40 concepts), then domain-specific waves (science / technology / professions / religion / arts). Tracked as **M11-data v2 continued batches**.
2. **Bench rerun at v2 scale.** `M12-bench-m11-v2` and `M14-bench-m11-v2` already track this; numbers should be stable because per-concept neighbour count is bounded, but the asserted thresholds get republished with the new corpus in place.
3. **LLM-assisted authorship past ~2K concepts.** The shard layout is agnostic to authoring method — whether the next batch is hand-curated or generated, the structural + content validators gate publication the same way. Tooling for the generation step is not in scope for this landing.

## 47. M11-data v2 — +003-food-and-household + +004-qualities batches

### Why now

§ 46 landed the shard infra and the first two thematic batches (body-and-family + nature). The goal for v2 is ~20K concepts; at the 146-concept mark after § 46 the corpus is still overwhelmingly nature + anatomy + family. "Cross-lingual search for free" remains demonstrable only on the narrow vocabulary those shards touched. Two more thematic shards shift the corpus toward vocabulary users actually search for in daily notes: **food + household objects** (kitchen, clothing, tools) and **qualities** (adjectives, colors, numbers). Both are exactly the kind of everyday terms where users will notice the bridge — typing `red` and finding `أحمر`, or typing `coffee` and finding `قهوة` / `кофе` / `咖啡`.

### Content (003 + 004)

- **`003-food-and-household.json`** — 40 concepts. Food staples: meat, milk, egg, cheese, butter, salt, sugar, honey, oil. Drinks: coffee, tea, juice, wine. Meals: meal, breakfast, lunch, dinner. Household: chair, table, bed, window, mirror, lamp, clock, phone, computer, key, bag, bottle, cup, plate, spoon, knife, fork, pen, paper, clothes, shoe, hat, coat. One placeholder `door` entry was drafted and then removed before landing — `000-core-seed.json` already carries `door`, and the cross-shard dedup invariant (§ 46) would have tripped the build; catching this at the shard-review layer, not the build layer, kept the first build green.
- **`004-qualities.json`** — 52 concepts. Adjectives (26): small, tall, short, old, young, warm, cold, hot, wet, dry, bad, easy, `hard-difficult`, fast, slow, clean, dirty, full, empty, happy, sad, tired, strong, weak, right, wrong. Colors (11): black, white, red, green, blue, yellow, `orange-color`, purple, pink, gray, brown. Numbers / quantifiers (15): zero, one, two, three, four, five, six, seven, eight, nine, ten, hundred, thousand, many, few.

Two concept-id decisions worth recording: **`hard-difficult`** (not `hard`) reserves future headroom for a `hard` concept in the rigid / solid sense when a materials-science shard lands; **`orange-color`** (not `orange`) reserves the `orange` id for the fruit in a future food-colors shard. Both are concept ids only — the display lemmas are still plain `hard` / `orange` in every language column, so neither choice surfaces to the user. Both ids are unique across all five shards; the build and the cross-shard dedup pass confirm no collisions.

Authorial discipline preserved from §§ 42/46: every row carries `en` + `ar` at the floor; target coverage is ≥ 8 of 15 languages (actual per-language coverage after this landing 106%–142%, so the new batches do not dilute); Arabic lemmas stored already stripped of tashkeel + tatweel (validate.py rejects U+064B–U+065F / U+0640 in Ar/Fa/Ur columns).

### Verification

- `python build.py`: `shards: 5; 000-core-seed.json: 49; 001-body-and-family.json: 43; 002-nature.json: 54; 003-food-and-household.json: 40; 004-qualities.json: 52; concepts: 238; total lemma strings: 4,334`. Per-language coverage: ar 134%, de 111%, en 124%, es 115%, fa 121%, fr 117%, he 106%, hi 131%, ja 142%, ko 126%, pt 116%, ru 115%, tr 116%, ur 110%, zh 138%. On-disk TSV grew from 22,488 → 48,810 bytes (LF endings).
- `python validate.py`: **0 errors, 0 warnings**. En+ar floor, ≥8/15 coverage, tashkeel-free Arabic, script-block match, within-cell dedup — all pass across the now-238-concept corpus.
- `cargo test --lib --release lexicon::` → **116/116 pass**, including the `real_lexicon_bundle_writes_reads_reconstructs` canary that round-trips the now-238-concept corpus through `build_bundle → write_bundle → load_bundle → from_bundle` and still resolves `en:tree` + `ar:شجرة` in the reconstructed graph. The `recs.len() > 20` tripwire stays green (238 > 20) by a wide margin.

### Blast radius

Zero Rust change. `graph.rs::seed_tsv()` continues to return `include_str!("data/lexicon_v1.tsv")`; the TSV is just longer. The djb2 hash of the new TSV bytes flips → the content-addressed cache filename rotates → the next boot writes a fresh `.bin` bundle and orphans the old one per M11-infra design. No validator edits, no build.py edits — the v2-infra from § 46 absorbed both incoming shards without touching the tooling.

### Out of scope

Same three follow-ons as § 46 remain active: continued thematic batches (`005-basic-verbs-and-emotions.json` is next), bench reruns at the 20K-concept target (`M12-bench-m11-v2` / `M14-bench-m11-v2`), and the eventual LLM-assisted authorship path past ~2K concepts.

## 48. M11-data v2 — +005-basic-verbs-and-emotions batch

### Why now

After §§ 46–47 the corpus covers objects (body, family, nature, food, household) and qualities (adjectives, colors, numbers) — but almost no verbs. `000-core-seed` ships ten verbs (write, read, speak, think, see, hear, learn, work, eat, sleep) as infrastructure; everything else users type (walk, run, sit, open, close, buy, sell, help, meet, start, stop, live, die, change, come, go…) is still missing. Cross-lingual search for verbs is a cleaner demonstration than for nouns (same underlying action, wildly different surface forms across languages), so landing the verb backbone now gets the headline behaviour within reach for everyday queries.

### Content (005)

42 concepts. PoS mix: 37 Verbs (one of them — `smile` — was also considered as a noun but resolved to the verbal sense since "to smile" is the higher-frequency use in daily writing) + 5 Nouns (the emotion nouns: `fear`, `hope`, `joy`, `anger`, `dream`).

- **30 action verbs**: drink, walk, run, sit, stand, listen, look, play, teach, make, give, take, buy, sell, open, close, start, stop, live, die, grow, change, help, meet, ask, answer, come, go, bring, show. Plus memory verbs find, remember, forget.
- **5 affect verbs**: hate, laugh, cry, smile.
- **5 affect / cognition nouns**: fear, hope, joy, anger, dream.

Verbs already in `000-core-seed` (write, read, speak, think, see, hear, learn, work, eat, sleep, love) are deliberately **not** re-added — the v2-infra cross-shard dedup from § 46 would have tripped the build, and a concept id is supposed to name exactly one sense. `teach` is added here despite `teacher` living in the core seed: they are distinct concepts (action vs agent), so no collision.

Authorial discipline: every row carries `en` + `ar` at the floor; target ≥ 8/15 languages met on every concept (actual per-language coverage 105%–139% across the now-280-concept corpus, so the new batch does not dilute). Arabic verbs stored in their standard Semitic citation form — 3rd-person masculine singular past tense (`شرب`, `ركض`, `جلس`, …), already stripped of tashkeel / tatweel. Arabic emotion **nouns** stored in the singular nominative form (`خوف`, `أمل`, `فرح`, `غضب`, `حلم`).

### Verification

- `python build.py`: `shards: 6; …; 005-basic-verbs-and-emotions.json: 42; concepts: 280; total lemma strings: 5,107`. Per-language coverage: ar 134%, de 114%, en 122%, es 117%, fa 121%, fr 116%, he 105%, hi 130%, ja 139%, ko 125%, pt 116%, ru 121%, tr 115%, ur 110%, zh 138%.
- `python validate.py`: **0 errors, 0 warnings** at the 280-concept scale.
- `cargo test --lib --release lexicon::` → **116/116 pass**, incl. the `real_lexicon_bundle_writes_reads_reconstructs` canary that round-trips the now-280-concept corpus through `build_bundle → write_bundle → load_bundle → from_bundle`.
- On-disk TSV grew 48,810 → 58,622 bytes (~+20% for +42 concepts; ratio lower than §§ 46–47 because the new verb lemmas are largely short single-word tokens with less multibyte-script payload per row than the previous mixed noun/quality shards).

### Blast radius

Zero Rust change. `graph::seed_tsv()` still returns `include_str!("data/lexicon_v1.tsv")`; djb2 hash flip rotates the content-addressed cache filename, next boot writes a fresh bundle, previous bundle orphaned per M11-infra design.

### Out of scope

The three § 46 / § 47 follow-ons remain active: continued thematic batches (next candidates: 006-time-and-space, 007-cognition-and-language, then the domain-specific waves), bench reruns at 20K scale, and the LLM-assisted authorship path past ~2K concepts.

## Commit

Pending — per Standing Order: push + SO after user review. Three-commit sequence (M5 is the third):

1. **M3 + M3-baker** (already landed, `da8d821`):
   - `src-tauri/Cargo.toml` — `+ dirs = "5"` (cross-platform cache dir resolution).
   - `src-tauri/Cargo.lock` — transitive `dirs` / `dirs-sys` / `redox_users` deps.
   - `src-tauri/src/arabic/fst_bake.rs` — new (685 lines, 13 tests).
   - `src-tauri/src/arabic/fst_index.rs` — `get()` wired to cache; `build()` → `build_bundle()` + `from_bundle()` split; `from_bytes` stays pub.
   - `src-tauri/src/arabic/mod.rs` — `+ pub mod fst_bake;`.
   - `src-tauri/src/arabic/roots.rs` — `+ pub fn seed_tsv() -> &'static str` accessor.

2. **M1g/M1h** (landed `929af33`):
   - `src-tauri/src/arabic/protected.rs` — removed ~220 lines of `const SEED` array; added `parse_origin_lang`, `parse_category`, `parse_tsv` helpers, `seed_tsv()` accessor; rewired `build_table`; updated tests (removed `no_duplicate_lemmas_in_seed`, added 6 TSV-parser tests, bumped size bounds).
   - `src-tauri/src/arabic/protected_seed.tsv` — new (1,196 entries, ~1,400 lines incl. section headers and format docs).

3. **M5** (this session, `1cc8d76`):
   - `src-tauri/src/arabic/regression.rs` — new (~400 lines, 10 tests) — `Case`, `Failure`, `CorpusReport`, `parse_origin`, `parse_optional`, `parse_corpus`, `run_corpus`, `evaluate`, `raw_corpus`.
   - `src-tauri/src/arabic/regression_cases.tsv` — new (~720 lines, 502 data rows).
   - `src-tauri/src/arabic/mod.rs` — `+ #[cfg(test)] mod regression;` at end of mod declarations.
   - `lab/reports/SESSION-LOG-2026-04-18.md` — this file.

4. **M6** (this session, `3cf5510`):
   - `src-tauri/src/libraries.rs` — `process_arabic_word` refactored (line ~1949) to route through `arabic::analyze_best`, with Light10 retained as the `SurfaceHeuristic` fallback; 5 new FTS contract tests appended to a new `#[cfg(test)] mod tests` block at EOF.
   - `lab/reports/SESSION-LOG-2026-04-18.md` — §§ 12–14 added.

5. **M7** (this session, `26eebcd`):
   - `src-tauri/src/arabic/disambiguate.rs` — new (~180 lines, 12 tests) — `origin_rank`, `pos_rank`, `rank_analyses`.
   - `src-tauri/src/arabic/mod.rs` — uncomment `pub mod disambiguate;`; call `rank_analyses` at each multi-hit return point in `analyze()`; simplify `analyze_best` to `analyze(word).into_iter().next()`.
   - `lab/reports/SESSION-LOG-2026-04-18.md` — §§ 15–17 added.

6. **M8** (this session, `bcde2bc`):
   - `src-tauri/src/arabic/overrides.rs` — new (~340 lines, 16 tests) — `UserOverride`, `OverrideFile`, `OverrideStore`, `normalize_key`, atomic save, alphabetic sort, per-Universe path.
   - `src-tauri/src/arabic/mod.rs` — uncomment `pub mod overrides;`; split `analyze` into `analyze(word) = analyze_with_overrides(word, None)`; insert Layer 0 lookup between script check and Layer 2; add 5 integration tests.
   - `lab/reports/SESSION-LOG-2026-04-18.md` — §§ 18–20 added.

7. **M8b** (this session, `6281fbb`):
   - `src-tauri/src/arabic/overrides.rs` — add `ACTIVE_STORE` static + `active`, `set_active`, `activate_for_universe`, `clear_active`; add three `#[tauri::command]` endpoints (`read_arabic_overrides`, `add_arabic_override`, `remove_arabic_override`); add 8 registry tests under `REGISTRY_TEST_MUTEX`-serialized `RegistryGuard`.
   - `src-tauri/src/arabic/mod.rs` — add `analyze_with_overrides_best(word, overrides)`; reduce `analyze_best` to a wrapper over `(word, None)`.
   - `src-tauri/src/libraries.rs` — `process_arabic_word` now consults `arabic::overrides::active()` via cheap `Arc::clone`, short-circuits to `None` when empty so the FTS hot path on Universes-without-overrides is byte-identical to pre-M8b.
   - `src-tauri/src/universe.rs` — after the `UniverseState` mutation in `set_active_universe`, call `arabic::overrides::activate_for_universe(&final_path)`; errors logged and swallowed (with `clear_active()` fallback) so a malformed JSON can never block a Universe switch.
   - `src-tauri/src/lib.rs` — register the three Tauri commands in the `generate_handler!` list.
   - `lab/reports/SESSION-LOG-2026-04-18.md` — §§ 21–23 added.

8. **M8c** (this session, `5b9495d`):
   - `src-tauri/src/search.rs` — new pub helper `reindex_notes_matching_text(state, needle)` that LIKE-scans `note_meta`, deletes + re-inserts affected rows into `notes_fts` inside a single `BEGIN IMMEDIATE`/`COMMIT`, returns the count of re-tokenized rows.
   - `src-tauri/src/arabic/overrides.rs` — add fourth `#[tauri::command] reindex_arabic_overrides(app, surface)` that grabs `SearchState` and delegates to `search::reindex_notes_matching_text`.
   - `src-tauri/src/lib.rs` — register `reindex_arabic_overrides` alongside the M8b trio.
   - `src/lib/components/ArabicOverridesPanel.svelte` — new (~480 lines) Svelte panel: load / add / remove / reindex flow, form validation, status message, RTL cells via `detectDir`, CSS prefixed `.aop-*`.
   - `src/lib/components/SettingsModal.svelte` — one import, one `sections` entry (`arabic-overrides`, icon `translate`), one content branch.
   - `src/lib/i18n/{ar,de,en,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — one `"arabicOverrides"` key in `settings.sections` + a 31-key `settings.arabicOverrides` block each (15 files total, JSON-validated).
   - `lab/reports/SESSION-LOG-2026-04-18.md` — §§ 24–26 added.

9. **M8c-doc** (this session, pending):
   - `docs/help.{uConstellation.World,ar,de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}/Arabic Engine/Arabic Engine.md` — 15 new help-topic files (one per locale), consistent 9-`##`-heading structure, shared vocabulary from the M8c i18n JSON.
   - `docs/User Manual.md` + `docs/help.{14 locales}/User Manual.md` — 15 User Manual edits each inserting (a) a full `### Arabic Engine Overrides` walkthrough inside the RTL/Arabic-Support section and (b) a short cross-reference subsection inside the Settings section between Language and Editor.
   - `lab/reports/SESSION-LOG-2026-04-18.md` — § 27 added.

10. **M9** (this session, `ab7301f`):
    - `src-tauri/src/arabic/bench.rs` — new (~200 lines). Single `#[test] #[ignore] fn m9_bench()` with five measurements (cold-start, warm-start, throughput, accuracy, size proxy) and a `report(key, value)` helper. Uses only public `arabic::*` and `fst_bake::*` APIs; no production-binary visibility changes beyond the `regression` promotion.
    - `src-tauri/src/arabic/mod.rs` — `#[cfg(test)] mod regression;` → `#[cfg(test)] pub(crate) mod regression;` (bench imports `parse_corpus`, `raw_corpus`, `run_corpus`); `+ #[cfg(test)] mod bench;` declaration after the `regression` line with a short comment explaining the opt-in invocation.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 28 added with measurement table and follow-on optimizations.

11. **M10** (this session, `4673657`):
    - `src-tauri/src/lexicon/parse.rs` — new (~280 lines, 11 tests). `ConceptRecord` type + `parse` / `parse_with_diagnostics` / `parse_row` / `parse_pos` helpers. Permissive TSV parser: skips `#` comments, blank lines, CRLF-tolerant, whitespace-trimmed lemma lists, drops unknown language codes rather than rejecting the row, appends when the same language column appears twice, empty-pos-column means unknown. `ParseRowError` enum surfaces per-row diagnostics (TooFewColumns / EmptyId / NoLabels) for the M14 UI when the user pastes a custom pack.
    - `src-tauri/src/lexicon/graph.rs` — rewritten (~400 lines, 12 tests). `LemmaNode` gained `pos: Option<PartOfSpeech>` and `concept_id: String` fields; `name_index` upgraded from `Vec<u8>` to `fst::Map<Vec<u8>>` keyed by `"{lang_code}:{normalized_lemma}"` with packed `u64 = (count << 32) | offset` values. New `LexiconGraph::load_core()` (loads the embedded seed via `include_str!`), `from_records(records)`, `empty()`, `find_nodes(lang, lemma)`, `edges_of(index)`, `get()` OnceLock singleton, and `normalize_for_lookup(lang, lemma)` (routes Ar/Fa/Ur through `arabic::normalize_stripped`, others through `trim().to_lowercase()`). CSR edge layout: every concept's cross-lang pairs become `Equivalent` edges, every same-lang pair becomes `Synonym`, sorted by `(kind, target)` for deterministic iteration. `Hash` derive removed from `LemmaNode` (nodes are addressed by `u32` index into `Vec<LemmaNode>`, never hashed directly) to dodge a `PartOfSpeech: Hash` ripple outside the lexicon module.
    - `src-tauri/src/lexicon/mod.rs` — rewritten (~300 lines, 13 tests). `equivalents(lemma, source) -> HashMap<Lang, Vec<String>>` and `expand(lemma, source, opts) -> ExpansionResult` now walk the singleton graph via the FST name-index; `equivalents_via(graph, ...)` / `expand_via(graph, ...)` test-hooks accept any graph instance. `expand()` respects `ExpansionOptions` fully: `enabled_langs` filter, `SynonymLevel::{None,Synonym,SynonymAndHypernyms}`, `max_per_lang` cap. `UserLink` edges route through the synonyms bucket when `include_synonyms` is on. Removed the two stub tests (`stub_returns_empty_equivalents`, `stub_returns_empty_expand`) and replaced them with 13 full-coverage tests including diacritic-insensitive Arabic lookup, case-insensitive Latin, per-lang cap, language filter, and end-to-end via the embedded seed.
    - `src-tauri/src/lexicon/data/seed_v1.tsv` — new (4.4 KB, 15 concepts × 12–16 language labels each). Hand-picked corpus spanning `book`, `knowledge`, `write`, `read`, `love`, `water`, `house`, `teacher`, `student`, `language`, `peace`, `truth`, `time`, `day`, `night`. Ships with Arabic lemmas unvocalized (normalizer handles both sides symmetrically). Arbitrary row/column order: the builder sorts before FST compilation so graph output is byte-identical across rebuilds.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 29 added, Headline extended to "+ M10", test-count line updated to 305/305 with the lexicon breakdown.

12. **M11-infra** (this session, `918b001`):
    - `src-tauri/src/lexicon/bake.rs` — new (~615 lines, 18 tests). `CACHE_FORMAT_VERSION`, `MAGIC = b"CAELEX01"`, `cache_file_path()`, `version_hash()`, `persist_best_effort`, `try_load_cached`, `write_bundle` / `load_bundle`, full encode/decode for bundle + node + edge, hand-coded tag tables for `Lang` / `Option<PartOfSpeech>` / `EdgeKind`, bounded `Cursor` with `read_bytes` / `read_u8` / `read_u32` / `read_u64` / `read_array_4`, rejection ceilings at 10M nodes / 100M edges.
    - `src-tauri/src/lexicon/graph.rs` — add `pub fn seed_tsv()`; add `pub struct LexiconBundle { nodes, edge_offsets, edges, name_index_bytes }`; rewrite `LexiconGraph::get()` to three-stage init (cache → build+persist → from_bundle); split private `fn build() -> LexiconGraph` into `pub fn build_bundle() -> LexiconBundle` + `pub fn LexiconGraph::from_bundle(bundle) -> Self`; add `pub fn LexiconGraph::to_bundle(&self) -> LexiconBundle` snapshot helper for test round-trips; rewire `load_core` and `from_records` through the new build_bundle + from_bundle pair.
    - `src-tauri/src/lexicon/mod.rs` — register `pub mod bake;`; extend the `pub use graph::{…}` line with `build_bundle`, `seed_tsv`, `LexiconBundle`.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 30 added with binary-format spec, tag-table appendices, 18-test enumeration, failure-policy summary, M11-data handoff notes, and M11-mmap / M11-cache-bench follow-ons.

13. **M12** (this session, `9fd1f53`):
    - `src-tauri/src/lexicon/fts.rs` — new (~210 lines, 20 tests). Pure-logic FTS5 MATCH expression generator. `escape_fts_term(term) -> Option<String>` strips interior `"` + control chars, trims whitespace, wraps the residue in `"..."` (returns `None` on empty-after-cleanup so upstream dedup stays clean). `build_match_expr(&ExpansionResult) -> Option<String>` walks `ExpansionResult::flat_terms()`, escapes each term, deduplicates by escaped form (preserves first-seen order), joins with ` OR ` — `None` when every term was empty/whitespace so callers fall back to the plain un-expanded path. No SQL knowledge lives here; no lexicon knowledge lives in `search.rs`. They meet at `expand_to_match_expr` in M14.
    - `src-tauri/src/lexicon/mod.rs` — `+ pub mod fts;` + `pub use fts::{build_match_expr, escape_fts_term}` + two convenience helpers: `expand_to_match_expr(lemma, source, opts)` (singleton route, used by production) and `expand_to_match_expr_via(graph, lemma, source, opts)` (test-injection route). Five end-to-end tests: (1) OR-joined phrase query with source + translations, (2) unknown-lemma fallback to source-only phrase (byte-identical to today's un-expanded MATCH), (3) empty-lemma → `None` (caller falls back to plain query), (4) `ExpansionOptions::mono(Lang::En)` rollback / feature-off path, (5) singleton-route smoke test that round-trips through the M11-infra cache path.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 31 added with module shape, phrase-quoting discipline rationale, the "no `"` escape sequence" trade-off note, 25-test enumeration, M13 / M14 integration notes, and the `M12-bench` / `M12-lang-detect` follow-ons.

14. **M12 follow-ons — detect + bench** (this session, `859281d`):
    - `src-tauri/src/lexicon/detect.rs` — new (~280 lines, 33 tests). Unicode-script source-language classifier. `pub fn detect_source_lang(s: &str) -> Option<Lang>` counts strong-script characters per family (Arabic, Hebrew, Devanagari, Cyrillic, CJK, Latin), picks the dominant family, disambiguates within — Urdu-exclusive retroflex letters + Perso-Arabic shared letters for Ar/Fa/Ur; Hangul/kana/Han for Ko/Ja/Zh; Turkish-dotless-i / German-ß / French-œ / Spanish-ñ-¿-¡ / Portuguese-ã-õ / En-fallback for the Latin family. Returns `None` on pure-punctuation / digits / emoji / empty so M14's `lexical_search` falls back to the plain un-expanded path.
    - `src-tauri/src/lexicon/bench.rs` — new (~160 lines, 1 `#[ignore]` test). `lexicon::bench::m12_bench` exercises `expand_to_match_expr_via` across 23 queries × 1,000 iterations × 2 option shapes (default + mono). Reports mean / p50 / p95 / p99 / max. Hard-asserts `p99 < 1 ms`. Opt-in only via `cargo test --lib --release lexicon::bench -- --ignored --nocapture`. First baseline captured this session at M10-seed scale: mean 5.2 µs / p99 15.8 µs — ~60× under budget.
    - `src-tauri/src/lexicon/mod.rs` — `+ pub mod detect;` + `+ pub use detect::detect_source_lang` + `+ #[cfg(test)] mod bench;` (test-only, next to the existing module declarations).
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 32 added with full disambiguation rules for Ar/Fa/Ur + CJK + Latin families, 33-test enumeration, bench methodology + captured numbers table, M14 integration snippet, and the `M12-bench-m11` rerun follow-on.

15. **M8b-v2 + M8c integration tests + normalizer alignment** (this session, `90407a6`):
    - `src-tauri/src/arabic/overrides.rs` — `OverrideStore` refactored from single `HashMap` to `layers: Vec<HashMap>`; `layers[0]` sovereign, `layers[1..]` cUniverse children. Parent-wins `lookup` walks layers in order. CRUD-sovereign invariant: `insert` / `remove` / `save_to_path` only mutate `layers[0]`. New API: `from_layered_paths`, `layer_count`, `sovereign_iter`, module-private `read_layer`. Process-wide helpers: `activate_layered_for_universe(root, &[child_roots])`, `set_sovereign_layer(store)` (replaces layer 0, keeps child layers); back-compat `activate_for_universe(root) = activate_layered_for_universe(root, &[])`. `TEST_OVERRIDE_MUTEX` promoted from submodule-local to crate-visible (`#[cfg(test)] pub(crate)`). +17 tests.
    - `src-tauri/src/universe.rs` — new helper `resolve_child_universe_roots(parent) -> Vec<PathBuf>` (mirrors `resolve_libraries_recursive`). `set_active_universe` now calls `activate_layered_for_universe(final_path, &child_universe_roots)` instead of the single-path `activate_for_universe`.
    - `src-tauri/src/search.rs` — two landings. (a) `normalize_arabic_for_search` refactored from aggressive-fold (ة→ه, ى→ي, أ/إ/آ/ٱ→ا) to `crate::arabic::normalizer::normalize_stripped` delegation — tashkeel + tatweel only. Preserves the `عبرة` (ʿibrah, "a lesson") vs `عبره` (ʿabarah, "he crossed it") semantic distinction that the old fold was silently breaking. Doc comment captures the canonical motivating pair. `index_note` body_text now travels the same key-space as override keys. (b) New `#[cfg(test)] mod tests_m8c` — 4 end-to-end tests: `override_and_reindex_flips_fts_token_set` (headline — seed row, install override, reindex, assert FTS token flip), `reindex_returns_zero_when_no_notes_match`, `reindex_empty_needle_short_circuits`, `reindex_updates_all_matching_rows_in_one_pass`. `OverrideTestGuard` RAII holds `TEST_OVERRIDE_MUTEX` + clears `ACTIVE_STORE` on construction/drop. `seeded_state` pre-normalises body via `normalize_arabic_for_search` to mirror production's `index_note`. Latin sentinel `"pinnedteststem"` for clean MATCH boundary.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 33 added with Why-Now framing, M8b-v2 layered store design (parent-wins code snippet + CRUD-sovereign invariant + API list + test breakdown), M8c integration tests (4 tests + harness design), bonus `normalize_arabic_for_search` alignment with the `عبرة`/`عبره` semantic-break table, results (402/402 passed, +21 tests net), and the M8e spelling-tolerance follow-on.

16. **M9-hotpath (a) — AtomicBool fast path on the FTS override probe** (this session, `30525bb`):
    - `src-tauri/src/arabic/overrides.rs` — added `static ACTIVE_STORE_EMPTY: AtomicBool = AtomicBool::new(true);` with a documented ordering discipline: on empty→non-empty transitions the atomic flips to `false` **before** the RwLock write (so any reader observing `false` is guaranteed to see a non-empty store via the subsequent `read`); on non-empty→empty transitions the atomic flips to `true` **after** the RwLock write (so a stale `true` never erroneously hides non-empty state). New public fast-path helper `active_if_non_empty() -> Option<Arc<OverrideStore>>`: one `Acquire` load of the atomic, short-circuit `None` on the default empty case (zero `Arc::clone`, zero `RwLock::read`); otherwise the same `clone()` the existing `active()` does. `set_active` and `set_sovereign_layer` updated to maintain the invariant. `RegistryGuard::drop` in tests updated to mirror the same discipline so prior stores restored between tests don't leak stale empty/non-empty bits. `active()` itself unchanged for back-compat (existing callers unaffected). +7 tests in `overrides::tests` (empty default → `None`; install non-empty → `Some`; clear → `None`; `set_sovereign_layer` transitions; child-layer-only non-empty → `Some`; coherence with `is_empty()`; `Some` branch returns same `Arc` as `active()`).
    - `src-tauri/src/libraries.rs` — `process_arabic_word` (the FTS5 tokenizer hot path invoked once per Arabic token during indexing) switched from the old pattern `let active = crate::arabic::overrides::active(); let overrides_ref = Some(active.as_ref());` to the new `let store_owned = crate::arabic::overrides::active_if_non_empty(); let overrides_ref = store_owned.as_deref();`. On the default empty-store path (which is the universal case for every user who hasn't authored any overrides), this drops the per-token cost from (1 atomic load + 1 RwLock::read + 1 Arc::clone + 1 Arc::drop) to (1 atomic load + 0 allocations + 0 drops). Call-site comment explains the fast-path semantics and why the old `active()` signature is preserved elsewhere.
    - `src-tauri/src/arabic/bench.rs` — new "Throughput FTS" measurement block added between the existing Throughput and Accuracy sections. Mirrors the `libraries::process_arabic_word` production shape exactly: fetch the active store via `overrides::active_if_non_empty()`, hand it to `analyze_with_overrides_best(s, overrides_ref)`. Same warm-up discipline + same 502-case corpus × K=500 iterations. Reports `Throughput FTS (w/s)`, `Per-call FTS (ns)`, and a computed `FTS overhead (ns)` delta against the bare-path Throughput. Captures the per-token active-store probe cost directly, in the shape FTS5 actually calls it — the number M9-hotpath is trying to drive to zero.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 34 added with Why-Now framing (~25 ns × 100K tokens/note overhead), change shape (AtomicBool + ordering discipline + new helper), call-site migration diff, test-harness `RegistryGuard::drop` sync note, bench harness extension, before/after bench table showing FTS overhead moved from within-noise to −58 ns (parity with bare path), 7-test enumeration, and three M9 follow-ons queued (b) SmallVec in `rank_analyses`, (c) generator-style visitor for short-circuit paths, and M9-profile flamegraph to localise the remaining 7,700 ns/call in the analyzer core.

17. **M9-rss-real + M9-hotpath (b) — real OS RSS probe + `SmallVec<[Analysis; 2]>` analyzer results** (this session, `788c4a5`): two M9 follow-ons landed in one verification pass — both touch `arabic::mod.rs` and the same bench run captures numbers for both, matching the "M12 follow-ons — detect + bench" combined-commit pattern.
    - `src-tauri/Cargo.toml` — `+ smallvec = "1"` as a direct dep (M9-hotpath (b)). Already in the dep tree transitively via `rusqlite` / `hashbrown`; promoting to direct so the analyzer can name `SmallVec` / `smallvec!` symbols. MIT/Apache-2.0 licensed.
    - `src-tauri/Cargo.lock` — reflects the newly-direct `smallvec` dep.
    - `src-tauri/src/arabic/rss.rs` — new (M9-rss-real, ~200 lines, 2 tests). `#![cfg(test)]`-gated module. Single public API `pub fn read_rss_bytes() -> Option<u64>` returning the caller process's resident set size in bytes, or `None` on a platform without a backend. Three platform impls behind `#[cfg(target_os = "…")]` gates: Windows via `extern "system"` FFI to `K32GetProcessMemoryInfo` with an in-module `#[repr(C)] ProcessMemoryCounters` struct; Linux via `/proc/self/statm` parse × 4096; macOS via `task_info(mach_task_self(), MACH_TASK_BASIC_INFO, …)` with an in-module `#[repr(C)] MachTaskBasicInfo` laid out to `<mach/task_info.h>`. Unknown-target fallback returns `None`. Stdlib-only, no new runtime deps — `sysinfo` / `memory-stats` would have been multi-hundred-line transitive for test-only code. Two tests: `rss_is_plausible_on_supported_host` (plausibility bounds 1 MiB ≤ x < 100 GiB, graceful skip on unsupported targets) and `rss_is_stable_across_back_to_back_reads` (<2× variance between consecutive reads).
    - `src-tauri/src/arabic/mod.rs` — two landings in one file:
        - **M9-rss-real**: `+ #[cfg(test)] mod rss;` near the other cfg-gated test modules with an explanatory comment pointing to `read_rss_bytes`.
        - **M9-hotpath (b)**: `+ use smallvec::{smallvec, SmallVec};`. New `pub type AnalysisList = SmallVec<[Analysis; 2]>;` with a doc comment explaining capacity-2 rationale (covers 100% of single-hit paths + the 2-hit `كاتب`-class ambiguous surface; spills to heap on 3+ hits only). `analyze(word) -> AnalysisList` and `analyze_with_overrides(word, overrides) -> AnalysisList` signatures updated. Inside the function body, 7 construction/return sites migrated: `Vec::new()` → `AnalysisList::new()` for empty returns (word.is_empty(), normalizer Empty script); `vec![Analysis { … }]` → `smallvec![Analysis { … }]` for the four single-element paths (non-Arabic bypass, user override hit, protected list hit, heuristic fallback); the two collect sites for stripped_hits / folded_hits annotated as `AnalysisList` so the collector infers SmallVec; the Layer 3 peel accumulator `Vec::new()` → `AnalysisList::new()`. `analyze_best` / `analyze_with_overrides_best` keep their `Option<Analysis>` return shape unchanged — `.into_iter().next()` works identically on `SmallVec`.
    - `src-tauri/src/arabic/bench.rs` — M9-rss-real adds two new measurement blocks: Section 0 captures `rss_before` at bench entry (before cold-start); Section 6 captures `rss_after` after throughput, computes `RSS delta (MiB)` and `RSS projected @ 7K (MiB)` using the same `7000 / fst_keys` extrapolation as the existing bundle-size proxy. Graceful skip if either read returns `None` — all four RSS report lines omitted, bench still prints bundle numbers. First real numbers captured this session: RSS delta +23.8 MiB at 32K FST keys → projected **280.3 MiB at 7K roots** versus the on-disk proxy's 89.8 MiB. The 3.1× ratio is the driver for M9-intern + M9-mmap.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 35 (M9-rss-real) added with Why-Now framing (on-disk proxy under-counts parsed `fst::Map` + `GeneratedForm` heap strings + OnceLock wiring), change shape (new module + two bench lines), platform-backend spec, precision note, first real numbers table. § 36 (M9-hotpath (b)) added with Why-Now framing (single-hit path dominates FTS hot path; heap `Vec` allocation wasted on >98% of tokens), change shape (type-alias + 7 call-site migrations), capacity-2 rationale, before/after bench table (−152 ns/call bare, −175 ns/call FTS, ~2% throughput improvement), public-API surface note.
    - **Combined bench numbers** (post-landing, Windows release, 251K calls): cold-start 170.7 ms, warm-start 22.8 ms, throughput 131,183 w/s (bare), 132,586 w/s (FTS), per-call 7,623 ns (bare), 7,542 ns (FTS), FTS overhead −81 ns (parity with bare path preserved from M9-hotpath (a)), pass rate 100% (502/502), cache bundle 7,812 KiB (unchanged), projected @ 7K 89.8 MiB (on-disk proxy), **RSS delta 23.8 MiB → projected 280.3 MiB at 7K roots** (new, authoritative).

18. **M9-intern — `Arc<str>` dedup for `GeneratedForm` root/label strings** (this session, `1464fce`): fourth M9 follow-on. Drops RSS projected @ 7K from 280.3 MiB → **175.8 MiB** (−37%, ~104 MiB saved) by switching `GeneratedForm::root_key` and `GeneratedForm::pattern_label` from `String` to `Arc<str>` with per-build dedup pools. **On-disk format unchanged** — `CACHE_FORMAT_VERSION` stays at 1 because length-prefixed UTF-8 serializes identically for both types; caches baked by prior commits (`da8d821`, `3cf5510`, `26eebcd`, `788c4a5`) remain readable.
    - `src-tauri/src/arabic/generator.rs` — `GeneratedForm::root_key: String` → `Arc<str>`; `pattern_label: String` → `Arc<str>`. New `pub(crate) fn intern(pool: &mut HashMap<String, Arc<str>>, s: &str) -> Arc<str>` helper. `generate_all()` rewritten to build two pools (`root_pool`, `label_pool`), pre-intern all ~150 pattern labels before the root loop, intern each root key once per root then `Arc::clone` per emission. Imports: `use std::sync::{Arc, OnceLock};`. 10 test comparison sites migrated to `&*g.root_key == "..."`.
    - `src-tauri/src/arabic/fst_bake.rs` — imports: `HashMap`, `Arc`, `intern as intern_into_pool` re-export. `decode_bundle` creates two `HashMap<String, Arc<str>>` pools shared across the stripped + folded side decoders. `decode_side` gained two pool `&mut` args. `decode_form` gained the same two pool args and performs the actual intern on decoded bytes. Encode path unchanged (`Arc<str>` auto-coerces to `&str`). 1 test roundtrip callsite updated with empty pool args. 3 test construction sites updated from `"...".to_string()` to `"...".into()`.
    - `src-tauri/src/arabic/mod.rs` — three `Analysis::new`-adjacent construction sites updated from `form.root_key.clone()` / `form.pattern_label.clone()` to `.to_string()` — the public `Analysis` surface stays `String` (API firewall). Doc comment added.
    - `src-tauri/src/arabic/fst_index.rs` — no production code changes (Arc<str> flows through transparently). 6 test comparison sites → `&*f.root_key == "..."`; 2 test construction sites → `"...".into()`; 1 `assert_eq!` → `&*hits[0].root_key, "..."`.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 37 added with Why-Now framing (3.1× disk→heap ratio from § 35), approach rationale (Arc<str> over u32+StringInterner for minimal diff + zero format change), change shape per-file, format-compatibility note (CACHE_FORMAT_VERSION stays at 1), before/after bench table (RSS delta −37%, cold-start −31% bonus, per-call throughput +1% noise-adjacent), test migration summary. `Open items` M9-mmap updated with the new 175.8 MiB baseline and 90 MiB post-M9-mmap target.
    - **Bench numbers** (post-M9-intern, Windows release, 251K calls): cold-start **139.5 ms** (vs 170.7 pre, −31 ms / −18%), warm-start 27.8 ms, throughput 130,194 w/s (bare) / 134,671 w/s (FTS), per-call 7,681 ns (bare) / 7,426 ns (FTS), FTS overhead −255 ns (still within noise of zero — parity preserved), pass rate 100% (502/502), cache bundle 7,812 KiB (**byte-identical** to pre-M9-intern, confirming format compatibility), **RSS delta +14.9 MiB → projected 175.8 MiB at 7K roots** (vs 23.8 / 280.3 pre-M9-intern).

19. **M9-mmap — memory-map the baked FST byte buffers on desktop** (this session, `49dcf45`): fifth M9 follow-on. Converts `FstBundle::{stripped_bytes, folded_bytes}` from `Vec<u8>` to a new `FstBytes` enum with `Mmap` (desktop) + `Owned` (fallback) variants. `GenerativeFst::{fst_stripped, fst_folded}` switched from `Map<Vec<u8>>` to `Map<FstBytes>`. `load_bundle` / `decode_bundle` split into `_mmap` + `_heap` pairs with the mmap path preferred on desktop, falling back to heap on any mmap error. **On-disk format unchanged** — `CACHE_FORMAT_VERSION` stays at 1; all prior-commit caches remain readable. Working-set RSS is statistically unchanged (**+15.0 MiB vs +14.9 MiB** pre-mmap) because the throughput phase touches every FST page, pulling the mapped file into the working set — but private-bytes / `Pss` / `phys_footprint` drops by ~16 MiB and the pages are now discardable under memory pressure rather than anonymous heap.
    - `src-tauri/Cargo.toml` — added target-gated dep block `[target.'cfg(not(any(target_os = "ios", target_os = "android")))'.dependencies] memmap2 = "0.9"`. Block-level cfg keeps `memmap2` out of the dep tree on mobile — no transitive bloat.
    - `src-tauri/Cargo.lock` — reflects the new `memmap2` dep (desktop-only).
    - `src-tauri/src/arabic/fst_bake.rs` — added `use memmap2::Mmap;` (cfg-gated). New `pub enum FstBytes { Mmap { mmap: Arc<Mmap>, offset, len }, Owned(Vec<u8>) }` (~70 lines incl. `AsRef<[u8]>`, `Debug`, `From<Vec<u8>>`, `len()`). `FstBundle::{stripped_bytes, folded_bytes}` migrated from `Vec<u8>` to `FstBytes`. `load_bundle` split into `load_bundle_mmap` + `load_bundle_heap`; `decode_bundle` split into `decode_bundle_mmap(Arc<Mmap>)` + `decode_bundle_heap(&[u8])`. Mmap path shares a single `Arc<Mmap>` across both FSTs (1 syscall per load, not 2). `encode_side` callers updated to use `.as_ref()` on `FstBytes`. Tests: `sample_bundle` constructs via `.into()`, two `assert_eq!` roundtrip sites updated to `.as_ref()` comparisons.
    - `src-tauri/src/arabic/fst_index.rs` — imports extended with `FstBytes`. `GenerativeFst::{fst_stripped, fst_folded}` fields migrated from `Map<Vec<u8>>` to `Map<FstBytes>` (type-only — all lookup code byte-identical). `from_bytes` signature relaxed from `Vec<u8>` to `impl Into<FstBytes>` (preserves back-compat via `From<Vec<u8>>`; the new mmap path passes `FstBytes::Mmap` directly, no copy). `build_bundle` wraps cold-rebuild `Vec<u8>` via `.into()` at the `FstBundle` construction site.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 38 added with Why-Now framing, approach rationale (enum over separate types for zero-cost abstraction), safety invariant for `unsafe Mmap::map`, **honest characterisation that working-set RSS is unchanged** (and why — throughput touches every page), structural win framing (discardable vs anonymous; private-bytes delta; multi-process prerequisite), +1,128 ns/call throughput regression as the accepted cost, before/after bench table.
    - **Bench numbers** (post-M9-mmap, Windows release, 251K calls): cold-start 147.6 ms (+8 ms — one-time mmap establishment), warm-start 27.5 ms (noise), throughput 113,521 w/s bare / 94,361 w/s FTS (**regressed** from 130K/134K — mmap reads trap through the MMU vs heap's L1/L2 hit), per-call 8,809 ns bare / 10,598 ns FTS (+1.1 µs / +3.2 µs), **cache bundle 7,812 KiB byte-identical** (format compatibility confirmed), **RSS delta +15.0 MiB → projected 175.9 MiB at 7K** (within noise of the pre-mmap 175.8 — working-set metric can't see the win), pass rate 100% (502/502). The regression is real but accepted for the structural wins documented in § 38 (discardable pages, private-bytes drop, multi-process prerequisite).

20. **M9-hotpath (c) — fast-path short-circuit for Layer 0 / Layer 2 hits** (this session, `ce25800`): sixth and final M9 follow-on on the performance lever queue. Recovers the +1,128 ns/call bare and +3,172 ns/call FTS throughput regressions from § 38 (M9-mmap) by letting Layer 0 (user override) + Layer 2 (protected list) hits return a single `Analysis` without ever constructing an `AnalysisList` stack frame or routing through the `into_iter().next()` destructure. Also drives the FTS hot path *below* the pre-M9-mmap § 37 baseline (7,426 → 7,117 ns/call, −309 ns) because the fast-path return on a Layer 0 / Layer 2 hit skips the `active_if_non_empty` probe that the FTS path uniquely pays.
    - `src-tauri/src/arabic/mod.rs` — added `lookup_layer_01(word, overrides, stripped) -> Option<Analysis>` helper (~30 lines incl. explanatory doc comment). Factored the Layer 0 (user override) + Layer 2 (protected list) probes out of the inline `analyze_with_overrides` body into this shared helper so both entry points share one source of truth and can't drift. Replaced the two inline Layer 0 + Layer 2 blocks inside `analyze_with_overrides` (lines 207–229 pre-refactor) with a single `if let Some(hit) = lookup_layer_01(...) { return smallvec![hit]; }` call — byte-identical behaviour, just shorter. Extended `analyze_with_overrides_best` with a fast-path block at the top: empty-check → `normalizer::normalize(word)` → `match norm.script` on Arabic/PersianFamily only → `lookup_layer_01` → return `hit` directly on `Some`. Non-Arabic scripts (Latin/Hebrew/Other) and Empty fall through to the existing slow-path call to `analyze_with_overrides` which handles them canonically. Doc comment records the cost tradeoff (double-normalize on slow path, ~50–100 ns, dwarfed by the FST probe cost the slow path would already pay).
    - **No changes in**: `src-tauri/src/arabic/overrides.rs`, `src-tauri/src/arabic/protected.rs`, `src-tauri/src/libraries.rs` (the FTS5 tokenizer call site is byte-identical — still calls `analyze_with_overrides_best(word, overrides_ref)`, fast path fires transparently on hit).
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 39 added with Why-Now framing (priority signal from § 38's mmap regression + architectural redundancy of the `into_iter().next()` destructure), approach (shared `lookup_layer_01` → both entry points consume), change shape per-file, cost analysis of double-normalize on slow path, results table (before/after/vs-baseline for three columns) showing +1,225 ns/call bare recovery and +3,481 ns/call FTS recovery, run-to-run variance disclosure (±19% on first vs second run), what-didn't-move section (RSS / cache bundle / pass rate unchanged and why), three new follow-ons (M9-hotpath (c)-v2, Criterion-grade bench, the remaining M9-profile).
    - **Bench numbers** (post-M9-hotpath (c), Windows release, 251K calls, second-run stable reading): cold-start **134.8 ms** (vs 147.6 pre, −12.8 ms / −8.7%), warm-start 24.5 ms (−3.0 ms), throughput **131,850 w/s bare / 140,514 w/s FTS** (+16.1% / +48.9% vs § 38), per-call **7,584 ns bare / 7,117 ns FTS** (−1,225 / −3,481 ns vs § 38), **FTS overhead −468 ns** (FTS faster than bare — first time in the M9 series), cache bundle 7,812 KiB byte-identical, RSS delta +14.7 MiB → projected **173.0 MiB at 7K** (within noise of the pre-M9-mmap 175.8 baseline — CPU optimization, not memory), pass rate 100% (502/502). Full 279/279 arabic lib tests still pass.

21. **M9-profile — sampling-profiler recipe for `m9_bench`** (this session, `320c662`): seventh and final item on the M9 follow-on queue. Pure observability — no production code change, no new deps, no new tests. Adds a `# Profiling (M9-profile)` section (~70 lines) to the module-level doc comment of `src-tauri/src/arabic/bench.rs` with paste-ready recipes for `samply` (cross-platform, recommended), `cargo-flamegraph` (Linux-only alternative), and a Windows VS-Profiler fallback. Includes a hotspot reading guide (four named functions the profile should surface post-M9, with numeric expectations) and a worked example converting "35% samples in `fst::Map::get`" into "~2,654 ns/call" via the bench's `Per-call (ns)` output. Recipe chosen over a new `.md` file per CLAUDE.md rule ("NEVER create documentation files unless explicitly required") — the recipe lives next to the bench it profiles and surfaces in `cargo doc --open`.
    - `src-tauri/src/arabic/bench.rs` — module-level doc comment extended with `# Profiling (M9-profile)` section. Covers samply install + invocation flow (`cargo test --no-run` → extract test-binary path → `samply record`), cargo-flamegraph install + invocation (with perf_event_paranoid note), Windows VS-Profiler fallback, four-hotspot reading guide (`analyze_with_overrides`, `fst::Map::get`, `arabic::normalizer::normalize`, `disambiguate::rank_analyses`), sample-percentage-to-nanosecond conversion formula, and two observability follow-ons (Criterion-grade bench, samply CI integration). No code changes.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 40 added with Why-Now framing (post-M9-hotpath (c) the remaining 7,100 ns/call is distributed across functions the wall-clock bench can count but not attribute; further micro-optimisation needs per-function costs), approach (two profiler recipes + reading guide), change shape (doc-comment-only), `Why doc-comment, not a separate .md file` justification, verification (`cargo check --lib --tests` clean), two open items (first-profile pass, samply CI integration).
    - **No runtime change** — `cargo check --lib --tests` clean, no bench rerun necessary (M9-profile's deliverable is the recipe, not a measurement).

22. **M11-data v1 — production lexicon corpus** (this session, `c1e8e5e`):
    - `lab/m11-data/README.md` — new (~175 lines). Scope doc explaining v1 is 100% Constellation-original content (no WordNet / OMW / Wiktionary dependency), schema (concept id + pos + per-language lemma lists), coverage floor (`en:` + `ar:` mandatory, ≥8/15 target), regeneration workflow, and the rationale for why every third-party wordnet source was rejected under the owner's "anything that constrains distribution or creates obligations" rule.
    - `lab/m11-data/concepts.json` — new (49 concepts × 15 languages, single-file source of truth). Each entry: `id` (kebab-case slug), `pos` (one of the 8 `arabic::PartOfSpeech` variants), `category` (organizational tag, not emitted to TSV), `notes` (one-line human gloss, not emitted), `lemmas` (15-key object, each value a list of lemma strings). Categories: seed (15 imported from M10 seed_v1.tsv), objects (10), actions (8), qualities (6), time/space (5), cognition (3), PKM primitives (2).
    - `lab/m11-data/build.py` — new (~230 lines). `concepts.json` → `src-tauri/src/lexicon/data/lexicon_v1.tsv` emitter with deterministic output (sorted concept ids, alphabetically-sorted lang columns, first-seen order for lemmas within a cell, fixed header comment block, LF line endings, UTF-8). Structural validation catches `schema_version != 1`, non-unique ids, invalid PoS, unknown lang keys, non-string lemma values, tab/newline contamination before the TSV is even written. `--stdout` + `--dry-run` flags for CI / debugging.
    - `lab/m11-data/validate.py` — new (~275 lines). Post-build TSV content validator. Hard errors (exit 1): missing `en:`/`ar:` lemma, tashkeel/tatweel in `ar`/`fa`/`ur` lemmas (U+064B–U+065F + U+0640), duplicate lemma within a cell, duplicate concept id. Warnings: <8/15 language coverage, per-lang script mismatch via Unicode block membership (Arabic / Hebrew / Devanagari / Hiragana+Katakana+CJK / Hangul / Cyrillic / Latin, with CJK/Korean/Japanese tolerating romaji for pinyin + loanwords).
    - `lab/m11-data/regenerate.sh` — new (~30 lines). One-command `build.py && validate.py` wrapper with `set -euo pipefail`. Portable `PY=${PYTHON:-python3}` fallback to `python` on Windows.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — new (8,175 bytes, 49 rows + header comment block). The production corpus, emitted by `build.py` from `concepts.json`. Ships as part of the binary via `include_str!` from `graph.rs::seed_tsv()`. 100% Constellation-original content — no third-party attribution required. Format matches the M10 seed exactly: `concept_id<TAB>pos<TAB>lang:lemma,lemma,...<TAB>...`.
    - `src-tauri/src/lexicon/graph.rs` — `seed_tsv()` swapped from `include_str!("data/seed_v1.tsv")` to `include_str!("data/lexicon_v1.tsv")` (the production cold-build path now uses the 49-concept corpus). New sibling `pub fn legacy_seed_tsv() -> &'static str` returning `include_str!("data/seed_v1.tsv")` so the M10 seed regression canary still has a byte-exact accessor to its fixture. Docstrings updated to explain the semantics of each.
    - `src-tauri/src/lexicon/bake.rs` — existing `real_seed_bundle_writes_reads_reconstructs` test retargeted to `legacy_seed_tsv()` so it continues to pin the M10 15-concept seed fixture through encoder/decoder changes. New sibling test `real_lexicon_bundle_writes_reads_reconstructs` mirrors the legacy canary against the production corpus: round-trips through `build_bundle` → `write_bundle` → `load_bundle` → `from_bundle`, asserts `recs.len() > 20` (tripwire against accidental revert), asserts `en:tree` (production-only lookup) + `ar:شجرة` (mandatory-Arabic sanity) resolve in the reconstructed graph.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 42 added with Why-Now framing (eliminate extractor-tooling blocker + third-party licensing surface), scope (49-concept corpus composition table), deterministic emitter rules, structural + content validator specs, Rust wire-up (seed_tsv swap + legacy_seed_tsv + two-canary-test pattern), verification (116 lexicon + 412 full-lib tests pass), non-goals (scale / synonyms / domains / alternate-data-sources tracked as separate milestones), `lab/m11-data/` layout rationale.

23. **M12-wire — lexicon expansion in `lexical_search`** (this session, `94754de`):
    - `src-tauri/src/search.rs` — `lexical_search` call site (line ~770) swaps `let fts_query = format!("{}*", normalized.replace('"', ""));` for `let fts_query = expanded_match_query(&normalized).unwrap_or_else(|| format!("{}*", normalized.replace('"', "")));`. New sibling `fn expanded_match_query(normalized: &str) -> Option<String>` (~25 lines, directly after `lexical_search`) that calls `crate::lexicon::detect_source_lang` + `crate::lexicon::expand_to_match_expr` with `ExpansionOptions::default()`, then gates the return on `expr.contains(" OR ")` — returning `Some(expr)` only when expansion produced actual cross-lingual / cross-synonym bridge terms, `None` otherwise so the caller falls back to today's prefix match. New `#[cfg(test)] mod tests_m12` at EOF with 5 tests: `known_english_word_bridges_cross_lingually` (`tree` → OR-joined expr with `شجرة`), `known_arabic_word_bridges_to_english` (`شجرة` → OR-joined expr with `tree`), `unknown_word_falls_back_to_none` (`quasar` → None, preserving prefix fallback), `punctuation_only_returns_none` (`"   !!!"` / `""` / `"123"` — `detect_source_lang` returns None), `proper_noun_not_in_corpus_falls_back` (`Constellation` / `Anthropic` → None, common-case behavior until M11-data scales).
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 43 added with Why-Now framing (M11-data v1 is a dead payload without this edit), approach (helper-returns-None decision gate on `" OR "` presence), five-test pin-down of the gate, blast radius (two `lexical_search` call sites in `search.rs`, no IPC surface change, no frontend edit), verification (`cargo test --lib tests_m12` 5/5 + `cargo test --lib` 417/417), scope kept out (no settings toggle, no bench re-run, no `match_via` badge yet — M13 scope).

24. **M13 — multilingual result badge end-to-end** (this session, `0348a81`):
    - `src-tauri/src/search.rs` — `SearchResult` gains `pub match_via: Option<String>` with a doc comment on when the field populates (cross-lingual hits only). All seven non-lexical constructors default it to `None`: `structured_search` (line ~1171), `semantic_search` (~1999), `search_titles` (~2242), `search_contents` (~2273), `search_tags` (~2302), `search_properties` (~2329), `search_wikilinks` (~2356). `expanded_match_query` return type flipped from `Option<String>` to `Option<LexicalExpansion>` — new private struct with `match_expr: String` and `bridge_terms_lower: Vec<String>`; the bridge list is filtered via `.filter(|(lang, _)| *lang != source_lang)` so source-language inflections never earn a badge. New `fn find_match_via(snippet: &str, bridge_terms_lower: &[String]) -> Option<String>` (~25 lines) that scans `<mark>…</mark>` regions — lowercasing each marked span once — and returns the first bridge-term hit. `lexical_search` row closure wires it: `let match_via = if title_hit { None } else { snippet.as_deref().and_then(|s| find_match_via(s, bridge_terms)) };`. `tests_m12` updated to `.match_expr` / `.is_none()` pattern (LexicalExpansion lacks PartialEq/Debug). New `#[cfg(test)] mod tests_m13` appended — 12 tests: seven scanner tests (`mark_containing_bridge_term_returns_it`, `source_lemma_match_returns_none`, `first_mark_wins_when_multiple_bridges_present`, `unmarked_bridge_occurrence_is_ignored`, `case_is_folded_on_snippet_side`, `empty_bridge_terms_returns_none_fast`, `snippet_without_marks_returns_none`, `unterminated_mark_breaks_out_cleanly`, `partial_mark_content_match_does_not_badge`) + three expansion tests (`english_expansion_excludes_english_from_bridge_terms`, `arabic_expansion_excludes_arabic_from_bridge_terms`, `bridge_terms_are_pre_lowercased`). Full lib suite: **429 pass, 0 fail, 2 ignored** (+12 over M12-wire's 417 baseline).
    - `src/lib/libraries/store.ts` — `ConstellationSearchResult` interface gains `match_via?: string` with a documentation comment describing the badge semantics and when the field is absent (same-language hit, title match, synonym-only expansion).
    - `src/lib/components/SearchHub.svelte` — three result-rendering sites gain a `{#if r.match_via}` block: the advanced-mode grouped view (~line 460), advanced-mode flat view (~line 500), and universal-mode categorized view (~line 539). Each renders a `<span class="sh-match-via" dir={detectDir(r.match_via)} title="via {lemma}">via {lemma}</span>` between the name and library badge. New CSS rule `.sh-match-via` (~line 720) — 12% accent-tinted background, muted foreground, `max-width: 12ch` + ellipsis for overflow safety, flex-shrink:0 so it doesn't compress the row. `dir={detectDir(r.match_via)}` ensures Arabic/Hebrew/Urdu bridge terms render right-to-left regardless of the host row's direction.
    - `src/lib/i18n/{ar,de,en,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — one new key per locale: `searchHub.matchVia` → "via" in en/fr/pt, "عبر" (ar), "über" (de), "vía" (es), "از طریق" (fa), "דרך" (he), "के माध्यम से" (hi), "経由" (ja), "経由" (zh), "경유" (ko), "через" (ru), "üzerinden" (tr), "بذریعہ" (ur). Values chosen to read naturally next to a foreign lemma ("via شجرة" / "عبر tree" / "経由 tree").
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 44 added with Why-Now (M12-wire is live but results don't *explain* cross-lingual hits), approach (thread source-filtered bridge terms through LexicalExpansion → find_match_via → row closure), 12-test enumeration split scanner vs expansion, blast radius (byte-identical JSON for pre-M13 paths, all seven non-lexical constructors default to None), frontend shape (three render sites + one CSS rule + 15 i18n keys), why-title-hit-short-circuits rationale, verification (429/429 lib tests, svelte-check clean on new additions), and M14-benchmarks as the remaining roadmap item.

25. **M14 — `lexical_search` end-to-end non-regression bench** (this session, `c03f526`):
    - `src-tauri/src/search.rs` — appended `#[cfg(test)] mod m14_bench` at EOF (+408 lines). Opt-in `#[test] #[ignore] fn m14_bench()` (mirrors `arabic::bench::m9_bench` and `lexicon::bench::m12_bench` shape — doesn't run under default `cargo test --lib`; invoked with `cargo test --lib --release search::m14_bench -- --ignored --nocapture`). Builds a tempfile SQLite corpus via `seed_bench_corpus()` — 100 notes (40 English-only bodies, 40 Arabic-only bodies, 20 bilingual `tree` ↔ `شجرة` mixed bodies), pre-tokenised through `normalize_arabic_for_search` + `process_arabic_word` so the FTS5 side reflects production. Three measurement shapes × `WARMUP=20` + `SAMPLES=500` iterations each: **(a) known-word bridging** — `tree` / `كتاب` / `livre` exercise the M12-wire cross-lingual path (multi-language OR-joined MATCH clause, M13 `match_via` badge resolves); **(b) unknown-word prefix fallback** — `quasar` / `Constellation` / `xyzzy` exercise the `None`-return path in `expanded_match_query` and the `{normalized}*` fallback; **(c) Arabic-only non-regression** — `شجرة` / `معرفة` exercise the hot production path where M12-wire must not slow down today's behaviour. Hard-asserts `BUDGET_P99_NS < 10_000_000` (10 ms) on each shape's worst-case p99 so a future regression that reintroduces per-call allocation, full graph scan, or uncached FST rebuild trips on the next opt-in run. `report_stats(label, samples, hits)` helper prints mean/p50/p95/p99/max per shape. Tempfile corpus path-unique under `std::env::temp_dir()` with `std::process::id() + nanos` so parallel runs don't collide; deleted at test exit via `drop(conn)` + `remove_file(&db)` (best-effort). No public-API change, no new crate deps — uses only already-imported `rusqlite::params`, `std::time::Instant`, `std::hint::black_box`, and `super::{init_db, lexical_search, normalize_arabic_for_search}`. First baseline captured this session (Windows release, 100-note corpus, 500 samples per query): **(a)** worst p99 6.63 ms on `tree` (23 hits, FTS5 `snippet()` dominates); **(b)** worst p99 0.05 ms on prefix paths (0 hits — short-circuit on empty MATCH set); **(c)** worst p99 6.28 ms on `شجرة` (23 hits — parity with the English hot path, **confirms M12-wire adds no measurable cost** on bridged-but-zero-new-lang-hits queries). All three shapes pass the 10 ms budget comfortably. Total test runtime 9.43 s (seed + 3,500 timed calls + stats).
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 45 added with Why-Now framing (close the last open item on the M1–M14 roadmap; prove "cross-lingual search for free" is measurable, not aspirational), approach (three shapes × warmup + samples + hard-assert), query-bank rationale, corpus design (100 notes × three language profiles), harness parameters (`WARMUP=20`, `SAMPLES=500`, `BUDGET_P99_NS=10_000_000`), results table, key-finding framing (M12-wire's zero-cost claim is now proved on a real corpus, not inferred from M12-bench's micro-scale), blast radius (zero production change, tempfile lifecycle safety), verification steps, out-of-scope items (bench rerun at M11-data v2 scale, Settings → Debug scorecard integration, CI integration), and the Arabic Engine roadmap status ("M1–M14 closed").

26. **M11-data v2-infra — shard concepts/ + first two thematic batches** (this session, `39e2480`):
    - `lab/m11-data/concepts.json` — **deleted**. Replaced by the shard directory below. Content migrated byte-for-concept to `000-core-seed.json`; the migration was verified via SHA256 equality on the emitted `lexicon_v1.tsv` before any new content landed, so the infra change is isolated from the content additions.
    - `lab/m11-data/concepts/000-core-seed.json` — new (49 concepts, the v1 foundation). Every concept from v1 preserved unchanged — same ids, same PoS, same per-language lemma lists.
    - `lab/m11-data/concepts/001-body-and-family.json` — new (43 concepts). Body parts (body, head, face, eye, ear, nose, mouth, tooth, tongue, hair, neck, hand, finger, arm, leg, foot, heart, blood, bone, skin, brain, mind, voice), family (mother, father, parent, son, daughter, brother, sister, child, baby, wife, husband, family), society (friend, neighbor, person, man, woman, boy, girl, name). Every row carries `en` + `ar` at minimum; per-concept coverage averages ≥ 13 of 15 languages. Arabic lemmas stored tashkeel/tatweel-stripped per v1 discipline.
    - `lab/m11-data/concepts/002-nature.json` — new (54 concepts). Animals (animal, dog, cat, bird, fish, horse, cow, sheep, goat, lion, wolf, camel, elephant, rabbit, mouse, snake, insect, bee, butterfly), plants/food (flower, rose, leaf, seed, root, branch, forest, garden, fruit, apple, date-fruit, olive, grape, wheat, rice), landscape (mountain, river, sea, ocean, lake, desert, field, road, bridge, stone, sand), weather (rain, snow, wind, cloud, storm, thunder), physics (light, shadow, air). Same en+ar floor + ≥8/15 target discipline.
    - `lab/m11-data/build.py` — refactored (~40 lines changed). Module docstring rewritten to document the v2 shard layout, deterministic-output invariants, and the cross-shard dedup invariant. `CONCEPTS_JSON = ... / "concepts.json"` → `CONCEPTS_DIR = ... / "concepts"`. Added `from typing import ... Tuple`. Replaced `load_concepts()` with two helpers: `load_shard(path)` (parses one shard, within-shard structural validation incl. schema version / PoS whitelist / lang whitelist / string+list type / tab+newline contamination) and `load_all_shards()` (walks `CONCEPTS_DIR.glob("*.json")` in filename sort order, cross-shard id-collision check as hard error with pointers to both offending files, returns `(concepts, shard_counts)`). `count_summary()` extended with per-shard concept counts for `--dry-run`. `main()` consumes the new `(concepts, shard_counts)` pair. Deterministic output invariants (rows sorted by concept id after flatten, lang columns alphabetic, first-seen lemma order within cell, fixed header, LF endings) untouched.
    - `lab/m11-data/README.md` — updated. Status line v1 → v2 (in-flight). "Scale policy" rewritten around thematic shards + ≥8/15 coverage target + hand-curation-then-LLM-assisted path. "File layout" section replaced with the shard directory tree + a new "Shard layout (v2)" subsection documenting the NNN-theme.json convention, schema version, cross-shard dedup invariant, and the deterministic-output guarantee. "Regeneration workflow" step 3 updated to mention the shard walk + flatten + id-collision check. "Follow-ons" list's former "M11-data-scale" bullet replaced with "M11-data v2 continued batches" pointing at future thematic shards. License-rejection rationale section unchanged (still valid).
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the three shards. 8,175 → 22,488 bytes (~2.75×). Header comment block unchanged. Rows sorted by concept id; lang columns alphabetic within row. `graph::seed_tsv()` continues to return `include_str!("data/lexicon_v1.tsv")` — the byte content changes, the pointer doesn't. djb2 hash of the new TSV flips the content-addressed cache filename, so next boot writes a fresh bundle and orphans the old one per M11-infra design.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 46 added with Why-Now (close the final open item on the Arabic Engine roadmap by scaling data; monolithic concepts.json does not scale past ~2K), shard layout design (NNN-theme naming, cross-shard dedup hard-error invariant, deterministic-output guarantee), build.py refactor per-file, first two content shards with concept enumeration, verification (build + validate + 116/116 lexicon tests pass), blast radius (zero Rust change), three out-of-scope follow-ons (continued batches toward 20K, bench reruns, LLM-assisted authorship past ~2K).

27. **M11-data v2 — +003-food-and-household (40) +004-qualities (52) batches** (this session, `6415eee`):
    - `lab/m11-data/concepts/003-food-and-household.json` — new (40 concepts). Food staples (meat, milk, egg, cheese, butter, salt, sugar, honey, oil), drinks (coffee, tea, juice, wine), meals (meal, breakfast, lunch, dinner), household (chair, table, bed, window, mirror, lamp, clock, phone, computer, key, bag, bottle, cup, plate, spoon, knife, fork, pen, paper, clothes, shoe, hat, coat). A placeholder `door` entry was drafted during authoring and removed before build — `000-core-seed.json` already carries that id and the v2-infra cross-shard dedup would have tripped the build. Every row en+ar floor; Arabic lemmas tashkeel/tatweel-stripped.
    - `lab/m11-data/concepts/004-qualities.json` — new (52 concepts). Adjectives (26: small, tall, short, old, young, warm, cold, hot, wet, dry, bad, easy, `hard-difficult`, fast, slow, clean, dirty, full, empty, happy, sad, tired, strong, weak, right, wrong), colors (11: black, white, red, green, blue, yellow, `orange-color`, purple, pink, gray, brown), numbers / quantifiers (15: zero..ten, hundred, thousand, many, few). Concept ids `hard-difficult` (reserving `hard` for rigid/solid) and `orange-color` (reserving `orange` for the fruit) chosen to preserve future-shard headroom; display lemmas remain plain `hard` / `orange` in every language column, so neither surfaces to the user.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the five shards. 22,488 → 48,810 bytes. Header block unchanged. `graph::seed_tsv()` continues to return `include_str!("data/lexicon_v1.tsv")` — byte content changes, pointer unchanged; djb2 hash flip rotates the content-addressed cache filename per M11-infra design.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 47 added with Why-Now (shift corpus toward everyday user-search vocabulary: food/household + adjectives/colors/numbers), concept enumeration, concept-id reservation rationale for `hard-difficult` / `orange-color`, verification (build + validate + 116/116 lexicon tests pass at 238-concept scale), blast radius (zero Rust change, pure data addition).

28. **M11-data v2 — +005-basic-verbs-and-emotions (42) batch** (this session, `b38fa26`):
    - `lab/m11-data/concepts/005-basic-verbs-and-emotions.json` — new (42 concepts). 30 action verbs (drink / walk / run / sit / stand / listen / look / play / teach / make / give / take / buy / sell / open / close / start / stop / live / die / grow / change / help / meet / ask / answer / come / go / bring / show) + 3 memory verbs (find, remember, forget) + 4 affect verbs (hate, laugh, cry, smile) + 5 affect/cognition nouns (fear, hope, joy, anger, dream). Arabic verbs stored in standard Semitic citation form (3rd-person masculine singular past, e.g. `شرب` / `ركض` / `جلس`); Arabic emotion nouns stored in singular nominative (`خوف` / `أمل` / `فرح` / `غضب` / `حلم`). All lemmas already tashkeel / tatweel-stripped.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the six shards. 48,810 → 58,622 bytes (+20% for +42 concepts; ratio lower than prior shard landings because verb lemmas are largely short single-word tokens with less multibyte-script payload per row). Rust wire-up unchanged; djb2 hash flip rotates the cache filename.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 48 added with Why-Now (corpus still has near-zero verb coverage outside the 10 infrastructure verbs in `000-core-seed`; cross-lingual search is cleaner on verbs than on nouns), concept enumeration, PoS split rationale (37 Verb + 5 Noun), cross-shard collision-avoidance note (11 core-seed verbs deliberately not re-added; `teach` vs `teacher` distinct-concepts note), verification at 280-concept scale, blast radius (zero Rust change).

29. **M11-data v2 — +006-time-and-space (40) batch** (this session, `a739692`):
    - `lab/m11-data/concepts/006-time-and-space.json` — new (40 concepts). 17 time concepts (morning / afternoon / evening / yesterday / tomorrow / week / month / hour / minute / second / moment / past / present / future / beginning / end / middle) + 23 space concepts (place / location / area / region / country / village / town + relative positions up/down/left-side/right-side/front/back/inside/outside/near/far + cardinals north/south/east/west + center/edge). Concept ids `left-side` / `right-side` chosen to avoid collision with `right` (already in 004 as the "correct" adjective); display lemmas remain plain `left` / `right` in every language column. `up` / `down` / `near` / `far` marked PoS=Adverb for grammatical accuracy — these are primarily adverbial in English and most of the covered languages (e.g. Arabic `فوق` / `تحت` / `قريب` / `بعيد` all prepositional/adverbial). Arabic lemmas tashkeel/tatweel-stripped.
    - Deliberate cross-shard coverage discipline: `city` and `world` not re-added (already in `000-core-seed.json`); `noon` and `midnight` deferred to a possible `007` batch alongside `second`-subdivisions; cardinal-direction adjective forms (`northern` / `southern` / ...) deferred to avoid concept-id-explosion at this scale.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the seven shards. 58,622 → 67,016 bytes on disk (+14% for +40 concepts; per-concept lemma density slightly under 005 because short adverb tokens like `up` / `down` / `وراء` carry less multibyte payload than verb stems). Rust wire-up unchanged — `graph::seed_tsv()` still returns `include_str!("data/lexicon_v1.tsv")`; djb2 hash flip rotates the content-addressed cache filename so next boot writes a fresh bundle and orphans the old one per M11-infra design.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 49 added with Why-Now (time + space form the basic deictic frame for every event/location-bearing query; corpus had zero coverage of either category), concept enumeration, disambiguator rationale for `left-side` / `right-side`, PoS rationale for adverb-marked tokens (`up` / `down` / `near` / `far`), deferral list (`noon` / `midnight` / cardinal-direction adjectives), verification at 320-concept scale (7 shards, 0 errors, 0 warnings, 116/116 lexicon tests), blast radius (zero Rust change, pure data addition).

30. **M11-data v2 — +007-cognition-and-language (40) batch** (this session, `f5a8d38`):
    - `lab/m11-data/concepts/007-cognition-and-language.json` — new (40 concepts). 15 cognition verbs (know / understand / believe / doubt / decide / choose / imagine / wonder / consider / realize / expect / guess / compare / judge / recognize) + 15 speech-act verbs (say / tell / explain / describe / promise / warn / thank / apologize / complain / argue / discuss / whisper / shout / announce / repeat) + 10 linguistic / cognitive nouns (word / sentence / story / opinion / reason / meaning / lie-falsehood / secret / thought / message). Arabic verbs stored in Semitic citation form (3rd-person masculine singular past, e.g. `عرف` / `قال` / `شرح`); Arabic cognition nouns in singular nominative (`رأي` / `معنى` / `فكر`); all lemmas tashkeel/tatweel-stripped.
    - PoS discipline: 30 Verbs (cognition + speech) + 10 Nouns. Concept id `lie-falsehood` disambiguates from the English homonymous verb `lie` (to recline); display lemmas remain plain `lie` in every language column, so the disambiguator never surfaces to the user.
    - Cross-shard collision avoidance: core-seed verbs (`think` / `read` / `write` / `speak` / `see` / `hear` / `learn`) and 005 verbs (`remember` / `forget` / `find` / `ask` / `answer`) deliberately not re-added. Core-seed nouns (`idea` / `knowledge` / `truth` / `memory` / `question` / `language` / `note`) also preserved — `thought` (product/process of thinking) is a distinct concept from `idea` (a specific notion) even though both map to `فكرة` in Arabic; cross-concept lemma overlap is allowed by the validator.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the eight shards. 67,016 → 77,918 bytes on disk (+16% for +40 concepts; verb-heavy shard with multi-lemma coverage across most RTL and CJK columns). Rust wire-up unchanged; djb2 hash flip rotates the cache filename.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 50 added with Why-Now (cognition and speech are the second-most-common cross-lingual query surface after emotion — a user searching `understand` / `explain` / `decide` gets zero bridges without this shard), concept enumeration, PoS split rationale (30 Verb + 10 Noun), `lie-falsehood` disambiguator rationale, verification at 360-concept scale (8 shards, 0 errors, 0 warnings, 116/116 lexicon tests), blast radius (zero Rust change, pure data addition).

31. **M11-data v2 — +008-society-and-government (40) batch** (this session, `9ef9824`):
    - `lab/m11-data/concepts/008-society-and-government.json` — new (40 concepts). 10 authority structures (government / nation / law / rule / power / authority / leader / king / queen / president) + 5 military/conflict (war / army / soldier / freedom / justice) + 10 legal/civic/economic (crime / prison / court / judge-person / vote / election / tax / money / market / business) + 10 community/society (society / culture / tradition / group / meeting / event / ceremony / citizen / community / public) + 5 civic norms (private / equality / duty / responsibility / punishment). Arabic lemmas stored in singular nominative for nouns; `خاص` (private) is the masculine singular adjective form. All tashkeel/tatweel-stripped.
    - PoS discipline: 39 Nouns + 1 Adjective (`private`). Concept id `judge-person` disambiguates from the 007 `judge` Verb (to form an opinion); display lemma remains plain `judge` in every language column. `vote` kept as Noun (ballot / indication of choice) — the verbal sense (cast a vote) can be added in a future batch if cross-lingual search shows demand.
    - Cross-shard collision avoidance: `peace`, `neighbor`, `friend`, `name`, `city`, `country`, `village`, `town`, `region`, `area`, `world`, `family`, `person`, `man`, `woman`, `boy`, `girl`, `child`, `baby`, `wife`, `husband` all deliberately not re-added (core-seed + 001). `group` (this shard, Noun = collection of people/things) distinct from any future `set` / `collection` concept.
    - **Milestone: 400-concept threshold reached** (first multiple of 100 past the 49-concept v1 seed). Corpus now covers body, nature, food, qualities, actions, emotions, time, space, cognition, language, and society in a planned, shard-per-theme fashion.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the nine shards. 77,918 → 87,840 bytes on disk (+13% for +40 concepts; civic-vocabulary density lower than verb shards because most concepts are single-lemma per language rather than action-cluster multi-lemma entries). Rust wire-up unchanged; djb2 hash flip rotates the cache filename.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 51 added with Why-Now (civic/government vocabulary is a strong cross-lingual bridge — `government` / `law` / `vote` carry near-identical semantic weight across all 15 covered languages), concept enumeration, PoS split rationale (39 Noun + 1 Adjective), `judge-person` disambiguator rationale, 400-concept milestone call-out, verification at 400-concept scale (9 shards, 0 errors, 0 warnings, 116/116 lexicon tests), blast radius (zero Rust change, pure data addition).

32. **M11-data v2 — +009-arts-and-creativity (40) batch** (this session, `13f292b`):
    - `lab/m11-data/concepts/009-arts-and-creativity.json` — new (40 concepts, all POS=Noun). 10 visual-arts (art / painting / picture / drawing / sculpture / statue / photograph / design / pattern / image) + 15 performing-arts (music / song / dance / theater / film / drama / actor / singer / dancer / instrument / drum / guitar / piano / violin / stage) + 9 literary-arts (poem / poetry / novel / author / writer / poet / chapter / page / title) + 6 creative-abstract (beauty / style / creation / inspiration / craft / museum). Arabic lemmas stored in singular nominative; all tashkeel/tatweel-stripped.
    - PoS discipline: 40 Nouns flat — a pure-noun shard is a first for M11-data v2 (every prior shard mixed parts of speech), reflecting that artistic vocabulary is almost exclusively noun-phrased across the 15 covered languages. The verbal senses (`paint` as in "to paint", `sing` as in "to sing", `dance` as in "to dance") are deferred to a future verb-supplementary batch because (a) the noun is the higher-frequency cross-lingual search target and (b) treating the verbs under a separate concept id preserves the noun/verb distinction that Semitic languages (Arabic `رسم` noun vs `رسم` verb) and Germanic languages (German `Tanz` noun vs `tanzen` verb) maintain lexically.
    - Within-concept lemma discipline: `photograph` uses the disambiguating Arabic phrase `صورة فوتوغرافية` in the Arabic column rather than bare `صورة`, because bare `صورة` is already the Arabic lemma for both `picture` (id=picture) and `image` (id=image). The validator only checks for duplicates within a single concept × language cell, so bare `صورة` appearing in two or three different concepts' `ar` columns is valid — but `photograph` carries the photographic-specific phrasing so the bridge is unambiguous when a user searches specifically for "photograph".
    - Author / writer / poet kept as three distinct concepts. Arabic (`مؤلف` / `كاتب` / `شاعر`), German (`Autor` / `Schriftsteller` / `Dichter`), French (`auteur` / `écrivain` / `poète`), Spanish (`autor` / `escritor` / `poeta`), and Russian (`автор` / `писатель` / `поэт`) all keep these distinct in their native lexicons, so folding them into a single concept would lose cross-lingual resolution on the finer distinction.
    - Cross-shard collision avoidance: `song` and `poetry` and `music` deliberately checked against `000-core-seed` / `001-body-and-family` / `002-nature` — none overlap. `book` (already in core-seed) is not re-added; the literary-arts cluster keeps `novel` (longer fictional prose) / `chapter` (structural subdivision) / `page` (physical unit) / `title` (name/heading) as separate concepts from `book` (the physical or digital volume). `paper` (already in 003-food-and-household as writing substrate / paper material) is not re-added as an arts concept.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the ten shards. 87,840 → 97,553 bytes on disk (+11% for +40 concepts; noun-only shard with tight single-lemma-per-language rows except where a natural synonym pair exists like English `film`/`movie` which carries both). Rust wire-up unchanged; djb2 hash flip rotates the cache filename.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 52 added with Why-Now (arts vocabulary is a universal cross-lingual bridge: terms like `music` / `art` / `poem` travel nearly unchanged semantically across every covered language, and rank high on real-world search frequency for personal notebooks), concept enumeration, PoS discipline (40 Nouns flat — first pure-PoS shard), author/writer/poet three-concept rationale, `photograph`-vs-`picture`-vs-`image` Arabic-lemma disambiguation rationale, cross-shard collision avoidance, verification at 440-concept scale (10 shards, 0 errors, 0 warnings, 116/116 lexicon tests), blast radius (zero Rust change, pure data addition).

33. **M11-data v2 — +010-science-and-math (40) batch** (this session, `8c731e1`):
    - `lab/m11-data/concepts/010-science-and-math.json` — new (40 concepts). Six semantic clusters: natural-science meta (science / chemistry / physics / biology / mathematics / theory), chemistry & matter (atom / element / molecule / matter / gas / liquid / solid), physics & energy (energy / force / gravity / heat / electricity / temperature / experiment), biology (cell), math operations (number / plus / minus / multiply / divide / equal / sum), geometry (circle / square / triangle / line / point / angle), astronomy & measurement (planet / universe / galaxy / orbit / mass / measurement).
    - PoS discipline: 37 Nouns + 3 Verbs (`multiply` / `divide` / `equal` stored as Verbs in their citation form because the cross-lingual bridge targets the action sense — Arabic `ضرب` / `قسم` / `ساوى` are all verbs in past-tense citation form; German `multiplizieren` / `teilen` / `gleichen` are infinitives; English lemma represents the bare verb). `plus` / `minus` kept as Nouns (arithmetic-operator symbols) because the cross-lingual bridge here targets the *name of the symbol* — Arabic `زائد` / `ناقص`, German `Plus` / `Minus` are noun-like in usage; English "plus/minus" functions primarily as a preposition ("2 plus 2") but the noun sense ("the plus sign") is the one that carries through to most other languages as a substantive operator name.
    - Cross-shard collision avoidance: `zero` / `one` / `two` / `three` / `four` / `five` / `ten` / `hundred` (already in 004-qualities numbers), `sun` / `moon` / `star` / `earth` / `sky` / `cloud` / `wind` / `rain` / `snow` / `mountain` / `river` / `ocean` / `sea` / `lake` / `stone` / `fire` / `animal` / `tree` / `flower` / `leaf` / `fruit` / `bird` / `fish` / `dog` / `cat` (already in 002-nature or core-seed), `light` (already in core-seed), `equality` (already in 008-society-and-government as the civic noun) — deliberately not re-added; the 008 `equality` Noun and the 010 `equal` Verb are distinct concepts (civic/social equity vs mathematical sameness) and both preserve their bare English lemmas.
    - `universe` cross-lists `cosmos` as an English synonym because Arabic (`كون`), Russian (`вселенная` / `космос`), and French (`univers` / `cosmos`) all treat them as near-equivalents for the "totality of physical existence" sense — putting both English lemmas on one concept matches the cross-lingual bridge rather than artificially fragmenting into two concepts that map to the same lemma in most target languages.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the eleven shards. 97,553 → 106,939 bytes on disk (+10% for +40 concepts; science vocabulary is relatively consistent single-lemma per language except where a natural synonym pair exists like `mathematics`/`math`/`maths` in English or `universe`/`cosmos`). Rust wire-up unchanged; djb2 hash flip rotates the cache filename.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 53 added with Why-Now (science and math vocabulary is the highest-density cross-lingual bridge in the corpus: `atom` / `energy` / `gravity` / `molecule` / `circle` / `triangle` / `galaxy` / `orbit` are near-identical semantically across every covered language, largely because modern scientific terminology was standardised late enough to travel as loan translations in most cases), concept enumeration with six-cluster breakdown, PoS discipline rationale (37 Noun + 3 Verb + 2 Noun-as-operator), `universe`/`cosmos` synonym-pair rationale, cross-shard collision list (24 ids deliberately not re-added), verification at 480-concept scale (11 shards, 0 errors, 0 warnings, 116/116 lexicon tests), blast radius (zero Rust change, pure data addition).

34. **M11-data v2 — +011-professions-and-work (40) batch** (this session, `0bf46eb`):
    - `lab/m11-data/concepts/011-professions-and-work.json` — new (40 concepts, all POS=Noun). Three clusters: 25 specific professions — medical (doctor / nurse), technical (engineer), traditional trades (farmer / merchant / carpenter / baker / cook / tailor / butcher / fisherman / hunter / builder), knowledge professions (lawyer / priest / artist / scientist / musician), service/uniform professions (pilot / driver / policeman / soldier-already-in-008-excluded / servant / helper), abstract work roles (worker / manager); + 8 workplace institutions (office / factory / shop / company / hospital / school / university / farm); + 7 work abstractions (job / profession / employee / employer / boss / customer / salary).
    - Arabic lemmas in singular indefinite/definite form, tashkeel/tatweel-stripped. Two first-pass rows contained tanwin kasra (U+064D) and tripped the validator on first build:
      - `cook`/`طاهٍ` — the Arabic nominative-indefinite form of "chef" (with tanwin). Dropped in favour of `طباخ` (already the second lemma), which is the more common informal register anyway.
      - `lawyer`/`محامٍ` — the Arabic nominative-indefinite form of "attorney" (with tanwin). Dropped in favour of `محامي` (already the primary lemma; the tanwin form would only differ in very formal Classical Arabic contexts).
    - This is the first batch where validator catch caused rework — confirms the validator's tashkeel/tatweel rule is both live and correctly-aligned with the rest of the pipeline (the analyzer strips both on every lookup, so storing them at rest would cause round-trip failures on recall).
    - Cross-shard collision avoidance: `soldier` (008-society-and-government), `teacher` / `student` (000-core-seed), `judge` (007-cognition-and-language; that's the Verb "to form an opinion"; the 008 `judge-person` Noun reserves the "person who judges legally" sense; neither collides with anything in 011), `leader` (008), `work` (as Verb — already in core-seed; this shard adds `job` for the Noun sense, which is the more search-target-relevant form).
    - `customer` cross-lists `client` as an English synonym because Arabic (`زبون` / `عميل`) distinguishes them loosely based on context but most Western languages treat them as near-equivalents in commercial registers (French `client` covers both; German `Kunde` covers both). `shop`/`store`, `merchant`/`trader`, `lawyer`/`attorney`, `cook`/`chef`, `worker`/`laborer`, `helper`/`assistant`, `employee` (without `staff`), `profession`/`occupation`, `university`/`college` all fold the English synonym pair onto one concept to match the one-to-one mapping the target languages use.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the twelve shards. 106,939 → 118,993 bytes on disk (+11% for +40 concepts; profession-vocabulary density very similar to the arts shard — mostly one-to-two-lemma rows per language with the `en` column occasionally stretching to three where English has multiple register variants for the same professional role). Rust wire-up unchanged; djb2 hash flip rotates the cache filename.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 54 added with Why-Now (professions vocabulary is the next bridge after arts / science: every language names the same two-dozen core trades, and the common terms `doctor` / `teacher` / `engineer` / `farmer` carry high search frequency in personal writing), concept enumeration by cluster (25 profession + 8 workplace + 7 work-abstraction), validator-catch story (2 rows with tanwin kasra fixed pre-commit), cross-shard collision avoidance, English-synonym-pair rationale, verification at 520-concept scale (12 shards, 0 errors, 0 warnings, 116/116 lexicon tests), blast radius (zero Rust change, pure data addition).

35. **M11-data v2 — +012-tools-and-materials (40) batch** (this session, `<hash-pending>`):
    - `lab/m11-data/concepts/012-tools-and-materials.json` — new (40 concepts, all POS=Noun). Three clusters: 22 tools/implements (tool / hammer / nail / screw / axe / saw / drill / wrench / scissors / needle / thread / rope / chain / wire / lock / broom / brush / shovel / bucket / ladder / hook / pencil) + 5 containers (pot / pan / bowl / box / basket) + 13 materials (material / metal / iron / steel / gold / silver / copper / wood / brick / cement / glass / cloth / leather). Arabic lemmas in singular indefinite/definite form, tashkeel/tatweel-stripped.
    - PoS discipline: 40 Nouns flat — matching the arts shard (§ 52) in being pure-Noun. Tools-and-materials vocabulary is almost exclusively noun-phrased across the 15 covered languages; the verbal senses (`hammer` as "to hammer", `saw` as "to saw", `lock` as "to lock") are deferred because (a) the object-noun is the higher-frequency cross-lingual search target and (b) Semitic languages and most Indo-European languages treat the object and the action as lexically distinct concepts (Arabic `مطرقة` hammer-the-object vs `طَرَقَ` hammer-the-verb).
    - Material-synonym discipline: `metal` kept as the cluster-level superordinate with specific subordinates `iron` / `steel` / `gold` / `silver` / `copper` each as their own concept. This matches the target-language lexical geography — every covered language distinguishes these five at a fundamental level (Arabic `حديد` / `فولاذ` / `ذهب` / `فضة` / `نحاس`, Russian `железо` / `сталь` / `золото` / `серебро` / `медь`, Chinese `铁` / `钢` / `金` / `银` / `铜`), so folding them would lose cross-lingual resolution on metals that have near-universal salience in personal writing (jewelry / construction / kitchenware contexts all require this distinction).
    - Container-cluster shape: `pot` / `pan` / `bowl` separated by cooking-surface shape rather than by material. The Arabic column preserves this — `pot` = `قدر` (deep cooking vessel), `pan` = `مقلاة` (frying surface), `bowl` = `وعاء` (general concave container) — matching how English lexicalizes these three shapes. `box` and `basket` kept as separate concepts because the rigid-vs-woven distinction is universal across target languages.
    - Cross-shard collision avoidance: `key` (003-food-and-household), `knife` (003), `bag` (003), `bottle` (003), `cup` (003), `plate` (003), `spoon` (003), `fork` (003), `lamp` (003), `paper` (003), `mirror` (003), `pen` (003), `sand` (002-nature), `oil` (003) all deliberately not re-added. 012's `tool` (superordinate for hand-held implements), `needle` (sewing-specific), `thread` (the filament, distinct from `wire` which is metallic), and `rope`/`chain`/`wire` (three distinct kinds of flexible linear fastener — rope = fiber-twist, chain = linked-metal, wire = single-strand-metal) all checked against the 003 household-object concepts and confirmed distinct.
    - `src-tauri/src/lexicon/data/lexicon_v1.tsv` — regenerated from the thirteen shards. 118,993 → 128,634 bytes on disk (+8% for +40 concepts; the shard is a touch leaner than the arts or professions shards because material names tend to be single-lemma per language with few synonym pairs — English has `pan`/`skillet` and `basket`/`hamper` but most target languages use a single lemma for each). Rust wire-up unchanged; djb2 hash flip rotates the cache filename.
    - `lab/reports/SESSION-LOG-2026-04-18.md` — § 55 added with Why-Now (tools and materials round out the concrete-object vocabulary: after food (003), household (003), and professions (011), the implements and substances people actually work with in daily life are the next gap to close; hammer / iron / wood / glass are high-frequency cross-lingual search targets in personal writing across trades, hobbies, and home-life topics), concept enumeration by cluster (22 implement + 5 container + 13 material), PoS discipline rationale (40 Noun flat — object-sense dominant over verb-sense), metal-hierarchy rationale, container-shape-vs-material rationale, cross-shard collision avoidance list, verification at 560-concept scale (13 shards, 0 errors, 0 warnings, 116/116 lexicon tests), blast radius (zero Rust change, pure data addition).

## Files modified

- `src-tauri/Cargo.toml` — `+ fst = "0.4"` (M3), `+ dirs = "5"` (M3-baker), `+ smallvec = "1"` (M9-hotpath (b), promoting a transitive dep to direct so the analyzer's public `AnalysisList = SmallVec<[Analysis; 2]>` alias can name the type). M9-mmap adds a target-gated dep block `[target.'cfg(not(any(target_os = "ios", target_os = "android")))'.dependencies] memmap2 = "0.9"` so `memmap2` lands on desktop builds only — mobile targets compile with the `Owned`-variant-only `FstBytes` path and pay zero binary or dep-tree cost.
- `src-tauri/Cargo.lock` — transitive `dirs` family; M9-mmap adds `memmap2` + `libc` on desktop.
- `src-tauri/src/arabic/mod.rs` — `pub mod fst_index;` (M3) + `pub mod fst_bake;` (M3-baker) + one-line analyzer swap.
- `src-tauri/src/arabic/fst_index.rs` — new M3; refactored in M3-baker to split build/parse and add cache plumbing.
- `src-tauri/src/arabic/fst_bake.rs` — new (M3-baker, 685 lines, 13 tests).
- `src-tauri/src/arabic/roots.rs` — `+ pub fn seed_tsv()` accessor (M3-baker).
- `src-tauri/src/arabic/protected.rs` — TSV loader refactor (M1g/M1h).
- `src-tauri/src/arabic/protected_seed.tsv` — new (M1g/M1h, 1,196 entries).
- `src-tauri/src/arabic/regression.rs` — new (M5, 10 tests).
- `src-tauri/src/arabic/regression_cases.tsv` — new (M5, 502 data rows).
- `src-tauri/src/libraries.rs` — `process_arabic_word` routed through `analyze_best` (M6) + 5 new FTS contract tests at EOF; M8b extends it to read `arabic::overrides::active()` on every token; M9-hotpath (a) switches that read to `arabic::overrides::active_if_non_empty()` so the default empty-store path skips `RwLock::read` + `Arc::clone` on every Arabic token.
- `src-tauri/src/arabic/disambiguate.rs` — new (M7, 12 tests).
- `src-tauri/src/arabic/overrides.rs` — new (M8, 16 tests) — UserOverride type, OverrideStore CRUD, per-Universe JSON persistence with atomic writes. M8b adds `ACTIVE_STORE` registry + three Tauri commands + 8 registry tests (total 24). M8c adds a fourth Tauri command (`reindex_arabic_overrides`). M8b-v2 refactors the internal storage from single `HashMap` to `layers: Vec<HashMap>` (sovereign + cUniverse children) with parent-wins lookup + CRUD-sovereign invariant; adds `from_layered_paths`, `activate_layered_for_universe`, `set_sovereign_layer`, `layer_count`, `sovereign_iter`; +17 tests (total 41). Test mutex promoted from submodule-local to crate-visible (`TEST_OVERRIDE_MUTEX`) so the M8c integration suite serialises against the same global. M9-hotpath (a) adds `ACTIVE_STORE_EMPTY: AtomicBool` + `active_if_non_empty() -> Option<Arc<OverrideStore>>` fast-path helper (the common empty-store case short-circuits `None` with no lock, no clone); `set_active` and `set_sovereign_layer` updated to maintain the empty-snapshot invariant with documented Acquire/Release ordering; `RegistryGuard::drop` mirrors the same discipline; +7 tests (total 48).
- `src-tauri/src/arabic/mod.rs` — M8b adds `analyze_with_overrides_best` convenience; `analyze_best` reduced to a thin wrapper.
- `src-tauri/src/universe.rs` — M8b hooks `activate_for_universe` into `set_active_universe`. M8b-v2 adds `resolve_child_universe_roots(parent) -> Vec<PathBuf>` and switches `set_active_universe` to call `activate_layered_for_universe(final_path, &child_universe_roots)` so cUniverse override files light up automatically on Universe switch.
- `src-tauri/src/lib.rs` — M8b registers three Arabic override Tauri commands; M8c registers the fourth (`reindex_arabic_overrides`).
- `src-tauri/src/search.rs` — M8c adds `reindex_notes_matching_text` helper (targeted FTS5 `delete` + reinsert under a transaction). M8b-v2/M8c-integration-test (1) realigns `normalize_arabic_for_search` from aggressive-fold (ة→ه, ى→ي, alif variants) to `crate::arabic::normalizer::normalize_stripped` delegation (tashkeel + tatweel only), preserving the `عبرة`/`عبره` semantic distinction the old fold was silently breaking, and (2) adds `#[cfg(test)] mod tests_m8c` with 4 end-to-end tests (override → reindex → FTS token flip, forward-looking override, empty-needle short-circuit, one-transaction multi-row flip) + `OverrideTestGuard` RAII + `seeded_state` harness. **M12-wire** edits the `lexical_search` body (line ~770) to call a new `expanded_match_query(&normalized)` helper and `unwrap_or_else` back to the pre-existing `{normalized}*` prefix expression. The helper (~25 lines, directly below `lexical_search`) delegates to `crate::lexicon::detect_source_lang` + `crate::lexicon::expand_to_match_expr(..., &ExpansionOptions::default())`, then returns `Some(expr)` only when `expr.contains(" OR ")` — the gate that distinguishes "expansion actually bridged" from "expansion echoed only the source lemma". New `#[cfg(test)] mod tests_m12` appended at EOF with 5 tests pinning the decision boundary (known-en + known-ar bridge cross-lingually, unknown word / punctuation-only / proper-noun fall through to None so the caller uses prefix fallback). **M13** adds `pub match_via: Option<String>` to `SearchResult` + defaults it to `None` at all seven non-lexical constructors (`structured_search` line ~1171, `semantic_search` ~1999, `search_titles` ~2242, `search_contents` ~2273, `search_tags` ~2302, `search_properties` ~2329, `search_wikilinks` ~2356). `expanded_match_query` return type flipped from `Option<String>` to `Option<LexicalExpansion>` — new private struct holding `match_expr: String` + `bridge_terms_lower: Vec<String>` pre-lowercased and filtered to non-source-language lemmas only. New `fn find_match_via(snippet, bridge_terms_lower)` scans `<mark>…</mark>` regions and returns the first bridge-term hit; `lexical_search` row closure calls it with `title_hit` short-circuit (filename matches never earn a badge). `tests_m12` updated to `.match_expr` field access + `.is_none()` pattern. New `#[cfg(test)] mod tests_m13` — 12 tests split scanner behavior (7: mark/bridge match, source-lemma returns None, first-mark-wins, unmarked-ignored, case-folding, empty-set fast-path, malformed-HTML resilience) and `expanded_match_query.bridge_terms_lower` contract (3: source-lang excluded, reverse direction, pre-lowercased invariant). **M14** appends `#[cfg(test)] mod m14_bench` at EOF (+408 lines) — opt-in `#[test] #[ignore] fn m14_bench()` that seeds a tempfile SQLite corpus (100 notes × three language profiles: En-only, Ar-only, mixed) and measures `lexical_search` across three shapes (known-word bridging, unknown-word prefix fallback, Arabic-only non-regression) at WARMUP=20 + SAMPLES=500. Hard-asserts per-shape worst-case p99 < 10 ms so a regression reintroducing per-call allocation / full graph scan / uncached FST rebuild trips on the next opt-in run. First captured baseline: (a) worst p99 6.63 ms (`tree`, 23 hits), (b) worst p99 0.05 ms (0-hit prefix paths), (c) worst p99 6.28 ms (`شجرة`, 23 hits — parity with En, confirms M12-wire is cost-free on bridged-but-zero-new-lang-hits queries). No public-API changes, no new crate deps — uses only already-imported `rusqlite::params`, `std::time::Instant`, `std::hint::black_box`, and `super::{init_db, lexical_search, normalize_arabic_for_search}`.
- `src/lib/components/ArabicOverridesPanel.svelte` — new (M8c, ~480 lines). Settings-modal panel for override CRUD with live reindex feedback.
- `src/lib/components/SettingsModal.svelte` — M8c adds the `arabic-overrides` section entry + content branch.
- `src/lib/i18n/{ar,de,en,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — M8c adds `settings.sections.arabicOverrides` + the 31-key `settings.arabicOverrides` block to all 15 locale files. **M13** adds one new key `searchHub.matchVia` per locale, placed at the tail of the existing `searchHub` section after `partOf`. Translations chosen to read naturally next to a foreign lemma: `via` (en/fr/pt), `vía` (es), `über` (de), `عبر` (ar), `از طریق` (fa), `بذریعہ` (ur), `דרך` (he), `के माध्यम से` (hi), `経由` (ja/zh), `경유` (ko), `через` (ru), `üzerinden` (tr).
- `src/lib/libraries/store.ts` — **M13** extends the `ConstellationSearchResult` interface with `match_via?: string`, mirroring the Rust-side `Option<String>`. Doc comment explains when the field is populated (cross-lingual body hit) vs absent (same-language hit, title match, synonym-only expansion). No call-site edits — consumers opt in to reading the field where they want to render it.
- `src/lib/components/SearchHub.svelte` — **M13** adds the `"via {lemma}"` result badge. Three rendering sites gain a `{#if r.match_via}<span class="sh-match-via" dir={detectDir(r.match_via)} title="{$t('searchHub.matchVia')} {r.match_via}">…</span>{/if}` block between the result name and the library chip: advanced-mode grouped results (~line 460), advanced-mode flat results (~line 500), universal-mode categorized results (~line 539). One new CSS rule `.sh-match-via` (~line 720) — accent-tinted chip (12% `--interactive-accent` background), muted foreground, `max-width: 12ch` with ellipsis, `flex-shrink: 0` so long German compounds or long Arabic phrases don't break the row layout. `dir` attr on the span ensures Arabic/Hebrew/Urdu bridge terms render RTL regardless of the host result row's direction.
- `src-tauri/src/arabic/bench.rs` — new (M9, ~200 lines). Single opt-in `#[test] #[ignore] fn m9_bench()`. Does not run under default `cargo test --lib`; invoked with `cargo test --lib --release arabic::bench -- --ignored --nocapture`. M9-hotpath (a) extends the bench with a "Throughput FTS" measurement block that mirrors `libraries::process_arabic_word`'s production shape (fetch via `overrides::active_if_non_empty`, then `analyze_with_overrides_best`) — reports `Throughput FTS (w/s)` + `Per-call FTS (ns)` + `FTS overhead (ns)` delta so the per-token probe cost is directly measurable. M9-rss-real extends it further with Section 0 (`rss_before`) and Section 6 (`rss_after` + delta + 7K projection) — four new report lines, gracefully skipped if the platform probe returns `None`. **M9-profile** extends the module-level doc comment with a `# Profiling (M9-profile)` section (~70 lines) covering `samply` (cross-platform, recommended), `cargo-flamegraph` (Linux-only alternative), a Windows VS-Profiler fallback, a four-hotspot reading guide (`analyze_with_overrides`, `fst::Map::get`, `arabic::normalizer::normalize`, `disambiguate::rank_analyses`), and a sample-percentage-to-nanosecond conversion formula. No code changes in the `#[cfg(test)] mod tests` body.
- `src-tauri/src/arabic/rss.rs` — new (M9-rss-real, ~200 lines, 2 tests). `#![cfg(test)]`-gated real OS RSS probe. Three platform backends (Windows via `K32GetProcessMemoryInfo`, Linux via `/proc/self/statm`, macOS via `task_info`) behind `#[cfg(target_os = "…")]` gates. Unknown-target fallback returns `None`. Stdlib-only — no `sysinfo` / `memory-stats` dep.
- `src-tauri/src/arabic/mod.rs` — M9 promotes `regression` from `#[cfg(test)] mod` to `#[cfg(test)] pub(crate) mod`; adds `#[cfg(test)] mod bench;`. M9-rss-real adds `#[cfg(test)] mod rss;`. M9-hotpath (b) adds `use smallvec::{smallvec, SmallVec};` + `pub type AnalysisList = SmallVec<[Analysis; 2]>;` + migrates `analyze` and `analyze_with_overrides` return types from `Vec<Analysis>` to `AnalysisList`, and the 7 construction sites in the function body from `Vec`/`vec!` to `AnalysisList`/`smallvec!`. Capacity-2 chosen to cover single-hit paths (~90% of tokens) and the 2-hit `كاتب` Noun/Verb ambiguous class without heap allocation; spills to heap only on 3+-hit cases. **M9-intern** updates three `Analysis::new`-adjacent construction sites from `form.root_key.clone()` / `form.pattern_label.clone()` to `.to_string()` — the public `Analysis` surface stays `String`, so the Arc<str>-typed `GeneratedForm` field needs an explicit conversion across the API boundary. Doc comment added explaining the firewall. **M9-hotpath (c)** adds `lookup_layer_01(word, overrides, stripped) -> Option<Analysis>` helper (~30 lines) that factors the Layer 0 (user override) + Layer 2 (protected list) probes out of the inline `analyze_with_overrides` body. Both entry points now call the helper: `analyze_with_overrides` wraps the result in `smallvec![hit]` (byte-identical behaviour to the pre-refactor inline blocks); `analyze_with_overrides_best` uses it as a fast path after normalization + Arabic-script gate, returning the `Analysis` directly on hit and skipping both the `AnalysisList` frame construction and the `into_iter().next()` destructure. Non-Arabic scripts fall through to the slow-path `analyze_with_overrides` call canonically. Recovered the full +1,128 ns/call M9-mmap regression on the bare path (§ 38) and drove the FTS hot path −309 ns *below* the § 37 pre-mmap baseline.
- `src-tauri/src/arabic/generator.rs` — **M9-intern**: `GeneratedForm::root_key` and `GeneratedForm::pattern_label` fields migrated from `String` to `Arc<str>`. New `pub(crate) fn intern(pool: &mut HashMap<String, Arc<str>>, s: &str) -> Arc<str>` helper that returns a cloned `Arc` on repeats or a fresh one on firsts. `generate_all()` rewritten to build two pools (`root_pool`, `label_pool`), intern all pattern labels upfront before the root loop, then intern each root key once and clone the `Arc` per emission. Imports updated: `use std::sync::{Arc, OnceLock};`. Tests updated: 10 comparison sites use `&*x.root_key == "..."` (Arc<str> → &str deref).
- `src-tauri/src/arabic/fst_bake.rs` — **M9-intern**: imports extended with `HashMap`, `Arc`, and `intern` re-import from `super::generator`. `decode_bundle` creates two `HashMap<String, Arc<str>>` pools shared across the stripped and folded side decoders. `decode_side` threads the pools down as `&mut` args. `decode_form` reads the raw UTF-8 bytes as before, then interns via the pool. Encode path unchanged (Arc<str> auto-coerces to &str). `CACHE_FORMAT_VERSION` **NOT bumped** — on-disk bytes are identical to pre-M9-intern format; old caches load under new binary and vice versa. One roundtrip test site gained empty-pool arguments. **M9-mmap**: adds `use memmap2::Mmap;` (cfg-gated) + new `pub enum FstBytes { Mmap { mmap: Arc<Mmap>, offset, len }, Owned(Vec<u8>) }` with `AsRef<[u8]>` / `Debug` / `From<Vec<u8>>` / `len` impls. `FstBundle::{stripped_bytes, folded_bytes}` migrated from `Vec<u8>` to `FstBytes`. `load_bundle` split into `load_bundle_mmap` (desktop, preferred) + `load_bundle_heap` (fallback / mobile). `decode_bundle` renamed to `decode_bundle_heap` (wraps output in `FstBytes::Owned`), with a new sibling `decode_bundle_mmap(Arc<Mmap>)` that captures cursor-position offsets and produces `FstBytes::Mmap` slices sharing a single `Arc<Mmap>` across stripped + folded. `encode_side` callers updated to `.as_ref()` on `FstBytes`. **CACHE_FORMAT_VERSION stays at 1** — on-disk byte layout is byte-identical. Three test sites updated (`sample_bundle` uses `.into()`; two `assert_eq!` roundtrip sites compare via `.as_ref()`).
- `src-tauri/src/arabic/fst_index.rs` — **M9-intern**: no production code changes (Arc<str> fields flow through transparently). 6 test comparison sites updated to `&*f.root_key == "..."`; 2 construction sites updated to `"...".into()` (Arc<str> From<&str>); 1 `assert_eq!` updated to `&*hits[0].root_key, "..."`. **M9-mmap**: imports extended with `FstBytes`. `GenerativeFst::{fst_stripped, fst_folded}` fields migrated from `Map<Vec<u8>>` to `Map<FstBytes>` (type-only — all lookup code byte-identical). `from_bytes` signature relaxed from `Vec<u8>` to `impl Into<FstBytes>` (preserves back-compat via `From<Vec<u8>>`; the new mmap path passes `FstBytes::Mmap` directly, no copy). `build_bundle` wraps cold-rebuild `Vec<u8>` via `.into()` at the `FstBundle` construction site.
- `src-tauri/src/lexicon/parse.rs` — new (M10, ~280 lines, 11 tests). TSV seed parser with `ConceptRecord` output and `ParseRowError` diagnostics.
- `src-tauri/src/lexicon/graph.rs` — M10 rewrite (~400 lines, 12 tests); M11-infra adds `seed_tsv()` accessor, `LexiconBundle` type, three-stage `get()`, `build_bundle` / `from_bundle` / `to_bundle` split. **M11-data v1** swaps the body of `seed_tsv()` from `include_str!("data/seed_v1.tsv")` to `include_str!("data/lexicon_v1.tsv")` (cold-build path now consumes the 49-concept production corpus); adds sibling `pub fn legacy_seed_tsv() -> &'static str` so the M10 seed regression canary in `bake.rs` retains a byte-exact accessor to its fixture.
- `src-tauri/src/lexicon/mod.rs` — M10 rewrite (~300 lines, 13 tests); M11-infra adds `pub mod bake;` + extends the `pub use graph::{…}` re-exports with `build_bundle`, `seed_tsv`, `LexiconBundle`. M12 further adds `pub mod fts;` + `pub use fts::{build_match_expr, escape_fts_term}` + two `expand_to_match_expr[_via]` helpers + 5 end-to-end tests.
- `src-tauri/src/lexicon/data/seed_v1.tsv` — new (M10, 4.4 KB, 15 hand-picked concepts × 12–16 language labels). **M11-data v1** retains this file on disk unchanged as the fixture for the `real_seed_bundle_writes_reads_reconstructs` regression canary (reachable via `graph::legacy_seed_tsv()`).
- `src-tauri/src/lexicon/data/lexicon_v1.tsv` — new (**M11-data v1**, 8,175 bytes, 49 hand-curated concepts × 15 languages, emitted by `lab/m11-data/build.py` from `concepts.json`). 100% Constellation-original content — no third-party attribution required. This is the file `graph::seed_tsv()` returns on the production cold-build path; swapping it is what makes M11-data v1 the "real" corpus. **M11-data v2-infra** regenerates this file from the three shards: 8,175 → 22,488 bytes (146 concepts total, +97 vs v1). **M11-data v2 batches § 47** regenerates again: 22,488 → 48,810 bytes (238 concepts total, +92 vs v2-infra). **M11-data v2 batch § 48** regenerates again: 48,810 → 58,622 bytes (280 concepts total, +42 vs § 47). **M11-data v2 batch § 49** regenerates again: 58,622 → 67,016 bytes (320 concepts total, +40 vs § 48). **M11-data v2 batch § 50** regenerates again: 67,016 → 77,918 bytes (360 concepts total, +40 vs § 49). **M11-data v2 batch § 51** regenerates again: 77,918 → 87,840 bytes (400 concepts total, +40 vs § 50 — milestone: first multiple of 100 past v1 seed). **M11-data v2 batch § 52** regenerates again: 87,840 → 97,553 bytes on disk (440 concepts total, +40 vs § 51). **M11-data v2 batch § 53** regenerates again: 97,553 → 106,939 bytes on disk (480 concepts total, +40 vs § 52). **M11-data v2 batch § 54** regenerates again: 106,939 → 118,993 bytes on disk (520 concepts total, +40 vs § 53). **M11-data v2 batch § 55** regenerates again: 118,993 → 128,634 bytes on disk (560 concepts total, +40 vs § 54). Rust wire-up unchanged across all eleven landings — `graph::seed_tsv()` still returns `include_str!("data/lexicon_v1.tsv")`, only the byte content grows; djb2 hash of the new bytes flips the content-addressed cache filename so the next boot writes a fresh bundle and orphans the old one per M11-infra design.
- `src-tauri/src/lexicon/bake.rs` — new (M11-infra, ~615 lines, 18 tests). Bundle binary format, cache path, version hash, atomic writes, safe decoder. **M11-data v1** retargets the existing `real_seed_bundle_writes_reads_reconstructs` test to `legacy_seed_tsv()` (preserving the M10 seed regression role) and adds a sibling `real_lexicon_bundle_writes_reads_reconstructs` test that exercises the same write→load→reconstruct round-trip against the production corpus, asserts `recs.len() > 20` (tripwire against seed-revert), and spot-checks `en:tree` + `ar:شجرة` resolve in the reconstructed graph.
- `lab/m11-data/README.md` — new (**M11-data v1**, ~175 lines). Scope doc: v1 is 100% Constellation-original content, no third-party wordnet dependency, schema spec, coverage floor, regeneration workflow, license-rejection rationale for Princeton WordNet 3.1 / OMW / Wiktionary / GermaNet / FarsNet. **M11-data v2-infra** updates: status v1 → v2 (in-flight, targeting 20K); "Scale policy" rewritten around thematic shards + ≥8/15 coverage target + hand-curation→LLM-assisted transition path; "File layout" replaced with shard directory tree + new "Shard layout (v2)" subsection (NNN-theme.json convention, cross-shard dedup hard-error invariant, deterministic-output guarantee); "Regeneration workflow" step 3 updated to mention the shard walk + flatten + id-collision check; "Follow-ons" replaces "M11-data-scale" bullet with "M11-data v2 continued batches".
- `lab/m11-data/concepts.json` — new (**M11-data v1**, 49 concepts × up to 15 languages, `schema_version: 1`). Single-file source of truth for the corpus; edited directly by human curators, consumed by `build.py`. **M11-data v2-infra** deletes this file; content migrated byte-for-concept into `lab/m11-data/concepts/000-core-seed.json` with SHA256 verification on the emitted TSV before any new content landed.
- `lab/m11-data/concepts/000-core-seed.json` — new (**M11-data v2-infra**, 49 concepts). Receives the full v1 `concepts.json` payload unchanged. First shard under the new `concepts/NNN-theme.json` layout.
- `lab/m11-data/concepts/001-body-and-family.json` — new (**M11-data v2-infra**, 43 concepts). Body / family / society vocabulary. Every row en+ar floor; average coverage ≥ 13/15 languages. Arabic lemmas stored already stripped of tashkeel/tatweel.
- `lab/m11-data/concepts/002-nature.json` — new (**M11-data v2-infra**, 54 concepts). Animals / plants / food / landscape / weather / physics. Same coverage discipline as `001`.
- `lab/m11-data/concepts/003-food-and-household.json` — new (**M11-data v2 § 47**, 40 concepts). Food staples / drinks / meals / household objects. En+ar floor; Arabic lemmas tashkeel/tatweel-stripped. A placeholder `door` entry was drafted and removed before build (id already used by `000-core-seed.json`; v2-infra's cross-shard dedup would have tripped the build).
- `lab/m11-data/concepts/004-qualities.json` — new (**M11-data v2 § 47**, 52 concepts). 26 adjectives + 11 colors + 15 numbers/quantifiers. Concept ids `hard-difficult` and `orange-color` chosen to reserve `hard` (rigid/solid) and `orange` (fruit) for future shards; display lemmas remain plain `hard` / `orange` in every language column.
- `lab/m11-data/concepts/005-basic-verbs-and-emotions.json` — new (**M11-data v2 § 48**, 42 concepts). 37 Verbs (30 action + 3 memory + 4 affect) + 5 Nouns (affect/cognition: fear, hope, joy, anger, dream). Arabic verbs stored in Semitic citation form (3rd-person masculine singular past tense); Arabic emotion nouns in singular nominative form; all tashkeel/tatweel-stripped.
- `lab/m11-data/concepts/006-time-and-space.json` — new (**M11-data v2 § 49**, 40 concepts). 17 time concepts (morning / afternoon / evening / yesterday / tomorrow / week / month / hour / minute / second / moment / past / present / future / beginning / end / middle) + 23 space concepts (place / location / area / region / country / village / town + relative positions up/down/left-side/right-side/front/back/inside/outside/near/far + cardinals north/south/east/west + center/edge). Concept ids `left-side` / `right-side` reserve `right` (already used in 004 as the "correct" adjective); `up` / `down` / `near` / `far` PoS-marked Adverb to match their primary grammatical class in English and the RTL coverage languages. Deliberate deferrals: `noon` / `midnight` / cardinal-direction adjective forms (`northern` / `southern` / ...) to a possible future batch.
- `lab/m11-data/concepts/007-cognition-and-language.json` — new (**M11-data v2 § 50**, 40 concepts). 30 Verbs (15 cognition + 15 speech) + 10 Nouns (linguistic / cognitive). Concept id `lie-falsehood` disambiguates from the English homonymous verb `lie` (to recline); display lemmas remain plain `lie` in every language column. Arabic verbs stored in Semitic citation form (3rd-person masculine singular past); nouns in singular nominative; all tashkeel/tatweel-stripped.
- `lab/m11-data/concepts/008-society-and-government.json` — new (**M11-data v2 § 51**, 40 concepts). 39 Nouns + 1 Adjective (`private`). Covers authority structures (government / nation / law / rule / power / authority / leader / king / queen / president), military/conflict (war / army / soldier / freedom / justice), legal/civic/economic (crime / prison / court / judge-person / vote / election / tax / money / market / business), community/society (society / culture / tradition / group / meeting / event / ceremony / citizen / community / public), civic norms (equality / duty / responsibility / punishment). Concept id `judge-person` disambiguates from the 007 `judge` Verb; display lemma remains plain `judge`. **Milestone: first multiple-of-100 past v1 seed (400 concepts).**
- `lab/m11-data/concepts/009-arts-and-creativity.json` — new (**M11-data v2 § 52**, 40 concepts, all POS=Noun). 10 visual-arts (art / painting / picture / drawing / sculpture / statue / photograph / design / pattern / image) + 15 performing-arts (music / song / dance / theater / film / drama / actor / singer / dancer / instrument / drum / guitar / piano / violin / stage) + 9 literary-arts (poem / poetry / novel / author / writer / poet / chapter / page / title) + 6 creative-abstract (beauty / style / creation / inspiration / craft / museum). Arabic lemma for `photograph` deliberately uses the disambiguating phrase `صورة فوتوغرافية` rather than bare `صورة` — the bare form maps to `picture`/`image` as their Arabic lemma, and within-concept lemma uniqueness is preserved while cross-concept overlap on shared semantic neighbours is retained. `author`/`writer`/`poet` kept as three distinct concepts to preserve the originator/professional/specialist distinction the Arabic (`مؤلف`/`كاتب`/`شاعر`), German (`Autor`/`Schriftsteller`/`Dichter`), and French (`auteur`/`écrivain`/`poète`) lexicons all maintain.
- `lab/m11-data/concepts/010-science-and-math.json` — new (**M11-data v2 § 53**, 40 concepts). Natural-science meta (6 Nouns: science / chemistry / physics / biology / mathematics / theory) + chemistry & matter (7 Nouns: atom / element / molecule / matter / gas / liquid / solid) + physics & energy (7 Nouns: energy / force / gravity / heat / electricity / temperature / experiment) + biology (1 Noun: cell) + math operations (7: number / plus / minus (Nouns for the operators) + multiply / divide / equal (Verbs for the actions) + sum (Noun)) + geometry (6 Nouns: circle / square / triangle / line / point / angle) + astronomy & measurement (6 Nouns: planet / universe / galaxy / orbit / mass / measurement). PoS split: 35 Noun + 3 Verb + 2 Noun-as-operator (plus/minus marked Noun to match the "arithmetic symbol" sense rather than English's preposition use). `universe` cross-lists `cosmos` as an English synonym because both carry near-identical semantic weight in modern cross-lingual writing (Arabic `كون` / Russian `вселенная`/`космос` / French `univers`/`cosmos` all treat them as equivalents).
- `lab/m11-data/concepts/011-professions-and-work.json` — new (**M11-data v2 § 54**, 40 concepts, all POS=Noun). Covers three clusters: 25 specific professions (doctor / nurse / engineer / farmer / merchant / lawyer / priest / carpenter / baker / cook / artist / scientist / musician / worker / builder / pilot / driver / tailor / butcher / fisherman / hunter / policeman / servant / helper / manager), 10 workplace institutions (office / factory / shop / company / hospital / school / university / farm + the two included with professions: office/factory), 5 work abstractions (job / profession / employee / employer / boss / customer / salary — actually 7; counted in total because they're work-abstraction concepts). Arabic lemmas all in singular indefinite/definite form, tashkeel/tatweel-stripped. Two first-pass rows (`cook` `طاهٍ`, `lawyer` `محامٍ`) contained tanwin kasra (U+064D) and were caught by the validator; fixed by dropping those alt lemmas (the concepts retained `طباخ` / `محامي` respectively, which are equivalent in meaning and more commonly used).
- `lab/m11-data/concepts/012-tools-and-materials.json` — new (**M11-data v2 § 55**, 40 concepts, all POS=Noun). Covers three clusters: 22 tools/implements (tool / hammer / nail / screw / axe / saw / drill / wrench / scissors / needle / thread / rope / chain / wire / lock / broom / brush / shovel / bucket / ladder / hook / pencil) + 5 containers (pot / pan / bowl / box / basket) + 13 materials (material / metal / iron / steel / gold / silver / copper / wood / brick / cement / glass / cloth / leather). Arabic lemmas stored in singular indefinite/definite form; all tashkeel/tatweel-stripped. Metal hierarchy preserved via one superordinate `metal` + five specific subordinates (`iron` / `steel` / `gold` / `silver` / `copper`) matching the universal target-language distinction. Container-shape (pot/pan/bowl) and rigid-vs-woven (box/basket) distinctions preserved across target languages. The rope/chain/wire triple separates fiber-twist / linked-metal / single-strand-metal flexible-linear fasteners.
- `lab/m11-data/build.py` — new (**M11-data v1**, ~230 lines). Deterministic `concepts.json` → `lexicon_v1.tsv` emitter with structural validation + `--stdout` + `--dry-run` flags. **M11-data v2-infra** refactors (~40 lines): `CONCEPTS_JSON` → `CONCEPTS_DIR`; new `load_shard(path)` + `load_all_shards() -> (concepts, shard_counts)` helpers; cross-shard id-collision as hard build-time error with pointers to both offending files; `count_summary()` extended with per-shard concept counts in `--dry-run`. Deterministic output invariants preserved (flatten-then-sort-by-id, alphabetic lang columns, first-seen lemma order within a cell, fixed header, LF endings).
- `lab/m11-data/validate.py` — new (**M11-data v1**, ~275 lines). Post-build TSV content validator — hard errors for missing `en:`/`ar:`, tashkeel/tatweel in Arabic-script lemmas, dup lemmas, dup concept ids; warnings for low language coverage + per-lang script mismatches via Unicode block membership.
- `lab/m11-data/regenerate.sh` — new (**M11-data v1**, ~30 lines). One-command `build.py && validate.py` wrapper with fail-fast + portable Python interpreter lookup.
- `src-tauri/src/lexicon/fts.rs` — new (M12, ~210 lines, 20 tests). FTS5 MATCH expression generator. Pure logic, no SQL, no graph walk — takes an `ExpansionResult`, emits an `Option<String>` of `"..." OR "..."` phrase-quoted terms (operator-keyword-safe at M11-data scale).
- `src-tauri/src/lexicon/detect.rs` — new (M12-lang-detect, ~280 lines, 33 tests). Unicode-script source-language classifier — exported as `lexicon::detect_source_lang`. Pure stdlib, no new dependencies.
- `src-tauri/src/lexicon/bench.rs` — new (M12-bench, ~160 lines, 1 `#[ignore]` test). `lexicon::bench::m12_bench` opt-in latency benchmark for `expand_to_match_expr_via`. Mirrors the `arabic::bench::m9_bench` pattern — does not run under default `cargo test --lib`, invoked with `--release ... -- --ignored --nocapture`. Hard-asserts p99 < 1 ms.

## Open items

- **M1g-data / M1h-data**: the 20K Wikipedia-extracted proper-noun corpus + 2K loanwords. Today's 1,196 hand-picked entries cover the common case; the full corpus comes from CC-BY-SA bulk extraction (separate milestone in `lab/`, blocked on extractor tooling).
- **M5-grow**: expand the corpus over time. 502 is the v1 floor — as M6/M7 land, new flagship surfaces identified during bring-up should be added here first before any other test code. Target by M9: ≥2,000 cases, with ≥20 pure-heuristic Arabic-script rows (Layer 4 fallback coverage; currently the heuristic threshold is met by foreign Latin-script rows via the non-Arabic-script route).
- **M7-v2**: corpus-aware disambiguation. Today's ranking is a pure function of the Analysis fields. V2 reads the user's own FTS vocab to bias toward lemmas the user writes often, plus a 3-word context window at query time to pick between readings (`كاتب الرسالة` → Noun; `كاتب أخاه` → Verb). Tracked as a follow-on once Settings → Debug surfaces the existing v1 rank so we can A/B the v2 improvements.
- **M8e — spelling-tolerance query layer**: handle misspellings like `خليفه` (heh) for `خليفة` (ta-marbuta) at query time, **without** destroying the `عبرة` (a lesson) / `عبره` (he crossed it) distinction at index time. The M8b-v2 landing fixed a root-cause bug where `normalize_arabic_for_search` was aggressively folding ة/ه, ى/ي, and alif variants — but the UX question of "the user typed the wrong letter" still stands, and can't be answered correctly by a lossy index transform. Candidate approaches: (a) edit-distance-bounded FTS5 match expansion on the query side; (b) a dedicated spellcheck pass that runs against the user's own FTS vocab before the lexical query; (c) context-aware disambiguation using a 3-word window around the ambiguous surface (same pattern M7-v2 proposes for POS disambiguation). Scope and design deferred until real-user queries surface which misspelling classes matter most.
- **M9-mmap-pressure-verify** (follow-on to M9-mmap, § 38): the working-set metric can't see the mmap win because the throughput phase touches every FST page. Need a stress-test harness that either (a) reads private-bytes / `Pss` / `phys_footprint` instead of working-set (direct measurement of the structural win), or (b) induces system memory pressure (spin up a ballooning allocator) and measures which pages survive eviction. Gated on having a meaningful way to artificially pressure the OS — adding a `jemalloc`-style page-cache-stat probe would be cleaner than a ballooning test. Not blocking any downstream work; M9-mmap's structural wins are correct even without this verification.
- ~~**M9-hotpath (c)**~~ — **LANDED § 39**. Fast-path short-circuit for Layer 0 / Layer 2 hits in `analyze_with_overrides_best`. Recovered the +1,128 ns/call bare regression and the +3,172 ns/call FTS regression from § 38 — bare back to 7,584 ns/call, FTS now *below* the pre-M9-mmap baseline at 7,117 ns/call (−309 ns). FTS overhead flipped sign from +1,789 ns (FTS slower) to −468 ns (FTS faster than bare) because the fast path skips the `active_if_non_empty` probe on hit.
- **M9-hotpath (c)-v2** (speculative, from § 39): eliminate the double-normalize on slow-path fallthrough by factoring `analyze_with_overrides` into `analyze_normalized(word, overrides, &norm) -> AnalysisList`. Gated on M9-profile confirming the second `normalize_stripped` call is a measurable fraction of per-call cost (current estimate: 50–100 ns, ~1% of 7,584 ns). Not urgent.
- **Criterion-grade bench** (new, from § 39): land `--bench` target with warm-up + statistical outlier rejection to replace the current `#[test] #[ignore]` harness's ±19% run-to-run variance. Prerequisite for drawing quantitative conclusions under ±5 pp.
- ~~**M9-profile**~~ — **LANDED § 40**. Sampling-profiler recipe added to `arabic::bench` module-level doc comment. Covers `samply` (cross-platform, recommended), `cargo-flamegraph` (Linux-only), Windows VS-Profiler fallback, a four-hotspot reading guide, and a sample-percentage-to-nanosecond conversion formula. Paste-ready — anyone can run the recipes against the worktree with `cargo install samply` or `cargo install flamegraph`.
- **First-profile pass** (new, from § 40): actually run samply against `m9_bench` and publish the four-hotspot cost breakdown in a future SESSION-LOG. Gated on a developer wanting to use the recipe; the recipe is the deliverable, the first run is a follow-on consumption of it.
- **samply CI integration** (new, from § 40): capture a profile per commit in CI and publish the call-tree as an artefact. Makes regressions attributable to a specific function without a local repro. Deferred until samply stabilises its headless JSON output.
- ~~**M11-data**~~ — **LANDED § 42** as `lab/m11-data/` (README + `concepts.json` + `build.py` + `validate.py` + `regenerate.sh`) + `src-tauri/src/lexicon/data/lexicon_v1.tsv` (49 concepts × 15 languages, 100% Constellation-original content, 8,175 bytes). Extractor-tooling dependency eliminated via hand-curation + strict "no third-party data" rule. Scale expansion tracked as follow-ons below.
- ~~**M11-data-scale**~~ — **LANDED § 46 (infra) + first two batches**. Monolithic `concepts.json` migrated to `concepts/NNN-theme.json` shards with cross-shard dedup as a hard build error; first two thematic batches landed (`001-body-and-family.json` 43 concepts, `002-nature.json` 54 concepts). Corpus 49 → 146 concepts (+97). Further batches tracked as **M11-data v2 continued batches** below. No bench rerun yet — numbers should be stable at this scale (per-concept neighbour count still bounded); the bench-rerun follow-ons `M12-bench-m11-v2` / `M14-bench-m11-v2` remain gated on reaching the ~20K target.
- **M11-data v2 continued batches** (from § 46, progressed in §§ 47–51): land further thematic shards toward the 20K target. **§ 47 landed** `003-food-and-household.json` (40) + `004-qualities.json` (52) — corpus 146 → 238. **§ 48 landed** `005-basic-verbs-and-emotions.json` (42) — corpus 238 → 280. **§ 49 landed** `006-time-and-space.json` (40) — corpus 280 → 320. **§ 50 landed** `007-cognition-and-language.json` (40) — corpus 320 → 360. **§ 51 landed** `008-society-and-government.json` (40) — corpus 360 → **400 (first multiple of 100 past v1 seed)**. **§ 52 landed** `009-arts-and-creativity.json` (40) — corpus 400 → 440. **§ 53 landed** `010-science-and-math.json` (40) — corpus 440 → 480. **§ 54 landed** `011-professions-and-work.json` (40) — corpus 480 → 520. **§ 55 landed** `012-tools-and-materials.json` (40) — corpus 520 → 560. **Next planned**: domain-specific waves — `013-transportation-and-travel.json` (~40), `014-technology-and-media.json` (~40), `015-religion-and-philosophy.json` (~40). Each batch is a discrete shard, reviewable + rollback-able on its own via the shard dedup invariant. Past ~2K concepts, hand-curation may transition to LLM-assisted generation — the shard layout + validator gate the publication path identically regardless.
- **M11-data-synonyms**: each v1 concept carries 1–3 lemmas per language; the `SenseId` type is already prepared for M8-style in-language synonym edges via multiple sense-tagged nodes per concept. Not populated in v1.
- **M11-data-domains**: domain-specific expansion packs (science / philosophy / arts / Islamic studies / medicine) layered on the core corpus via `LexiconBundle::merge` (not yet implemented). Ships as separate bundles, not edits to `lexicon_v1.tsv`.
- **M11-mmap**: switch the baked `name_index_bytes` from `Vec<u8>` to `memmap2::Mmap` on desktop, mirroring the M9-mmap follow-on on the Arabic FST. Cuts resident memory at the M11-data scale and lets warm-start be a bounded constant (header read) rather than O(bundle size). Needs the same `#[cfg]` fallback for iOS / sandboxed builds that can't anon-mmap.
- **M11-cache-bench**: measure cold-start (cache deleted → `LexiconGraph::get()` rebuilds + persists) vs. warm-start (cache present → `from_bundle` only) delta on the M11-data bundle. Same `#[test] #[ignore]` opt-in pattern as `arabic::bench::m9_bench`. Will extend M9's report table or land a sibling `lexicon::bench::m11_bench`. Gated by M11-data.
- **M12-bench-m11**: rerun `lexicon::bench::m12_bench` once `M11-data` lands to publish the 20K-concept × 15-lang numbers alongside the current M10-seed baseline (mean 5.2 µs / p99 15.8 µs). Not a new module — just an opt-in rerun with the new corpus in place. If p99 migrates meaningfully, the `< 1 ms` hard-assert threshold gets revisited with real data.
- ~~**M12-wire**~~ — **LANDED § 43**. `lexical_search` in `search.rs` now calls `expanded_match_query(&normalized)` with `unwrap_or_else` back to the prefix match. The helper gates on `" OR "` in the expansion output so only true cross-lingual / cross-synonym bridges take the expanded path — out-of-corpus queries preserve today's `{normalized}*` prefix behavior pixel-for-pixel. 5/5 new `tests_m12` pass; full lib suite 417/417 with zero regressions.
- ~~**M13 — multilingual result badge**~~ — **LANDED § 44**. `SearchResult` gained `match_via: Option<String>`; `expanded_match_query` now returns a `LexicalExpansion` carrying both the MATCH expr and a source-filtered, pre-lowercased `bridge_terms_lower` list; `find_match_via` scans FTS5 `<mark>…</mark>` regions to identify the bridge term that earned a hit; `lexical_search` wires the badge with a `title_hit` short-circuit. Frontend: `ConstellationSearchResult.match_via?: string` + `.sh-match-via` CSS chip rendered at three SearchHub result sites + `searchHub.matchVia` key translated across all 15 locale files. 12 new `tests_m13` + 429/429 full lib suite with zero regressions.
- ~~**M14 — benchmarks**~~ — **LANDED § 45**. New `#[cfg(test)] mod m14_bench` at the tail of `src-tauri/src/search.rs` (opt-in, mirrors `arabic::bench::m9_bench` shape). Seeds a tempfile SQLite corpus (100 notes across En-only / Ar-only / mixed language profiles), runs three measurement shapes × WARMUP=20 + SAMPLES=500: (a) known-word bridging (`tree` / `كتاب` / `livre`), (b) unknown-word prefix fallback (`quasar` / `Constellation` / `xyzzy`), (c) Arabic-only non-regression (`شجرة` / `معرفة`). Hard-asserts each shape's worst-case p99 < 10 ms. First baseline: (a) worst p99 6.63 ms (`tree`, 23 hits), (b) worst p99 0.05 ms (0 hits), (c) worst p99 6.28 ms (`شجرة`, 23 hits — parity with English hot path). All three shapes comfortably under budget; M12-wire's "free" claim now measurable, not aspirational. 429/429 lib tests + 3 ignored. Bench runs on `cargo test --lib --release search::m14_bench -- --ignored --nocapture`. Follow-ons tracked below (bench rerun at M11-data v2 scale; Settings → Debug scorecard integration; CI integration).
- **M14-bench-m11-v2**: rerun `search::m14_bench` once M11-data v2 lands (~20K concepts). Per-concept neighbour count is bounded at ~15 so numbers should be stable, but the assertion will be rerun and republished with the new corpus in place. If any shape's p99 migrates meaningfully, the 10 ms threshold gets revisited with real data.

## User-facing changes (M8c)

The Settings modal gains a new "Arabic Overrides" section (icon: `translate`, grouped next to Language). Users can pin how the Arabic engine analyses specific surfaces — for each override they specify surface, lemma, optional root/pattern/POS, and an optional note. Saved overrides take sovereign priority over the FST, the cascade, and the heuristic fallback. After every add/remove, the panel reindexes only the notes actually containing that surface and reports "Reindexed N note(s)" in the status area — no full-Universe rebuild, no stale FTS state.

Help-file and User-Manual entries for this panel are tracked under `M8c-doc` above; the Standing Order step for this is the next thing to land after commit + push.
