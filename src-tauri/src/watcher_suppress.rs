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

/// Threshold at which `mark` and `was_recent` run an opportunistic full-map
/// sweep. Below this, both functions stay O(1) on the steady state — and
/// since every watcher event hits `was_recent`, that O(1) matters: a 256-entry
/// cap keeps the sweep amortized cost negligible while still bounding growth.
const SWEEP_THRESHOLD: usize = 256;

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
///
/// Opportunistic GC: when the map has grown past `SWEEP_THRESHOLD` we evict
/// expired entries before inserting. This bounds growth even when many marks
/// happen with no intervening `was_recent` call (e.g. cascade rewrites a
/// file the OS coalesces or the user deletes before the notify event fires).
pub fn mark(path: &Path) {
    if let Ok(mut guard) = map().lock() {
        if guard.len() >= SWEEP_THRESHOLD {
            let now = Instant::now();
            guard.retain(|_, stamp| now.duration_since(*stamp) < TTL);
        }
        guard.insert(path.to_path_buf(), Instant::now());
    }
}

/// True if `path` was marked within the TTL window.
///
/// Hot path — runs on every watcher event. The cheap path is the common one
/// (path was never marked by the cascade): a single `HashMap::get` returns
/// `None` and we exit. Only when we find an expired entry do we drop it; the
/// full-map sweep runs only when the map has grown past `SWEEP_THRESHOLD`,
/// keeping the steady-state cost O(1) while still bounding the map size.
pub fn was_recent(path: &Path) -> bool {
    let Ok(mut guard) = map().lock() else { return false };
    let Some(stamp) = guard.get(path).copied() else {
        return false;
    };
    let now = Instant::now();
    let fresh = now.duration_since(stamp) < TTL;
    if !fresh {
        guard.remove(path);
        if guard.len() >= SWEEP_THRESHOLD {
            guard.retain(|_, s| now.duration_since(*s) < TTL);
        }
    }
    fresh
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
