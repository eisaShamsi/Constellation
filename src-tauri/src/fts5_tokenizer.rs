//! Custom FTS5 tokenizer for Constellation.
//!
//! Wraps the existing Rust stemming pipeline (Arabic Light10 + Hebrew
//! prefix stripping + Persian / Cyrillic / Devanagari / German / Spanish /
//! Portuguese / French / Turkish / English stemmers + bigrams) and
//! registers it with SQLite FTS5 so that:
//!
//!   * `notes_fts` stores stemmed tokens (e.g. "كتاب" not "الكتاب",
//!     "run" not "running"), collapsing the ~452k surface forms of a
//!     7,600-note Arabic-heavy Universe to ~30-60k meaningful stems.
//!   * `notes_vocab` (the `fts5vocab(notes_fts, 'row')` view consumed by
//!     the Index panel) shows those stems as its primary rows, with
//!     bigrams as colocated tokens at the same logical position.
//!   * `MATCH` queries stem symmetrically: FTS5 calls our tokenizer with
//!     `FTS5_TOKENIZE_QUERY` on the query string too, so searching
//!     "running" finds documents containing "ran" / "runs" / "runner".
//!
//! ## Why a real tokenizer and not pre-stemming in Rust
//!
//! FTS5 is designed so the tokenizer is invoked symmetrically on write
//! (document insert) and read (MATCH query). That means:
//!
//!   * No query interception needed in app code — fragile and error-prone.
//!   * `snippet()` / `highlight()` still render original text because
//!     FTS5 stores token byte offsets separately from token contents.
//!     We emit the stem as the token bytes but report the original
//!     word's byte range, so highlighting points into the real note body.
//!   * `notes_vocab` collapses naturally to stems — no custom rebuild.
//!
//! ## FFI glue credit
//!
//! The `register_tokenizer` helper and the four `unsafe extern "C"`
//! shims (`c_xcreate`, `c_xdelete`, `c_xtokenize`, `c_xdestroy`) are
//! adapted from ColonelThirtyTwo's MIT-licensed gist:
//!   <https://gist.github.com/ColonelThirtyTwo/3dd1fe04e4cff0502fa70d12f3a6e72e>
//! Used per LL-024 (research before reinvention) — the FFI wrapping
//! pattern is load-bearing and well-trodden; rewriting it from scratch
//! would have been reinventing the wheel at the exact spot the rule
//! was written for.
//!
//! The `Tokenizer` impl (`ConstellationTokenizer`) is ours and delegates
//! every per-word decision to `crate::libraries::process_word_for_fts`
//! so there is a single stemming pipeline across the codebase.

use std::{
    collections::HashSet,
    convert::TryFrom,
    ffi::{c_void, CStr, CString},
    ops::Range,
    os::raw::{c_char, c_int},
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Arc,
};

use rusqlite::ffi::{self, SQLITE_ERROR, SQLITE_OK};

// ─── FFI glue (adapted from ColonelThirtyTwo, MIT) ─────────────────────

/// Pattern-matchable combined flag: query + prefix.
const FTS5_TOKENIZE_QUERY_PREFIX: c_int =
    ffi::FTS5_TOKENIZE_QUERY | ffi::FTS5_TOKENIZE_PREFIX;

/// Why FTS5 is invoking `xTokenize`. Useful if a tokenizer wants to
/// behave differently at write vs. query time (we don't — the whole
/// point of a real FTS5 tokenizer is symmetric stemming).
pub enum TokenizeReason {
    /// A document is being inserted, updated, or deleted — tokenize the
    /// stored column so the inverted index reflects its contents.
    Document,
    /// A `MATCH` query is running — tokenize the query string so lookups
    /// key on the same token space as the stored documents.
    Query {
        /// If set, the last token emitted is treated as a prefix match
        /// (e.g. `MATCH 'const*'`). Currently unused by
        /// `ConstellationTokenizer` — we stem identically for plain and
        /// prefix queries — but exposed on the public trait so future
        /// tokenizer impls can branch on it.
        #[allow(dead_code)]
        prefix: bool,
    },
    /// Manually invoked via `fts5_api.xTokenize` by an auxiliary function.
    Aux,
}

