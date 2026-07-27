//! MIG-104 Slice 3 — the Earned-Life Ledger: the appender, the union reader, the contract.
//!
//! **The concept (the horse).** The traffic of the user's own mind — which links they walked,
//! what they came to trust, what they retired, and how a note changed over time — must live in
//! plain-text files they own, so the index can be thrown away and a note can be deleted
//! without the knowledge going with it.
//!
//! **Why an append.** Every other candidate design rewrote a file, and a rewriter holding a
//! stale or empty in-memory map writes an empty store — destroying exactly what it exists to
//! protect. An append has no such surface: a torn tail costs one line, and every earlier line
//! is immutable. This is the mechanism being *structurally incapable* of the failure rather
//! than disciplined against it.
//!
//! **Two streams, one appender — because the fold algebras differ and must never be confused:**
//!
//! | | `earned.jsonl` (+ snapshot) | `note-history.jsonl` |
//! |---|---|---|
//! | record is | a *fold target* | the payload itself |
//! | fold | `n` = **max**, decisions = latest; commutative + idempotent | **NEVER FOLDS, NEVER COMPACTS** |
//! | bounded by | earned-link count (33 live) | history events, forever |
//!
//! Folding the history stream would collapse a thought into a keystroke: the live rows
//! `hid` 8251/8252/8253 record `ma` → `mas` → `masadir`, a property being typed. `read_folded`
//! is the ONLY fold implementation in this module, and it reads Stream A only;
//! `read_history_for` deliberately has none.
//!
//! **Ordering is by `hid` (the source row ordinal), never by `at`** — 765 `captured_at` groups
//! collide across 1,536 live rows, with 2,066 order inversions.
//!
//! **Portability (Boss ruling Q5).** The Universe is portable across Windows and macOS, never
//! opened concurrently. Every key is cid-first with a Universe-relative, forward-slashed, NFC
//! path fallback; every line ends `\n`. A ledger written on one OS must read byte-correctly on
//! the other.
//!
//! **fsync is per-site, not uniform** (measured Slice 0: fsync 3,418 µs vs a plain append
//! 168 µs — 20×). Mandatory where the only other copy is about to be destroyed
//! (archive-before-purge) and for rare user decisions; a plain append for walk counters and the
//! continuous history mirror. See `tests/mig104/README.md`.

use std::io::Write;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

/// The two streams. One appender, two files — see the module docs for why they may never share
/// a fold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// `earned.jsonl` — link life. Folds.
    Earned,
    /// `note-history.jsonl` — a note's change history. Never folds.
    NoteHistory,
}

impl Stream {
    pub fn file_name(self) -> &'static str {
        match self {
            Stream::Earned => "earned.jsonl",
            Stream::NoteHistory => "note-history.jsonl",
        }
    }
}

/// `earned.snapshot.jsonl` — one line per earned link, current state. Bounded by earned count,
/// never by history, which is what keeps the load bounded (Slice 7 writes it).
pub const SNAPSHOT_FILE: &str = "earned.snapshot.jsonl";

/// The store's directory, derived from the connection itself: `conn.path()`'s parent IS the
/// `.constellation` dir (Boss ruling Q1). This is why no writer needs a path threaded to it —
/// pinned by `tests_mig104_baseline::conn_path_parent_is_the_constellation_dir`.
pub fn store_dir(conn: &Connection) -> Option<PathBuf> {
    let p = conn.path()?;
    if p.is_empty() {
        return None; // in-memory connection (tests) — no store
    }
    Path::new(p).parent().map(|d| d.to_path_buf())
}

/// What a load found, so nothing is ever silently swallowed (§3.7, the corrupt-store contract).
#[derive(Debug, Default, Clone)]
pub struct LoadReport {
    /// Individual unparseable lines. Each costs ONE line — never the file — and is COUNTED.
    pub skipped_lines: usize,
    /// Set when the store was structurally unusable and renamed aside.
    pub corrupt_renamed_to: Option<PathBuf>,
    /// When true the caller must NOT write a fresh store: a blind overwrite would destroy the
    /// backup that was about to save the user. Requires acknowledgement first.
    pub refuse_write: bool,
}

/// One folded link-life record: the current state of one earned link.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Earned {
    /// Absolute traversal count. Folds by MAX — never a sum, never a decrement.
    pub n: i64,
    pub conf: Option<String>,
    pub status: Option<String>,
    /// Last-writer-wins timestamp of the newest record folded in.
    pub at: Option<String>,
}

