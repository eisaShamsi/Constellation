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
use std::collections::{BTreeMap, HashSet};
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

/// Which class an outcome belongs to (PJ-466).
///
/// `ok: bool` could not tell "left alone, nothing owed" from "still broken", so the door counted
/// both as failures: a file that needed nothing halted the run, and a genuine write failure was
/// reported to the user as "skipped". The three-way split is not invented here — it is the shipped
/// `PruneReceipt`'s (`phantom_prune.rs:585-604`), whose own doc comments draw the same line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MoldOutcomeKind {
    /// Changed on disk and verified. Nothing is owed.
    Repaired,
    /// Nothing was attempted and nothing is owed — this file no longer needs the repair.
    /// **Never an error** (`phantom_prune.rs:590-592`): the second look disagreeing with the first
    /// is the system working, not failing. A `Kept` outcome must never halt the cascade.
    Kept,
    /// The repair did not complete. The user is owed an account of every one of these, and a
    /// `Failed` in the delicate batch halts the run — the same rule a consented batched file
    /// operation already follows at `mig108.rs:1044-1046`.
    ///
    /// **Do NOT write "the birth date is still on disk" here, however natural it reads.** It is
    /// true of most members and FALSE of `readBackFailed`, where `gate_write` already succeeded and
    /// only the verifying read failed; and it is UNKNOWN for `verifyNotRestored`. A false claim in
    /// a doc comment is how a removed sentence gets re-derived by the next author — PJ-466 removed
    /// exactly that sentence from the user-facing copy.
    Failed,
}

