//! MIG-021v2 §1F'.b — Bulk approve/reject for the Source Review queue.
//!
//! After the §1F' background scan, a fresh Universe ends up with
//! thousands of pending suggestions. Reviewing each one by hand is
//! impractical for a large vault — the user needs a "clear the queue"
//! affordance.
//!
//! Two operations:
//!   * `sources_accept_all_pending` — for every pending record, write
//!     ALL of its top-3 suggestions per axis to the note's frontmatter
//!     and clear the suggestion row. Mirrors the per-card Accept's
//!     semantics exactly. Runs on a background thread; emits
//!     `sources:bulk_accept` events with phase = start / progress /
//!     done / cancelled / error.
//!   * `sources_reject_all_pending` — single SQL DELETE; instant.
//!     Returns the count of rows cleared so the UI can show a toast.
//!
//! Cooperative cancellation via an AtomicBool checked between records.
//! Per-record errors are logged but don't abort the loop — one corrupt
//! note shouldn't block clearing the other 6,000.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

use super::{
    clear_suggestions, extract_content_type, extract_sources, is_valid_content_type_id,
    is_valid_source_id, read_suggestions, rewrite_frontmatter_content_type,
    rewrite_frontmatter_sources, union_preserve_order, write_content_type_to_db,
    write_sources_to_db, Suggestion,
};

#[derive(Default)]
pub struct BulkAcceptState {
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    pub completed: AtomicUsize,
    pub total: AtomicUsize,
    pub last_error: Mutex<Option<String>>,
}

impl BulkAcceptState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Serialize, Clone)]
pub struct BulkAcceptStatus {
    pub running: bool,
    pub cancelling: bool,
    pub completed: usize,
    pub total: usize,
    pub last_error: Option<String>,
}

#[derive(Serialize, Clone)]
struct BulkAcceptEvent {
    phase: String,
    total: usize,
    completed: usize,
    error: Option<String>,
}

/// Snapshot the bulk-accept state. Called by the frontend on mount or
/// after a missed event.
#[tauri::command]
pub fn sources_bulk_accept_status(app: AppHandle) -> BulkAcceptStatus {
    let state = app.state::<BulkAcceptState>();
    BulkAcceptStatus {
        running: state.running.load(Ordering::Relaxed),
        cancelling: state.cancel.load(Ordering::Relaxed),
        completed: state.completed.load(Ordering::Relaxed),
        total: state.total.load(Ordering::Relaxed),
        last_error: state
            .last_error
            .lock()
            .ok()
            .and_then(|g| g.clone()),
    }
}

