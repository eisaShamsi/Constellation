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

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use super::generator::{intern as intern_into_pool, GeneratedForm};
use super::types::PatternKind;

// ──────────────────────────────────────────────────────────────────────
// M9-mmap: FstBytes — backing storage for an FST byte buffer.
//
// On desktop targets (Windows / macOS / Linux) we prefer `Mmap` so the
// FST bytes live in the OS page cache rather than the process heap —
// at 7K-root scale this saves roughly 80 MiB of resident memory per
// launch. On iOS / Android where anon-mmap may be denied inside the
// sandbox we fall back to `Owned(Vec<u8>)`, the pre-M9-mmap behaviour.
//
// The two FSTs (stripped + folded) share a single `Arc<Mmap>` over the
// whole cache file; each `FstBytes::Mmap` value slices its own region
// out of the shared map. This keeps the mmap count at 1 per load, not
// 2 — important because each mmap costs a kernel syscall, a VMA entry,
// and (on Windows) a section-handle.
//
// `fst::Map<D>` requires `D: AsRef<[u8]>`, which this enum implements
// uniformly across variants. The `GenerativeFst` struct is therefore
// opaque to the storage choice — all call sites go through the same
// `.get(bytes)` FST API regardless of backend.
// ──────────────────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "ios", target_os = "android")))]
use memmap2::Mmap;

/// Read-only byte buffer backing a baked FST. See module-level comment.
///
/// The `Mmap` variant is only available on desktop targets (guarded by
/// `#[cfg(not(any(target_os = "ios", target_os = "android")))]`); on
/// mobile targets this enum collapses to a single `Owned` variant and
/// all loads go through `Vec<u8>`.
pub enum FstBytes {
    /// A slice into a shared memory-mapped file. `mmap` is wrapped in
    /// an `Arc` so the stripped and folded FSTs can share one map;
    /// `offset` / `len` identify this FST's byte region within the map.
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    Mmap {
        mmap: Arc<Mmap>,
        offset: usize,
        len: usize,
    },
    /// A heap-owned byte buffer. Produced by the cold-rebuild path
    /// (fresh FST bytes from `MapBuilder::into_inner`) and by any
    /// target where mmap isn't available.
    Owned(Vec<u8>),
}

impl AsRef<[u8]> for FstBytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            #[cfg(not(any(target_os = "ios", target_os = "android")))]
            Self::Mmap { mmap, offset, len } => &mmap[*offset..*offset + *len],
            Self::Owned(v) => v.as_slice(),
        }
    }
}

impl std::fmt::Debug for FstBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(any(target_os = "ios", target_os = "android")))]
            Self::Mmap { offset, len, .. } => f
                .debug_struct("FstBytes::Mmap")
                .field("offset", offset)
                .field("len", len)
                .finish(),
            Self::Owned(v) => f.debug_tuple("FstBytes::Owned").field(&v.len()).finish(),
        }
    }
}

impl From<Vec<u8>> for FstBytes {
    fn from(v: Vec<u8>) -> Self {
        Self::Owned(v)
    }
}

impl FstBytes {
    /// Length of the byte buffer. `as_ref().len()` would also work, but
    /// this is friendlier at struct-field sites that don't want to
    /// match on the enum.
    pub fn len(&self) -> usize {
        match self {
            #[cfg(not(any(target_os = "ios", target_os = "android")))]
            Self::Mmap { len, .. } => *len,
            Self::Owned(v) => v.len(),
        }
    }

}

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
/// and the two side-tables of [`GeneratedForm`].
///
/// The FST byte fields use [`FstBytes`] (M9-mmap) so the load path can
/// hand in mmap-backed slices over the cache file without copying the
/// FST bytes to heap. The cold-rebuild path wraps fresh `Vec<u8>` in
/// `FstBytes::Owned` via `From<Vec<u8>>`. The write path walks
/// `as_ref()` on the enum — the on-disk format is byte-identical to
/// pre-M9-mmap bundles, so `CACHE_FORMAT_VERSION` stays at 1 and old
/// caches remain readable.
#[derive(Debug)]
pub struct FstBundle {
    pub stripped_bytes: FstBytes,
    pub values_stripped: Vec<GeneratedForm>,
    pub folded_bytes: FstBytes,
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
    try_load_cached_at(&cache_file_path()?)
}

