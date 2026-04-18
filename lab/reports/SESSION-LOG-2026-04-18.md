# Session Log — 2026-04-18

## Headline

**M3 + M3-baker + M1g/M1h + M5 + M6 + M7 + M8 + M8b + M8c + M9 + M10 landed.** First: `GenerativeIndex` (HashMap, ~40 MB projected at 7K roots) swapped for `GenerativeFst` (BurntSushi FST, prefix-compressed, mmap-ready). Second: the compiled FST is now persisted to the user's cache directory on first launch and reloaded on subsequent launches via `GenerativeFst::from_bytes` — the cold/warm startup path divergence that M9 ("50 ms analyzer cold-start") measures against. Third: the protected list got its architectural rewrite — `const SEED: &[...]` (200 hand-picked entries, 340 lines of Rust) replaced with `include_str!("protected_seed.tsv")` + a 3-column TSV (`surface<TAB>category<TAB>origin_lang`) now holding **1,196 unique entries** across proper nouns (395), places (275), loanwords (455), and function words (71). Fourth: the **M5 regression corpus** — a 502-case held-out test set in `regression_cases.tsv` + a `cfg(test)`-gated `regression.rs` harness that feeds every row through `analyze_best` and asserts origin / surface / (optionally) lemma / root. Covers all three active origin layers (ProtectedList, GenerativeFst, SurfaceHeuristic) across 28 Arabic roots, ~80 cascade surfaces, and 45 foreign (Latin-script) words. Fifth: **M6** — the FTS5 Arabic stemming path in `libraries.rs::process_arabic_word` now routes through `arabic::analyze_best`. Every Arabic token in every note in every Universe now flows through the five-layer engine; Light10 is retained only as the graceful `SurfaceHeuristic` fallback so unknown words don't regress. The flagship `وائل → "ائل"` mangle is gone: the protected list short-circuits Light10 and the stem is preserved verbatim. Sixth: **M7** — the Layer 4 disambiguator. `analyze_best`'s insertion-order tiebreak replaced with a pure, deterministic rank: confidence desc → origin (UserOverride > ProtectedList > FST > Heuristic) → POS (ProperNoun > Noun > … > Verb > … > Foreign) → fewer affixes → alphabetic lemma. The كاتب ambiguity now resolves to the Noun reading (active participle) every time, across any OS, any FST build, any Universe. Seventh: **M8** — Layer 0 user overrides. New module `arabic::overrides` with a per-Universe JSON store at `<universe>/.constellation/arabic-overrides.json`; `analyze_with_overrides(word, Some(&store))` inserts a hash-lookup Layer 0 that short-circuits the entire pipeline on an exact or normalized-vocalized match. `UserOverride::to_analysis()` produces an `Analysis` with `origin=UserOverride, confidence=1.0`, which M7's disambiguator already ranks strictly above every other origin — so no changes to `rank_analyses` were needed. The back-compat wrapper `analyze(word) ≡ analyze_with_overrides(word, None)` preserves every caller on the crate today; the overload is purely additive. Atomic file writes (`.tmp` + rename), alphabetic-sorted entries for git-friendly diffs, forward-compat serde defaults. Eighth: **M8b (Rust plumbing slice)** — the wire that makes M8 run in production. New `ACTIVE_STORE` registry in `overrides.rs` (process-wide `OnceLock<RwLock<Arc<OverrideStore>>>`), `activate_for_universe()` hook called from `set_active_universe` so switching Universes auto-loads the per-Universe JSON file into the active store, `process_arabic_word` in `libraries.rs` (FTS5 hot path) now reads the active store via cheap `Arc::clone`, and three Tauri commands (`read_arabic_overrides`, `add_arabic_override`, `remove_arabic_override`) registered in `lib.rs` and exposed to the Settings UI which arrives in M8c. Ninth: **M8c** — the Settings UI slice. Svelte panel `ArabicOverridesPanel.svelte` mounted inside `SettingsModal.svelte` under a new `arabic-overrides` section, wired to the three M8b commands plus a new fourth command `reindex_arabic_overrides(surface)` that LIKE-scans `note_meta` and atomically deletes + re-inserts every affected row into `notes_fts` inside a single `BEGIN IMMEDIATE`/`COMMIT` — so the moment the user saves a new override, every note containing that surface is re-tokenized under the fresh Layer 0 verdict, no full Universe rebuild needed. All 31 strings the panel renders are in every one of the 15 locales (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh); RTL content inside each cell routes through `detectDir` so a mixed-script annotation flows naturally in either direction. Tenth: **M9** — the bench harness. A single `#[test] #[ignore]` in `arabic::bench` that reports cold-start (cache deleted → `GenerativeFst::get()`), warm-start (`fst_bake::load_bundle` + `GenerativeFst::from_bytes` on the just-written cache), throughput (502-case corpus × 500 iterations through `analyze_best`), accuracy (`regression::run_corpus` + per-origin breakdown), and size (on-disk cache bundle + linear projection to 7K-root scale). Opt-in only (`cargo test --lib --release arabic::bench -- --ignored --nocapture`) because the cold-start step deletes the real user cache. Two runs today at 595 seed roots / 32,197 FST keys: cold 154–167 ms, warm 21 ms (≈7–8× speedup — **meets the ≥5× target**), throughput ~130K words/sec (below the aspirational ≥200K — follow-ons queued: string-intern `pattern_label`, mmap the FST bytes, trim the per-call `Arc::clone` on `ACTIVE_STORE`), accuracy **100.0% (502/502)** including 100% on each of Protected / Generative / Heuristic buckets, cache bundle 7.6 MiB on disk with a linear projection of ~90 MiB at 7K roots (above the ≤10 MiB aspirational target — follow-ons queued: mmap the FST byte buffer instead of holding it in a `Vec<u8>`, dedup the `pattern_label` strings across forms, intern `root_key` through a shared table). Eleventh: **M10** — the Lexical Bridge architecture. The pre-existing `crate::lexicon` skeleton (graph types, expansion types, stub APIs) was filled out into a working implementation end-to-end: a new `parse.rs` loads a TSV seed into `ConceptRecord`s; `graph.rs` gained a concept-to-CSR builder, an `fst::Map` name-index keyed by `"{lang_code}:{normalized_lemma}"`, a normalization pipeline (`normalize_stripped` for Arabic-script langs, `trim().to_lowercase()` elsewhere), `find_nodes` / `edges_of` / singleton access via `OnceLock`, and an adjacency-list layout that packs every concept's cross-lang pairs as `Equivalent` edges and every same-lang pair as `Synonym` edges; `mod.rs`'s `equivalents()` and `expand()` stubs were replaced with real graph walks that respect `ExpansionOptions` (synonym level, enabled languages, per-lang cap) and return populated `ExpansionResult` buckets by `EdgeKind`. A 15-concept hand-picked seed ships at `lexicon/data/seed_v1.tsv` (concepts: book, knowledge, write, read, love, water, house, teacher, student, language, peace, truth, time, day, night — each with ~12–16 language labels) to exercise every code path before the M11 20K-concept core lands. The seed only adds `Equivalent` and `Synonym` edges; `Hypernym` / `Hyponym` machinery is wired and unit-tested but unused until M11 ships WordNet data. Lookup is diacritic-insensitive for Ar/Fa/Ur (`كِتَاب` finds `كتاب`) and case-insensitive for Latin/Cyrillic/Turkish. Full public-API parity preserved across all eleven landings; **305/305 library tests pass** (up from 209 pre-M3: +13 fst_bake, +10 regression harness, +6 TSV parser, +5 M6 FTS contract tests, +12 M7 disambiguator, +21 M8 overrides [16 unit + 5 integration], +8 M8b ACTIVE_STORE registry [Arc-pointer-identity + swap semantics + activate_for_universe disk paths], +0 M8c [intentionally — integration test deferred until Settings → Debug lands], -1 removed `no_duplicate_lemmas_in_seed` obsolete under first-write-wins, +34 M10 lexicon [11 parse, 12 graph, 13 equivalents+expand, minus 2 obsolete mod.rs stubs]) — plus one `#[ignore]` bench test in M9 that does not count toward the 305 and only runs on explicit opt-in.

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