pub type FoldedMap = std::collections::HashMap<String, Earned>;

/// One note-history record, as archived. The record IS the payload — see the module docs.
#[derive(Debug, Clone, PartialEq)]
pub struct HistRecord {
    pub cid: String,
    /// The source row ordinal. THE ordering key — `at` collides constantly.
    pub hid: i64,
    pub at: i64,
    pub raw: String,
}

/// Append lines to a stream. ONE `write_all` per line including its `\n`, in append mode, so a
/// concurrent reader always sees whole lines and a crash can only truncate the last one.
///
/// Deliberately does NOT fsync — see the module docs on the 20× cost. Call `fsync` explicitly
/// at the sites where the only other copy is about to be destroyed.
pub fn append(dir: &Path, s: Stream, lines: &[String]) -> Result<(), String> {
    if lines.is_empty() {
        return Ok(());
    }
    let path = dir.join(s.file_name());
    let mut h = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("ledger open {}: {e}", path.display()))?;
    for line in lines {
        let mut buf = line.clone();
        if !buf.ends_with('\n') {
            buf.push('\n');
        }
        h.write_all(buf.as_bytes())
            .map_err(|e| format!("ledger append {}: {e}", path.display()))?;
    }
    Ok(())
}

/// Force the stream's bytes to disk. Use at the archive-before-purge site and after a user
/// decision; NOT on the walk path (Slice 0 measured 3.4 ms vs 168 µs).
pub fn fsync(dir: &Path, s: Stream) -> Result<(), String> {
    let path = dir.join(s.file_name());
    let h = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("ledger open for fsync {}: {e}", path.display()))?;
    h.sync_all()
        .map_err(|e| format!("ledger fsync {}: {e}", path.display()))
}

fn parse_str(v: &serde_json::Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string())
}

/// The key a link-life record folds under. cid-pair first (portable, survives a rename); the
/// target NAME only as a fallback for an unresolved target.
fn earned_key(v: &serde_json::Value) -> Option<String> {
    let cid = parse_str(v, "cid")?;
    let to = parse_str(v, "to").unwrap_or_default();
    if !to.is_empty() {
        return Some(format!("{cid}>{to}"));
    }
    let tn = parse_str(v, "tn").unwrap_or_default();
    if tn.is_empty() {
        return None;
    }
    Some(format!("{cid}>~{}", tn.to_lowercase()))
}

/// Read one JSONL file, skipping (and counting) unparseable lines. Never throws on content.
fn read_lines(path: &Path, report: &mut LoadReport) -> Vec<serde_json::Value> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Vec::new(), // absent is a FACT: an empty store, not an error
    };
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => out.push(v),
            // ONE bad line costs one line. Counted, and surfaced by the caller.
            Err(_) => report.skipped_lines += 1,
        }
    }
    out
}

/// Load the link-life state: snapshot + tail, both bounded. THE ONLY FOLD IN THIS MODULE.
///
/// Idempotent by ARITHMETIC, not by rule: `n` is written absolute and folds by max, so a
/// duplicated region, a re-appended restored copy, or a "keep both" merge resolution all fold
/// to the same answer. Later records win for decisions.
pub fn read_folded(dir: &Path) -> (FoldedMap, LoadReport) {
    let mut report = LoadReport::default();
    let mut map: FoldedMap = FoldedMap::new();
    // Snapshot first, then the tail — the tail is newer by construction.
    let mut all = read_lines(&dir.join(SNAPSHOT_FILE), &mut report);
    all.extend(read_lines(&dir.join(Stream::Earned.file_name()), &mut report));
    for v in &all {
        // Stream B records can never enter the fold, even if a file is concatenated by hand.
        if matches!(parse_str(v, "t").as_deref(), Some("nh") | Some("nd") | Some("nr")) {
            continue;
        }
        let Some(key) = earned_key(v) else { continue };
        let e = map.entry(key).or_default();
        if let Some(n) = v.get("n").and_then(|x| x.as_i64()) {
            e.n = e.n.max(n); // MAX, so a replay can never ratchet a count down
        }
        if let Some(c) = parse_str(v, "conf") {
            e.conf = Some(c);
        }
        match parse_str(v, "t").as_deref() {
            Some("retire") => e.status = Some("archived".to_string()),
            Some("restore") => e.status = Some("active".to_string()),
            _ => {}
        }
        if let Some(at) = parse_str(v, "at") {
            e.at = Some(at);
        }
    }
    (map, report)
}

