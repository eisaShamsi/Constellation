//! M9 — analyzer benchmark harness.
//!
//! Four measurements, one test, ignored by default so `cargo test --lib`
//! stays fast. Run with:
//!
//! ```bash
//! cargo test --lib --release arabic::bench -- --ignored --nocapture
//! ```
//!
//! # What this measures
//!
//! 1. **Cold-start** — `GenerativeFst::get()` with the on-disk cache
//!    cleared. Exercises `fst_bake::try_load_cached()` → (cache miss) →
//!    `build_bundle()` → `persist_best_effort()` → `from_bundle()`. This
//!    is the first-launch experience after a fresh install, or after a
//!    cache file format bump.
//!
//! 2. **Warm-start** — time to go from "cache file on disk" to "live
//!    `GenerativeFst` in memory" via `load_bundle()` + `from_bytes()`.
//!    The same code path `get()` uses on a warm launch — we can't re-call
//!    `get()` in the same process (`OnceLock`), so we exercise its
//!    constituent steps directly on the file the cold run just wrote.
//!    Target: ≥5× faster than cold.
//!
//! 3. **Throughput** — `analyze_best()` on the 502-case regression
//!    corpus × K iterations, reporting words/sec. Covers every origin
//!    layer (protected, generative, heuristic, foreign). This is the
//!    production path FTS5 tokenisation walks on every Arabic token in
//!    every note — so it includes the `overrides::active()` Arc clone
//!    that M8b added. Target: ≥200 K words/sec.
//!
//! 4. **Accuracy** — `regression::run_corpus()` pass rate, broken down
//!    by origin. Target: ≥92 % pass rate overall.
//!
//! Plus a **size proxy**: the on-disk cache bundle size, which is the
//! dominant RSS component for the analyzer singleton. Not a direct RSS
//! measurement (Rust has no portable in-process RSS API short of
//! platform bindings we don't want as a prod dep), but a stable
//! lower-bound that tracks the in-memory footprint.
//!
//! # Why ignored by default
//!
//! The cold-start step deletes the real user cache file. That's fine
//! for an explicit `--ignored` run on a dev machine, but not something
//! `cargo test` should do silently on every commit — CI and local test
//! runs would then pay a rebuild cost the user didn't ask for.
//!
//! # Why a single monolithic test
//!
//! `OnceLock` inside `GenerativeFst::get()` means cold-start can only be
//! measured once per process. Splitting into multiple `#[test]` fns
//! would require fresh subprocesses — more machinery than this bench
//! warrants. One test, sequential measurements, report to stdout.

#[cfg(test)]
mod tests {
    use crate::arabic::fst_bake;
    use crate::arabic::fst_index::GenerativeFst;
    use crate::arabic::regression::{parse_corpus, raw_corpus, run_corpus};
    use crate::arabic::{analyze_best, analyze_with_overrides_best, overrides, AnalysisOrigin};
    use std::fs;
    use std::hint::black_box;
    use std::time::Instant;

    /// Emit a one-line "key: value" entry in the bench report.
    fn report(key: &str, value: impl std::fmt::Display) {
        println!("  {key:<24} {value}");
    }

