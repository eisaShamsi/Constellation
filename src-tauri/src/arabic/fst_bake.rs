//! M3-baker — on-disk cache for the compiled Arabic FST.
//!
//! The M3 `GenerativeFst` is a pure function of three inputs:
//!
//!   1. `roots_seed.tsv` (embedded via `include_str!`),
//!   2. the pattern corpus in `patterns.rs`,
//!   3. the generator rules in `generator.rs`.
//!
//! Because all three are frozen at compile time, the FST can be built once
//! and reused forever — until the binary rebuilds with any of them changed.
//! This module persists the compiled bundle to the user's cache directory
//! on first launch and reads it back on subsequent launches, sparing us
//! the generate → normalize → sort → dedup → FST-build pipeline on every
//! start. At the 7K-root target corpus that's ~2–5 seconds saved per
//! launch; at today's 595-root seed it's well under a second, but the
//! infrastructure is worth landing now because:
//!
//!   * it's "Write-Time Derivation" (CLAUDE.md Rule 8) applied to the
//!     analyzer — compute once, persist, read cheap;
//!   * it proves the [`crate::arabic::fst_index::GenerativeFst::from_bytes`]
//!     contract end-to-end;
//!   * it is the exact entry point that M9 ("50 ms cold-start analyzer")
//!     will measure against.
//!
//! ## File layout
//!
//! The cache is a single self-describing binary blob. All multi-byte
//! integers are little-endian. String lengths are `u32`; form counts and
//! FST byte lengths are `u64`. Layout:
//!
//! ```text
//!   magic:                 [u8; 8]  = b"CAEFST01"
//!   version_hash:          u64      (djb2(SEED_TSV) XOR CACHE_FORMAT_VERSION)
//!   stripped_fst_byte_len: u64
//!   stripped_fst_bytes:    [u8]
//!   stripped_value_count:  u64
//!   stripped_values:       [GeneratedForm_encoded × stripped_value_count]
//!   folded_fst_byte_len:   u64
//!   folded_fst_bytes:      [u8]
//!   folded_value_count:    u64
//!   folded_values:         [GeneratedForm_encoded × folded_value_count]
//!
//!   GeneratedForm_encoded:
//!     pattern_kind_tag:    u8       (see `encode_kind` / `decode_kind`)
//!     root_key_len:        u32
//!     root_key_bytes:      [u8]     UTF-8
//!     pattern_label_len:   u32
//!     pattern_label_bytes: [u8]     UTF-8
//!     surface_len:         u32
//!     surface_bytes:       [u8]     UTF-8
//! ```
//!
//! ## Invalidation
//!
//! The cache filename includes the `version_hash`, so when the seed TSV
//! changes the old cache is orphaned (new filename → clean rebuild). When
//! the generator rules or pattern corpus change without touching the TSV,
//! bump [`CACHE_FORMAT_VERSION`] by one. The mismatch between the bytes
//! on disk and the compiled-in const flips the hash and triggers a
//! rebuild — no code in the cache path itself needs to change.
//!
//! ## Failure policy
//!
//! Every disk operation is best-effort: a corrupt file, a read-only cache
//! dir, a concurrent writer — **none** of these should stop the analyzer
//! from coming up. On any persistence error we silently fall back to
//! building in-memory, so the worst case is the old M3 behaviour
//! (HashMap-class startup cost).

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::OnceLock;

use super::generator::GeneratedForm;
use super::types::PatternKind;

/// Bump whenever generator rules, pattern corpus, or the on-disk layout
/// change. Folded into the version hash, so the next launch detects the
/// mismatch and transparently rebuilds. Seed-TSV edits don't need a bump
/// — the TSV content is already part of the hash.
pub const CACHE_FORMAT_VERSION: u32 = 1;

/// Magic bytes at the start of the cache file. Guards against loading an
/// unrelated `arabic-fst-*.bin` from a different application or partial
/// write from a previous process.
const MAGIC: [u8; 8] = *b"CAEFST01";

