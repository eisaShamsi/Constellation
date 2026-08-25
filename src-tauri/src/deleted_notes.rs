// ─── PJ-385: reading the delete archive back ─────────────────────────────────────────────
//
// # Concept
//
// *When Constellation destroys something permanently, the person must be able to see what it
// destroyed.*
//
// Every delete in this app writes an envelope here before anything is purged, and refuses to
// purge if that write fails. That guarantee has been true and useless in the same breath: the
// only reader, `read_history_for`, takes a cid the caller must already know and returns only the
// `nh` change-events — it cannot enumerate what was deleted at all. So the app could say "its
// history was kept" while offering no way to look at it.
//
// The Boss ruled on 2026-08-25, when asked whether to proceed with a 603-row removal or build
// this first: **build this first.** He is right that a record nobody can read is not a record.

/// One archived deletion, as the list needs it. The body is NOT carried here — a universe with
/// thousands of deletions would otherwise serialise every note's full text to draw a list.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedNote {
    /// Content id — the archive's key, and what `note_history` events are joined on.
    pub cid: String,
    /// Milliseconds since the epoch, as recorded at the moment of deletion.
    pub at: i64,
    /// Where the note lived when it was deleted.
    pub path: String,
    pub name: String,
    pub library: String,
    /// `trash` | `system_trash` | `permanent` | `vanished` | `reconcile_gone` | `phantom_prune`.
    /// Rendered by the frontend, which owns the user's language.
    pub reason: String,
    /// Where the file went, when that was knowable (the de-collided trash destination).
    pub dest: Option<String>,
    /// Characters of note text held in the archive. Zero is a real answer, not a failure: the
    /// archive keeps what `note_meta.body_text` held, which is empty for a note with nothing
    /// beyond its frontmatter — measured on `Eisa Universe`, 101 of 2,731 rows carry no body
    /// text, of which **99** have a file on disk (all 99 opened: frontmatter and nothing after
    /// it). An earlier version of this line said "every one of them a file still on disk"; the
    /// other **2** are `…\Eisa Test\.trash\Collision Test.md` and `…\Eisa Test\Town Eisa v2.md`
    /// — which are exactly the 2 of the 603 phantoms that carry no text, which is why the error
    /// was invisible inside its own arithmetic.
    ///
    /// It does NOT mean the file was missing. An earlier version of this comment said a phantom
    /// has no text "because its file was already gone when it was indexed", and the measurement
    /// is the other way round: **601 of the 603 phantoms carry body text — 20,484,230 characters,
    /// median 18,984** (that is the median of the 601; the median of all 603 is 18,944, and an
    /// earlier version of this line attached the 603's figure to the 601). The premise was
    /// backwards, and it had propagated into the user-facing explanation before it was caught.
    ///
    /// What the archive keeps is a SEARCH RENDERING, never the file: `note_meta.body_text` is
    /// `strip_markdown` (`search.rs:8234`) then `normalize_arabic_for_search` (`:8274`).
    /// Measured against 60 notes still on disk: 0 byte-identical, heading markers gone in 60,
    /// fenced code gone in 59, `[[...]]` brackets gone in 49, Arabic diacritics reduced in 60.
    pub body_chars: usize,
    /// How many change-history events were archived alongside it.
    pub history_events: usize,
}

/// What a read of the archive found, including what it could NOT read.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedNotesPage {
    pub notes: Vec<DeletedNote>,
    /// Total envelopes in the file, before any limit was applied.
    pub total: usize,
    /// Lines the parser could not read. Surfaced rather than swallowed: an archive that is
    /// partly unreadable must not look like a complete one (the `LoadReport` contract).
    pub unreadable_lines: usize,
    /// False when the archive file does not exist at all — distinct from "exists and is empty",
    /// which is a real answer.
    pub archive_present: bool,
}

/// Every deletion this universe has archived, newest first.
///
/// Read-only. It opens no database and writes nothing; the archive is a plain append-only file
/// beside `search.db`, which is what lets it be read after the rows it describes are gone.
#[tauri::command(async)]
pub fn deleted_notes_list(app: tauri::AppHandle, limit: Option<usize>) -> Result<DeletedNotesPage, String> {
    let dir = archive_dir(&app)?;
    let file = dir.join("note-history.jsonl");
    let archive_present = file.exists();

    let mut report = crate::link_life::LoadReport::default();
    let lines = crate::link_life::read_archive_lines(&file, &mut report);

    // 2026-08-25 inspection, HIGH false-success — REFUSE rather than report an empty archive.
    // The file exists (`archive_present` is true) and could not be read at all. Falling through
    // would return `{ notes: [], total: 0, unreadableLines: 0, archivePresent: true }`, which the
    // UI renders as "The record exists and is empty — no removal has been recorded in this
    // universe": a definite statement of fact about the LAST surviving record of notes whose
    // files are gone, produced by a failure to open the file. `unreadableLines` cannot rescue it
    // — that banner only renders when `total > 0`. The frontend's documented contract is that
    // this call THROWS on failure, and this is the failure it was written for.
    if let Some(err) = report.unreadable_file {
        return Err(format!(
            "the record file exists but could not be read: {err} — this is a read failure, not an empty record; nothing has been changed"
        ));
    }

    let page = parse_archive(&lines, report.skipped_lines, archive_present, limit);
    Ok(page)
}