/// Set the cancel flag on a running bulk-accept. Returns immediately;
/// the worker thread observes it on its next iteration.
#[tauri::command]
pub fn sources_bulk_accept_cancel(app: AppHandle) -> Result<(), String> {
    let state = app.state::<BulkAcceptState>();
    state.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

/// Kick off bulk-accept on every pending suggestion in the queue.
/// Idempotent: returns an error if already running.
///
/// Each record is committed exactly the way per-card Accept does:
/// `sources:` gets every horizontal suggestion ID and `content_type:`
/// gets every vertical suggestion ID. The user can subsequently trim
/// via the PropertyEditor pickers.
///
/// V3-§8.r5.5 (audit UX agent): added `skip_split` parameter (default
/// true from the frontend). When true, cards whose composite_json
/// reports a Split regime on either axis are EXCLUDED from the bulk
/// accept — the engine "refused to assign" on those cards and
/// auto-applying the top suggestion would defeat the Sibling
/// Disambiguation design. They stay in the queue for the user to
/// resolve via the radio-chip form.
// App-freeze audit Batch-D (2026-07-03): ensure_search_db_ready moved INSIDE
// the spawned worker (it parked the dispatch thread for the whole cold init);
// the worker's error path covers an ensure failure.
#[tauri::command]
pub fn sources_accept_all_pending(app: AppHandle, skip_split: Option<bool>) -> Result<(), String> {
    let state = app.state::<BulkAcceptState>();
    if state.running.swap(true, Ordering::Relaxed) {
        return Err("Bulk accept already running".into());
    }
    state.cancel.store(false, Ordering::Relaxed);
    state.completed.store(0, Ordering::Relaxed);
    state.total.store(0, Ordering::Relaxed);
    if let Ok(mut g) = state.last_error.lock() {
        *g = None;
    }

    let skip_split_flag = skip_split.unwrap_or(true);
    let app_clone = app.clone();
    thread::spawn(move || {
        let result = crate::search::ensure_search_db_ready(&app_clone)
            .and_then(|_| run_bulk_accept(app_clone.clone(), skip_split_flag));
        let state = app_clone.state::<BulkAcceptState>();
        if let Err(e) = result {
            if let Ok(mut g) = state.last_error.lock() {
                *g = Some(e.clone());
            }
            let _ = app_clone.emit(
                "sources:bulk_accept",
                BulkAcceptEvent {
                    phase: "error".into(),
                    total: state.total.load(Ordering::Relaxed),
                    completed: state.completed.load(Ordering::Relaxed),
                    error: Some(e),
                },
            );
        }
        state.running.store(false, Ordering::Relaxed);
        state.cancel.store(false, Ordering::Relaxed);
    });

    Ok(())
}

fn run_bulk_accept(app: AppHandle, skip_split: bool) -> Result<(), String> {
    // Snapshot the queue once up front so we don't race against new
    // entries being added (e.g. a scan still running in the background).
    // V3-§8.r5.5: when skip_split is true, also pull composite_json so
    // we can filter out Split-regime cards in Rust.
    let pending_paths: Vec<String> = {
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard
            .as_ref()
            .ok_or("Search database not initialized")?;
        let sql = if skip_split {
            "SELECT note_path, composite_json FROM sources_suggestions ORDER BY created_at ASC"
        } else {
            "SELECT note_path, NULL FROM sources_suggestions ORDER BY created_at ASC"
        };
        let mut stmt = conn.prepare(sql).map_err(|e| format!("prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1).ok().flatten(),
                ))
            })
            .map_err(|e| format!("query: {}", e))?;
        let mut out = Vec::new();
        for r in rows {
            let (path, composite_json) = r.map_err(|e| format!("row: {}", e))?;
            if skip_split {
                if let Some(json) = &composite_json {
                    if has_split_regime(json) {
                        continue; // skip Split-regime cards
                    }
                }
            }
            out.push(path);
        }
        out
    };

    let total = pending_paths.len();
    let state = app.state::<BulkAcceptState>();
    state.total.store(total, Ordering::Relaxed);

    let _ = app.emit(
        "sources:bulk_accept",
        BulkAcceptEvent {
            phase: "start".into(),
            total,
            completed: 0,
            error: None,
        },
    );

    if total == 0 {
        let _ = app.emit(
            "sources:bulk_accept",
            BulkAcceptEvent {
                phase: "done".into(),
                total: 0,
                completed: 0,
                error: None,
            },
        );
        return Ok(());
    }

    let mut completed: usize = 0;
    // PJ-187 — notes whose frontmatter landed on disk but whose search entry could not be
    // updated. Counted, not swallowed: the batch still succeeds, but the panel is told.
    let mut mirror_failures: usize = 0;
    // 2026-07-24 inspection (APP-KILLER). Every note whose frontmatter this batch
    // actually rewrote must be ANNOUNCED: the write goes through the gate, which
    // marks the path watcher-suppressed, so without this an OPEN note keeps its
    // open-time frontmatter base and its next debounced save silently erases the
    // accepted `sources:` / `content_type:` blocks from disk.
    //
    // 2026-08-01 inspection (HIGH, same class). The announce used to ride the
    // `% 5` PROGRESS boundary, so a written note could wait up to four more notes'
    // worth of file I/O before the frontend heard about it — and a debounced editor
    // save firing inside that window composes from the STALE base, erases the
    // just-accepted blocks, and leaves the later announce a byte-identical no-op.
    // The DELAY is the bug, so the announce now leaves Rust in the SAME loop
    // iteration as the disk write; only the progress event is still throttled.
    //
    // Why one emit per written note does not violate Performance Rule 3:
    //   * Rust cannot tell which notes are OPEN — verified, not assumed: no managed
    //     state in lib.rs holds an open-note registry, no command reports open tabs,
    //     and the only tab snapshot on disk (MIG-100 `.constellation/session.json`)
    //     is a frontend-written ~1s debounce that is DELETED outright when the
    //     "remember tabs" toggle is off. With no cheap discriminator, announcing only
    //     the first N notes would leave every open note deeper in the queue exposed
    //     to exactly this bug — a half-fix for an app-killer class.
    //   * The emit count is bounded by the notes the user explicitly chose to accept,
    //     and each one is gated behind a per-note fsync'd read-modify-write — nothing
    //     like the per-keystroke IPC storm Rule 3 forbids.
    //   * Both listeners early-return when none of the announced paths is open
    //     (`adoptExternalChangeIntoTabs` in +layout.svelte, `adoptFreshDiskIntoSS` in
    //     SecondScreenPage), and the expensive half — reindex + stats + cache refresh —
    //     stays coalesced behind the frontend's existing 300ms watcher-flush debounce,
    //     so more events do NOT multiply the work.
    // The buffer stays: it is what makes double-announcing impossible and keeps every
    // exit path (cancel, post-loop) drained by construction.
    let mut announce_pending = AnnounceBuffer::default();
    for note_path in pending_paths {
        if state.cancel.load(Ordering::Relaxed) {
            flush_announce(&app, &mut announce_pending);
            let _ = app.emit(
                "sources:bulk_accept",
                BulkAcceptEvent {
                    phase: "cancelled".into(),
                    total,
                    completed,
                    error: None,
                },
            );
            return Ok(());
        }

        match accept_one(&app, &note_path) {
            Ok(outcome) => {
                // ANNOUNCE FIRST — the bytes are on disk regardless of what step 4 did —
                // and announce NOW: an open note's re-base must not queue behind the next
                // four notes' file I/O (see the schedule note above). A no-op accept
                // recorded nothing, so this is a no-op emit for it.
                announce_pending.record(&note_path, outcome.wrote);
                flush_announce(&app, &mut announce_pending);
                if let Some(e) = outcome.mirror_error {
                    mirror_failures += 1;
                    if let Ok(mut g) = state.last_error.lock() {
                        *g = Some(format!("{}: {}", note_path, e));
                    }
                }
            }
            Err(e) => {
                // Record but don't abort.
                if let Ok(mut g) = state.last_error.lock() {
                    *g = Some(format!("{}: {}", note_path, e));
                }
            }
        }

        completed += 1;
        state.completed.store(completed, Ordering::Relaxed);

        // Throttle the PROGRESS event per Performance Rule 3. Only the progress
        // counter rides this boundary — the announce above must not, because a
        // deferred re-base is how the accepted blocks get erased.
        if completed % 5 == 0 || completed == total {
            let _ = app.emit(
                "sources:bulk_accept",
                BulkAcceptEvent {
                    phase: "progress".into(),
                    total,
                    completed,
                    error: None,
                },
            );
        }
    }

    // Belt-and-braces: the loop now flushes every iteration, so this is normally a
    // no-op — it stays because the drain contract must hold on EVERY exit path,
    // including any future early `break` added above.
    flush_announce(&app, &mut announce_pending);

    let _ = app.emit(
        "sources:bulk_accept",
        BulkAcceptEvent {
            phase: "done".into(),
            total,
            completed,
            error: if mirror_failures > 0 {
                Some(format!(
                    "{} note(s) were updated on disk, but their search entries could not be refreshed.",
                    mirror_failures
                ))
            } else {
                None
            },
        },
    );
    Ok(())
}