/// An in-memory bundle of everything needed to reconstruct a
/// [`crate::arabic::fst_index::GenerativeFst`]: the two FST byte-buffers
/// and the two side-tables of [`GeneratedForm`]. Owned on both sides so
/// the cache writer can persist it and the cache reader can hand it
/// straight to `GenerativeFst::from_bytes`.
///
/// `Debug` is derived so the `expect_err(...)` calls in the tests compile —
/// the underlying Vecs format predictably (byte slices + form Debug impls).
#[derive(Debug)]
pub struct FstBundle {
    pub stripped_bytes: Vec<u8>,
    pub values_stripped: Vec<GeneratedForm>,
    pub folded_bytes: Vec<u8>,
    pub values_folded: Vec<GeneratedForm>,
}

// ──────────────────────────────────────────────────────────────────────
// Cache path resolution
// ──────────────────────────────────────────────────────────────────────

/// Resolve the preferred cache path for the compiled FST, or `None` if
/// the platform's cache directory can't be determined (e.g. `$HOME` is
/// unset in a sandboxed test harness).
///
/// Path shape: `<cache_dir>/constellation/arabic-fst-v{hash:016x}.bin`.
/// The hash is baked into the filename so stale caches from previous
/// seed versions coexist peacefully — they're inert, and the user can
/// garbage-collect at leisure.
pub fn cache_file_path() -> Option<PathBuf> {
    let base = dirs::cache_dir()?;
    let hash = version_hash();
    Some(
        base.join("constellation")
            .join(format!("arabic-fst-v{hash:016x}.bin")),
    )
}

/// Content-addressed version identifier. Stable across processes with the
/// same binary, changes on any seed or generator edit (via manual
/// [`CACHE_FORMAT_VERSION`] bump).
///
/// We use a hand-rolled djb2 rather than `std::hash::DefaultHasher` because
/// `DefaultHasher` is explicitly documented as not guaranteed stable
/// across Rust releases — and we *need* stability here, otherwise a
/// compiler upgrade would orphan every user's cache.
pub fn version_hash() -> u64 {
    static HASH: OnceLock<u64> = OnceLock::new();
    *HASH.get_or_init(|| {
        let tsv_hash = djb2(super::roots::seed_tsv().as_bytes());
        tsv_hash ^ (CACHE_FORMAT_VERSION as u64)
    })
}