/// Turn archive lines into the page the UI renders.
///
/// Extracted 2026-08-25 because the first version's TEST re-implemented this parsing, so fixing
/// the real one left the copy asserting the old behaviour — the same shape that has bitten this
/// codebase repeatedly today. One parser, called by the command and by every test.
pub(crate) fn parse_archive(
    lines: &[serde_json::Value],
    unreadable_lines: usize,
    archive_present: bool,
    limit: Option<usize>,
) -> DeletedNotesPage {
    // Walk the file IN ORDER, attributing each change-event to the deletion it was written with.
    //
    // 2026-08-25 inspection — the first version tallied events per CID across the whole file, so a
    // note deleted twice (a sync agent removing and re-adding a file makes the watcher archive a
    // `vanished` envelope, and the same frontmatter cid is re-indexed afterwards) had BOTH of its
    // rows quote the combined count. `build_delete_archive` emits the envelope and then its own
    // `nh` records in one append, so file order is the attribution — no timestamp arithmetic, no
    // guessing.
    let mut notes: Vec<DeletedNote> = Vec::new();
    for v in lines {
        match v.get("t").and_then(|x| x.as_str()) {
            Some("del") => {
                let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
                notes.push(DeletedNote {
                    body_chars: v.get("body").and_then(|x| x.as_str()).map(|b| b.chars().count()).unwrap_or(0),
                    history_events: 0,
                    at: v.get("at").and_then(|x| x.as_i64()).unwrap_or(0),
                    path: s("path"),
                    name: s("name"),
                    library: s("library"),
                    reason: s("reason"),
                    dest: v.get("dest").and_then(|x| x.as_str()).map(|d| d.to_string()),
                    cid: s("cid"),
                });
            }
            Some("nh") => {
                // Belongs to the most recent envelope for the SAME cid — the one it was appended
                // with. An `nh` with no preceding envelope of its cid is an orphan and is not
                // attributed to some unrelated deletion just because it is nearby.
                let cid = v.get("cid").and_then(|x| x.as_str()).unwrap_or("");
                if let Some(owner) = notes.iter_mut().rev().find(|n| n.cid == cid) {
                    owner.history_events += 1;
                }
            }
            _ => {}
        }
    }

    // Newest first: the thing a person wants after a removal is the removal they just did.
    notes.sort_by(|a, b| b.at.cmp(&a.at));
    let total = notes.len();
    if let Some(n) = limit {
        // `limit` bounds the PAYLOAD, not the work: the whole append-only file is read and parsed
        // first, and `maybe_compact` deliberately never folds this stream, so it only grows.
        // Named rather than hidden (2026-08-25 inspection). Acceptable today — the live archives
        // are single-digit KB and even a 603-entry prune adds 603 lines — and `total` above always
        // reports the true count, so a truncated page can never read as a complete one. If this
        // stream ever reaches a size where the read is felt, the fix is to stream from the tail
        // backwards rather than to quietly cap what is shown.
        notes.truncate(n);
    }

    DeletedNotesPage { notes, total, unreadable_lines, archive_present }
}