/// What happened to one file.
#[derive(Debug, Clone, Serialize)]
pub struct MoldRepairOutcome {
    pub path: String,
    pub kind: MoldOutcomeKind,
    /// A STABLE code naming why, for the frontend to translate. The codes are the contract:
    /// `repaired` · `repairedIndexLater` · `gone` · `notStamped` · `notOurs` · `unreadable` ·
    /// `notPreparable` · `backupFailed` · `backupMismatch` · `writeFailed` · `readBackFailed` ·
    /// `verifyFailed` · `verifyNotRestored`. Never render this to a user untranslated.
    pub reason: &'static str,
    /// The raw technical detail (English, may carry an OS error). Shown as a dimmed tail under the
    /// translated sentence, the way `settings.index.repair.familyFailed` already does it.
    pub detail: String,
    /// True once this file's backup was written AND read back identical — so the report can say
    /// "each file that was changed was backed up first" as a counted fact rather than an
    /// assumption. Several return paths come back before the backup is ever attempted.
    pub backed_up: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoldRepairReport {
    /// Every file the caller submitted — including those that turned out to need nothing.
    pub attempted: usize,
    pub repaired: usize,
    /// Left alone with nothing owed. NOT a failure, and deliberately not counted as one.
    pub kept: usize,
    /// Not repaired AND still carrying the defect. This is what the halt reads.
    pub failed: usize,
    /// How many files actually got a verified backup. `backup_dir` names where.
    pub backed_up: usize,
    pub backup_dir: String,
    pub outcomes: Vec<MoldRepairOutcome>,
    /// Notes re-read because they linked to a repaired mold by identity — only those whose
    /// re-index actually SUCCEEDED (PJ-470: the failures used to be counted as refreshed).
    pub relinked_sources: Vec<String>,
}

/// The pre-flight verdict for ONE file — what the repair WOULD do, computed WITHOUT writing.
///
/// The door shows this before the Boss approves, so "what will change" is a proven claim and not a
/// promise. `will_apply` runs the real edit and the real verifier in memory — a file that no longer
/// qualifies (edited since the scan, already repaired) or that the verifier would reject comes back
/// `false` with the reason, and is caught BEFORE the write batch rather than by the engine's undo
/// after it.
#[derive(Debug, Clone, Serialize)]
pub struct MoldPreview {
    pub path: String,
    /// True when the edit prepares cleanly AND the verifier accepts it — i.e. the repair will
    /// verify on disk.
    pub will_apply: bool,
    /// The exact stamp line that will be removed (empty when `will_apply` is false).
    pub removes: String,
    /// The exact line that will be added — always `kind: template` on success.
    pub adds: String,
    /// Plain-language reason, especially when `will_apply` is false.
    pub reason: String,
    /// True for a file whose shape needs extra care — a `---` divider in the body, no final
    /// newline, or a Templater `<% %>` stamp. The engine handles all of these (tested), but the
    /// door runs them FIRST as their own batch, so a systematic problem with the tricky cases
    /// halts before the ordinary files are touched.
    pub needs_care: bool,
}

/// Does this file's shape warrant running it in the delicate batch first?
///
/// Not a correctness signal — the edit is proven on all three shapes — but a sequencing one:
/// these are the files where a future regression would most plausibly appear, so they go first.
fn needs_care(content: &str) -> bool {
    // Templater-only stamp.
    if matches!(mold_evidence(content), Some((_, "<% %>"))) {
        return true;
    }
    // No final newline.
    if !content.ends_with('\n') {
        return true;
    }
    // A `---` divider in the BODY (below the frontmatter fence).
    if let Some((_, _, rest)) = split_frontmatter(content) {
        // `rest` begins at the closing "\n---"; skip that fence line, then look for another.
        let body = rest.strip_prefix('\n').and_then(|r| r.strip_prefix("---")).unwrap_or("");
        if body.lines().any(|l| l.trim_end_matches('\r').trim() == "---") {
            return true;
        }
    }
    false
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

/// Every stamped mold in the active universe's OWN libraries, with its evidence.
///
/// Read-only, and **own-libraries only** — never the federated (linked-universe) libraries.
///
/// Safety inspection 2026-09-01 (MED, index-divergence / write-sovereignty): the first version
/// used `resolve_universe_libraries` (recursive, federated), so a linked universe's mold became a
/// repair candidate. The repair would then WRITE that universe's file (`gate_write` has no
/// own-scope check, and stripped content has no `cid_cn` so it takes the unchecked path) but could
/// NOT update that universe's index — the parent attaches it READ-ONLY and never re-walks it — and
/// `owning_own_library_name` returned `None` for the foreign path, so the re-index was silently
/// skipped and a clean "Repaired" was reported over a diverged index. Two rulings settle it: a
/// template repair is a WRITE, and write sovereignty (MIG-111) says a linked universe's bookkeeping
/// happens in ITS OWN database — which the parent cannot touch. So a federated mold is repaired
/// when ITS universe is the active one, never reached into from here. `load_libraries` is the
/// same own-only resolver the whole write path already uses (MIG-065 §J), so the repair can only
/// write files whose index it can also fix — no divergence is possible.
#[tauri::command(async)]
pub fn scan_stamped_molds(app: tauri::AppHandle) -> Result<Vec<MoldCandidate>, String> {
    let libs = crate::libraries::load_libraries(&app);
    let foreign = crate::libraries::foreign_library_roots(&app, &libs);
    // Walk FIRST, then ask about the handful of identities the walk actually found.
    //
    // This scan runs on every boot (`refreshMoldRepairCount`), so its cost is boot cost. The
    // whole-table `GROUP BY` that used to feed it was cheap only because `COUNT(*)` could be
    // answered from the covering index; counting NOTES instead of LINK ROWS needs `source_path`,
    // which drops that plan and seeks into a 2 GB table for every one of ~234,000 rows, then
    // builds a temp B-tree per group. MEASURED on Eisa Cognitive Knowledge: 23 ms → 1,128 ms warm
    // and 8,215 ms cold. Asking for ~43 known cids instead is 0.10 ms, with no new index.
    // Rule 8's hard constraint: no new feature may regress boot.
    let mut found = scan_stamped_molds_in(&libs, &foreign, &BTreeMap::new());
    let inbound = inbound_cid_counts(&app, &found.iter().map(|c| c.cid_cn.clone()).collect::<Vec<_>>());
    for c in &mut found {
        c.inbound_cid_links = inbound.get(&c.cid_cn).copied().unwrap_or(0);
    }
    Ok(found)
}

/// The scan, free of `AppHandle`, so a test drives THIS function and not a copy of its loop.
///
/// Scoped as `index_repair::run_full` scopes the same collector (panel ruling 2026-09-12):
/// top-level own roots only, a linked universe's files dropped after collection, and each file
/// attributed by `library_name_for_path` — the resolver that stamps `note_meta.library_name`, so
/// the review screen's group heads agree with the index by identity of function.
///
/// Deliberately NO dedupe: a dedupe would make the on-screen count unable to disagree with a
/// re-introduced double walk. The guard is
/// `pj454_scan_lists_a_mold_in_a_nested_own_library_once_under_its_owner`, which carries the
/// incident (74 rows for 39 files) and was written RED against the previous loop.
///
/// One thing the plain top-level rule got wrong (safety inspection 2026-09-12): a nested own
/// library the parent's walk cannot ENTER — a dot-named path component, or a junction — was
/// skipped as a start point too, so its molds were silently absent (the previous loop, which
/// started at every own root, listed them). Rather than a second copy of the collector's skip
/// rules, which would drift, the collector's own behaviour is the oracle: after the top-level
/// walk, a nested root that no collected path falls under is walked from its own root —
/// shortest first, so a nested-in-nested root sees its parent's second-pass walk and is not
/// listed twice. Guard: `pj454_scan_reaches_a_nested_own_library_the_parent_walk_cannot_enter`.
pub(crate) fn scan_stamped_molds_in(
    libs: &[crate::libraries::LibraryInfo],
    foreign: &HashSet<String>,
    inbound: &BTreeMap<String, u32>,
) -> Vec<MoldCandidate> {
    let mut out: Vec<MoldCandidate> = Vec::new();
    // Every path a walk has collected so far, normalized — the oracle for "did a walk enter it".
    let mut reached: Vec<String> = Vec::new();

    let (top, mut nested): (Vec<_>, Vec<_>) = libs.iter().partition(|lib| {
        !crate::libraries::path_is_under_any(&lib.path, &crate::libraries::nested_library_paths(libs, &lib.path))
    });
    // A root under another own root is normally reached by the parent's walk — walk the top level first.
    for lib in top {
        walk_own_root(lib, libs, foreign, inbound, &mut out, &mut reached);
    }
    // Then the nested roots the walks so far did not enter, shortest path first.
    nested.sort_by_key(|lib| norm_path(&lib.path).len());
    for lib in nested {
        let root: HashSet<String> = [norm_path(&lib.path)].into_iter().collect();
        if reached.iter().any(|r| crate::libraries::path_is_under_any(r, &root)) {
            continue;
        }
        walk_own_root(lib, libs, foreign, inbound, &mut out, &mut reached);
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The normalization `nested_library_paths` / `path_is_under_any` use for a root: forward
/// slashes, no trailing slash, lowercase.
fn norm_path(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// One walk from `lib`'s own root: collect, drop a linked universe's files, keep the molds, and
/// attribute each to its longest-root owner. Records every collected path in `reached`.
fn walk_own_root(
    lib: &crate::libraries::LibraryInfo,
    libs: &[crate::libraries::LibraryInfo],
    foreign: &HashSet<String>,
    inbound: &BTreeMap<String, u32>,
    out: &mut Vec<MoldCandidate>,
    reached: &mut Vec<String>,
) {
    let mut paths: Vec<PathBuf> = Vec::new();
    crate::libraries::collect_md_paths(Path::new(&lib.path), &mut paths);
    for p in paths {
        let path = p.to_string_lossy().into_owned();
        reached.push(norm_path(&path));
        // A linked universe's note is never this universe's candidate (write sovereignty).
        if !foreign.is_empty() && crate::libraries::path_is_under_any(&path, foreign) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&p) else { continue };
        let Some((cid, syntax)) = mold_evidence(&content) else { continue };
        // The fallback is reachable: a root registered WITH a trailing separator resolves to
        // None (`library_name_for_path` does not trim it, unlike its siblings) — then the
        // walking library is the owner by construction. Display-only either way.
        let library = crate::libraries::library_name_for_path(libs, &path)
            .unwrap_or_else(|| lib.name.clone());
        out.push(MoldCandidate {
            stamp_date: stamp_date(&cid),
            inbound_cid_links: inbound.get(&cid).copied().unwrap_or(0),
            path,
            library,
            cid_cn: cid,
            syntax: syntax.to_string(),
        });
    }
}

// ─── Repair ──────────────────────────────────────────────────────────────────

/// **Pre-flight: what the repair WOULD do, touching nothing.**
///
/// Runs the real edit and the real verifier in memory for each path, so the door can show a proven
/// per-file verdict before the Boss approves — and flag the awkward files (a `---` divider in the
/// body, a missing final newline, a Templater-only stamp) or a file gone stale since the scan,
/// BEFORE the write batch rather than after. Read-only: it opens no writer, holds no lock.
#[tauri::command(async)]
pub fn preview_mold_repair(_app: tauri::AppHandle, paths: Vec<String>) -> Vec<MoldPreview> {
    paths
        .into_iter()
        .map(|p| {
            let path = Path::new(&p);
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    return MoldPreview {
                        path: p,
                        will_apply: false,
                        removes: String::new(),
                        adds: String::new(),
                        reason: format!("Could not read the file: {e}"),
                        needs_care: false,
                    }
                }
            };
            let Some((cid, _)) = mold_evidence(&content) else {
                return MoldPreview {
                    path: p,
                    will_apply: false,
                    removes: String::new(),
                    adds: String::new(),
                    reason: "No longer a stamped template — it may have been edited or already repaired.".to_string(),
                    needs_care: needs_care(&content),
                };
            };
            let care = needs_care(&content);
            match strip_stamp_and_mark_template(&content) {
                Some(after) if verify_change(&content, &after).is_ok() => MoldPreview {
                    path: p,
                    will_apply: true,
                    removes: format!("cid_cn: {cid}"),
                    adds: "kind: template".to_string(),
                    reason: "Ready: the stamp will be removed and the file marked as a template.".to_string(),
                    needs_care: care,
                },
                Some(_) => MoldPreview {
                    path: p,
                    will_apply: false,
                    removes: String::new(),
                    adds: String::new(),
                    reason: "The prepared change did not verify cleanly, so this file would be skipped.".to_string(),
                    needs_care: care,
                },
                None => MoldPreview {
                    path: p,
                    will_apply: false,
                    removes: String::new(),
                    adds: String::new(),
                    reason: "The change could not be prepared safely, so this file would be skipped.".to_string(),
                    needs_care: care,
                },
            }
        })
        .collect()
}

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

    // PJ-466: refuse the RUN, not every file in it. Without this, an unreadable registry makes the
    // per-file sovereignty check answer "not ours" for all 43 rows — a receipt that blames the
    // user's files for a problem with the registry. The house already refuses whole runs this way:
    // `PruneReceipt` carries a `refused` field precisely so "the run did not happen" is not told as
    // N failures.
    //
    // It MUST be `try_load_libraries` — the strict, own-only, UNCACHED loader that `require_own_library`
    // itself resolves through. The federation-recursive `load_all_libraries` would be the wrong
    // predicate twice over: it reads a module-level cache that can answer from a warm copy after
    // `libraries.json` has become unreadable, and it counts a Linked Universe's libraries, which can
    // make the list non-empty while THIS universe's own registry is unreadable. Either way the guard
    // would pass and the 43 refusals it exists to prevent would print anyway. `index_repair.rs`'s
    // `no_index_write_module_resolves_libraries_through_the_federation` test now pins this module too.
    let own = crate::libraries::try_load_libraries(&app).map_err(|e| {
        format!("Constellation could not read this universe's libraries just now, so it has stopped and changed nothing: {e}")
    })?;
    let foreign = crate::libraries::foreign_library_roots(&app, &own);

    let backup_dir = crate::universe::constellation_dir(&root).join("pj454-backup");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Could not create the backup folder: {e}"))?;

    let mut outcomes: Vec<MoldRepairOutcome> = Vec::new();
    let mut repaired_cids: Vec<String> = Vec::new();

    for p in &paths {
        let path = Path::new(p);
        // PJ-466: `repair_one` returns an outcome directly. It previously returned a `Result` whose
        // `Err` arm none of its exits could produce, so the `unwrap_or_else` fallback here was dead
        // code that could never be constructed — dead code in a receipt path is a class nobody reads.
        let outcome = repair_one(&app, path, &backup_dir, &own, &foreign);
        if outcome.kind == MoldOutcomeKind::Repaired {
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

    let Tally { repaired, kept, failed, backed_up } = tally(&outcomes);
    Ok(MoldRepairReport {
        attempted: outcomes.len(),
        repaired,
        kept,
        failed,
        backed_up,
        backup_dir: backup_dir.to_string_lossy().to_string(),
        outcomes,
        relinked_sources: relinked,
    })
}

struct Tally {
    pub repaired: usize,
    pub kept: usize,
    pub failed: usize,
    pub backed_up: usize,
}

/// Count the outcomes BY CLASS.
///
/// PJ-466: the report used to compute `failed = attempted - repaired`, which is what made a file
/// that needed nothing indistinguishable from one the app could not write — and what let a benign
/// second-look disagreement halt the whole cascade. Counting each class on its own terms is the
/// whole fix; pulled out here so the arithmetic is provable without a running app.
fn tally(outcomes: &[MoldRepairOutcome]) -> Tally {
    Tally {
        repaired: outcomes.iter().filter(|o| o.kind == MoldOutcomeKind::Repaired).count(),
        kept: outcomes.iter().filter(|o| o.kind == MoldOutcomeKind::Kept).count(),
        failed: outcomes.iter().filter(|o| o.kind == MoldOutcomeKind::Failed).count(),
        backed_up: outcomes.iter().filter(|o| o.backed_up).count(),
    }
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
    own: &[crate::libraries::LibraryInfo],
    foreign: &HashSet<String>,
) -> MoldRepairOutcome {
    let p = path.to_string_lossy().to_string();
    // KEPT — nothing attempted, nothing owed. FAILED — the defect is still on disk.
    // The two are not interchangeable: KEPT must never halt the cascade, FAILED must.
    let keep = |reason: &'static str, detail: String| MoldRepairOutcome {
        path: p.clone(), kind: MoldOutcomeKind::Kept, reason, detail, backed_up: false,
    };
    let fail = |reason: &'static str, detail: String| MoldRepairOutcome {
        path: p.clone(), kind: MoldOutcomeKind::Failed, reason, detail, backed_up: false,
    };
    // After the backup verifies, a failure still carries `backed_up` — the file has a restore point
    // even though the repair did not land, and the receipt is entitled to say so.
    let fail_backed = |reason: &'static str, detail: String| MoldRepairOutcome {
        path: p.clone(), kind: MoldOutcomeKind::Failed, reason, detail, backed_up: true,
    };

