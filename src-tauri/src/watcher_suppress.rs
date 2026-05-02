//! §3-redo.2 — Watcher loop suppression for the wikilink rename cascade.
//!
//! When the cascade walker rewrites a source note's body to update its
//! wikilink targets, the `fs::write` bubbles back through `notify` as an
//! external-edit event ([`watcher.rs:52-83`](watcher.rs)). Without filtering, that
//! event triggers the frontend's `library-changed` listener, which reloads the
//! affected file from disk and re-emits the cascade flow — an infinite loop
//! between cascade and watcher.
//!
//! This module is the suppression primitive. The cascade walker calls
//! [`mark`] immediately before each `fs::write`. The watcher's emit path
//! checks [`was_recent`] and skips paths whose write was within the
//! TTL window.
//!
//! Closes the F3-watcher-loop failure mode defined in the Rename Function
//! Concept Paper (P4 / D2). Anchored to the Architect plan
//! `lab/reports/MIG-006-3-REDO-ARCHITECT.md` Step §3-redo.2.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// 2.5 seconds — covers two debounced autosave cycles plus a safety margin.
/// Tuned in the original §3 plan; locking the constant here so future readers
/// know not to lower it without verifying the autosave-vs-cascade interaction.
const TTL: Duration = Duration::from_millis(2_500);

fn map() -> &'static Mutex<HashMap<PathBuf, Instant>> {
    static CELL: OnceLock<Mutex<HashMap<PathBuf, Instant>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Mark `path` as recently-written by the cascade walker. The watcher's emit
/// path will skip emit for this path until the TTL expires.
///
/// Safe to call from any thread. A poisoned mutex (extremely rare —
/// only happens if a thread panics while holding the lock) silently
/// no-ops; the watcher will then emit a spurious external-edit event,
/// which is no worse than the pre-§3 state.
pub fn mark(path: &Path) {
    if let Ok(mut guard) = map().lock() {
        guard.insert(path.to_path_buf(), Instant::now());
    }
}

/// True if `path` was marked within the TTL window.
///
/// §3-redo.6: opportunistic full-map GC on every call — `retain` evicts
/// every stale entry while we hold the lock, not just the queried one.
/// Without the sweep, an entry whose path was marked but never followed
/// by a `was_recent` lookup (e.g. the file was deleted before the OS
/// emitted a notify event) would persist forever; long sessions with
/// many cascades could accumulate stale entries unbounded.
pub fn was_recent(path: &Path) -> bool {
    let Ok(mut guard) = map().lock() else { return false };
    let now = Instant::now();
    guard.retain(|_, stamp| now.duration_since(*stamp) < TTL);
    guard.contains_key(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::thread::sleep;

    #[test]
    fn mark_then_was_recent_returns_true() {
        let p = PathBuf::from("/tmp/test_a.md");
        mark(&p);
        assert!(was_recent(&p));
    }

    #[test]
    fn unmarked_path_returns_false() {
        let p = PathBuf::from("/tmp/test_b.md");
        assert!(!was_recent(&p));
    }

    #[test]
    fn ttl_expiry_returns_false() {
        let p = PathBuf::from("/tmp/test_c.md");
        mark(&p);
        sleep(TTL + Duration::from_millis(100));
        assert!(!was_recent(&p));
    }
}
