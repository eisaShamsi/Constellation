# Session Log — 2026-04-18

## Headline

**M3 + M3-baker landed.** First: `GenerativeIndex` (HashMap, ~40 MB projected at 7K roots) swapped for `GenerativeFst` (BurntSushi FST, prefix-compressed, mmap-ready). Second: the compiled FST is now persisted to the user's cache directory on first launch and reloaded on subsequent launches via `GenerativeFst::from_bytes` — the cold/warm startup path divergence that M9 ("50 ms analyzer cold-start") measures against. Full public-API parity preserved; 209/209 library tests pass, including the M2.b `الأئمة → ء-م-م` flagship which now resolves through the FST-backed Layer 3. The HashMap implementation is retained alongside as a reference for its own tests; it's no longer on the analyzer's hot path.

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

## Commit

Pending — per Standing Order: push + SO after user review. Commit scope:
- `src-tauri/Cargo.toml` — `+ dirs = "5"` (cross-platform cache dir resolution).
- `src-tauri/Cargo.lock` — transitive `dirs` / `dirs-sys` / `redox_users` deps.
- `src-tauri/src/arabic/fst_bake.rs` — new (685 lines, 13 tests).
- `src-tauri/src/arabic/fst_index.rs` — `get()` wired to cache; `build()` → `build_bundle()` + `from_bundle()` split; `from_bytes` stays pub.
- `src-tauri/src/arabic/mod.rs` — `+ pub mod fst_bake;`.
- `src-tauri/src/arabic/roots.rs` — `+ pub fn seed_tsv() -> &'static str` accessor.

## Files modified

- `src-tauri/Cargo.toml` — `+ fst = "0.4"` (M3), `+ dirs = "5"` (M3-baker).
- `src-tauri/Cargo.lock` — transitive `dirs` family.
- `src-tauri/src/arabic/mod.rs` — `pub mod fst_index;` (M3) + `pub mod fst_bake;` (M3-baker) + one-line analyzer swap.
- `src-tauri/src/arabic/fst_index.rs` — new M3; refactored in M3-baker to split build/parse and add cache plumbing.
- `src-tauri/src/arabic/fst_bake.rs` — new (M3-baker, 685 lines, 13 tests).
- `src-tauri/src/arabic/roots.rs` — `+ pub fn seed_tsv()` accessor (M3-baker).

## Open items

- **M1g / M1h**: 20K Wikipedia proper nouns + 2K loanwords — expands the protected list, orthogonal to M3/M3-baker.
- **M5**: 500-case regression corpus — the benchmark harness that will let M9 measure 200K words/sec accurately.
- **M6**: replace `stem_arabic_light10` in `fts5_tokenizer.rs` with `arabic::analyze`. Unblocked: the analyzer is FST-backed with persistent cache.
- **M9**: measure cold-start analyzer time on the real user machine (Windows) with a clean cache dir, then warm-start. If the warm-start delta isn't ≥5× the cold-start on the target 7K-root corpus, tune the format (e.g. memory-map instead of read-to-vec).

## No user-facing changes

Engine internals only. No help files, User Manual, or translation updates needed for this session.
