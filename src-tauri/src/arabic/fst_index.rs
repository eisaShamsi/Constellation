//! M3 — FST-backed generative index.
//!
//! This module is the compact, read-only successor to
//! [`generator::GenerativeIndex`]. Same public surface, same semantics; the
//! underlying storage switches from two `HashMap<String, Vec<GeneratedForm>>`
//! buckets to two [`fst::Map`]s over the same sorted key sets. The forms
//! themselves live in two flat [`Vec<GeneratedForm>`] side-tables; the
//! `u64` value on each FST edge packs `(offset:u32, count:u32)` into that
//! table.
//!
//! ## Why FSTs
//!
//! On the target 7K-root × 158-pattern corpus we will produce ~1.1M forms
//! and ~300K distinct stripped keys. A `HashMap<String, _>` of that size
//! spends ~40 MB on string allocations alone (each key is its own heap
//! `String`) — unacceptable for an embedded PKM whose whole RSS budget is
//! 350 MB.
//!
//! A BurntSushi [`fst::Map`] over the same key set compresses to single
//! digits of megabytes because it *shares common prefixes* — which is
//! exactly what Arabic morphology produces (every Form I active participle
//! starts with a fixed prefix, etc.). The FST is also **read-only** and
//! **mmap-able**, so the next step (baking the generated index to a file
//! on first run and mapping it in on subsequent runs) gives us effectively
//! zero-cost startup.
//!
//! ## Public API parity with `GenerativeIndex`
//!
//! The analyzer calls `GenerativeFst::get()`, `lookup(&str)`,
//! `lookup_folded(&str)`, `len()`, `is_empty()` — matching the old
//! `GenerativeIndex` 1:1. Swapping the backing store is a single-line
//! change in `arabic::mod::analyze()`.
//!
//! The HashMap-backed [`generator::GenerativeIndex`] is kept alongside as
//! a reference implementation — its tests assert corpus-level properties
//! (e.g. "أئمة is findable") that we want to keep validated through the
//! transition. Once the FST path has been battle-tested we can mark the
//! HashMap version `#[cfg(test)]`-only or remove it outright.
//!
//! ## Payload packing
//!
//! Each FST value is a `u64` formed as `(offset as u64) << 32 | count as
//! u64`. With `u32` offset/count we can address ~4 billion forms in each
//! side-table — comfortable headroom even at the full 7K-root corpus.
//! We assert during build that neither field overflows `u32::MAX`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use fst::{Map, MapBuilder};
// `Streamer` is only touched by the `iter_stripped` diagnostic (test-only),
// so keep it behind `cfg(test)` to avoid a "unused import" warning in
// production builds.
#[cfg(test)]
use fst::Streamer;

use super::fst_bake::{self, FstBundle};
use super::generator::{generate_all, GeneratedForm};
use super::normalizer::{normalize_folded, normalize_stripped};

/// Read-only, FST-backed generative index. Two FST maps (stripped + folded)
/// over the same corpus; each maps a normalized surface key to a packed
/// `u64` pointing into a parallel [`Vec<GeneratedForm>`] side-table.
///
/// See the module-level docs for the rationale behind the FST representation.
pub struct GenerativeFst {
    /// FST over stripped (tashkeel/tatweel removed) surface keys.
    fst_stripped: Map<Vec<u8>>,
    /// FST over folded (aggressive normalization) surface keys. Only
    /// populated when the folded key differs from the stripped one —
    /// saves roughly a third of the entries on real corpora.
    fst_folded: Map<Vec<u8>>,
    /// Flat values array for `fst_stripped`. An FST entry `(offset, count)`
    /// slices into `&values_stripped[offset..offset + count]`.
    values_stripped: Vec<GeneratedForm>,
    /// Flat values array for `fst_folded`. Separate from `values_stripped`
    /// because the two FSTs may point at overlapping-but-not-identical
    /// form sets (the folded side strictly subsets the stripped one).
    values_folded: Vec<GeneratedForm>,
}