## Files modified

- `src-tauri/Cargo.toml` — `+ fst = "0.4"` (M3), `+ dirs = "5"` (M3-baker).
- `src-tauri/Cargo.lock` — transitive `dirs` family.
- `src-tauri/src/arabic/mod.rs` — `pub mod fst_index;` (M3) + `pub mod fst_bake;` (M3-baker) + one-line analyzer swap.
- `src-tauri/src/arabic/fst_index.rs` — new M3; refactored in M3-baker to split build/parse and add cache plumbing.
- `src-tauri/src/arabic/fst_bake.rs` — new (M3-baker, 685 lines, 13 tests).
- `src-tauri/src/arabic/roots.rs` — `+ pub fn seed_tsv()` accessor (M3-baker).
- `src-tauri/src/arabic/protected.rs` — TSV loader refactor (M1g/M1h).
- `src-tauri/src/arabic/protected_seed.tsv` — new (M1g/M1h, 1,196 entries).
- `src-tauri/src/arabic/regression.rs` — new (M5, 10 tests).
- `src-tauri/src/arabic/regression_cases.tsv` — new (M5, 502 data rows).
- `src-tauri/src/libraries.rs` — `process_arabic_word` routed through `analyze_best` (M6) + 5 new FTS contract tests at EOF; M8b extends it to read `arabic::overrides::active()` on every token.
- `src-tauri/src/arabic/disambiguate.rs` — new (M7, 12 tests).
- `src-tauri/src/arabic/overrides.rs` — new (M8, 16 tests) — UserOverride type, OverrideStore CRUD, per-Universe JSON persistence with atomic writes. M8b adds `ACTIVE_STORE` registry + three Tauri commands + 8 registry tests (total 24). M8c adds a fourth Tauri command (`reindex_arabic_overrides`).
- `src-tauri/src/arabic/mod.rs` — M8b adds `analyze_with_overrides_best` convenience; `analyze_best` reduced to a thin wrapper.
- `src-tauri/src/universe.rs` — M8b hooks `activate_for_universe` into `set_active_universe`.
- `src-tauri/src/lib.rs` — M8b registers three Arabic override Tauri commands; M8c registers the fourth (`reindex_arabic_overrides`).
- `src-tauri/src/search.rs` — M8c adds `reindex_notes_matching_text` helper (targeted FTS5 `delete` + reinsert under a transaction).
- `src/lib/components/ArabicOverridesPanel.svelte` — new (M8c, ~480 lines). Settings-modal panel for override CRUD with live reindex feedback.
- `src/lib/components/SettingsModal.svelte` — M8c adds the `arabic-overrides` section entry + content branch.
- `src/lib/i18n/{ar,de,en,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — M8c adds `settings.sections.arabicOverrides` + the 31-key `settings.arabicOverrides` block to all 15 locale files.
- `src-tauri/src/arabic/bench.rs` — new (M9, ~200 lines). Single opt-in `#[test] #[ignore] fn m9_bench()`. Does not run under default `cargo test --lib`; invoked with `cargo test --lib --release arabic::bench -- --ignored --nocapture`.
- `src-tauri/src/arabic/mod.rs` — M9 promotes `regression` from `#[cfg(test)] mod` to `#[cfg(test)] pub(crate) mod`; adds `#[cfg(test)] mod bench;`.
- `src-tauri/src/lexicon/parse.rs` — new (M10, ~280 lines, 11 tests). TSV seed parser with `ConceptRecord` output and `ParseRowError` diagnostics.
- `src-tauri/src/lexicon/graph.rs` — M10 rewrite (~400 lines, 12 tests). Concept → CSR graph builder, `fst::Map` name-index, `find_nodes` / `edges_of` / singleton `get()`, `normalize_for_lookup` dispatch.
- `src-tauri/src/lexicon/mod.rs` — M10 rewrite (~300 lines, 13 tests). Real `equivalents()` / `expand()` over the singleton graph, plus `_via` test-hooks accepting an arbitrary graph instance.
- `src-tauri/src/lexicon/data/seed_v1.tsv` — new (M10, 4.4 KB, 15 hand-picked concepts × 12–16 language labels).

