//! M9-rss-real — real OS-level RSS (Resident Set Size) probe.
//!
//! Test-only helper for `arabic::bench::m9_bench`. Replaces the
//! on-disk-size proxy with an authoritative number the bench can report
//! alongside the projected-at-7K figure.
//!
//! # Why this needs to exist
//!
//! The original M9 bench reports the on-disk cache bundle size as a
//! proxy for the analyzer's in-memory footprint. That proxy is a
//! **lower** bound — it doesn't include:
//!
//!   * the parsed `fst::Map` state (BurntSushi's Map decodes the header
//!     but keeps the rest of the bytes accessible for traversal);
//!   * the `Vec<GeneratedForm>` side-tables (a `GeneratedForm` today
//!     owns two heap `String`s — `root_key` + `pattern_label` — plus
//!     the `surface` string and a `PatternKind` tag);
//!   * the `OnceLock` singleton overhead + any lazy-init state.
//!
//! For the M9 target ("≤ 100 MB RSS at 7K-root corpus" per `arabic::bench`)
//! a real number is what tells us whether we've made it. On-disk-size
//! tracks direction but not magnitude.
//!
//! # API shape
//!
//! One public function:
//!
//! ```ignore
//! pub fn read_rss_bytes() -> Option<u64>
//! ```
//!
//! Returns the caller process's resident memory in bytes, or `None` if
//! the platform's RSS query fails (rare — only a kernel-level refusal or
//! a missing `/proc`). The bench treats `None` as "skip this line of
//! the report" rather than erroring — the bench must still finish on
//! exotic CI runners that lack the usual probe.
//!
//! # Platform implementations
//!
//! Each backend is behind a `#[cfg(target_os = "…")]` gate with no
//! shared dependencies — a new platform is a new small module. No
//! cross-platform `sysinfo`-style crate, because:
//!
//!   * this is test-only (`#[cfg(test)]`) so a new runtime dep on the
//!     release binary is the wrong trade-off;
//!   * each platform's probe is 10–30 lines of direct OS FFI — cheaper
//!     than the transitive dep surface of a full sysinfo crate;
//!   * the test harness runs in a single-threaded context so the
//!     complexity of sysinfo's cross-platform abstractions isn't paid
//!     for anything.
//!
//! # Precision note
//!
//! The numbers are "dirty" RSS — what the OS thinks the process is
//! currently holding, including pages from shared libraries mapped in,
//! pages read from disk, etc. It's a lower bound on actual memory
//! pressure in the most useful sense: if this number is 100 MB, the
//! kernel will feel at least 100 MB of pressure if something needs
//! memory. For the bench's purpose (tracking Arabic-engine allocations
//! against a budget), that's the correct abstraction.

#![cfg(test)]

/// Read the current process's resident set size in bytes.
///
/// Returns `None` when the OS probe fails — callers MUST tolerate this
/// and degrade gracefully (skip the line in the bench report rather than
/// panicking).
pub fn read_rss_bytes() -> Option<u64> {
    imp::read_rss_bytes()
}

// ──────────────────────────────────────────────────────────────────────
// Windows — PROCESS_MEMORY_COUNTERS via psapi
// ──────────────────────────────────────────────────────────────────────
#[cfg(target_os = "windows")]
mod imp {
    // Direct extern "system" FFI to psapi.dll + kernel32.dll, sized to
    // match the documented PROCESS_MEMORY_COUNTERS layout. Avoids a
    // `windows-sys` dev-dep — this is ~30 lines of trivia that we'd
    // otherwise pay a full crate to pull in.
    use std::mem::{size_of, zeroed};

    #[repr(C)]
    #[derive(Default)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    // K32GetProcessMemoryInfo lives in kernel32.dll on Win7+ (redirected
    // from psapi.dll). Using the K32-prefixed symbol avoids a separate
    // link to psapi.dll — kernel32 is always in the default linker set.
    extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn K32GetProcessMemoryInfo(
            process: *mut std::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }

    pub fn read_rss_bytes() -> Option<u64> {
        // SAFETY: all pointer arguments are to stack memory we own;
        // K32GetProcessMemoryInfo writes up to `cb` bytes into `counters`,
        // which we size exactly to `size_of::<ProcessMemoryCounters>()`.
        unsafe {
            let mut counters: ProcessMemoryCounters = zeroed();
            counters.cb = size_of::<ProcessMemoryCounters>() as u32;
            let ok = K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters as *mut _,
                counters.cb,
            );
            if ok == 0 {
                return None;
            }
            Some(counters.working_set_size as u64)
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Linux — /proc/self/statm
// ──────────────────────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
mod imp {
    use std::fs;

    pub fn read_rss_bytes() -> Option<u64> {
        // statm format: "size resident shared text lib data dt"
        // — all in pages. We want `resident`.
        let contents = fs::read_to_string("/proc/self/statm").ok()?;
        let resident_pages: u64 = contents.split_whitespace().nth(1)?.parse().ok()?;
        // Page size is 4 KiB on every platform we target; querying
        // `sysconf(_SC_PAGESIZE)` would need an extra FFI call for a
        // number that's been 4096 on x86/x86_64/aarch64 Linux for
        // decades. If that ever changes, the bench number is off by a
        // small factor — not a correctness issue.
        Some(resident_pages * 4096)
    }
}

// ──────────────────────────────────────────────────────────────────────
// macOS — mach_task_basic_info
// ──────────────────────────────────────────────────────────────────────
#[cfg(target_os = "macos")]
mod imp {
    use std::mem::{size_of, zeroed};

    // Per <mach/task_info.h> — MACH_TASK_BASIC_INFO = 20, count = 5.
    const MACH_TASK_BASIC_INFO: i32 = 20;
    const MACH_TASK_BASIC_INFO_COUNT: u32 = 5;

    #[repr(C)]
    #[derive(Default)]
    struct MachTaskBasicInfo {
        virtual_size: u64,
        resident_size: u64,
        resident_size_max: u64,
        user_time: [u32; 2],   // time_value_t = { seconds, microseconds }
        system_time: [u32; 2], // time_value_t
        policy: i32,
        suspend_count: i32,
    }

    extern "C" {
        fn mach_task_self() -> u32;
        fn task_info(
            target_task: u32,
            flavor: i32,
            task_info_out: *mut std::ffi::c_void,
            task_info_count: *mut u32,
        ) -> i32;
    }

    pub fn read_rss_bytes() -> Option<u64> {
        // SAFETY: `info` is a stack-local we own; task_info writes at
        // most `count * sizeof(natural_t)` bytes into it; we size
        // `count` to the documented flavor layout.
        unsafe {
            let mut info: MachTaskBasicInfo = zeroed();
            let mut count = MACH_TASK_BASIC_INFO_COUNT;
            let kr = task_info(
                mach_task_self(),
                MACH_TASK_BASIC_INFO,
                &mut info as *mut _ as *mut std::ffi::c_void,
                &mut count,
            );
            if kr != 0 {
                return None;
            }
            Some(info.resident_size)
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Fallback for unknown targets
// ──────────────────────────────────────────────────────────────────────
#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "macos"
)))]
mod imp {
    pub fn read_rss_bytes() -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::read_rss_bytes;

    /// Sanity check: the probe returns `Some` on a supported host and
    /// reports a number in a physically plausible range. We don't pin
    /// the exact value (it varies with test-harness state), just that
    /// it's:
    ///
    ///   * present (the cfg-gated backend compiled and linked);
    ///   * above 1 MiB (any Rust process has allocated at least that);
    ///   * below 100 GiB (ceiling guards against a garbage pointer
    ///     read surfacing as a huge RSS number).
    #[test]
    fn rss_is_plausible_on_supported_host() {
        // Skip on unsupported platforms — the stub returns None by design.
        let Some(bytes) = read_rss_bytes() else {
            // Unsupported target. Not an error — just nothing to check.
            return;
        };
        let mib = bytes as f64 / (1024.0 * 1024.0);
        assert!(
            mib >= 1.0,
            "RSS implausibly low: {mib:.2} MiB — probe likely broken"
        );
        assert!(
            mib < 100.0 * 1024.0,
            "RSS implausibly high: {mib:.2} MiB — probe likely returning garbage"
        );
    }

    /// Two consecutive reads should land in the same order of magnitude.
    /// Allocating inside the test isn't necessary — we're checking the
    /// probe is stable, not tracking allocations. Tolerance is generous
    /// because the bench harness is allowed to page things in/out.
    #[test]
    fn rss_is_stable_across_back_to_back_reads() {
        let Some(a) = read_rss_bytes() else { return };
        let Some(b) = read_rss_bytes() else { return };
        let lo = a.min(b) as f64;
        let hi = a.max(b) as f64;
        assert!(
            hi <= lo * 2.0,
            "RSS varied by more than 2× across two reads \
             ({a} vs {b}) — probe is either racy or broken"
        );
    }
}