impl GenerativeFst {
    /// Access the lazily-initialised singleton.
    ///
    /// Three-stage init, in preference order:
    ///
    ///   1. **Cache hit** — `fst_bake::try_load_cached()` reads a bundle
    ///      from the user's cache directory (content-addressed by a hash
    ///      of `roots_seed.tsv` + `CACHE_FORMAT_VERSION`). If the file is
    ///      present, well-formed, and parses back into a live `Map`, we
    ///      use it and skip generation entirely. This is the warm path
    ///      at steady state — expected cost is a file read and two
    ///      `fst::Map::new()` calls (which just validate the header).
    ///   2. **Cache miss / corrupt cache** — we fall through to an
    ///      in-memory rebuild via [`Self::build_bundle`], then persist
    ///      the result best-effort so the *next* launch hits the cache.
    ///      A failed persist is silent: read-only cache dirs must never
    ///      gate the analyzer from coming up.
    ///   3. **Final reconstruction** — the freshly-built bundle always
    ///      parses (we wrote those bytes ourselves seconds ago), so the
    ///      `expect` is unreachable in practice; we keep it as a
    ///      loud-failure tripwire in case a future refactor breaks the
    ///      build/parse invariant.
    ///
    /// Subsequent calls to `get()` after the first are free — the
    /// [`OnceLock`] short-circuits.
    pub fn get() -> &'static GenerativeFst {
        static INDEX: OnceLock<GenerativeFst> = OnceLock::new();
        INDEX.get_or_init(|| {
            // Stage 1: try the on-disk cache. Any failure (missing file,
            // hash mismatch, truncation, decode error) returns None and
            // we fall through — the cache layer never panics.
            if let Some(bundle) = fst_bake::try_load_cached() {
                if let Ok(fst) = Self::from_bundle(bundle) {
                    return fst;
                }
                // FST-byte parse failure from a successfully-decoded
                // bundle would indicate a mismatch between what
                // `fst_bake` wrote and what `fst::Map` can read — treat
                // it as a corrupt cache and rebuild. No panic.
            }

            // Stage 2: cold start. Build the bundle in memory, persist it
            // for next launch (best-effort), then hand the same bytes to
            // `from_bundle` so stages 1 and 2 go through the exact same
            // reconstruction path — no cold/warm behavioural divergence.
            let bundle = Self::build_bundle();
            fst_bake::persist_best_effort(&bundle);

            Self::from_bundle(bundle).expect(
                "freshly-built FST bundle must parse back into a live Map — \
                 if this trips, the build/parse invariant in build_bundle \
                 has been broken",
            )
        })
    }

    /// Build a serialisable bundle (raw FST bytes + both side-tables)
    /// from `generator::generate_all()`. Extracted from the old `build`
    /// so the bake layer can consume the same intermediate representation
    /// it will persist — no duplicate bucketing code between the cold
    /// and warm paths.
    ///
    /// Steps:
    ///   1. Walk every generated form, bucket into `BTreeMap` under
    ///      stripped + folded keys. `BTreeMap` is deliberate — FST
    ///      builder requires strictly sorted insertions, and
    ///      `BTreeMap::into_iter` yields keys in UTF-8 byte order
    ///      (which is also Unicode code point order for the Arabic we
    ///      handle).
    ///   2. Per-bucket dedup on `(root_key, pattern_label)` — matches
    ///      the HashMap implementation's semantics so parity tests stay
    ///      valid across backends.
    ///   3. Flatten each bucket into a side-table; pack the FST value as
    ///      `(offset << 32) | count`.
    ///
    /// Returned `FstBundle` is owned — the caller can persist, parse, or
    /// both.
    fn build_bundle() -> FstBundle {
        let mut buckets_stripped: BTreeMap<String, Vec<GeneratedForm>> = BTreeMap::new();
        let mut buckets_folded: BTreeMap<String, Vec<GeneratedForm>> = BTreeMap::new();

        for form in generate_all() {
            let stripped = normalize_stripped(&form.surface);
            let folded = normalize_folded(&form.surface);
            buckets_stripped
                .entry(stripped.clone())
                .or_default()
                .push(form.clone());
            // Only record in the folded bucket when folding actually
            // changes the key. Avoids doubling memory on keys where
            // stripped == folded (the common case for names written
            // without hamza ambiguity).
            if folded != stripped {
                buckets_folded.entry(folded).or_default().push(form);
            }
        }

        dedup_buckets(&mut buckets_stripped);
        dedup_buckets(&mut buckets_folded);

        let (stripped_bytes, values_stripped) = build_map_bytes(buckets_stripped);
        let (folded_bytes, values_folded) = build_map_bytes(buckets_folded);

        FstBundle {
            stripped_bytes,
            values_stripped,
            folded_bytes,
            values_folded,
        }
    }

    /// Reconstruct a live `GenerativeFst` from an owned [`FstBundle`].
    /// Single entry point for both cache-hit and fresh-build paths, so
    /// any bug in FST-byte handling surfaces identically in either.
    fn from_bundle(bundle: FstBundle) -> Result<Self, fst::Error> {
        Self::from_bytes(
            bundle.stripped_bytes,
            bundle.values_stripped,
            bundle.folded_bytes,
            bundle.values_folded,
        )
    }

    /// Construct from pre-built FST bytes + matching side-tables. This
    /// is the raw mmap / bake entry point — `from_bundle` wraps it in
    /// the caller-friendly `FstBundle` shape.
    ///
    /// Retained as `pub` for external tools (regression corpus dumper,
    /// future mmap loader) that want to hand in FST bytes without going
    /// through `FstBundle`.
    pub fn from_bytes(
        stripped_bytes: Vec<u8>,
        values_stripped: Vec<GeneratedForm>,
        folded_bytes: Vec<u8>,
        values_folded: Vec<GeneratedForm>,
    ) -> Result<Self, fst::Error> {
        let fst_stripped = Map::new(stripped_bytes)?;
        let fst_folded = Map::new(folded_bytes)?;
        Ok(GenerativeFst {
            fst_stripped,
            fst_folded,
            values_stripped,
            values_folded,
        })
    }

    /// Lookup by stripped surface (tashkeel/tatweel removed). Empty slice
    /// means "no hit" — callers can fall through to [`lookup_folded`].
    pub fn lookup(&self, stripped: &str) -> &[GeneratedForm] {
        match self.fst_stripped.get(stripped.as_bytes()) {
            Some(packed) => slice_from_packed(&self.values_stripped, packed),
            None => &[],
        }
    }

    /// Lookup by folded surface (aggressive normalization). Should only
    /// fire after [`lookup`] returned empty; results carry a slightly
    /// lower confidence downstream to reflect the fuzzier match.
    pub fn lookup_folded(&self, folded: &str) -> &[GeneratedForm] {
        match self.fst_folded.get(folded.as_bytes()) {
            Some(packed) => slice_from_packed(&self.values_folded, packed),
            None => &[],
        }
    }

    /// Number of distinct stripped-surface keys. Useful for sanity
    /// checks and benchmarks; matches `GenerativeIndex::len`.
    pub fn len(&self) -> usize {
        self.fst_stripped.len()
    }

    /// `true` when no keys are indexed. Should never return `true` in
    /// production (seed corpus always yields >0 forms); exposed for
    /// clippy / test clarity — and matches `GenerativeIndex::is_empty`.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.fst_stripped.is_empty()
    }

    /// Diagnostic: iterate every (key, form-slice) pair. Not used on any
    /// hot path — reserved for the M3 baker and regression-corpus tooling.
    /// Allocates a `Vec<u8>` per key (FST streams return refs that can't
    /// outlive the stream), so stays test-only.
    #[cfg(test)]
    pub fn iter_stripped(&self) -> Vec<(String, &[GeneratedForm])> {
        let mut out = Vec::with_capacity(self.fst_stripped.len());
        let mut stream = self.fst_stripped.stream();
        while let Some((key_bytes, packed)) = stream.next() {
            let key = String::from_utf8_lossy(key_bytes).into_owned();
            out.push((key, slice_from_packed(&self.values_stripped, packed)));
        }
        out
    }
}