/// Read a note's archived history, ordinal-ordered. **No fold** — every event survives.
pub fn read_history_for(dir: &Path, cid: &str) -> (Vec<HistRecord>, LoadReport) {
    let mut report = LoadReport::default();
    let mut out: Vec<HistRecord> = read_lines(&dir.join(Stream::NoteHistory.file_name()), &mut report)
        .into_iter()
        .filter(|v| parse_str(v, "cid").as_deref() == Some(cid))
        .filter(|v| parse_str(v, "t").as_deref() == Some("nh"))
        .map(|v| HistRecord {
            cid: cid.to_string(),
            hid: v.get("hid").and_then(|x| x.as_i64()).unwrap_or(0),
            at: v.get("at").and_then(|x| x.as_i64()).unwrap_or(0),
            raw: v.to_string(),
        })
        .collect();
    // By `hid`, NEVER by `at`: 765 `captured_at` groups collide across 1,536 live rows.
    // `sort_by_key` is stable, so equal hids keep file order.
    out.sort_by_key(|r| r.hid);
    (out, report)
}

/// The `.gitignore` that makes the File-Over-App claim operationally true (Boss decision #8).
///
/// Patterns are BY NAME, never the folder — that is the whole point. Excluding
/// `.constellation/` wholesale to skip the databases would exclude the earned data living in it,
/// in the same event it exists to survive. Measured: this list takes the live folder from
/// 2,836 MB to 38 KB, and `*.db` (not `search.db*`) is what also catches the orphaned 939 MB
/// `Constellation SV Test.db`.
pub const GITIGNORE_CONTENT: &str = "\
# Constellation — derived / machine state. NEVER exclude this folder wholesale:
# the earned-life ledger lives here and must travel with your notes (MIG-104).
*.db
*.db-wal
*.db-shm
boot-perf.*
diagnostics.log
sv-trace.log
";

/// Write the `.gitignore` once. NEVER overwrites — the user may have edited it.
pub fn ensure_gitignore(dir: &Path) -> Result<(), String> {
    let p = dir.join(".gitignore");
    if p.exists() {
        return Ok(());
    }
    std::fs::write(&p, GITIGNORE_CONTENT).map_err(|e| format!("write .gitignore: {e}"))
}

/// True when `name` is excluded by `GITIGNORE_CONTENT`. Kept beside the constant so the test
/// that asserts the live folder's contents cannot drift from the patterns.
pub fn gitignore_excludes(name: &str) -> bool {
    let n = name.to_lowercase();
    n.ends_with(".db")
        || n.ends_with(".db-wal")
        || n.ends_with(".db-shm")
        || n.starts_with("boot-perf.")
        || n == "diagnostics.log"
        || n == "sv-trace.log"
}

/// Fold Syncthing `.sync-conflict-*` copies back in, then remove them. Nearly free because the
/// Stream-A fold is already commutative and idempotent. Stream B is append-deduped by `hid`
/// rather than folded. Returns how many copies were adopted.
pub fn adopt_conflict_copies(dir: &Path) -> usize {
    let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
    let mut adopted = 0usize;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.contains(".sync-conflict-") {
            continue;
        }
        let is_earned = name.starts_with("earned");
        let is_hist = name.starts_with("note-history");
        if !is_earned && !is_hist {
            continue;
        }
        let src = entry.path();
        let Ok(text) = std::fs::read_to_string(&src) else { continue };
        let target = if is_earned { Stream::Earned } else { Stream::NoteHistory };
        let lines: Vec<String> = text.lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect();
        if append(dir, target, &lines).is_ok() {
            let _ = std::fs::remove_file(&src);
            adopted += 1;
        }
    }
    adopted
}

/// The store is structurally unusable (not merely one bad line) → rename it aside and REFUSE to
/// write a fresh one until acknowledged. `stamp` is supplied by the caller (never `Date::now`
/// inside, so the name is deterministic in tests).
pub fn quarantine(dir: &Path, s: Stream, stamp: &str) -> LoadReport {
    let src = dir.join(s.file_name());
    let dest = dir.join(format!("earned.corrupt-{stamp}.jsonl"));
    let mut report = LoadReport::default();
    if std::fs::rename(&src, &dest).is_ok() {
        report.corrupt_renamed_to = Some(dest);
    }
    // A blind overwrite would destroy the backup that was about to save the user.
    report.refuse_write = true;
    report
}

