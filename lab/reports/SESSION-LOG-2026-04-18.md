# Session Log — 2026-04-18

## Headline

**M3 landed: `GenerativeIndex` (HashMap, ~40 MB projected at 7K roots) swapped for `GenerativeFst` (BurntSushi FST, prefix-compressed, mmap-ready).** Full public-API parity, zero analyzer regressions: 196/196 library tests pass, including the M2.b `الأئمة → ء-م-م` flagship which now resolves through the FST-backed Layer 3. The HashMap implementation is retained alongside as a reference for its own tests; it's no longer on the analyzer's hot path.

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

| Suite | Before M3 | After M3 | Delta |
|---|---|---|---|
| arabic module | 171 | 183 | +12 (new fst_index tests) |
| library total  | 184 | 196 | +12 |

- All 196 pass. Test wall time: 0.50s (no FST-build regression against the HashMap baseline on today's seed).
- Zero edits required to any mod.rs integration test — the `الأئمة → ء-م-م` cascade still succeeds, now via the FST.

## Commit

Pending — per Standing Order: push + SO after user review.

## Files modified

- `src-tauri/Cargo.toml` — `+ fst = "0.4"`.
- `src-tauri/src/arabic/mod.rs` — module registration + one-line analyzer swap + doc comment updates.
- `src-tauri/src/arabic/fst_index.rs` — new (369 lines, 12 tests).

## Open items

- **M3-baker**: persist compiled FST bytes to a file on first run of a Universe; mmap on subsequent runs via `GenerativeFst::from_bytes`. This is what cashes out the "50 ms startup" budget in M9. The `from_bytes` entry point and its roundtrip test are already in place.
- **M1g / M1h**: 20K Wikipedia proper nouns + 2K loanwords — expands the protected list, unrelated to M3.
- **M5**: 500-case regression corpus — the benchmark harness that will let M9 measure 200K words/sec accurately.
- **M6**: replace `stem_arabic_light10` in `fts5_tokenizer.rs` with `arabic::analyze`. Now unblocked: the analyzer is FST-backed with a clean public surface.

## No user-facing changes

Engine internals only. No help files, User Manual, or translation updates needed for this session.