impl TokenizeReason {
    fn from_const(v: c_int) -> Option<Self> {
        Some(match v {
            ffi::FTS5_TOKENIZE_DOCUMENT => Self::Document,
            ffi::FTS5_TOKENIZE_QUERY => Self::Query { prefix: false },
            FTS5_TOKENIZE_QUERY_PREFIX => Self::Query { prefix: true },
            ffi::FTS5_TOKENIZE_AUX => Self::Aux,
            _ => return None,
        })
    }
}

/// Trait a Rust-side tokenizer implements. Instances are owned by FTS5
/// and live for the lifetime of the virtual-table reference (they are
/// recreated on connection reopen). Must be `Send + 'static` because
/// SQLite owns them across an FFI boundary.
pub trait Tokenizer: Sized + Send + 'static {
    /// Data that is shared across every instance and outlives them.
    /// Most often `()` or an `Arc`-wrapped config.
    type Global: Send + Sync + 'static;

    /// Construct a new instance. `args` is the space-separated option
    /// list passed in the `tokenize='name arg1 arg2'` clause.
    fn new(global: &Self::Global, args: Vec<String>) -> Result<Self, rusqlite::Error>;

    /// Emit tokens for `text`. Each call to `push_token` adds one token
    /// to the inverted index:
    ///   * `token` — the bytes to store (may be arbitrary, not UTF-8).
    ///   * `range` — byte offsets into `text` that the token corresponds
    ///     to in the original input. Used by `snippet()` / `highlight()`.
    ///   * `colocated` — if true, the token shares a position with the
    ///     previous non-colocated one. Used for synonyms / bigrams.
    fn tokenize<TKF>(
        &mut self,
        reason: TokenizeReason,
        text: &[u8],
        push_token: TKF,
    ) -> Result<(), rusqlite::Error>
    where
        TKF: FnMut(&[u8], Range<usize>, bool) -> Result<(), rusqlite::Error>;
}

unsafe extern "C" fn c_xcreate<T: Tokenizer>(
    global: *mut c_void,
    args: *mut *const c_char,
    nargs: c_int,
    out_tok: *mut *mut ffi::Fts5Tokenizer,
) -> c_int {
    let global = &*global.cast::<T::Global>();
    let args = (0..nargs as usize)
        .map(|i| *args.add(i))
        .map(|s| CStr::from_ptr(s).to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    match catch_unwind(AssertUnwindSafe(move || T::new(global, args))) {
        Ok(Ok(v)) => {
            let bp = Box::into_raw(Box::new(v));
            *out_tok = bp.cast::<ffi::Fts5Tokenizer>();
            SQLITE_OK
        }
        Ok(Err(rusqlite::Error::SqliteFailure(e, _))) => e.extended_code,
        Ok(Err(_)) => SQLITE_ERROR,
        Err(msg) => {
            eprintln!(
                "[fts5_tokenizer] <{} as Tokenizer>::new panicked: {}",
                std::any::type_name::<T>(),
                panic_err_to_str(&msg),
            );
            SQLITE_ERROR
        }
    }
}

unsafe extern "C" fn c_xdelete<T: Tokenizer>(v: *mut ffi::Fts5Tokenizer) {
    let b = Box::from_raw(v.cast::<T>());
    let _ = catch_unwind(AssertUnwindSafe(move || std::mem::drop(b)));
}

unsafe extern "C" fn c_xdestroy<T: Tokenizer>(v: *mut c_void) {
    let b = Box::from_raw(v.cast::<T::Global>());
    let _ = catch_unwind(AssertUnwindSafe(move || std::mem::drop(b)));
}

unsafe extern "C" fn c_xtokenize<T: Tokenizer>(
    this: *mut ffi::Fts5Tokenizer,
    ctx: *mut c_void,
    flags: c_int,
    data: *const c_char,
    data_len: c_int,
    push_token: Option<
        unsafe extern "C" fn(*mut c_void, c_int, *const c_char, c_int, c_int, c_int) -> c_int,
    >,
) -> c_int {
    let this = &mut *this.cast::<T>();
    let reason = match TokenizeReason::from_const(flags) {
        Some(v) => v,
        None => {
            eprintln!("[fts5_tokenizer] unrecognized flags in xTokenize: {}", flags);
            return SQLITE_ERROR;
        }
    };
    let data = std::slice::from_raw_parts(data.cast::<u8>(), data_len as usize);
    let push_token_fn = push_token.unwrap();
    let push = |token: &[u8], range: Range<usize>, colocated: bool| -> Result<(), rusqlite::Error> {
        // Use SqliteFailure for the overflow path — ModuleError is gated on
        // the `vtab` feature which we don't enable (we only need `bundled`).
        let ntoken = c_int::try_from(token.len()).map_err(|_| {
            rusqlite::Error::SqliteFailure(
                ffi::Error::new(ffi::SQLITE_TOOBIG),
                Some("token longer than c_int".into()),
            )
        })?;
        debug_assert!(
            range.start <= data.len() && range.end <= data.len(),
            "token range out of bounds: {:?} vs data.len={}",
            range,
            data.len(),
        );
        let flags = if colocated { ffi::FTS5_TOKEN_COLOCATED } else { 0 };
        let res = push_token_fn(
            ctx,
            flags,
            token.as_ptr().cast::<c_char>(),
            ntoken,
            range.start as c_int,
            range.end as c_int,
        );
        if res == SQLITE_OK {
            Ok(())
        } else {
            Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(res),
                None,
            ))
        }
    };
    match catch_unwind(AssertUnwindSafe(|| this.tokenize(reason, data, push))) {
        Ok(Ok(())) => SQLITE_OK,
        Ok(Err(rusqlite::Error::SqliteFailure(e, _))) => e.extended_code,
        Ok(Err(_)) => SQLITE_ERROR,
        Err(msg) => {
            eprintln!(
                "[fts5_tokenizer] <{} as Tokenizer>::tokenize panicked: {}",
                std::any::type_name::<T>(),
                panic_err_to_str(&msg),
            );
            SQLITE_ERROR
        }
    }
}