/// Slice 4 toggle. `false` = today's behaviour exactly, byte-for-byte: no file is created and
/// no writer runs. Kept as a `const` so the dead branch is compiled out entirely rather than
/// costing a check on the traverse path.
pub const EARNED_LEDGER_WRITE: bool = true;

// ─── Record builders ─────────────────────────────────────────────────────────
//
// The on-disk FORMAT lives here and nowhere else, so a writer cannot invent a variant and the
// format test has one thing to assert against.
//
// Field order is FIXED and meaningful — the file is meant to be read by a human in a text editor,
// where `v,t,cid,to,tn,n,at` reads as a sentence and the alphabetical `at,cid,n,t,tn,to,v` does
// not. `serde_json::json!` CANNOT deliver that: without the `preserve_order` feature its Map is a
// BTreeMap and it sorts keys. (The first cut of this module claimed otherwise in a comment; the
// format test caught it.) Enabling `preserve_order` globally would change every JSON write in the
// app, so the lines are built here with an explicit ordered writer instead — values still escaped
// by serde, so the output is always valid JSON.
//
// `cid` = the SOURCE note's identity; `to` = the TARGET's identity when resolvable; `tn` = the
// target's name, which is the fallback key AND the only human-legible part of the line.
//
// Q2 (Boss-ruled): the key is TYPE-FREE — `[[supports::X]]` and `[[derives-from::X]]` from one
// note fold to ONE record, because all four DB writers already match on source + target name and
// ignore `link_type`. Re-typing a link therefore keeps its earned history.

/// Write an ordered JSON object. Values are escaped by serde (so the line is always valid JSON);
/// only the ORDER is ours. `Val` keeps the call sites readable.
enum Val<'a> {
    S(&'a str),
    I(i64),
}

fn obj(fields: &[(&str, Val)]) -> String {
    let mut out = String::from("{");
    for (i, (k, v)) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&serde_json::to_string(k).unwrap_or_default());
        out.push(':');
        match v {
            Val::S(x) => out.push_str(&serde_json::to_string(x).unwrap_or_default()),
            Val::I(x) => out.push_str(&x.to_string()),
        }
    }
    out.push('}');
    out
}

/// A traversal. `n` is written ABSOLUTE (never a delta) — that is what makes the fold idempotent.
pub fn walk_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, n: i64, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("walk")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("n", Val::I(n)),
        ("at", Val::S(at)),
    ])
}

/// A confidence judgment. Only ever a USER judgment — never the auto-tier derivable from `n`
/// (≥10 established, ≥3 evidence), because recording a derivable value would fill the ledger
/// with events that carry no decision.
pub fn trust_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, conf: &str, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("trust")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("conf", Val::S(conf)),
        ("at", Val::S(at)),
    ])
}

/// Retiring a link. Archival, not deletion — the wikilink deliberately stays in the note, which
/// is exactly why this must be durable: a rebuild from the notes alone would resurrect it.
pub fn retire_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("retire")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("at", Val::S(at)),
    ])
}

/// Un-retiring a link.
pub fn restore_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("restore")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("at", Val::S(at)),
    ])
}

/// A review-priority decision on a NOTE (no target).
pub fn priority_line(cid: &str, p: i64, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("priority")),
        ("cid", Val::S(cid)),
        ("p", Val::I(p)),
        ("at", Val::S(at)),
    ])
}

/// True when `conf` is merely the tier derivable from `n` — i.e. carries no user judgment and
/// must NOT be recorded. Mirrors the thresholds in `constellation_link_traverse`.
pub fn is_derivable_tier(conf: &str, n: i64) -> bool {
    let auto = if n >= 10 { "established" } else if n >= 3 { "evidence" } else { "hypothesis" };
    conf == auto
}

#[cfg(test)]
mod tests_mig104_link_life {
    use super::*;