/// The path-taking core of [`try_load_cached`] — **the form tests must use.**
///
/// PJ-303: `cache_file_path()` names ONE file per machine, and it is shared with
/// the production initialiser [`crate::arabic::fst_index::GenerativeFst::get`],
/// which on any cache miss rebuilds the real bundle and re-persists it. A test
/// that writes a hand-built bundle to that path and reads it back is racing that
/// initialiser for a shared resource: when it loses, the read returns the real
/// FST bytes and the assertion fails. Measured at ~1 run in 6 before this split,
/// and reproduced deterministically by forcing `get()` into the window.
pub fn try_load_cached_at(path: &std::path::Path) -> Option<FstBundle> {
    load_bundle(path).ok()
}

/// Persist a bundle to the preferred cache path, best-effort. Any error
/// is swallowed — the caller's startup must never fail because the cache
/// dir is read-only or full.
pub fn persist_best_effort(bundle: &FstBundle) {
    if let Some(path) = cache_file_path() {
        persist_best_effort_at(&path, bundle);
    }
}

/// The path-taking core of [`persist_best_effort`] — **the form tests must use.**
/// See [`try_load_cached_at`] for why.
pub fn persist_best_effort_at(path: &std::path::Path, bundle: &FstBundle) {
    let _ = write_bundle(path, bundle);
}

/// Read a bundle from an arbitrary path. Exposed for tests so we don't
/// have to race the user's real cache dir.
///
/// On desktop targets (Windows / macOS / Linux) this prefers the mmap
/// path: the whole file is mapped once, both FST byte regions are
/// handed to `FstBytes::Mmap` as shared slices into that map, and only
/// the side-tables are decoded onto the heap. Mmap failure (which is
/// rare but possible — e.g. the file is on a filesystem that doesn't
/// support mapping, like a strict-sandboxed network share) falls back
/// to the heap-read path below.
///
/// On iOS / Android we skip the mmap attempt entirely and always read
/// into a `Vec<u8>` — the mobile sandboxes routinely deny anon-mmap.
pub fn load_bundle(path: &std::path::Path) -> io::Result<FstBundle> {
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        match load_bundle_mmap(path) {
            Ok(bundle) => return Ok(bundle),
            Err(_) => {
                // Fall through to the heap-read path. We don't log the
                // specific error — a subsequent heap-read failure will
                // surface the real issue with a matching error shape.
            }
        }
    }
    load_bundle_heap(path)
}

/// Heap-backed load path — used as the mobile default and as a fallback
/// when mmap isn't available. Reads the whole file into a `Vec<u8>`
/// and copies the FST bytes out into a second `Vec<u8>` per side.
fn load_bundle_heap(path: &std::path::Path) -> io::Result<FstBundle> {
    let mut file = fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    decode_bundle_heap(&buf)
}