fn panic_err_to_str(msg: &Box<dyn std::any::Any + Send>) -> &str {
    if let Some(msg) = msg.downcast_ref::<String>() {
        msg.as_str()
    } else if let Some(msg) = msg.downcast_ref::<&'static str>() {
        *msg
    } else {
        "<non-string panic reason>"
    }
}

/// Register a tokenizer on the given connection. The returned tokenizer
/// is connection-local — each new `Connection::open` needs its own
/// registration (see `crate::search::register_fts5_tokenizer`).
///
/// `global_data` is boxed and owned by FTS5; SQLite calls our
/// `xDestroy` when the tokenizer is unregistered (on connection close).
pub fn register_tokenizer<T: Tokenizer>(
    db: &mut rusqlite::Connection,
    global_data: T::Global,
    name: &str,
) -> Result<(), String> {
    unsafe {
        let dbp = db.handle();
        let mut api: *mut ffi::fts5_api = std::ptr::null_mut();
        let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();

        // The canonical way to fetch the FTS5 API pointer: prepare a
        // `SELECT fts5(?1)` statement and bind a pointer-typed
        // parameter — SQLite fills in our pointer via the out-binding.
        let q = "SELECT fts5(?1)";
        if ffi::sqlite3_prepare(
            dbp,
            q.as_ptr().cast::<c_char>(),
            q.len() as c_int,
            &mut stmt,
            std::ptr::null_mut(),
        ) != SQLITE_OK
        {
            return Err("sqlite3_prepare(SELECT fts5(?1)) failed".into());
        }
        ffi::sqlite3_bind_pointer(
            stmt,
            1,
            (&mut api) as *mut _ as *mut c_void,
            "fts5_api_ptr\0".as_ptr().cast::<c_char>(),
            None,
        );
        ffi::sqlite3_step(stmt);
        ffi::sqlite3_finalize(stmt);

        if api.is_null() {
            return Err("could not obtain fts5_api pointer (FTS5 missing from build?)".into());
        }

        let cname = CString::new(name).map_err(|_| "tokenizer name contains NUL")?;
        let boxed_global = Box::into_raw(Box::new(global_data));

        let e = ((*api).xCreateTokenizer.as_ref().unwrap())(
            api,
            cname.as_ptr(),
            boxed_global.cast::<c_void>(),
            &mut ffi::fts5_tokenizer {
                xCreate: Some(c_xcreate::<T>),
                xDelete: Some(c_xdelete::<T>),
                xTokenize: Some(c_xtokenize::<T>),
            },
            Some(c_xdestroy::<T>),
        );
        if e != SQLITE_OK {
            // Take back the Box so we don't leak if registration failed.
            let _ = Box::from_raw(boxed_global);
            return Err(format!("xCreateTokenizer('{}') failed: code {}", name, e));
        }
        Ok(())
    }
}

