//! PJ-454 — **the mold that was given a birthday.**
//!
//! Boss ruling (MIG-TPL §1, 2026-07-19, restated 2026-09-01): *a template never carries a
//! `cid_cn`. A template is a MOLD; identity and birth belong to the CAST* — because a note cast
//! from a stamped mold would inherit the mold's birth date.
//!
//! `canonical::is_template_file` now stops any NEW stamp. This module repairs the ones already on
//! disk. It is deliberately a separate, explicit, user-approved operation rather than anything
//! automatic: the whole hazard here is misidentifying a real note, and stripping identity from a
//! real note silently severs its earned reading history and leaves links pointing at a dead
//! identity, with nothing shown on screen.
//!
//! ## The identification rule, and why it is this narrow
//!
//! A file qualifies only when its LEADING frontmatter fence holds BOTH a root `cid_cn:` and a
//! template placeholder (`{{ … }}` or Templater's `<% … %>`). **The rule never matches the word
//! "template"**, and that is not fastidiousness — it is measured. During this investigation four
//! different counts were produced on the Boss's own disk:
//!
//! | count | rule | why it was wrong |
//! |---|---|---|
//! | 102 | any stamped file in a template-*named* folder | also swept ~107 genuine notes — `قوالب` is the ordinary Arabic word for *mold*, so it caught a concrete-formwork note |
//! | 67 | braces anywhere in the first 25 lines | matched body text, not frontmatter |
//! | 0 | the app's own two questions (declares `kind: template` / sits in the configured folder) | not one real mold answers either |
//! | **43** | **this rule** | independently reproduced twice, exactly |
//!
//! ## Why both edits, always together
//!
//! Removing the stamp alone does not hold. The boot healer (`search.rs` MIG-003 step 3) probes for
//! notes with an empty `cid_cn` and re-injects one, exempting only files that declare
//! `kind: template` or sit under the configured templates folder — and the Boss's molds live in
//! per-domain Arabic folders, not the configured one. **Without the marker the repair silently
//! undoes itself on the next launch.** So the two edits land in ONE write, or not at all.

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One file the strict rule identifies, with the evidence a human needs to judge it.
#[derive(Debug, Clone, Serialize)]
pub struct MoldCandidate {
    pub path: String,
    pub library: String,
    /// The identity stamp it currently carries.
    pub cid_cn: String,
    /// The birth date that stamp claims, decoded — the thing that would be inherited.
    pub stamp_date: String,
    /// Which placeholder syntax proved it a mold.
    pub syntax: String,
    /// Notes linking to it BY IDENTITY. These need re-reading after the repair or they are left
    /// pointing at an identity nothing holds.
    pub inbound_cid_links: u32,
}

/// What happened to one file.
#[derive(Debug, Clone, Serialize)]
pub struct MoldRepairOutcome {
    pub path: String,
    pub ok: bool,
    /// Plain-language detail — the reason on failure, the confirmation on success.
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoldRepairReport {
    pub attempted: usize,
    pub repaired: usize,
    pub failed: usize,
    pub backup_dir: String,
    pub outcomes: Vec<MoldRepairOutcome>,
    /// Notes re-read because they linked to a repaired mold by identity.
    pub relinked_sources: Vec<String>,
}

// ─── The rule (pure, so it is testable without a disk) ───────────────────────

/// Split a note into (everything before the fence, the fence's inner text, the rest from `\n---`).
///
/// Returns `None` when there is no closing fence — an unterminated `---` is not frontmatter, and
/// treating it as such is how a body gets mangled.
fn split_frontmatter(content: &str) -> Option<(&str, &str, &str)> {
    let trimmed = content.trim_start();
    let lead_len = content.len() - trimmed.len();
    let after = trimmed.strip_prefix("---")?;
    let end = after.find("\n---")?;
    Some((&content[..lead_len], &after[..end], &after[end..]))
}

/// The template placeholder syntaxes. Templater's `<% %>` matters: exactly one of the Boss's molds
/// is identified only by it (`Templater Template (up, related, created).md`), so a rule that knew
/// only about `{{ }}` would have left it stamped.
fn placeholder_syntax(fm: &str) -> Option<&'static str> {
    if fm.contains("{{") && fm.contains("}}") {
        Some("{{ }}")
    } else if fm.contains("<%") && fm.contains("%>") {
        Some("<% %>")
    } else {
        None
    }
}