/// Mmap-backed load path. Memory-maps the whole cache file once,
/// parses the headers + side tables eagerly (those stay on heap), and
/// hands the FST byte regions back as `FstBytes::Mmap` slices sharing
/// the single `Arc<Mmap>`. At 7K-root scale this keeps ~80 MiB of FST
/// bytes out of the process heap — the OS page cache supplies pages on
/// demand and the kernel evicts them under memory pressure.
///
/// The file format is unchanged (same byte layout as `encode_bundle`).
/// We parse offsets by reading the fixed-size header + the per-side
/// `fst_len` length field, then compute `offset = pos_after_len` and
/// `len = fst_len` for the FST byte region; `cur.pos` advances past
/// the bytes before the side-table decode so the rest of the code path
/// is shape-identical to the heap path.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
fn load_bundle_mmap(path: &std::path::Path) -> io::Result<FstBundle> {
    let file = fs::File::open(path)?;
    // SAFETY: `Mmap::map` is `unsafe` because the caller must ensure the
    // underlying file is not modified while the mapping is live — a
    // concurrent writer could make the mapped bytes inconsistent. In
    // our deployment the cache file is written once at startup (via
    // atomic rename in `write_bundle`), lives for the lifetime of the
    // process, and is never mutated in place. A second process baking
    // the same cache would also write via atomic rename, so the mapped
    // bytes either remain the old file (backed by its inode until we
    // drop the mmap) or become stale but internally consistent. The
    // invariant holds for our deployment pattern.
    let mmap = unsafe { Mmap::map(&file)? };
    let mmap = Arc::new(mmap);
    decode_bundle_mmap(mmap)
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

    // `.as_ref()` produces `&[u8]` whether the underlying storage is a
    // heap `Vec<u8>` (cold-rebuild path) or a slice into the mmap (warm
    // load path). The on-disk byte layout is identical in both cases.
    encode_side(&mut out, bundle.stripped_bytes.as_ref(), &bundle.values_stripped);
    encode_side(&mut out, bundle.folded_bytes.as_ref(), &bundle.values_folded);

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
    // M9-intern: `form.root_key` and `form.pattern_label` are now
    // `Arc<str>` (shared across forms that share the same string).
    // `Arc<str>` derefs to `str`, so `&form.root_key` coerces cleanly
    // to `&str` at the call site — on-disk format is byte-identical to
    // the pre-M9-intern layout (same length prefix + UTF-8 bytes). No
    // CACHE_FORMAT_VERSION bump required.
    encode_str(out, &form.root_key);
    encode_str(out, &form.pattern_label);
    encode_str(out, &form.surface);
}

fn encode_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

/// Heap-backed decode path. Copies the FST byte regions out of `buf` into
/// owned `Vec<u8>`s wrapped in [`FstBytes::Owned`]. Used by the mobile
/// default and as the fallback when mmap isn't available. Parses side
/// tables identically to the mmap path — the two only differ in where the
/// FST bytes physically live.
fn decode_bundle_heap(buf: &[u8]) -> io::Result<FstBundle> {
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

    // M9-intern — dedup pools for the `Arc<str>` fields on decoded forms.
    // Shared across both sides (stripped + folded) because the two side
    // tables overlap heavily (the folded table is a strict subset of the
    // stripped table in key space but has its own value rows). Without
    // the shared pool each side would allocate its own `Arc` per string
    // and lose half the sharing win.
    let mut root_pool: HashMap<String, Arc<str>> = HashMap::new();
    let mut label_pool: HashMap<String, Arc<str>> = HashMap::new();

    let (stripped_bytes, values_stripped) =
        decode_side_heap(&mut cur, &mut root_pool, &mut label_pool)?;
    let (folded_bytes, values_folded) =
        decode_side_heap(&mut cur, &mut root_pool, &mut label_pool)?;

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
        stripped_bytes: FstBytes::Owned(stripped_bytes),
        values_stripped,
        folded_bytes: FstBytes::Owned(folded_bytes),
        values_folded,
    })
}

fn decode_side_heap(
    cur: &mut Cursor,
    root_pool: &mut HashMap<String, Arc<str>>,
    label_pool: &mut HashMap<String, Arc<str>>,
) -> io::Result<(Vec<u8>, Vec<GeneratedForm>)> {
    let fst_len = cur.read_u64()? as usize;
    let fst_bytes = cur.read_bytes(fst_len)?.to_vec();
    let val_count = cur.read_u64()? as usize;

    // Pre-reserve; values are bounded (u32 offsets in FST packing cap the
    // total values per side at 4B, far beyond any plausible corpus).
    let mut values = Vec::with_capacity(val_count);
    for _ in 0..val_count {
        values.push(decode_form(cur, root_pool, label_pool)?);
    }
    Ok((fst_bytes, values))
}