    // "Gone" must mean GENUINELY ABSENT. `is_file()` also returns false when the metadata call
    // itself fails — a permission denial, an antivirus lock — and classing those as Kept would tell
    // the user a file is no longer there while it sits on disk still carrying the stamp, AND let it
    // pass the halt as "nothing owed". Only NotFound is Kept; every other error is Failed.
    match path.symlink_metadata() {
        Ok(m) if m.is_file() => {}
        Ok(_) => return fail("unreadable", "That path is not a file.".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return keep("gone", "The file is no longer there.".into());
        }
        Err(e) => return fail("unreadable", format!("Could not look at the file: {e}")),
    }
    // Write-sovereignty guard (defense in depth — the scan already returns own-only paths, but a
    // caller must not be able to write a linked universe's file by passing its path directly).
    //
    // `require_own_library` rather than `owning_own_library_name`: it is FAIL-CLOSED on an
    // unreadable own registry (a bare prefix match cannot tell "not ours" from "cannot tell", and
    // its own doc records the inspection proving the prefix test admits a Linked Universe nested
    // under the root). It also hands back the owning library's name, so the post-write re-index
    // does not resolve the same path a second time.
    //
    // FAILED, not kept, and deliberately so: the file still carries the stamp, so something IS
    // owed; and this guard firing at all means the scan and the guard disagree about which
    // universe owns the file — the most systematic fault in the set, exactly what the halt is for.
    // The user-facing sentence says only "could not confirm it belongs to a library this universe
    // owns", which is true of BOTH the foreign case and the unreadable-registry case.
    let lib_name = match crate::libraries::require_own_library_in(own, foreign, &p) {
        Ok(name) => name,
        Err(e) => return fail("notOurs", e),
    };
    let before = match std::fs::read_to_string(path) {
        // FAILED: nothing was attempted, but the file is NOT known to be fine — an unreadable file
        // is a lock, a permission denial or bad bytes, and the stamp is still there.
        Err(e) => return fail("unreadable", format!("Could not read it: {e}")),
        Ok(c) => c,
    };
    // Re-prove it, now — never trust the list alone. A second look disagreeing with the first is
    // the system working: KEPT (`phantom_prune.rs:590-592` states the same rule for its own class).
    let Some((cid, _)) = mold_evidence(&before) else {
        return keep(
            "notStamped",
            "This file is no longer a stamped template (it may have been edited or already repaired)."
                .into(),
        );
    };
    let Some(after) = strip_stamp_and_mark_template(&before) else {
        return fail("notPreparable", "The change could not be prepared safely.".into());
    };
    // Prove the edit BEFORE writing it, on the bytes just read.
    //
    // PJ-469 reported that a file the preview already knew would refuse was still written and then
    // rolled back. The first fix filtered those out in the DOOR, using a preview taken minutes
    // earlier — which put the judgement in the wrong layer and on staler evidence, and could label
    // a file "kept · already fixed" that had been re-stamped since. This is the same check the
    // preview runs (`preview_mold_repair`), made here on fresh bytes: the needless write is gone
    // and there is exactly one judgement, made once, where the file is.
    if let Err(why) = verify_change(&before, &after) {
        return fail("notPreparable", format!("The change could not be prepared safely: {why}"));
    }

