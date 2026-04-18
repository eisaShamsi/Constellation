//! M12-bench — `expand_to_match_expr` microbench.
//!
//! Run with:
//!
//! ```bash
//! cargo test --lib --release lexicon::bench -- --ignored --nocapture
//! ```
//!
//! # What this measures
//!
//! One `#[test] #[ignore]` function that exercises
//! [`crate::lexicon::expand_to_match_expr_via`] across a diverse query
//! bank — mixing Latin, Arabic, CJK, hits, misses, and both
//! `SynonymLevel` settings — then reports mean / p50 / p95 / p99 / max
//! per-call latency.
//!
//! # Why it matters
//!
//! M14 will wire this helper into `search.rs::lexical_search`, which
//! runs on every query keystroke (debounced). A sub-millisecond budget
//! keeps the expansion invisible to the user even on the slowest hit
//! path (CPU-bound graph walk + FST probe + MATCH builder).
//!
//! # Scale note
//!
//! The graph under test today is the M10 seed (~15 concepts × 15
//! languages). M11-data lands the 20K-concept × 15-language core.
//! Expansion per call walks **one concept's neighbours** (bounded at
//! ~15–30 nodes regardless of corpus size), so the per-call budget
//! should not change meaningfully — but this bench will be re-run
//! once M11-data ships to confirm, and the target threshold adjusted
//! if required.
//!
//! # Budget
//!
//! p99 < 1 ms (1,000,000 ns). Hard failure if exceeded so a regression
//! in the expansion hot path trips the bench on the next opt-in run.
//! Not gated on `cargo test --lib` — only on an explicit `--ignored`
//! invocation, matching `arabic::bench::m9_bench`.

#[cfg(test)]
mod tests {
    use crate::arabic::Lang;
    use crate::lexicon::{expand_to_match_expr_via, ExpansionOptions, LexiconGraph};
    use std::hint::black_box;
    use std::time::Instant;

    fn report(key: &str, value: impl std::fmt::Display) {
        println!("  {key:<28} {value}");
    }

    /// Percentile by rank on a sorted slice (no interpolation — the
    /// sample size is large enough that bucket choice dominates).
    fn percentile(sorted: &[u64], p: f64) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let idx = ((sorted.len() as f64) * p).clamp(0.0, (sorted.len() - 1) as f64) as usize;
        sorted[idx]
    }

    #[test]
    #[ignore]
    fn m12_bench() {
        println!("\n=== M12 Bench — lexicon::expand_to_match_expr ===\n");

        let graph = LexiconGraph::get();

        // Diverse query bank covering:
        //   - every concept in the M10 seed (hits);
        //   - multiple source languages for the same concept (graph
        //     walks starting from different nodes);
        //   - two unknown lemmas in two scripts (miss path — source
        //     echo only);
        //   - two empty / whitespace strings (short-circuit path).
        //
        // Results are mean/percentile across the **union** of all paths
        // — the typical search bar sees a similar mix across a session.
        let queries: Vec<(&str, Lang)> = vec![
            // English seeds (hit path through the FST name index).
            ("book", Lang::En),
            ("knowledge", Lang::En),
            ("read", Lang::En),
            ("write", Lang::En),
            ("house", Lang::En),
            ("water", Lang::En),
            ("love", Lang::En),
            ("time", Lang::En),
            ("peace", Lang::En),
            ("truth", Lang::En),
            ("teacher", Lang::En),
            ("student", Lang::En),
            // Arabic seeds (exercises the RTL normalisation path).
            ("كتاب", Lang::Ar),
            ("معرفة", Lang::Ar),
            ("قرأ", Lang::Ar),
            ("ماء", Lang::Ar),
            // Other-language seeds (exercise graph walks starting from
            // non-English nodes).
            ("livre", Lang::Fr),
            ("Wissen", Lang::De),
            ("libro", Lang::Es),
            // Miss path — source-echo only, no graph walk succeeds.
            ("xyzzy", Lang::En),
            ("لا_موجود", Lang::Ar),
            // Short-circuit path — empty / whitespace returns None.
            ("", Lang::En),
            ("    ", Lang::En),
        ];

        // Two option shapes — default (all 15 langs, synonyms on) and
        // mono-mode (one lang, synonyms off — the rollback / "🌐 off"
        // toggle). Both paths must stay in budget.
        let opts_default = ExpansionOptions::default();
        let opts_mono_en = ExpansionOptions::mono(Lang::En);

        // Warm up: one pass per option shape so any lazy allocations
        // inside the graph / FST / ExpansionResult are paid before
        // sampling.
        for (q, lang) in &queries {
            black_box(expand_to_match_expr_via(graph, q, *lang, &opts_default));
            black_box(expand_to_match_expr_via(graph, q, *lang, &opts_mono_en));
        }

        const ITERATIONS: usize = 1_000;
        let total_calls = queries.len() * ITERATIONS * 2; // two option shapes
        let mut samples: Vec<u64> = Vec::with_capacity(total_calls);

        for _ in 0..ITERATIONS {
            for (q, lang) in &queries {
                // Default options (cross-lang + synonyms).
                let t0 = Instant::now();
                let r = expand_to_match_expr_via(graph, q, *lang, &opts_default);
                samples.push(t0.elapsed().as_nanos() as u64);
                black_box(r);

                // Mono options (rollback path).
                let t1 = Instant::now();
                let r = expand_to_match_expr_via(graph, q, *lang, &opts_mono_en);
                samples.push(t1.elapsed().as_nanos() as u64);
                black_box(r);
            }
        }

        samples.sort_unstable();

        let sum: u128 = samples.iter().map(|&n| n as u128).sum();
        let mean_ns = (sum / samples.len() as u128) as u64;
        let p50 = percentile(&samples, 0.50);
        let p95 = percentile(&samples, 0.95);
        let p99 = percentile(&samples, 0.99);
        let max = *samples.last().unwrap();

        report("Query bank size", queries.len());
        report("Iterations per query", ITERATIONS);
        report("Option shapes", 2);
        report("Total samples", samples.len());
        report("Mean (ns)", mean_ns);
        report("p50 (ns)", p50);
        report("p95 (ns)", p95);
        report("p99 (ns)", p99);
        report("Max (ns)", max);
        report("Mean (µs)", format!("{:.2}", mean_ns as f64 / 1_000.0));
        report("p99 (µs)", format!("{:.2}", p99 as f64 / 1_000.0));

        // Budget check — p99 must stay under 1 ms at M10-seed scale. If
        // this trips at M11-data scale the budget gets revisited with
        // the new numbers; for now it's the guardrail against a
        // regression that reintroduces per-call allocation or graph
        // full-scan.
        assert!(
            p99 < 1_000_000,
            "M12 p99 exceeded 1ms budget: {} ns (mean {} ns)",
            p99,
            mean_ns
        );
    }
}