/// djb2 — chosen for determinism and simplicity. Not cryptographic, not
/// meant to be: we only need collision resistance strong enough that two
/// *intentional* version changes produce two different filenames, which
/// this is more than enough for.
fn djb2(bytes: &[u8]) -> u64 {
    let mut h: u64 = 5381;
    for &b in bytes {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

// ──────────────────────────────────────────────────────────────────────
// Load / persist — public API
// ──────────────────────────────────────────────────────────────────────

/// Try to load a previously-baked bundle from the preferred cache path.
/// Returns `None` on any error (missing file, wrong magic, version-hash
/// mismatch, truncated data, unreadable `PatternKind` tag, I/O failure).
/// Never panics — falling back to an in-memory rebuild is always safe.
pub fn try_load_cached() -> Option<FstBundle> {
    let path = cache_file_path()?;
    load_bundle(&path).ok()
}

/// Persist a bundle to the preferred cache path, best-effort. Any error
/// is swallowed — the caller's startup must never fail because the cache
/// dir is read-only or full.
pub fn persist_best_effort(bundle: &FstBundle) {
    if let Some(path) = cache_file_path() {
        let _ = write_bundle(&path, bundle);
    }
}

/// Read a bundle from an arbitrary path. Exposed for tests so we don't
/// have to race the user's real cache dir.
pub fn load_bundle(path: &std::path::Path) -> io::Result<FstBundle> {
    let mut file = fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    decode_bundle(&buf)
}

/// Write a bundle to an arbitrary path. Creates parent directories if
/// needed. Exposed for tests.
pub fn write_bundle(path: &std::path::Path, bundle: &FstBundle) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let encoded = encode_bundle(bundle);
    // Atomic-ish write: stage to `<path>.tmp`, rename on success. Avoids
    // a partial file if we die mid-write, which would otherwise fail the
    // magic check on the next read (graceful but wasted rebuild).
    let tmp = path.with_extension("bin.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&encoded)?;
        f.sync_all().ok(); // best-effort fsync, don't fail on sync error
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────
// Encode / decode
// ──────────────────────────────────────────────────────────────────────

fn encode_bundle(bundle: &FstBundle) -> Vec<u8> {
    // Pre-size: magic(8) + hash(8) + 2×(fst_len(8) + fst_bytes + vals_count(8)
    // + values). A precise pre-size avoids several reallocations on big
    // corpora; rough overestimate is fine.
    let rough = 8 + 8
        + 16
        + bundle.stripped_bytes.len()
        + bundle.folded_bytes.len()
        + (bundle.values_stripped.len() + bundle.values_folded.len()) * 64;
    let mut out = Vec::with_capacity(rough);

    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&version_hash().to_le_bytes());

    encode_side(&mut out, &bundle.stripped_bytes, &bundle.values_stripped);
    encode_side(&mut out, &bundle.folded_bytes, &bundle.values_folded);

    out
}

fn encode_side(out: &mut Vec<u8>, fst_bytes: &[u8], values: &[GeneratedForm]) {
    out.extend_from_slice(&(fst_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(fst_bytes);
    out.extend_from_slice(&(values.len() as u64).to_le_bytes());
    for form in values {
        encode_form(out, form);
    }
}

fn encode_form(out: &mut Vec<u8>, form: &GeneratedForm) {
    out.push(encode_kind(form.pattern_kind));
    encode_str(out, &form.root_key);
    encode_str(out, &form.pattern_label);
    encode_str(out, &form.surface);
}

fn encode_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

fn decode_bundle(buf: &[u8]) -> io::Result<FstBundle> {
    let mut cur = Cursor { buf, pos: 0 };

    let magic = cur.read_bytes(8)?;
    if magic != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "arabic-fst cache: magic mismatch (not our file, or corruption)",
        ));
    }

    let got_hash = cur.read_u64()?;
    if got_hash != version_hash() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "arabic-fst cache: version hash mismatch (seed or code changed)",
        ));
    }

    let (stripped_bytes, values_stripped) = decode_side(&mut cur)?;
    let (folded_bytes, values_folded) = decode_side(&mut cur)?;

    // Trailing garbage is a protocol violation — reject rather than
    // silently accept. A partial write that happened to pass earlier
    // length checks would land here.
    if cur.pos != cur.buf.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "arabic-fst cache: {} trailing bytes after folded side",
                cur.buf.len() - cur.pos
            ),
        ));
    }

    Ok(FstBundle {
        stripped_bytes,
        values_stripped,
        folded_bytes,
        values_folded,
    })
}

fn decode_side(cur: &mut Cursor) -> io::Result<(Vec<u8>, Vec<GeneratedForm>)> {
    let fst_len = cur.read_u64()? as usize;
    let fst_bytes = cur.read_bytes(fst_len)?.to_vec();
    let val_count = cur.read_u64()? as usize;

    // Pre-reserve; values are bounded (u32 offsets in FST packing cap the
    // total values per side at 4B, far beyond any plausible corpus).
    let mut values = Vec::with_capacity(val_count);
    for _ in 0..val_count {
        values.push(decode_form(cur)?);
    }
    Ok((fst_bytes, values))
}

