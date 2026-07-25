//! IPC arrival trace (Round 6 diagnostic, 2026-04-19).
//!
//! After Round 5's falsification (DashboardView gate → `core_queue_ms`
//! unchanged at 19,418 ms) and the Round 5+ heartbeat diagnostic
//! (`boot_heartbeat_max_gap_ms` = 112 ms → JS event loop is fully
//! responsive during the queue window), the remaining live hypothesis is
//! that the 18.6-second gap between JS `invoke('cache_boot_snapshot_core')`
//! and the Rust body starting lives **inside the Tauri IPC pipeline** —
//! WebView2 host pump, wry's `web_message_received` handler, Tauri's IPC
//! router, or the command dispatcher itself.
//!
//! To locate it, we log a Unix-millisecond timestamp every time a command
//! reaches the `invoke_handler` callback (the earliest Rust-side observable
//! point). The wrapper around `generate_handler!` in `lib.rs` calls
//! [`record`] on every dispatch; the collected log is exposed to the
//! frontend via [`get_perf_trace_log`] and bundled into the boot-perf
//! scorecard JSON so the Debug → Boot Performance UI surfaces the full
//! command arrival timeline.
//!
//! If, during the 18.6-second window, the log shows many command arrivals
//! with interleaved timestamps → UI-thread serialization IS occurring, and
//! we can identify the dominant consumers. If the log shows NO arrivals
//! during the window → the delay is upstream of the dispatcher
//! (WebView2/wry), and the next diagnostic moves to that layer.
//!
//! Footprint is intentionally tiny — one Mutex lock per command dispatch,
//! a (String, u64) append. Mutex contention from concurrent async command
//! bodies is negligible (microseconds); for sync commands running inline
//! on the UI thread, dispatch is already serialized so the mutex is free.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Static append-only log of `(command_name, unix_ms_when_dispatched)`
/// pairs. Grows during the process lifetime; cleared on explicit
/// [`clear_perf_trace_log`] from the frontend.
static TRACE_LOG: Mutex<Vec<(String, u64)>> = Mutex::new(Vec::new());

/// 2026-07-25 PJ-140 #61: cap the trace. It is pushed on EVERY IPC dispatch, has no
/// production clear path, and its only reader runs once at boot — so unbounded it grows
/// for the whole session. Bounding it keeps the earliest (boot-window) entries, which
/// are the diagnostic's entire purpose, and stops the leak. ~4096 short entries ≈ a few
/// hundred KB, fixed.
const MAX_TRACE_ENTRIES: usize = 4096;

/// Append a command arrival record. Called from the `invoke_handler`
/// wrapper in `lib.rs` before the generated dispatcher runs.
pub fn record(cmd: &str) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    if let Ok(mut log) = TRACE_LOG.lock() {
        if log.len() < MAX_TRACE_ENTRIES {
            log.push((cmd.to_string(), ts));
        }
    }
}

/// Clone and return the full trace log. Called from `recordBootPerf` right
/// before writing the boot-perf JSON, so the log is captured at the
/// boot:hydrated boundary.
#[tauri::command]
pub fn get_perf_trace_log() -> Vec<(String, u64)> {
    TRACE_LOG
        .lock()
        .map(|l| l.clone())
        .unwrap_or_default()
}

/// Clear the log. Not currently called from the frontend, but exposed so
/// future diagnostic cycles can reset between measurements without a
/// process restart.
#[tauri::command]
pub fn clear_perf_trace_log() {
    if let Ok(mut log) = TRACE_LOG.lock() {
        log.clear();
    }
}