/// Bookkeeping for the announce, kept PURE (no Tauri) so the two properties that
/// actually matter are unit-testable: **every note whose frontmatter was rewritten is
/// announced exactly once**, and **it is announced in its OWN iteration** — never
/// deferred behind another note's progress (2026-08-01 inspection). Whichever exit
/// path the run takes — its own iteration, cancellation, or normal completion —
/// nothing is left buffered. The emit itself is a one-liner re-using the event shape
/// already proven at the four per-card seams; the drain schedule is the part that can
/// silently strand a path, and a stranded path is the app-killer coming straight back.
#[derive(Default)]
struct AnnounceBuffer {
    pending: Vec<String>,
}

impl AnnounceBuffer {
    /// A no-op accept (`wrote == false`) changed no bytes, so there is nothing for an
    /// open note to re-base from — recording it would emit a pointless reload.
    fn record(&mut self, note_path: &str, wrote: bool) {
        if wrote {
            self.pending.push(note_path.to_string());
        }
    }

    fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Hand back everything buffered and reset. Draining rather than peeking is what
    /// makes double-announcing impossible.
    fn drain(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending)
    }
}

/// Announce whatever frontmatter writes are buffered as ONE `library-changed` event
/// and clear the buffer (in the accept loop that is the single note just written —
/// the buffer is drained every iteration; the multi-path form is what keeps the
/// cancel and post-loop exits correct by construction). Empty buffer → no event.
/// Re-uses the existing adopt path (`adoptExternalChangeIntoTabs`):
/// a CLEAN open model adopts the new bytes, a DIRTY one keeps its unsaved work and
/// preserves the incoming change to a `.conflict` sidecar.
fn flush_announce(app: &AppHandle, buffer: &mut AnnounceBuffer) {
    let batch = buffer.drain();
    if batch.is_empty() {
        return;
    }
    let refs: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();
    super::announce_frontmatter_writes(app, &refs);
}