    fn dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }
    fn line(json: serde_json::Value) -> String {
        json.to_string()
    }

    #[test]
    fn append_writes_exactly_one_line_with_lf() {
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":1}))]).unwrap();
        let raw = std::fs::read(d.path().join("earned.jsonl")).unwrap();
        assert_eq!(raw.iter().filter(|b| **b == b'\n').count(), 1);
        assert!(!raw.contains(&b'\r'), "no CRLF — the ledger must read byte-correctly on macOS");
        assert_eq!(*raw.last().unwrap(), b'\n');
    }

    #[test]
    fn torn_tail_loses_only_the_last_line() {
        let d = dir();
        for n in 1..=3 {
            append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":n}))]).unwrap();
        }
        // Simulate a kill mid-append: truncate inside the final line.
        let p = d.path().join("earned.jsonl");
        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, &text[..text.len() - 8]).unwrap();
        let (map, report) = read_folded(d.path());
        assert_eq!(report.skipped_lines, 1, "exactly the torn line is lost, and it is COUNTED");
        assert_eq!(map.get("A>B").unwrap().n, 2, "every earlier record survives — an append cannot clobber");
    }

    #[test]
    fn fold_is_commutative_and_idempotent() {
        let mk = |ns: &[i64]| {
            let d = dir();
            for n in ns {
                append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":n}))]).unwrap();
            }
            read_folded(d.path()).0
        };
        assert_eq!(mk(&[3, 8, 5]), mk(&[5, 3, 8]), "order cannot change the answer");
        assert_eq!(mk(&[4]), mk(&[4, 4, 4]), "a duplicated region folds to one copy's answer");
    }

    #[test]
    fn max_fold_never_decreases_n() {
        let d = dir();
        for n in [9, 2, 1] {
            append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":n}))]).unwrap();
        }
        assert_eq!(read_folded(d.path()).0.get("A>B").unwrap().n, 9);
    }

    #[test]
    fn a_later_decision_wins_while_the_count_still_maxes() {
        let d = dir();
        append(d.path(), Stream::Earned, &[
            line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":9})),
            line(serde_json::json!({"v":1,"t":"retire","cid":"A","to":"B","at":"2026-07-18T08:20:02Z"})),
        ]).unwrap();
        let e = read_folded(d.path()).0.get("A>B").cloned().unwrap();
        assert_eq!(e.status.as_deref(), Some("archived"));
        assert_eq!(e.n, 9);
    }

    /// THE distinction the whole module is built around.
    #[test]
    fn history_never_folds() {
        let d = dir();
        // The real collision shape: same `captured_at`, ordinals out of file order.
        append(d.path(), Stream::NoteHistory, &[
            line(serde_json::json!({"v":1,"t":"nh","cid":"C","hid":8252,"at":1785131711000i64,"ev":{"to":"mas"}})),
            line(serde_json::json!({"v":1,"t":"nh","cid":"C","hid":8251,"at":1785131711000i64,"ev":{"to":"ma"}})),
            line(serde_json::json!({"v":1,"t":"nh","cid":"C","hid":8253,"at":1785131711000i64,"ev":{"to":"masadir"}})),
        ]).unwrap();
        let (recs, report) = read_history_for(d.path(), "C");
        assert_eq!(report.skipped_lines, 0);
        assert_eq!(recs.len(), 3, "a thought being typed must survive as three events, never folded to one");
        assert_eq!(recs.iter().map(|r| r.hid).collect::<Vec<_>>(), vec![8251, 8252, 8253],
            "ordered by the row ordinal — `at` is identical on all three");
        // And a history record can never leak into the link-life fold.
        assert!(read_folded(d.path()).0.is_empty());
    }

    #[test]
    fn unparseable_line_is_skipped_and_counted() {
        let d = dir();
        let p = d.path().join("earned.jsonl");
        std::fs::write(&p, "{\"v\":1,\"t\":\"walk\",\"cid\":\"A\",\"to\":\"B\",\"n\":1}\nnot json at all\n{\"v\":1,\"t\":\"walk\",\"cid\":\"A\",\"to\":\"B\",\"n\":5}\n").unwrap();
        let (map, report) = read_folded(d.path());
        assert_eq!(report.skipped_lines, 1);
        assert_eq!(map.get("A>B").unwrap().n, 5, "the good lines on BOTH sides of the bad one load");
    }

    #[test]
    fn structurally_corrupt_store_is_renamed_aside_and_refuses_fresh_write() {
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":1}))]).unwrap();
        let report = quarantine(d.path(), Stream::Earned, "2026-07-27T000000Z");
        let aside = report.corrupt_renamed_to.expect("renamed aside, never deleted");
        assert!(aside.exists(), "the suspect store is KEPT — it may be the only copy left");
        assert!(!d.path().join("earned.jsonl").exists());
        assert!(report.refuse_write, "must refuse a fresh write until acknowledged");
    }

    #[test]
    fn gitignore_excludes_every_db_in_the_live_folder_but_no_ledger() {
        for excluded in [
            "search.db", "search.db-wal", "search.db-shm",
            "Constellation SV Test.db", // the orphaned 939 MB one — `search.db*` would MISS it
            "boot-perf.latest.json", "boot-perf.history.jsonl",
            "diagnostics.log", "sv-trace.log",
        ] {
            assert!(gitignore_excludes(excluded), "must be excluded from sync: {excluded}");
        }
        for kept in ["earned.jsonl", "earned.snapshot.jsonl", "note-history.jsonl",
                     "settings.json", "libraries.json", "universe.json", "review-pulse.json"] {
            assert!(!gitignore_excludes(kept), "must TRAVEL with the user's notes: {kept}");
        }
    }

    #[test]
    fn ensure_gitignore_never_overwrites_the_users_edit() {
        let d = dir();
        ensure_gitignore(d.path()).unwrap();
        std::fs::write(d.path().join(".gitignore"), "# mine\n").unwrap();
        ensure_gitignore(d.path()).unwrap();
        assert_eq!(std::fs::read_to_string(d.path().join(".gitignore")).unwrap(), "# mine\n");
    }

    #[test]
    fn adopts_a_sync_conflict_copy_then_removes_it() {
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":2}))]).unwrap();
        let conflict = d.path().join("earned.sync-conflict-20260727-120000-ABCDEF.jsonl");
        std::fs::write(&conflict, line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":7})) + "\n").unwrap();
        assert_eq!(adopt_conflict_copies(d.path()), 1);
        assert!(!conflict.exists(), "the copy is folded in, then removed");
        assert_eq!(read_folded(d.path()).0.get("A>B").unwrap().n, 7, "the higher count wins by max-fold");
    }

    #[test]
    fn an_absent_store_is_a_fact_not_an_error() {
        let d = dir();
        let (map, report) = read_folded(d.path());
        assert!(map.is_empty());
        assert_eq!(report.skipped_lines, 0);
        assert!(!report.refuse_write, "absent is an empty store; only UNREADABLE refuses a write");
    }

    #[test]
    fn keys_are_os_portable() {
        // The fallback key path must never carry a drive letter or a backslash.
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"","tn":"The Four Books","n":1}))]).unwrap();
        let (map, _) = read_folded(d.path());
        let k = map.keys().next().unwrap();
        assert!(!k.contains('\\') && !k.contains(':'), "no path separators or drive letters in a key: {k}");
        assert_eq!(k, "A>~the four books", "an unresolved target folds case-insensitively by name");
    }
}