/// Mmap-backed decode path. Hands back [`FstBytes::Mmap`] slices that share
/// a single `Arc<Mmap>` covering the whole cache file — the FST bytes stay
/// in the OS page cache and never hit the process heap. Side tables are
/// decoded eagerly (same as the heap path) because they own their own
/// `Arc<str>` / `String` heap allocations and can't be slice-backed.
///
/// The byte layout is identical to [`decode_bundle_heap`] — offsets into
/// the mmap are computed from the cursor position after each length prefix
/// is read. Any parse error (short buffer, bad magic, hash mismatch,
/// trailing garbage) returns `Err` and the load path falls back to the
/// heap reader.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
fn decode_bundle_mmap(mmap: Arc<Mmap>) -> io::Result<FstBundle> {
    // Borrow the mmap as a byte slice for the cursor's lifetime. Because
    // the cursor's `'a` is tied to this local borrow, every `&'a [u8]`
    // the cursor hands out is valid for the body of this function. The
    // returned `FstBundle` carries its own `Arc<Mmap>` clones inside each
    // `FstBytes::Mmap`, not the borrow — so the mmap lives on past the
    // cursor's drop.
    let buf: &[u8] = &mmap[..];
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

    let mut root_pool: HashMap<String, Arc<str>> = HashMap::new();
    let mut label_pool: HashMap<String, Arc<str>> = HashMap::new();

    let (stripped_bytes, values_stripped) =
        decode_side_mmap(&mut cur, &mmap, &mut root_pool, &mut label_pool)?;
    let (folded_bytes, values_folded) =
        decode_side_mmap(&mut cur, &mmap, &mut root_pool, &mut label_pool)?;

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

/// Side decoder paired with [`decode_bundle_mmap`]. Captures the cursor's
/// pre-advance position as the FST byte region's offset in the mmap,
/// advances past the region, then decodes the side table. The returned
/// `FstBytes::Mmap` carries an `Arc::clone` of the shared map plus the
/// captured offset/length — no FST bytes are copied.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
fn decode_side_mmap(
    cur: &mut Cursor,
    mmap: &Arc<Mmap>,
    root_pool: &mut HashMap<String, Arc<str>>,
    label_pool: &mut HashMap<String, Arc<str>>,
) -> io::Result<(FstBytes, Vec<GeneratedForm>)> {
    let fst_len = cur.read_u64()? as usize;
    // Snapshot the cursor position *before* advancing past the FST bytes
    // so the mmap slice spans exactly the same byte range the heap path
    // would have copied into `fst_bytes`.
    let offset = cur.pos;
    let _ = cur.read_bytes(fst_len)?; // advance; drop the returned slice
    let fst_bytes = FstBytes::Mmap {
        mmap: Arc::clone(mmap),
        offset,
        len: fst_len,
    };
    let val_count = cur.read_u64()? as usize;

    let mut values = Vec::with_capacity(val_count);
    for _ in 0..val_count {
        values.push(decode_form(cur, root_pool, label_pool)?);
    }
    Ok((fst_bytes, values))
}

fn decode_form(
    cur: &mut Cursor,
    root_pool: &mut HashMap<String, Arc<str>>,
    label_pool: &mut HashMap<String, Arc<str>>,
) -> io::Result<GeneratedForm> {
    let kind_tag = cur.read_u8()?;
    let pattern_kind = decode_kind(kind_tag).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("arabic-fst cache: unknown PatternKind tag {kind_tag}"),
        )
    })?;
    let root_key_str = decode_str(cur)?;
    let pattern_label_str = decode_str(cur)?;
    let surface = decode_str(cur)?;
    // M9-intern: funnel the decoded string through a shared pool so
    // every form that references the same root or pattern shares a
    // single heap allocation. Cache-hit loads now get the same sharing
    // as cold rebuilds.
    let root_key = intern_into_pool(root_pool, &root_key_str);
    let pattern_label = intern_into_pool(label_pool, &pattern_label_str);
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
        //
        // M9-mmap: FST byte fields moved from `Vec<u8>` to `FstBytes`.
        // Cold-build paths land on `FstBytes::Owned`; tests follow suit
        // via `From<Vec<u8>>` (`.into()`).
        FstBundle {
            stripped_bytes: vec![0xAA, 0xBB, 0xCC].into(),
            values_stripped: vec![GeneratedForm {
                root_key: "ك-ت-ب".into(),
                pattern_label: "فَعَلَ".into(),
                pattern_kind: PatternKind::VerbPerfect,
                surface: "كَتَبَ".to_string(),
            }],
            folded_bytes: vec![0xDD, 0xEE].into(),
            values_folded: vec![GeneratedForm {
                root_key: "ك-ت-ب".into(),
                pattern_label: "فَاعِل".into(),
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
            root_key: "ض-ر-ب".into(),
            pattern_label: "فَعَلَ".into(),
            pattern_kind: PatternKind::VerbPerfect,
            surface: "ضَرَبَ".to_string(),
        };
        let mut buf = Vec::new();
        encode_form(&mut buf, &form);
        let mut cur = Cursor { buf: &buf, pos: 0 };
        // M9-intern: decode_form now requires mutable intern pools so
        // roundtrip tests must provide them (empty is fine for one-off).
        let mut root_pool: HashMap<String, Arc<str>> = HashMap::new();
        let mut label_pool: HashMap<String, Arc<str>> = HashMap::new();
        let back = decode_form(&mut cur, &mut root_pool, &mut label_pool).expect("decode");
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

        // M9-mmap: compare via `.as_ref()` since `FstBytes` is backed by
        // either a `Vec<u8>` or an mmap slice and we don't derive
        // `PartialEq` on the enum (the `Arc<Mmap>` field isn't
        // meaningfully comparable). The underlying bytes must still
        // match exactly.
        assert_eq!(loaded.stripped_bytes.as_ref(), original.stripped_bytes.as_ref());
        assert_eq!(loaded.folded_bytes.as_ref(), original.folded_bytes.as_ref());
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

    /// End-to-end check on the persist/load pair — `persist_best_effort_at`
    /// then `try_load_cached_at` — over an ISOLATED path.
    ///
    /// **PJ-303 (2026-08-17).** This used the real `cache_file_path()`, and that
    /// made it flaky at ~1 run in 6: that path names one file per machine, shared
    /// with `GenerativeFst::get`, which rebuilds and re-persists the REAL bundle
    /// on any cache miss — and a hand-built `sample_bundle()` guarantees a miss,
    /// because `[0xAA,0xBB,0xCC]` is not a valid `fst::Map`. So the production
    /// initialiser overwrote this test's bundle between its write and its read,
    /// and the assertion failed printing the real FST bytes. Reproduced
    /// deterministically by forcing `get()` into the window.
    ///
    /// It also deleted the developer's real Arabic cache on every run.
    ///
    /// `tmp_path`'s own doc already said *"Avoids stomping on the real user cache
    /// during tests"* — this test simply was not using it. It is now, and the
    /// no-arg `try_load_cached` / `persist_best_effort` (whose only extra logic is
    /// resolving `cache_file_path()`) belong to production alone.
    #[test]
    fn persist_then_try_load_cached_roundtrip() {
        let path = tmp_path("cached_roundtrip");
        let original = sample_bundle();
        persist_best_effort_at(&path, &original);

        let loaded =
            try_load_cached_at(&path).expect("try_load_cached_at must hit after persist");
        // `.as_ref()` to compare through the `FstBytes` enum regardless
        // of which backing variant (`Owned` / `Mmap`) each path produced.
        assert_eq!(loaded.stripped_bytes.as_ref(), original.stripped_bytes.as_ref());
        assert_eq!(
            loaded.values_stripped[0].surface,
            original.values_stripped[0].surface
        );

        // The error-swallowing half of the contract, which `write_bundle` /
        // `load_bundle` do not have: a miss is `None`, never a panic or an Err.
        let _ = fs::remove_file(&path);
        assert!(
            try_load_cached_at(&path).is_none(),
            "a missing cache file must read as None, not an error"
        );
    }
}