    // ─── THE NO-WRITE LINE ────────────────────────────────────────────────────────────────────
    // Every return ABOVE this point happens before anything is written, and that is the one
    // invariant every "nothing was changed" sentence on the receipt rests on: no KEPT outcome may
    // ever reach a write. A test pins it, because a future return site could break it in silence.
    // ──────────────────────────────────────────────────────────────────────────────────────────

    // Back up, then READ THE BACKUP BACK. A backup nobody verified is not a backup.
    let bpath = backup_path_for(backup_dir, path);
    if let Err(e) = std::fs::write(&bpath, before.as_bytes()) {
        return fail("backupFailed", format!("Could not write the backup, so nothing was changed: {e}"));
    }
    match std::fs::read_to_string(&bpath) {
        Ok(v) if v == before => {}
        _ => return fail("backupMismatch", "The backup did not read back identical, so nothing was changed.".into()),
    }

    // ONE write, through the gate the rest of the app writes through.
    if let Err(e) = crate::write_gate::gate_write(path, &after, None, "pj454_mold_repair") {
        // `atomic_write` writes a same-dir temp and replaces; the original is never opened for
        // writing, and the temp is removed when the retries are exhausted (`write_gate.rs`). So
        // the note itself is byte-identical here — which is what lets the receipt say so.
        return fail_backed("writeFailed", format!("The change could not be written: {e}"));
    }