    #[test]
    #[ignore]
    fn m9_bench() {
        println!("\n=== M9 Bench — Arabic Engine ===\n");

        // ── 1. Cold-start ────────────────────────────────────────────
        // Clear the on-disk cache so GenerativeFst::get() falls through
        // to build_bundle(). Best-effort: if the path resolver fails
        // (e.g. no HOME in a stripped sandbox), we skip the cold metric.
        if let Some(path) = fst_bake::cache_file_path() {
            let _ = fs::remove_file(&path);
        }
        let t_cold = Instant::now();
        let fst = GenerativeFst::get();
        let cold_ms = t_cold.elapsed().as_secs_f64() * 1000.0;
        let fst_keys = fst.len();
        report("FST keys", fst_keys);
        report("Cold-start (ms)", format!("{cold_ms:.2}"));

        // ── 2. Warm-start ────────────────────────────────────────────
        // get()'s OnceLock is now hot — we can't re-measure it. Instead,
        // run the SAME steps get() uses on warm: load_bundle + from_bytes.
        // This is the dominant cost of a warm launch; the OnceLock init
        // overhead itself is one acquire-release, negligible.
        let warm_result = fst_bake::cache_file_path().and_then(|path| {
            let t_warm = Instant::now();
            let bundle = match fst_bake::load_bundle(&path) {
                Ok(b) => b,
                Err(_) => return None,
            };
            let fst2 = GenerativeFst::from_bytes(
                bundle.stripped_bytes,
                bundle.values_stripped,
                bundle.folded_bytes,
                bundle.values_folded,
            )
            .ok()?;
            black_box(&fst2);
            Some(t_warm.elapsed().as_secs_f64() * 1000.0)
        });
        if let Some(warm_ms) = warm_result {
            report("Warm-start (ms)", format!("{warm_ms:.2}"));
            report("Cold/warm ratio", format!("{:.1}×", cold_ms / warm_ms));
        } else {
            report("Warm-start (ms)", "skipped (no cache path)");
        }

        // ── 3. Throughput ────────────────────────────────────────────
        // The 502-case corpus × K iterations = N calls through
        // analyze_best. Includes every layer (overrides → protected →
        // FST → cascade → heuristic) plus M7 disambiguation and the
        // M8b ACTIVE_STORE Arc clone per call.
        let cases = parse_corpus(raw_corpus());
        let surfaces: Vec<String> = cases.iter().map(|c| c.surface.clone()).collect();
        let corpus_n = surfaces.len();
        const K: usize = 500;
        let total_calls = K * corpus_n;

        // Warm-up: one full pass to ensure any lazy inits beyond the
        // FST singleton (e.g. protected_list::get) are paid before we
        // start the clock.
        for s in &surfaces {
            black_box(analyze_best(s));
        }

        let t_tput = Instant::now();
        for _ in 0..K {
            for s in &surfaces {
                black_box(analyze_best(s));
            }
        }
        let elapsed_s = t_tput.elapsed().as_secs_f64();
        let wps = total_calls as f64 / elapsed_s;
        let ns_per_call = elapsed_s * 1e9 / total_calls as f64;
        report("Corpus size", corpus_n);
        report("Iterations", K);
        report("Total calls", total_calls);
        report("Wall (s)", format!("{elapsed_s:.2}"));
        report("Throughput (words/s)", format!("{wps:.0}"));
        report("Per-call (ns)", format!("{ns_per_call:.0}"));

        // ── 3b. Throughput (FTS production path) ─────────────────────
        // M9-hotpath added `active_if_non_empty` specifically to cut
        // the per-token cost of the `overrides::active()` probe. The
        // "Throughput" measurement above calls `analyze_best` directly
        // and so doesn't exercise that probe at all — it's the raw
        // analyzer speed, useful for comparing across algorithmic
        // changes but not representative of what FTS5 tokenization
        // actually pays per token.
        //
        // This measurement mirrors `libraries::process_arabic_word`'s
        // production shape: fetch the active store via
        // `active_if_non_empty`, hand it to `analyze_with_overrides_best`.
        // When the bench runs with an empty active store (the default
        // in this harness — no `set_active` call), the probe returns
        // `None` via the fast path. The delta between "Throughput" and
        // "Throughput FTS" is the per-token active-store cost we've
        // been trying to drive down.
        //
        // Same warm-up discipline: one full pre-pass before the clock.
        for s in &surfaces {
            let store_owned = overrides::active_if_non_empty();
            let overrides_ref = store_owned.as_deref();
            black_box(analyze_with_overrides_best(s, overrides_ref));
        }

        let t_fts = Instant::now();
        for _ in 0..K {
            for s in &surfaces {
                let store_owned = overrides::active_if_non_empty();
                let overrides_ref = store_owned.as_deref();
                black_box(analyze_with_overrides_best(s, overrides_ref));
            }
        }
        let elapsed_fts = t_fts.elapsed().as_secs_f64();
        let wps_fts = total_calls as f64 / elapsed_fts;
        let ns_per_fts = elapsed_fts * 1e9 / total_calls as f64;
        let overhead_ns = ns_per_fts - ns_per_call;
        report("Throughput FTS (w/s)", format!("{wps_fts:.0}"));
        report("Per-call FTS (ns)", format!("{ns_per_fts:.0}"));
        report("FTS overhead (ns)", format!("{overhead_ns:+.0}"));

        // ── 4. Accuracy ──────────────────────────────────────────────
        let rpt = run_corpus();
        let rate = rpt.pass_rate() * 100.0;
        report("Pass rate (%)", format!("{rate:.1}"));
        report("Passed / total", format!("{} / {}", rpt.passed, rpt.total()));

        // Per-origin accuracy: walk the failures and bucket them.
        let mut per_origin: [(usize, usize); 4] = [(0, 0); 4]; // (passed, total)
        for case in parse_corpus(raw_corpus()) {
            let idx = match case.origin {
                AnalysisOrigin::ProtectedList => 0,
                AnalysisOrigin::GenerativeFst => 1,
                AnalysisOrigin::SurfaceHeuristic => 2,
                AnalysisOrigin::UserOverride => 3,
            };
            per_origin[idx].1 += 1;
        }
        for fail in &rpt.failed {
            let idx = match fail.case.origin {
                AnalysisOrigin::ProtectedList => 0,
                AnalysisOrigin::GenerativeFst => 1,
                AnalysisOrigin::SurfaceHeuristic => 2,
                AnalysisOrigin::UserOverride => 3,
            };
            // failures don't count as passes; reconstruct passed = total
            // - failed_in_this_bucket below after accumulating.
            per_origin[idx].0 += 1; // temporarily: count of failures
        }
        // Convert: passed = total - failures_collected
        for bucket in per_origin.iter_mut() {
            let fails = bucket.0;
            bucket.0 = bucket.1.saturating_sub(fails);
        }
        let names = ["Protected", "Generative", "Heuristic", "UserOverride"];
        for (i, (passed, total)) in per_origin.iter().enumerate() {
            if *total > 0 {
                let pct = (*passed as f64 / *total as f64) * 100.0;
                report(
                    &format!("  {} (%)", names[i]),
                    format!("{pct:.1} ({passed}/{total})"),
                );
            }
        }

        // ── 5. Size proxy ────────────────────────────────────────────
        if let Some(path) = fst_bake::cache_file_path() {
            if let Ok(meta) = fs::metadata(&path) {
                let kib = meta.len() as f64 / 1024.0;
                report("Cache bundle (KiB)", format!("{kib:.1}"));
                // Project to target 7K-root scale. Today's seed is ~595
                // roots; linear projection is approximate but useful.
                let ratio = 7000.0 / 595.0;
                let projected_mib = kib * ratio / 1024.0;
                report(
                    "Projected @ 7K (MiB)",
                    format!("{projected_mib:.1}"),
                );
            }
        }

        println!();
    }
}