/// Returns `true` when the note's frontmatter actually changed on disk — the
/// caller batches an announce for those paths so an OPEN note re-bases instead of
/// silently overwriting the accepted blocks on its next save (2026-07-24
/// inspection, APP-KILLER).
/// What accepting ONE note produced.
///
/// PJ-187 — `wrote` and `mirror_error` are deliberately separate. Step 3 puts the accepted
/// `sources:` / `content_type:` blocks on DISK through the gate, which marks the path
/// watcher-suppressed; step 4 only mirrors that into SQLite. When step 4 used `?`, a mirror
/// failure returned `Err` and the caller skipped `announce_pending.record` — so an OPEN note kept
/// its open-time frontmatter base and its next debounced save ERASED from disk the blocks the user
/// had just accepted. The announce must follow the DISK write, never the mirror.
struct AcceptOutcome {
    wrote: bool,
    mirror_error: Option<String>,
}

fn accept_one(app: &AppHandle, note_path: &str) -> Result<AcceptOutcome, String> {
    // ★ MIG-111 §0.4 (R1) — Approve-All is the FIFTH sources writer, and it reaches disk through
    // its own `gate_rmw` rather than the two guarded helpers in `mod.rs`.
    //
    // It was missed twice over. First because §0.4 guarded the writers it went looking for and
    // this one lives in a sibling file; then because the test written to prove no command could
    // write around the helpers counted `gate_rmw` occurrences in `mod.rs` ALONE — so it read
    // green over exactly the bypass it was named for.
    //
    // The consequence was the ugliest asymmetry in the boundary: per-card Accept correctly
    // REFUSED a linked universe's note while Approve-All silently injected `sources:` and
    // `content_type:` into that same note on disk, watcher-suppressed, counted as a completed
    // batch with no error — the safe path erroring and the bulk path writing.
    crate::libraries::require_own_library(app, note_path)?;
    // 1. Read the suggestion record from the queue.
    let record_opt = {
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard
            .as_ref()
            .ok_or("Search database not initialized")?;
        read_suggestions(conn, note_path)?
    };
    let record = match record_opt {
        Some(r) => r,
        None => {
            // already cleared by another path; nothing to do
            return Ok(AcceptOutcome { wrote: false, mirror_error: None });
        }
    };

    // 2. Split by axis + validate (defense-in-depth: strip any IDs the
    //    classifier emitted that aren't in the current taxonomy).
    let horizontal_ids: Vec<String> = record
        .suggestions
        .iter()
        .filter(|s: &&Suggestion| s.axis == "horizontal" && is_valid_source_id(&s.source))
        .map(|s| s.source.clone())
        .collect();
    let vertical_ids: Vec<String> = record
        .suggestions
        .iter()
        .filter(|s: &&Suggestion| s.axis == "vertical" && is_valid_content_type_id(&s.source))
        .map(|s| s.source.clone())
        .collect();

    // 3. Rewrite BOTH axes' frontmatter as ONE locked read-modify-write (gate_rmw), so a concurrent
    //    editor save can land before or after but NEVER inside the window. PJ-071: the bulk path was
    //    the last source-accept still on the racy unlocked-read + gate_write — the per-card path
    //    already moved to gate_rmw (sources/mod.rs::rewrite_note_sources_on_disk). The closure is
    //    pure string work: NO gate_* and NO DB lock inside it (gate_rmw's two hard rules) — the
    //    SQLite-mirror update is step 4, after this returns. Idempotent rewrite → Ok(None), no write.
    let path = std::path::Path::new(note_path);
    if !path.exists() {
        return Err(format!("Note not found: {}", note_path));
    }
    // PJ-091: accept ENRICHES, it must not SUBTRACT. Union each axis's suggestion
    // with the note's CURRENT on-disk values (read inside the gate — race-free),
    // so Approve-All never drops a source/type the user set by hand after this
    // suggestion was queued. `merged_*` are stashed for the step-4 DB mirror so
    // note_meta reflects exactly what landed on disk.
    let mut merged_h = Vec::new();
    let mut merged_v = Vec::new();
    let outcome = crate::write_gate::gate_rmw(path, "bulk_accept", |content| {
        merged_h = union_preserve_order(&extract_sources(content), &horizontal_ids);
        merged_v = union_preserve_order(&extract_content_type(content), &vertical_ids);
        let after_h = rewrite_frontmatter_sources(content, &merged_h);
        let after_both = rewrite_frontmatter_content_type(&after_h, &merged_v);
        Ok(if after_both == content { None } else { Some(after_both) })
    })?;
    // Did the bytes on disk actually change? `gate_rmw` returns OkUnchecked for an
    // idempotent no-op (nothing written) and Ok when it wrote. Only a real write
    // needs announcing — see the caller, which batches the announce.
    let wrote = matches!(outcome, crate::write_gate::WriteOutcome::Ok);

    // 4. Update the SQLite mirror + clear the suggestion row. The disk write above has already
    //    happened and is irreversible from here, so NOTHING in this step may abort the outcome —
    //    every error is captured and handed back for the caller to count and surface.
    let mirror_error = mirror_to_db(app, note_path, &merged_h, &merged_v).err();

    Ok(AcceptOutcome {
        wrote,
        mirror_error,
    })
}

