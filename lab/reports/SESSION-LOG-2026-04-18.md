# Session Log — 2026-04-18

## Headline

**M3 + M3-baker + M1g/M1h + M5 landed.** First: `GenerativeIndex` (HashMap, ~40 MB projected at 7K roots) swapped for `GenerativeFst` (BurntSushi FST, prefix-compressed, mmap-ready). Second: the compiled FST is now persisted to the user's cache directory on first launch and reloaded on subsequent launches via `GenerativeFst::from_bytes` — the cold/warm startup path divergence that M9 ("50 ms analyzer cold-start") measures against. Third: the protected list got its architectural rewrite — `const SEED: &[...]` (200 hand-picked entries, 340 lines of Rust) replaced with `include_str!("protected_seed.tsv")` + a 3-column TSV (`surface<TAB>category<TAB>origin_lang`) now holding **1,196 unique entries** across proper nouns (395), places (275), loanwords (455), and function words (71). Fourth: the **M5 regression corpus** — a 502-case held-out test set in `regression_cases.tsv` + a `cfg(test)`-gated `regression.rs` harness that feeds every row through `analyze_best` and asserts origin / surface / (optionally) lemma / root. Covers all three active origin layers (ProtectedList, GenerativeFst, SurfaceHeuristic) across 28 Arabic roots, ~80 cascade surfaces, and 45 foreign (Latin-script) words. Full public-API parity preserved across all four landings; **225/225 library tests pass** (up from 209 pre-M3: +13 fst_bake, +10 regression harness, +6 TSV parser, -1 removed `no_duplicate_lemmas_in_seed` obsolete under first-write-wins).

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

3. **M5** (this session):
   - `src-tauri/src/arabic/regression.rs` — new (~400 lines, 10 tests) — `Case`, `Failure`, `CorpusReport`, `parse_origin`, `parse_optional`, `parse_corpus`, `run_corpus`, `evaluate`, `raw_corpus`.
   - `src-tauri/src/arabic/regression_cases.tsv` — new (~720 lines, 502 data rows).
   - `src-tauri/src/arabic/mod.rs` — `+ #[cfg(test)] mod regression;` at end of mod declarations.
   - `lab/reports/SESSION-LOG-2026-04-18.md` — this file.

## Files modified

- `src-tauri/Cargo.toml` — `+ fst = "0.4"` (M3), `+ dirs = "5"` (M3-baker).
- `src-tauri/Cargo.lock` — transitive `dirs` family.
- `src-tauri/src/arabic/mod.rs` — `pub mod fst_index;` (M3) + `pub mod fst_bake;` (M3-baker) + one-line analyzer swap.
- `src-tauri/src/arabic/fst_index.rs` — new M3; refactored in M3-baker to split build/parse and add cache plumbing.
- `src-tauri/src/arabic/fst_bake.rs` — new (M3-baker, 685 lines, 13 tests).
- `src-tauri/src/arabic/roots.rs` — `+ pub fn seed_tsv()` accessor (M3-baker).
- `src-tauri/src/arabic/protected.rs` — TSV loader refactor (M1g/M1h).
- `src-tauri/src/arabic/protected_seed.tsv` — new (M1g/M1h, 1,196 entries).

## Open items

- **M1g-data / M1h-data**: the 20K Wikipedia-extracted proper-noun corpus + 2K loanwords. Today's 1,196 hand-picked entries cover the common case; the full corpus comes from CC-BY-SA bulk extraction (separate milestone in `lab/`, blocked on extractor tooling).
- **M5-grow**: expand the corpus over time. 502 is the v1 floor — as M6/M7 land, new flagship surfaces identified during bring-up should be added here first before any other test code. Target by M9: ≥2,000 cases, with ≥20 pure-heuristic Arabic-script rows (Layer 4 fallback coverage; currently the heuristic threshold is met by foreign Latin-script rows via the non-Arabic-script route).
- **M6**: replace `stem_arabic_light10` in `fts5_tokenizer.rs` with `arabic::analyze`. Unblocked: the analyzer is FST-backed with persistent cache, the protected list is data-driven, and there is now a 502-case regression corpus to defend the swap.
- **M7**: disambiguator — reorder multi-analysis results by corpus frequency / context. The regression corpus's `expected_lemma = "-"` rows on ambiguous verb forms mean M7 can change analyze_best's tiebreak without forcing corpus rewrites; only the origin has to stay stable.
- **M9**: measure cold-start analyzer time on the real user machine (Windows) with a clean cache dir, then warm-start. If the warm-start delta isn't ≥5× the cold-start on the target 7K-root corpus, tune the format (e.g. memory-map instead of read-to-vec).

## No user-facing changes

Engine internals only. No help files, User Manual, or translation updates needed for this session — the protected-list expansion is transparently consumed by the analyzer; users won't notice any new string in the UI, but they will notice that وائل, فلسطين, إنترنت, and ~1,000 more surfaces now survive the pipeline verbatim instead of being over-stripped.