fn decode_form(cur: &mut Cursor) -> io::Result<GeneratedForm> {
    let kind_tag = cur.read_u8()?;
    let pattern_kind = decode_kind(kind_tag).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("arabic-fst cache: unknown PatternKind tag {kind_tag}"),
        )
    })?;
    let root_key = decode_str(cur)?;
    let pattern_label = decode_str(cur)?;
    let surface = decode_str(cur)?;
    Ok(GeneratedForm {
        root_key,
        pattern_label,
        pattern_kind,
        surface,
    })
}

fn decode_str(cur: &mut Cursor) -> io::Result<String> {
    let len = cur.read_u32()? as usize;
    let bytes = cur.read_bytes(len)?.to_vec();
    String::from_utf8(bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("arabic-fst cache: invalid UTF-8 in string field: {e}"),
        )
    })
}

// ──────────────────────────────────────────────────────────────────────
// PatternKind <-> u8 tag
//
// Hand-coded rather than `#[repr(u8)] as u8` so a developer reordering
// the enum for readability cannot silently reshuffle tags and orphan
// existing caches. Tags are append-only: NEVER renumber an existing one.
// When adding a new PatternKind, give it the next unused tag here and
// bump `CACHE_FORMAT_VERSION` so old caches invalidate.
// ──────────────────────────────────────────────────────────────────────

fn encode_kind(kind: PatternKind) -> u8 {
    match kind {
        PatternKind::VerbPerfect => 0,
        PatternKind::VerbImperfect => 1,
        PatternKind::VerbImperative => 2,
        PatternKind::VerbalNoun => 3,
        PatternKind::ActiveParticiple => 4,
        PatternKind::PassiveParticiple => 5,
        PatternKind::DerivedNoun => 6,
        PatternKind::BrokenPlural => 7,
        PatternKind::Diminutive => 8,
        PatternKind::Relative => 9,
        PatternKind::Elative => 10,
        PatternKind::Feminine => 11,
    }
}

fn decode_kind(tag: u8) -> Option<PatternKind> {
    Some(match tag {
        0 => PatternKind::VerbPerfect,
        1 => PatternKind::VerbImperfect,
        2 => PatternKind::VerbImperative,
        3 => PatternKind::VerbalNoun,
        4 => PatternKind::ActiveParticiple,
        5 => PatternKind::PassiveParticiple,
        6 => PatternKind::DerivedNoun,
        7 => PatternKind::BrokenPlural,
        8 => PatternKind::Diminutive,
        9 => PatternKind::Relative,
        10 => PatternKind::Elative,
        11 => PatternKind::Feminine,
        _ => return None,
    })
}