/// The archived TEXT of one deleted note, by content id.
///
/// Separate from the list on purpose (see `DeletedNote::body_chars`): the list must stay cheap
/// enough to open on a universe with thousands of deletions, and the body is only wanted for the
/// one the user clicked.
///
/// Returns `None` when the cid is not in the archive, or its envelope carried no body.
///
/// An earlier version of this line added "— which is the honest answer for a phantom, whose file
/// was already gone before it was ever archived." **That is false and is recorded here so it is
/// not written again.** The body comes from `note_meta.body_text`, never from the file
/// (`reindex_delete_note`, `search.rs:12796`, performs no filesystem read of the note), and
/// `index_note_impl` returns `Skipped` when the path does not exist — so a row cannot come into
/// existence for a file that is gone. An empty body means the INDEX held no text. On the live
/// data, 601 of the 603 phantoms carry text.
#[tauri::command(async)]
pub fn deleted_note_body(app: tauri::AppHandle, cid: String, at: i64) -> Result<Option<String>, String> {
    let file = archive_dir(&app)?.join("note-history.jsonl");
    let mut report = crate::link_life::LoadReport::default();
    let lines = crate::link_life::read_archive_lines(&file, &mut report);
    // Same refusal as `deleted_notes_list`, for the same reason and with more at stake: `Ok(None)`
    // renders as "The record kept no text for this note", while a rejection renders
    // `settings.deleted.readFailed` — "this is a read failure, not an empty entry". That
    // distinction was deliberately built into the UI and a swallowed read error bypassed it.
    if let Some(err) = report.unreadable_file {
        return Err(format!(
            "the record file exists but could not be read: {err} — this is a read failure, not an empty entry"
        ));
    }
    Ok(lines
        .into_iter()
        .filter(|v| v.get("t").and_then(|x| x.as_str()) == Some("del"))
        // 2026-08-25 inspection — addressed by cid AND time, not cid alone. The first version took
        // the LAST envelope for a cid, so a note deleted more than once showed the newest
        // envelope's text under every one of its rows: a row could advertise "12,000 characters
        // kept" and then display none, or — worse — display a DIFFERENT deletion's text as though
        // it were what was destroyed. On the last surviving copy of a destroyed note, silently.
        .filter(|v| {
            v.get("cid").and_then(|x| x.as_str()) == Some(cid.as_str())
                && v.get("at").and_then(|x| x.as_i64()) == Some(at)
        })
        .next_back()
        .and_then(|v| v.get("body").and_then(|x| x.as_str()).map(|b| b.to_string())))
}