// ──────────────────────────────────────────────────────────────────────
// Build helpers
// ──────────────────────────────────────────────────────────────────────

/// Collapse each bucket down to `(root_key, pattern_label)`-unique forms.
/// A single surface can legitimately arise from multiple (root, pattern)
/// pairs (homonyms), but not from the same pair twice — those are always
/// artefacts of pattern template overlap.
fn dedup_buckets(buckets: &mut BTreeMap<String, Vec<GeneratedForm>>) {
    for v in buckets.values_mut() {
        v.sort_by(|a, b| {
            a.root_key
                .cmp(&b.root_key)
                .then_with(|| a.pattern_label.cmp(&b.pattern_label))
        });
        v.dedup_by(|a, b| a.root_key == b.root_key && a.pattern_label == b.pattern_label);
    }
}

/// Consume a sorted-keys bucket map, emit `(fst_bytes, values)` where
/// each key's FST value is `(offset << 32) | count` into `values`.
///
/// `BTreeMap::into_iter` yields keys in byte-lex order, which is exactly
/// the order `MapBuilder::insert` requires. This function panics on
/// builder errors because:
///   - Inserting out-of-order is a logic bug (we just iterated a BTreeMap).
///   - An overflow of `u32::MAX` offsets/counts would mean our corpus
///     grew past 4 billion forms — ten thousand times today's size. Fail
///     loud at build time rather than silently truncate.
///
/// Returning raw bytes (rather than `Map<Vec<u8>>`) lets `build_bundle`
/// hand the same buffer to two consumers — the on-disk baker and the
/// in-memory `fst::Map::new` — without an extra clone.
fn build_map_bytes(
    buckets: BTreeMap<String, Vec<GeneratedForm>>,
) -> (Vec<u8>, Vec<GeneratedForm>) {
    let total_forms: usize = buckets.values().map(|v| v.len()).sum();
    let mut values: Vec<GeneratedForm> = Vec::with_capacity(total_forms);
    let mut builder = MapBuilder::memory();

    for (key, forms) in buckets {
        let offset = values.len();
        let count = forms.len();
        assert!(
            offset <= u32::MAX as usize,
            "GenerativeFst value offset overflowed u32 at key={key:?} (corpus too large)"
        );
        assert!(
            count <= u32::MAX as usize,
            "GenerativeFst per-key form count overflowed u32 at key={key:?}"
        );
        let packed = ((offset as u64) << 32) | (count as u64);
        builder
            .insert(key.as_bytes(), packed)
            .expect("FST insertion order violated (BTreeMap promised sorted keys)");
        values.extend(forms);
    }

    let bytes = builder
        .into_inner()
        .expect("FST builder finalize failed (unreachable on in-memory writer)");
    (bytes, values)
}