    // Verify on disk: exactly one line gone, one line added, everything else identical.
    let Ok(written) = std::fs::read_to_string(path) else {
        // The ONE non-success class where the file WAS changed. It must never be worded as though
        // nothing happened — the write landed and only the read-back failed.
        return fail_backed("readBackFailed", "Wrote the file but could not read it back to verify it.".into());
    };
    if let Err(why) = verify_change(&before, &written) {
        // PJ-466: this restore's Result used to be discarded (`let _ = …`) while the message
        // asserted "so it was undone" regardless — a sentence nothing checked, which is precisely
        // the class of claim this job exists to remove. Check it, and give the failure its own code.
        return match std::fs::write(path, before.as_bytes()) {
            Ok(()) => fail_backed("verifyFailed", format!("The change did not verify, so it was undone: {why}")),
            Err(e) => fail_backed(
                "verifyNotRestored",
                format!("The change did not verify and could not be put back ({e}): {why}"),
            ),
        };
    }

    // Tell the index. A failure here is NOT a failed repair — the file on disk is correct, which is
    // what matters (files are the source of truth). It is reported honestly rather than swallowed
    // or dressed up as an error, and the receipt tells the user where to put it right.
    //
    // PJ-466: this comment used to add "and the boot walk's mtime gate re-reads it on the next
    // launch." That is FALSE, and the user-facing string that repeated it was removed in the same
    // pass — `reconcile.rs` states in its own words that the boot pass heals EXISTENCE drift only,
    // and that "every drifted note in the report is still drifted when the user reads it." The
    // launch NOTICES; it does not repair. Leaving the claim in a comment is how a removed sentence
    // gets re-derived by the next author, which is why it is corrected here and not just deleted.
    {
        use tauri::Manager as _;
        let search_state = app.state::<crate::search::SearchState>();
        if let Err(e) = crate::search::reindex_single_note(&search_state, &p, &lib_name) {
            return MoldRepairOutcome {
                path: p,
                kind: MoldOutcomeKind::Repaired,
                reason: "repairedIndexLater",
                // PJ-466: this used to promise the index "will catch up on the next launch". It
                // does not — the boot pass NOTICES drift and reports it; it does not silently
                // re-read. The dimmed technical tail must not carry in English the exact promise
                // the translated sentence above it was corrected to drop.
                detail: format!(
                    "Repaired (stamp {cid} removed, marked as a template). The search index could not be updated just now: {e}"
                ),
                backed_up: true,
            };
        }
    }
    MoldRepairOutcome {
        path: p,
        kind: MoldOutcomeKind::Repaired,
        reason: "repaired",
        detail: format!("Repaired: stamp {cid} removed, marked as a template."),
        backed_up: true,
    }
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
    // PJ-466/PJ-470: report what was actually REFRESHED, not what was attempted. This number is
    // rendered to the user as "{n} notes … were refreshed"; a discarded `Err` used to be counted
    // among them, so the sentence could overstate by exactly the failures it hid.
    let mut refreshed: Vec<String> = Vec::new();
    for s in &sources {
        if let Some(lib) = crate::libraries::owning_own_library_name(app, s) {
            if crate::search::reindex_single_note(&state, s, &lib).is_ok() {
                refreshed.push(s.clone());
            }
        }
    }
    refreshed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A registered library for the scan tests. `is_universe_notes` is not read by anything the
    /// scan calls, so it is not a parameter.
    fn test_lib(name: &str, path: &Path) -> crate::libraries::LibraryInfo {
        crate::libraries::LibraryInfo {
            id: name.into(),
            name: name.into(),
            path: path.to_string_lossy().into(),
            is_universe_notes: false,
            canonical_mode: "native".into(),
        }
    }

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