// ──────────────────────────────────────────────────────────────────────
// Byte cursor — a tiny read helper that errors cleanly on short buffers
// instead of panicking with an out-of-bounds slice access.
// ──────────────────────────────────────────────────────────────────────

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn read_bytes(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "length overflow")
        })?;
        if end > self.buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "arabic-fst cache: wanted {n} bytes at {}, only {} left",
                    self.pos,
                    self.buf.len() - self.pos
                ),
            ));
        }
        let out = &self.buf[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bundle() -> FstBundle {
        // Minimal hand-built bundle: one key, one form per side. We don't
        // need a real FST for encode/decode roundtrip tests — the cache
        // treats FST bytes as opaque blobs.
        FstBundle {
            stripped_bytes: vec![0xAA, 0xBB, 0xCC],
            values_stripped: vec![GeneratedForm {
                root_key: "ك-ت-ب".to_string(),
                pattern_label: "فَعَلَ".to_string(),
                pattern_kind: PatternKind::VerbPerfect,
                surface: "كَتَبَ".to_string(),
            }],
            folded_bytes: vec![0xDD, 0xEE],
            values_folded: vec![GeneratedForm {
                root_key: "ك-ت-ب".to_string(),
                pattern_label: "فَاعِل".to_string(),
                pattern_kind: PatternKind::ActiveParticiple,
                surface: "كاتب".to_string(),
            }],
        }
    }

    /// Pick a temp path unique to this test run, under the OS temp dir.
    /// Avoids stomping on the real user cache during tests.
    fn tmp_path(label: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        // `std::process::id` keeps parallel `cargo test` workers distinct;
        // `nanos` disambiguates two tests on the same process.
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        p.push(format!("constellation-fst-bake-{label}-{pid}-{nanos}.bin"));
        p
    }

    #[test]
    fn encode_kind_is_injective_and_total() {
        // Every PatternKind must round-trip through the tag space.
        use PatternKind::*;
        let all = [
            VerbPerfect,
            VerbImperfect,
            VerbImperative,
            VerbalNoun,
            ActiveParticiple,
            PassiveParticiple,
            DerivedNoun,
            BrokenPlural,
            Diminutive,
            Relative,
            Elative,
            Feminine,
        ];
        for k in all {
            let tag = encode_kind(k);
            assert_eq!(decode_kind(tag), Some(k), "{k:?} round-trip");
        }
        // Tags must be unique.
        let mut tags: Vec<u8> = all.iter().map(|&k| encode_kind(k)).collect();
        tags.sort_unstable();
        let n = tags.len();
        tags.dedup();
        assert_eq!(tags.len(), n, "duplicate PatternKind tags — orphans caches");
    }

    #[test]
    fn decode_kind_rejects_unknown_tag() {
        assert!(decode_kind(250).is_none());
    }

    #[test]
    fn encode_decode_form_roundtrip() {
        let form = GeneratedForm {
            root_key: "ض-ر-ب".to_string(),
            pattern_label: "فَعَلَ".to_string(),
            pattern_kind: PatternKind::VerbPerfect,
            surface: "ضَرَبَ".to_string(),
        };
        let mut buf = Vec::new();
        encode_form(&mut buf, &form);
        let mut cur = Cursor { buf: &buf, pos: 0 };
        let back = decode_form(&mut cur).expect("decode");
        assert_eq!(back.root_key, form.root_key);
        assert_eq!(back.pattern_label, form.pattern_label);
        assert_eq!(back.pattern_kind, form.pattern_kind);
        assert_eq!(back.surface, form.surface);
        assert_eq!(cur.pos, buf.len(), "decoded bytes must match encoded length");
    }

    #[test]
    fn bundle_write_read_roundtrip() {
        let path = tmp_path("roundtrip");
        let original = sample_bundle();
        write_bundle(&path, &original).expect("write");
        let loaded = load_bundle(&path).expect("load");

        assert_eq!(loaded.stripped_bytes, original.stripped_bytes);
        assert_eq!(loaded.folded_bytes, original.folded_bytes);
        assert_eq!(loaded.values_stripped.len(), original.values_stripped.len());
        assert_eq!(loaded.values_folded.len(), original.values_folded.len());
        assert_eq!(
            loaded.values_stripped[0].surface,
            original.values_stripped[0].surface
        );
        assert_eq!(
            loaded.values_folded[0].pattern_kind,
            original.values_folded[0].pattern_kind
        );

        // Cleanup. Don't fail the test if the file was already removed.
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_missing_file() {
        let path = tmp_path("missing");
        // We never write this path.
        assert!(load_bundle(&path).is_err(), "missing file must Err");
    }

    #[test]
    fn load_rejects_wrong_magic() {
        let path = tmp_path("wrongmagic");
        fs::write(&path, b"NOTMAGIC....garbage....").unwrap();
        let err = load_bundle(&path).expect_err("wrong magic must Err");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_truncated_file() {
        let path = tmp_path("truncated");
        let mut encoded = encode_bundle(&sample_bundle());
        // Lop off the last half. decode_bundle must detect the short read.
        encoded.truncate(encoded.len() / 2);
        fs::write(&path, &encoded).unwrap();
        assert!(load_bundle(&path).is_err(), "truncated file must Err");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_wrong_version_hash() {
        // Encode with the real hash, then flip bytes of the hash field
        // to simulate what a seed-TSV edit would look like to the reader.
        let path = tmp_path("wronghash");
        let mut encoded = encode_bundle(&sample_bundle());
        // The hash lives at bytes [8..16] (right after magic).
        encoded[8] ^= 0xFF;
        fs::write(&path, &encoded).unwrap();
        let err = load_bundle(&path).expect_err("wrong hash must Err");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_trailing_garbage() {
        let path = tmp_path("trailing");
        let mut encoded = encode_bundle(&sample_bundle());
        encoded.extend_from_slice(b"EXTRA BYTES");
        fs::write(&path, &encoded).unwrap();
        let err = load_bundle(&path).expect_err("trailing bytes must Err");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cache_file_path_includes_version_hash() {
        // Sanity: the computed filename carries the hash, which is how
        // seed-TSV changes invalidate caches even without rewriting the
        // file atomically.
        let Some(p) = cache_file_path() else {
            // Some minimal CI images have no cache dir — skip.
            return;
        };
        let name = p.file_name().and_then(|n| n.to_str()).unwrap();
        assert!(name.starts_with("arabic-fst-v"));
        assert!(name.ends_with(".bin"));
        let hash_hex = &name["arabic-fst-v".len()..name.len() - ".bin".len()];
        assert_eq!(
            hash_hex.len(),
            16,
            "hash must render as 16 lowercase hex chars (u64), got {hash_hex:?}"
        );
        assert!(
            hash_hex.chars().all(|c| c.is_ascii_hexdigit()),
            "hash filename segment must be pure hex, got {hash_hex:?}"
        );
    }

    #[test]
    fn version_hash_is_stable_across_calls() {
        // The OnceLock caching means successive calls must never disagree.
        // If this ever fails, seed hashing has become nondeterministic
        // and every launch would rebuild the cache.
        let a = version_hash();
        let b = version_hash();
        assert_eq!(a, b);
    }

    #[test]
    fn djb2_matches_known_values() {
        // Hand-verified seeds — guards against a refactor accidentally
        // changing the hash output and silently orphaning every cache on
        // disk. (If this test ever breaks intentionally, bump
        // CACHE_FORMAT_VERSION in the same commit.)
        // djb2("") = 5381 per Knuth's "Notes on Hashing".
        assert_eq!(djb2(b""), 5381);
        // djb2("a") = 5381 * 33 + 97 = 177670.
        assert_eq!(djb2(b"a"), 177670);
        // Determinism over an arbitrary blob.
        let x = djb2(b"hello world");
        let y = djb2(b"hello world");
        assert_eq!(x, y);
        assert_ne!(djb2(b"hello world"), djb2(b"hello worlD"));
    }

    #[test]
    fn persist_then_try_load_cached_roundtrip() {
        // End-to-end check on the real `cache_file_path()`: write via
        // `persist_best_effort`, read via `try_load_cached`. Only runs
        // when the platform has a cache dir.
        //
        // Uses a bundle whose contents we can recognise, so we can tell
        // a hit from an unrelated leftover file. If a stale cache from a
        // prior run of this same test is sitting in place, we overwrite
        // it (atomic rename) and still get a clean read.
        let Some(real_path) = cache_file_path() else {
            return;
        };
        let original = sample_bundle();
        persist_best_effort(&original);

        let loaded = try_load_cached().expect("try_load_cached must hit after persist");
        assert_eq!(loaded.stripped_bytes, original.stripped_bytes);
        assert_eq!(
            loaded.values_stripped[0].surface,
            original.values_stripped[0].surface
        );

        // Don't leave a hand-built (non-real) bundle in the user's
        // cache — the next real boot would load it, notice FST bytes
        // aren't valid fst::Map data, and fall through. But cleaner to
        // just delete.
        let _ = fs::remove_file(&real_path);
    }
}