/// Step 4 in isolation, so its failures are values rather than early returns.
fn mirror_to_db(
    app: &AppHandle,
    note_path: &str,
    merged_h: &[String],
    merged_v: &[String],
) -> Result<(), String> {
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    write_sources_to_db(conn, note_path, merged_h)?;
    write_content_type_to_db(conn, note_path, merged_v)?;
    clear_suggestions(conn, note_path)?;
    Ok(())
}

/// V3-§8.r5.5: Check if a composite_json blob reports a Split regime
/// on either axis. Defensive: malformed/missing JSON returns false
/// (don't skip cards we can't read — better to bulk-accept than to
/// silently leave them in the queue forever).
fn has_split_regime(composite_json: &str) -> bool {
    let value: serde_json::Value = match serde_json::from_str(composite_json) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let regime_is_split = |key: &str| -> bool {
        value
            .get(key)
            .and_then(|v| v.get("regime"))
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("Split"))
            .unwrap_or(false)
    };
    regime_is_split("horizontal") || regime_is_split("vertical")
}

/// Clear EVERY pending suggestion from the queue without writing to
/// any frontmatter. Synchronous + instant; returns the count of rows
/// cleared so the UI can show a toast.
///
/// Reversible in the loose sense that the user can re-run the
/// classifier scan to regenerate suggestions; not reversible in the
/// strict sense (the originally-suggested IDs are gone).
// Note-open-freeze Batch-2 §B2-2 (2026-07-03): `(async)` — off the IPC dispatch thread.
// Discovery-verified async-only-safe: DB-only / mutex-covered body, no note-file writes,
// all callers await. See SESSION-LOG-2026-07-03 (Architect findings).
#[tauri::command(async)]
pub fn sources_reject_all_pending(app: AppHandle) -> Result<usize, String> {
    crate::search::ensure_search_db_ready(&app)?;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    let n = conn
        .execute("DELETE FROM sources_suggestions", [])
        .map_err(|e| format!("delete: {}", e))?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::AnnounceBuffer;

    /// Replays `run_bulk_accept`'s announce schedule against a scripted run, returning
    /// each emitted batch paired with the `completed` counter at the moment it was
    /// emitted. Mirrors the loop exactly: record + flush INSIDE the iteration that
    /// wrote the note (2026-08-01 — the announce no longer rides the `% 5` progress
    /// boundary), flush on cancel, flush after the loop.
    ///
    /// `outcomes` is one entry per queued note: `Some(true)` wrote, `Some(false)` was a
    /// no-op, `None` errored. `cancel_before` cancels the run before that index.
    fn replay(outcomes: &[Option<bool>], cancel_before: Option<usize>) -> Vec<(usize, Vec<String>)> {
        let mut buf = AnnounceBuffer::default();
        let mut batches: Vec<(usize, Vec<String>)> = Vec::new();
        let mut completed = 0usize;
        fn flush(buf: &mut AnnounceBuffer, completed: usize, out: &mut Vec<(usize, Vec<String>)>) {
            let b = buf.drain();
            if !b.is_empty() {
                out.push((completed, b));
            }
        }

        for (i, outcome) in outcomes.iter().enumerate() {
            if cancel_before == Some(i) {
                flush(&mut buf, completed, &mut batches);
                return batches; // the cancel path returns immediately
            }
            let path = format!("note{}.md", i);
            match outcome {
                Some(wrote) => buf.record(&path, *wrote),
                None => {} // an error records nothing, exactly like the real loop
            }
            completed += 1;
            // The announce leaves in this iteration; only the progress event is
            // throttled on `completed % 5 == 0 || completed == total`.
            flush(&mut buf, completed, &mut batches);
        }
        flush(&mut buf, completed, &mut batches);
        batches
    }

    fn announced(batches: &[(usize, Vec<String>)]) -> Vec<String> {
        batches.iter().flat_map(|(_, b)| b.iter().cloned()).collect()
    }

    #[test]
    fn only_actual_writes_are_announced() {
        let mut buf = AnnounceBuffer::default();
        buf.record("a.md", true);
        buf.record("b.md", false); // idempotent no-op — nothing changed on disk
        buf.record("c.md", true);
        assert_eq!(buf.drain(), vec!["a.md".to_string(), "c.md".to_string()]);
    }

    #[test]
    fn draining_twice_cannot_double_announce() {
        let mut buf = AnnounceBuffer::default();
        buf.record("a.md", true);
        assert_eq!(buf.drain().len(), 1);
        assert!(buf.is_empty());
        assert!(buf.drain().is_empty());
    }

    /// THE FIRST PROPERTY. Whatever the queue length, every written path is announced
    /// exactly once, in queue order — a stranded path (never announced) and a doubled
    /// one (a peek instead of a drain) are both regressions of the 2026-07-24 app-killer.
    #[test]
    fn every_write_is_announced_exactly_once_for_any_queue_length() {
        for total in 0..40usize {
            let outcomes: Vec<Option<bool>> = (0..total)
                .map(|i| match i % 4 {
                    0 => Some(false), // no-op
                    1 => None,        // error
                    _ => Some(true),  // wrote
                })
                .collect();
            let expected: Vec<String> = outcomes
                .iter()
                .enumerate()
                .filter(|(_, o)| **o == Some(true))
                .map(|(i, _)| format!("note{}.md", i))
                .collect();

            let batches = replay(&outcomes, None);
            let got = announced(&batches);
            assert_eq!(got, expected, "queue length {}", total);

            let mut sorted = got.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), got.len(), "duplicate announce at length {}", total);
        }
    }

    /// THE SECOND PROPERTY (2026-08-01 inspection, HIGH). A note's announce leaves in
    /// the SAME iteration as its disk write — never queued behind other notes' progress.
    /// The old `% 5` schedule failed this: `note0.md` was not announced until four more
    /// notes had been rewritten, and a debounced editor save landing in that window
    /// composes from the stale base, erases the just-accepted blocks, and reduces the
    /// late announce to a byte-identical no-op.
    #[test]
    fn a_write_is_announced_in_its_own_iteration_never_deferred() {
        // Interleave no-ops and errors so the property is not an artefact of a
        // write-every-time run: whatever else the loop does, a written note's announce
        // lands at `completed == its own index + 1`.
        let outcomes: Vec<Option<bool>> = (0..23)
            .map(|i| match i % 4 {
                0 => Some(false), // no-op
                1 => None,        // error
                _ => Some(true),  // wrote
            })
            .collect();
        let batches = replay(&outcomes, None);
        for (completed_at, batch) in &batches {
            assert_eq!(batch.len(), 1, "an announce carried more than its own note: {:?}", batch);
            let idx: usize = batch[0]
                .trim_start_matches("note")
                .trim_end_matches(".md")
                .parse()
                .unwrap();
            assert_eq!(
                *completed_at,
                idx + 1,
                "note{} was announced after {} completions — deferred behind {} other note(s)",
                idx,
                completed_at,
                completed_at - idx - 1
            );
        }
        // …and the run really did announce every write (the loop above is vacuous otherwise).
        assert_eq!(batches.len(), outcomes.iter().filter(|o| **o == Some(true)).count());
    }

    /// Cancelling must not strand the writes already made — they are on disk, so an
    /// open note still has to re-base or it will overwrite them on the next keystroke.
    #[test]
    fn cancelling_still_announces_what_was_already_written() {
        // 12 notes, all written, cancelled just before index 7: 0..=6 are on disk.
        let outcomes: Vec<Option<bool>> = (0..12).map(|_| Some(true)).collect();
        let batches = replay(&outcomes, Some(7));
        let got = announced(&batches);
        let expected: Vec<String> = (0..7).map(|i| format!("note{}.md", i)).collect();
        assert_eq!(got, expected);
    }

    /// The window a cancel used to expose: under the `% 5` schedule, notes 5 and 6 were
    /// written after the flush at 5 and rescued only by the cancel-path drain. Nothing is
    /// buffered at all now, so the cancel drain is a no-op — but the property it guarded
    /// (a written note is announced before the run can exit) still has to hold.
    #[test]
    fn cancel_after_the_old_modulus_window_strands_nothing() {
        let outcomes: Vec<Option<bool>> = (0..10).map(|_| Some(true)).collect();
        let batches = replay(&outcomes, Some(7));
        let got = announced(&batches);
        assert!(got.contains(&"note5.md".to_string()));
        assert!(got.contains(&"note6.md".to_string()));
        assert_eq!(got.len(), 7);
    }

    #[test]
    fn an_all_noop_run_emits_nothing() {
        let outcomes: Vec<Option<bool>> = (0..9).map(|_| Some(false)).collect();
        assert!(replay(&outcomes, None).is_empty());
    }
}