    /// The door's pre-flight verdict must agree with what the repair will actually do — a green
    /// preview that then fails to verify, or a red one that would have succeeded, is exactly the
    /// "a cross-check that cannot disagree" trap. This exercises the same pure functions the
    /// command wraps (the command itself only adds file I/O), so a real mold previews green with
    /// the exact two lines, and a non-mold previews red.
    #[test]
    fn pj454_preview_agrees_with_the_real_edit() {
        // A real mold: preview must be green, name the exact lines, and the edit it previews must
        // itself verify — the two can never disagree.
        let after = strip_stamp_and_mark_template(REAL_MOLD).expect("a mold is repairable");
        assert!(verify_change(REAL_MOLD, &after).is_ok(), "the previewed edit must verify");
        let (cid, _) = mold_evidence(REAL_MOLD).unwrap();
        assert_eq!(format!("cid_cn: {cid}"), "cid_cn: 20260414T152113Z_NOTE_207B");

        // A real note (stamped, no placeholder): preview must be red — no edit, nothing removed.
        let real_note = "---\ntitle: Real\ncid_cn: 20260414T152113Z_NOTE_0001\n---\nbody\n";
        assert!(mold_evidence(real_note).is_none(), "a real note is never previewed for repair");
    }

    /// PJ-454 (panel 2026-09-12) — **the scan lists a mold in a nested own library ONCE, under its
    /// owner.** Eisa Universe registers its root plus four libraries nested under it; the root's walk
    /// reaches the nested folders and each nested library's own walk reaches them again, so the door
    /// showed 74 rows for 39 files, ran three delicate files twice, and the second pass's refusal
    /// halted the cascade before the ordinary files. `Nested 2` shares a prefix with `Nested` and
    /// must NOT be treated as its boundary (separator-bounded compare). Written RED first: on the
    /// verbatim-extracted loop this yields FOUR rows with `Nested mold.md` under both libraries.
    #[test]
    fn pj454_scan_lists_a_mold_in_a_nested_own_library_once_under_its_owner() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let nested = root.join("Nested");
        let lookalike = root.join("Nested 2"); // shares the prefix; must NOT be a boundary
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(&lookalike).unwrap();
        std::fs::write(root.join("Root mold.md"), REAL_MOLD).unwrap();
        std::fs::write(nested.join("Nested mold.md"), REAL_MOLD).unwrap();
        std::fs::write(lookalike.join("Lookalike mold.md"), REAL_MOLD).unwrap();
        let libs = vec![test_lib("Root", root), test_lib("Nested", &nested)];
        let out = scan_stamped_molds_in(&libs, &Default::default(), &Default::default());
        let mut seen: Vec<(&str, &str)> = out
            .iter()
            .map(|c| (c.library.as_str(), Path::new(&c.path).file_name().unwrap().to_str().unwrap()))
            .collect();
        seen.sort();
        assert_eq!(
            seen,
            [("Nested", "Nested mold.md"), ("Root", "Lookalike mold.md"), ("Root", "Root mold.md")],
            "each mold exactly once, under its owner"
        );
    }

    /// PJ-454 (safety inspection 2026-09-12, LOW, confirmed) — **a nested own library the parent's
    /// walk cannot ENTER is still listed, once, under its owner.** `collect_md_paths` skips a
    /// dot-named entry and a junction on descent but reads either as a START directory, so the
    /// pre-fix loop (a walk from every own root) listed such a library's molds and the plain
    /// top-level-roots rule silently dropped them. `A` and `B` nested inside `.archive` check the
    /// nested-in-nested case too: `B` must be listed once, under `B`, not again from `A`'s walk.
    /// Written RED first against the top-level-only rule.
    #[test]
    fn pj454_scan_reaches_a_nested_own_library_the_parent_walk_cannot_enter() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let a = root.join(".archive").join("A");
        let b = a.join("B");
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(root.join("Root mold.md"), REAL_MOLD).unwrap();
        std::fs::write(a.join("A mold.md"), REAL_MOLD).unwrap();
        std::fs::write(b.join("B mold.md"), REAL_MOLD).unwrap();
        let libs = vec![test_lib("Root", root), test_lib("A", &a), test_lib("B", &b)];
        let out = scan_stamped_molds_in(&libs, &Default::default(), &Default::default());
        let mut seen: Vec<(&str, &str)> = out
            .iter()
            .map(|c| (c.library.as_str(), Path::new(&c.path).file_name().unwrap().to_str().unwrap()))
            .collect();
        seen.sort();
        assert_eq!(
            seen,
            [("A", "A mold.md"), ("B", "B mold.md"), ("Root", "Root mold.md")],
            "every own library's molds, each once, under its owner"
        );
    }

    /// PJ-454 — **a linked universe's file under the own root is never a candidate** (write
    /// sovereignty). The collector fences a root that carries a universe manifest, but a linked
    /// universe's pre-MIG-108 external library, or a legacy bare manifest without a `name`, is a
    /// plain folder to the walk; only the post-collection `foreign` drop keeps it out. Nothing else
    /// in this module exercises a non-empty `foreign`.
    #[test]
    fn pj454_scan_drops_a_linked_universe_file_that_sits_under_the_root() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let linked = root.join("Linked"); // no manifest — a plain folder to the collector
        std::fs::create_dir_all(&linked).unwrap();
        std::fs::write(root.join("Root mold.md"), REAL_MOLD).unwrap();
        std::fs::write(linked.join("Linked mold.md"), REAL_MOLD).unwrap();
        let libs = vec![test_lib("Root", root)];
        // As `foreign_library_roots` spells its set: forward slashes, no trailing slash, lowercase.
        let foreign: HashSet<String> =
            [linked.to_string_lossy().replace('\\', "/").trim_end_matches('/').to_lowercase()]
                .into_iter()
                .collect();
        let out = scan_stamped_molds_in(&libs, &foreign, &Default::default());
        let names: Vec<&str> =
            out.iter().map(|c| Path::new(&c.path).file_name().unwrap().to_str().unwrap()).collect();
        assert_eq!(names, ["Root mold.md"], "the linked universe's mold must be dropped");
    }

    /// PJ-454 — **the on-demand reproduction against a REAL registry, read-only.** Point it at a
    /// universe's `libraries.json` and it prints what the door would list: rows, distinct paths,
    /// paths listed more than once. On the verbatim loop Eisa Universe printed 74 / 39 / 35; with
    /// the top-level-roots rule it prints 39 / 39 / 0. Reads note files only; opens no database,
    /// writes nothing. `foreign` is empty here (no `AppHandle`), so a linked universe's molds
    /// would appear only if its root were not manifest-fenced by the collector.
    ///
    /// ```text
    ///   PJ454_LIBS="E:/…/Universe/.constellation/libraries.json" \
    ///     cargo test --lib pj454_scan_real_registry_read_only -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore]
    fn pj454_scan_real_registry_read_only() {
        let Ok(reg) = std::env::var("PJ454_LIBS") else {
            eprintln!("[pj454] PJ454_LIBS unset — skipping");
            return;
        };
        let libs = crate::libraries::try_load_libraries_at(Path::new(&reg)).expect("a readable libraries.json");
        let out = scan_stamped_molds_in(&libs, &Default::default(), &Default::default());
        let mut counts = std::collections::HashMap::<String, usize>::new();
        for c in &out {
            // Normalized the way the boundary helpers spell paths, so two rows for one file that
            // differ only in separator (two roots registered in different styles) still count as one.
            *counts.entry(c.path.replace('\\', "/").to_lowercase()).or_default() += 1;
        }
        let dup = counts.values().filter(|&&n| n > 1).count();
        eprintln!("[pj454] rows {} / distinct {} / listed more than once {}", out.len(), counts.len(), dup);
        for c in &out {
            eprintln!("  {} | {} | {}", c.library, c.path, c.cid_cn);
        }
    }

    /// The three delicate shapes must be flagged for the first batch; an ordinary mold must not be.
    #[test]
    fn pj454_needs_care_flags_the_three_delicate_shapes() {
        // ordinary — not delicate
        assert!(!needs_care(REAL_MOLD));
        // Templater-only stamp
        assert!(needs_care("---\ncreated: <% tp.file.creation_date() %>\ncid_cn: 20251229T125213Z_NOTE_7C1D\n---\nb\n"));
        // no final newline
        assert!(needs_care("---\ncreated: \"{{date}}\"\ncid_cn: 20260414T152113Z_NOTE_0007\n---\nbody"));
        // a `---` divider in the body
        assert!(needs_care("---\ncreated: \"{{date}}\"\ncid_cn: 20260414T152113Z_NOTE_264E\n---\nintro\n\n---\n\nmore\n"));
    }

    // ── PJ-466 — the outcome classification ───────────────────────────────────────────────────

    /// The source text of `repair_one`, for the tests that assert against the SHIPPING code rather
    /// than a paraphrase of it. One helper, so a signature change breaks one place instead of three
    /// — which it just did: the marker was `fn repair_one(app: &tauri::AppHandle` until the
    /// function grew a multi-line signature, and all three tests would have panicked.
    fn repair_one_source() -> &'static str {
        let src = include_str!("mold_repair.rs");
        let start = src.find("fn repair_one(").expect("repair_one moved");
        let end = src[start..].find("
fn verify_change(").expect("verify_change moved") + start;
        &src[start..end]
    }

    fn outcome(kind: MoldOutcomeKind, reason: &'static str, backed_up: bool) -> MoldRepairOutcome {
        MoldRepairOutcome { path: "p".into(), kind, reason, detail: String::new(), backed_up }
    }

    /// The whole point of PJ-466: a file that needed nothing is NOT a failure, and the halt reads
    /// `failed`. Before this, `failed = attempted - repaired` put both in the same bucket.
    #[test]
    fn pj466_a_kept_outcome_is_not_counted_as_a_failure() {
        let outcomes = vec![
            outcome(MoldOutcomeKind::Repaired, "repaired", true),
            outcome(MoldOutcomeKind::Kept, "notStamped", false),
            outcome(MoldOutcomeKind::Kept, "gone", false),
            outcome(MoldOutcomeKind::Failed, "writeFailed", true),
        ];
        let t = tally(&outcomes);
        assert_eq!(t.repaired, 1);
        assert_eq!(t.kept, 2, "a second look disagreeing with the first is never an error");
        assert_eq!(t.failed, 1, "only the write failure is owed an account");
        assert_eq!(t.backed_up, 2, "counted, so the receipt never assumes every file was backed up");
        // The defect, stated as the test that would have caught it: subtraction says 3.
        assert_ne!(t.failed, outcomes.len() - t.repaired);
    }

    /// A run in which nothing needed doing must report ZERO failures — the shape that produced
    /// "0 fixed, 1 skipped." over a healthy file and halted the cascade behind it.
    #[test]
    fn pj466_an_all_kept_run_has_nothing_to_halt_on() {
        let outcomes = vec![
            outcome(MoldOutcomeKind::Kept, "notStamped", false),
            outcome(MoldOutcomeKind::Kept, "gone", false),
        ];
        let t = tally(&outcomes);
        assert_eq!(t.failed, 0);
        assert_eq!(t.backed_up, 0);
    }

    /// THE INVARIANT every "nothing was changed" sentence on the receipt rests on: no KEPT outcome
    /// may be produced after anything has been written. Asserted against the shipping source rather
    /// than a paraphrase of it, because the risk is a FUTURE return site added below the write —
    /// which no behavioural test of today's code could see.
    #[test]
    fn pj466_no_kept_outcome_can_be_returned_after_the_first_write() {
        let body = repair_one_source();

        // The first byte written by this function, found in the source rather than assumed.
        let first_write = body.find("std::fs::write(&bpath").expect("the backup write moved");

        // Every call that MINTS a Kept outcome. `keep(` is the only one.
        let mut kept_sites = 0;
        for (idx, _) in body.match_indices("keep(") {
            // skip the closure's own definition
            if body[..idx].ends_with("let ") { continue; }
            kept_sites += 1;
            assert!(
                idx < first_write,
                "a `keep(` at byte {idx} sits AFTER the first write at {first_write} — a Kept \
                 outcome would then be claiming nothing was changed about a file that was"
            );
        }
        assert!(kept_sites >= 2, "expected the gone/notStamped keeps, found {kept_sites}");
    }

    /// The `reason` codes are an IPC contract: the frontend translates them into fifteen languages,
    /// so a code invented in Rust without a matching string ships an untranslated screen. This
    /// fails the moment a new one appears, which is the point.
    #[test]
    fn pj466_reason_codes_are_exactly_the_documented_contract() {
        let body = repair_one_source();

        let documented = [
            "repaired", "repairedIndexLater", "gone", "notStamped", "notOurs", "unreadable",
            "notPreparable", "backupFailed", "backupMismatch", "writeFailed", "readBackFailed",
            "verifyFailed", "verifyNotRestored",
        ];
        // Every code the function actually emits, read out of the source.
        let mut emitted: Vec<&str> = Vec::new();
        for (idx, _) in body.match_indices("\"") {
            let rest = &body[idx + 1..];
            if let Some(end) = rest.find('"') {
                let lit = &rest[..end];
                if documented.contains(&lit) && !emitted.contains(&lit) {
                    emitted.push(lit);
                }
            }
        }
        for code in documented {
            assert!(
                emitted.contains(&code),
                "documented reason code `{code}` is no longer emitted — the contract and the code \
                 have drifted, and the frontend has a string nothing produces"
            );
        }
        assert_eq!(
            emitted.len(),
            documented.len(),
            "emitted {emitted:?} vs documented {documented:?}"
        );
    }

    /// The rollback used to be `let _ = std::fs::write(...)` under a message that asserted
    /// "so it was undone" regardless. Pinned in the source: the restore's result is now consumed.
    #[test]
    fn pj466_the_verify_rollback_is_checked() {
        let body = repair_one_source();
        assert!(
            !body.contains("let _ = std::fs::write(path, before.as_bytes())"),
            "the rollback discards its Result again — the receipt would assert an undo nobody checked"
        );
        // (that `verifyNotRestored` is EMITTED is pinned by
        // `pj466_reason_codes_are_exactly_the_documented_contract`, which reads the same slice.)
    }
}