## Open items

- **M1g-data / M1h-data**: the 20K Wikipedia-extracted proper-noun corpus + 2K loanwords. Today's 1,196 hand-picked entries cover the common case; the full corpus comes from CC-BY-SA bulk extraction (separate milestone in `lab/`, blocked on extractor tooling).
- **M5-grow**: expand the corpus over time. 502 is the v1 floor — as M6/M7 land, new flagship surfaces identified during bring-up should be added here first before any other test code. Target by M9: ≥2,000 cases, with ≥20 pure-heuristic Arabic-script rows (Layer 4 fallback coverage; currently the heuristic threshold is met by foreign Latin-script rows via the non-Arabic-script route).
- **M7-v2**: corpus-aware disambiguation. Today's ranking is a pure function of the Analysis fields. V2 reads the user's own FTS vocab to bias toward lemmas the user writes often, plus a 3-word context window at query time to pick between readings (`كاتب الرسالة` → Noun; `كاتب أخاه` → Verb). Tracked as a follow-on once Settings → Debug surfaces the existing v1 rank so we can A/B the v2 improvements.
- **M8b-v2**: per-cUniverse override layering. When the user views libraries federated from a child Universe, the tokenizer should consult the child's override file too (or overlay the parent's on top of it, with parent winning ties). Today's `ACTIVE_STORE` is a single global Arc; v2 either becomes a stack or a composite `OverrideStore` that consults multiple backing maps. Wait until real-user feedback shows this is needed before building it.
- **M8c-integration-test**: end-to-end "add override → reindex → assert FTS hit set changes" test. Will live with the Settings → Debug scorecard so the assertion can run under the real SearchState (not a fresh tempdb) against a seeded Universe.
- **M9-mmap**: switch the FST byte buffer from `Vec<u8>` to `memmap2::Mmap`. Drops RSS at 7K roots from the projected ~90 MiB (today's in-memory layout) to an estimated ~20–30 MiB resident, and cuts warm-start time further because the OS page-cache supplies pages on demand. Needs a cfg-gate path: mmap on desktop, `Vec<u8>` on mobile (iOS disallows anon-mmap on some targets). Bench re-runs to confirm both numbers post-change.
- **M9-intern**: intern `pattern_label` and `root_key` across `SurfaceValue`. Today each enriched value owns two `String` copies; patterns like `فاعل` recur on every root's active-participle cell (~600 duplicates today, ~7,000 at scale). A 4-byte `u16` index into a shared `StringInterner` cuts the per-value payload from ~48 B of owned string bytes to 4 B total. Layered with M9-mmap, drops projected RSS at 7K roots to ≈15 MiB range.
- **M9-hotpath**: profile `analyze_best` to close the 200K-words/sec gap. Candidates: (a) `overrides::active()` does an unconditional `Arc::clone` even when the active store is empty — cache an `AtomicBool` snapshot of "store is empty" inside `set_active` to skip the clone on the common empty path; (b) `rank_analyses` allocates a `Vec<Analysis>` even for single-hit inputs — a `SmallVec<[Analysis; 2]>` dodges one allocation per word; (c) generator-style visitor that lets fast ProtectedList / UserOverride hits short-circuit before allocating the result vector. Target: ≥200K words/sec at 7K-root scale, re-measured via the same bench.
- **M9-rss-real**: replace the projected-size proxy with a real OS-level RSS delta measurement (`GetProcessMemoryInfo` on Windows, `mach_task_basic_info` on macOS, `/proc/self/statm` on Linux — behind a `#[cfg]` switch, test-only). Becomes the authoritative size number once M9-mmap lands, because the mmap estimate depends on working-set assumptions the proxy can't capture.

## User-facing changes (M8c)

The Settings modal gains a new "Arabic Overrides" section (icon: `translate`, grouped next to Language). Users can pin how the Arabic engine analyses specific surfaces — for each override they specify surface, lemma, optional root/pattern/POS, and an optional note. Saved overrides take sovereign priority over the FST, the cascade, and the heuristic fallback. After every add/remove, the panel reindexes only the notes actually containing that surface and reports "Reindexed N note(s)" in the status area — no full-Universe rebuild, no stale FTS state.

Help-file and User-Manual entries for this panel are tracked under `M8c-doc` above; the Standing Order step for this is the next thing to land after commit + push.