/// The root `cid_cn` value, if the fence carries one.
///
/// Root only — `is_top_level_key_line` — for the same reason `canonical::frontmatter_declares_template`
/// is root-only: indentation is data, and a nested `cid_cn:` belongs to some other property.
fn root_cid_cn(fm: &str) -> Option<String> {
    fm.lines().find_map(|line| {
        if !crate::yaml_lines::is_top_level_key_line(line) {
            return None;
        }
        let rest = line.trim_end().strip_prefix("cid_cn:")?;
        let v = rest.trim().trim_matches(|c| c == '"' || c == '\'');
        (!v.is_empty()).then(|| v.to_string())
    })
}

/// **The identification rule.** `Some((stamp, syntax))` when this content is a stamped mold.
pub(crate) fn mold_evidence(content: &str) -> Option<(String, &'static str)> {
    let (_, fm, _) = split_frontmatter(content)?;
    let cid = root_cid_cn(fm)?;
    let syntax = placeholder_syntax(fm)?;
    Some((cid, syntax))
}

/// Decode the birth date a stamp claims. Stamps look like `YYYYMMDDTHHMMSSZ_KIND_HEX`.
pub(crate) fn stamp_date(cid: &str) -> String {
    if cid.len() >= 8 && cid.as_bytes()[..8].iter().all(u8::is_ascii_digit) {
        format!("{}-{}-{}", &cid[0..4], &cid[4..6], &cid[6..8])
    } else {
        String::new()
    }
}

/// **The edit: strip the stamp and mark the mold, in one transformation.**
///
/// Returns `None` when the content is not a stamped mold — so a caller cannot repair a file the
/// rule does not claim, even by mistake.
///
/// Preserves, deliberately and by test: the bytes before the fence (a BOM, or blank lines); the
/// line terminators exactly as found, per line (these files are hand-authored and mixed CRLF/LF is
/// real); every other property; and the entire body — including a `---` divider further down,
/// which three of the Boss's molds contain and which a naive "split on ---" would mistake for the
/// fence.
pub(crate) fn strip_stamp_and_mark_template(content: &str) -> Option<String> {
    mold_evidence(content)?;
    let (lead, fm, rest) = split_frontmatter(content)?;

    let nl = if fm.contains("\r\n") { "\r\n" } else { "\n" };

    // `fm` begins with the newline that followed the opening `---`, so splitting on '\n' yields a
    // leading "" that is an ARTEFACT of that newline, not a line of the file. Drop exactly that
    // one — never "leading blanks" — because these molds genuinely open with a blank line and
    // swallowing it would make the change larger than the one-line-out/one-line-in this operation
    // promises, and larger than the verification checks for.
    let mut parts = fm.split('\n');
    let _artefact = parts.next();
    let kept: Vec<&str> = parts
        .filter(|line| {
            !(crate::yaml_lines::is_top_level_key_line(line)
                && line.trim_end().starts_with("cid_cn:"))
        })
        .collect();

    // `rest` starts at the closing "\n---"; drop that newline and re-emit it with the file's own
    // terminator, so a CRLF file does not acquire a bare LF at its fence.
    let tail = rest.strip_prefix('\n').unwrap_or(rest);

    let mut out = String::with_capacity(content.len() + 16);
    out.push_str(lead);
    out.push_str("---");
    out.push_str(nl);
    out.push_str("kind: template");
    for line in kept {
        out.push_str(nl);
        out.push_str(line.trim_end_matches('\r'));
    }
    out.push_str(nl);
    out.push_str(tail);
    Some(out)
}

// ─── Scan ────────────────────────────────────────────────────────────────────