/// The folder the archive lives in — beside `search.db`, so it travels with the notes.
fn archive_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let db = crate::search::db_path(app)?;
    db.parent()
        .map(|d| d.to_path_buf())
        .ok_or_else(|| "could not resolve the universe's .constellation folder".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Reads a temp archive through the REAL parser. Not a copy of it — a copy is what made the
    /// first version of `a_note_deleted_twice_keeps_its_two_deletions_apart` fail against fixed
    /// production code.
    fn page_from(dir: &std::path::Path) -> DeletedNotesPage {
        let file = dir.join("note-history.jsonl");
        let archive_present = file.exists();
        let mut report = crate::link_life::LoadReport::default();
        let lines = crate::link_life::read_archive_lines(&file, &mut report);
        parse_archive(&lines, report.skipped_lines, archive_present, None)
    }

    #[test]
    fn an_absent_archive_is_a_fact_not_an_error() {
        // "Nothing has ever been deleted here" is a real answer, and must be distinguishable
        // from "the archive is missing" — which is why `archive_present` exists at all.
        let d = TempDir::new().unwrap();
        let p = page_from(d.path());
        assert!(!p.archive_present);
        assert_eq!(p.total, 0);
        assert_eq!(p.unreadable_lines, 0);
    }

    #[test]
    fn one_unreadable_line_costs_one_line_and_is_counted() {
        // The archive is the last copy of a destroyed note. A single corrupt line must not
        // discard the file, and must not pass silently either.
        let d = TempDir::new().unwrap();
        let mut f = std::fs::File::create(d.path().join("note-history.jsonl")).unwrap();
        writeln!(f, r#"{{"v":1,"t":"del","cid":"A","at":10,"path":"E:/x/A.md","name":"A","library":"L","reason":"trash","body":"hello"}}"#).unwrap();
        writeln!(f, "{{ this is not json").unwrap();
        writeln!(f, r#"{{"v":1,"t":"del","cid":"B","at":20,"path":"E:/x/B.md","name":"B","library":"L","reason":"permanent","body":"bodyB"}}"#).unwrap();
        drop(f);

        let p = page_from(d.path());
        assert_eq!(p.total, 2, "the good lines survive a bad one");
        assert_eq!(p.unreadable_lines, 1, "and the bad one is COUNTED, never swallowed");
        assert_eq!(p.notes[0].cid, "B", "newest first");
        assert_eq!(p.notes[0].reason, "permanent");
        assert_eq!(p.notes[1].body_chars, 5);
    }

    #[test]
    fn history_events_are_attributed_to_their_own_deletion() {
        let d = TempDir::new().unwrap();
        let mut f = std::fs::File::create(d.path().join("note-history.jsonl")).unwrap();
        writeln!(f, r#"{{"v":1,"t":"del","cid":"A","at":10,"path":"E:/x/A.md","name":"A","library":"L","reason":"trash","body":"a"}}"#).unwrap();
        writeln!(f, r#"{{"v":1,"t":"nh","cid":"A","hid":1,"at":9,"ev":{{}}}}"#).unwrap();
        writeln!(f, r#"{{"v":1,"t":"nh","cid":"A","hid":2,"at":9,"ev":{{}}}}"#).unwrap();
        writeln!(f, r#"{{"v":1,"t":"del","cid":"B","at":20,"path":"E:/x/B.md","name":"B","library":"L","reason":"trash","body":"b"}}"#).unwrap();
        drop(f);

        let p = page_from(d.path());
        let a = p.notes.iter().find(|n| n.cid == "A").unwrap();
        let b = p.notes.iter().find(|n| n.cid == "B").unwrap();
        assert_eq!(a.history_events, 2);
        assert_eq!(b.history_events, 0, "B's count must not inherit A's");
    }

    #[test]
    fn a_phantom_envelope_with_no_body_reports_zero_not_a_failure() {
        // An envelope can carry no body — a note with nothing beyond its frontmatter indexes an
        // empty `body_text`. That must read as "no text kept", never as an error. (Note this is
        // NOT the phantom case: 601 of the Boss's 603 phantoms DO carry text. The test keeps its
        // name for continuity, but the reason in the old comment was backwards.)
        let d = TempDir::new().unwrap();
        let mut f = std::fs::File::create(d.path().join("note-history.jsonl")).unwrap();
        writeln!(f, r#"{{"v":1,"t":"del","cid":"P","at":30,"path":"E:/gone/P.md","name":"P","library":"L","reason":"phantom_prune"}}"#).unwrap();
        drop(f);
        let p = page_from(d.path());
        assert_eq!(p.total, 1);
        assert_eq!(p.notes[0].body_chars, 0);
        assert_eq!(p.notes[0].reason, "phantom_prune");
    }

    #[test]
    fn a_note_deleted_twice_keeps_its_two_deletions_apart() {
        // The 2026-08-25 inspection's finding. A sync agent removing and re-adding a file makes
        // the watcher archive a `vanished` envelope; the same note can be deleted again later.
        // Both envelopes share a cid, and the first version attributed every `nh` row to the cid
        // rather than to the deletion it was written with — so both rows quoted the combined
        // count, and both showed the newest envelope's text.
        let d = TempDir::new().unwrap();
        let mut f = std::fs::File::create(d.path().join("note-history.jsonl")).unwrap();
        // First deletion: has text, and two change-events.
        writeln!(f, r#"{{"v":1,"t":"del","cid":"X","at":100,"path":"E:/x/X.md","name":"X","library":"L","reason":"vanished","body":"the first text"}}"#).unwrap();
        writeln!(f, r#"{{"v":1,"t":"nh","cid":"X","hid":1,"at":99,"ev":{{}}}}"#).unwrap();
        writeln!(f, r#"{{"v":1,"t":"nh","cid":"X","hid":2,"at":99,"ev":{{}}}}"#).unwrap();
        // Second deletion of the SAME note: no text kept, one event.
        writeln!(f, r#"{{"v":1,"t":"del","cid":"X","at":200,"path":"E:/x/X.md","name":"X","library":"L","reason":"permanent"}}"#).unwrap();
        writeln!(f, r#"{{"v":1,"t":"nh","cid":"X","hid":3,"at":199,"ev":{{}}}}"#).unwrap();
        drop(f);

        let p = page_from(d.path());
        assert_eq!(p.total, 2, "two deletions, two rows");
        let newest = &p.notes[0];
        let oldest = &p.notes[1];
        assert_eq!(newest.at, 200);
        assert_eq!(oldest.at, 100);
        assert_eq!(
            oldest.history_events, 2,
            "the first deletion keeps its own two events, not the combined three"
        );
        assert_eq!(newest.history_events, 1, "and the second keeps its one");
        assert_eq!(oldest.body_chars, 14, "the older row advertises ITS text");
        assert_eq!(newest.body_chars, 0, "the newer row kept none, and says so");
    }

    /// Against the Boss's REAL archive. Ignored by default (machine-specific); skips if absent.
    #[test]
    #[ignore]
    fn live_reads_the_real_archives() {
        for uni in ["Eisa Universe", "Eisa Cognitive Knowledge"] {
            let dir = std::path::Path::new(r"E:\Constellation Universes").join(uni).join(".constellation");
            if !dir.join("note-history.jsonl").exists() {
                println!("SKIP {} — no archive", uni);
                continue;
            }
            let p = page_from(&dir);
            println!(
                "{}: {} deletion(s), {} unreadable line(s), present={}",
                uni, p.total, p.unreadable_lines, p.archive_present
            );
            for n in p.notes.iter().take(4) {
                println!("   [{}] {} — {} chars, {} history event(s)  ({})",
                         n.reason, n.name, n.body_chars, n.history_events, n.path);
            }
            assert_eq!(p.unreadable_lines, 0, "{}'s archive should parse cleanly", uni);
        }
    }
}