// ─── Constellation tokenizer ───────────────────────────────────────────

/// Byte sentinel joining the two halves of a bigram token.
/// `\x1f` (Unit Separator, C0 control) can't appear in user text, so
/// any vocab entry containing it is guaranteed to be a bigram. The
/// Index panel's read path (`read_index_entries` in libraries.rs) can
/// split on this byte to render bigrams as two-word phrases.
pub const BIGRAM_SEP: u8 = 0x1f;

/// The tokenizer's global data. Holds the stopword set once per
/// registration; cloned (via `Arc`) into every `ConstellationTokenizer`
/// instance so tokenize calls don't need to reach back through FTS5.
pub struct ConstellationGlobal {
    pub stopwords: Arc<HashSet<String>>,
}

/// Per-instance tokenizer state. FTS5 may create multiple of these
/// (one per virtual-table operation is the typical pattern); sharing
/// stopwords by `Arc` keeps that cheap.
pub struct ConstellationTokenizer {
    stopwords: Arc<HashSet<String>>,
}

impl Tokenizer for ConstellationTokenizer {
    type Global = ConstellationGlobal;

    fn new(global: &Self::Global, _args: Vec<String>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            stopwords: Arc::clone(&global.stopwords),
        })
    }

    fn tokenize<TKF>(
        &mut self,
        _reason: TokenizeReason,
        text: &[u8],
        mut push_token: TKF,
    ) -> Result<(), rusqlite::Error>
    where
        TKF: FnMut(&[u8], Range<usize>, bool) -> Result<(), rusqlite::Error>,
    {
        // FTS5 stores columns as text (our schema uses TEXT for
        // `body_text` and `name`), so `text` is always UTF-8 in
        // practice. If it isn't, yield no tokens rather than panic
        // — a malformed row is better than a broken connection.
        let text_str = match std::str::from_utf8(text) {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let stopwords = &*self.stopwords;
        // Previous non-stopword stem, for bigram formation. Reset to
        // None on every boundary that breaks the bigram chain
        // (filtered word, stopword, script mismatch).
        let mut prev_stem: Option<String> = None;

        // Walk char boundaries, splitting on the same rules as
        // `libraries::tokenize_note_body`:
        //   * apostrophe: NOT a boundary (keeps contractions together)
        //   * em/en/hyphen/underscore: boundary
        //   * anything non-alphabetic: boundary
        let mut word_start: Option<usize> = None;

        for (byte_idx, ch) in text_str.char_indices() {
            if is_word_boundary(ch) {
                if let Some(start) = word_start.take() {
                    let end = byte_idx;
                    emit_word(
                        &text_str[start..end],
                        start,
                        end,
                        stopwords,
                        &mut prev_stem,
                        &mut push_token,
                    )?;
                }
            } else if word_start.is_none() {
                word_start = Some(byte_idx);
            }
        }
        // Tail word (no trailing boundary char).
        if let Some(start) = word_start {
            let end = text_str.len();
            emit_word(
                &text_str[start..end],
                start,
                end,
                stopwords,
                &mut prev_stem,
                &mut push_token,
            )?;
        }
        Ok(())
    }
}

/// Boundary predicate matching `libraries::tokenize_note_body`.
#[inline]
fn is_word_boundary(c: char) -> bool {
    if c == '\'' {
        return false;
    }
    if c == '—' || c == '–' || c == '-' || c == '_' {
        return true;
    }
    !c.is_alphabetic()
}