/// Every stamped mold in the active universe's own libraries, with its evidence.
///
/// Read-only. Federated (linked-universe) libraries are INCLUDED because a mold is a mold wherever
/// it lives — but each candidate carries its library name so the user can see what he is approving.
#[tauri::command(async)]
pub fn scan_stamped_molds(app: tauri::AppHandle) -> Result<Vec<MoldCandidate>, String> {
    let libs = crate::universe::resolve_universe_libraries(app.clone())?;
    let inbound = inbound_cid_counts(&app);
    let mut out: Vec<MoldCandidate> = Vec::new();
    for lib in &libs {
        let mut paths: Vec<PathBuf> = Vec::new();
        crate::libraries::collect_md_paths(Path::new(&lib.path), &mut paths);
        for p in paths {
            let Ok(content) = std::fs::read_to_string(&p) else { continue };
            let Some((cid, syntax)) = mold_evidence(&content) else { continue };
            let path = p.to_string_lossy().to_string();
            out.push(MoldCandidate {
                stamp_date: stamp_date(&cid),
                inbound_cid_links: inbound.get(&cid).copied().unwrap_or(0),
                path,
                library: lib.name.clone(),
                cid_cn: cid,
                syntax: syntax.to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

// ─── Repair ──────────────────────────────────────────────────────────────────

/// Repair exactly the paths the user approved.
///
/// **It repairs only what it can re-prove.** Each file is re-read and re-tested against the rule
/// at the moment of the write; a path that no longer qualifies — edited since the scan, already
/// repaired, or never a mold — is refused with a reason rather than rewritten. So an approved list
/// that has gone stale cannot damage anything.
///
/// Order per file: back up (verified by re-reading the copy) → ONE write → verify against the
/// backup → re-index. A file that fails any step leaves the original untouched and does not stop
/// the others; every outcome is reported.
#[tauri::command(async)]
pub fn repair_stamped_molds(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<MoldRepairReport, String> {
    let root = crate::universe::active_universe_dir(&app)?;
    let backup_dir = crate::universe::constellation_dir(&root).join("pj454-backup");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Could not create the backup folder: {e}"))?;

    let mut outcomes: Vec<MoldRepairOutcome> = Vec::new();
    let mut repaired_cids: Vec<String> = Vec::new();

    for p in &paths {
        let path = Path::new(p);
        let outcome = repair_one(&app, path, &backup_dir).unwrap_or_else(|e| MoldRepairOutcome {
            path: p.clone(),
            ok: false,
            detail: e,
        });
        if outcome.ok {
            if let Ok(prev) = std::fs::read_to_string(backup_path_for(&backup_dir, path)) {
                if let Some((cid, _)) = mold_evidence(&prev) {
                    repaired_cids.push(cid);
                }
            }
        }
        outcomes.push(outcome);
    }

    // Notes that linked to a repaired mold BY IDENTITY now point at an identity no row holds.
    // Nothing clears a stale edge on its own — it is recomputed only when the SOURCE note is
    // re-indexed — so re-index them here rather than leave the residue.
    let relinked = reindex_sources_linking_to(&app, &repaired_cids);

    let repaired = outcomes.iter().filter(|o| o.ok).count();
    Ok(MoldRepairReport {
        attempted: outcomes.len(),
        repaired,
        failed: outcomes.len() - repaired,
        backup_dir: backup_dir.to_string_lossy().to_string(),
        outcomes,
        relinked_sources: relinked,
    })
}

/// Where a file's backup lives: flattened, with the stamp in the name so two same-named molds from
/// different folders cannot collide.
fn backup_path_for(backup_dir: &Path, file: &Path) -> PathBuf {
    let name = file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let digest: u64 = file
        .to_string_lossy()
        .bytes()
        .fold(1469598103934665603u64, |h, b| (h ^ b as u64).wrapping_mul(1099511628211));
    backup_dir.join(format!("{digest:016x}__{name}"))
}

fn repair_one(
    app: &tauri::AppHandle,
    path: &Path,
    backup_dir: &Path,
) -> Result<MoldRepairOutcome, String> {
    let p = path.to_string_lossy().to_string();
    let refuse = |detail: String| {
        Ok(MoldRepairOutcome { path: p.clone(), ok: false, detail })
    };

    if !path.is_file() {
        return refuse("The file is no longer there.".into());
    }
    let before = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return refuse(format!("Could not read it: {e}")),
    };
    // Re-prove it, now — never trust the list alone.
    let Some((cid, _)) = mold_evidence(&before) else {
        return refuse(
            "Skipped: this file is no longer a stamped template (it may have been edited or already repaired)."
                .into(),
        );
    };
    let Some(after) = strip_stamp_and_mark_template(&before) else {
        return refuse("Skipped: the change could not be prepared safely.".into());
    };

    // Back up, then READ THE BACKUP BACK. A backup nobody verified is not a backup.
    let bpath = backup_path_for(backup_dir, path);
    if let Err(e) = std::fs::write(&bpath, before.as_bytes()) {
        return refuse(format!("Could not write the backup, so nothing was changed: {e}"));
    }
    match std::fs::read_to_string(&bpath) {
        Ok(v) if v == before => {}
        _ => return refuse("The backup did not read back identical, so nothing was changed.".into()),
    }

    // ONE write, through the gate the rest of the app writes through.
    if let Err(e) = crate::write_gate::gate_write(path, &after, None, "pj454_mold_repair") {
        return refuse(format!("The change could not be written: {e}"));
    }

    // Verify on disk: exactly one line gone, one line added, everything else identical.
    let Ok(written) = std::fs::read_to_string(path) else {
        return refuse("Wrote the file but could not read it back to verify it.".into());
    };
    if let Err(why) = verify_change(&before, &written) {
        let _ = std::fs::write(path, before.as_bytes()); // put it back
        return refuse(format!("The change did not verify, so it was undone: {why}"));
    }

    // Tell the index. A failure here is NOT a failed repair — the file on disk is correct, which
    // is what matters (files are the source of truth), and the boot walk's mtime gate re-reads it
    // on the next launch. Reported honestly rather than swallowed or dressed up as an error.
    if let Some(lib_name) = crate::libraries::owning_own_library_name(app, &p) {
        use tauri::Manager as _;
        let search_state = app.state::<crate::search::SearchState>();
        if let Err(e) = crate::search::reindex_single_note(&search_state, &p, &lib_name) {
            return Ok(MoldRepairOutcome {
                path: p,
                ok: true,
                detail: format!(
                    "Repaired (stamp {cid} removed, marked as a template). The index could not be updated now, so it will catch up on the next launch: {e}"
                ),
            });
        }
    }
    Ok(MoldRepairOutcome {
        path: p,
        ok: true,
        detail: format!("Repaired: stamp {cid} removed, marked as a template."),
    })
}

/// Prove the write did exactly the two intended things and nothing else.
fn verify_change(before: &str, after: &str) -> Result<(), String> {
    let (b_lead, b_fm, b_rest) = split_frontmatter(before).ok_or("no frontmatter before")?;
    let (a_lead, a_fm, a_rest) = split_frontmatter(after).ok_or("no frontmatter after")?;
    if b_lead != a_lead {
        return Err("the bytes before the frontmatter changed".into());
    }
    if b_rest.trim_end_matches(['\n', '\r']) != a_rest.trim_end_matches(['\n', '\r']) {
        return Err("the body changed".into());
    }
    let norm = |fm: &str| -> Vec<String> {
        fm.split('\n').skip(1).map(|l| l.trim_end_matches('\r').to_string()).collect()
    };
    let (b_lines, a_lines) = (norm(b_fm), norm(a_fm));
    let removed: Vec<&String> = b_lines.iter().filter(|l| !a_lines.contains(l)).collect();
    let added: Vec<&String> = a_lines.iter().filter(|l| !b_lines.contains(l)).collect();
    if removed.len() != 1 || !removed[0].trim_start().starts_with("cid_cn:") {
        return Err(format!("expected exactly the stamp line to go, saw {removed:?}"));
    }
    if added.len() != 1 || added[0].trim() != "kind: template" {
        return Err(format!("expected exactly the template line to appear, saw {added:?}"));
    }
    Ok(())
}

/// Re-index every note that links BY IDENTITY to one of the repaired molds, so none is left
/// pointing at an identity nothing holds. Returns the paths it re-read.
fn reindex_sources_linking_to(app: &tauri::AppHandle, cids: &[String]) -> Vec<String> {
    if cids.is_empty() {
        return Vec::new();
    }
    use tauri::Manager as _;
    let state = app.state::<crate::search::SearchState>();
    let mut sources: Vec<String> = Vec::new();
    let _ = crate::search::with_read_conn(&state, |conn| {
        for cid in cids {
            let mut stmt = conn
                .prepare("SELECT DISTINCT source_path FROM note_links WHERE target_cid_cn = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([cid], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            for s in rows.flatten() {
                if !sources.contains(&s) {
                    sources.push(s);
                }
            }
        }
        Ok(())
    });
    for s in &sources {
        if let Some(lib) = crate::libraries::owning_own_library_name(app, s) {
            let _ = crate::search::reindex_single_note(&state, s, &lib);
        }
    }
    sources
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact shape of the Boss's molds: a blank line after the fence, blank fields waiting to
    /// be filled, `created` holding a placeholder, and the stamp beneath it.
    const REAL_MOLD: &str = "---\n\nup:\nrelated:\ncreated: \"{{date}}\"\ncid_cn: 20260414T152113Z_NOTE_207B\n---\n\nbody\n";

    #[test]
    fn pj454_the_rule_finds_a_stamped_mold_by_either_placeholder_syntax() {
        let (cid, syntax) = mold_evidence(REAL_MOLD).expect("this is a mold");
        assert_eq!(cid, "20260414T152113Z_NOTE_207B");
        assert_eq!(syntax, "{{ }}");
        assert_eq!(stamp_date(&cid), "2026-04-14");

        // Templater syntax — exactly ONE of the Boss's 43 is identified only by this, so a rule
        // that knew only about braces would have left it stamped.
        let templater = "---\ncreated: <% tp.file.creation_date() %>\ncid_cn: 20251229T125213Z_NOTE_7C1D\n---\nb\n";
        assert_eq!(mold_evidence(templater).unwrap().1, "<% %>");
    }

    /// The false-positive direction — the one that would strip a REAL note's identity, silently
    /// severing its earned reading history. Each of these must be refused.
    #[test]
    fn pj454_the_rule_refuses_everything_that_is_not_a_stamped_mold() {
        // a real note: stamped, no placeholder anywhere in the fence
        assert!(mold_evidence("---\ntitle: Real\ncid_cn: 20260414T152113Z_NOTE_0001\n---\nbody\n").is_none());
        // a mold with no stamp — nothing to repair
        assert!(mold_evidence("---\ncreated: \"{{date}}\"\n---\nb\n").is_none());
        // braces in the BODY, not the fence — a note ABOUT templates
        assert!(mold_evidence("---\ntitle: T\ncid_cn: 20260414T152113Z_NOTE_0002\n---\nUse {{date}} in a template.\n").is_none());
        // an INDENTED cid_cn belongs to another property, not the note
        assert!(mold_evidence("---\ncreated: \"{{date}}\"\nmeta:\n  cid_cn: 20260414T152113Z_NOTE_0003\n---\nb\n").is_none());
        // an unterminated fence is not frontmatter
        assert!(mold_evidence("---\ncreated: \"{{date}}\"\ncid_cn: 20260414T152113Z_NOTE_0004\nnever closed\n").is_none());
        // no frontmatter at all
        assert!(mold_evidence("cid_cn: x {{date}}\n").is_none());
    }

    /// The edit itself: exactly one line out, one line in, everything else byte-identical —
    /// INCLUDING the blank line these molds open with. Swallowing it would make the change larger
    /// than the operation promises and larger than the verification checks for.
    #[test]
    fn pj454_the_edit_removes_the_stamp_marks_the_mold_and_touches_nothing_else() {
        let after = strip_stamp_and_mark_template(REAL_MOLD).expect("repairable");
        assert_eq!(
            after,
            "---\nkind: template\n\nup:\nrelated:\ncreated: \"{{date}}\"\n---\n\nbody\n"
        );
        verify_change(REAL_MOLD, &after).expect("the verifier must accept the real edit");
        // and it is idempotent-by-refusal: the repaired file is no longer a candidate
        assert!(strip_stamp_and_mark_template(&after).is_none());
    }

    /// Three of the Boss's molds contain a `---` divider further down the body. A naive
    /// "split on ---" would treat that as the fence and mangle the note.
    #[test]
    fn pj454_a_divider_in_the_body_is_not_the_fence() {
        let with_divider = "---\ncreated: \"{{date}}\"\ncid_cn: 20260414T152113Z_NOTE_264E\n---\nintro\n\n---\n\nafter the divider\n";
        let after = strip_stamp_and_mark_template(with_divider).expect("repairable");
        assert!(after.ends_with("intro\n\n---\n\nafter the divider\n"), "body must survive: {after:?}");
        assert!(!after.contains("cid_cn"));
        assert_eq!(after.matches("kind: template").count(), 1);
        verify_change(with_divider, &after).unwrap();
    }

    /// CRLF files must not acquire mixed endings — these are hand-authored notes edited in
    /// Windows tools, and a mangled line ending is a diff on every line.
    #[test]
    fn pj454_crlf_files_stay_crlf() {
        let crlf = "---\r\ncreated: \"{{date}}\"\r\ncid_cn: 20260414T152113Z_NOTE_0005\r\n---\r\nbody\r\n";
        let after = strip_stamp_and_mark_template(crlf).expect("repairable");
        assert!(!after.contains('\n') || after.matches('\n').count() == after.matches("\r\n").count(),
            "every newline must remain CRLF: {after:?}");
        assert!(after.contains("kind: template"));
        assert!(!after.contains("cid_cn"));
    }

    /// The verifier is the last line of defence — it must REJECT a change that does more than the
    /// two intended things, because that is what triggers the automatic undo.
    #[test]
    fn pj454_the_verifier_rejects_any_change_beyond_the_two_intended_lines() {
        let good = strip_stamp_and_mark_template(REAL_MOLD).unwrap();
        // body altered
        let tampered = good.replace("body", "BODY");
        assert!(verify_change(REAL_MOLD, &tampered).is_err(), "a body change must be rejected");
        // another property dropped
        let dropped = good.replace("related:\n", "");
        assert!(verify_change(REAL_MOLD, &dropped).is_err(), "losing a property must be rejected");
        // marker missing
        let unmarked = good.replace("kind: template\n", "");
        assert!(verify_change(REAL_MOLD, &unmarked).is_err(), "the marker is not optional");
    }

    /// The repaired file must be protected by the guard built alongside this — that is the whole
    /// reason the marker is written, and without it the boot healer re-stamps the file.
    #[test]
    fn pj454_a_repaired_mold_is_then_refused_by_the_stamping_engine() {
        let after = strip_stamp_and_mark_template(REAL_MOLD).unwrap();
        assert!(
            crate::canonical::frontmatter_declares_template(&after),
            "the repair must leave the file recognisable to the guard, or the healer re-stamps it",
        );
    }
}

/// `cid_cn` → how many notes link to it BY IDENTITY.
///
/// Best-effort: an unavailable index yields an empty map, and the caller then reports zero rather
/// than failing the scan. The count is shown to the user and drives the post-repair re-read; it is
/// never a gate, because a link by identity does not make a mold a note.
fn inbound_cid_counts(app: &tauri::AppHandle) -> BTreeMap<String, u32> {
    use tauri::Manager as _;
    let state = app.state::<crate::search::SearchState>();
    let mut map = BTreeMap::new();
    let _ = crate::search::with_read_conn(&state, |conn| {
        let mut stmt = conn
            .prepare(
                "SELECT target_cid_cn, COUNT(*) FROM note_links \
                 WHERE target_cid_cn IS NOT NULL AND target_cid_cn != '' GROUP BY target_cid_cn",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            map.insert(row.0, row.1 as u32);
        }
        Ok(())
    });
    map
}