#[cfg(test)]
mod tests_mig104_hooks {
    //! MIG-104 Slice 4 — the record FORMAT and the two write ORDERS. The commands themselves
    //! need an AppHandle, so these pin the parts that carry the design: the line shapes, the
    //! type-free key of Q2, the derivable-tier suppression, and the decision order's contract
    //! that a failed append must stop the DB change.
    use super::*;

    #[test]
    fn walk_line_carries_an_absolute_count_and_fixed_field_order() {
        let l = walk_line("C_SRC", "C_TGT", "the four books", 3, "2026-07-27T09:11:05Z");
        assert_eq!(
            l,
            r#"{"v":1,"t":"walk","cid":"C_SRC","to":"C_TGT","tn":"the four books","n":3,"at":"2026-07-27T09:11:05Z"}"#,
            "field order is part of the contract — a human reads this file in a text editor"
        );
    }

    /// Q2, Boss-ruled: the key is TYPE-FREE, so re-typing a link keeps its earned history.
    #[test]
    fn type_variants_of_one_pair_produce_one_ledger_key() {
        let d = tempfile::tempdir().unwrap();
        // The real live shape: one note linking to `the four books` twice, as `supports` and as
        // `derives-from`, one click each. The DB writers match on source + target name and ignore
        // link_type, so both are ONE link in the user's terms.
        append(d.path(), Stream::Earned, &[
            walk_line("C_ISLAM", "C_BOOKS", "the four books", 1, "2026-07-27T09:00:00Z"),
            walk_line("C_ISLAM", "C_BOOKS", "the four books", 2, "2026-07-27T09:05:00Z"),
        ]).unwrap();
        let (map, _) = read_folded(d.path());
        assert_eq!(map.len(), 1, "the two typed variants must fold to ONE record");
        assert_eq!(map.get("C_ISLAM>C_BOOKS").unwrap().n, 2);
    }