/// `cid_cn` → how many notes link to it BY IDENTITY.
///
/// Best-effort: an unavailable index yields an empty map, and the caller then reports zero rather
/// than failing the scan. The count is shown to the user and drives the post-repair re-read; it is
/// never a gate, because a link by identity does not make a mold a note.
fn inbound_cid_counts(app: &tauri::AppHandle, cids: &[String]) -> BTreeMap<String, u32> {
    let mut map = BTreeMap::new();
    if cids.is_empty() {
        return map;
    }
    use tauri::Manager as _;
    let state = app.state::<crate::search::SearchState>();
    // COUNT(DISTINCT source_path), not COUNT(*): the label says "Linked from N notes", and one
    // note linking twice is still one note. Bounded to the cids the scan found, so the distinct
    // pass touches ~43 identities rather than the whole link table — see `scan_stamped_molds`.
    let placeholders = std::iter::repeat("?").take(cids.len()).collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT target_cid_cn, COUNT(DISTINCT source_path) FROM note_links \
         WHERE target_cid_cn IN ({placeholders}) GROUP BY target_cid_cn"
    );
    let _ = crate::search::with_read_conn(&state, |conn| {
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(cids.iter()), |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            map.insert(row.0, row.1 as u32);
        }
        Ok(())
    });
    map
}