/// Process a single word span and emit 0, 1, or 2 tokens:
///   * 0 tokens if the word is filtered (too short, stopword, noise).
///   * 1 token — the stem — if emitted normally. Position advances.
///   * 2 tokens if a bigram forms with the previous non-stopword word:
///     the stem first, then `prev_stem \x1f cur_stem` as colocated.
/// MIG-012-fix-8 — same tokenization pipeline as the FTS5 tokenizer's
/// `tokenize` method, but emits to a `Vec<String>` instead of an FTS5
/// cursor. Used by `ctse::hooks::on_note_indexed` (MIG-013 §1C) to
/// maintain the `term_vocab` shadow table on every note save with
/// byte-identical tokens to what `notes_fts` actually stores.
///
/// Output includes both stems (primary tokens) AND bigrams (joined by
/// `BIGRAM_SEP`), matching the FTS5 tokenizer's emission. Stopwords
/// are filtered the same way.
pub fn tokenize_to_vec(text: &str, stopwords: &HashSet<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut prev_stem: Option<String> = None;
    let mut word_start: Option<usize> = None;

    for (byte_idx, ch) in text.char_indices() {
        if is_word_boundary(ch) {
            if let Some(start) = word_start.take() {
                let end = byte_idx;
                emit_word_collect(
                    &text[start..end],
                    stopwords,
                    &mut prev_stem,
                    &mut out,
                );
            }
        } else if word_start.is_none() {
            word_start = Some(byte_idx);
        }
    }
    if let Some(start) = word_start {
        let end = text.len();
        emit_word_collect(
            &text[start..end],
            stopwords,
            &mut prev_stem,
            &mut out,
        );
    }
    out
}

/// Vec-emitting twin of `emit_word`. Mirrors that function exactly:
/// strip-quotes → process_word_for_fts → stopword check → emit primary
/// stem → emit bigram if previous same-script stem exists.
fn emit_word_collect(
    raw_word: &str,
    stopwords: &HashSet<String>,
    prev_stem: &mut Option<String>,
    out: &mut Vec<String>,
) {
    let word = raw_word.trim_matches('\'');
    if word.is_empty() {
        *prev_stem = None;
        return;
    }
    let (stem, norm_lower) = match crate::libraries::process_word_for_fts(word) {
        Some(v) => v,
        None => {
            *prev_stem = None;
            return;
        }
    };
    if stopwords.contains(&stem) || stopwords.contains(&norm_lower) {
        *prev_stem = None;
        return;
    }
    out.push(stem.clone());
    if let Some(prev) = prev_stem.as_ref() {
        if crate::libraries::is_same_script(prev, &stem) {
            let mut bigram = String::with_capacity(prev.len() + 1 + stem.len());
            bigram.push_str(prev);
            bigram.push('\u{001F}');
            bigram.push_str(&stem);
            out.push(bigram);
        }
    }
    *prev_stem = Some(stem);
}

fn emit_word<TKF>(
    raw_word: &str,
    start: usize,
    end: usize,
    stopwords: &HashSet<String>,
    prev_stem: &mut Option<String>,
    push_token: &mut TKF,
) -> Result<(), rusqlite::Error>
where
    TKF: FnMut(&[u8], Range<usize>, bool) -> Result<(), rusqlite::Error>,
{
    // Strip only leading/trailing apostrophes, same as tokenize_note_body.
    let word = raw_word.trim_matches('\'');
    if word.is_empty() {
        *prev_stem = None;
        return Ok(());
    }

    let (stem, norm_lower) = match crate::libraries::process_word_for_fts(word) {
        Some(v) => v,
        None => {
            // Length-filtered — break bigram chain, no token.
            *prev_stem = None;
            return Ok(());
        }
    };

    if stopwords.contains(&stem) || stopwords.contains(&norm_lower) {
        // Stopword — no token, break bigram chain (bigrams should not
        // span stopwords, matching the original tokenize_note_body
        // semantic).
        *prev_stem = None;
        return Ok(());
    }

    // Primary token: the stem, with the original word's byte range.
    push_token(stem.as_bytes(), start..end, false)?;

    // Bigram: `<prev_stem> BIGRAM_SEP <cur_stem>`, colocated with the
    // current position. Only emitted when the previous non-stopword
    // word is in the same script (avoids "الكتاب_english" nonsense).
    if let Some(prev) = prev_stem.as_ref() {
        if crate::libraries::is_same_script(prev, &stem) {
            let mut bigram = Vec::with_capacity(prev.len() + 1 + stem.len());
            bigram.extend_from_slice(prev.as_bytes());
            bigram.push(BIGRAM_SEP);
            bigram.extend_from_slice(stem.as_bytes());
            push_token(&bigram, start..end, true)?;
        }
    }

    *prev_stem = Some(stem);
    Ok(())
}