/// Decode a packed `(offset, count)` `u64` and slice the side-table.
///
/// Inlined as a free function so both `lookup` and `lookup_folded` can
/// share it without borrowing issues (each takes a different `&[GeneratedForm]`).
#[inline]
fn slice_from_packed(values: &[GeneratedForm], packed: u64) -> &[GeneratedForm] {
    let offset = (packed >> 32) as usize;
    let count = (packed & 0xFFFF_FFFF) as usize;
    debug_assert!(
        offset + count <= values.len(),
        "packed slice ({offset}..{}) exceeds side-table len {}",
        offset + count,
        values.len()
    );
    &values[offset..offset + count]
}

// ──────────────────────────────────────────────────────────────────────
// Tests — parity with GenerativeIndex
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arabic::types::PatternKind;

    #[test]
    fn fst_builds_nonempty() {
        let idx = GenerativeFst::get();
        assert!(!idx.is_empty(), "GenerativeFst must be non-empty after build");
        assert!(
            idx.len() > 0,
            "expected >0 distinct stripped keys from the seed corpus"
        );
    }

    #[test]
    fn fst_finds_kaatib() {
        // Active participle of ك-ت-ب via فَاعِل.
        let idx = GenerativeFst::get();
        let hits = idx.lookup("كاتب");
        assert!(!hits.is_empty(), "كاتب must be findable in the FST");
        assert!(
            hits.iter().any(|f| &*f.root_key == "ك-ت-ب"),
            "expected root ك-ت-ب among hits for كاتب; got {:?}",
            hits.iter().map(|f| &f.root_key).collect::<Vec<_>>()
        );
    }

    #[test]
    fn fst_finds_maktub() {
        // Passive participle of ك-ت-ب via مَفْعُول.
        let idx = GenerativeFst::get();
        let hits = idx.lookup("مكتوب");
        assert!(!hits.is_empty(), "مكتوب must be findable in the FST");
        assert!(
            hits.iter().any(|f| &*f.root_key == "ك-ت-ب"),
            "expected root ك-ت-ب among hits for مكتوب"
        );
    }

    #[test]
    fn fst_finds_dahraj() {
        // Quadriliteral Form I perfect of د-ح-ر-ج.
        let idx = GenerativeFst::get();
        let hits = idx.lookup("دحرج");
        assert!(!hits.is_empty(), "دحرج must be findable in the FST");
        assert!(
            hits.iter().any(|f| &*f.root_key == "د-ح-ر-ج"),
            "expected root د-ح-ر-ج among hits for دحرج"
        );
    }

    #[test]
    fn fst_finds_aimma() {
        // Broken plural أئمة of ء-م-م — the flagship M2.b case.
        // Must remain findable after the HashMap → FST swap, or the
        // الأئمة end-to-end test in mod.rs regresses.
        let idx = GenerativeFst::get();
        let hits = idx.lookup("أئمة");
        assert!(
            !hits.is_empty(),
            "أئمة must be findable in the FST (flagship M2.b regression)"
        );
        assert!(
            hits.iter().any(|f| &*f.root_key == "ء-م-م"),
            "expected root ء-م-م among hits for أئمة; got {:?}",
            hits.iter()
                .map(|f| (&f.root_key, &f.pattern_label))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn fst_finds_qala() {
        // Hollow perfect قال from ق-و-ل via M2.c `<fatha><weak><vowel>` → `<fatha><ا>`.
        let idx = GenerativeFst::get();
        let hits = idx.lookup("قال");
        assert!(
            !hits.is_empty(),
            "قال must be findable in the FST (M2.c hollow-perfect regression)"
        );
        assert!(
            hits.iter().any(|f| &*f.root_key == "ق-و-ل"),
            "expected root ق-و-ل among hits for قال"
        );
    }

    #[test]
    fn fst_finds_yaidu() {
        // Assimilated imperfect يعد from و-ع-د via M2.c: drop `<fatha><و><sukun>`
        // after a tense prefix. If any و-initial root + kasra-stem imperfect
        // combination yields يعد we accept it — this test is a sanity check
        // that M2.c assimilated rule survived the FST swap.
        let idx = GenerativeFst::get();
        let hits = idx.lookup("يعد");
        assert!(
            !hits.is_empty(),
            "يعد must be findable (M2.c assimilated-imperfect regression)"
        );
        assert!(
            hits.iter().any(|f| &*f.root_key == "و-ع-د"),
            "expected root و-ع-د among hits for يعد; got {:?}",
            hits.iter().map(|f| &f.root_key).collect::<Vec<_>>()
        );
    }

    #[test]
    fn fst_misses_unknown_word() {
        // A random non-Arabic byte sequence must miss cleanly (empty slice).
        let idx = GenerativeFst::get();
        assert!(
            idx.lookup("zzzzzzzz").is_empty(),
            "non-Arabic input must return empty slice, not panic"
        );
        assert!(
            idx.lookup_folded("zzzzzzzz").is_empty(),
            "non-Arabic folded input must also miss cleanly"
        );
    }

    #[test]
    fn fst_folded_fallback_works() {
        // The folded map is strictly a subset of the stripped map when the
        // two normalizations agree. For a key where they differ, a hit on
        // the folded path must still resolve to a real form slice.
        //
        // We don't hard-code a specific folded key here (corpus-dependent);
        // we instead iterate the whole stripped side to find one whose
        // folded normalization differs, then assert the folded lookup hits.
        let idx = GenerativeFst::get();
        let mut found = false;
        for (key, _) in idx.iter_stripped() {
            let folded = normalize_folded(&key);
            if folded != key && !idx.lookup_folded(&folded).is_empty() {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "expected at least one key whose folded form resolves via lookup_folded"
        );
    }

    #[test]
    fn fst_values_point_to_valid_forms() {
        // Every FST slice must carry at least one form, and each form's
        // surface must normalize back to the FST key. Guards against the
        // packed-offset decoding getting out of sync with the side-table.
        let idx = GenerativeFst::get();
        for (key, forms) in idx.iter_stripped() {
            assert!(
                !forms.is_empty(),
                "empty slice at key {key:?} — packed decode desync?"
            );
            for form in forms {
                let back = normalize_stripped(&form.surface);
                assert_eq!(
                    back, key,
                    "form at key {key:?} re-normalizes to {back:?} — \
                     bucketing or key computation desync"
                );
            }
        }
    }

    #[test]
    fn fst_preserves_pattern_kind() {
        // The packed side-table must carry `pattern_kind` intact so
        // `analyze()` can call `pos_for_kind(form.pattern_kind)` on it.
        let idx = GenerativeFst::get();
        let hits = idx.lookup("كاتب");
        assert!(!hits.is_empty());
        let has_participle_or_verb = hits.iter().any(|f| {
            matches!(
                f.pattern_kind,
                PatternKind::ActiveParticiple | PatternKind::VerbPerfect
            )
        });
        assert!(
            has_participle_or_verb,
            "كاتب should include at least one ActiveParticiple or VerbPerfect hit"
        );
    }

    #[test]
    fn fst_from_bytes_roundtrip() {
        // The `from_bytes` constructor is the future mmap entry point;
        // verify that raw FST bytes + a matching side-table can be
        // reassembled into a working map. Builds the bytes directly via
        // `MapBuilder` so this test stays independent of whatever helper
        // we eventually use to persist the corpus to disk.
        let value = GeneratedForm {
            root_key: "ك-ت-ب".into(),
            pattern_label: "فَعَلَ".into(),
            pattern_kind: PatternKind::VerbPerfect,
            surface: "كَتَبَ".to_string(),
        };

        // One key, one form → offset=0, count=1, packed = 1.
        let mut builder = MapBuilder::memory();
        builder.insert("كتب".as_bytes(), 1_u64).unwrap();
        let stripped_bytes = builder.into_inner().unwrap();

        let empty_folded = MapBuilder::memory().into_inner().unwrap();

        let idx = GenerativeFst::from_bytes(
            stripped_bytes,
            vec![value.clone()],
            empty_folded,
            Vec::new(),
        )
        .expect("from_bytes must accept freshly-built FST bytes");

        let hits = idx.lookup("كتب");
        assert_eq!(hits.len(), 1, "reassembled FST must return the one form");
        assert_eq!(&*hits[0].root_key, "ك-ت-ب");
        assert!(idx.lookup_folded("كتب").is_empty(), "empty folded map must miss");
    }
}