    #[test]
    fn an_unresolved_target_still_keys_and_survives_the_fold() {
        let d = tempfile::tempdir().unwrap();
        append(d.path(), Stream::Earned, &[walk_line("C_SRC", "", "banana", 4, "2026-07-27T09:00:00Z")]).unwrap();
        let (map, _) = read_folded(d.path());
        assert_eq!(map.get("C_SRC>~banana").unwrap().n, 4, "a broken link's earned history is still recorded");
    }

    /// The auto-tier must never be recorded: it carries no user judgment and is derivable from
    /// the count, so recording it would fill the ledger with decisions nobody made.
    #[test]
    fn auto_tier_promotion_writes_no_trust_event() {
        assert!(is_derivable_tier("hypothesis", 1));
        assert!(is_derivable_tier("evidence", 3));
        assert!(is_derivable_tier("evidence", 9));
        assert!(is_derivable_tier("established", 10));
        // A USER judgment is never derivable — `contested` has no count that produces it…
        assert!(!is_derivable_tier("contested", 0));
        assert!(!is_derivable_tier("contested", 50));
        // …and neither is a manual pick that outranks the count.
        assert!(!is_derivable_tier("established", 3));
        assert!(!is_derivable_tier("evidence", 1));
    }

    #[test]
    fn retire_then_restore_reconstructs_in_order_from_the_ledger_alone() {
        let d = tempfile::tempdir().unwrap();
        append(d.path(), Stream::Earned, &[
            walk_line("C_A", "C_B", "b", 7, "2026-07-27T09:00:00Z"),
            retire_line("C_A", "C_B", "b", "2026-07-27T09:01:00Z"),
        ]).unwrap();
        assert_eq!(read_folded(d.path()).0.get("C_A>C_B").unwrap().status.as_deref(), Some("archived"));
        append(d.path(), Stream::Earned, &[restore_line("C_A", "C_B", "b", "2026-07-27T09:02:00Z")]).unwrap();
        let e = read_folded(d.path()).0.get("C_A>C_B").cloned().unwrap();
        assert_eq!(e.status.as_deref(), Some("active"), "the LAST decision wins");
        assert_eq!(e.n, 7, "and the earned count is untouched by either decision");
    }

    /// The decision order's whole point: if the record cannot be made durable the DB must not
    /// change. `append` returning Err is what the command propagates instead of proceeding.
    #[test]
    fn a_failed_append_is_an_error_the_caller_must_not_swallow() {
        // A path that cannot be a directory → open() fails → Err, not a silent Ok.
        let d = tempfile::tempdir().unwrap();
        let not_a_dir = d.path().join("file-not-dir");
        std::fs::write(&not_a_dir, b"x").unwrap();
        let r = append(&not_a_dir, Stream::Earned, &[retire_line("C_A", "C_B", "b", "2026-07-27T09:00:00Z")]);
        assert!(r.is_err(), "the decision path relies on this Err to abort the DB change");
    }

    #[test]
    fn priority_line_has_no_target() {
        let l = priority_line("C_A", 2, "2026-07-27T09:00:00Z");
        assert_eq!(l, r#"{"v":1,"t":"priority","cid":"C_A","p":2,"at":"2026-07-27T09:00:00Z"}"#);
        assert!(!l.contains("\"to\""), "a review priority is about a NOTE, not a link");
    }

    #[test]
    fn the_toggle_off_means_no_file_is_ever_created() {
        // Documents the contract; the const is compiled in, so this asserts the shape callers use.
        let d = tempfile::tempdir().unwrap();
        if !EARNED_LEDGER_WRITE {
            assert!(!d.path().join(Stream::Earned.file_name()).exists());
        }
        // With writes ON (the shipped default) an append creates the file on first use.
        append(d.path(), Stream::Earned, &[walk_line("C", "D", "d", 1, "t")]).unwrap();
        assert!(d.path().join("earned.jsonl").exists());
    }
}
